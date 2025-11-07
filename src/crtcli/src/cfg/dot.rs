use anstyle::{AnsiColor, Color, Style};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const DOT_CONFIG_FILENAME: &str = ".crtcli.toml";

#[derive(Debug, Deserialize, Default, Clone)]
pub struct DotConfig {
    root: Option<bool>,

    #[serde(default)]
    apps: HashMap<String, DotAppConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DotAppConfig {
    pub url: String,

    pub username: Option<String>,
    pub password: Option<String>,

    pub oauth_url: Option<String>,
    pub oauth_client_id: Option<String>,
    pub oauth_client_secret: Option<String>,

    pub insecure: Option<bool>,

    #[serde(alias = "netframework")]
    pub net_framework: Option<bool>,
}

#[derive(Debug, Error)]
pub enum DotConfigLoadFileError {
    #[error("failed to read {0} config file: {1}")]
    Read(PathBuf, #[source] std::io::Error),

    #[error("failed to parse {0} config file: {1}")]
    Parse(PathBuf, #[source] toml::de::Error),

    #[error("failed to validate {0} config file: {1}")]
    Validate(PathBuf, #[source] DotConfigValidationError),
}

#[derive(Debug, Error)]
pub enum DotConfigValidationError {}

impl DotConfig {
    // to make this fn public, we should post validate the result
    fn from_str(config_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str::<DotConfig>(config_str)
    }

    pub fn from_filepath(
        config_filepath: impl AsRef<Path>,
    ) -> Result<Self, DotConfigLoadFileError> {
        let config_str = std::fs::read_to_string(config_filepath.as_ref()).map_err(|err| {
            DotConfigLoadFileError::Read(config_filepath.as_ref().to_owned(), err)
        })?;

        let config = Self::from_str(&config_str).map_err(|err| {
            DotConfigLoadFileError::Parse(config_filepath.as_ref().to_owned(), err)
        })?;

        config.validate().map_err(|err| {
            DotConfigLoadFileError::Validate(config_filepath.as_ref().to_owned(), err)
        })?;

        Ok(config)
    }

    pub fn load_from_current_dir() -> Result<Self, DotConfigLoadFileError> {
        let current_dir = std::env::current_dir().ok();
        Self::load_from_directory_hierarchy(current_dir.as_deref())
    }

    fn load_from_directory_hierarchy(
        mut current_dir: Option<&Path>,
    ) -> Result<Self, DotConfigLoadFileError> {
        let mut current_config = Self::default();

        while let Some(dir) = current_dir {
            if let Some(config) = Self::try_load_config_from_dir(dir)? {
                let is_root = config.root.unwrap_or_default();
                current_config.merge_other_with_less_priority(config);

                if is_root {
                    break;
                }
            }

            // Move to parent directory
            current_dir = dir.parent();
        }

        Ok(current_config)
    }

    fn try_load_config_from_dir(dir: &Path) -> Result<Option<Self>, DotConfigLoadFileError> {
        let config_path = dir.join(DOT_CONFIG_FILENAME);

        if !config_path.exists() {
            return Ok(None);
        }

        match Self::from_filepath(&config_path) {
            Ok(config) => Ok(Some(config)),
            Err(DotConfigLoadFileError::Read(path, err)) => {
                eprintln!(
                    "{style}warning: failed to read {path} config file: {err}{style:#}",
                    path = path.display(),
                    err = err,
                    style = Style::new()
                        .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                        .dimmed()
                );
                Ok(None)
            }
            Err(err) => Err(err),
        }
    }

    pub fn apps(&self) -> &HashMap<String, DotAppConfig> {
        &self.apps
    }

    fn merge_other_with_less_priority(&mut self, other: Self) {
        for (app_name, app_config) in other.apps {
            if self.apps.contains_key(&app_name) {
                continue;
            }

            self.apps.insert(app_name, app_config);
        }
    }

    fn validate(&self) -> Result<(), DotConfigValidationError> {
        Ok(())
    }
}
