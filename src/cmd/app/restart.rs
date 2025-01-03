use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Args;
use std::error::Error;

#[derive(Args, Debug)]
pub struct RestartCommand;

impl AppCommand for RestartCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let client = app.build_client()?;

        client.app_installer_service().restart_app()?;

        eprintln!("Application restart has been requested");

        if !client.is_net_framework() {
            eprintln!("Note: if restart does not work, please check if you need to use --net-framework flag");
        }

        Ok(())
    }
}
