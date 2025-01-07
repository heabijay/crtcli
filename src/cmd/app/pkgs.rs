use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use anstyle::Style;
use clap::Args;
use std::error::Error;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct PkgsCommand {
    /// Display the output in JSON format
    #[arg(long)]
    json: bool,
}

impl AppCommand for PkgsCommand {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        let packages = client.workspace_explorer_service().get_packages()?;

        match self.json {
            true => println!("{}", serde_json::json!(packages)),
            false => {
                for package in &packages {
                    println!("{package}");
                }
            }
        }

        eprintln!(
            "{style}Total: {} packages{style:#}",
            packages.len(),
            style = Style::new().underline()
        );

        Ok(())
    }
}
