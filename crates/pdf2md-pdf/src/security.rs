use std::collections::HashSet;
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("File size ({0} bytes) exceeded maximum allowed limit of {1} bytes")]
    FileSizeExceeded(usize, usize),

    #[error("Decompressed stream size ({0} bytes) exceeded maximum allowed limit of {1} bytes")]
    StreamSizeExceeded(usize, usize),

    #[error("Decompression bomb detected: expansion ratio ({0:.1}x) exceeded limit ({1:.1}x)")]
    DecompressionBomb(f32, f32),

    #[error("Exceeded maximum page limit: document has {0} pages, limit is {1}")]
    PageCountExceeded(usize, usize),

    #[error("Exceeded maximum page limit ({0})")]
    PageLimitExceeded(usize),

    #[error("Cyclic reference loop detected on object {0}")]
    CyclicReference(u32),

    #[error("Object nesting depth exceeded {0}")]
    NestingDepthExceeded(usize),

    #[error("External references not permitted")]
    ExternalReferenceBlocked,

    #[error("Security limit violated: {0}")]
    LimitExceeded(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityLimits {
    /// Maximum allowed input file size in bytes (default: 512MB)
    pub max_file_size_bytes: usize,

    /// Maximum allowed decompressed bytes for a single content stream (default: 32MB)
    pub max_decompressed_stream_bytes: usize,

    /// Maximum allowed decompression expansion ratio (e.g. 100.0 means 1KB compressed cannot expand beyond 100KB)
    pub max_decompression_ratio: f32,

    /// Maximum number of pages allowed to be processed in a single document (default: 5000)
    pub max_pages: usize,

    /// Maximum total number of indirect objects allowed in a document (default: 100,000)
    pub max_objects_count: usize,

    /// Maximum nesting depth for PDF object graphs, arrays, and dictionaries (default: 64)
    pub max_object_depth: usize,

    /// Maximum raster image width in pixels to prevent memory bombs (default: 10,000 px)
    pub max_image_width: u32,

    /// Maximum raster image height in pixels to prevent memory bombs (default: 10,000 px)
    pub max_image_height: u32,

    /// Total conversion timeout in seconds (default: 60)
    pub timeout_seconds: u64,

    /// Whether to allow execution of embedded JavaScript (Always false for security)
    pub allow_javascript: bool,

    /// Whether to allow external URI / file references (Always false for security)
    pub allow_external_references: bool,
}

impl Default for SecurityLimits {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 512 * 1024 * 1024, // 512 MB
            max_decompressed_stream_bytes: 32 * 1024 * 1024, // 32 MB
            max_decompression_ratio: 100.0,
            max_pages: 5000,
            max_objects_count: 100_000,
            max_object_depth: 64,
            max_image_width: 10_000,
            max_image_height: 10_000,
            timeout_seconds: 60,
            allow_javascript: false,
            allow_external_references: false,
        }
    }
}

pub struct CycleDetector {
    visited: HashSet<u32>,
    max_depth: usize,
    current_depth: usize,
}

impl CycleDetector {
    pub fn new(max_depth: usize) -> Self {
        Self {
            visited: HashSet::new(),
            max_depth,
            current_depth: 0,
        }
    }

    pub fn enter_object(&mut self, obj_id: u32) -> Result<(), SecurityError> {
        if self.current_depth >= self.max_depth {
            return Err(SecurityError::NestingDepthExceeded(self.max_depth));
        }

        if !self.visited.insert(obj_id) {
            return Err(SecurityError::CyclicReference(obj_id));
        }

        self.current_depth += 1;
        Ok(())
    }

    pub fn exit_object(&mut self, obj_id: u32) {
        self.visited.remove(&obj_id);
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.visited.clear();
        self.current_depth = 0;
    }
}
