use crate::pkg::bundling::PkgGZipDecoder;
use crate::pkg::bundling::utils::{
    FolderIsEmptyValidationError, remove_dir_all_files_predicate, validate_folder_is_empty,
};
use crate::pkg::transforms::*;
use crate::pkg::utils::contains_hidden_path;
use anstyle::{AnsiColor, Color, Style};
use std::borrow::Cow;
use std::cell::LazyCell;
use std::collections::HashSet;
use std::io::{Read, Seek};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::ZipArchive;
use zip::result::ZipError;

#[derive(Error, Debug)]
pub enum ExtractGzipPackageError {
    #[error("destination path {0} is a file, not a folder")]
    DestinationPathIsFile(PathBuf),

    #[error("unable to access output folder {0}: {1}")]
    AccessOutputFolder(PathBuf, #[source] std::io::Error),

    #[error("{0}")]
    FolderIsNotEmpty(#[from] FolderIsEmptyValidationError),

    #[error("failure in decode package process: {0}")]
    PkgGZipDecoder(#[from] crate::pkg::bundling::PkgGZipDecoderError),

    #[error("error occurred in apply pkg file conversion/feature: {0}")]
    PkgFileTransformError(#[from] CombinedPkgFileTransformError),

    #[error("unable to extract parent folder or file destination path {0}")]
    GetParentFolderOrDestinationPath(PathBuf),

    #[error("unable to create out folder or file {0}: {1}")]
    CreateFolderOrFile(PathBuf, #[source] std::io::Error),

    #[error("failed to delete files during merge: {0}")]
    DeleteFilesDuringMerge(#[source] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ExtractSingleZipPackageError {
    #[error("unable to open zip file for reading: {0}")]
    OpenZipFileForReading(#[source] ZipError),

    #[error("unable to get gzip file in zip: {0}")]
    GetGZipInZip(#[source] ZipError),

    #[error(
        "multiple package in zip file was found when extracting single gzip package. Consider to specify package filename parameter or use extract_zip_package_to_folder method instead"
    )]
    MultiplePackageInZipFile,

    #[error("unable to extract gzip package ({filename}): {source}")]
    ExtractGZipPackage {
        filename: String,

        #[source]
        source: ExtractGzipPackageError,
    },
}

#[derive(Error, Debug)]
pub enum ExtractZipPackageError {
    #[error("{0}")]
    FolderIsNotEmpty(#[from] FolderIsEmptyValidationError),

    #[error("unable to open zip file for reading: {0}")]
    OpenZipFileForReading(#[source] ZipError),

    #[error("unable to get gzip file in zip: {0}")]
    GetGZipInZip(#[source] ZipError),

    #[error("unable to extract gzip package ({filename}): {source}")]
    ExtractGZipPackage {
        filename: String,
        #[source]
        source: ExtractGzipPackageError,
    },
}

#[derive(Default, Eq, PartialEq, Debug, Copy, Clone)]
pub enum FilesAlreadyExistsInFolderStrategy {
    #[default]
    ThrowError,
    Merge,
    SmartMerge,
}

#[derive(Default, Debug)]
pub struct PackageToFolderExtractorConfig {
    files_already_exists_in_folder_strategy: FilesAlreadyExistsInFolderStrategy,
    file_transform: CombinedPkgFileTransform,
    print_merge_log: bool,
}

impl PackageToFolderExtractorConfig {
    pub fn with_files_already_exists_in_folder_strategy(
        mut self,
        strategy: FilesAlreadyExistsInFolderStrategy,
    ) -> Self {
        self.files_already_exists_in_folder_strategy = strategy;
        self
    }

    pub fn with_transform(mut self, transform: CombinedPkgFileTransform) -> Self {
        self.file_transform = transform;
        self
    }

    pub fn print_merge_log(mut self, value: bool) -> Self {
        self.print_merge_log = value;
        self
    }
}

struct MergeContext {
    destination_folder: PathBuf,
    files: HashSet<PathBuf>,
}

impl MergeContext {
    pub fn new(destination_folder: PathBuf) -> Self {
        Self {
            destination_folder,
            files: HashSet::new(),
        }
    }

    pub fn new_if_needed(
        destination_folder: &Path,
        config: &PackageToFolderExtractorConfig,
    ) -> Option<Self> {
        match config.files_already_exists_in_folder_strategy {
            FilesAlreadyExistsInFolderStrategy::Merge
            | FilesAlreadyExistsInFolderStrategy::SmartMerge => {
                Some(Self::new(destination_folder.to_path_buf()))
            }
            FilesAlreadyExistsInFolderStrategy::ThrowError => None,
        }
    }

    pub fn execute_remove(
        self,
        config: &PackageToFolderExtractorConfig,
    ) -> Result<(), std::io::Error> {
        let pkg_folders = crate::pkg::paths::PKG_FOLDERS
            .iter()
            .map(|&p| self.destination_folder.join(p))
            .filter(|p| p.exists());

        for folder in pkg_folders {
            remove_dir_all_files_predicate(&folder, |f| {
                let path = f.path();
                let relative_path = path.strip_prefix(&self.destination_folder).unwrap();
                let result = !self.files.contains(path)
                    && !contains_hidden_path(relative_path)
                    && !check_pkg_file_content_equal(
                        config,
                        relative_path.to_str().unwrap(),
                        Some(LazyCell::new(|| {
                            Cow::Owned(std::fs::read(path).unwrap_or_default())
                        })),
                        None,
                    );

                if result && config.print_merge_log {
                    eprintln!(
                        "{style}\tdeleted:\t{}{style:#}",
                        relative_path.display(),
                        style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)))
                    );
                }

                result
            })?;
        }

        Ok(())
    }
}

pub fn extract_gzip_package_to_folder(
    gzip_reader: impl Read,
    destination_folder: &Path,
    config: &PackageToFolderExtractorConfig,
) -> Result<(), ExtractGzipPackageError> {
    prepare_destination_folder(destination_folder, config)?;

    let mut merge_ctx = MergeContext::new_if_needed(destination_folder, config);
    let decoder = PkgGZipDecoder::from(gzip_reader);

    for file in decoder {
        let file = file?;
        let filename = file.to_native_path_string().into_owned();
        let file_content = config.file_transform.transform(&filename, file.content)?;

        if file_content.is_none() {
            continue;
        }

        let file_content = file_content.unwrap();

        let destination_path = destination_folder.join(&filename);
        let destination_path_parent = destination_path.parent().ok_or_else(|| {
            ExtractGzipPackageError::GetParentFolderOrDestinationPath(
                destination_path.to_path_buf(),
            )
        })?;

        if !destination_path_parent.exists() {
            std::fs::create_dir_all(destination_path_parent).map_err(|err| {
                ExtractGzipPackageError::AccessOutputFolder(
                    destination_path_parent.to_path_buf(),
                    err,
                )
            })?;
        }

        if should_write_to_file(
            &filename,
            destination_path_parent,
            &destination_path,
            &file_content,
            config,
        )? {
            std::fs::write(&destination_path, &file_content).map_err(|err| {
                ExtractGzipPackageError::CreateFolderOrFile(destination_path.to_path_buf(), err)
            })?;
        }

        if let Some(x) = merge_ctx.as_mut() {
            x.files.insert(destination_path);
        }
    }

    merge_ctx.map(|ctx| {
        ctx.execute_remove(config)
            .map_err(ExtractGzipPackageError::DeleteFilesDuringMerge)
    });

    return Ok(());

    fn prepare_destination_folder(
        destination_folder: &Path,
        config: &PackageToFolderExtractorConfig,
    ) -> Result<(), ExtractGzipPackageError> {
        if destination_folder.is_file() {
            return Err(ExtractGzipPackageError::DestinationPathIsFile(
                destination_folder.to_path_buf(),
            ));
        }

        if config.files_already_exists_in_folder_strategy
            == FilesAlreadyExistsInFolderStrategy::ThrowError
        {
            validate_folder_is_empty(destination_folder)?;
        }

        if !destination_folder.exists() {
            std::fs::create_dir_all(destination_folder).map_err(|err| {
                ExtractGzipPackageError::AccessOutputFolder(destination_folder.to_path_buf(), err)
            })?;
        }

        Ok(())
    }

    fn should_write_to_file(
        relative_path: &str,
        destination_path_parent: &Path,
        destination_path: &Path,
        content: &[u8],
        config: &PackageToFolderExtractorConfig,
    ) -> Result<bool, ExtractGzipPackageError> {
        if destination_path.exists() {
            match config.files_already_exists_in_folder_strategy {
                FilesAlreadyExistsInFolderStrategy::ThrowError => {
                    Err(ExtractGzipPackageError::FolderIsNotEmpty(
                        FolderIsEmptyValidationError::FilesAlreadyExistsInFolder {
                            folder_path: destination_path_parent.to_path_buf(),
                        },
                    ))
                }
                FilesAlreadyExistsInFolderStrategy::Merge
                | FilesAlreadyExistsInFolderStrategy::SmartMerge => {
                    if check_pkg_file_content_equal(
                        config,
                        relative_path,
                        Some(LazyCell::new(|| {
                            Cow::Owned(std::fs::read(destination_path).unwrap_or_default())
                        })),
                        Some(content),
                    ) {
                        Ok(false)
                    } else {
                        if config.print_merge_log {
                            eprintln!("\tmodified:\t{relative_path}");
                        }

                        Ok(true)
                    }
                }
            }
        } else if check_pkg_file_content_equal(
            config,
            relative_path,
            None::<LazyCell<Cow<'_, [u8]>, fn() -> Cow<'static, [u8]>>>,
            Some(content),
        ) {
            Ok(false)
        } else {
            if config.print_merge_log {
                eprintln!(
                    "{style}\tcreated:\t{relative_path}{style:#}",
                    style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)))
                );
            }

            Ok(true)
        }
    }
}

pub fn extract_single_zip_package_to_folder(
    zip_reader: impl Read + Seek,
    destination_folder: &Path,
    package_name: Option<&str>,
    config: &PackageToFolderExtractorConfig,
) -> Result<(), ExtractSingleZipPackageError> {
    let mut zip =
        ZipArchive::new(zip_reader).map_err(ExtractSingleZipPackageError::OpenZipFileForReading)?;

    let gzip = match package_name {
        Some(package_name) => zip_get_file_by_package_name(&mut zip, package_name)
            .map_err(ExtractSingleZipPackageError::GetGZipInZip)?,
        None => {
            if zip.len() > 1 {
                return Err(ExtractSingleZipPackageError::MultiplePackageInZipFile);
            }

            zip.by_index(0)
                .map_err(ExtractSingleZipPackageError::GetGZipInZip)?
        }
    };

    let gzip_filename = gzip.name().to_owned();

    return extract_gzip_package_to_folder(gzip, destination_folder, config).map_err(|err| {
        ExtractSingleZipPackageError::ExtractGZipPackage {
            filename: gzip_filename,
            source: err,
        }
    });

    fn zip_get_file_by_package_name<'a, R: Read + Seek>(
        zip: &'a mut ZipArchive<R>,
        package_name: &str,
    ) -> Result<zip::read::ZipFile<'a, R>, ZipError> {
        let index = zip
            .index_for_name(package_name)
            .or_else(|| zip.index_for_name(&format!("{package_name}.gz")))
            .ok_or(ZipError::FileNotFound)?;

        zip.by_index(index)
    }
}

pub fn extract_zip_package_to_folder(
    reader: impl Read + Seek,
    destination_folder: &Path,
    config: &PackageToFolderExtractorConfig,
) -> Result<Vec<PathBuf>, ExtractZipPackageError> {
    let mut zip = ZipArchive::new(reader).map_err(ExtractZipPackageError::OpenZipFileForReading)?;

    if config.files_already_exists_in_folder_strategy
        == FilesAlreadyExistsInFolderStrategy::ThrowError
    {
        validate_folder_is_empty(destination_folder)?;
    }

    let mut package_folders = Vec::with_capacity(zip.len());

    for i in 0..zip.len() {
        let gzip = zip
            .by_index(i)
            .map_err(ExtractZipPackageError::GetGZipInZip)?;

        let gzip_filename = gzip
            .name()
            .strip_suffix(".gz")
            .unwrap_or(gzip.name())
            .to_owned();

        let package_folder = destination_folder.join(&gzip_filename);

        extract_gzip_package_to_folder(gzip, package_folder.as_path(), config).map_err(|err| {
            ExtractZipPackageError::ExtractGZipPackage {
                filename: gzip_filename,
                source: err,
            }
        })?;

        package_folders.push(package_folder);
    }

    Ok(package_folders)
}

fn check_pkg_file_content_equal<'a>(
    config: &PackageToFolderExtractorConfig,
    relative_path: &str,
    source: Option<LazyCell<Cow<'a, [u8]>, impl FnOnce() -> Cow<'a, [u8]>>>,
    target: Option<&[u8]>,
) -> bool {
    if source.is_none() && target.is_none() {
        return true;
    }

    if matches!(
        config.files_already_exists_in_folder_strategy,
        FilesAlreadyExistsInFolderStrategy::SmartMerge
    ) && let Some(result) = smart_equality_check(relative_path, &source, &target)
    {
        return result;
    }

    return match (target, source) {
        (Some(t), Some(s)) => s.deref().as_ref() == t,
        _ => false,
    };

    fn smart_equality_check<'a>(
        relative_path: &str,
        source: &Option<LazyCell<Cow<'a, [u8]>, impl FnOnce() -> Cow<'a, [u8]>>>,
        target: &Option<&[u8]>,
    ) -> Option<bool> {
        if crate::pkg::json::PKG_SCHEMAS_CS_PATH_REGEX.is_match(relative_path)
            && target.is_none_or(|x| x.is_empty())
            && source.as_ref().is_none_or(|x| x.is_empty())
        {
            return Some(true);
        }

        if crate::pkg::json::PKG_SCHEMAS_DESCRIPTOR_PATH_REGEX.is_match(relative_path)
            && let Some(target_content) = target
            && let Some(source_content) = source
        {
            let mut source = crate::pkg::json::PkgSchemasDescriptorJsonWrapper::from(
                crate::pkg::json::PkgJsonWrapper::new(source_content).unwrap(),
            );

            let mut target = crate::pkg::json::PkgSchemasDescriptorJsonWrapper::from(
                crate::pkg::json::PkgJsonWrapper::new(target_content).unwrap(),
            );

            *source.modified_on_utc_mut() = target.modified_on_utc_mut().clone();
            *source.caption_mut() = target.caption_mut().clone();

            if source
                .depends_on_mut()
                .as_array()
                .is_none_or(|x| x.is_empty())
                && target
                    .depends_on_mut()
                    .as_array()
                    .is_none_or(|x| x.is_empty())
            {
                *source.depends_on_mut() = target.depends_on_mut().clone();
            }

            return Some(source.deref() == target.deref());
        }

        None
    }
}
