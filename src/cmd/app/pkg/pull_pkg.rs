use crate::app::{CrtClient, CrtClientGenericError};
use crate::cmd::app::pkg::DetectTargetPackageNameError;
use crate::cmd::app::AppCommand;
use crate::cmd::pkg::config_file::{combine_apply_features_from_args_and_config, CrtCliPkgConfig};
use crate::pkg::bundling::extractor::*;
use anstyle::{AnsiColor, Color, Style};
use clap::Args;
use std::error::Error;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct PullPkgCommand {
    /// Package name to pull (default: package name in ./descriptor.json of destination folder)
    #[arg(short, long = "package", value_hint = clap::ValueHint::Other)]
    package_name: Option<String>,

    /// Destination folder where package will be unpacked (default: current directory)
    #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
    destination_folder: Option<PathBuf>,

    #[command(flatten)]
    apply_features: Option<crate::cmd::pkg::PkgApplyFeatures>,
}

#[derive(Debug, Error)]
pub enum PullPkgCommandError {
    #[error("{0}")]
    DetectPackageName(#[from] DetectTargetPackageNameError),

    #[error("cannot download package from remote: {0}")]
    DownloadPackage(#[from] CrtClientGenericError),

    #[error("cannot unpack package: {0}")]
    ExtractPackage(#[from] ExtractSingleZipPackageError),
}

impl AppCommand for PullPkgCommand {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        let destination_folder = match &self.destination_folder {
            Some(destination_folder) => destination_folder,
            None => &PathBuf::from("."),
        };

        let pkg_config = CrtCliPkgConfig::from_package_folder(destination_folder)?;

        let apply_features = combine_apply_features_from_args_and_config(
            self.apply_features.as_ref(),
            pkg_config.as_ref(),
        )
        .unwrap_or_default();

        let package_name = detect_target_package_name!(self.package_name, &destination_folder);

        let progress = spinner!(
            "Pulling {bold}{package_name}{bold:#} package from {bold}{url}{bold:#}",
            bold = Style::new().bold(),
            url = client.base_url()
        );

        let mut package = client
            .package_installer_service()
            .get_zip_packages([package_name])
            .map_err(PullPkgCommandError::DownloadPackage)?;

        let mut package_data = vec![];

        package.read_to_end(&mut package_data)?;

        progress.finish_and_clear();

        let extract_config = PackageToFolderExtractorConfig::default()
            .with_files_already_exists_in_folder_strategy(
                FilesAlreadyExistsInFolderStrategy::SmartMerge,
            )
            .print_merge_log(true)
            .with_converter(apply_features.build_combined_converter());

        extract_single_zip_package_to_folder(
            std::io::Cursor::new(package_data),
            destination_folder,
            Some(package_name),
            &extract_config,
        )
        .map_err(PullPkgCommandError::ExtractPackage)?;

        eprintln!(
            "{green}✔ Package {green_bold}{package_name}{green_bold:#}{green} successfully pulled from {green_bold}{url}{green_bold:#}{green}!{green:#}",
            green=Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            green_bold=Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))).bold(),
            url=client.base_url(),
        );

        Ok(())
    }
}
