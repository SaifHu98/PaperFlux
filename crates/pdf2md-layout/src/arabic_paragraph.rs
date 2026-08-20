use pdf2md_pdf::elements::TextSpan;
use pdf2md_text::arabic::context::ArabicProcessingContext;
use pdf2md_text::arabic::pipeline::ArabicTextPipeline;
use serde::{Deserialize, Serialize};

pub struct ArabicWordBoundaryDetector;

impl ArabicWordBoundaryDetector {
    /// Checks if a character connects to both right and left (dual-joining)
    pub fn is_dual_joining(ch: char) -> bool {
        matches!(
            ch,
            'ب' | 'ت' | 'ث' | 'ج' | 'ح' | 'خ' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ'
            | 'ع' | 'غ' | 'ف' | 'ق' | 'ك' | 'ل' | 'م' | 'ن' | 'ه' | 'ي' | 'ئ' | 'ـ'
            // Persian / Urdu / Kurdish extensions
            | 'پ' | 'چ' | 'گ' | 'ک' | 'ٹ' | 'ں' | 'ے' | 'ہ' | 'ڵ' | 'ۆ' | 'ێ'
            // Pashto extensions
            | 'ټ' | 'ځ' | 'څ' | 'ښ' | 'ګ' | 'ڼ' | 'ې' | 'ۍ'
            // Sindhi extensions
            | 'ٻ' | 'ٿ' | 'ڀ' | 'ٽ' | 'ڄ' | 'ڃ' | 'ڇ' | 'ڪ' | 'ڳ' | 'ڱ' | 'ڦ' | 'ڻ'
        )
    }

    /// Checks if a character connects only to the right (right-joining)
    pub fn is_right_joining(ch: char) -> bool {
        matches!(
            ch,
            'ا' | 'أ' | 'إ' | 'آ' | 'ٱ' | 'د' | 'ذ' | 'ر' | 'ز' | 'و' | 'ؤ' | 'ة' | 'ى' | 'ء'
            // Persian / Urdu / Kurdish extensions
            | 'ژ' | 'ڈ' | 'ڑ' | 'ڕ'
            // Pashto extensions
            | 'ډ' | 'ړ' | 'ږ' | 'ۀ'
            // Sindhi extensions
            | 'ڌ' | 'ڍ' | 'ڊ' | 'ڙ' | 'ڏ'
        )
    }

    /// Determines if two adjacent glyphs belong to the same Arabic word
    pub fn should_join_glyphs(prev_char: char, next_char: char, gap: f32, font_size: f32) -> bool {
        // If gap is negative or very small (kerning / ligature overlap), always join
        if gap <= 0.20 * font_size {
            return true;
        }

        // If previous character is dual-joining and gap is moderate (kashida / wide layout), join
        if Self::is_dual_joining(prev_char) && gap <= 0.38 * font_size {
            return true;
        }

        // If next character is a diacritic, always join
        if pdf2md_text::arabic::shaping::is_arabic_diacritic(next_char) {
            return true;
        }

        // Otherwise, gap represents a true inter-word space
        false
    }
}

pub struct ArabicLineReconstructor;

