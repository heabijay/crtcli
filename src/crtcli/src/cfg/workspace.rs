use crate::pkg::utils::GetPackageNameFromFolderError;
use anstyle::{AnsiColor, Color, Style};
use serde::Deserialize;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use thiserror::Error;

const WORKSPACE_CONFIG_FILENAME: &str = "workspace.crtcli.toml";

#[derive(Debug, Default, Deserialize)]
pub struct WorkspaceConfig {
    packages: Vec<WorkspacePkgConfig>,
}

#[derive(Debug, Deserialize)]
pub struct WorkspacePkgConfig {
    // name: String,
    path: PathBuf,
}

impl WorkspacePkgConfig {
    pub fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn package_name(&self) -> Result<Cow<'_, str>, GetPackageNameFromFolderError> {
        crate::pkg::utils::get_package_name_from_folder(&self.path).map(Cow::Owned)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Error)]
pub enum WorkspaceConfigLoadFileError {
    #[error("failed to read {0} config file: {1}")]
    Read(PathBuf, #[source] std::io::Error),

    #[error("failed to parse {0} config file: {1}")]
    Parse(PathBuf, #[source] toml::de::Error),
}

impl WorkspaceConfig {
    fn from_str(config_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str::<WorkspaceConfig>(config_str)
    }

    pub fn from_filepath(
        config_filepath: impl AsRef<Path>,
    ) -> Result<Self, WorkspaceConfigLoadFileError> {
        let config_str = std::fs::read_to_string(config_filepath.as_ref()).map_err(|err| {
            WorkspaceConfigLoadFileError::Read(config_filepath.as_ref().to_owned(), err)
        })?;

        let config = Self::from_str(&config_str).map_err(|err| {
            WorkspaceConfigLoadFileError::Parse(config_filepath.as_ref().to_owned(), err)
        })?;

        Ok(config)
    }

    fn load_config_in_current_dir() -> Result<Option<Self>, WorkspaceConfigLoadFileError> {
        let config_path = PathBuf::from(".").join(WORKSPACE_CONFIG_FILENAME);

        if !config_path.exists() {
            return Ok(None);
        }

        Self::from_filepath(config_path).map(Some)
    }

    fn load_as_pkg_descriptor_in_current_dir() -> Option<WorkspaceConfig> {
        let current_dir = PathBuf::from(".");
        let descriptor_path = current_dir.join(crate::pkg::paths::PKG_DESCRIPTOR_FILE);

        if descriptor_path.exists() {
            Some(Self {
                packages: vec![WorkspacePkgConfig::from_path(current_dir)],
            })
        } else {
            None
        }
    }

    pub fn load_default_from_current_dir() -> Result<Self, WorkspaceConfigLoadFileError> {
        let pkg_descriptor_based = Self::load_as_pkg_descriptor_in_current_dir();
        let config_based = Self::load_config_in_current_dir()?;

        if pkg_descriptor_based.is_some() && config_based.is_some() {
            eprintln!(
                "{style}warning: both the ./descriptor.json and workspace.crtcli.toml files are present in the current folder; workspace.crtcli.toml is preferred{style:#}",
                style = Style::new()
                    .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                    .dimmed()
            );
        }

        Ok(config_based.unwrap_or_else(|| pkg_descriptor_based.unwrap_or_default()))
    }

    pub fn packages(&self) -> &Vec<WorkspacePkgConfig> {
        &self.packages
    }
}
