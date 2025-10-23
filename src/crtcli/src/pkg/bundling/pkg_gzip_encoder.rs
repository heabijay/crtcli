use crate::pkg::bundling::PkgGZipFile;
use crate::pkg::bundling::utils::write_ascii_string_with_len;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::{BufWriter, Write};
use thiserror::Error;
use zip::unstable::LittleEndianWriteExt;

pub struct PkgGZipEncoder<W: Write> {
    gz_encoder: BufWriter<GzEncoder<W>>,
}

#[derive(Error, Debug)]
pub enum PkgGZipEncoderError {
    #[error("failed to write filename size: {0}")]
    FilenameSize(#[source] std::io::Error),

    #[error("failed to write filename: {0}")]
    Filename(#[source] std::io::Error),

    #[error("failed to write content size: {0}")]
    ContentSize(#[source] std::io::Error),

    #[error("failed to write content: {0}")]
    Content(#[source] std::io::Error),
}

impl<W: Write> PkgGZipEncoder<W> {
    pub fn new(writer: W, compression: Option<Compression>) -> Self {
        Self::from(GzEncoder::new(
            writer,
            compression.unwrap_or(Compression::fast()),
        ))
    }

    fn write_as_filename(&mut self, filename: &str) -> Result<(), PkgGZipEncoderError> {
        self.gz_encoder
            .write_u32_le(filename.len() as u32)
            .map_err(PkgGZipEncoderError::FilenameSize)?;
        write_ascii_string_with_len(&mut self.gz_encoder, filename)
            .map_err(PkgGZipEncoderError::Filename)?;

        Ok(())
    }

    fn write_as_content(&mut self, content: &[u8]) -> Result<(), PkgGZipEncoderError> {
        self.gz_encoder
            .write_u32_le(content.len() as u32)
            .map_err(PkgGZipEncoderError::ContentSize)?;
        self.gz_encoder
            .write_all(content)
            .map_err(PkgGZipEncoderError::Content)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn write_file(&mut self, file: &PkgGZipFile) -> Result<(), PkgGZipEncoderError> {
        self.write_as_filename(&file.filename)?;
        self.write_as_content(&file.content)?;

        Ok(())
    }
}

impl<W: Write> From<GzEncoder<W>> for PkgGZipEncoder<W> {
    fn from(value: GzEncoder<W>) -> Self {
        Self {
            gz_encoder: BufWriter::new(value),
        }
    }
}

impl<W: Write> From<BufWriter<GzEncoder<W>>> for PkgGZipEncoder<W> {
    fn from(value: BufWriter<GzEncoder<W>>) -> Self {
        Self { gz_encoder: value }
    }
}

impl<W: Write> From<W> for PkgGZipEncoder<W> {
    fn from(value: W) -> Self {
        Self::new(value, None)
    }
}
