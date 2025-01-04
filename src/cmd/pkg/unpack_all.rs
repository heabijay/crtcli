use crate::cmd::cli::CliCommand;
use crate::cmd::pkg::apply::PkgApplyFeatures;
use crate::pkg::bundling::extractor::*;
use clap::Args;
use std::error::Error;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Args)]
pub struct UnpackAllCommand {
    /// Path to the zip package archive file
    #[arg(value_hint = clap::ValueHint::FilePath)]
    package_filepath: PathBuf,

    /// Destination folder where all extracted package files will be saved
    #[arg(value_hint = clap::ValueHint::DirPath)]
    destination_folder: Option<PathBuf>,

    /// If destination folder is not empty, attempt to merge package files (smart merge)
    #[arg(short, long)]
    merge: bool,

    #[command(flatten)]
    apply_features: PkgApplyFeatures,
}

#[derive(Error, Debug)]
enum UnpackAllCommandError {
    #[error(
        "failed to get valid current directory (also you can specify destination_folder arg): {0}"
    )]
    GetCurrentDir(#[source] std::io::Error),

    #[error("invalid package filename in path")]
    InvalidPackageFilename(),

    #[error("file access error: {0}")]
    FileAccess(#[from] std::io::Error),

    #[error("{0}")]
    ExtractZipPackage(#[from] ExtractZipPackageError),
}

impl CliCommand for UnpackAllCommand {
    fn run(self) -> Result<(), Box<dyn Error>> {
        let destination_folder = match self.destination_folder {
            Some(folder) => folder,
            None => {
                let current_dir =
                    std::env::current_dir().map_err(UnpackAllCommandError::GetCurrentDir)?;
                let filename = self
                    .package_filepath
                    .file_stem()
                    .ok_or(UnpackAllCommandError::InvalidPackageFilename())?
                    .to_str()
                    .ok_or(UnpackAllCommandError::InvalidPackageFilename())?;

                crate::cmd::utils::get_next_filename_if_exists(current_dir.join(filename))
            }
        };

        let file = std::fs::File::open(&self.package_filepath)
            .map_err(UnpackAllCommandError::FileAccess)?;
        let config = PackageToFolderExtractorConfig::default()
            .with_files_already_exists_in_folder_strategy(match self.merge {
                true => FilesAlreadyExistsInFolderStrategy::SmartMerge,
                false => FilesAlreadyExistsInFolderStrategy::ThrowError,
            })
            .with_converter(self.apply_features.build_combined_converter());

        extract_zip_package_to_folder(file, &destination_folder, &config)
            .map_err(UnpackAllCommandError::ExtractZipPackage)?;

        println!("{}", destination_folder.display());

        Ok(())
    }
}