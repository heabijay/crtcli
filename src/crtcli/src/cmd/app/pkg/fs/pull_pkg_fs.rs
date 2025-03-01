use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::app::pkg::fs::prepare_pkg_fs_folder;
use crate::cmd::cli::{CliCommand, CommandResult};
use crate::pkg::utils::get_package_name_from_folder;
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::Args;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct PullPkgFsCommand {
    /// Package folder where package is already pulled previously (default: current directory)
    /// (Sample: Terrasoft.Configuration/Pkg/.../)
    #[arg(long, value_hint = clap::ValueHint::DirPath)]
    package_folder: Option<PathBuf>,

    #[command(flatten)]
    apply_features: Option<crate::cmd::pkg::PkgApplyFeatures>,
}

#[async_trait]
impl AppCommand for PullPkgFsCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let package_folder = match &self.package_folder {
            Some(package_folder) => package_folder,
            None => &PathBuf::from("."),
        };

        let package_name = get_package_name_from_folder(package_folder)?;

        prepare_pkg_fs_folder(package_folder)?;

        crate::cmd::app::fs::pull_fs::PullFsCommand {
            packages: vec![package_name.clone()],
        }
        .run(Arc::clone(&client))
        .await?;

        eprintln!(
            "{green}✔ Package {green_bold}{package_name}{green_bold:#}{green} successfully pulled to filesystem from {green_bold}{url}{green_bold:#}{green}!{green:#}",
            green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            green_bold = Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::Green)))
                .bold(),
            url = client.base_url(),
        );

        crate::cmd::pkg::apply::ApplyCommand {
            package_folder: package_folder.to_owned(),
            file: None,
            apply_features: self.apply_features.clone(),
            check_only: false,
            no_feature_present_warning_disabled: true,
        }
        .run()?;

        Ok(())
    }
}
