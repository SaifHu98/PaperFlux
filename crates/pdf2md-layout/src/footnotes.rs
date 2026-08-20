use pdf2md_ast::{InlineNode, Node};
use pdf2md_pdf::elements::TextSpan;
use pdf2md_text::normalizer::TextNormalizer;

pub struct FootnoteDetector {
    pub page_height: f32,
    pub base_body_font_size: f32,
}

impl FootnoteDetector {
    pub fn new(page_height: f32, base_body_font_size: f32) -> Self {
        Self {
            page_height,
            base_body_font_size: base_body_font_size.max(8.0),
        }
    }

    pub fn detect_footnote(&self, spans: &[TextSpan]) -> Option<Node> {
        if spans.is_empty() {
            return None;
        }

        let full_text = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let trimmed = full_text.trim();

        // Must be in bottom 25% of the page
        let min_y = spans
            .iter()
            .map(|s| s.bbox.y_min())
            .fold(f32::MAX, f32::min);
        if min_y < self.page_height * 0.70 {
            return None;
        }

        // Must have font size smaller than body text
        let avg_font_size = spans.iter().map(|s| s.font_size).sum::<f32>() / (spans.len() as f32);
        if avg_font_size > self.base_body_font_size * 0.90 {
            return None;
        }

        // Check for footnote ID pattern (e.g. "1.", "[1]", "*", "1 ")
        let (id, content) = extract_footnote_prefix(trimmed)?;
        let normalized = TextNormalizer::normalize(&content);

        Some(Node::Footnote {
            id,
            content: vec![InlineNode::Text(normalized)],
        })
    }
}

fn extract_footnote_prefix(text: &str) -> Option<(String, String)> {
    if text.starts_with('[') {
        if let Some(end) = text.find(']') {
            let id = &text[1..end];
            let after = text[end + 1..].trim();
            return Some((id.to_string(), after.to_string()));
        }
    }

    let first_token = text.split_whitespace().next()?;
    let clean_token = first_token.trim_end_matches('.');
    if clean_token.parse::<u32>().is_ok() && clean_token.len() <= 3 {
        let after = text[first_token.len()..].trim();
        return Some((clean_token.to_string(), after.to_string()));
    }

    if text.starts_with('*') || text.starts_with('†') || text.starts_with('‡') {
        let first_ch = text.chars().next().unwrap();
        let after = text[first_ch.len_utf8()..].trim();
        return Some((first_ch.to_string(), after.to_string()));
    }

    None
}
