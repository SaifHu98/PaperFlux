use pdf2md_ast::{CaptionTarget, InlineNode, Node};
use pdf2md_pdf::elements::TextSpan;
use pdf2md_text::normalizer::TextNormalizer;

pub struct CaptionDetector;

impl CaptionDetector {
    pub fn detect_caption(spans: &[TextSpan]) -> Option<Node> {
        if spans.is_empty() {
            return None;
        }

        let full_text = spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
        let trimmed = full_text.trim();
        let lower = trimmed.to_lowercase();

        let target_type = if lower.starts_with("figure ")
            || lower.starts_with("fig. ")
            || lower.starts_with("fig ")
            || lower.starts_with("illustration ")
        {
            CaptionTarget::Figure
        } else if lower.starts_with("table ") || lower.starts_with("tab. ") || lower.starts_with("tab ") {
            CaptionTarget::Table
        } else if lower.starts_with("listing ") || lower.starts_with("algorithm ") || lower.starts_with("code ") {
            CaptionTarget::Code
        } else if lower.starts_with("equation ") || lower.starts_with("eq. ") || lower.starts_with("eq ") {
            CaptionTarget::Equation
        } else {
            return None;
        };

        let normalized = TextNormalizer::normalize(trimmed);
        Some(Node::Caption {
            target_type,
            text: vec![InlineNode::Text(normalized)],
            bbox: None,
        })
    }
}
