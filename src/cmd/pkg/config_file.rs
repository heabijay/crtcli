use crate::cmd::pkg::PkgApplyFeatures;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

pub const PKG_CONFIG_FILENAME: &str = "package.crtcli.toml";

#[derive(Debug, Deserialize)]
pub struct CrtCliPkgConfig {
    apply: PkgApplyFeatures,
}

#[derive(Debug, Error)]
pub enum CrtCliPkgConfigError {
    #[error("failed to read {} file: {}", PKG_CONFIG_FILENAME, .0)]
    Read(#[from] std::io::Error),

    #[error("failed to parse {} config: {}", PKG_CONFIG_FILENAME, .0)]
    Parse(#[from] toml::de::Error),
}

impl CrtCliPkgConfig {
    pub fn apply(&self) -> &PkgApplyFeatures {
        &self.apply
    }

    pub fn from_str(config_str: &str) -> Result<CrtCliPkgConfig, CrtCliPkgConfigError> {
        let config: CrtCliPkgConfig =
            toml::from_str(config_str).map_err(CrtCliPkgConfigError::Parse)?;

        Ok(config)
    }

    pub fn from_package_folder(
        package_folder: impl AsRef<Path>,
    ) -> Result<Option<CrtCliPkgConfig>, CrtCliPkgConfigError> {
        let config_filepath = package_folder.as_ref().join(PKG_CONFIG_FILENAME);

        match config_filepath.exists() {
            false => Ok(None),
            true => {
                let config_str =
                    std::fs::read_to_string(config_filepath).map_err(CrtCliPkgConfigError::Read)?;

                Ok(Some(CrtCliPkgConfig::from_str(&config_str)?))
            }
        }
    }
}

pub fn combine_apply_features_from_args_and_config(
    args_features: Option<&PkgApplyFeatures>,
    pkg_config: Option<&CrtCliPkgConfig>,
) -> Option<PkgApplyFeatures> {
    args_features
        .map(|c| c.combine(pkg_config.map(|c| c.apply())))
        .or_else(|| pkg_config.map(|c| c.apply().clone()))
}
