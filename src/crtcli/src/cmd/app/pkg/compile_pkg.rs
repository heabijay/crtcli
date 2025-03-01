use crate::app::CrtClient;
use crate::cmd::app;
use crate::cmd::app::{AppCommand, print_build_response};
use crate::cmd::cli::CommandResult;
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::Args;
use std::error::Error;
use std::sync::Arc;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct CompilePkgCommand {
    /// Name of package to compile (default: package name from ./descriptor.json)
    #[arg(value_hint = clap::ValueHint::Other)]
    pub package_name: Option<String>,

    /// Restart the Creatio application after successful package compilation
    #[arg(short, long)]
    pub restart: bool,
}

#[derive(Debug, Error)]
pub enum CompilePkgCommandError {
    #[error("App restart error: {0}")]
    AppRestart(#[source] Box<dyn Error + Send + Sync>),
}

#[async_trait]
impl AppCommand for CompilePkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let package_name = detect_target_package_name!(&self.package_name);

        let progress = spinner_precise!(
            "Compiling {bold}{package_name}{bold:#} package at {bold}{url}{bold:#}",
            bold = Style::new().bold(),
            url = client.base_url()
        );

        let response = client
            .workspace_explorer_service()
            .build_package(package_name)
            .await?;

        progress.finish_and_clear();

        print_build_response(&response)?;

        eprintln!(
            "{green}✔ Package {green_bold}{package_name}{green_bold:#}{green} successfully compiled at {green_bold}{url}{green_bold:#}{green}!{green:#}",
            green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            green_bold = Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::Green)))
                .bold(),
            url = client.base_url(),
        );

        if self.restart {
            app::restart::RestartCommand
                .run(client)
                .await
                .map_err(CompilePkgCommandError::AppRestart)?;
        }

        Ok(())
    }
}
