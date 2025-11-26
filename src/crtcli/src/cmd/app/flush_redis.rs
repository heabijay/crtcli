use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use clap::Args;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct FlushRedisCommand;

impl AppCommand for FlushRedisCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        client.app_installer_service().clear_redis_db().await?;

        Ok(())
    }
}
