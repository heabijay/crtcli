use crate::app::{CrtClient, CrtClientError, InstallLogWatcherBuilder, InstallLogWatcherEvent};
use crate::cmd::app::AppCommand;
use crate::cmd::cli::{CommandDynError, CommandResult};
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::Args;
use std::io::{Cursor, Read, Seek, Write, stdin};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use zip::result::ZipError;
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{ZipArchive, ZipWriter};

#[derive(Args, Debug)]
pub struct InstallPkgCommand {
    /// Paths to the package archive files (Use single '@-' or '-' value to read data from stdin)
    #[arg(required = true, value_hint = clap::ValueHint::FilePath)]
    filepaths: Vec<PathBuf>,

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

    /// Disables the display of the installation log updates in real-time
    #[arg(long)]
    disable_install_log_polling: bool,
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
    AppRestart(#[source] CommandDynError),
}

#[derive(Debug, Error)]
enum BeforeInstallPkgCombineError {
    #[error("failed to process {0} file while combining packages into single package archive: {1}")]
    ProcessFile(PathBuf, CommandDynError),

    #[error("failed to format zip archive for combined packages: {0}")]
    Zip(#[from] ZipError),
}

#[async_trait]
impl AppCommand for InstallPkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let (package_content, package_name) = if self.filepaths.len() == 1 {
            let filepath = &self.filepaths[0];

            if Some("@-") == filepath.to_str() || Some("@-") == filepath.to_str() {
                read_package_input_from_stdin()?
            } else {
                (std::fs::read(filepath)?, path_to_filename_str(filepath)?)
            }
        } else {
            (
                combine_packages_to_single_zip(&self.filepaths)?,
                "Packages.zip".to_owned(),
            )
        };

        install_package_from_stream_command(
            client,
            Cursor::new(package_content),
            &package_name,
            &self.install_pkg_options,
        )
        .await?;

        return Ok(());

        fn read_package_input_from_stdin() -> Result<(Vec<u8>, String), CommandDynError> {
            let mut data = vec![];

            stdin().read_to_end(&mut data)?;

            let mut reader = Cursor::new(data);
            let filename = get_filename_for_package_reader(&mut reader)?;

            Ok((reader.into_inner(), filename))
        }

        fn combine_packages_to_single_zip(
            filepaths: &[impl AsRef<Path>],
        ) -> Result<Vec<u8>, BeforeInstallPkgCombineError> {
            let mut zip = ZipWriter::new(Cursor::new(vec![]));
            let file_options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

            for filepath in filepaths {
                process_file_combine(&mut zip, file_options, filepath).map_err(|e| {
                    BeforeInstallPkgCombineError::ProcessFile(filepath.as_ref().to_path_buf(), e)
                })?;
            }

            return Ok(zip.finish()?.into_inner());

            fn process_file_combine(
                mut zip: &mut ZipWriter<impl Write + Seek>,
                file_options: FileOptions<()>,
                filepath: &impl AsRef<Path>,
            ) -> Result<(), CommandDynError> {
                let mut file = std::fs::File::open(filepath)?;

                if crate::pkg::utils::is_gzip_stream(&mut file)? {
                    zip.start_file(path_to_filename_str(filepath.as_ref())?, file_options)?;

                    std::io::copy(&mut file, &mut zip)?;
                } else {
                    let mut zip_inner = ZipArchive::new(file)?;

                    for i in 0..zip_inner.len() {
                        let mut file = zip_inner.by_index(i)?;

                        zip.start_file(file.name(), file_options)?;

                        std::io::copy(&mut file, &mut zip)?;
                    }
                }

                Ok(())
            }
        }

        fn path_to_filename_str(path: &Path) -> Result<String, CommandDynError> {
            Ok(path
                .file_name()
                .ok_or("unable to get filename of specified path")?
                .to_str()
                .ok_or("unable to get filename str of specified path")?
                .to_string())
        }
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
    R: AsyncReadExt + AsyncSeekExt + Read + Seek + Send + Sync + Unpin + 'static,
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

    let log_watcher = (!options.disable_install_log_polling).then(|| {
        let progress_clone = Arc::clone(&progress);

        InstallLogWatcherBuilder::new(Arc::clone(&client))
            .fetch_last_log_on_stop(true)
            .start(move |event| match event {
                InstallLogWatcherEvent::Clear => {}
                InstallLogWatcherEvent::Append(text) => {
                    progress_clone.suspend(move || print!("{}", text))
                }
                InstallLogWatcherEvent::FetchError(error) => {
                    progress_clone.suspend(move || {
                        eprintln!(
                            "{style}warning (log polling): {error}{style:#}",
                            error = error,
                            style = Style::new()
                                .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                                .dimmed()
                        )
                    });
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

        progress.set_message(
            match install_result {
                Ok(_) => format!(
                    "{green}Package archive {green_bold}{package_name}{green_bold:#}{green} successfully installed at {green_bold}{url}{green_bold:#}. Trying to get final logs...",
                    green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
                    green_bold = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))).bold(),
                    url = client.base_url()
                ),
                Err(_) => format!(
                    "{red}Package archive {red_bold}{package_name}{red_bold:#}{red} installation failed at {red_bold}{url}{red_bold:#}. Trying to get final logs...",
                    red = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))),
                    red_bold = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))).bold(),
                    url = client.base_url()
                ),
            }
        );

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
        crate::cmd::app::pkg::compile_pkg::CompilePkgCommand {
            packages_names: descriptors
                .iter()
                .filter_map(|d| d.value.as_str().map(|s| s.to_owned()))
                .collect(),
            force_rebuild: false,
            restart: options.restart,
        }
        .run(client)
        .await
        .map_err(InstallPkgCommandError::PkgCompile)?
    } else if options.restart {
        crate::cmd::app::restart::RestartCommand
            .run(client)
            .await
            .map_err(InstallPkgCommandError::AppRestart)?
    }

    return Ok(());

    async fn apply_options_before_install(
        client: &Arc<CrtClient>,
        options: &InstallPkgCommandOptions,
        descriptors: &Vec<crate::pkg::json::PkgPackageDescriptorJsonWrapper>,
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
                    .clear_schema_content(
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
