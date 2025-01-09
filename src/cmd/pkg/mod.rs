use crate::cmd::cli::CliCommand;
use clap::Subcommand;
use std::error::Error;

pub mod apply;
pub mod config_file;
mod pack;
mod unpack;
mod unpack_all;

pub use apply::PkgApplyFeatures;

#[derive(Debug, Subcommand)]
pub enum PkgCommands {
    /// Applies transformations to the contents of a package folder
    Apply(apply::ApplyCommand),

    /// Creates a package archive (.zip or .gz) from package folders
    Pack(pack::PackCommand),

    /// Extract a single package from a package archive (.zip or .gz)
    Unpack(unpack::UnpackCommand),

    /// Extract all packages from a zip archive
    UnpackAll(unpack_all::UnpackAllCommand),
}

impl CliCommand for PkgCommands {
    fn run(self) -> Result<(), Box<dyn Error>> {
        match self {
            PkgCommands::Apply(command) => command.run(),
            PkgCommands::Pack(command) => command.run(),
            PkgCommands::Unpack(command) => command.run(),
            PkgCommands::UnpackAll(command) => command.run(),
        }
    }
}
