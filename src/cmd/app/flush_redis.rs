use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Args;
use std::error::Error;

#[derive(Args, Debug)]
pub struct FlushRedisCommand;

impl AppCommand for FlushRedisCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        app.build_client()?
            .app_installer_service()
            .clear_redis_db()?;

        Ok(())
    }
}
