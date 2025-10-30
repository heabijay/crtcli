use anstyle::{AnsiColor, Color, Style};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const WORKSPACE_CONFIG_FILENAME: &str = "workspace.crtcli.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct WorkspaceAppConfig {
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

#[derive(Debug, Deserialize, Default, Clone)]
pub struct WorkspaceConfig {
    #[serde(default)]
    apps: HashMap<String, WorkspaceAppConfig>,
}

#[derive(Debug, Error)]
pub enum WorkspaceConfigLoadError {
    #[error("failed to read workspace config file {0}: {1}")]
    ReadFile(PathBuf, #[source] std::io::Error),

    #[error("failed to parse workspace config {0}: {1}")]
    ParseFile(PathBuf, #[source] toml::de::Error),
}

impl WorkspaceConfig {
    pub fn from_str(config_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str::<WorkspaceConfig>(config_str)
    }

    pub fn from_filepath(
        config_filepath: impl AsRef<Path>,
    ) -> Result<Self, WorkspaceConfigLoadError> {
        let config_str = std::fs::read_to_string(config_filepath.as_ref()).map_err(|err| {
            WorkspaceConfigLoadError::ReadFile(config_filepath.as_ref().to_owned(), err)
        })?;

        Self::from_str(&config_str).map_err(|err| {
            WorkspaceConfigLoadError::ParseFile(config_filepath.as_ref().to_owned(), err)
        })
    }

    pub fn load_from_current_dir() -> Result<Self, WorkspaceConfigLoadError> {
        let current_dir = if let Ok(current_dir) = std::env::current_dir() {
            current_dir
        } else {
            return Ok(Self::default());
        };

        let config = load_configs_vec_from_current_dir(current_dir)?
            .into_iter()
            .fold(Self::default(), |acc, mut config| {
                config.merge_with(acc);
                config
            });

        return Ok(config);

        fn load_configs_vec_from_current_dir(
            mut current_dir: PathBuf,
        ) -> Result<Vec<WorkspaceConfig>, WorkspaceConfigLoadError> {
            let mut configs = Vec::new();

            loop {
                let config_path = current_dir.join(WORKSPACE_CONFIG_FILENAME);

                if config_path.exists() {
                    match WorkspaceConfig::from_filepath(&config_path) {
                        Ok(config) => configs.push(config),
                        Err(WorkspaceConfigLoadError::ReadFile(path, err)) => {
                            eprintln!(
                                "{style}warning: failed to read workspace config file {path}: {err}{style:#}",
                                path = path.display(),
                                err = err,
                                style = Style::new()
                                    .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                                    .dimmed()
                            );
                        }
                        Err(err) => return Err(err),
                    }
                }

                if !current_dir.pop() {
                    // No more parent directories
                    break;
                }
            }

            Ok(configs)
        }
    }

    pub fn apps(&self) -> &HashMap<String, WorkspaceAppConfig> {
        &self.apps
    }

    /// Merge with another config, with the other config having higher priority
    pub fn merge_with(&mut self, other: Self) {
        for (alias_name, alias) in other.apps {
            self.apps.insert(alias_name, alias);
        }
    }
}
