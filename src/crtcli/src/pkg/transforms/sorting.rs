use crate::pkg::json_wrappers::*;
use crate::pkg::transforms::PkgFileTransform;
use crate::pkg::xml_wrappers::*;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

pub struct SortingPkgFileTransform {
    comparer: SortingComparer,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, ValueEnum)]
pub enum SortingComparer {
    /// Alphanumeric comparer, pretending to be equivalent to PostgreSQL collation comparing. This means that it ignores non-alphanumeric characters when comparing.
    #[default]
    #[clap(aliases = ["psql", "postgresql"])]
    #[serde(rename = "alnum", alias = "psql", alias = "postgresql")]
    Alnum,

    /// Standard comparer, which uses all characters in a string and compares bytes byte by byte.
    #[clap(aliases = ["standard", "mssql"])]
    #[serde(rename = "std", alias = "standard", alias = "mssql")]
    Std,
}

impl SortingComparer {
    pub fn cmp(&self, a: &[u8], b: &[u8]) -> Ordering {
        match self {
            SortingComparer::Alnum => crate::utils::lexical_str::ascii_alnum_cmp(a, b),
            SortingComparer::Std => a.cmp(b),
        }
    }
}

#[derive(Error, Debug)]
pub enum SortingPkgFileTransformError {
    #[error("failed to parse json file: {0}")]
    ParseJsonFile(#[from] PkgJsonWrapperCreateError),

    #[error("failed to apply package descriptor sorting: {0}")]
    ApplyPackageDescriptorSorting(#[from] PkgPackageDescriptorSortingError),

    #[error("failed to apply json data data sorting: {0}")]
    ApplyDataDataSorting(#[from] PkgDataDataSortingError),

    #[error("failed to apply json data descriptor sorting: {0}")]
    ApplyDataDescriptorSorting(#[from] PkgDataDescriptorSortingError),

    #[error("failed to apply resource sorting: {0}")]
    ApplyResourceSorting(#[from] resource::ResourceProcessingError),

    #[error("failed to apply csproj sorting: {0}")]
    ApplyCsprojSorting(#[from] csproj::CsprojProcessingError),

    #[error("failed to serialize/save json: {0}")]
    Serialize(#[from] PkgJsonWrapperSerializeError),
}

impl SortingPkgFileTransform {
    pub fn new(comparer: SortingComparer) -> Self {
        Self { comparer }
    }
}

impl PkgFileTransform for SortingPkgFileTransform {
    type Error = SortingPkgFileTransformError;

    fn transform(&self, filename: &str, content: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error> {
        let mut out = vec![];

        if filename == crate::pkg::paths::PKG_DESCRIPTOR_FILE {
            PkgPackageDescriptorJsonWrapper::from(PkgJsonWrapper::new(&content)?)
                .apply_sorting()?
                .serialize(&mut out)?;

            return Ok(Some(out));
        }

        if PKG_DATA_DATA_PATH_REGEX.is_match(filename)
            || PKG_DATA_LCZ_DATA_PATH_REGEX.is_match(filename)
        {
            PkgDataDataJsonWrapper::from(PkgJsonWrapper::new(&content)?)
                .apply_sorting()?
                .serialize(&mut out)?;

            return Ok(Some(out));
        }

        if PKG_DATA_DESCRIPTOR_PATH_REGEX.is_match(filename) {
            PkgDataDescriptorJsonWrapper::from(PkgJsonWrapper::new(&content)?)
                .apply_sorting()?
                .serialize(&mut out)?;

            return Ok(Some(out));
        }

        if resource::PKG_RESOURCE_PATH_REGEX.is_match(filename) {
            return Ok(Some(resource::apply_sorting(&content, self.comparer)?));
        }

        if csproj::PKG_CSPROJ_PATH_REGEX.is_match(filename) {
            return Ok(Some(csproj::apply_sorting(&content)?));
        }

        Ok(Some(content))
    }

    fn is_applicable(&self, filename: &str) -> bool {
        filename == crate::pkg::paths::PKG_DESCRIPTOR_FILE
            || PKG_DATA_DATA_PATH_REGEX.is_match(filename)
            || PKG_DATA_LCZ_DATA_PATH_REGEX.is_match(filename)
            || PKG_DATA_DESCRIPTOR_PATH_REGEX.is_match(filename)
            || resource::PKG_RESOURCE_PATH_REGEX.is_match(filename)
            || csproj::PKG_CSPROJ_PATH_REGEX.is_match(filename)
    }
}
