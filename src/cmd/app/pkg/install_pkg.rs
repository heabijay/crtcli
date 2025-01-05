use crate::app::{CrtClient, CrtClientGenericError, InstallLogWatcher, InstallLogWatcherEvent};
use crate::cmd::app::restart::print_app_restart_requested;
use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Args;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct InstallPkgCommand {
    /// Path to the package archive file
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
    SqlBeforePackage(#[source] CrtClientGenericError),

    #[error("package descriptor.json was found, but the package uid value is null")]
    PackageUidValueNull,

    #[error("failed to upload package: {0}")]
    Upload(#[source] CrtClientGenericError),

    #[error("failed to install package: {0}")]
    Install(#[source] CrtClientGenericError),

    #[error("failed to restart app: {0}")]
    AppRestart(#[source] CrtClientGenericError),
}

impl AppCommand for InstallPkgCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let client = Arc::new(app.build_client()?);

        install_package_from_stream_command(
            client,
            File::open(&self.filepath)?,
            self.filepath
                .file_name()
                .ok_or("unable to get filename of specified path")?
                .to_str()
                .ok_or("unable to get filename str of specified path")?,
            &self.install_pkg_options,
        )?;

        Ok(())
    }
}

pub fn install_package_from_stream_command(
    client: Arc<CrtClient>,
    mut package_reader: impl Read + Send + Seek + 'static,
    package_name: &str,
    options: &InstallPkgCommandOptions,
) -> Result<(), InstallPkgCommandError> {
    let descriptors =
        crate::pkg::utils::get_package_descriptors_from_package_reader(&mut package_reader)
            .map_err(InstallPkgCommandError::ReadDescriptor)?;

    apply_options_before_install(&client, options, &descriptors)?;

    client
        .package_installer_service()
        .upload_package(package_reader, package_name.to_owned())
        .map_err(InstallPkgCommandError::Upload)?;

    let log_watcher = (!options.disable_install_log_pooling).then(|| {
        InstallLogWatcher::new(Arc::clone(&client))
            .with_handler(|event| match event {
                InstallLogWatcherEvent::Clear() => {}
                InstallLogWatcherEvent::Append(text) => print!("{text}"),
            })
            .fetch_last_log_on_stop(true)
            .start()
    });

    let install_result = client
        .package_installer_service()
        .install_package(package_name)
        .map_err(InstallPkgCommandError::Install);

    if let Some(log_watcher) = log_watcher {
        log_watcher.stop();
        log_watcher.wait_next_check_complete();
    }

    install_result?;

    if options.restart {
        client
            .app_installer_service()
            .restart_app()
            .map_err(InstallPkgCommandError::AppRestart)?;

        print_app_restart_requested(&client);
    }

    return Ok(());

    fn apply_options_before_install(
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
