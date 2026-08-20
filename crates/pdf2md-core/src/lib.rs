pub mod arabic_benchmark;
pub mod buffer_pool;
pub mod cache;
pub mod config;
pub mod converter;
pub mod error;
pub mod pipeline;
pub mod profile;
pub mod scheduler;

pub use arabic_benchmark::*;
pub use buffer_pool::*;
pub use cache::*;
pub use config::*;
pub use converter::*;
pub use error::*;
pub use pipeline::*;
pub use profile::*;
pub use scheduler::*;

// Re-export core AST and markdown types for consumer convenience
pub use pdf2md_ast::{ConversionDiagnostics, Document, DocumentMetadata, Node, Section};
pub use pdf2md_markdown::{MarkdownDialect, PageBreakStyle, RenderOptions};
pub use pdf2md_ocr::{OCRProvider, OcrMode};
