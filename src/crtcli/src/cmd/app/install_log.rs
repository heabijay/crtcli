use crate::app::{CrtClient, InstallLogWatcherBuilder, InstallLogWatcherEvent};
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use anstyle::{AnsiColor, Color, Style};
use clap::Args;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct InstallLogCommand {
    /// Watch for and display installation log updates in real-time
    #[arg(long)]
    watch: bool,
}

impl AppCommand for InstallLogCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        if self.watch {
            start_log_watcher_and_wait_forever(client).await;
        } else {
            let log_file = client.package_installer_service().get_log_file().await?;

            println!("{}", log_file.trim_end());
        }

        Ok(())
    }
}

async fn start_log_watcher_and_wait_forever(client: Arc<CrtClient>) {
    InstallLogWatcherBuilder::new_with_current_session(client)
        .start(|event| match event {
            InstallLogWatcherEvent::Clear => {
                println!("----------------------------------------")
            }
            InstallLogWatcherEvent::Append(text) => {
                print!("{}", text)
            }
            InstallLogWatcherEvent::FetchError(error) => {
                eprintln!(
                    "{style}warning (log polling): {error}{style:#}",
                    error = error,
                    style = Style::new()
                        .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                        .dimmed()
                )
            }
        })
        .wait_until_stopped() // This will never be stopped, except for some signal like Ctrl+C, SIGKILL, etc.
        .await;
}
