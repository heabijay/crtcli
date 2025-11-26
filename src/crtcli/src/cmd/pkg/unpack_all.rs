use crate::cmd::cli::{CliCommand, CommandResult};
use crate::pkg::bundling::extractor::*;
use crate::pkg::transforms::post::{CombinedPkgFolderPostTransformError, PkgFolderPostTransform};
use anstream::stderr;
use clap::Args;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Args)]
pub struct UnpackAllCommand {
    /// Path to the zip package archive file
    #[arg(value_hint = clap::ValueHint::FilePath)]
    package_filepath: PathBuf,

    /// Destination folder where all extracted package files will be saved
    #[arg(short, long = "destination", value_hint = clap::ValueHint::DirPath)]
    destination_folder: Option<PathBuf>,

    /// If destination folder is not empty, attempt to merge package files (smart merge)
    #[arg(short, long)]
    merge: bool,

    #[command(flatten)]
    apply_features: crate::pkg::transforms::PkgApplyFeatures,

    #[command(flatten)]
    apply_post_features: crate::pkg::transforms::post::PkgApplyPostFeatures,
}

#[derive(Error, Debug)]
enum UnpackAllCommandError {
    #[error("invalid package filename in path")]
    InvalidPackageFilename,

    #[error("file access error: {0}")]
    FileAccess(#[from] std::io::Error),

    #[error("{0}")]
    ExtractZipPackage(#[from] ExtractZipPackageError),

    #[error("failed to apply post transforms for package folder {0}: {1}")]
    ApplyPostTransforms(PathBuf, #[source] CombinedPkgFolderPostTransformError),
}

impl CliCommand for UnpackAllCommand {
    fn run(self) -> CommandResult {
        let destination_folder = match self.destination_folder {
            Some(folder) => folder,
            None => {
                let filename = self
                    .package_filepath
                    .file_stem()
                    .ok_or(UnpackAllCommandError::InvalidPackageFilename)?
                    .to_str()
                    .ok_or(UnpackAllCommandError::InvalidPackageFilename)?;

                crate::cmd::utils::get_next_filename_if_exists(PathBuf::from(".").join(filename))
            }
        };

        let file = std::fs::File::open(&self.package_filepath)
            .map_err(UnpackAllCommandError::FileAccess)?;

        let config = PackageToFolderExtractorConfig::default()
            .with_files_already_exists_in_folder_strategy(match self.merge {
                true => FilesAlreadyExistsInFolderStrategy::SmartMerge,
                false => FilesAlreadyExistsInFolderStrategy::ThrowError,
            })
            .with_transform(self.apply_features.build_combined_transform());

        let package_folders = extract_zip_package_to_folder(file, &destination_folder, &config)
            .map_err(UnpackAllCommandError::ExtractZipPackage)?;

        let post_transforms = self.apply_post_features.build_combined_transform();

        for package_folder in package_folders {
            post_transforms
                .transform(&package_folder, false, stderr())
                .map_err(|err| UnpackAllCommandError::ApplyPostTransforms(package_folder, err))?;
        }

        println!("{}", destination_folder.display());

        Ok(())
    }
}
