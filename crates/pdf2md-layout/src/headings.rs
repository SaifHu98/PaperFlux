use std::collections::HashMap;
use pdf2md_pdf::elements::TextSpan;

pub struct HeadingClassifier {
    pub base_body_font_size: f32,
    pub font_size_levels: Vec<f32>,
}

impl HeadingClassifier {
    pub fn new(base_body_font_size: f32) -> Self {
        Self {
            base_body_font_size: base_body_font_size.max(8.0),
            font_size_levels: Vec::new(),
        }
    }

    pub fn from_spans(spans: &[TextSpan]) -> Self {
        let mut size_histogram: HashMap<u32, usize> = HashMap::new();
        for span in spans {
            let key = (span.font_size * 10.0).round() as u32;
            *size_histogram.entry(key).or_insert(0) += span.text.len();
        }

        // Dominant font size is the body text font size
        let dominant_size = size_histogram
            .iter()
            .max_by_key(|&(_, count)| *count)
            .map(|(&k, _)| (k as f32) / 10.0)
            .unwrap_or(11.0);

        // Collect all distinct font sizes larger than body font size
        let mut larger_sizes: Vec<f32> = size_histogram
            .keys()
            .map(|&k| (k as f32) / 10.0)
            .filter(|&s| s > dominant_size * 1.08)
            .collect();

        larger_sizes.sort_by(|a, b| b.partial_cmp(a).unwrap()); // Descending

        Self {
            base_body_font_size: dominant_size,
            font_size_levels: larger_sizes,
        }
    }

    /// Classifies a candidate line as a heading, returning `Some((level, confidence))`.
    pub fn classify_heading(&self, spans: &[TextSpan]) -> Option<(u8, f32)> {
        if spans.is_empty() {
            return None;
        }

        let full_text = spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
        let trimmed = full_text.trim();

        if trimmed.is_empty() || trimmed.len() > 160 {
            return None; // Headings are concise
        }

        let max_font_size = spans.iter().map(|s| s.font_size).fold(0.0f32, f32::max);
        let is_bold = spans.iter().any(|s| s.is_bold);
        let is_all_caps = trimmed.len() > 3 && trimmed.chars().all(|c| !c.is_alphabetic() || c.is_uppercase());
        let has_numbered_clause = is_clause_number(trimmed);

        // 1. Check relative to discovered font size levels
        for (idx, &level_size) in self.font_size_levels.iter().enumerate() {
            if (max_font_size - level_size).abs() <= 0.8 {
                let level = (idx as u8 + 1).min(6);
                let confidence = if is_bold { 0.96 } else { 0.88 };
                return Some((level, confidence));
            }
        }

        let ratio = max_font_size / self.base_body_font_size;

        if ratio >= 1.75 {
            Some((1, 0.95))
        } else if ratio >= 1.40 {
            Some((2, 0.90))
        } else if ratio >= 1.20 {
            Some((3, 0.85))
        } else if ratio >= 1.08 && (is_bold || has_numbered_clause) {
            Some((4, 0.82))
        } else if is_bold && (has_numbered_clause || is_all_caps) && trimmed.len() < 80 {
            Some((5, 0.78))
        } else if is_bold && trimmed.len() < 60 && !trimmed.ends_with('.') {
            Some((6, 0.70))
        } else {
            None
        }
    }
}

fn is_clause_number(text: &str) -> bool {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if let Some(first) = parts.first() {
        let clean = first.trim_end_matches('.');
        if clean.contains('.') {
            return clean.split('.').all(|seg| seg.parse::<u32>().is_ok());
        }
    }
    false
}
