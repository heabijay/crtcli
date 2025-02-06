use crate::app::CrtClient;
use crate::cmd::app::fs::print_fs_sync_result;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use anstyle::Style;
use async_trait::async_trait;
use clap::Args;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct PushFsCommand {
    /// A space-separated or comma-separated list of package names to push. Example: "CrtBase,CrtCore"
    #[arg(value_delimiter = ',', value_hint = clap::ValueHint::Other)]
    pub packages: Vec<String>,
}

#[async_trait]
impl AppCommand for PushFsCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let bold = Style::new().bold();

        let push_target_str = match &self.packages.len() {
            0 => "all packages",
            1 => &format!("{bold}{}{bold:#} package", &self.packages[0]),
            _ => &format!("{bold}{}{bold:#} packages", &self.packages.join(", ")),
        };

        let progress = spinner!(
            "Pushing {push_target_str} from filesystem to {bold}{url}{bold:#}",
            url = client.base_url()
        );

        let result = client
            .app_installer_service()
            .load_packages_to_db(match &self.packages.len() {
                0 => None,
                _ => Some(&self.packages),
            })
            .await?;

        progress.finish_and_clear();

        print_fs_sync_result(&result);

        Ok(())
    }
}
