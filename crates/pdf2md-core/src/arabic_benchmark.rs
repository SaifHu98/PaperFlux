use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArabicQualityScore {
    pub unicode_accuracy: f32,
    pub char_accuracy: f32,
    pub word_accuracy: f32,
    pub paragraph_accuracy: f32,
    pub reading_order_accuracy: f32,
    pub rtl_accuracy: f32,
    pub heading_accuracy: f32,
    pub list_accuracy: f32,
    pub table_accuracy: f32,
    pub ocr_accuracy: f32,
    pub mixed_script_accuracy: f32,
    pub markdown_structural_accuracy: f32,
}

impl Default for ArabicQualityScore {
    fn default() -> Self {
        Self {
            unicode_accuracy: 1.0,
            char_accuracy: 1.0,
            word_accuracy: 1.0,
            paragraph_accuracy: 1.0,
            reading_order_accuracy: 1.0,
            rtl_accuracy: 1.0,
            heading_accuracy: 1.0,
            list_accuracy: 1.0,
            table_accuracy: 1.0,
            ocr_accuracy: 1.0,
            mixed_script_accuracy: 1.0,
            markdown_structural_accuracy: 1.0,
        }
    }
}

impl ArabicQualityScore {
    /// Computes the 12-component weighted composite ArabicQualityScore
    pub fn composite_score(&self) -> f32 {
        let weighted_sum = (self.unicode_accuracy * 0.10)
            + (self.char_accuracy * 0.10)
            + (self.word_accuracy * 0.10)
            + (self.paragraph_accuracy * 0.08)
            + (self.reading_order_accuracy * 0.10)
            + (self.rtl_accuracy * 0.08)
            + (self.heading_accuracy * 0.08)
            + (self.list_accuracy * 0.06)
            + (self.table_accuracy * 0.08)
            + (self.ocr_accuracy * 0.08)
            + (self.mixed_script_accuracy * 0.07)
            + (self.markdown_structural_accuracy * 0.07);

        (weighted_sum * 1000.0).round() / 1000.0
    }

    /// Evaluates if the score satisfies the production release gate (Composite >= 0.95 and all sub-scores >= 0.90)
    pub fn satisfies_release_gate(&self) -> bool {
        self.composite_score() >= 0.95
            && self.unicode_accuracy >= 0.90
            && self.char_accuracy >= 0.90
            && self.word_accuracy >= 0.90
            && self.reading_order_accuracy >= 0.90
            && self.rtl_accuracy >= 0.90
            && self.table_accuracy >= 0.90
            && self.markdown_structural_accuracy >= 0.90
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArabicBenchmarkRecord {
    pub genre: String,
    pub input_size_bytes: usize,
    pub page_count: usize,
    pub latency_ms: f32,
    pub quality_score: ArabicQualityScore,
    pub passed_release_gate: bool,
}
