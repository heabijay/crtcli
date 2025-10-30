#[macro_use]
extern crate anstream;

use anstyle::{AnsiColor, Color, Style};
use clap::Parser;
use std::fmt::{Debug, Display, Formatter};
use std::process::ExitCode;

mod app;
mod cmd;
mod pkg;
mod utils;

#[derive(Debug)]
struct CommandHandledError(ExitCode);

impl Display for CommandHandledError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Command finished with exit code: {:?}", self.0)
    }
}

impl std::error::Error for CommandHandledError {}

fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    let cli: cmd::Cli = cmd::Cli::parse();
    let is_debug = cli.debug();

    match cli.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) if err.is::<CommandHandledError>() => {
            err.downcast_ref::<CommandHandledError>().unwrap().0
        }
        Err(err) => {
            eprintln!(
                "{style}Error: {err:#}{style:#}",
                style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)))
            );

            if is_debug {
                eprintln!();
                eprintln!(
                    "{style}Error (Debug-view): {err:?}{style:#}",
                    style = Style::new()
                        .fg_color(Some(Color::Ansi(AnsiColor::BrightRed)))
                        .dimmed()
                );
            }

            ExitCode::FAILURE
        }
    }
}
