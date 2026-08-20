use crate::config::Config;
use crate::error::{ConversionError, ConversionResult};
use crate::pipeline::Pipeline;
use std::fs;
use std::io::Read;
use std::path::Path;

pub struct Converter {
    pipeline: Pipeline,
}

impl Converter {
    pub fn new(config: Config) -> Self {
        Self {
            pipeline: Pipeline::new(config),
        }
    }

    pub fn convert_bytes(&self, bytes: &[u8]) -> Result<ConversionResult, ConversionError> {
        self.pipeline.convert_bytes(bytes)
    }

    pub fn convert_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<ConversionResult, ConversionError> {
        let bytes = fs::read(path)?;
        self.convert_bytes(&bytes)
    }

    pub fn convert_reader<R: Read>(
        &self,
        mut reader: R,
    ) -> Result<ConversionResult, ConversionError> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        self.convert_bytes(&buffer)
    }
}
