use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Args;
use std::error::Error;

#[derive(Args, Debug)]
pub struct GetUidPkgCommand {
    /// UId of the package.
    #[arg(value_hint = clap::ValueHint::Other)]
    package_uid: String,

    /// Display the output in JSON format.
    #[arg(long)]
    json: bool,
}

impl AppCommand for GetUidPkgCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let package = app
            .build_client()?
            .package_service()
            .get_package_properties(&self.package_uid)?;

        match &self.json {
            true => println!("{}", serde_json::json!(package)),
            false => {
                println!("{} ({})", package.name, package.uid);
                println!("| Id: {}", package.id);
                println!("| Created on: {}", package.created_on);
                println!("| Modified on: {}", package.modified_on);
                println!("| Maintainer: {}", package.maintainer);
                println!("| Type: {}", package.package_type);
            }
        }

        Ok(())
    }
}
