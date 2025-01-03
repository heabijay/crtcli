use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Args;
use std::error::Error;

#[derive(Args, Debug)]
pub struct InstallLogCommand;

impl AppCommand for InstallLogCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let log_file = app
            .build_client()?
            .package_installer_service()
            .get_log_file()?;

        println!("{}", log_file.trim_end());

        Ok(())
    }
}
