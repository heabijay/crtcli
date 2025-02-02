use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
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

    /// Output path where the downloaded package archive will be saved (default: current directory & auto-generated name)
    ///
    /// - If a directory is provided, the archive will be saved there with an auto-generated name.
    /// - If a file path is provided, the archive will be saved with that name.
    /// - If not specified, the archive will be saved in the current directory with an auto-generated name.
    #[arg(short, long, value_name = "PATH", value_hint = clap::ValueHint::AnyPath)]
    #[clap(verbatim_doc_comment)]
    output: Option<PathBuf>,
}

impl AppCommand for DownloadPkgCommand {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        let output_path = match &self.output {
            Some(output_path) => output_path,
            None => &PathBuf::from("."),
        };

        let packages = if self.packages.is_empty() {
            &vec![detect_target_package_name!()]
        } else {
            &self.packages
        };

        let output_path = output_has_filename_or!(output_path, {
            let default_filename = match packages.len() {
                1 => packages.iter().next().unwrap(),
                _ => "Packages",
            };

            &crate::cmd::utils::get_next_filename_if_exists(output_path.join(
                crate::cmd::utils::generate_zip_package_filename(default_filename),
            ))
        });

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

        let mut file = File::create(output_path)?;

        std::io::copy(&mut result, &mut file)?;

        progress.finish_and_clear();

        println!("{}", output_path.display());

        Ok(())
    }
}
