use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use clap::Args;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Args, Debug)]
pub struct PingCommand {
    /// Keep sending ping requests until a successful connection is established
    #[arg(long)]
    pub wait: bool,

    #[clap(skip)]
    pub initial_delay: Option<Duration>,
}

impl AppCommand for PingCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        if self.wait {
            handle_wait_cmd(self, client).await
        } else {
            if client.process_schema_manager_service().ping().await? {
                println!("Pong!");
                Ok(())
            } else {
                Err("Ping failed".into())
            }
        }
    }
}

async fn handle_wait_cmd(_self: &PingCommand, client: Arc<CrtClient>) -> CommandResult {
    let progress = spinner!("Pinging application: waiting for a while...");

    if let Some(initial_delay) = _self.initial_delay {
        tokio::time::sleep(initial_delay).await;
    }

    let mut attempt = 1;

    loop {
        progress.set_message(format!(
            "Pinging application: attempt #{attempt} in progress..."
        ));

        if client
            .process_schema_manager_service()
            .ping()
            .await
            .is_ok_and(|x| x)
        {
            break;
        }

        progress.set_message(format!(
            "Pinging application: attempt #{attempt} failed. Retrying..."
        ));
        tokio::time::sleep(DEFAULT_PING_INTERVAL).await;
        attempt += 1;
    }

    progress.finish_with_message("Pinging application: connection established!");

    Ok(())
}
