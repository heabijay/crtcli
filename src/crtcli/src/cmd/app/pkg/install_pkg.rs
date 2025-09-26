use crate::app::{CrtClient, CrtClientError, InstallLogWatcherBuilder, InstallLogWatcherEvent};
use crate::cmd::app::AppCommand;
use crate::cmd::app::restart::print_app_restart_requested;
use crate::cmd::cli::{CommandDynError, CommandResult};
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::Args;
use std::io::{Cursor, Read, Seek, stdin};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

#[derive(Args, Debug)]
pub struct InstallPkgCommand {
    /// Path to the package archive file (Use '@-' or '-' value to read data from stdin)
    #[arg(value_hint = clap::ValueHint::FilePath)]
    filepath: PathBuf,

    #[command(flatten)]
    install_pkg_options: InstallPkgCommandOptions,
}

#[derive(Debug, Default, Args)]
pub struct InstallPkgCommandOptions {
    /// Restart the Creatio application after successful installation
    #[arg(short, long)]
    restart: bool,

    /// Compile the package in Creatio after successful installation
    #[arg(short, long)]
    compile_package: bool,

    /// Overrides changed schemas in the database: executes SQL to mark package schemas as not changed before installation
    #[arg(short, long)]
    force: bool,

    /// Same as -f but also clears localization data
    #[arg(short = 'F', long)]
    force_and_clear_localizations: bool,

    /// Clears existing schema content and checksums before installation
    #[arg(long)]
    clear_schemas_content: bool,

    /// Disables the display of the installation log
    #[arg(long)]
    disable_install_log_pooling: bool,
}

