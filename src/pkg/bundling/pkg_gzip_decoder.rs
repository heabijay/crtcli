use crate::pkg::bundling::utils::{read_ascii_string_with_len, ReadAsciiStringWithLenError};
use crate::pkg::bundling::PkgGZipFile;
use flate2::read::GzDecoder;
use std::io::{BufReader, ErrorKind, Read};
use thiserror::Error;
use zip::unstable::LittleEndianReadExt;

pub struct PkgGZipDecoder<R: Read> {
    gz_decoder: BufReader<GzDecoder<R>>,
}

#[derive(Error, Debug)]
pub enum PkgGZipDecoderError {
    #[error("failed to read filename size: {0}")]
    FilenameSize(#[source] std::io::Error),

    #[error("failed to read filename: {0}")]
    Filename(#[source] ReadAsciiStringWithLenError),

    #[error("failed to read file content size: {0}")]
    FileContentSize(#[source] std::io::Error),

    #[error("failed to read file content: {0}")]
    FileContent(#[source] std::io::Error),
}

impl<R: Read> PkgGZipDecoder<R> {
    pub fn new(gz_decoder: GzDecoder<R>) -> Self {
        Self::from(gz_decoder)
    }

    fn next_as_filename(&mut self) -> Result<Option<String>, PkgGZipDecoderError> {
        let filename_size = match self.gz_decoder.read_u32_le() {
            Ok(size) => size,
            Err(err) if err.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(err) => return Err(PkgGZipDecoderError::FilenameSize(err)),
        };

        let string = read_ascii_string_with_len(&mut self.gz_decoder, filename_size)
            .map_err(PkgGZipDecoderError::Filename)?;

        Ok(Some(string))
    }

    fn next_as_content_vec(&mut self) -> Result<Vec<u8>, PkgGZipDecoderError> {
        let content_size = self
            .gz_decoder
            .read_u32_le()
            .map_err(PkgGZipDecoderError::FileContentSize)?;

        let mut content = vec![0u8; content_size as usize];

        self.gz_decoder
            .read_exact(&mut content)
            .map_err(PkgGZipDecoderError::FileContent)?;

        Ok(content)
    }
}

impl<R: Read> From<GzDecoder<R>> for PkgGZipDecoder<R> {
    fn from(value: GzDecoder<R>) -> Self {
        Self {
            gz_decoder: BufReader::new(value),
        }
    }
}

impl<R: Read> From<R> for PkgGZipDecoder<R> {
    fn from(value: R) -> Self {
        Self {
            gz_decoder: BufReader::new(GzDecoder::new(value)),
        }
    }
}

impl<R: Read> From<BufReader<GzDecoder<R>>> for PkgGZipDecoder<R> {
    fn from(value: BufReader<GzDecoder<R>>) -> Self {
        Self { gz_decoder: value }
    }
}

impl<R: Read> Iterator for PkgGZipDecoder<R> {
    type Item = Result<PkgGZipFile, PkgGZipDecoderError>;

    fn next(&mut self) -> Option<Self::Item> {
        let filename = match self.next_as_filename() {
            Ok(None) => return None,
            Ok(Some(filename)) => filename,
            Err(err) => return Some(Err(err)),
        };

        let content = match self.next_as_content_vec() {
            Ok(content) => content,
            Err(err) => return Some(Err(err)),
        };

        Some(Ok(PkgGZipFile { filename, content }))
    }
}
