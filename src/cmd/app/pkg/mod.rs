use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::pkg::utils::GetPackageNameFromFolderError;
use clap::Subcommand;
use std::error::Error;
use std::sync::Arc;
use thiserror::Error;

mod fs;

mod compile_pkg;

mod download_pkg;

mod install_pkg;

mod get_uid_pkg;

mod pull_pkg;

mod push_pkg;

mod lock_pkg;

mod unlock_pkg;

#[derive(Debug, Subcommand)]
pub enum PkgCommands {
    /// Compiles a specific package within the Creatio instance
    Compile(compile_pkg::CompilePkgCommand),

    /// Downloads one or more packages from the Creatio instance as a zip archive
    Download(download_pkg::DownloadPkgCommand),

    /// Commands/aliases to simplify manipulate with package insides File System Development mode (FSD) location
    Fs {
        #[command(subcommand)]
        command: fs::PkgFsCommands,
    },

    /// Installs a package archive (.zip or .gz) into the Creatio instance
    Install(install_pkg::InstallPkgCommand),

    /// Print installed package information by Package UId
    GetUid(get_uid_pkg::GetUidPkgCommand),

    /// Execute SQL to make package locked if it is unlocked in Creatio
    Lock(lock_pkg::LockPkgCommand),

    /// Downloads a package from Creatio, unpacks it to a destination folder, and applies configured transforms
    Pull(pull_pkg::PullPkgCommand),

    /// Packs a package from a source folder and installs it into the Creatio instance
    Push(push_pkg::PushPkgCommand),

    /// Execute SQL to make package unlocked if it is locked in Creatio
    Unlock(unlock_pkg::UnlockPkgCommand),
}

impl AppCommand for PkgCommands {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        match self {
            PkgCommands::Compile(command) => command.run(client),
            PkgCommands::Download(command) => command.run(client),
            PkgCommands::Fs { command } => command.run(client),
            PkgCommands::Install(command) => command.run(client),
            PkgCommands::GetUid(command) => command.run(client),
            PkgCommands::Lock(command) => command.run(client),
            PkgCommands::Pull(command) => command.run(client),
            PkgCommands::Push(command) => command.run(client),
            PkgCommands::Unlock(command) => command.run(client),
        }
    }
}

#[derive(Debug, Error)]
pub enum DetectTargetPackageNameError {
    #[error("failed to detect package name in folder (also you can specify package name as argument): {0}")]
    GetPackageNameFromFolder(#[from] GetPackageNameFromFolderError),
}
