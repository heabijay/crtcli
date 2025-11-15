use crate::pkg::transforms::post::*;
use clap::Args;
use serde::Deserialize;

#[derive(Args, Debug, Default, Deserialize, Clone)]
pub struct PkgApplyPostFeatures {
    /// Regenerates the package assembly "Files/*.csproj" based on the dependencies in descriptor.json
    #[arg(long)]
    #[serde(rename = "post_csproj_pkg_refs_regenerate")]
    apply_post_csproj_pkg_refs_regenerate: Option<bool>,
}

impl PkgApplyPostFeatures {
    pub fn combine(&self, other: Option<&PkgApplyPostFeatures>) -> PkgApplyPostFeatures {
        PkgApplyPostFeatures {
            apply_post_csproj_pkg_refs_regenerate: self.apply_post_csproj_pkg_refs_regenerate.or(
                other
                    .as_ref()
                    .and_then(|x| x.apply_post_csproj_pkg_refs_regenerate),
            ),
        }
    }

    pub fn build_combined_transform(&self) -> CombinedPkgFolderPostTransform {
        let mut combined = CombinedPkgFolderPostTransform::new();

        if self
            .apply_post_csproj_pkg_refs_regenerate
            .is_some_and(|x| x)
        {
            combined.add(CsprojPkgRefsRegeneratePkgFolderPostTransform::new());
        }

        combined
    }
}
