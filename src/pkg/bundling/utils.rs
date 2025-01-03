use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum ReadAsciiStringWithLenError {
    #[error("reader failure: {0}")]
    Read(#[from] std::io::Error),

    #[error("invalid UTF-8 sequence at {index} byte - {byte:x} in {segment:?} (as str \"{}\")", String::from_utf8(segment.clone()).unwrap_or("[error]".to_owned()))]
    InvalidUtf8SequenceInAsciiString {
        index: usize,
        byte: u8,
        segment: Vec<u8>,
    },

    #[error("error while converting byte array to UTF-8 string")]
    StringFromUtf8(#[from] std::string::FromUtf8Error),
}

pub fn decode_ascii_string_from_byte_array(
    bytes: Vec<u8>,
) -> Result<String, ReadAsciiStringWithLenError> {
    let mut out = vec![0u8; bytes.len() / 2];

    for (index, &byte) in bytes.iter().enumerate() {
        if index % 2 == 0 {
            out[index / 2] = byte;
            continue;
        }

        if byte != 0 {
            return Err(
                ReadAsciiStringWithLenError::InvalidUtf8SequenceInAsciiString {
                    index,
                    byte,
                    segment: bytes.clone(),
                },
            );
        }
    }

    String::from_utf8(out).map_err(ReadAsciiStringWithLenError::StringFromUtf8)
}

pub fn encode_ascii_string_to_byte_array(string: &str) -> Vec<u8> {
    let mut buffer = vec![0u8; string.len() * 2];

    for (index, &byte) in string.as_bytes().iter().enumerate() {
        buffer[index * 2] = byte;
    }

    buffer
}

pub fn read_ascii_string_with_len(
    reader: &mut impl Read,
    len: u32,
) -> Result<String, ReadAsciiStringWithLenError> {
    let mut buffer = vec![0u8; ((len) * 2) as usize];

    reader
        .read_exact(&mut buffer)
        .map_err(ReadAsciiStringWithLenError::Read)?;

    decode_ascii_string_from_byte_array(buffer)
}

pub fn write_ascii_string_with_len(
    writer: &mut impl Write,
    string: &str,
) -> Result<(), std::io::Error> {
    let buffer = encode_ascii_string_to_byte_array(string);

    writer.write_all(&buffer)?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum FolderIsEmptyValidationError {
    #[error("unable to access folder {}: {}", .0.display(), 1)]
    AccessDenied(PathBuf, #[source] std::io::Error),

    #[error("folder \"{folder_path}\" is not empty, consider to use merge or select empty folder")]
    FilesAlreadyExistsInFolder { folder_path: PathBuf },
}

pub fn validate_folder_is_empty(
    destination_folder: &Path,
) -> Result<(), FolderIsEmptyValidationError> {
    if destination_folder.exists() {
        let has_entry = std::fs::read_dir(destination_folder)
            .map_err(|err| {
                FolderIsEmptyValidationError::AccessDenied(destination_folder.to_path_buf(), err)
            })?
            .next()
            .is_some();

        if has_entry {
            return Err(FolderIsEmptyValidationError::FilesAlreadyExistsInFolder {
                folder_path: destination_folder.to_path_buf(),
            });
        }
    }

    Ok(())
}

pub fn remove_dir_all_files_predicate(
    path: &Path,
    delete_file_predicate: impl Fn(&walkdir::DirEntry) -> bool,
) -> Result<(), std::io::Error> {
    let mut previous_valid_file: Option<walkdir::DirEntry> = None;

    for dir_entry in WalkDir::new(path).contents_first(true) {
        let dir_entry = dir_entry?;
        let dir_entry_type = dir_entry.file_type();
        let dir_entry_path = dir_entry.path();

        if dir_entry_type.is_dir() {
            let is_dir_valid = previous_valid_file.as_ref().map_or(false, |p| {
                p.path()
                    .parent()
                    .map_or(false, |p| p.starts_with(dir_entry_path))
            });

            if !is_dir_valid {
                std::fs::remove_dir_all(dir_entry_path)?;
            }
        } else {
            match delete_file_predicate(&dir_entry) {
                true => std::fs::remove_file(dir_entry_path)?,
                false => previous_valid_file = Some(dir_entry),
            }
        }
    }

    Ok(())
}
