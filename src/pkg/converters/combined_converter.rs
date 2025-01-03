use crate::pkg::converters::PkgFileConverter;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Default)]
pub struct CombinedPkgFileConverter {
    converters: Vec<Box<dyn CombinedPkgFileConverterAdapter>>,
}

impl CombinedPkgFileConverter {
    pub fn new() -> Self {
        Self { converters: vec![] }
    }

    pub fn add<C: PkgFileConverter + 'static>(&mut self, converter: C) -> &mut Self {
        self.converters
            .push(Box::new(CombinedPkgFileConverterAdapterImpl(converter)));
        self
    }
}

impl PkgFileConverter for CombinedPkgFileConverter {
    type Error = CombinedPkgFileConverterError;

    fn convert(&self, filename: &str, content: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error> {
        let mut content = content;

        for converter in &self.converters {
            if let Some(c) = converter.convert(filename, content)? {
                content = c;
            } else {
                return Ok(None);
            }
        }

        Ok(Some(content))
    }

    fn is_applicable(&self, filename: &str) -> bool {
        self.converters.iter().any(|c| c.is_applicable(filename))
    }
}

impl Debug for CombinedPkgFileConverter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CombinedPkgFileConverter")
            .field("converters.len()", &self.converters.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct CombinedPkgFileConverterError(Box<dyn std::error::Error>);

impl Display for CombinedPkgFileConverterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for CombinedPkgFileConverterError {}

impl From<Box<dyn std::error::Error>> for CombinedPkgFileConverterError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self(error)
    }
}

impl Deref for CombinedPkgFileConverterError {
    type Target = Box<dyn std::error::Error>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

trait CombinedPkgFileConverterAdapter {
    fn convert(
        &self,
        filename: &str,
        content: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>>;

    fn is_applicable(&self, filename: &str) -> bool;
}

impl<E: std::error::Error + 'static> CombinedPkgFileConverterAdapter
    for dyn PkgFileConverter<Error = E>
{
    fn convert(
        &self,
        filename: &str,
        content: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        self.convert(filename, content).map_err(|err| err.into())
    }

    fn is_applicable(&self, filename: &str) -> bool {
        self.is_applicable(filename)
    }
}

struct CombinedPkgFileConverterAdapterImpl<C: PkgFileConverter>(C);

impl<C: PkgFileConverter> CombinedPkgFileConverterAdapter
    for CombinedPkgFileConverterAdapterImpl<C>
{
    fn convert(
        &self,
        filename: &str,
        content: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        Ok(self.0.convert(filename, content)?)
    }

    fn is_applicable(&self, filename: &str) -> bool {
        self.0.is_applicable(filename)
    }
}

impl<C: PkgFileConverter> From<C> for CombinedPkgFileConverterAdapterImpl<C> {
    fn from(value: C) -> Self {
        Self(value)
    }
}
