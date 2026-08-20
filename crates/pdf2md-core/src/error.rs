use thiserror::Error;
use pdf2md_ast::{ConversionDiagnostics, Document};
use pdf2md_ocr::OcrError;
use pdf2md_pdf::document::PdfError;
use pdf2md_pdf::security::SecurityError;

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("PDF parsing error: {0}")]
    Pdf(#[from] PdfError),

    #[error("Security violation: {0}")]
    Security(#[from] SecurityError),

    #[error("OCR error: {0}")]
    Ocr(#[from] OcrError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Conversion timed out after {0} seconds")]
    Timeout(u64),

    #[error("Generic conversion error: {0}")]
    Generic(String),
}

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub markdown: String,
    pub document: Document,
    pub diagnostics: ConversionDiagnostics,
}
