use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use clap::Args;
use std::error::Error;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct LockPkgCommand {
    /// Package name to lock
    #[arg(value_hint = clap::ValueHint::Other)]
    package_name: Option<String>,
}

impl AppCommand for LockPkgCommand {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        let package_name = detect_target_package_name!(&self.package_name);

        let result = client.sql_scripts().lock_package(package_name)?;

        eprintln!("Rows affected: {}", result);

        Ok(())
    }
}
