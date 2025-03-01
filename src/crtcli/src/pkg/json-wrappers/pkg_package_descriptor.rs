use crate::pkg::json_wrappers::pkg_json_wrapper::PkgJsonWrapper;
use serde_json::Value;
use std::ops::Deref;
use thiserror::Error;

pub struct PkgPackageDescriptorJsonWrapper {
    inner_wrapper: PkgJsonWrapper,
}

impl From<PkgJsonWrapper> for PkgPackageDescriptorJsonWrapper {
    fn from(wrapper: PkgJsonWrapper) -> Self {
        Self {
            inner_wrapper: wrapper,
        }
    }
}

impl Deref for PkgPackageDescriptorJsonWrapper {
    type Target = PkgJsonWrapper;

    fn deref(&self) -> &Self::Target {
        &self.inner_wrapper
    }
}

#[derive(Error, Debug)]
pub enum PkgPackageDescriptorSortingError {
    #[error("failed to get package descriptor DependsOn array")]
    FailedToGetDependsOnArray,
}

#[allow(dead_code)]
impl PkgPackageDescriptorJsonWrapper {
    fn descriptor(&self) -> &Value {
        &self.inner_wrapper.value["Descriptor"]
    }

    fn descriptor_mut(&mut self) -> &mut Value {
        &mut self.inner_wrapper.value["Descriptor"]
    }

    pub fn name(&self) -> Option<&str> {
        (*self.descriptor())["Name"].as_str()
    }

    pub fn name_mut(&mut self) -> &mut Value {
        &mut (*self.descriptor_mut())["Name"]
    }

    pub fn uid(&self) -> Option<&str> {
        (*self.descriptor())["UId"].as_str()
    }

    pub fn uid_mut(&mut self) -> &mut Value {
        &mut (*self.descriptor_mut())["UId"]
    }

    fn depends_on(&self) -> &Value {
        &self.descriptor()["DependsOn"]
    }

    fn depends_on_mut(&mut self) -> &mut Value {
        &mut self.descriptor_mut()["DependsOn"]
    }

    fn pkg_type(&self) -> u64 {
        self.descriptor()["Type"].as_u64().unwrap_or(0)
    }

    fn pkg_type_exact(&self) -> Option<u64> {
        self.descriptor()["Type"].as_u64()
    }

    fn pkg_type_on_mut(&mut self) -> &mut Value {
        &mut self.descriptor_mut()["Type"]
    }

    pub fn apply_sorting(&mut self) -> Result<&mut Self, PkgPackageDescriptorSortingError> {
        let columns = self
            .depends_on_mut()
            .as_array_mut()
            .ok_or(PkgPackageDescriptorSortingError::FailedToGetDependsOnArray)?;

        columns.sort_by(|k1, k2| k1["UId"].as_str().cmp(&k2["UId"].as_str()));

        Ok(self)
    }
}
