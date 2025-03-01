use crate::pkg::json_wrappers::PkgJsonWrapper;
use regex::Regex;
use serde_json::Value;
use std::ops::Deref;
use std::sync::LazyLock;
use thiserror::Error;

pub static PKG_DATA_DESCRIPTOR_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Data{sep}.+?{sep}descriptor.json$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package data descriptor path regex")
});

pub struct PkgDataDescriptorJsonWrapper {
    inner_wrapper: PkgJsonWrapper,
}

impl From<PkgJsonWrapper> for PkgDataDescriptorJsonWrapper {
    fn from(wrapper: PkgJsonWrapper) -> Self {
        Self {
            inner_wrapper: wrapper,
        }
    }
}

impl Deref for PkgDataDescriptorJsonWrapper {
    type Target = PkgJsonWrapper;

    fn deref(&self) -> &Self::Target {
        &self.inner_wrapper
    }
}

#[derive(Error, Debug)]
pub enum PkgDataDescriptorSortingError {
    #[error("failed to get package data array")]
    FailedToGetColumnsArray,
}

#[allow(dead_code)]
impl PkgDataDescriptorJsonWrapper {
    fn descriptor(&self) -> &Value {
        &self.inner_wrapper.value["Descriptor"]
    }

    fn descriptor_mut(&mut self) -> &mut Value {
        &mut self.inner_wrapper.value["Descriptor"]
    }

    fn columns(&self) -> &Value {
        &self.descriptor()["Columns"]
    }

    fn columns_mut(&mut self) -> &mut Value {
        &mut self.descriptor_mut()["Columns"]
    }

    pub fn apply_sorting(&mut self) -> Result<&mut Self, PkgDataDescriptorSortingError> {
        let columns = self
            .columns_mut()
            .as_array_mut()
            .ok_or(PkgDataDescriptorSortingError::FailedToGetColumnsArray)?;

        columns.sort_by(|k1, k2| k1["ColumnUId"].as_str().cmp(&k2["ColumnUId"].as_str()));

        Ok(self)
    }
}
