use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Subcommand;
use std::error::Error;
use std::path::Path;
use thiserror::Error;
use walkdir::WalkDir;

mod pull_pkg_fs;

mod push_pkg_fs;

#[derive(Debug, Subcommand)]
pub enum PkgFsCommands {
    /// Unload package in current folder from Creatio database into filesystem and applies any configured transforms
    Pull(pull_pkg_fs::PullPkgFsCommand),

    /// Load package in current folder from filesystem into Creatio database and optionally compiles it
    Push(push_pkg_fs::PushPkgFsCommand),
}

impl AppCommand for PkgFsCommands {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        match self {
            PkgFsCommands::Pull(command) => command.run(app),
            PkgFsCommands::Push(command) => command.run(app),
        }
    }
}

#[derive(Debug, Error)]
#[error("prepare fs package folder failed: {0}")]
pub struct PreparePkgFsFolderError(#[from] std::io::Error);

fn prepare_pkg_fs_folder(package_folder: impl AsRef<Path>) -> Result<(), PreparePkgFsFolderError> {
    delete_empty_folders_in_package_schemas(package_folder)?;

    return Ok(());

    fn delete_empty_folders_in_package_schemas(
        package_folder: impl AsRef<Path>,
    ) -> Result<(), std::io::Error> {
        [
            crate::pkg::paths::ASSEMBLIES_FOLDER,
            crate::pkg::paths::DATA_FOLDER,
            crate::pkg::paths::RESOURCES_FOLDER,
            crate::pkg::paths::SCHEMAS_FOLDER,
            crate::pkg::paths::SQL_SCRIPTS_FOLDER,
        ]
        .into_iter()
        .map(|p| package_folder.as_ref().join(p))
        .filter(|p| p.exists())
        .try_for_each(delete_empty_folders_in_folder)?;

        return Ok(());

        fn delete_empty_folders_in_folder(folder: impl AsRef<Path>) -> Result<(), std::io::Error> {
            for entry in folder.as_ref().read_dir()? {
                let entry = entry?;
                let path = entry.path();

                if !path.is_dir() {
                    continue;
                }

                let has_any_file_recursive = WalkDir::new(path)
                    .contents_first(true)
                    .into_iter()
                    .next()
                    .is_some_and(|x| x.is_ok_and(|x| x.file_type().is_file()));

                if !has_any_file_recursive {
                    std::fs::remove_dir_all(entry.path())?;
                }
            }

            Ok(())
        }
    }
}
