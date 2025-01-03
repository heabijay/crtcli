use crate::pkg::converters::PkgFileConverter;
use crate::pkg::json_wrappers::*;
use crate::pkg::xml_wrappers::*;
use thiserror::Error;

pub struct SortingPkgFileConverter;

#[derive(Error, Debug)]
pub enum SortingPkgFileConverterError {
    #[error("failed to parse json file: {0}")]
    ParseJsonFile(#[from] PkgJsonWrapperCreateError),

    #[error("failed to apply package descriptor sorting: {0}")]
    ApplyPackageDescriptorSorting(#[from] PkgPackageDescriptorSortingError),

    #[error("failed to apply json data data sorting: {0}")]
    ApplyDataDataSorting(#[from] PkgDataDataSortingError),

    #[error("failed to apply json data descriptor sorting: {0}")]
    ApplyDataDescriptorSorting(#[from] PkgDataDescriptorSortingError),

    #[error("failed to apply csproj sorting: {0}")]
    ApplyCsprojSorting(#[from] csproj::CsprojProcessingError),

    #[error("failed to serialize/save json: {0}")]
    Serialize(#[from] PkgJsonWrapperSerializeError),
}

impl PkgFileConverter for SortingPkgFileConverter {
    type Error = SortingPkgFileConverterError;

    fn convert(&self, filename: &str, content: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error> {
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
            || csproj::PKG_CSPROJ_PATH_REGEX.is_match(filename)
    }
}
