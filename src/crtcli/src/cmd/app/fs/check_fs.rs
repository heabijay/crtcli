use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use anstream::stdout;
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::Args;
use std::io::Write;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct CheckFsCommand;

#[async_trait]
impl AppCommand for CheckFsCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let result = client
            .workspace_explorer_service()
            .get_is_file_system_development_mode()
            .await?;

        let mut stdout = stdout().lock();

        write!(stdout, "File System Development mode (FSD): ").unwrap();

        match result {
            true => writeln!(
                stdout,
                "{style}Enabled{style:#}",
                style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)))
            )
            .unwrap(),
            false => writeln!(
                stdout,
                "{style}Disabled{style:#}",
                style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)))
            )
            .unwrap(),
        }

        Ok(())
    }
}
