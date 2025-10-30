use crate::pkg::transforms::PkgFileTransform;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Default)]
pub struct CombinedPkgFileTransform {
    transforms: Vec<Box<dyn CombinedPkgFileTransformAdapter>>,
}

impl CombinedPkgFileTransform {
    pub fn new() -> Self {
        Self { transforms: vec![] }
    }

    pub fn add<C: PkgFileTransform + 'static>(&mut self, transform: C) -> &mut Self {
        self.transforms
            .push(Box::new(CombinedPkgFileTransformAdapterImpl(transform)));
        self
    }
}

impl PkgFileTransform for CombinedPkgFileTransform {
    type Error = CombinedPkgFileTransformError;

    fn transform(&self, filename: &str, content: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error> {
        let mut content = content;

        for transform in &self.transforms {
            if let Some(c) = transform.convert(filename, content)? {
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
pub struct CombinedPkgFileTransformError(Box<dyn std::error::Error + Send + Sync>);

type CombinedPkgFileTransformErrorInnerType = dyn std::error::Error + Send + Sync;

impl Display for CombinedPkgFileTransformError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for CombinedPkgFileTransformError {}

impl From<Box<CombinedPkgFileTransformErrorInnerType>> for CombinedPkgFileTransformError {
    fn from(error: Box<CombinedPkgFileTransformErrorInnerType>) -> Self {
        Self(error)
    }
}

impl Deref for CombinedPkgFileTransformError {
    type Target = Box<CombinedPkgFileTransformErrorInnerType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

trait CombinedPkgFileTransformAdapter {
    fn convert(
        &self,
        filename: &str,
        content: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, Box<CombinedPkgFileTransformErrorInnerType>>;

    fn is_applicable(&self, filename: &str) -> bool;
}

impl<E> CombinedPkgFileTransformAdapter for dyn PkgFileTransform<Error = E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn convert(
        &self,
        filename: &str,
        content: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, Box<CombinedPkgFileTransformErrorInnerType>> {
        self.transform(filename, content).map_err(|err| err.into())
    }

    fn is_applicable(&self, filename: &str) -> bool {
        self.is_applicable(filename)
    }
}

struct CombinedPkgFileTransformAdapterImpl<C: PkgFileTransform>(C);

impl<C: PkgFileTransform> CombinedPkgFileTransformAdapter
    for CombinedPkgFileTransformAdapterImpl<C>
{
    fn convert(
        &self,
        filename: &str,
        content: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, Box<CombinedPkgFileTransformErrorInnerType>> {
        Ok(self.0.transform(filename, content)?)
    }

    fn is_applicable(&self, filename: &str) -> bool {
        self.0.is_applicable(filename)
    }
}

impl<C: PkgFileTransform> From<C> for CombinedPkgFileTransformAdapterImpl<C> {
    fn from(value: C) -> Self {
        Self(value)
    }
}
