use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use async_trait::async_trait;
use clap::Args;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct FlushRedisCommand;

#[async_trait]
impl AppCommand for FlushRedisCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        client.app_installer_service().clear_redis_db().await?;

        Ok(())
    }
}
