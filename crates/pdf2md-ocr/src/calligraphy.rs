use pdf2md_pdf::elements::{RawPage, TextSpan};
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
pub struct StatisticalGlyphMetrics {
    pub overlap_density: f32,
    pub baseline_deviation: f32,
    pub diagonal_slope_ratio: f32,
    pub mean_diagonal_slope: f32,
    pub calligraphic_confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalligraphyDetectionResult {
    pub is_calligraphic: bool,
    pub script_type: Option<CalligraphicScriptType>,
    pub confidence: f32,
    pub recommended_dpi: u32,
    pub reason: String,
    pub metrics: Option<StatisticalGlyphMetrics>,
}

pub struct CalligraphyDetector;

impl CalligraphyDetector {
    /// Evaluates font names and geometric bounding box patterns to identify calligraphic scripts
    pub fn detect(page: &RawPage) -> CalligraphyDetectionResult {
        // 1. Check explicit font names
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
                    metrics: None,
                };
            }
            if font.contains("diwan") || font.contains("diwani") {
                return CalligraphyDetectionResult {
                    is_calligraphic: true,
                    script_type: Some(CalligraphicScriptType::Diwani),
                    confidence: 0.95,
                    recommended_dpi: 300,
                    reason: format!("Diwani font detected: {}", span.font_name),
                    metrics: None,
                };
            }
            if font.contains("thuluth") {
                return CalligraphyDetectionResult {
                    is_calligraphic: true,
                    script_type: Some(CalligraphicScriptType::Thuluth),
                    confidence: 0.95,
                    recommended_dpi: 300,
                    reason: format!("Thuluth font detected: {}", span.font_name),
                    metrics: None,
                };
            }
            if font.contains("ruqaa") || font.contains("ruq'ah") || font.contains("reqa") {
                return CalligraphyDetectionResult {
                    is_calligraphic: true,
                    script_type: Some(CalligraphicScriptType::Ruqah),
                    confidence: 0.95,
                    recommended_dpi: 300,
                    reason: format!("Ruq'ah font detected: {}", span.font_name),
                    metrics: None,
                };
            }
        }

        // 2. Statistical glyph analysis (baseline deviation, overlap density, diagonal slope)
        let metrics = Self::analyze_glyph_metrics(&page.text_spans);

        if metrics.calligraphic_confidence >= 0.75 {
            let is_nastaliq = metrics.diagonal_slope_ratio >= 0.35;
            let script_type = if is_nastaliq {
                CalligraphicScriptType::Nastaliq
            } else {
                CalligraphicScriptType::GenericCalligraphic
            };

            let reason = if is_nastaliq {
                format!(
                    "Statistical Nastaliq clustering detected (Confidence: {:.2}, Diagonal Ratio: {:.1}%, Overlap: {:.1}%)",
                    metrics.calligraphic_confidence,
                    metrics.diagonal_slope_ratio * 100.0,
                    metrics.overlap_density * 100.0
                )
            } else {
                format!(
                    "Statistical calligraphic clustering detected (Confidence: {:.2}, Overlap: {:.1}%, Baseline Dev: {:.2})",
                    metrics.calligraphic_confidence,
                    metrics.overlap_density * 100.0,
                    metrics.baseline_deviation
                )
            };

            return CalligraphyDetectionResult {
                is_calligraphic: true,
                script_type: Some(script_type),
                confidence: metrics.calligraphic_confidence,
                recommended_dpi: 300,
                reason,
                metrics: Some(metrics),
            };
        }

        CalligraphyDetectionResult {
            is_calligraphic: false,
            script_type: None,
            confidence: 1.0 - metrics.calligraphic_confidence,
            recommended_dpi: 150,
            reason: "Standard horizontal typography".to_string(),
            metrics: Some(metrics),
        }
    }

    /// Measures baseline deviation, overlapping density, and diagonal slope from character bounding boxes
    pub fn analyze_glyph_metrics(spans: &[TextSpan]) -> StatisticalGlyphMetrics {
        if spans.len() < 3 {
            return StatisticalGlyphMetrics {
                overlap_density: 0.0,
                baseline_deviation: 0.0,
                diagonal_slope_ratio: 0.0,
                mean_diagonal_slope: 0.0,
                calligraphic_confidence: 0.0,
            };
        }

        let mut overlapping_pairs = 0;
        let mut diagonal_pairs = 0;
        let mut total_compared = 0;
        let mut slopes = Vec::new();
        let y_values: Vec<f32> = spans.iter().map(|s| s.bbox.y_min()).collect();

        for i in 0..spans.len().saturating_sub(1) {
            let s1 = &spans[i];
            let s2 = &spans[i + 1];

            // Horizontal overlap check
            let x_overlap =
                s1.bbox.x_min().max(s2.bbox.x_min()) < s1.bbox.x_max().min(s2.bbox.x_max());
            let dx = (s2.bbox.x_min() - s1.bbox.x_min()).abs().max(1.0);
            let dy = (s2.bbox.y_min() - s1.bbox.y_min()).abs();

            if x_overlap && dy > 1.5 && dy < s1.font_size * 2.0 {
                overlapping_pairs += 1;
            }

            // Diagonal baseline slope check (|dy / dx| in range [0.10, 2.5])
            let slope = dy / dx;
            if (0.10..=2.5).contains(&slope) && dy >= 1.5 {
                diagonal_pairs += 1;
                slopes.push(slope);
            }

            total_compared += 1;
        }

        let overlap_density = if total_compared > 0 {
            overlapping_pairs as f32 / total_compared as f32
        } else {
            0.0
        };

        let diagonal_slope_ratio = if total_compared > 0 {
            diagonal_pairs as f32 / total_compared as f32
        } else {
            0.0
        };

        let mean_diagonal_slope = if !slopes.is_empty() {
            slopes.iter().sum::<f32>() / slopes.len() as f32
        } else {
            0.0
        };

        // Calculate standard deviation of baseline Y positions across the cluster
        let baseline_deviation = if !y_values.is_empty() {
            let mean_y = y_values.iter().sum::<f32>() / y_values.len() as f32;
            let var =
                y_values.iter().map(|&v| (v - mean_y).powi(2)).sum::<f32>() / y_values.len() as f32;
            var.sqrt()
        } else {
            0.0
        };

        // Composite calligraphic confidence score [0.0..1.0]
        let overlap_weight = (overlap_density / 0.25).min(1.0) * 0.40;
        let diagonal_weight = (diagonal_slope_ratio / 0.30).min(1.0) * 0.45;
        let baseline_weight = (baseline_deviation / 4.0).min(1.0) * 0.15;

        let calligraphic_confidence = (overlap_weight + diagonal_weight + baseline_weight).min(1.0);

        StatisticalGlyphMetrics {
            overlap_density,
            baseline_deviation,
            diagonal_slope_ratio,
            mean_diagonal_slope,
            calligraphic_confidence,
        }
    }
}
