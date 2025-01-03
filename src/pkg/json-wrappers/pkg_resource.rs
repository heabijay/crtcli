use regex::Regex;
use std::sync::LazyLock;

pub static PKG_RESOURCE_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Resources{sep}.+?{sep}resource\.(?<culture>.+?)\.xml$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package resource path regex")
});
