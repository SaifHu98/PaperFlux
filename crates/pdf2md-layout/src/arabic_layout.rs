use pdf2md_ast::{CaptionTarget, InlineNode, Node};
use pdf2md_pdf::elements::TextSpan;
use pdf2md_text::arabic::context::ArabicProcessingContext;
use pdf2md_text::arabic::pipeline::ArabicTextPipeline;

pub struct ArabicLayoutAnalyzer;

impl ArabicLayoutAnalyzer {
    /// Detects if text is an Arabic heading (e.g. "المبحث الأول", "المادة 1", "الفصل الثاني")
    pub fn classify_arabic_heading(text: &str) -> Option<(u8, f32)> {
        let trimmed = text.trim();

        if trimmed.starts_with("الباب ") || trimmed.starts_with("الكتاب ") {
            return Some((1, 0.96));
        }
        if trimmed.starts_with("الفصل ") || trimmed.starts_with("المحور ") {
            return Some((2, 0.95));
        }
        if trimmed.starts_with("المبحث ")
            || trimmed.starts_with("المادة ")
            || trimmed.starts_with("مادة ")
        {
            return Some((3, 0.94));
        }
        if trimmed.starts_with("المطلب ")
            || trimmed.starts_with("الفرع ")
            || trimmed.starts_with("أولاً:")
            || trimmed.starts_with("ثانياً:")
        {
            return Some((4, 0.92));
        }

        None
    }

    /// Detects Arabic list items (e.g. "أ-", "ب-", "١.", "٢.", "أولاً:", "•")
    pub fn detect_arabic_list_item(
        text: &str,
        indent_pt: f32,
    ) -> Option<(bool, Option<String>, String, usize)> {
        let trimmed = text.trim_start();
        if trimmed.is_empty() {
            return None;
        }

        let indent_level = ((indent_pt / 16.0).floor() as usize).min(6);

        // 1. Arabic Alphabet bullets: أ-, ب-, ج-, د-, (أ), (ب)
        let arabic_abjad = [
            'أ', 'ب', 'ج', 'د', 'ه', 'و', 'ز', 'ح', 'ط', 'ي', 'ك', 'ل', 'م', 'ن', 'س', 'ع', 'ف',
            'ص', 'ق', 'ر', 'ش', 'ت', 'ث', 'خ', 'ذ', 'ض', 'ظ', 'غ',
        ];

        if let Some(first_ch) = trimmed.chars().next() {
            if arabic_abjad.contains(&first_ch) {
                let rest = &trimmed[first_ch.len_utf8()..];
                if rest.starts_with('-') || rest.starts_with('.') || rest.starts_with(')') {
                    let after = rest[1..].trim_start();
                    return Some((
                        true,
                        Some(first_ch.to_string()),
                        after.to_string(),
                        indent_level,
                    ));
                }
            }
        }

        // 2. Eastern Arabic-Indic numerals: ١., ٢., ٣., (١), (٢)
        if let Some(pos) = trimmed.find(['.', ')', ']']) {
            let prefix = trimmed[..pos]
                .trim_start_matches('(')
                .trim_start_matches('[')
                .trim();
            if is_eastern_arabic_numeral(prefix) {
                let after = trimmed[pos + 1..].trim_start();
                return Some((
                    true,
                    Some(prefix.to_string()),
                    after.to_string(),
                    indent_level,
                ));
            }
        }

        None
    }

    /// Detects Arabic captions (e.g. "شكل 1:", "جدول 2:", "مخطط 3:")
    pub fn detect_arabic_caption(spans: &[TextSpan]) -> Option<Node> {
        if spans.is_empty() {
            return None;
        }

        let full_text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let trimmed = full_text.trim();

        let target_type = if trimmed.starts_with("شكل ")
            || trimmed.starts_with("الشكل ")
            || trimmed.starts_with("صورة ")
        {
            CaptionTarget::Figure
        } else if trimmed.starts_with("جدول ") || trimmed.starts_with("الجدول ") {
            CaptionTarget::Table
        } else if trimmed.starts_with("مخطط ")
            || trimmed.starts_with("كود ")
            || trimmed.starts_with("خوارزمية ")
        {
            CaptionTarget::Code
        } else if trimmed.starts_with("معادلة ") || trimmed.starts_with("المعادلة ") {
            CaptionTarget::Equation
        } else {
            return None;
        };

        let ctx = ArabicProcessingContext::new_arabic();
        let (processed, _) = ArabicTextPipeline::process(trimmed, &ctx);

        Some(Node::Caption {
            target_type,
            text: vec![InlineNode::Text(processed)],
            bbox: None,
        })
    }
}

fn is_eastern_arabic_numeral(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| matches!(c, '٠'..='٩'))
}
