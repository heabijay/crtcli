use crate::cmd::app::{AppCommand, AppCommandArgs};
use anstream::stdout;
use anstyle::{AnsiColor, Color, Style};
use clap::Args;
use std::error::Error;
use std::io::Write;

#[derive(Args, Debug)]
pub struct CheckFsCommand;

impl AppCommand for CheckFsCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let result = app
            .build_client()?
            .workspace_explorer_service()
            .get_is_file_system_development_mode()?;

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
