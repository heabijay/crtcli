use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Args;
use std::error::Error;

#[derive(Args, Debug)]
pub struct PkgsCommand {
    /// Display the output in JSON format
    #[arg(long)]
    json: bool,
}

impl AppCommand for PkgsCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let packages = app
            .build_client()?
            .workspace_explorer_service()
            .get_packages()?;

        match self.json {
            true => {
                println!("{}", serde_json::json!(packages))
            }
            false => {
                for package in &packages {
                    println!("{package}");
                }
            }
        }

        eprintln!("Total: {} packages", packages.len());

        Ok(())
    }
}
