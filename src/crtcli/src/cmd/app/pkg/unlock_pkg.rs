use crate::app::CrtClient;
use crate::cfg::WorkspaceConfig;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use crate::cmd::pkg::WorkspaceConfigCmdPkgExt;
use anstyle::Style;
use clap::Args;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct UnlockPkgCommand {
    /// A space-separated or comma-separated list of package names to unlock (default: packages names from ./workspace.crtcli.toml or ./descriptor.json)
    #[arg(value_delimiter = ',', value_hint = clap::ValueHint::Other)]
    package_names: Vec<String>,
}

impl AppCommand for UnlockPkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let package_names = if self.package_names.is_empty() {
            &WorkspaceConfig::load_default_from_current_dir()?
                .packages_or_print_error()?
                .iter()
                .map(|p| p.package_name().map(|x| x.into_owned()))
                .collect::<Result<Vec<String>, _>>()?
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
