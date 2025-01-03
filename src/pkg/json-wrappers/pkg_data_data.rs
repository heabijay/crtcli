use crate::pkg::json_wrappers::PkgJsonWrapper;
use regex::Regex;
use serde_json::Value;
use std::ops::Deref;
use std::sync::LazyLock;
use thiserror::Error;

pub static PKG_DATA_DATA_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Data{sep}.+?{sep}data\.json$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package data data file path regex")
});

pub static PKG_DATA_LCZ_DATA_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Data{sep}.+?{sep}Localization{sep}data\.(?<culture>.+?)\.json$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package data lcz data file path regex")
});

pub struct PkgDataDataJsonWrapper {
    inner_wrapper: PkgJsonWrapper,
}

impl From<PkgJsonWrapper> for PkgDataDataJsonWrapper {
    fn from(wrapper: PkgJsonWrapper) -> Self {
        Self {
            inner_wrapper: wrapper,
        }
    }
}

impl Deref for PkgDataDataJsonWrapper {
    type Target = PkgJsonWrapper;

    fn deref(&self) -> &Self::Target {
        &self.inner_wrapper
    }
}

#[derive(Error, Debug)]
pub enum PkgDataDataSortingError {
    #[error("failed to get package data array")]
    FailedToGetPackageDataArray,

    #[error("failed to get package data row array")]
    FailedToGetPackageDataRowArray,
}

#[allow(dead_code)]
impl PkgDataDataJsonWrapper {
    fn package_data(&self) -> &Value {
        &self.inner_wrapper.value["PackageData"]
    }

    fn package_data_mut(&mut self) -> &mut Value {
        &mut self.inner_wrapper.value["PackageData"]
    }

    pub fn apply_sorting(&mut self) -> Result<&mut Self, PkgDataDataSortingError> {
        for data in self
            .package_data_mut()
            .as_array_mut()
            .ok_or(PkgDataDataSortingError::FailedToGetPackageDataArray)?
        {
            let row = (*data)["Row"]
                .as_array_mut()
                .ok_or(PkgDataDataSortingError::FailedToGetPackageDataRowArray)?;

            row.sort_by(|k1, k2| {
                k1["SchemaColumnUId"]
                    .as_str()
                    .cmp(&k2["SchemaColumnUId"].as_str())
            });
        }

        Ok(self)
    }
}
