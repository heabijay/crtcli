use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use anstyle::Style;
use clap::Args;
use std::error::Error;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct RestartCommand;

impl AppCommand for RestartCommand {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        client.app_installer_service().restart_app()?;

        print_app_restart_requested(&client);

        Ok(())
    }
}

pub fn print_app_restart_requested(client: &CrtClient) {
    eprintln!(
        "✔ Application restart has been requested at {bold}{url}{bold:#}",
        bold = Style::new().bold(),
        url = client.base_url()
    );

    if !client.is_net_framework() {
        eprintln!(
            "{style}Note: if restart does not work, please check if you need to use --net-framework flag{style:#}",
            style=Style::new().dimmed()
        );
    }
}
