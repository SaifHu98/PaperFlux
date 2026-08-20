use pdf2md_pdf::elements::RawPage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalligraphicScriptType {
    Nastaliq,
    Diwani,
    Thuluth,
    Ruqah,
    GenericCalligraphic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalligraphyDetectionResult {
    pub is_calligraphic: bool,
    pub script_type: Option<CalligraphicScriptType>,
    pub confidence: f32,
    pub recommended_dpi: u32,
    pub reason: String,
}

pub struct CalligraphyDetector;

impl CalligraphyDetector {
    /// Evaluates font names and geometric bounding box patterns to identify calligraphic scripts
    pub fn detect(page: &RawPage) -> CalligraphyDetectionResult {
        // 1. Check font names
        for span in &page.text_spans {
            let font = span.font_name.to_lowercase();
            if font.contains("nastaliq")
                || font.contains("nastaleeq")
                || font.contains("nafees")
                || font.contains("jameel")
                || font.contains("kasheeda")
                || font.contains("shekasteh")
                || font.contains("farsi_nastaliq")
                || font.contains("iranian")
            {
                return CalligraphyDetectionResult {
                    is_calligraphic: true,
                    script_type: Some(CalligraphicScriptType::Nastaliq),
                    confidence: 0.95,
                    recommended_dpi: 300,
                    reason: format!("Nastaliq font detected: {}", span.font_name),
                };
            }
            if font.contains("diwan") || font.contains("diwani") {
                return CalligraphyDetectionResult {
                    is_calligraphic: true,
                    script_type: Some(CalligraphicScriptType::Diwani),
                    confidence: 0.95,
                    recommended_dpi: 300,
                    reason: format!("Diwani font detected: {}", span.font_name),
                };
            }
            if font.contains("thuluth") {
                return CalligraphyDetectionResult {
                    is_calligraphic: true,
                    script_type: Some(CalligraphicScriptType::Thuluth),
                    confidence: 0.95,
                    recommended_dpi: 300,
                    reason: format!("Thuluth font detected: {}", span.font_name),
                };
            }
            if font.contains("ruqaa") || font.contains("ruq'ah") || font.contains("reqa") {
                return CalligraphyDetectionResult {
                    is_calligraphic: true,
                    script_type: Some(CalligraphicScriptType::Ruqah),
                    confidence: 0.95,
                    recommended_dpi: 300,
                    reason: format!("Ruq'ah font detected: {}", span.font_name),
                };
            }
        }

        // 2. Geometric analysis: high intra-line vertical overlap and non-horizontal baseline clustering
        let spans = &page.text_spans;
        if spans.len() >= 4 {
            let mut overlapping_pairs = 0;
            let mut total_compared = 0;

            for i in 0..spans.len().saturating_sub(1) {
                let s1 = &spans[i];
                let s2 = &spans[i + 1];

                // Check if spans horizontally overlap significantly but have different vertical baselines
                let x_overlap =
                    s1.bbox.x_min().max(s2.bbox.x_min()) < s1.bbox.x_max().min(s2.bbox.x_max());
                let dy = (s1.bbox.y_min() - s2.bbox.y_min()).abs();

                if x_overlap && dy > 2.0 && dy < s1.font_size * 1.5 {
                    overlapping_pairs += 1;
                }
                total_compared += 1;
            }

            if total_compared > 0 {
                let overlap_ratio = overlapping_pairs as f32 / total_compared as f32;
                if overlap_ratio >= 0.25 {
                    return CalligraphyDetectionResult {
                        is_calligraphic: true,
                        script_type: Some(CalligraphicScriptType::GenericCalligraphic),
                        confidence: 0.85,
                        recommended_dpi: 300,
                        reason: format!(
                            "High vertical overlap ({:.1}%) and cascading baselines characteristic of calligraphic text",
                            overlap_ratio * 100.0
                        ),
                    };
                }
            }
        }

        CalligraphyDetectionResult {
            is_calligraphic: false,
            script_type: None,
            confidence: 0.90,
            recommended_dpi: 150,
            reason: "Standard horizontal typography".to_string(),
        }
    }
}
