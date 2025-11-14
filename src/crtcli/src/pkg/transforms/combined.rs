use crate::pkg::transforms::PkgFileTransform;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Default)]
pub struct CombinedPkgFileTransform {
    transforms: Vec<Box<dyn DynPkgFileTransform>>,
}

impl CombinedPkgFileTransform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<C: PkgFileTransform + 'static>(&mut self, transform: C) -> &mut Self {
        self.transforms.push(Box::new(transform));
        self
    }

    pub const fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }
}

impl PkgFileTransform for CombinedPkgFileTransform {
    type Error = CombinedPkgFileTransformError;

    fn transform(&self, filename: &str, content: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error> {
        let mut content = content;

        for transform in &self.transforms {
            if let Some(c) = transform.transform_dyn(filename, content)? {
                content = c;
            } else {
                return Ok(None);
            }
        }

        Ok(Some(content))
    }

    fn is_applicable(&self, filename: &str) -> bool {
        self.transforms.iter().any(|c| c.is_applicable(filename))
    }
}

impl Debug for CombinedPkgFileTransform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CombinedPkgFileTransform")
            .field("transforms.len()", &self.transforms.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct CombinedPkgFileTransformError(Box<dyn Error + Send + Sync>);

impl Display for CombinedPkgFileTransformError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Error for CombinedPkgFileTransformError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl From<Box<dyn Error + Send + Sync>> for CombinedPkgFileTransformError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        Self(error)
    }
}

trait DynPkgFileTransform {
    fn transform_dyn(
        &self,
        filename: &str,
        content: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error + Send + Sync>>;

    fn is_applicable(&self, filename: &str) -> bool;
}

impl<T> DynPkgFileTransform for T
where
    T: PkgFileTransform,
{
    fn transform_dyn(
        &self,
        filename: &str,
        content: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error + Send + Sync>> {
        self.transform(filename, content)
            .map_err(|e| Box::new(e) as _)
    }

    fn is_applicable(&self, filename: &str) -> bool {
        self.is_applicable(filename)
    }
}
