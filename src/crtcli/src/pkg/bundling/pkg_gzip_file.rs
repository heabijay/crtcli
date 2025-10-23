use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PkgGZipFile {
    pub filename: String,
    pub content: Vec<u8>,
}

impl PkgGZipFile {
    pub fn open_fs_file_relative(
        pkg_path: impl AsRef<Path>,
        relative_path: impl AsRef<Path>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            filename: relative_path.as_ref().to_str().unwrap().to_owned(),
            content: std::fs::read(pkg_path.as_ref().join(relative_path))?,
        })
    }

    pub fn open_fs_file_absolute(
        pkg_path: impl AsRef<Path>,
        absolute_path: impl AsRef<Path>,
    ) -> Result<Self, std::io::Error> {
        let relative = absolute_path
            .as_ref()
            .strip_prefix(pkg_path.as_ref())
            .unwrap();

        Self::open_fs_file_relative(pkg_path, relative)
    }

    pub fn to_native_path_string(&self) -> Cow<'_, str> {
        if std::path::MAIN_SEPARATOR == '/' && self.filename.contains('\\') {
            Cow::Owned(self.filename.replace('\\', "/"))
        } else if std::path::MAIN_SEPARATOR != '/' && self.filename.contains('/') {
            Cow::Owned(self.filename.replace('/', "\\"))
        } else {
            Cow::Borrowed(&self.filename)
        }
    }
}

impl Display for PkgGZipFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({} bytes)", self.filename, self.content.len())
    }
}
