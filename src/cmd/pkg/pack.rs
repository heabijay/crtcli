use crate::cmd::cli::CliCommand;
use crate::pkg::bundling::packer::*;
use clap::{Args, ValueEnum};
use std::error::Error;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Args)]
pub struct PackCommand {
    /// Source folder containing the package to be packaged
    #[arg(value_hint = clap::ValueHint::DirPath)]
    package_folder: PathBuf,

    /// Destination folder where the output package archive will be saved (defaults to the current directory)
    #[arg(short = 'f', long, value_hint = clap::ValueHint::DirPath)]
    output_folder: Option<PathBuf>,

    /// Filename of the output package archive file (optional, will be auto-generated if not specified)
    #[arg(short = 'n', long, value_hint = clap::ValueHint::FilePath)]
    output_filename: Option<String>,

    #[arg(long, default_value = "zip")]
    format: PackFormat,

    #[arg(long, default_value = "fast")]
    compression: PackCompression,
}

#[derive(Debug, Clone, Eq, PartialEq, ValueEnum)]
pub enum PackFormat {
    Gzip,
    Zip,
}

#[derive(Debug, Clone, Eq, PartialEq, ValueEnum)]
pub enum PackCompression {
    Fast,
    Normal,
    Best,
}

#[derive(Error, Debug)]
enum PackCommandError {
    #[error("failed to get valid current directory (also you can specify output_folder arg): {0}")]
    GetCurrentDir(#[source] std::io::Error),

    #[error("failed to write output package bundle: {0}")]
    WriteBundleFile(#[from] std::io::Error),
}

impl CliCommand for PackCommand {
    fn run(self) -> Result<(), Box<dyn Error>> {
        let output_folder = match self.output_folder {
            Some(path) => path,
            None => std::env::current_dir().map_err(PackCommandError::GetCurrentDir)?,
        };

        let output_filename = match &self.output_filename {
            Some(filename) => filename,
            None => {
                let pkg_name =
                    crate::pkg::utils::get_package_name_from_folder(&self.package_folder)?;

                match self.format {
                    PackFormat::Gzip => &format!("{pkg_name}.gz"),
                    PackFormat::Zip => &crate::cmd::utils::generate_zip_package_filename(&pkg_name),
                }
            }
        };

        let output_path = output_folder.join(output_filename);
        let output_path = match &self.output_filename.is_none() {
            true => crate::cmd::utils::get_next_filename_if_exists(output_path),
            false => output_path,
        };

        let gzip_config = GZipPackageFromFolderPackerConfig {
            compression: match self.compression {
                PackCompression::Fast => Some(flate2::Compression::fast()),
                PackCompression::Normal => Some(flate2::Compression::default()),
                PackCompression::Best => Some(flate2::Compression::best()),
            },
        };

        let zip_config = ZipPackageFromFolderPackerConfig {
            gzip_config,
            zip_compression_method: None,
        };

        let gzip_config = &zip_config.gzip_config;

        let file = std::fs::File::create(&output_path)?;

        match self.format {
            PackFormat::Gzip => {
                pack_gzip_package_from_folder(&self.package_folder, file, gzip_config)?
            }
            PackFormat::Zip => {
                pack_single_zip_package_from_folder(&self.package_folder, file, &zip_config)?
            }
        }

        println!("{}", output_path.display());

        Ok(())
    }
}
