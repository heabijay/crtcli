use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use clap::Args;
use std::error::Error;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct InstallLogCommand;

impl AppCommand for InstallLogCommand {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        let log_file = client.package_installer_service().get_log_file()?;

        println!("{}", log_file.trim_end());

        Ok(())
    }
}
