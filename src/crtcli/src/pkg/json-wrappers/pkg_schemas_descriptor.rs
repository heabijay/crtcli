use regex::Regex;
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
