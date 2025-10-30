mod sorting;
pub use sorting::*;

mod combined_transform;
pub use combined_transform::*;

mod localization_cleanup;
pub use localization_cleanup::*;

mod bom_normalization;
pub use bom_normalization::*;

pub trait PkgFileTransform {
    type Error: std::error::Error + Send + Sync + 'static;

    fn transform(&self, filename: &str, content: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error>;

    fn is_applicable(&self, filename: &str) -> bool;
}
