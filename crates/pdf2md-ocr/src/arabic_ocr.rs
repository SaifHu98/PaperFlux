use pdf2md_ast::geometry::{BoundingBox, WritingDirection};
use pdf2md_pdf::elements::RawPage;
use pdf2md_text::bidi::is_rtl_char;
use pdf2md_text::quality::TextQualityAssessor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArabicDialectHint {
    GeneralArabic, // ar
    SaudiArabia,   // ar-SA
    Egypt,         // ar-EG
    Iraq,          // ar-IQ
    UAE,           // ar-AE
    Morocco,       // ar-MA
    Algeria,       // ar-DZ
    Tunisia,       // ar-TN
    Jordan,        // ar-JO
    Syria,         // ar-SY
    Lebanon,       // ar-LB
}

impl ArabicDialectHint {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().replace('_', "-").as_str() {
            "ar-sa" | "sa" => Self::SaudiArabia,
            "ar-eg" | "eg" => Self::Egypt,
            "ar-iq" | "iq" => Self::Iraq,
            "ar-ae" | "ae" => Self::UAE,
            "ar-ma" | "ma" => Self::Morocco,
            "ar-dz" | "dz" => Self::Algeria,
            "ar-tn" | "tn" => Self::Tunisia,
            "ar-jo" | "jo" => Self::Jordan,
            "ar-sy" | "sy" => Self::Syria,
            "ar-lb" | "lb" => Self::Lebanon,
            _ => Self::GeneralArabic,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArabicPagePreflight {
    pub orientation_degrees: f32,
    pub skew_degrees: f32,
    pub estimated_dpi: u32,
    pub has_handwriting_indicators: bool,
    pub is_damaged_or_noisy: bool,
    pub is_bilingual: bool,
    pub is_calligraphic: bool,
    pub calligraphic_script: Option<String>,
    pub native_text_quality: f32,
    pub arabic_char_count: usize,
    pub latin_char_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArabicOcrWord {
    pub text: String,
    pub bbox: BoundingBox,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArabicOcrLine {
    pub words: Vec<ArabicOcrWord>,
    pub text: String,
    pub bbox: BoundingBox,
    pub confidence: f32,
    pub direction: WritingDirection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArabicOcrBlock {
    pub lines: Vec<ArabicOcrLine>,
    pub bbox: BoundingBox,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArabicOcrPage {
    pub blocks: Vec<ArabicOcrBlock>,
    pub full_text: String,
    pub overall_confidence: f32,
    pub dialect: ArabicDialectHint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArabicOcrDecision {
    /// Digital text is clean and high-fidelity; OCR is unnecessary (0% overhead)
    SkipOcr { reason: String },

    /// Document is scanned, image-only, or has unmapped/corrupted font CMaps
    RequireOcr {
        preflight: ArabicPagePreflight,
        reason: String,
    },

    /// Both native text and OCR are available; apply confidence-weighted multi-criteria fusion
    FuseStreams {
        native_quality: f32,
        expected_ocr_weight: f32,
    },
}

pub struct ArabicOcrDecisionEngine;

impl ArabicOcrDecisionEngine {
    /// Evaluates whether an Arabic page requires OCR, can skip OCR, or requires stream fusion
    pub fn evaluate_page(page: &RawPage, dialect: ArabicDialectHint) -> ArabicOcrDecision {
        let spans = &page.text_spans;
        let images = &page.images;

        // 1. Check calligraphic script indicators (Nastaliq, Diwani, Thuluth, Ruq'ah)
        let calligraphy = crate::calligraphy::CalligraphyDetector::detect(page);

        // 2. Check if page is image-only (scanned document)
        if spans.is_empty() {
            let preflight = ArabicPagePreflight {
                orientation_degrees: 0.0,
                skew_degrees: 0.0,
                estimated_dpi: 300,
                has_handwriting_indicators: false,
                is_damaged_or_noisy: false,
                is_bilingual: false,
                is_calligraphic: calligraphy.is_calligraphic,
                calligraphic_script: calligraphy.script_type.map(|s| format!("{:?}", s)),
                native_text_quality: 0.0,
                arabic_char_count: 0,
                latin_char_count: 0,
            };

            return ArabicOcrDecision::RequireOcr {
                preflight,
                reason: format!(
                    "Scanned image-only Arabic page detected (contains {} raster images)",
                    images.len()
                ),
            };
        }

        // 3. Extract and analyze native text quality
        let full_text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let quality_report = TextQualityAssessor::assess(&full_text);

        let mut arabic_count = 0;
        let mut latin_count = 0;

        for c in full_text.chars() {
            if is_rtl_char(c) {
                arabic_count += 1;
            } else if c.is_ascii_alphabetic() {
                latin_count += 1;
            }
        }

        let is_bilingual = arabic_count > 0
            && latin_count > 0
            && (latin_count as f32 / (arabic_count + latin_count) as f32) > 0.15;
        let native_quality = quality_report.quality_score;

        let estimated_dpi = if calligraphy.is_calligraphic {
            calligraphy.recommended_dpi
        } else {
            150
        };

        let preflight = ArabicPagePreflight {
            orientation_degrees: 0.0,
            skew_degrees: 0.0,
            estimated_dpi,
            has_handwriting_indicators: false,
            is_damaged_or_noisy: quality_report.replacement_char_count > 0,
            is_bilingual,
            is_calligraphic: calligraphy.is_calligraphic,
            calligraphic_script: calligraphy.script_type.map(|s| format!("{:?}", s)),
            native_text_quality: native_quality,
            arabic_char_count: arabic_count,
            latin_char_count: latin_count,
        };

        // 4. Heavily corrupted font stream / PUA leakage -> Require OCR
        if native_quality < 0.50 || quality_report.pua_char_count > 10 {
            return ArabicOcrDecision::RequireOcr {
                preflight,
                reason: format!(
                    "Corrupted font encoding detected (Quality: {:.2}, {} unmapped PUA glyphs)",
                    native_quality, quality_report.pua_char_count
                ),
            };
        }

        // 5. Calligraphic script with raster image, moderate quality, or high confidence -> Require 300+ DPI OCR / Fusion
        if calligraphy.is_calligraphic
            && (!images.is_empty() || native_quality < 0.95 || calligraphy.confidence >= 0.75)
        {
            return ArabicOcrDecision::RequireOcr {
                preflight,
                reason: format!(
                    "Calligraphic script ({}) detected requiring {} DPI OCR resolution: {}",
                    calligraphy
                        .script_type
                        .map(|s| format!("{:?}", s))
                        .unwrap_or_else(|| "Calligraphy".to_string()),
                    estimated_dpi,
                    calligraphy.reason
                ),
            };
        }

        // 6. High quality clean digital text -> Skip OCR completely
        if native_quality >= 0.88 && quality_report.pua_char_count == 0 {
            return ArabicOcrDecision::SkipOcr {
                reason: format!(
                    "High-fidelity Arabic digital text (Quality: {:.2}, {} chars, dialect: {:?})",
                    native_quality, arabic_count, dialect
                ),
            };
        }

        // 7. Intermediate quality -> Fuse native and OCR streams
        ArabicOcrDecision::FuseStreams {
            native_quality,
            expected_ocr_weight: 1.0 - native_quality,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionOutput {
    pub fused_text: String,
    pub fusion_confidence: f32,
    pub native_chosen_chars: usize,
    pub ocr_chosen_chars: usize,
    pub stream_source: &'static str,
}

pub struct ArabicOcrFusionEngine;

impl ArabicOcrFusionEngine {
    /// Merges native extracted text and Arabic OCR text using multi-criteria quality scoring
    pub fn fuse_streams(
        native_text: &str,
        ocr_text: &str,
        native_quality: f32,
        ocr_confidence: f32,
    ) -> (String, f32) {
        let output = Self::fuse_character_by_character(
            native_text,
            ocr_text,
            native_quality,
            ocr_confidence,
        );
        (output.fused_text, output.fusion_confidence)
    }

    /// Performs granular character-by-character and token-level fusion
    pub fn fuse_character_by_character(
        native_text: &str,
        ocr_text: &str,
        native_quality: f32,
        ocr_confidence: f32,
    ) -> FusionOutput {
        // Edge Case 1: Both empty
        if native_text.trim().is_empty() && ocr_text.trim().is_empty() {
            return FusionOutput {
                fused_text: String::new(),
                fusion_confidence: 1.0,
                native_chosen_chars: 0,
                ocr_chosen_chars: 0,
                stream_source: "empty",
            };
        }

        // Edge Case 2: One empty
        if native_text.trim().is_empty() {
            return FusionOutput {
                fused_text: ocr_text.to_string(),
                fusion_confidence: ocr_confidence,
                native_chosen_chars: 0,
                ocr_chosen_chars: ocr_text.chars().count(),
                stream_source: "ocr",
            };
        }
        if ocr_text.trim().is_empty() {
            return FusionOutput {
                fused_text: native_text.to_string(),
                fusion_confidence: native_quality,
                native_chosen_chars: native_text.chars().count(),
                ocr_chosen_chars: 0,
                stream_source: "native",
            };
        }

        // Edge Case 3: Identical streams
        if native_text.trim() == ocr_text.trim() {
            let n_chars = native_text.chars().count();
            return FusionOutput {
                fused_text: native_text.to_string(),
                fusion_confidence: native_quality.max(ocr_confidence),
                native_chosen_chars: n_chars,
                ocr_chosen_chars: 0,
                stream_source: "identical",
            };
        }

        // Clear winner cases
        let corrupted_count = native_text
            .chars()
            .filter(|&c| c == '\u{FFFD}' || ('\u{E000}'..='\u{F8FF}').contains(&c))
            .count();
        let total_native_chars = native_text.chars().count().max(1);
        let corrupted_ratio = (corrupted_count as f32) / (total_native_chars as f32);

        if native_quality >= 0.92 && corrupted_count == 0 {
            let n_chars = native_text.chars().count();
            return FusionOutput {
                fused_text: native_text.to_string(),
                fusion_confidence: native_quality,
                native_chosen_chars: n_chars,
                ocr_chosen_chars: 0,
                stream_source: "native",
            };
        }

        if (native_quality < 0.20 || corrupted_ratio > 0.50) && ocr_confidence >= 0.85 {
            let ocr_chars = ocr_text.chars().count();
            return FusionOutput {
                fused_text: ocr_text.to_string(),
                fusion_confidence: ocr_confidence,
                native_chosen_chars: 0,
                ocr_chosen_chars: ocr_chars,
                stream_source: "ocr",
            };
        }

        // Character-by-character alignment and fusion
        let native_chars: Vec<char> = native_text.chars().collect();
        let ocr_chars: Vec<char> = ocr_text.chars().collect();

        let mut fused = String::new();
        let mut nat_count = 0;
        let mut ocr_count = 0;

        let max_len = native_chars.len().max(ocr_chars.len());

        for i in 0..max_len {
            let n_ch = native_chars.get(i).copied();
            let o_ch = ocr_chars.get(i).copied();

            match (n_ch, o_ch) {
                (Some(nc), Some(oc)) => {
                    if nc == oc {
                        fused.push(nc);
                        nat_count += 1;
                    } else if nc == '\u{FFFD}' || ('\u{E000}'..='\u{F8FF}').contains(&nc) {
                        fused.push(oc);
                        ocr_count += 1;
                    } else if oc == '\u{FFFD}' {
                        fused.push(nc);
                        nat_count += 1;
                    } else if native_quality >= ocr_confidence {
                        fused.push(nc);
                        nat_count += 1;
                    } else {
                        fused.push(oc);
                        ocr_count += 1;
                    }
                }
                (Some(nc), None) => {
                    if nc != '\u{FFFD}' && !('\u{E000}'..='\u{F8FF}').contains(&nc) {
                        fused.push(nc);
                        nat_count += 1;
                    }
                }
                (None, Some(oc)) => {
                    if oc != '\u{FFFD}' {
                        fused.push(oc);
                        ocr_count += 1;
                    }
                }
                (None, None) => break,
            }
        }

        let composite_conf = ((native_quality * (nat_count as f32))
            + (ocr_confidence * (ocr_count as f32)))
            / ((nat_count + ocr_count).max(1) as f32);

        // Repairing corrupted glyphs elevates fused stream quality
        let post_corrupted_count = fused
            .chars()
            .filter(|&c| c == '\u{FFFD}' || ('\u{E000}'..='\u{F8FF}').contains(&c))
            .count();
        let quality_boost = if corrupted_count > 0 && post_corrupted_count == 0 {
            0.15
        } else {
            0.0
        };
        let final_conf = (composite_conf + quality_boost).min(ocr_confidence.max(native_quality));

        FusionOutput {
            fused_text: fused,
            fusion_confidence: final_conf.clamp(0.0, 1.0),
            native_chosen_chars: nat_count,
            ocr_chosen_chars: ocr_count,
            stream_source: "fused",
        }
    }
}
