use crate::cmd::cli::CliCommand;
use crate::cmd::pkg::config_file::CrtCliPkgConfig;
use crate::pkg::bundling;
use crate::pkg::converters::*;
use crate::pkg::utils::{walk_over_package_files, WalkOverPackageFilesContentError};
use clap::Args;
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Args)]
pub struct ApplyCommand {
    /// Path to the package folder
    #[arg(value_hint = clap::ValueHint::DirPath)]
    pub package_folder: PathBuf,

    #[command(flatten)]
    pub apply_features: Option<PkgApplyFeatures>,

    /// Apply transforms only to a specific file within the package folder
    #[arg(short = 'f', long, value_hint = clap::ValueHint::FilePath)]
    pub file: Option<PathBuf>,

    #[clap(skip)]
    pub no_feature_present_warning_disabled: bool,
}

#[derive(Args, Debug, Default, Deserialize, Clone)]
pub struct PkgApplyFeatures {
    /// Sorts files like in the "Data/../*.json", "descriptor.json", ... by some property to simplify merge operations in Git, SVN, etc.
    #[arg(short = 'S', long)]
    #[serde(rename = "sorting")]
    apply_sorting: Option<bool>,

    /// Removes localization files except for the specified cultures (comma-separated list).
    /// Example: --apply-localization-cleanup "en-US,uk-UA"
    #[arg(
        short = 'L',
        long,
        value_name = "EXCEPT-LOCALIZATIONS", 
        value_delimiter = ',',
        value_hint = clap::ValueHint::Other)]
    #[serde(rename = "localization_cleanup")]
    apply_localization_cleanup: Option<Vec<String>>,
}

impl PkgApplyFeatures {
    pub fn combine(&self, other: Option<&PkgApplyFeatures>) -> PkgApplyFeatures {
        PkgApplyFeatures {
            apply_sorting: self
                .apply_sorting
                .or(other.as_ref().and_then(|x| x.apply_sorting)),
            apply_localization_cleanup: self.apply_localization_cleanup.clone().or(other
                .as_ref()
                .and_then(|x| x.apply_localization_cleanup.clone())),
        }
    }

    pub fn build_combined_converter(&self) -> CombinedPkgFileConverter {
        let mut combined = CombinedPkgFileConverter::new();

        if self.apply_sorting.is_some_and(|x| x) {
            combined.add(SortingPkgFileConverter);
        }

        if let Some(localization_cultures) = &self.apply_localization_cleanup {
            combined.add(LocalizationCleanupPkgFileConverter::new(
                HashSet::from_iter(localization_cultures.iter().cloned()),
            ));
        }

        combined
    }
}

#[derive(Error, Debug)]
enum ApplyCommandError {
    #[error("failed to access package file path: {0}")]
    WalkOverPackageFilesContent(#[from] WalkOverPackageFilesContentError),

    #[error("unable to apply features to {0}: {1}")]
    ApplyConverters(String, #[source] CombinedPkgFileConverterError),

    #[error("unable to change file {0}: {1}")]
    FileChangeAccessError(PathBuf, #[source] std::io::Error),
}

impl CliCommand for ApplyCommand {
    fn run(self) -> Result<(), Box<dyn Error>> {
        let pkg_config = CrtCliPkgConfig::from_package_folder(&self.package_folder)?;

        let features = pkg_config
            .map(|c| c.apply().combine(self.apply_features.as_ref()))
            .or_else(|| self.apply_features.clone());

        let features = match features {
            Some(f) => f,
            None if self.no_feature_present_warning_disabled => return Ok(()),
            None => {
                return Err(
                    "Please pass any feature(s) to apply like --apply-sorting, ... to continue"
                        .into(),
                )
            }
        };

        let converter = features.build_combined_converter();

        match &self.file {
            None => {
                for file in walk_over_package_files(self.package_folder.clone()) {
                    let file_path = file
                        .map_err(WalkOverPackageFilesContentError::FolderAccess)
                        .map_err(ApplyCommandError::WalkOverPackageFilesContent)?;

                    apply_file(&self, &converter, file_path)?;
                }
            }
            Some(for_single_file) => {
                apply_file(&self, &converter, for_single_file.to_owned())?;
            }
        }

        return Ok(());

        fn apply_file(
            _self: &ApplyCommand,
            converter: &CombinedPkgFileConverter,
            file_path: PathBuf,
        ) -> Result<(), Box<dyn Error>> {
            let relative_path = file_path
                .strip_prefix(&_self.package_folder)
                .unwrap_or(&file_path);

            if !converter.is_applicable(relative_path.to_str().unwrap()) {
                return Ok(());
            }

            let file =
                bundling::PkgGZipFile::open_fs_file_relative(&_self.package_folder, relative_path)
                    .map_err(|err| WalkOverPackageFilesContentError::FileAccess {
                        path: file_path.clone(),
                        source: err,
                    })
                    .map_err(ApplyCommandError::WalkOverPackageFilesContent)?;

            let converted_content = converter
                .convert(&file.get_escaped_filename(), file.content.clone())
                .map_err(|err| {
                    ApplyCommandError::ApplyConverters(relative_path.display().to_string(), err)
                })?;

            if let Some(content) = converted_content {
                if content != file.content {
                    std::fs::write(&file_path, content)
                        .map_err(|err| ApplyCommandError::FileChangeAccessError(file_path, err))?;

                    eprintln!("\t{}\t{}", "modified:", file.filename);
                }
            } else {
                std::fs::remove_file(&file_path)
                    .map_err(|err| ApplyCommandError::FileChangeAccessError(file_path, err))?;

                eprintln!("\t{}\t{}", "deleted:".red(), file.filename.red());
            }

            Ok(())
        }
    }
}
