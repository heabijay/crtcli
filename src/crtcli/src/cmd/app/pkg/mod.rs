use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use crate::pkg::utils::GetPackageNameFromFolderError;
use clap::Subcommand;
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

    /// Downloads packages from the Creatio instance as a zip archive
    #[clap(visible_aliases = &["d", "dl"])]
    Download(download_pkg::DownloadPkgCommand),

    /// Commands/aliases to simplify manipulate with packages insides File System Development mode (FSD) location
    Fs {
        #[command(subcommand)]
        command: fs::PkgFsCommands,
    },

    /// Installs a package archive (.zip or .gz) into the Creatio instance
    #[clap(visible_alias = "i")]
    Install(install_pkg::InstallPkgCommand),

    /// Print installed package information by Package UId
    GetUid(get_uid_pkg::GetUidPkgCommand),

    /// Execute SQL to make packages locked if it is unlocked in Creatio
    Lock(lock_pkg::LockPkgCommand),

    /// Downloads packages from Creatio, unpacks it to destination folders, and applies configured transforms
    Pull(pull_pkg::PullPkgCommand),

    /// Packs packages from source folders and installs it into the Creatio instance
    Push(push_pkg::PushPkgCommand),

    /// Execute SQL to make packages unlocked if it is locked in Creatio
    Unlock(unlock_pkg::UnlockPkgCommand),
}

impl AppCommand for PkgCommands {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        match self {
            PkgCommands::Compile(command) => command.run(client).await,
            PkgCommands::Download(command) => command.run(client).await,
            PkgCommands::Fs { command } => command.run(client).await,
            PkgCommands::Install(command) => command.run(client).await,
            PkgCommands::GetUid(command) => command.run(client).await,
            PkgCommands::Lock(command) => command.run(client).await,
            PkgCommands::Pull(command) => command.run(client).await,
            PkgCommands::Push(command) => command.run(client).await,
            PkgCommands::Unlock(command) => command.run(client).await,
        }
    }
}

#[derive(Debug, Error)]
pub enum DetectTargetPackageNameError {
    #[error(
        "failed to detect package name in folder (also you can specify package name as argument): {0}"
    )]
    GetPackageNameFromFolder(#[from] GetPackageNameFromFolderError),
}
