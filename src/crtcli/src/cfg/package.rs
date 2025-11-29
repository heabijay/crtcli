use crate::pkg::transforms::PkgApplyFeatures;
use crate::pkg::transforms::post::PkgApplyPostFeatures;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

const PKG_CONFIG_FILENAME: &str = "package.crtcli.toml";

#[derive(Debug, Deserialize)]
pub struct PkgConfig {
    apply: PkgConfigApply,
}

#[derive(Debug, Default, Deserialize)]
pub struct PkgConfigApply {
    #[serde(flatten)]
    apply_features: PkgApplyFeatures,

    #[serde(flatten)]
    apply_post_features: PkgApplyPostFeatures,
}

#[derive(Debug, Error)]
pub enum PkgConfigError {
    #[error("failed to read {} file: {}", PKG_CONFIG_FILENAME, .0)]
    Read(#[from] std::io::Error),

    #[error("failed to parse {} config: {}", PKG_CONFIG_FILENAME, .0)]
    Parse(#[from] toml::de::Error),
}

impl PkgConfig {
    pub fn apply(&self) -> &PkgConfigApply {
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

impl PkgConfigApply {
    pub fn apply(&self) -> &PkgApplyFeatures {
        &self.apply_features
    }

    pub fn apply_post(&self) -> &PkgApplyPostFeatures {
        &self.apply_post_features
    }

    pub fn combine(&self, other: Option<&PkgConfigApply>) -> PkgConfigApply {
        PkgConfigApply {
            apply_features: self
                .apply_features
                .combine(other.map(|x| &x.apply_features)),
            apply_post_features: self
                .apply_post_features
                .combine(other.map(|x| &x.apply_post_features)),
        }
    }
}

pub fn combine_apply_config_from_args_and_config(
    (arg_features, arg_post_features): (Option<&PkgApplyFeatures>, Option<&PkgApplyPostFeatures>),
    pkg_config: Option<&PkgConfigApply>,
) -> Option<PkgConfigApply> {
    if arg_features.is_none() && arg_post_features.is_none() && pkg_config.is_none() {
        return None;
    }

    Some(
        PkgConfigApply {
            apply_features: arg_features.map(|x| x.to_owned()).unwrap_or_default(),
            apply_post_features: arg_post_features.map(|x| x.to_owned()).unwrap_or_default(),
        }
        .combine(pkg_config),
    )
}
