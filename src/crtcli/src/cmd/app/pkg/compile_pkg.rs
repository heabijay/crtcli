use crate::app::CrtClient;
use crate::cmd::app;
use crate::cmd::app::{AppCommand, print_build_response};
use crate::cmd::cli::CommandResult;
use crate::pkg::utils::get_package_name_from_current_dir;
use anstyle::{AnsiColor, Color, Style};
use clap::Args;
use std::error::Error;
use std::sync::Arc;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct CompilePkgCommand {
    /// A space-separated or comma-separated list of package names to compile (default: package name from ./descriptor.json)
    #[arg(value_delimiter = ',', value_hint = clap::ValueHint::Other)]
    pub packages_names: Vec<String>,

    /// Use Rebuild method instead of just Build
    #[arg(short = 'f', long)]
    pub force_rebuild: bool,

    /// Restart the Creatio application after successful package compilation
    #[arg(short, long)]
    pub restart: bool,
}

#[derive(Debug, Error)]
pub enum CompilePkgCommandError {
    #[error("App restart error: {0}")]
    AppRestart(#[source] Box<dyn Error + Send + Sync>),
}

impl AppCommand for CompilePkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let packages_names = if self.packages_names.is_empty() {
            &vec![get_package_name_from_current_dir()?]
        } else {
            &self.packages_names
        };

        if packages_names.len() > 1 {
            eprintln!(
                "{style}warning (pkg-compile): multiple packages are specified, crtcli prefer to use app compile in this case{style:#}",
                style = Style::new()
                    .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                    .dimmed()
            );

            return app::compile::CompileCommand {
                restart: self.restart,
                force_rebuild: self.force_rebuild,
            }
            .run(client)
            .await;
        }

        let package_name = packages_names[0].as_str();

        let progress = spinner_precise!(
            "{operation_str} {bold}{package_name}{bold:#} package at {bold}{url}{bold:#}",
            bold = Style::new().bold(),
            operation_str = if self.force_rebuild {
                "Rebuilding"
            } else {
                "Compiling"
            },
            url = client.base_url(),
        );

        let response = if self.force_rebuild {
            client
                .workspace_explorer_service()
                .rebuild_package(package_name)
                .await?
        } else {
            client
                .workspace_explorer_service()
                .build_package(package_name)
                .await?
        };

        progress.suspend(|| print_build_response(&response))?;

        progress.finish_with_message(format!(
            "{green}Package {green_bold}{package_name}{green_bold:#}{green} successfully {operation_str} at {green_bold}{url}{green_bold:#}{green}!{green:#}",
            green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            green_bold = Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::Green)))
                .bold(),
            operation_str = if self.force_rebuild {
                "rebuilt"
            } else {
                "compiled"
            },
            url = client.base_url(),
        ));

        if self.restart {
            app::restart::RestartCommand
                .run(client)
                .await
                .map_err(CompilePkgCommandError::AppRestart)?;
        }

        Ok(())
    }
}
