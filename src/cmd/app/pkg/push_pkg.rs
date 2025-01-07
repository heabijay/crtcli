use crate::app::CrtClient;
use crate::cmd::app::pkg::install_pkg::*;
use crate::cmd::app::pkg::DetectTargetPackageNameError;
use crate::cmd::app::AppCommand;
use crate::pkg::bundling::packer::*;
use clap::Args;
use flate2::Compression;
use std::error::Error;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct PushPkgCommand {
    /// Folders containing packages to be packed and installed (default: current directory)
    #[arg(short = 's', long, value_name = "SOURCE_FOLDERS", value_hint = clap::ValueHint::DirPath)]
    source_folder: Option<Vec<PathBuf>>,

    #[command(flatten)]
    install_pkg_options: InstallPkgCommandOptions,
}

#[derive(Debug, Error)]
pub enum PushPkgCommandError {
    #[error(
        "failed to get valid current directory (also you can specify --package_folder arg): {0}"
    )]
    GetCurrentDir(#[source] std::io::Error),

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
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        let source_folder = match &self.source_folder {
            Some(f) => f,
            None => &vec![std::env::current_dir().map_err(PushPkgCommandError::GetCurrentDir)?],
        };

        let (package_filename, package_content) = match source_folder.len() {
            1 => pack_folder_as_gzip(source_folder.iter().next().unwrap())?,
            _ => pack_folders_as_zip(source_folder)?,
        };

        install_package_from_stream_command(
            client,
            std::io::Cursor::new(package_content),
            &package_filename,
            &self.install_pkg_options,
        )
        .map_err(PushPkgCommandError::InstallPackage)?;

        return Ok(());

        fn pack_folder_as_gzip(folder: &Path) -> Result<(String, Vec<u8>), Box<dyn Error>> {
            let package_name = detect_target_package_name!(None, folder);
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
            source_folders: &Vec<PathBuf>,
        ) -> Result<(String, Vec<u8>), Box<dyn Error>> {
            let mut package_zip_cursor = Cursor::new(vec![]);

            pack_zip_package_from_folders(
                source_folders,
                &mut package_zip_cursor,
                &ZipPackageFromFolderPackerConfig {
                    gzip_config: GZipPackageFromFolderPackerConfig {
                        compression: Some(Compression::fast()),
                    },
                    zip_compression_method: Some(zip::CompressionMethod::Deflated),
                },
            )
            .map_err(PushPkgCommandError::PackZipPackage)?;

            Ok(("Packages.zip".to_owned(), package_zip_cursor.into_inner()))
        }
    }
}
