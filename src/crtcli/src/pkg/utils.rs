use crate::pkg::json::{
    PkgJsonWrapper, PkgJsonWrapperCreateError, PkgPackageDescriptorJsonWrapper,
};
use crate::pkg::*;
use anstyle::{AnsiColor, Color, Style};
use flate2::read::GzDecoder;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};
use walkdir::WalkDir;
use zip::ZipArchive;
use zip::unstable::LittleEndianReadExt;

pub fn walk_over_package_dir(
    pkg_folder: &Path,
) -> impl Iterator<Item = walkdir::Result<walkdir::DirEntry>> + use<> {
    let valid_folders = paths::PKG_FOLDERS.map(|f| pkg_folder.join(f));
    let valid_files = [pkg_folder.join(paths::PKG_DESCRIPTOR_FILE)];

    valid_files
        .into_iter()
        .chain(valid_folders)
        .filter(|x| x.exists())
        .flat_map(|x| WalkDir::new(x).into_iter())
}

#[derive(Error, Debug)]
pub enum WalkOverPackageFilesContentError {
    #[error("unable to access folder: {0}")]
    FolderAccess(#[from] walkdir::Error),

    #[error("unable to access file {path}: {source}")]
    FileAccess {
        path: PathBuf,

        #[source]
        source: std::io::Error,
    },
}

pub fn walk_over_package_files(
    pkg_folder: &Path,
) -> impl Iterator<Item = Result<PathBuf, walkdir::Error>> {
    walk_over_package_dir(pkg_folder)
        .filter(|e| e.as_ref().is_ok_and(|e| !e.file_type().is_dir()))
        .filter_map(move |f| -> Option<Result<PathBuf, walkdir::Error>> {
            match f {
                Err(err) => Some(Err(err)),
                Ok(file) => {
                    let file_path = file.path().to_path_buf();
                    let filename = file_path.strip_prefix(pkg_folder).unwrap();

                    if contains_hidden_path(filename) {
                        return None;
                    }

                    Some(Ok(file_path))
                }
            }
        })
}

pub fn walk_over_package_files_content(
    pkg_folder: &Path,
) -> impl Iterator<Item = Result<bundling::PkgGZipFile, WalkOverPackageFilesContentError>> {
    walk_over_package_files(pkg_folder)
        .map(|e| e.map_err(WalkOverPackageFilesContentError::FolderAccess))
        .map(move |f| {
            f.and_then(|f| {
                bundling::PkgGZipFile::open_fs_file_absolute(pkg_folder, &f).map_err(|err| {
                    WalkOverPackageFilesContentError::FileAccess {
                        path: f,
                        source: err,
                    }
                })
            })
        })
}

pub fn is_gzip_stream(reader: &mut (impl Read + Seek)) -> std::io::Result<bool> {
    let format = reader.read_u16_le()?;

    reader.seek_relative(-2)?;

    Ok(format == 0x8b1f)
}

pub async fn is_gzip_async_stream(
    reader: &mut (impl AsyncRead + AsyncSeek + Unpin),
) -> std::io::Result<bool> {
    let format = reader.read_u16_le().await?;

    reader.seek(SeekFrom::Current(-2)).await?;

    Ok(format == 0x8b1f)
}

#[derive(Error, Debug)]
pub enum GetPackageNameFromFolderError {
    #[error("cannot read package descriptor: {0}")]
    PkgJsonWrapperCreate(#[from] PkgJsonWrapperCreateError),

    #[error("descriptor was correctly read from folder, but filename was not found")]
    PackageNameIsNone,
}

pub fn get_package_name_from_current_dir() -> Result<String, GetPackageNameFromFolderError> {
    get_package_name_from_folder(&PathBuf::from("."))
}

pub fn get_package_name_from_folder(
    pkg_folder: &Path,
) -> Result<String, GetPackageNameFromFolderError> {
    let pkg_descriptor = PkgPackageDescriptorJsonWrapper::from(PkgJsonWrapper::from_file(
        &pkg_folder.join(paths::PKG_DESCRIPTOR_FILE),
    )?);

    pkg_descriptor
        .name()
        .ok_or(GetPackageNameFromFolderError::PackageNameIsNone)
        .map(|x| x.to_owned())
}

pub fn contains_hidden_path(path: &Path) -> bool {
    let str = path.to_string_lossy();

    str.starts_with('.') || str.contains(&format!("{sep}.", sep = std::path::MAIN_SEPARATOR))
}

#[derive(Debug, Error)]
pub enum GetPackageDescriptorFromReaderError {
    #[error("failed to read from reader: {0}")]
    Reader(#[from] std::io::Error),

    #[error("failed to get descriptor from package bundle as gzip: {0}")]
    AsGzipError(#[source] GetPackageDescriptorFromGzipReaderError),

    #[error("failed to read package bundle as zip: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("failed to get descriptor in {filename_in_zip} file in zip: {source}")]
    AsZipError {
        filename_in_zip: String,

        #[source]
        source: GetPackageDescriptorFromGzipReaderError,
    },
}

#[derive(Debug, Error)]
pub enum GetPackageDescriptorFromGzipReaderError {
    #[error("failed to read gzip: {0}")]
    Reader(#[from] std::io::Error),

    #[error("failed to decode gzip in package bundle: {0}")]
    Decoder(#[from] bundling::PkgGZipDecoderError),

    #[error("failed to find descriptor.json file in package gzip")]
    DescriptorNotFound,

    #[error("failed to parse gzip package descriptor: {0}")]
    Parsing(#[from] PkgJsonWrapperCreateError),
}

pub fn get_package_descriptors_from_package_reader(
    mut reader: &mut (impl Read + Seek),
) -> Result<Vec<PkgPackageDescriptorJsonWrapper>, GetPackageDescriptorFromReaderError> {
    let position = reader.stream_position()?;

    let mut results = vec![];

    if is_gzip_stream(reader)? {
        results.push(
            get_package_descriptor_as_gzip(&mut reader)
                .map_err(GetPackageDescriptorFromReaderError::AsGzipError)?,
        );
    } else {
        let mut zip =
            ZipArchive::new(&mut reader).map_err(GetPackageDescriptorFromReaderError::Zip)?;

        for i in 0..zip.len() {
            let mut gzip = zip
                .by_index(i)
                .map_err(GetPackageDescriptorFromReaderError::Zip)?;

            results.push(get_package_descriptor_as_gzip(&mut gzip).map_err(|err| {
                GetPackageDescriptorFromReaderError::AsZipError {
                    filename_in_zip: gzip.name().to_owned(),
                    source: err,
                }
            })?);
        }
    }

    reader.seek(SeekFrom::Start(position))?;

    return Ok(results);

    fn get_package_descriptor_as_gzip(
        reader: &mut impl Read,
    ) -> Result<PkgPackageDescriptorJsonWrapper, GetPackageDescriptorFromGzipReaderError> {
        let descriptor = bundling::PkgGZipDecoder::new(GzDecoder::new(reader))
            .filter_map(|f| -> Option<Result<_, _>> {
                match f {
                    Err(error) => Some(Err(error)),
                    Ok(file) => match file.filename == paths::PKG_DESCRIPTOR_FILE {
                        true => Some(Ok(file)),
                        false => None,
                    },
                }
            })
            .next();

        let descriptor =
            descriptor.ok_or(GetPackageDescriptorFromGzipReaderError::DescriptorNotFound)??;

        let descriptor =
            PkgPackageDescriptorJsonWrapper::from(PkgJsonWrapper::new(&descriptor.content)?);

        Ok(descriptor)
    }
}

pub fn cmp_file_content_and_apply_with_log(
    file_path: &Path,
    relative_file_path: &str,
    source_content: Option<Vec<u8>>,
    pending_content: Option<Vec<u8>>,
    check_only: bool,
    mut stdout: impl Write,
) -> Result<bool, std::io::Error> {
    match (source_content, pending_content) {
        (Some(source_content), Some(pending_content)) if source_content != pending_content => {
            if check_only {
                writeln!(stdout, "\tto change:\t{}", relative_file_path).unwrap();
            } else {
                std::fs::write(file_path, pending_content)?;

                writeln!(stdout, "\tmodified:\t{}", relative_file_path).unwrap();
            }

            Ok(true)
        }
        (Some(_), None) => {
            if check_only {
                writeln!(
                    stdout,
                    "{style}\tto delete:\t{}{style:#}",
                    relative_file_path,
                    style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)))
                )
                .unwrap();
            } else {
                std::fs::remove_file(file_path)?;

                writeln!(
                    stdout,
                    "{style}\tdeleted:\t{}{style:#}",
                    relative_file_path,
                    style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)))
                )
                .unwrap();
            }

            Ok(true)
        }
        (None, Some(pending_content)) => {
            if check_only {
                writeln!(stdout, "\tto create:\t{}", relative_file_path).unwrap();
            } else {
                std::fs::write(file_path, pending_content)?;

                writeln!(stdout, "\tcreated:\t{}", relative_file_path).unwrap();
            }

            Ok(true)
        }
        _ => Ok(false),
    }
}
