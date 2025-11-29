use crate::app::CrtClient;
use crate::cfg::WorkspaceConfig;
use crate::cmd::app::AppCommand;
use crate::cmd::app::pkg::fs::prepare_pkg_fs_folder;
use crate::cmd::cli::CommandResult;
use crate::cmd::pkg::WorkspaceConfigCmdPkgExt;
use crate::pkg::utils::get_package_name_from_folder;
use clap::Args;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct PushPkgFsCommand {
    /// Package folders where package was already pulled previously (default: package folders from ./workspace.crtcli.toml or current directory)
    /// (Sample: Terrasoft.Configuration/Pkg/.../)
    #[arg(long = "package-folder", value_hint = clap::ValueHint::DirPath)]
    package_folders: Vec<PathBuf>,

    /// Compile package in Creatio after successful push
    #[arg(short, long)]
    compile_package: bool,

    /// Restart application after successful push (and package compilation in Creatio)
    #[arg(short, long)]
    restart: bool,
}

impl AppCommand for PushPkgFsCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let package_folders = if self.package_folders.is_empty() {
            &WorkspaceConfig::load_default_from_current_dir()?
                .packages_or_print_error()?
                .iter()
                .map(|p| p.path().to_path_buf())
                .collect()
        } else {
            &self.package_folders
        };

        let mut package_names = Vec::with_capacity(package_folders.len());

        for package_folder in package_folders {
            package_names.push(get_package_name_from_folder(package_folder)?);

            prepare_pkg_fs_folder(package_folder)?;
        }

        crate::cmd::app::fs::push_fs::PushFsCommand {
            packages: package_names.clone(),
        }
        .run(Arc::clone(&client))
        .await?;

        if self.compile_package {
            crate::cmd::app::pkg::compile_pkg::CompilePkgCommand {
                package_names,
                force_rebuild: false,
                restart: self.restart,
            }
            .run(client)
            .await?;
        } else if self.restart {
            crate::cmd::app::restart::RestartCommand.run(client).await?;
        }

        Ok(())
    }
}
