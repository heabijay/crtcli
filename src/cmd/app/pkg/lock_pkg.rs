use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Args;
use std::error::Error;

#[derive(Args, Debug)]
pub struct LockPkgCommand {
    /// Package name to lock
    #[arg(value_hint = clap::ValueHint::Other)]
    package_name: Option<String>,
}

impl AppCommand for LockPkgCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let package_name = detect_target_package_name!(&self.package_name);

        let result = app
            .build_client()?
            .sql_scripts()
            .lock_package(package_name)?;

        eprintln!("Rows affected: {}", result);

        Ok(())
    }
}
