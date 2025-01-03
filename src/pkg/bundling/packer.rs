use crate::pkg::bundling::{PkgGZipEncoder, PkgGZipEncoderError};
use crate::pkg::utils::{walk_over_package_files_content, WalkOverPackageFilesContentError};
use flate2::Compression;
use std::io::{Seek, Write};
use std::path::Path;
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Error, Debug)]
pub enum PackGzipPackageFromFolderError {
    #[error("unable to walk over package files: {0}")]
    WalkOverPackageFilesContent(#[from] WalkOverPackageFilesContentError),

    #[error("error during encoding gzip: {0}")]
    PkgGZipEncoder(#[from] PkgGZipEncoderError),
}

#[derive(Error, Debug)]
pub enum PackZipPackageFromFolderError {
    #[error("unable to access folder: {0}")]
    FolderAccess(#[from] walkdir::Error),

    #[error("failed to detect package name from package folder: {0}")]
    DetectPackageName(#[from] crate::pkg::utils::GetPackageNameFromFolderError),

    #[error("zip error occurred: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("failed to create gzip package: {0}")]
    PackGzipPackageFromFolder(#[from] PackGzipPackageFromFolderError),
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct GZipPackageFromFolderPackerConfig {
    pub compression: Option<Compression>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ZipPackageFromFolderPackerConfig {
    pub zip_compression_method: Option<CompressionMethod>,
    pub gzip_config: GZipPackageFromFolderPackerConfig,
}

pub fn pack_gzip_package_from_folder(
    pkg_folder: &Path,
    gzip_writer: impl Write,
    config: &GZipPackageFromFolderPackerConfig,
) -> Result<(), PackGzipPackageFromFolderError> {
    let mut encoder = PkgGZipEncoder::new(gzip_writer, config.compression);

    for pkg_file in walk_over_package_files_content(pkg_folder.to_path_buf()) {
        encoder.write(&pkg_file?)?;
    }

    Ok(())
}

pub fn pack_zip_package_from_folders<P: AsRef<Path>>(
    pkg_folders: impl AsRef<[P]>,
    zip_writer: impl Write + Seek,
    config: &ZipPackageFromFolderPackerConfig,
) -> Result<(), PackZipPackageFromFolderError> {
    let mut zip = ZipWriter::new(zip_writer);
    let zip_file_options = SimpleFileOptions::default().compression_method(
        config
            .zip_compression_method
            .unwrap_or(CompressionMethod::Stored),
    );

    for pkg_folder in pkg_folders.as_ref() {
        let filename = format!(
            "{pkg_name}.gz",
            pkg_name = crate::pkg::utils::get_package_name_from_folder(pkg_folder.as_ref())?
        );

        zip.start_file(filename, zip_file_options)?;

        pack_gzip_package_from_folder(pkg_folder.as_ref(), &mut zip, &config.gzip_config)?;
    }

    Ok(())
}

pub fn pack_single_zip_package_from_folder(
    pkg_folder: impl AsRef<Path>,
    zip_writer: impl Write + Seek,
    config: &ZipPackageFromFolderPackerConfig,
) -> Result<(), PackZipPackageFromFolderError> {
    pack_zip_package_from_folders(&[pkg_folder], zip_writer, config)
}
