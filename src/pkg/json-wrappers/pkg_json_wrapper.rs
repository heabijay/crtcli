use crate::utils::bom::{is_bom, trim_bom, BOM_CHAR_BYTES};
use crate::utils::JsonMsDatePreserveFormatter;
use serde::Serialize;
use serde_json::{Serializer, Value};
use std::io::Write;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PkgJsonWrapper {
    is_bom: bool,

    pub value: Value,
}

#[derive(Error, Debug)]
pub enum PkgJsonWrapperCreateError {
    #[error("file read error: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum PkgJsonWrapperSerializeError {
    #[error("failure on write preserved bom bytes: {0}")]
    WritePreservedBomBytes(#[source] std::io::Error),

    #[error("{0}")]
    Serialize(#[from] serde_json::Error),
}

impl PkgJsonWrapper {
    pub fn new(bytes: &[u8]) -> Result<Self, PkgJsonWrapperCreateError> {
        Ok(Self {
            is_bom: is_bom(bytes),
            value: serde_json::from_slice(trim_bom(bytes))?,
        })
    }

    pub fn from_file(path: &Path) -> Result<Self, PkgJsonWrapperCreateError> {
        Self::new(&std::fs::read(path)?)
    }

    pub fn serialize(&self, writer: &mut impl Write) -> Result<(), PkgJsonWrapperSerializeError> {
        if self.is_bom {
            writer
                .write_all(BOM_CHAR_BYTES)
                .map_err(PkgJsonWrapperSerializeError::WritePreservedBomBytes)?;
        }

        let formatter = JsonMsDatePreserveFormatter::new_pretty();
        let mut serializer = Serializer::with_formatter(writer, formatter);

        self.value.serialize(&mut serializer)?;

        Ok(())
    }
}
