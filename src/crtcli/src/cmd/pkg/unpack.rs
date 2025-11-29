use crate::cfg::PkgConfig;
use crate::cfg::package::combine_apply_config_from_args_and_config;
use crate::cmd::cli::{CliCommand, CommandResult};
use crate::pkg::bundling::extractor::*;
use crate::pkg::transforms::post::PkgFolderPostTransform;
use anstream::stderr;
use clap::Args;
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;
use thiserror::Error;
use zip::result::ZipError;

#[derive(Debug, Args)]
pub struct UnpackCommand {
    /// Path to the package archive file
    #[arg(value_hint = clap::ValueHint::FilePath)]
    package_filepath: PathBuf,

    /// Destination folder where the extracted package files will be saved
    #[arg(short, long = "destination", value_hint = clap::ValueHint::DirPath)]
    destination_folder: Option<PathBuf>,

    /// If the archive is a zip file containing multiple packages, specify the name of the package to extract.
    #[arg(short, long = "package", value_hint = clap::ValueHint::Other)]
    package_name: Option<String>,

    /// If destination folder is not empty, attempt to merge package files
    #[arg(short, long)]
    merge: bool,

    // TODO
    #[arg(long)]
    smart_merge: bool,

    #[command(flatten)]
    apply_features: Option<crate::pkg::transforms::PkgApplyFeatures>,

    #[command(flatten)]
    apply_post_features: Option<crate::pkg::transforms::post::PkgApplyPostFeatures>,
}

#[derive(Error, Debug)]
enum UnpackCommandError {
    #[error("invalid package filename in path")]
    InvalidPackageFilename,

    #[error("failed to read the package file: {0}")]
    ReadPackageFile(#[from] std::io::Error),

    #[error("failed to extract as single zip package: {0}")]
    ExtractAsSingleZipPackage(#[from] ExtractSingleZipPackageError),

    #[error("failed to extract as gzip package: {0}")]
    ExtractAsGzipPackage(#[from] ExtractGzipPackageError),

    #[error(
        "failed to extract as single zip package: {single_zip_package_error} and failed to extract as gzip package: {gzip_package_error}"
    )]
    ExtractAsSingleZipOrGzipPackage {
        single_zip_package_error: ExtractSingleZipPackageError,

        gzip_package_error: ExtractGzipPackageError,
    },

    #[error(
        "multiple files found in zip package bundle, please specify file using -p argument or use 'unpack-all' command"
    )]
    MultipleFilesInZipPackage,
}

impl CliCommand for UnpackCommand {
    fn run(self) -> CommandResult {
        let destination_folder = match self.destination_folder {
            Some(folder) => folder,
            None => {
                let filename = self
                    .package_filepath
                    .file_stem()
                    .ok_or(UnpackCommandError::InvalidPackageFilename)?
                    .to_str()
                    .ok_or(UnpackCommandError::InvalidPackageFilename)?;

                crate::cmd::utils::get_next_filename_if_exists(PathBuf::from(".").join(filename))
            }
        };

        let pkg_config = PkgConfig::from_package_folder(&destination_folder)?;

        let apply_config = combine_apply_config_from_args_and_config(
            (
                self.apply_features.as_ref(),
                self.apply_post_features.as_ref(),
            ),
            pkg_config.as_ref().map(|x| x.apply()),
        )
        .unwrap_or_default();

        let smart_merge = self.smart_merge
            || pkg_config
                .and_then(|x| x.unpack().smart_merge())
                .unwrap_or_default();

        let extractor_config = PackageToFolderExtractorConfig::default()
            .with_files_already_exists_in_folder_strategy(match (self.merge, smart_merge) {
                (_, true) => FilesAlreadyExistsInFolderStrategy::SmartMerge,
                (true, false) => FilesAlreadyExistsInFolderStrategy::Merge,
                _ => FilesAlreadyExistsInFolderStrategy::ThrowError,
            })
            .with_transform(apply_config.apply().build_combined_transform());

        let mut file = std::fs::File::open(self.package_filepath)
            .map_err(UnpackCommandError::ReadPackageFile)?;

        let zip_result = extract_single_zip_package_to_folder(
            &file,
            &destination_folder,
            self.package_name.as_deref(),
            &extractor_config,
        );

        let gzip_result: Option<_> = match &zip_result {
            Err(ExtractSingleZipPackageError::MultiplePackageInZipFile) => {
                return Err(UnpackCommandError::MultipleFilesInZipPackage.into());
            }
            Err(ExtractSingleZipPackageError::OpenZipFileForReading(ZipError::InvalidArchive(
                _,
            ))) => {
                file.seek(SeekFrom::Start(0))
                    .map_err(UnpackCommandError::ReadPackageFile)?;

                let gzip_result =
                    extract_gzip_package_to_folder(file, &destination_folder, &extractor_config);

                Some(gzip_result)
            }
            _ => None,
        };

        let is_any_success =
            zip_result.is_ok() || gzip_result.as_ref().map(|x| x.is_ok()).unwrap_or(false);

        if !is_any_success {
            return if let Some(gzip_result) = gzip_result {
                Err(UnpackCommandError::ExtractAsSingleZipOrGzipPackage {
                    single_zip_package_error: zip_result.unwrap_err(),
                    gzip_package_error: gzip_result.unwrap_err(),
                }
                .into())
            } else {
                Err(UnpackCommandError::ExtractAsSingleZipPackage(zip_result.unwrap_err()).into())
            };
        }

        apply_config
            .apply_post()
            .build_combined_transform()
            .transform(&destination_folder, false, stderr())?;

        println!("{}", destination_folder.display());

        Ok(())
    }
}
