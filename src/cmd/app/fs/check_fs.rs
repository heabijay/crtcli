use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Args;
use owo_colors::OwoColorize;
use std::error::Error;

#[derive(Args, Debug)]
pub struct CheckFsCommand;

impl AppCommand for CheckFsCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let result = app
            .build_client()?
            .workspace_explorer_service()
            .get_is_file_system_development_mode()?;

        eprintln!(
            "File System Development mode (FSD): {}",
            match result {
                true => "Enabled".green().to_string(),
                false => "Disabled".red().to_string(),
            }
        );

        Ok(())
    }
}
