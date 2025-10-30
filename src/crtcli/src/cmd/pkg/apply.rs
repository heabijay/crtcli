use crate::cfg::PkgConfig;
use crate::cfg::package::combine_apply_features_from_args_and_config;
use crate::cmd::cli::{CliCommand, CommandDynError, CommandResult};
use crate::pkg::bundling;
use crate::pkg::transforms::*;
use crate::pkg::utils::{WalkOverPackageFilesContentError, walk_over_package_files};
use anstream::stdout;
use anstyle::{AnsiColor, Color, Style};
use clap::Args;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Args)]
pub struct ApplyCommand {
    /// Paths to the packages folders (default: current directory)
    #[arg(value_hint = clap::ValueHint::DirPath)]
    pub packages_folders: Vec<PathBuf>,

    #[command(flatten)]
    pub apply_features: Option<crate::pkg::PkgApplyFeatures>,

    /// Apply transforms only to a specific file within the package folder
    #[arg(short = 'f', long, value_hint = clap::ValueHint::FilePath)]
    pub file: Option<PathBuf>,

    /// Checks for potential changes without applying them, exiting with a non-zero code if changes are needed
    #[arg(long = "check")]
    pub check_only: bool,

    #[clap(skip)]
    pub no_feature_present_warning_disabled: bool,
}

#[derive(Error, Debug)]
enum ApplyCommandError {
    #[error("failed to access package file path: {0}")]
    WalkOverPackageFilesContent(#[from] WalkOverPackageFilesContentError),

    #[error("unable to apply features to {0}: {1}")]
    ApplyTransforms(String, #[source] CombinedPkgFileTransformError),

    #[error("unable to change file {0}: {1}")]
    FileChangeAccessError(PathBuf, #[source] std::io::Error),

    #[error("apply check not passed, there are some files with non-applied transforms")]
    CheckNotPassed,
}

impl CliCommand for ApplyCommand {
    fn run(self) -> CommandResult {
        let packages_folders = if self.packages_folders.is_empty() {
            &vec![PathBuf::from(".")]
        } else {
            &self.packages_folders
        };

        let mut any_applied = false;

        for package_folder in packages_folders {
            if packages_folders.len() > 1 {
                println!(
                    "{verb} {bold}{package_folder}{bold:#}:",
                    verb = if self.check_only {
                        "Checking"
                    } else {
                        "Applying"
                    },
                    package_folder = package_folder.display(),
                    bold = Style::new().bold(),
                )
            }

            if apply_package_folder(&self, package_folder)? {
                any_applied = true;
            } else if packages_folders.len() > 1 {
                println!(
                    "\t{style}— Nothing to do —{style:#}",
                    style = Style::new().italic().dimmed()
                );
            }
        }

        if self.check_only && any_applied {
            return Err(ApplyCommandError::CheckNotPassed.into());
        }

        return Ok(());

        fn apply_package_folder(
            _self: &ApplyCommand,
            package_folder: &Path,
        ) -> Result<bool, CommandDynError> {
            let pkg_config = PkgConfig::from_package_folder(package_folder)?;

            let apply_features = combine_apply_features_from_args_and_config(
                _self.apply_features.as_ref(),
                pkg_config.as_ref(),
            );

            let apply_features = match apply_features {
                Some(f) => f,
                None if _self.no_feature_present_warning_disabled => return Ok(false),
                None => {
                    println!(
                        "{style}warning: no feature is present, please use an option like --apply-sorting to do something{style:#}",
                        style = Style::new()
                            .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                            .dimmed(),
                    );
                    return Ok(false);
                }
            };

            let transform = apply_features.build_combined_transform();
            let mut stdout = stdout().lock();

            let mut any_applied = false;

            match &_self.file {
                None => {
                    for file in walk_over_package_files(package_folder) {
                        let file_path = file
                            .map_err(WalkOverPackageFilesContentError::FolderAccess)
                            .map_err(ApplyCommandError::WalkOverPackageFilesContent)?;

                        if apply_file(_self, package_folder, &mut stdout, &transform, file_path)? {
                            any_applied = true;
                        };
                    }
                }
                Some(for_single_file) => {
                    if apply_file(
                        _self,
                        package_folder,
                        &mut stdout,
                        &transform,
                        for_single_file.to_owned(),
                    )? {
                        any_applied = true;
                    }
                }
            }

            Ok(any_applied)
        }

        fn apply_file(
            _self: &ApplyCommand,
            package_folder: &Path,
            mut stdout: impl Write,
            transform: &CombinedPkgFileTransform,
            file_path: PathBuf,
        ) -> Result<bool, CommandDynError> {
            let relative_path = file_path.strip_prefix(package_folder).unwrap_or(&file_path);

            if !transform.is_applicable(relative_path.to_str().unwrap()) {
                return Ok(false);
            }

            let file = bundling::PkgGZipFile::open_fs_file_relative(package_folder, relative_path)
                .map_err(|err| WalkOverPackageFilesContentError::FileAccess {
                    path: file_path.clone(),
                    source: err,
                })
                .map_err(ApplyCommandError::WalkOverPackageFilesContent)?;

            let converted_content = transform
                .transform(
                    &file.filename, // No need to use file.to_native_path_string because in this case the file was read from the native package folder
                    file.content.clone(),
                )
                .map_err(|err| {
                    ApplyCommandError::ApplyTransforms(relative_path.display().to_string(), err)
                })?;

            if let Some(content) = converted_content {
                if content != file.content {
                    if _self.check_only {
                        writeln!(stdout, "\tto be modified:\t{}", file.filename).unwrap();
                    } else {
                        std::fs::write(&file_path, content).map_err(|err| {
                            ApplyCommandError::FileChangeAccessError(file_path, err)
                        })?;

                        writeln!(stdout, "\tmodified:\t{}", file.filename).unwrap();
                    }

                    return Ok(true);
                }
            } else {
                if _self.check_only {
                    writeln!(
                        stdout,
                        "{style}\tto be deleted:\t{}{style:#}",
                        file.filename,
                        style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)))
                    )
                    .unwrap();
                } else {
                    std::fs::remove_file(&file_path)
                        .map_err(|err| ApplyCommandError::FileChangeAccessError(file_path, err))?;

                    writeln!(
                        stdout,
                        "{style}\tdeleted:\t{}{style:#}",
                        file.filename,
                        style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)))
                    )
                    .unwrap();
                }

                return Ok(true);
            }

            Ok(false)
        }
    }
}
