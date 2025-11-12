use crate::pkg::transforms::post::PkgFolderPostTransform;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;

#[derive(Default)]
pub struct CombinedPkgFolderPostTransform {
    transforms: Vec<Box<dyn DynPkgFolderPostTransform>>,
}

impl CombinedPkgFolderPostTransform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<C: PkgFolderPostTransform + 'static>(&mut self, transform: C) -> &mut Self {
        self.transforms.push(Box::new(transform));
        self
    }
}

impl PkgFolderPostTransform for CombinedPkgFolderPostTransform {
    type Error = CombinedPkgFolderPostTransformError;

    fn transform(&self, pkg_folder: &Path, check_only: bool) -> Result<bool, Self::Error> {
        let mut any_applied = false;

        for transform in &self.transforms {
            if transform.transform_dyn(pkg_folder, check_only)? {
                any_applied = true;
            }
        }

        Ok(any_applied)
    }
}

impl Debug for CombinedPkgFolderPostTransform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CombinedPkgFolderPostTransform")
            .field("transforms.len()", &self.transforms.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct CombinedPkgFolderPostTransformError(Box<dyn Error + Send + Sync>);

impl Display for CombinedPkgFolderPostTransformError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Error for CombinedPkgFolderPostTransformError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl From<Box<dyn Error + Send + Sync>> for CombinedPkgFolderPostTransformError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        Self(error)
    }
}

trait DynPkgFolderPostTransform {
    fn transform_dyn(
        &self,
        pkg_folder: &Path,
        check_only: bool,
    ) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

impl<T> DynPkgFolderPostTransform for T
where
    T: PkgFolderPostTransform,
{
    fn transform_dyn(
        &self,
        pkg_folder: &Path,
        check_only: bool,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.transform(pkg_folder, check_only)
            .map_err(|e| Box::new(e) as _)
    }
}
