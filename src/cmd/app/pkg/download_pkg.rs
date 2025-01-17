use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::utils::{generate_zip_package_filename, get_next_filename_if_exists};
use anstyle::Style;
use clap::Args;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

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

impl AppCommand for DownloadPkgCommand {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        let output_folder = match &self.output_folder {
            Some(output_folder) => output_folder,
            None => &PathBuf::from("."),
        };

        let packages = if self.packages.is_empty() {
            &vec![detect_target_package_name!()]
        } else {
            &self.packages
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

        let progress = spinner!(
            "Downloading {bold}{target}{bold:#} {target_label} from {bold}{url}{bold:#}",
            target = packages.join(", "),
            target_label = match packages.len() {
                0 | 1 => "package",
                _ => "packages",
            },
            bold = Style::new().bold(),
            url = client.base_url()
        );

        let mut result = client
            .package_installer_service()
            .get_zip_packages(&packages)?;

        let mut file = File::create(&output_path)?;

        std::io::copy(&mut result, &mut file)?;

        progress.finish_and_clear();

        println!("{}", output_path.display());

        Ok(())
    }
}