impl ArabicLineReconstructor {
    /// Reconstructs a single coherent line from fragmented PDF glyph spans
    pub fn reconstruct_line(spans: &[TextSpan]) -> String {
        if spans.is_empty() {
            return String::new();
        }

        // Sort spans right-to-left (X descending)
        let mut sorted = spans.to_vec();
        sorted.sort_by(|a, b| {
            b.bbox
                .x
                .partial_cmp(&a.bbox.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut line_text = String::new();
        let mut prev_span: Option<&TextSpan> = None;

        for span in &sorted {
            if let Some(prev) = prev_span {
                // Gap between previous span left edge and current span right edge (RTL geometry)
                let prev_left = prev.bbox.x;
                let current_right = span.bbox.x + span.bbox.width;
                let gap = (prev_left - current_right).max(0.0);
                let font_size = span.font_size.max(prev.font_size);

                let prev_last_char = prev.text.chars().last().unwrap_or(' ');
                let curr_first_char = span.text.chars().next().unwrap_or(' ');

                let join_without_space = ArabicWordBoundaryDetector::should_join_glyphs(
                    prev_last_char,
                    curr_first_char,
                    gap,
                    font_size,
                );

                if !join_without_space && !line_text.ends_with(' ') && !span.text.starts_with(' ') {
                    line_text.push(' ');
                }
            }

            line_text.push_str(&span.text);
            prev_span = Some(span);
        }

        let ctx = ArabicProcessingContext::default();
        let (processed, _) = ArabicTextPipeline::process(&line_text, &ctx);
        processed
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineGapStatistics {
    pub mean_gap: f32,
    pub std_dev: f32,
    pub median_gap: f32,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapClassification {
    IntraParagraph,
    InterParagraph,
}

pub struct ArabicParagraphReconstructor;

impl ArabicParagraphReconstructor {
    /// Computes statistical distribution of vertical line gaps
    pub fn analyze_line_gaps(gaps: &[f32]) -> LineGapStatistics {
        if gaps.is_empty() {
            return LineGapStatistics {
                mean_gap: 0.0,
                std_dev: 0.0,
                median_gap: 0.0,
                sample_count: 0,
            };
        }

        let count = gaps.len() as f32;
        let sum: f32 = gaps.iter().sum();
        let mean = sum / count;

        let variance: f32 = gaps.iter().map(|&g| (g - mean).powi(2)).sum::<f32>() / count;
        let std_dev = variance.sqrt();

        let mut sorted = gaps.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = sorted[sorted.len() / 2];

        LineGapStatistics {
            mean_gap: mean,
            std_dev,
            median_gap: median,
            sample_count: gaps.len(),
        }
    }

    /// Classifies a vertical line gap using statistical clustering
    pub fn classify_gap(
        gap: f32,
        avg_height: f32,
        stats: &LineGapStatistics,
        threshold_override: Option<f32>,
    ) -> GapClassification {
        let threshold = threshold_override.unwrap_or(1.4);

        if stats.sample_count >= 3 && stats.std_dev > 0.5 {
            // Statistical cluster boundary: gaps > median + 1.2 * std_dev represent paragraph breaks
            let cluster_boundary =
                stats.median_gap + (stats.std_dev * 1.2).max(avg_height * (threshold - 1.0));
            if gap > cluster_boundary {
                GapClassification::InterParagraph
            } else {
                GapClassification::IntraParagraph
            }
        } else {
            // Standard height-multiplier fallback
            if gap > avg_height * threshold {
                GapClassification::InterParagraph
            } else {
                GapClassification::IntraParagraph
            }
        }
    }

    /// Merges multiple reconstructed lines into semantic paragraphs
    pub fn reconstruct_paragraphs(lines: &[String]) -> Vec<String> {
        let mut paragraphs = Vec::new();
        let mut current_para = String::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if !current_para.is_empty() {
                    paragraphs.push(current_para.clone());
                    current_para.clear();
                }
                continue;
            }

            // Check if line starts a new clause, heading, or bullet item
            let is_new_clause = trimmed.starts_with("المادة ")
                || trimmed.starts_with("الفصل ")
                || trimmed.starts_with("الباب ")
                || trimmed.starts_with("أولاً:")
                || trimmed.starts_with("ثانياً:")
                || trimmed.starts_with('•')
                || trimmed.starts_with('-')
                || trimmed.starts_with("1.")
                || trimmed.starts_with("١.");

            if is_new_clause && !current_para.is_empty() {
                paragraphs.push(current_para.clone());
                current_para.clear();
                current_para.push_str(trimmed);
            } else if current_para.is_empty() {
                current_para.push_str(trimmed);
            } else {
                current_para.push(' ');
                current_para.push_str(trimmed);
            }
        }

        if !current_para.is_empty() {
            paragraphs.push(current_para);
        }

        paragraphs
    }

    /// Reconstructs paragraphs using statistical vertical spacing clustering
    pub fn reconstruct_paragraphs_with_geometry(
        lines: &[(String, f32, f32)], // (text, y, height)
        threshold_override: Option<f32>,
    ) -> Vec<String> {
        if lines.is_empty() {
            return Vec::new();
        }

        // Calculate line gaps
        let mut gaps = Vec::new();
        for i in 0..lines.len().saturating_sub(1) {
            let gap = (lines[i + 1].1 - lines[i].1).abs();
            gaps.push(gap);
        }

        let stats = Self::analyze_line_gaps(&gaps);
        let mut paragraphs = Vec::new();
        let mut current_para = String::new();

        for (i, (text, y, height)) in lines.iter().enumerate() {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                if !current_para.is_empty() {
                    paragraphs.push(current_para.clone());
                    current_para.clear();
                }
                continue;
            }

            if i > 0 {
                let prev_y = lines[i - 1].1;
                let prev_height = lines[i - 1].2;
                let gap = (y - prev_y).abs();
                let avg_height = (height + prev_height) / 2.0;

                let classification =
                    Self::classify_gap(gap, avg_height, &stats, threshold_override);
                if classification == GapClassification::InterParagraph && !current_para.is_empty() {
                    paragraphs.push(current_para.clone());
                    current_para.clear();
                }
            }

            // Check if line starts a new clause
            let is_new_clause = trimmed.starts_with("المادة ")
                || trimmed.starts_with("الفصل ")
                || trimmed.starts_with("الباب ")
                || trimmed.starts_with("أولاً:")
                || trimmed.starts_with("ثانياً:")
                || trimmed.starts_with('•')
                || trimmed.starts_with('-')
                || trimmed.starts_with("1.")
                || trimmed.starts_with("١.");

            if is_new_clause && !current_para.is_empty() {
                paragraphs.push(current_para.clone());
                current_para.clear();
                current_para.push_str(trimmed);
            } else if current_para.is_empty() {
                current_para.push_str(trimmed);
            } else {
                current_para.push(' ');
                current_para.push_str(trimmed);
            }
        }

        if !current_para.is_empty() {
            paragraphs.push(current_para);
        }

        paragraphs
    }
}
