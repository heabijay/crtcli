use crate::app::CrtClient;
use crate::cmd::app::{AppCommand, AppCommandArgs};
use anstyle::Style;
use clap::Args;
use std::error::Error;

#[derive(Args, Debug)]
pub struct RestartCommand;

impl AppCommand for RestartCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let client = app.build_client()?;

        client.app_installer_service().restart_app()?;

        print_app_restart_requested(&client);

        Ok(())
    }
}

pub fn print_app_restart_requested(client: &CrtClient) {
    eprintln!("Application restart has been requested");

    if !client.is_net_framework() {
        eprintln!(
            "{style}Note: if restart does not work, please check if you need to use --net-framework flag{style:#}",
            style=Style::new().dimmed()
        );
    }
}
