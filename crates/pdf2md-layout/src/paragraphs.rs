use pdf2md_ast::{InlineNode, Node};
use pdf2md_pdf::elements::TextSpan;
use pdf2md_text::cjk::join_lines_cjk_aware;
use pdf2md_text::hyphenation::merge_hyphenated_lines;
use pdf2md_text::normalizer::TextNormalizer;
use crate::captions::CaptionDetector;
use crate::footnotes::FootnoteDetector;
use crate::headings::HeadingClassifier;
use crate::lists::{create_list_node, detect_list_item};

#[derive(Debug, Clone)]
pub struct TextLine {
    pub spans: Vec<TextSpan>,
    pub y: f32,
    pub height: f32,
    pub x_start: f32,
    pub x_end: f32,
}

pub struct ParagraphReconstructor {
    pub line_spacing_threshold: f32,
    pub column_left_margin: f32,
}

impl Default for ParagraphReconstructor {
    fn default() -> Self {
        Self {
            line_spacing_threshold: 1.4,
            column_left_margin: 72.0,
        }
    }
}

impl ParagraphReconstructor {
    pub fn reconstruct_nodes(
        &self,
        spans: &[TextSpan],
        heading_classifier: &HeadingClassifier,
        footnote_detector: Option<&FootnoteDetector>,
    ) -> Vec<Node> {
        if spans.is_empty() {
            return Vec::new();
        }

        // 1. Check if the whole block is a caption ("Figure 1: ...")
        if let Some(caption_node) = CaptionDetector::detect_caption(spans) {
            return vec![caption_node];
        }

        // 2. Check if the whole block is a footnote
        if let Some(fn_detector) = footnote_detector {
            if let Some(fn_node) = fn_detector.detect_footnote(spans) {
                return vec![fn_node];
            }
        }

        let lines = group_spans_into_lines(spans);
        let mut nodes = Vec::new();
        let mut current_para_lines: Vec<TextLine> = Vec::new();
        let mut current_list_items: Vec<(bool, Option<String>, String, usize)> = Vec::new();

        for line in lines {
            let line_text = line_to_plain_text(&line);

            // Check if this line is a list item
            if let Some((ordered, bullet, content, level)) = detect_list_item(&line_text, line.x_start) {
                if !current_para_lines.is_empty() {
                    nodes.push(self.create_paragraph_node(&current_para_lines));
                    current_para_lines.clear();
                }
                current_list_items.push((ordered, bullet, content, level));
                continue;
            } else if !current_list_items.is_empty() {
                nodes.push(create_list_node(current_list_items.clone()));
                current_list_items.clear();
            }

            // Check if this line is a heading
            if let Some((level, confidence)) = heading_classifier.classify_heading(&line.spans) {
                if !current_para_lines.is_empty() {
                    nodes.push(self.create_paragraph_node(&current_para_lines));
                    current_para_lines.clear();
                }
                let heading_inlines = build_inlines_from_spans(&line.spans);
                nodes.push(Node::Heading {
                    level,
                    text: heading_inlines,
                    confidence,
                    id: None,
                    bbox: None,
                });
                continue;
            }

            // Check paragraph break via line spacing
            if let Some(prev_line) = current_para_lines.last() {
                let vertical_gap = (line.y - prev_line.y).abs();
                let avg_height = (line.height + prev_line.height) / 2.0;

                if vertical_gap > avg_height * self.line_spacing_threshold {
                    nodes.push(self.create_paragraph_node(&current_para_lines));
                    current_para_lines.clear();
                }
            }

            current_para_lines.push(line);
        }

        if !current_list_items.is_empty() {
            nodes.push(create_list_node(current_list_items));
        }

        if !current_para_lines.is_empty() {
            nodes.push(self.create_paragraph_node(&current_para_lines));
        }

        nodes
    }

