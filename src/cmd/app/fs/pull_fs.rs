use crate::cmd::app::fs::print_fs_sync_result;
use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Args;
use std::error::Error;

#[derive(Args, Debug)]
pub struct PullFsCommand {
    /// A space-separated or comma-separated list of package names to pull. Example: "CrtBase,CrtCore"
    #[arg(value_delimiter = ',', value_hint = clap::ValueHint::Other)]
    pub packages: Option<Vec<String>>,
}

impl AppCommand for PullFsCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let result = app
            .build_client()?
            .app_installer_service()
            .load_packages_to_fs(self.packages.as_ref())?;

        print_fs_sync_result(&result);

        Ok(())
    }
}
