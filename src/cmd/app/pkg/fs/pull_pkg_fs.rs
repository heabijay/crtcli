use crate::cmd::app::pkg::fs::prepare_pkg_fs_folder;
use crate::cmd::app::{AppCommand, AppCommandArgs};
use crate::cmd::cli::CliCommand;
use crate::pkg::utils::get_package_name_from_folder;
use anstyle::Style;
use clap::Args;
use std::error::Error;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct PullPkgFsCommand {
    /// Package folder where package is already pulled previously (default: current directory)
    /// (Sample: Terrasoft.Configuration/Pkg/.../)
    #[arg(long, value_hint = clap::ValueHint::DirPath)]
    package_folder: Option<PathBuf>,

    #[command(flatten)]
    apply_features: Option<crate::cmd::pkg::PkgApplyFeatures>,
}

impl AppCommand for PullPkgFsCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let package_folder = match &self.package_folder {
            Some(f) => f,
            None => &std::env::current_dir()?,
        };

        let package_name = get_package_name_from_folder(package_folder)?;

        prepare_pkg_fs_folder(package_folder)?;

        crate::cmd::app::fs::pull_fs::PullFsCommand {
            packages: Some(vec![package_name.clone()]),
        }
        .run(app)?;

        eprintln!(
            "{bold}{package_name}{bold:#} package pulled successfully!",
            bold = Style::new().bold()
        );

        crate::cmd::pkg::apply::ApplyCommand {
            package_folder: package_folder.to_owned(),
            file: None,
            apply_features: self.apply_features.clone(),
            no_feature_present_warning_disabled: true,
        }
        .run()?;

        Ok(())
    }
}
