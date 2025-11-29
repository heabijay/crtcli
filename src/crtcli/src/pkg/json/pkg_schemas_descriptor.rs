use crate::pkg::json::PkgJsonWrapper;
use regex::Regex;
use serde_json::Value;
use std::ops::Deref;
use std::sync::LazyLock;

pub static PKG_SCHEMAS_DESCRIPTOR_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Schemas{sep}.+?{sep}descriptor.json$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package schemas descriptor path regex")
});

pub static PKG_SCHEMAS_METADATA_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Schemas{sep}.+?{sep}metadata.json$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package schemas metadata path regex")
});

pub static PKG_SCHEMAS_PROPERTIES_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Schemas{sep}.+?{sep}properties.json$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package schemas properties path regex")
});

pub static PKG_SCHEMAS_CS_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Schemas{sep}.+?{sep}.+\.cs$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package schemas cs path regex")
});

pub struct PkgSchemasDescriptorJsonWrapper {
    inner_wrapper: PkgJsonWrapper,
}

impl From<PkgJsonWrapper> for PkgSchemasDescriptorJsonWrapper {
    fn from(wrapper: PkgJsonWrapper) -> Self {
        Self {
            inner_wrapper: wrapper,
        }
    }
}

impl Deref for PkgSchemasDescriptorJsonWrapper {
    type Target = PkgJsonWrapper;

    fn deref(&self) -> &Self::Target {
        &self.inner_wrapper
    }
}

#[allow(dead_code)]
impl PkgSchemasDescriptorJsonWrapper {
    fn descriptor(&self) -> &Value {
        &self.inner_wrapper.value["Descriptor"]
    }

    fn descriptor_mut(&mut self) -> &mut Value {
        &mut self.inner_wrapper.value["Descriptor"]
    }

    fn modified_on_utc(&self) -> Option<&str> {
        self.descriptor()["ModifiedOnUtc"].as_str()
    }

    pub fn modified_on_utc_mut(&mut self) -> &mut Value {
        &mut self.descriptor_mut()["ModifiedOnUtc"]
    }

    fn caption(&self) -> Option<&str> {
        self.descriptor()["Caption"].as_str()
    }

    pub fn caption_mut(&mut self) -> &mut Value {
        &mut self.descriptor_mut()["Caption"]
    }

    fn depends_on(&self) -> &Value {
        &self.descriptor()["DependsOn"]
    }

    pub fn depends_on_mut(&mut self) -> &mut Value {
        &mut self.descriptor_mut()["DependsOn"]
    }
}
