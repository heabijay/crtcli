use crate::app::{CrtClient, CrtClientError};
use crate::cmd::app::AppCommand;
use crate::cmd::app::pkg::DetectTargetPackageNameError;
use crate::cmd::cli::CommandResult;
use crate::cmd::pkg::config_file::{CrtCliPkgConfig, combine_apply_features_from_args_and_config};
use crate::pkg::bundling::extractor::*;
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::Args;
use clap::builder::{ValueParser, ValueParserFactory};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::AsyncReadExt;

#[derive(Args, Debug)]
pub struct PullPkgCommand {
    /// Packages to pull and their destination folders (comma-separated `PackageName:DestinationFolder` pairs) (default: package name in ./descriptor.json of current folder)
    ///
    /// Examples:
    /// `crtcli app pkg pull` (Pulls package from `./descriptor.json` to current dir)
    /// `crtcli app pkg pull UsrPackage` (Pulls `UsrPackage` to current dir)
    /// `crtcli app pkg pull UsrPackage:Src,UsrPackage2:Src2` (Pulls `UsrPackage` to `./Src`, `UsrPackage2` to `./Src2`)
    /// `crtcli app pkg pull :Src` (Pulls package from `./Src/descriptor.json` to `./Src`)
    #[arg(value_name = "PACKAGE:DESTINATION", value_delimiter = ',', value_hint = clap::ValueHint::DirPath)]
    #[clap(verbatim_doc_comment)]
    packages_map: Vec<PackageDestinationArg>,

    #[command(flatten)]
    apply_features: Option<crate::cmd::pkg::PkgApplyFeatures>,
}

#[derive(Debug, Error)]
pub enum PullPkgCommandError {
    #[error("{0}")]
    DetectPackageName(#[from] DetectTargetPackageNameError),

    #[error("cannot download package from remote: {0}")]
    DownloadPackage(#[from] CrtClientError),

    #[error("cannot unpack package: {0}")]
    ExtractPackage(#[from] ExtractSingleZipPackageError),
}

#[derive(Debug, Clone)]
struct PackageDestinationArg {
    package_name: String,
    destination_folder: PathBuf,
}

#[derive(Error, Debug)]
enum HeaderArgParsingError {
    #[error("value cannot be empty, use \"PackageName:DestinationFolder\" format")]
    EmptyValue,

    #[error("{0}")]
    DetectPackageName(#[from] DetectTargetPackageNameError),
}

impl TryFrom<&str> for PackageDestinationArg {
    type Error = HeaderArgParsingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(HeaderArgParsingError::EmptyValue);
        }

        let (package_name, destination_folder) = value
            .split_once(":")
            .map(|(package_name, destination_folder)| {
                (package_name.trim(), destination_folder.trim())
            })
            .unwrap_or((value, ""));

        let package_name = if package_name.is_empty() {
            None
        } else {
            Some(package_name.to_owned())
        };

        let destination_folder = destination_folder
            .is_empty()
            .then_some(PathBuf::from("."))
            .unwrap_or_else(|| PathBuf::from(destination_folder));

        let package_name = detect_target_package_name!(package_name, &destination_folder);

        Ok(Self {
            package_name: package_name.to_string(),
            destination_folder,
        })
    }
}

impl ValueParserFactory for PackageDestinationArg {
    type Parser = ValueParser;

    fn value_parser() -> Self::Parser {
        ValueParser::new(|s: &str| PackageDestinationArg::try_from(s))
    }
}

#[async_trait]
impl AppCommand for PullPkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let current_folder = PathBuf::from(".");

        let packages_map: &[_] = if self.packages_map.is_empty() {
            &[PackageDestinationArg {
                package_name: detect_target_package_name!(),
                destination_folder: current_folder.clone(),
            }]
        } else {
            &self.packages_map
        };

        if packages_map
            .iter()
            .filter(|p| p.destination_folder == current_folder)
            .count()
            > 1
        {
            return Err("destination folders expected to be unique for each package".into());
        }

        let packages_str = packages_map
            .iter()
            .map(|p| p.package_name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        let progress = spinner!(
            "Pulling {bold}{packages_str}{bold:#} package{packages_suffix} from {bold}{url}{bold:#}",
            bold = Style::new().bold(),
            url = client.base_url(),
            packages_suffix = if packages_map.len() > 1 { "s" } else { "" }
        );

        let mut packages = client
            .package_installer_service()
            .get_zip_packages(
                packages_map
                    .iter()
                    .map(|p| p.package_name.as_str())
                    .collect::<Vec<_>>(),
            )
            .await
            .map_err(PullPkgCommandError::DownloadPackage)?;

        let mut package_data = vec![];

        packages.read_to_end(&mut package_data).await?;

        progress.finish_and_clear();

        for package_map in packages_map {
            let pkg_config = CrtCliPkgConfig::from_package_folder(&package_map.destination_folder)?;

            let apply_features = combine_apply_features_from_args_and_config(
                self.apply_features.as_ref(),
                pkg_config.as_ref(),
            )
            .unwrap_or_default();

            let extract_config = PackageToFolderExtractorConfig::default()
                .with_files_already_exists_in_folder_strategy(
                    FilesAlreadyExistsInFolderStrategy::SmartMerge,
                )
                .print_merge_log(true)
                .with_converter(apply_features.build_combined_converter());

            extract_single_zip_package_to_folder(
                std::io::Cursor::new(&package_data),
                &package_map.destination_folder,
                Some(&package_map.package_name),
                &extract_config,
            )
            .map_err(PullPkgCommandError::ExtractPackage)?;
        }

        eprintln!(
            "{green}✔ Package{packages_suffix} {green_bold}{packages_str}{green_bold:#}{green} successfully pulled from {green_bold}{url}{green_bold:#}{green}!{green:#}",
            green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            green_bold = Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::Green)))
                .bold(),
            packages_suffix = if packages_map.len() > 1 { "s" } else { "" },
            url = client.base_url(),
        );

        Ok(())
    }
}
