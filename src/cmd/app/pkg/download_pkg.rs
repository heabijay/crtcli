use crate::cmd::app::{AppCommand, AppCommandArgs};
use crate::cmd::utils::{generate_zip_package_filename, get_next_filename_if_exists};
use clap::Args;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct DownloadPkgCommand {
    /// A space-separated or comma-separated list of package names to download. Example: "CrtBase,CrtCore"
    #[arg(value_delimiter = ',', value_hint = clap::ValueHint::Other)]
    packages: Vec<String>,

    /// Directory where the downloaded package archive will be saved (default: current directory)
    #[arg(short = 'f', long, value_hint = clap::ValueHint::DirPath)]
    output_folder: Option<PathBuf>,

    /// Name of the output zip file (optional, will be auto-generated if not specified)
    #[arg(short = 'n', long, value_hint = clap::ValueHint::FilePath)]
    output_filename: Option<String>,
}

#[derive(Error, Debug)]
enum DownloadPkgCommandError {
    #[error("failed to get valid current directory (also you can specify output_folder arg): {0}")]
    GetCurrentDir(#[source] std::io::Error),
}

impl AppCommand for DownloadPkgCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let output_folder = match &self.output_folder {
            Some(path) => path,
            None => &std::env::current_dir().map_err(DownloadPkgCommandError::GetCurrentDir)?,
        };

        let packages = match self.packages.len() {
            0 => &vec![detect_target_package_name!()],
            _ => &self.packages,
        };

        let default_filename = match packages.len() {
            1 => packages.iter().next().unwrap(),
            _ => "Packages",
        };

        let output_filename = match &self.output_filename {
            Some(filename) => filename,
            None => &generate_zip_package_filename(default_filename),
        };

        let output_path = output_folder.join(output_filename);
        let output_path = match self.output_filename.is_none() {
            true => get_next_filename_if_exists(output_path),
            false => output_path,
        };

        let mut result = app
            .build_client()?
            .package_installer_service()
            .get_zip_packages(&packages.iter().map(String::as_str).collect::<Vec<&str>>())?;

        let mut file = File::create(&output_path)?;

        std::io::copy(&mut result, &mut file)?;

        println!("{}", output_path.display());

        Ok(())
    }
}
