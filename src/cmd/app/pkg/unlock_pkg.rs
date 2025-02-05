use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use async_trait::async_trait;
use clap::Args;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct UnlockPkgCommand {
    /// Name of the package to unlock
    #[arg(value_hint = clap::ValueHint::Other)]
    package_name: Option<String>,
}

#[async_trait]
impl AppCommand for UnlockPkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let package_name = detect_target_package_name!(&self.package_name);

        let result = client.sql_scripts().unlock_package(package_name).await?;

        eprintln!("Rows affected: {}", result);

        Ok(())
    }
}
