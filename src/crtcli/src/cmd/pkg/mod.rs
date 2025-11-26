use crate::CommandHandledError;
use crate::cfg::WorkspaceConfig;
use crate::cfg::workspace::WorkspacePkgConfig;
use crate::cmd::cli::{CliCommand, CommandDynError, CommandResult};
use anstyle::{AnsiColor, Color, Style};
use clap::Subcommand;
use std::process::ExitCode;

pub mod apply;
mod pack;
mod unpack;
mod unpack_all;

#[derive(Debug, Subcommand)]
pub enum PkgCommands {
    /// Applies transformations to the contents of a packages folders
    Apply(apply::ApplyCommand),

    /// Creates a package archive (.zip or .gz) from package folders
    #[clap(visible_alias = "p")]
    Pack(pack::PackCommand),

    /// Extract a single package from a package archive (.zip or .gz)
    #[clap(visible_alias = "u")]
    Unpack(unpack::UnpackCommand),

    /// Extract all packages from a zip archive
    #[clap(visible_alias = "ua")]
    UnpackAll(unpack_all::UnpackAllCommand),
}

impl CliCommand for PkgCommands {
    fn run(self) -> CommandResult {
        match self {
            PkgCommands::Apply(command) => command.run(),
            PkgCommands::Pack(command) => command.run(),
            PkgCommands::Unpack(command) => command.run(),
            PkgCommands::UnpackAll(command) => command.run(),
        }
    }
}

pub trait WorkspaceConfigCmdPkgExt {
    fn packages_or_print_error(&self) -> Result<&Vec<WorkspacePkgConfig>, CommandDynError>;
}

impl WorkspaceConfigCmdPkgExt for WorkspaceConfig {
    fn packages_or_print_error(&self) -> Result<&Vec<WorkspacePkgConfig>, CommandDynError> {
        if !self.packages().is_empty() {
            return Ok(self.packages());
        }

        let bold = Style::new().bold();

        eprintln!(
            "{red_bold}error:{red_bold:#} the following required arguments were not provided:",
            red_bold = Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::Red)))
                .bold(),
        );

        eprintln!(
            "  {green}[PACKAGE_NAME(S) or PACKAGE_FOLDER(S)]{green:#}",
            green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)))
        );

        eprintln!();
        eprintln!("You can specify the target package(s) using one of the following methods:");
        eprintln!(
            " - Pass `PACKAGE_NAME(S)` or `PACKAGE_FOLDER(S)` as an argument to `crtcli [COMMAND]`"
        );
        eprintln!(
            " - Execute the command inside a package directory (containing `descriptor.json`)"
        );
        eprintln!(
            " - Configure the `packages` parameter in the `workspace.crtcli.toml` file in the current directory"
        );

        eprintln!();
        eprintln!("For more information, try '{bold}crtcli [OPTIONS] [COMMAND] --help{bold:#}'.");

        Err(CommandHandledError(ExitCode::FAILURE).into())
    }
}
