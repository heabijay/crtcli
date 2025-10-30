use crate::pkg::*;
use flate2::read::GzDecoder;
use std::io::{Read, Seek, SeekFrom};
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
    #[error("unable to read package descriptor: {0}")]
    PkgJsonWrapperCreate(#[from] json_wrappers::PkgJsonWrapperCreateError),

    #[error("descriptor was correctly read from folder, but filename was not found")]
    PackageNameIsNone,
}

pub fn get_package_name_from_current_dir() -> Result<String, GetPackageNameFromFolderError> {
    get_package_name_from_folder(&PathBuf::from("."))
}

pub fn get_package_name_from_folder(
    pkg_folder: &Path,
) -> Result<String, GetPackageNameFromFolderError> {
    let pkg_descriptor = crate::pkg::json_wrappers::PkgPackageDescriptorJsonWrapper::from(
        json_wrappers::PkgJsonWrapper::from_file(&pkg_folder.join(paths::PKG_DESCRIPTOR_FILE))?,
    );

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
    Parsing(#[from] json_wrappers::PkgJsonWrapperCreateError),
}

pub fn get_package_descriptors_from_package_reader(
    mut reader: &mut (impl Read + Seek),
) -> Result<Vec<json_wrappers::PkgPackageDescriptorJsonWrapper>, GetPackageDescriptorFromReaderError>
{
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
    ) -> Result<
        json_wrappers::PkgPackageDescriptorJsonWrapper,
        GetPackageDescriptorFromGzipReaderError,
    > {
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

        let descriptor = json_wrappers::PkgPackageDescriptorJsonWrapper::from(
            json_wrappers::PkgJsonWrapper::new(&descriptor.content)?,
        );

        Ok(descriptor)
    }
}
