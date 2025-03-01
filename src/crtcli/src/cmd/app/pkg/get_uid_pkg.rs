use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use async_trait::async_trait;
use clap::Args;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct GetUidPkgCommand {
    /// UId of the package.
    #[arg(value_hint = clap::ValueHint::Other)]
    package_uid: String,

    /// Display the output in JSON format.
    #[arg(long)]
    json: bool,
}

#[async_trait]
impl AppCommand for GetUidPkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let package = client
            .package_service()
            .get_package_properties(&self.package_uid)
            .await?;

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
