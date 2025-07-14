use regex::Regex;
use std::sync::LazyLock;

pub static PKG_SQLSCRIPTS_DESCRIPTOR_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^SqlScripts{sep}.+?{sep}descriptor.json$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package sqlscripts descriptor path regex")
});
