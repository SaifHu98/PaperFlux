use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionDiagnostics {
    pub total_pages: usize,
    pub text_pages: usize,
    pub ocr_pages: usize,
    pub tables_detected: usize,
    pub images_extracted: usize,
    pub overall_confidence: f32,
    pub confidence_breakdown: ConfidenceScores,
    pub pages: Vec<PageDiagnostics>,
    pub warnings: Vec<ConversionWarning>,
    pub stats: ProcessingStats,
}

impl Default for ConversionDiagnostics {
    fn default() -> Self {
        Self {
            total_pages: 0,
            text_pages: 0,
            ocr_pages: 0,
            tables_detected: 0,
            images_extracted: 0,
            overall_confidence: 1.0,
            confidence_breakdown: ConfidenceScores::default(),
            pages: Vec::new(),
            warnings: Vec::new(),
            stats: ProcessingStats::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScores {
    pub text_confidence: f32,
    pub reading_order_confidence: f32,
    pub heading_confidence: f32,
    pub table_confidence: f32,
    pub ocr_confidence: f32,
    pub language_confidence: f32,
    pub layout_confidence: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stitch_confidence: Option<f32>,
}

impl Default for ConfidenceScores {
    fn default() -> Self {
        Self {
            text_confidence: 1.0,
            reading_order_confidence: 1.0,
            heading_confidence: 1.0,
            table_confidence: 1.0,
            ocr_confidence: 1.0,
            language_confidence: 1.0,
            layout_confidence: 1.0,
            stitch_confidence: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDiagnostics {
    pub page_number: usize,
    pub is_scanned: bool,
    pub ocr_applied: bool,
    pub glyph_count: usize,
    pub image_count: usize,
    pub table_count: usize,
    pub detected_language: Option<String>,
    pub confidence: f32,
    pub reading_order_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionWarning {
    pub code: String,
    pub message: String,
    pub page: Option<usize>,
    pub category: WarningCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningCategory {
    FontDecoding,
    MalformedStructure,
    DecompressionBombAttempt,
    MissingGlyphMap,
    LowConfidenceLayout,
    TableAmbiguity,
    OcrFailure,
    SecurityLimitReached,
    UnsupportedFeature,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub parse_time_ms: u64,
    pub layout_time_ms: u64,
    pub render_time_ms: u64,
    pub total_time_ms: u64,
    pub memory_peak_bytes: usize,
}
