use crate::cmd::app::pkg::fs::prepare_pkg_fs_folder;
use crate::cmd::app::{AppCommand, AppCommandArgs};
use crate::pkg::utils::get_package_name_from_folder;
use anstyle::{AnsiColor, Color, Style};
use clap::Args;
use std::error::Error;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct PushPkgFsCommand {
    /// Package folder where package is already pulled previously (default: current directory)
    /// (Sample: Terrasoft.Configuration/Pkg/.../)
    #[arg(long, value_hint = clap::ValueHint::DirPath)]
    package_folder: Option<PathBuf>,

    /// Compile package in Creatio after successful push
    #[arg(short, long)]
    compile_package_after_push: bool,

    /// Restart application after successful package compilation in Creatio
    #[arg(short, long)]
    restart_app_after_compile: bool,
}

impl AppCommand for PushPkgFsCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let destination_folder = match &self.package_folder {
            Some(f) => f,
            None => &std::env::current_dir()?,
        };

        let package_name = get_package_name_from_folder(destination_folder)?;

        prepare_pkg_fs_folder(destination_folder)?;

        let client = app.build_client()?;

        crate::cmd::app::fs::push_fs::PushFsCommand {
            packages: Some(vec![package_name.clone()]),
        }
        .run(app)?;

        eprintln!(
            "{green}✔ Package {green_bold}{package_name}{green_bold:#}{green} successfully pushed from filesystem to {green_bold}{url}{green_bold:#}{green}!{green:#}",
            green=Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            green_bold=Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))).bold(),
            url=client.base_url(),
        );

        if self.compile_package_after_push {
            crate::cmd::app::pkg::compile_pkg::CompilePkgCommand {
                package_name: Some(package_name),
                restart: self.restart_app_after_compile,
            }
            .run(app)?;
        }

        Ok(())
    }
}
