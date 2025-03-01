use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use async_trait::async_trait;
use clap::Args;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct InstallLogCommand;

#[async_trait]
impl AppCommand for InstallLogCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let log_file = client.package_installer_service().get_log_file().await?;

        println!("{}", log_file.trim_end());

        Ok(())
    }
}
