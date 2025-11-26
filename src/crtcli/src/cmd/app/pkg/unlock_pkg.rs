use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use crate::pkg::utils::get_package_name_from_current_dir;
use anstyle::Style;
use clap::Args;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct UnlockPkgCommand {
    /// A space-separated or comma-separated list of package names to unlock
    #[arg(value_delimiter = ',', value_hint = clap::ValueHint::Other)]
    package_names: Vec<String>,
}

impl AppCommand for UnlockPkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let package_names = if self.package_names.is_empty() {
            &vec![get_package_name_from_current_dir()?]
        } else {
            &self.package_names
        };

        for package_name in package_names {
            let result = client.sql_scripts().unlock_package(package_name).await?;

            eprintln!(
                "Unlocking {bold}{package_name}{bold:#} package -> Rows affected: {}",
                result,
                bold = Style::new().bold()
            );
        }

        Ok(())
    }
}
