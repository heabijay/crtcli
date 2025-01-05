#[macro_use]
extern crate anstream;

use anstyle::{AnsiColor, Color, Style};
use clap::Parser;
use std::process::ExitCode;

mod app;
mod cmd;
mod pkg;
mod utils;

fn main() -> ExitCode {
    load_envs();

    let cli: cmd::Cli = cmd::Cli::parse();
    let is_debug = cli.debug();

    match cli.run() {
        Ok(_) => ExitCode::SUCCESS,
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

fn load_envs() {
    dotenvy::dotenv().ok();
    dotenvy::from_filename(".crtcli.env").ok();

    if let Ok(env_filenames) = std::env::var("CRTCLI_LOAD_ENV_FILENAME") {
        for env_filename in env_filenames.split(";").map(str::trim) {
            dotenvy::from_filename(env_filename).ok();
        }
    }
}