    fn create_paragraph_node(&self, lines: &[TextLine]) -> Node {
        let mut full_text = String::new();
        for (i, line) in lines.iter().enumerate() {
            let line_str = line_to_plain_text(line);
            if i == 0 {
                full_text.push_str(&line_str);
            } else {
                let (merged, _) = merge_hyphenated_lines(&full_text, &line_str);
                full_text = join_lines_cjk_aware(&full_text, &line_str);
                if merged.len() < full_text.len() {
                    full_text = merged;
                }
            }
        }

        let normalized = TextNormalizer::normalize(&full_text);

        // Check for CodeBlock: all spans are monospace
        let is_monospace = lines.iter().all(|l| l.spans.iter().all(|s| s.is_monospace));
        if is_monospace && (lines.len() >= 2 || normalized.contains('{') || normalized.contains("fn ") || normalized.contains("def ")) {
            return Node::CodeBlock {
                language: None,
                code: normalized,
                bbox: None,
            };
        }

        // Check for BlockQuote: lines are significantly indented from left margin (> 24pt) or start with quote marks
        let is_indented_quote = lines.iter().all(|l| l.x_start > self.column_left_margin + 20.0);
        let is_italic_quote = lines.iter().all(|l| l.spans.iter().all(|s| s.is_italic));
        if (is_indented_quote || is_italic_quote) && normalized.len() > 20 {
            let inner_para = Node::Paragraph {
                inlines: vec![InlineNode::Text(normalized)],
                confidence: 0.90,
                bbox: None,
            };
            return Node::BlockQuote {
                children: vec![inner_para],
                bbox: None,
            };
        }

        let inlines = vec![InlineNode::Text(normalized)];
        Node::Paragraph {
            inlines,
            confidence: 0.92,
            bbox: None,
        }
    }
}

pub fn group_spans_into_lines(spans: &[TextSpan]) -> Vec<TextLine> {
    if spans.is_empty() {
        return Vec::new();
    }

    let mut sorted_spans = spans.to_vec();
    // Sort by Y first, then by X
    sorted_spans.sort_by(|a, b| {
        if (a.bbox.y - b.bbox.y).abs() < 3.0 {
            a.bbox.x.partial_cmp(&b.bbox.x).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            a.bbox.y.partial_cmp(&b.bbox.y).unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    let mut lines: Vec<TextLine> = Vec::new();

    for span in sorted_spans {
        if let Some(last_line) = lines.last_mut() {
            if (span.bbox.y - last_line.y).abs() <= 4.0 {
                last_line.x_end = last_line.x_end.max(span.bbox.x_max());
                last_line.height = last_line.height.max(span.bbox.height);
                last_line.spans.push(span);
                continue;
            }
        }

        lines.push(TextLine {
            x_start: span.bbox.x_min(),
            x_end: span.bbox.x_max(),
            y: span.bbox.y,
            height: span.bbox.height,
            spans: vec![span],
        });
    }

    lines
}

fn line_to_plain_text(line: &TextLine) -> String {
    let mut s = String::new();
    for (i, span) in line.spans.iter().enumerate() {
        if i > 0 && !s.ends_with(' ') && !span.text.starts_with(' ') {
            s.push(' ');
        }
        s.push_str(&span.text);
    }
    s
}

fn build_inlines_from_spans(spans: &[TextSpan]) -> Vec<InlineNode> {
    let mut inlines = Vec::new();
    for span in spans {
        let text = TextNormalizer::normalize(&span.text);
        if text.is_empty() {
            continue;
        }

        let mut node = InlineNode::Text(text);
        if span.is_bold && span.is_italic {
            node = InlineNode::Strong(vec![InlineNode::Emphasis(vec![node])]);
        } else if span.is_bold {
            node = InlineNode::Strong(vec![node]);
        } else if span.is_italic {
            node = InlineNode::Emphasis(vec![node]);
        } else if span.is_monospace {
            if let InlineNode::Text(t) = node {
                node = InlineNode::InlineCode(t);
            }
        }
        inlines.push(node);
    }
    inlines
}
