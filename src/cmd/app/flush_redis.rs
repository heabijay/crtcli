use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use clap::Args;
use std::error::Error;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct FlushRedisCommand;

impl AppCommand for FlushRedisCommand {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        client.app_installer_service().clear_redis_db()?;

        Ok(())
    }
}
