use regex::Regex;
use std::sync::LazyLock;

pub static PKG_ASSEMBLIES_DESCRIPTOR_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Assemblies{sep}.+?{sep}descriptor.json$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package assemblies descriptor path regex")
});