#[derive(Debug, Error)]
pub enum InstallPkgCommandError {
    #[error("failed to read package descriptors: {0}")]
    ReadDescriptor(#[from] crate::pkg::utils::GetPackageDescriptorFromReaderError),

    #[error("failed to apply SQL options before package install: {0}")]
    SqlBeforePackage(#[source] CrtClientError),

    #[error("package descriptor.json was found, but the package uid value is null")]
    PackageUidValueNull,

    #[error("failed to upload package: {0}")]
    Upload(#[source] CrtClientError),

    #[error("failed to install package: {0}")]
    Install(#[source] CrtClientError),

    #[error("failed to compile package: {0}")]
    PkgCompile(#[source] CommandDynError),

    #[error("failed to restart app: {0}")]
    AppRestart(#[source] CrtClientError),
}

#[async_trait]
impl AppCommand for InstallPkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let (package_content, package_name) = match &self.filepath.to_str() {
            Some("@-") | Some("-") => {
                let mut data = vec![];

                stdin().read_to_end(&mut data)?;

                let mut reader = Cursor::new(data);
                let filename = get_filename_for_package_reader(&mut reader)?;

                (reader.into_inner(), filename)
            }
            _ => (
                std::fs::read(&self.filepath)?,
                self.filepath
                    .file_name()
                    .ok_or("unable to get filename of specified path")?
                    .to_str()
                    .ok_or("unable to get filename str of specified path")?
                    .to_string(),
            ),
        };

        install_package_from_stream_command(
            client,
            Cursor::new(package_content),
            &package_name,
            &self.install_pkg_options,
        )
        .await?;

        Ok(())
    }
}

fn get_filename_for_package_reader(
    mut reader: impl Read + Seek,
) -> Result<String, CommandDynError> {
    let extension = if crate::pkg::utils::is_gzip_stream(&mut reader)? {
        ".gz"
    } else {
        ".zip"
    };

    let descriptors = crate::pkg::utils::get_package_descriptors_from_package_reader(&mut reader)?;

    if descriptors.len() == 1 {
        Ok(format!(
            "{name}{extension}",
            name = descriptors[0].name().unwrap_or("Package"),
        ))
    } else {
        Ok(format!("Packages{extension}"))
    }
}

pub async fn install_package_from_stream_command<R>(
    client: Arc<CrtClient>,
    mut package_reader: R,
    package_name: &str,
    options: &InstallPkgCommandOptions,
) -> Result<(), InstallPkgCommandError>
where
    R: AsyncReadExt + AsyncSeekExt + std::io::Read + std::io::Seek + Send + Sync + Unpin + 'static,
{
    let descriptors =
        crate::pkg::utils::get_package_descriptors_from_package_reader(&mut package_reader)
            .map_err(InstallPkgCommandError::ReadDescriptor)?;

    apply_options_before_install(&client, options, &descriptors).await?;

    let progress = spinner_precise!(
        "Installing {bold}{package_name}{bold:#} package archive at {bold}{url}{bold:#}",
        bold = Style::new().bold(),
        url = client.base_url()
    );

    let progress = Arc::new(progress);

    client
        .package_installer_service()
        .upload_package(package_reader, package_name.to_owned())
        .await
        .map_err(InstallPkgCommandError::Upload)?;

    let log_watcher = (!options.disable_install_log_pooling).then(|| {
        let progress_clone = Arc::clone(&progress);

        // Sometimes, Creatio based on .NET Framework (IIS) does not allow retrieval of the installation log in real-time.
        // Instead, it appears that Creatio blocks the log request until package installation is finished.
        // The reason for this is unknown, but hopefully, this could help.
        if client.is_net_framework() {
            InstallLogWatcherBuilder::new_with_new_session(&client).unwrap()
        } else {
            InstallLogWatcherBuilder::new(Arc::clone(&client))
        }
        .fetch_last_log_on_stop(true)
        .start(move |event| match event {
            InstallLogWatcherEvent::Clear => {}
            InstallLogWatcherEvent::Append(text) => {
                progress_clone.suspend(move || print!("{}", text))
            }
        })
    });

    let install_result = client
        .package_installer_service()
        .install_package(package_name)
        .await
        .map_err(InstallPkgCommandError::Install);

    if let Some(log_watcher) = log_watcher {
        log_watcher.stop();
        log_watcher.wait_until_stopped().await;
    }

    progress.finish_with_message(
        match install_result {
            Ok(_) => format!(
                "{green}Package archive {green_bold}{package_name}{green_bold:#}{green} successfully installed at {green_bold}{url}{green_bold:#}",
                green=Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
                green_bold=Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))).bold(),
                url=client.base_url()
            ),
            Err(_) => format!(
                "{red}Package archive {red_bold}{package_name}{red_bold:#}{red} installation failed at {red_bold}{url}{red_bold:#}",
                red=Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))),
                red_bold=Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))).bold(),
                url=client.base_url()
            ),
        }
    );

    install_result?;

    if options.compile_package {
        match descriptors.len() {
            0 => {}
            1 if descriptors.first().unwrap().name().is_some() => {
                crate::cmd::app::pkg::compile_pkg::CompilePkgCommand {
                    package_name: Some(descriptors.first().unwrap().name().unwrap().to_owned()),
                    force_rebuild: false,
                    restart: options.restart,
                }
                .run(client)
                .await
                .map_err(InstallPkgCommandError::PkgCompile)?
            }
            _ => crate::cmd::app::compile::CompileCommand {
                restart: options.restart,
                force_rebuild: false,
            }
            .run(client)
            .await
            .map_err(InstallPkgCommandError::PkgCompile)?,
        }
    } else if options.restart {
        client
            .app_installer_service()
            .restart_app()
            .await
            .map_err(InstallPkgCommandError::AppRestart)?;

        print_app_restart_requested(&client);
    }

    return Ok(());

    async fn apply_options_before_install(
        client: &Arc<CrtClient>,
        options: &InstallPkgCommandOptions,
        descriptors: &Vec<crate::pkg::json_wrappers::PkgPackageDescriptorJsonWrapper>,
    ) -> Result<(), InstallPkgCommandError> {
        if options.force || options.force_and_clear_localizations {
            for descriptor in descriptors {
                let rows_affected = client
                    .sql_scripts()
                    .mark_package_as_not_changed(
                        descriptor
                            .uid()
                            .ok_or(InstallPkgCommandError::PackageUidValueNull)?,
                    )
                    .await
                    .map_err(InstallPkgCommandError::SqlBeforePackage)?;

                eprintln!(
                    "Package content {} has been marked as not changed, affected {} rows",
                    descriptor.name().unwrap_or("_"),
                    rows_affected
                );
            }
        }

        if options.force_and_clear_localizations {
            for descriptor in descriptors {
                let rows_affected = client
                    .sql_scripts()
                    .delete_package_localizations(
                        descriptor
                            .uid()
                            .ok_or(InstallPkgCommandError::PackageUidValueNull)?,
                    )
                    .await
                    .map_err(InstallPkgCommandError::SqlBeforePackage)?;

                eprintln!(
                    "Package localizations {} has been deleted, affected {} rows",
                    descriptor.name().unwrap_or("_"),
                    rows_affected
                );
            }
        }

        if options.clear_schemas_content {
            for descriptor in descriptors {
                let rows_affected = client
                    .sql_scripts()
                    .reset_schema_content(
                        descriptor
                            .uid()
                            .ok_or(InstallPkgCommandError::PackageUidValueNull)?,
                    )
                    .await
                    .map_err(InstallPkgCommandError::SqlBeforePackage)?;

                eprintln!(
                    "Schema content has been reset for package {}, affected {} rows",
                    descriptor.name().unwrap_or("_"),
                    rows_affected
                );
            }
        }

        Ok(())
    }
}
