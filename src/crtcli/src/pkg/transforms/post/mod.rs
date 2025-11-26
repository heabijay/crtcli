mod apply_post_features;
pub use apply_post_features::PkgApplyPostFeatures;

mod combined;
pub use combined::*;

mod csproj_pkg_refs_regenerate;
pub use csproj_pkg_refs_regenerate::*;

use std::path::Path;

pub trait PkgFolderPostTransform {
    type Error: std::error::Error + Send + Sync + 'static;

    fn transform(
        &self,
        pkg_folder: &Path,
        check_only: bool,
        stdout: impl std::io::Write,
    ) -> Result<bool, Self::Error>;
}
