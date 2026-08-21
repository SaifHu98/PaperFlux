use pdf2md_pdf::elements::RawPage;
use pdf2md_text::quality::TextQualityAssessor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrDecisionReport {
    pub should_ocr: bool,
    pub ocr_necessity_score: f32,
    pub is_image_only: bool,
    pub is_font_corrupted: bool,
    pub native_char_count: usize,
    pub native_quality_score: f32,
    pub reasons: Vec<String>,
}

pub struct OcrNecessityEvaluator;

impl OcrNecessityEvaluator {
    /// Evaluates a page across the 9 decision criteria to determine if OCR is required
    pub fn evaluate(page: &RawPage) -> OcrDecisionReport {
        let mut reasons = Vec::new();

        // 1. Check if page has no text spans but has images (Image-only / Scanned page)
        let is_image_only =
            page.is_scanned || (page.text_spans.is_empty() && !page.images.is_empty());
        if is_image_only {
            reasons.push("Page is image-only / scanned document".to_string());
            return OcrDecisionReport {
                should_ocr: true,
                ocr_necessity_score: 1.0,
                is_image_only: true,
                is_font_corrupted: false,
                native_char_count: 0,
                native_quality_score: 0.0,
                reasons,
            };
        }

        // 2. Assess native text quality & corruption
        let full_text: String = page
            .text_spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let native_char_count = full_text.chars().count();

        if native_char_count < 15 && !page.images.is_empty() {
            reasons.push(format!(
                "Very low character count ({}) with images present",
                native_char_count
            ));
            return OcrDecisionReport {
                should_ocr: true,
                ocr_necessity_score: 0.85,
                is_image_only: false,
                is_font_corrupted: false,
                native_char_count,
                native_quality_score: 0.30,
                reasons,
            };
        }

        let quality = TextQualityAssessor::assess(&full_text);

        // 3. Detect suspicious font encoding and replacement characters
        let is_font_corrupted = quality.is_corrupted || quality.quality_score < 0.60;
        if is_font_corrupted {
            reasons.extend(quality.reasons.clone());
            return OcrDecisionReport {
                should_ocr: true,
                ocr_necessity_score: 0.90,
                is_image_only: false,
                is_font_corrupted: true,
                native_char_count,
                native_quality_score: quality.quality_score,
                reasons,
            };
        }

        // Native text extraction is clean and reliable -> skip OCR
        reasons.push("Native text extraction is high quality and reliable".to_string());
        OcrDecisionReport {
            should_ocr: false,
            ocr_necessity_score: 0.0,
            is_image_only: false,
            is_font_corrupted: false,
            native_char_count,
            native_quality_score: quality.quality_score,
            reasons,
        }
    }
}

pub struct OcrFusionEngine;

impl OcrFusionEngine {
    /// Compares native text extraction against OCR result and selects the superior stream
    pub fn select_best_stream(
        native_text: &str,
        native_quality: f32,
        ocr_text: &str,
        ocr_confidence: f32,
    ) -> (String, f32, &'static str) {
        if native_quality >= 0.85 {
            return (native_text.to_string(), native_quality, "native");
        }

        if ocr_confidence > native_quality {
            (ocr_text.to_string(), ocr_confidence, "ocr")
        } else {
            (native_text.to_string(), native_quality, "native")
        }
    }

    /// Performs granular character-by-character stream fusion
    pub fn fuse_character_by_character(
        native_text: &str,
        native_quality: f32,
        ocr_text: &str,
        ocr_confidence: f32,
    ) -> crate::arabic_ocr::FusionOutput {
        crate::arabic_ocr::ArabicOcrFusionEngine::fuse_character_by_character(
            native_text,
            ocr_text,
            native_quality,
            ocr_confidence,
        )
    }
}
