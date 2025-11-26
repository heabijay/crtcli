use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use anstream::stdout;
use anstyle::Style;
use clap::Args;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct PkgsCommand {
    /// Display the output in JSON format
    #[arg(long)]
    json: bool,
}

impl AppCommand for PkgsCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let packages = client.workspace_explorer_service().get_packages().await?;

        match self.json {
            true => println!("{}", serde_json::json!(packages)),
            false => {
                for package in &packages {
                    println!("{package}");
                }
            }
        }

        if stdout().is_terminal() {
            eprintln!(
                "{style}Total: {} packages{style:#}",
                packages.len(),
                style = Style::new().underline()
            );
        }

        Ok(())
    }
}
