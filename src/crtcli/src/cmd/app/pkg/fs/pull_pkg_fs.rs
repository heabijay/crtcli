use crate::app::CrtClient;
use crate::cfg::WorkspaceConfig;
use crate::cmd::app::AppCommand;
use crate::cmd::app::pkg::fs::prepare_pkg_fs_folder;
use crate::cmd::cli::{CliCommand, CommandResult};
use crate::cmd::pkg::WorkspaceConfigCmdPkgExt;
use crate::pkg::utils::get_package_name_from_folder;
use clap::Args;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct PullPkgFsCommand {
    /// Packages folders where package was already pulled previously (default: packages folders from ./workspace.crtcli.toml or current directory)
    /// (Sample: Terrasoft.Configuration/Pkg/.../)
    #[arg(long = "package-folder", value_hint = clap::ValueHint::DirPath)]
    packages_folders: Vec<PathBuf>,

    #[command(flatten)]
    apply_features: Option<crate::pkg::transforms::PkgApplyFeatures>,

    #[command(flatten)]
    apply_post_features: Option<crate::pkg::transforms::post::PkgApplyPostFeatures>,
}

impl AppCommand for PullPkgFsCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let packages_folders = if self.packages_folders.is_empty() {
            &WorkspaceConfig::load_default_from_current_dir()?
                .packages_or_print_error()?
                .iter()
                .map(|p| p.path().to_path_buf())
                .collect()
        } else {
            &self.packages_folders
        };

        let mut packages_names = Vec::with_capacity(packages_folders.len());

        for package_folder in packages_folders {
            packages_names.push(get_package_name_from_folder(package_folder)?);

            prepare_pkg_fs_folder(package_folder)?;
        }

        crate::cmd::app::fs::pull_fs::PullFsCommand {
            packages: packages_names,
        }
        .run(Arc::clone(&client))
        .await?;

        crate::cmd::pkg::apply::ApplyCommand {
            packages_folders: packages_folders.to_owned(),
            file: None,
            apply_features: self.apply_features.clone(),
            apply_post_features: self.apply_post_features.clone(),
            check_only: false,
            nothing_to_do_message_disabled: true,
            no_feature_present_warning_disabled: true,
        }
        .run()?;

        Ok(())
    }
}
