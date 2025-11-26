use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::app::pkg::fs::prepare_pkg_fs_folder;
use crate::cmd::cli::CommandResult;
use crate::pkg::utils::get_package_name_from_folder;
use clap::Args;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct PushPkgFsCommand {
    /// Packages folders where package was already pulled previously (default: current directory)
    /// (Sample: Terrasoft.Configuration/Pkg/.../)
    #[arg(long = "package-folder", value_hint = clap::ValueHint::DirPath)]
    packages_folders: Vec<PathBuf>,

    /// Compile package in Creatio after successful push
    #[arg(short, long)]
    compile_package: bool,

    /// Restart application after successful push (and package compilation in Creatio)
    #[arg(short, long)]
    restart: bool,
}

impl AppCommand for PushPkgFsCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let packages_folders = if self.packages_folders.is_empty() {
            &vec![PathBuf::from(".")]
        } else {
            &self.packages_folders
        };

        let mut packages_names = Vec::with_capacity(packages_folders.len());

        for package_folder in packages_folders {
            packages_names.push(get_package_name_from_folder(package_folder)?);

            prepare_pkg_fs_folder(package_folder)?;
        }

        crate::cmd::app::fs::push_fs::PushFsCommand {
            packages: packages_names.clone(),
        }
        .run(Arc::clone(&client))
        .await?;

        if self.compile_package {
            crate::cmd::app::pkg::compile_pkg::CompilePkgCommand {
                packages_names,
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
