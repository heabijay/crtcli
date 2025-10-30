use crate::pkg::PkgApplyFeatures;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

pub const PKG_CONFIG_FILENAME: &str = "package.crtcli.toml";

#[derive(Debug, Deserialize)]
pub struct PkgConfig {
    apply: PkgApplyFeatures,
}

#[derive(Debug, Error)]
pub enum PkgConfigError {
    #[error("failed to read {} file: {}", PKG_CONFIG_FILENAME, .0)]
    Read(#[from] std::io::Error),

    #[error("failed to parse {} config: {}", PKG_CONFIG_FILENAME, .0)]
    Parse(#[from] toml::de::Error),
}

impl PkgConfig {
    pub fn apply(&self) -> &PkgApplyFeatures {
        &self.apply
    }

    pub fn from_str(config_str: &str) -> Result<PkgConfig, PkgConfigError> {
        let config: PkgConfig = toml::from_str(config_str).map_err(PkgConfigError::Parse)?;

        Ok(config)
    }

    pub fn from_package_folder(
        package_folder: impl AsRef<Path>,
    ) -> Result<Option<PkgConfig>, PkgConfigError> {
        let config_filepath = package_folder.as_ref().join(PKG_CONFIG_FILENAME);

        match config_filepath.exists() {
            false => Ok(None),
            true => {
                let config_str =
                    std::fs::read_to_string(config_filepath).map_err(PkgConfigError::Read)?;

                Ok(Some(PkgConfig::from_str(&config_str)?))
            }
        }
    }
}

pub fn combine_apply_features_from_args_and_config(
    args_features: Option<&PkgApplyFeatures>,
    pkg_config: Option<&PkgConfig>,
) -> Option<PkgApplyFeatures> {
    args_features
        .map(|c| c.combine(pkg_config.map(|c| c.apply())))
        .or_else(|| pkg_config.map(|c| c.apply().clone()))
}
