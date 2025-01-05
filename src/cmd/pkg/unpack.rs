use crate::cmd::cli::CliCommand;
use crate::cmd::pkg::apply::PkgApplyFeatures;
use crate::cmd::pkg::config_file::{combine_apply_features_from_args_and_config, CrtCliPkgConfig};
use crate::pkg::bundling::extractor::*;
use clap::Args;
use std::error::Error;
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
    #[arg(value_hint = clap::ValueHint::DirPath)]
    destination_folder: Option<PathBuf>,

    /// If the archive is a zip file containing multiple packages, specify the name of the package to extract.
    #[arg(short, long = "package", value_hint = clap::ValueHint::Other)]
    package_name: Option<String>,

    /// If destination folder is not empty, attempt to merge package files (smart merge)
    #[arg(short, long)]
    merge: bool,

    #[command(flatten)]
    apply_features: Option<PkgApplyFeatures>,
}

#[derive(Error, Debug)]
enum UnpackCommandError {
    #[error(
        "failed to get valid current directory (also you can specify destination_folder arg): {0}"
    )]
    GetCurrentDir(#[source] std::io::Error),

    #[error("invalid package filename in path")]
    InvalidPackageFilename(),

    #[error("failed to read the package file: {0}")]
    ReadPackageFile(#[from] std::io::Error),

    #[error("failed to extract as single zip package: {0}")]
    ExtractAsSingleZipPackage(#[from] ExtractSingleZipPackageError),

    #[error("failed to extract as gzip package: {0}")]
    ExtractAsGzipPackage(#[from] ExtractGzipPackageError),

    #[error("failed to extract as single zip package: {single_zip_package_error} and failed to extract as gzip package: {gzip_package_error}")]
    ExtractAsSingleZipOrGzipPackage {
        single_zip_package_error: ExtractSingleZipPackageError,

        gzip_package_error: ExtractGzipPackageError,
    },

    #[error("multiple files found in zip package bundle, please specify file using -p argument or use 'unpack-all' command")]
    MultipleFilesInZipPackage(),
}

impl CliCommand for UnpackCommand {
    fn run(self) -> Result<(), Box<dyn Error>> {
        let destination_folder = match self.destination_folder {
            Some(folder) => folder,
            None => {
                let current_dir =
                    std::env::current_dir().map_err(UnpackCommandError::GetCurrentDir)?;
                let filename = self
                    .package_filepath
                    .file_stem()
                    .ok_or(UnpackCommandError::InvalidPackageFilename())?
                    .to_str()
                    .ok_or(UnpackCommandError::InvalidPackageFilename())?;

                crate::cmd::utils::get_next_filename_if_exists(current_dir.join(filename))
            }
        };

        let pkg_config = CrtCliPkgConfig::from_package_folder(&destination_folder)?;

        let apply_features = combine_apply_features_from_args_and_config(
            self.apply_features.as_ref(),
            pkg_config.as_ref(),
        )
        .unwrap_or_default();

        let extractor_config = PackageToFolderExtractorConfig::default()
            .with_files_already_exists_in_folder_strategy(match self.merge {
                true => FilesAlreadyExistsInFolderStrategy::SmartMerge,
                false => FilesAlreadyExistsInFolderStrategy::ThrowError,
            })
            .with_converter(apply_features.build_combined_converter());

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
                return Err(UnpackCommandError::MultipleFilesInZipPackage().into());
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

        println!("{}", destination_folder.display());

        Ok(())
    }
}
