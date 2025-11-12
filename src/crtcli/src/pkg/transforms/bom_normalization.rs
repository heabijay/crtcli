use crate::pkg::json::*;
use crate::pkg::paths::*;
use crate::pkg::transforms::PkgFileTransform;
use crate::pkg::xml::resource::PKG_RESOURCE_PATH_REGEX;
use crate::utils::bom;
use clap::ValueEnum;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Deserialize, ValueEnum)]
pub enum BomNormalizationMode {
    /// Adds Byte Order Mark bytes to files if not present.
    #[serde(rename = "add")]
    Add,

    /// Removes Byte Order Mark bytes from files if present.
    #[serde(rename = "remove")]
    Remove,
}

pub struct BomNormalizationPkgFileTransform {
    mode: BomNormalizationMode,
}

impl BomNormalizationPkgFileTransform {
    pub fn new(mode: BomNormalizationMode) -> Self {
        Self { mode }
    }
}

#[derive(Error, Debug)]
pub enum BomNormalizationPkgFileTransformError {}

impl PkgFileTransform for BomNormalizationPkgFileTransform {
    type Error = BomNormalizationPkgFileTransformError;

    fn transform(
        &self,
        filename: &str,
        mut content: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        if self.is_applicable(filename) {
            match self.mode {
                BomNormalizationMode::Add => {
                    if !bom::is_bom(&content) {
                        content.splice(0..0, bom::BOM_CHAR_BYTES.iter().cloned());

                        return Ok(Some(content));
                    }
                }
                BomNormalizationMode::Remove => {
                    if bom::is_bom(&content) {
                        let content_out = Vec::from(bom::trim_bom(&content));

                        return Ok(Some(content_out));
                    }
                }
            }
        }

        Ok(Some(content))
    }

    fn is_applicable(&self, filename: &str) -> bool {
        if filename.starts_with(FILES_FOLDER) {
            return false;
        }

        PKG_DESCRIPTOR_FILE == filename
            || PKG_ASSEMBLIES_DESCRIPTOR_PATH_REGEX.is_match(filename)
            || PKG_DATA_DESCRIPTOR_PATH_REGEX.is_match(filename)
            || PKG_DATA_DATA_PATH_REGEX.is_match(filename)
            || PKG_DATA_LCZ_DATA_PATH_REGEX.is_match(filename)
            || PKG_RESOURCE_PATH_REGEX.is_match(filename)
            || PKG_SCHEMAS_DESCRIPTOR_PATH_REGEX.is_match(filename)
            || PKG_SCHEMAS_METADATA_PATH_REGEX.is_match(filename)
            || PKG_SCHEMAS_PROPERTIES_PATH_REGEX.is_match(filename)
            || PKG_SQLSCRIPTS_DESCRIPTOR_PATH_REGEX.is_match(filename)
    }
}
