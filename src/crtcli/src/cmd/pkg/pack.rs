use crate::cmd::cli::{CliCommand, CommandResult};
use crate::pkg::bundling::packer::*;
use clap::{Args, ValueEnum};
use std::path::PathBuf;
use thiserror::Error;
use zip::CompressionMethod;

#[derive(Debug, Args)]
pub struct PackCommand {
    /// Source folders containing packages to be packaged (default: current directory)
    #[arg(value_delimiter = ',', value_hint = clap::ValueHint::DirPath)]
    packages_folders: Vec<PathBuf>,

    /// Output path where the output package archive will be saved (default: current directory & auto-generated name)
    ///
    /// - If a directory is provided, the archive will be saved there with an auto-generated name.
    /// - If a file path is provided, the archive will be saved with that name.
    /// - If not specified, the archive will be saved in the current directory with an auto-generated name.
    #[arg(short, long, value_name = "PATH", value_hint = clap::ValueHint::AnyPath)]
    #[clap(verbatim_doc_comment)]
    output: Option<PathBuf>,

    #[arg(long, default_value = "zip")]
    format: PackFormat,

    #[arg(long, default_value = "fast")]
    compression: PackCompression,
}

#[derive(Debug, Clone, Eq, PartialEq, ValueEnum)]
pub enum PackFormat {
    #[value(alias = "gz")]
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
    #[error("failed to write output package bundle: {0}")]
    WriteBundleFile(#[from] std::io::Error),

    #[error(
        "cannot pack multiple packages into a gzip file, please use the zip format or pack only one package."
    )]
    MultiplePackagesIntoGzip,
}

impl CliCommand for PackCommand {
    fn run(self) -> CommandResult {
        let packages_folders = if self.packages_folders.is_empty() {
            &vec![PathBuf::from(".")]
        } else {
            &self.packages_folders
        };

        let output_path = match &self.output {
            Some(output_path) => output_path,
            None => &PathBuf::from("."),
        };

        let output_path = output_has_filename_or!(output_path, {
            let filename = if packages_folders.len() == 1 {
                let pkg_name =
                    crate::pkg::utils::get_package_name_from_folder(&packages_folders[0])?;

                match self.format {
                    PackFormat::Gzip => &format!("{pkg_name}.gz"),
                    PackFormat::Zip => &crate::cmd::utils::generate_zip_package_filename(&pkg_name),
                }
            } else {
                match self.format {
                    PackFormat::Zip => {
                        &crate::cmd::utils::generate_zip_package_filename("Packages")
                    }
                    PackFormat::Gzip => "Packages.gz",
                }
            };

            &crate::cmd::utils::get_next_filename_if_exists(output_path.join(filename))
        });

        let gzip_config = GZipPackageFromFolderPackerConfig {
            compression: match self.compression {
                PackCompression::Fast => Some(flate2::Compression::fast()),
                PackCompression::Normal => Some(flate2::Compression::default()),
                PackCompression::Best => Some(flate2::Compression::best()),
            },
        };

        let zip_config = ZipPackageFromFolderPackerConfig {
            gzip_config,
            zip_compression_method: match self.compression {
                PackCompression::Fast => Some(CompressionMethod::Stored),
                PackCompression::Normal => None,
                PackCompression::Best => Some(CompressionMethod::Deflated),
            },
        };

        let gzip_config = &zip_config.gzip_config;

        let file = std::fs::File::create(output_path)?;

        match self.format {
            PackFormat::Gzip => {
                if packages_folders.len() > 1 {
                    Err(PackCommandError::MultiplePackagesIntoGzip)?
                }

                pack_gzip_package_from_folder(&packages_folders[0], file, gzip_config)?
            }
            PackFormat::Zip => pack_zip_package_from_folders(packages_folders, file, &zip_config)?,
        }

        println!("{}", output_path.display());

        Ok(())
    }
}
