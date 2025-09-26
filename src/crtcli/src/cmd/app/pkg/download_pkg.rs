use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use anstyle::Style;
use async_trait::async_trait;
use clap::Args;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct DownloadPkgCommand {
    /// A space-separated or comma-separated list of package names to download. Example: "CrtBase,CrtCore"
    #[arg(value_delimiter = ',', value_hint = clap::ValueHint::Other)]
    packages: Vec<String>,

    /// Output path where the downloaded package archive will be saved (default: current directory & auto-generated name) (Use '@-' or '-' value to write data to stdout)
    ///
    /// - If a directory is provided, the archive will be saved there with an auto-generated name.
    /// - If a file path is provided, the archive will be saved with that name.
    /// - If not specified, the archive will be saved in the current directory with an auto-generated name.
    #[arg(short, long, value_name = "PATH", value_hint = clap::ValueHint::AnyPath)]
    #[clap(verbatim_doc_comment)]
    output: Option<PathBuf>,
}

#[async_trait]
impl AppCommand for DownloadPkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let output_path = match &self.output {
            Some(output_path) => output_path,
            None => &PathBuf::from("."),
        };

        let packages = if self.packages.is_empty() {
            &vec![detect_target_package_name!()]
        } else {
            &self.packages
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
            .get_zip_packages(&packages)
            .await?;

        progress.finish_and_clear();

        match output_path.to_str() {
            Some("@-") | Some("-") => {
                tokio::io::copy(&mut result, &mut tokio::io::stdout()).await?;
            }
            _ => {
                let output_path = output_has_filename_or!(output_path, {
                    let default_filename = match packages.len() {
                        1 => packages.iter().next().unwrap(),
                        _ => "Packages",
                    };

                    &crate::cmd::utils::get_next_filename_if_exists(output_path.join(
                        crate::cmd::utils::generate_zip_package_filename(default_filename),
                    ))
                });

                let mut file = tokio::fs::File::create(output_path).await?;

                tokio::io::copy(&mut result, &mut file).await?;

                println!("{}", output_path.display());
            }
        }

        Ok(())
    }
}
