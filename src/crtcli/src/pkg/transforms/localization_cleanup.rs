use crate::pkg::json::PKG_DATA_LCZ_DATA_PATH_REGEX;
use crate::pkg::transforms::PkgFileTransform;
use crate::pkg::xml::resource::PKG_RESOURCE_PATH_REGEX;
use std::collections::HashSet;
use thiserror::Error;

pub struct LocalizationCleanupPkgFileTransform {
    allow_cultures: HashSet<String>,
}

impl LocalizationCleanupPkgFileTransform {
    pub fn new(allow_cultures: HashSet<String>) -> Self {
        Self { allow_cultures }
    }
}

#[derive(Error, Debug)]
pub enum LocalizationCleanupPkgFileTransformError {}

impl PkgFileTransform for LocalizationCleanupPkgFileTransform {
    type Error = LocalizationCleanupPkgFileTransformError;

    fn transform(&self, filename: &str, content: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error> {
        if let Some(caps) = PKG_DATA_LCZ_DATA_PATH_REGEX
            .captures(filename)
            .or_else(|| PKG_RESOURCE_PATH_REGEX.captures(filename))
        {
            return if self.allow_cultures.contains(&caps["culture"]) {
                Ok(Some(content))
            } else {
                Ok(None)
            };
        }

        Ok(Some(content))
    }

    fn is_applicable(&self, filename: &str) -> bool {
        PKG_DATA_LCZ_DATA_PATH_REGEX.is_match(filename)
            || PKG_RESOURCE_PATH_REGEX.is_match(filename)
    }
}
