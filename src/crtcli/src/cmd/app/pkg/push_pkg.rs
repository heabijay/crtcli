use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use crate::cmd::app::pkg::DetectTargetPackageNameError;
use crate::cmd::app::pkg::install_pkg::*;
use crate::cmd::cli::{CommandDynError, CommandResult};
use crate::pkg::bundling::packer::*;
use crate::pkg::utils::get_package_name_from_folder;
use clap::Args;
use flate2::Compression;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct PushPkgCommand {
    /// Folders containing packages to be packed and installed (default: current directory)
    #[arg(value_name = "SOURCE_FOLDERS", value_hint = clap::ValueHint::DirPath)]
    source_folders: Vec<PathBuf>,

    #[command(flatten)]
    install_pkg_options: InstallPkgCommandOptions,
}

#[derive(Debug, Error)]
pub enum PushPkgCommandError {
    #[error("{0}")]
    DetectPackageName(#[from] DetectTargetPackageNameError),

    #[error("cannot pack gzip package: {0}")]
    PackGzipPackage(#[from] PackGzipPackageFromFolderError),

    #[error("cannot pack zip package: {0}")]
    PackZipPackage(#[from] PackZipPackageFromFolderError),

    #[error("package installation failed: {0}")]
    InstallPackage(#[from] InstallPkgCommandError),
}

impl AppCommand for PushPkgCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let source_folder: &[PathBuf] = if self.source_folders.is_empty() {
            &[PathBuf::from(".")]
        } else {
            &self.source_folders
        };

        let (package_filename, package_content) = match source_folder.len() {
            1 => pack_folder_as_gzip(source_folder.iter().next().unwrap())?,
            _ => pack_folders_as_zip(source_folder)?,
        };

        install_package_from_stream_command(
            client,
            Cursor::new(package_content),
            &package_filename,
            &self.install_pkg_options,
        )
        .await
        .map_err(PushPkgCommandError::InstallPackage)?;

        return Ok(());

        fn pack_folder_as_gzip(folder: &Path) -> Result<(String, Vec<u8>), CommandDynError> {
            let package_name = get_package_name_from_folder(folder)?;
            let mut package_gzip = vec![];

            pack_gzip_package_from_folder(
                folder,
                &mut package_gzip,
                &GZipPackageFromFolderPackerConfig {
                    compression: Some(Compression::fast()),
                },
            )
            .map_err(PushPkgCommandError::PackGzipPackage)?;

            Ok((format!("{package_name}.gz"), package_gzip))
        }

        fn pack_folders_as_zip(
            source_folders: &[PathBuf],
        ) -> Result<(String, Vec<u8>), CommandDynError> {
            let mut package_zip_cursor = Cursor::new(vec![]);

            pack_zip_package_from_folders(
                source_folders,
                &mut package_zip_cursor,
                &ZipPackageFromFolderPackerConfig {
                    gzip_config: GZipPackageFromFolderPackerConfig {
                        compression: Some(Compression::fast()),
                    },
                    zip_compression_method: Some(zip::CompressionMethod::Stored),
                },
            )
            .map_err(PushPkgCommandError::PackZipPackage)?;

            Ok(("Packages.zip".to_owned(), package_zip_cursor.into_inner()))
        }
    }
}
