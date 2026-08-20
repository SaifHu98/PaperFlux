use pdf2md_pdf::elements::TextSpan;
use pdf2md_text::arabic::pipeline::ArabicTextPipeline;
use pdf2md_text::arabic::context::ArabicProcessingContext;

pub struct ArabicWordBoundaryDetector;

impl ArabicWordBoundaryDetector {
    /// Checks if a character connects to both right and left (dual-joining)
    pub fn is_dual_joining(ch: char) -> bool {
        matches!(ch,
            'ب' | 'ت' | 'ث' | 'ج' | 'ح' | 'خ' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ'
            | 'ع' | 'غ' | 'ف' | 'ق' | 'ك' | 'ل' | 'م' | 'ن' | 'ه' | 'ي' | 'ئ' | 'ـ'
            // Persian / Urdu / Kurdish extensions
            | 'پ' | 'چ' | 'گ' | 'ک' | 'ٹ' | 'ں' | 'ے' | 'ہ' | 'ڵ' | 'ۆ' | 'ێ'
        )
    }

    /// Checks if a character connects only to the right (right-joining)
    pub fn is_right_joining(ch: char) -> bool {
        matches!(ch,
            'ا' | 'أ' | 'إ' | 'آ' | 'ٱ' | 'د' | 'ذ' | 'ر' | 'ز' | 'و' | 'ؤ' | 'ة' | 'ى' | 'ء'
            // Persian / Urdu / Kurdish extensions
            | 'ژ' | 'ڈ' | 'ڑ' | 'ڕ'
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
        sorted.sort_by(|a, b| b.bbox.x.partial_cmp(&a.bbox.x).unwrap_or(std::cmp::Ordering::Equal));

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

pub struct ArabicParagraphReconstructor;

impl ArabicParagraphReconstructor {
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
}
