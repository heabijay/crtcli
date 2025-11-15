use crate::pkg::transforms::*;
use clap::Args;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Args, Debug, Default, Deserialize, Clone)]
pub struct PkgApplyFeatures {
    /// Sorts files like in the "Data/../*.json", "descriptor.json", ... by some property to simplify merge operations in Git, SVN, etc.
    #[arg(short = 'S', long)]
    #[serde(rename = "sorting")]
    apply_sorting: Option<bool>,

    /// Configures sorting comparer for `--apply-sorting | -S` transform which will be used to sort strings.
    #[arg(
        long,
        default_value = "alnum",
        value_name = "COMPARER", 
        value_hint = clap::ValueHint::Other)]
    #[serde(rename = "sorting_comparer")]
    apply_sorting_comparer: Option<SortingComparer>,

    /// Removes localization files except for the specified cultures (comma-separated list).
    /// Example: --apply-localization-cleanup "en-US,uk-UA"
    #[arg(
        short = 'L',
        long,
        value_name = "EXCEPT-LOCALIZATIONS", 
        value_delimiter = ',',
        value_hint = clap::ValueHint::Other)]
    #[serde(rename = "localization_cleanup")]
    apply_localization_cleanup: Option<Vec<String>>,

    /// Normalizes a Byte Order Mark (BOM) in package schema files (.json / .xml) by adding or removing BOM bytes.
    #[arg(long, value_name = "BOM_NORMALIZATION_MODE")]
    #[serde(rename = "bom_normalization")]
    apply_bom_normalization: Option<BomNormalizationMode>,
}

impl PkgApplyFeatures {
    pub fn combine(&self, other: Option<&PkgApplyFeatures>) -> PkgApplyFeatures {
        PkgApplyFeatures {
            apply_sorting: self
                .apply_sorting
                .or(other.as_ref().and_then(|x| x.apply_sorting)),
            apply_sorting_comparer: self
                .apply_sorting_comparer
                .or(other.as_ref().and_then(|x| x.apply_sorting_comparer)),
            apply_localization_cleanup: self.apply_localization_cleanup.clone().or(other
                .as_ref()
                .and_then(|x| x.apply_localization_cleanup.clone())),
            apply_bom_normalization: self
                .apply_bom_normalization
                .or(other.and_then(|x| x.apply_bom_normalization)),
        }
    }

    pub fn build_combined_transform(&self) -> CombinedPkgFileTransform {
        let mut combined = CombinedPkgFileTransform::new();

        if let Some(localization_cultures) = &self.apply_localization_cleanup {
            combined.add(LocalizationCleanupPkgFileTransform::new(
                HashSet::from_iter(localization_cultures.iter().cloned()),
            ));
        }

        if self.apply_sorting.is_some_and(|x| x) {
            combined.add(SortingPkgFileTransform::new(
                self.apply_sorting_comparer.unwrap_or_default(),
            ));
        }

        if let Some(bom_normalization) = self.apply_bom_normalization {
            combined.add(BomNormalizationPkgFileTransform::new(bom_normalization));
        }

        combined
    }
}
