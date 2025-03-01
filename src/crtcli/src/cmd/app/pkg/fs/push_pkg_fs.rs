use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::app::pkg::fs::prepare_pkg_fs_folder;
use crate::cmd::cli::CommandResult;
use crate::pkg::utils::get_package_name_from_folder;
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::Args;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct PushPkgFsCommand {
    /// Package folder where package is already pulled previously (default: current directory)
    /// (Sample: Terrasoft.Configuration/Pkg/.../)
    #[arg(long, value_hint = clap::ValueHint::DirPath)]
    package_folder: Option<PathBuf>,

    /// Compile package in Creatio after successful push
    #[arg(short, long)]
    compile_package: bool,

    /// Restart application after successful push (and package compilation in Creatio)
    #[arg(short, long)]
    restart: bool,
}

#[async_trait]
impl AppCommand for PushPkgFsCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let destination_folder = match &self.package_folder {
            Some(package_folder) => package_folder,
            None => &PathBuf::from("."),
        };

        let package_name = get_package_name_from_folder(destination_folder)?;

        prepare_pkg_fs_folder(destination_folder)?;

        crate::cmd::app::fs::push_fs::PushFsCommand {
            packages: vec![package_name.clone()],
        }
        .run(Arc::clone(&client))
        .await?;

        eprintln!(
            "{green}✔ Package {green_bold}{package_name}{green_bold:#}{green} successfully pushed from filesystem to {green_bold}{url}{green_bold:#}{green}!{green:#}",
            green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            green_bold = Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::Green)))
                .bold(),
            url = client.base_url(),
        );

        if self.compile_package {
            crate::cmd::app::pkg::compile_pkg::CompilePkgCommand {
                package_name: Some(package_name),
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
