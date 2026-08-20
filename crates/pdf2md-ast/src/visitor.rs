use crate::ast::*;

pub trait AstVisitor {
    fn visit_document(&mut self, doc: &Document) {
        for section in &doc.sections {
            self.visit_section(section);
        }
    }

    fn visit_section(&mut self, section: &Section) {
        for element in &section.elements {
            self.visit_node(element);
        }
    }

    fn visit_node(&mut self, node: &Node) {
        match node {
            Node::Heading { text, .. } => {
                for inline in text {
                    self.visit_inline(inline);
                }
            }
            Node::Paragraph { inlines, .. } => {
                for inline in inlines {
                    self.visit_inline(inline);
                }
            }
            Node::CodeBlock { .. } => {}
            Node::BlockQuote { children, .. } => {
                for child in children {
                    self.visit_node(child);
                }
            }
            Node::List { items, .. } => {
                for item in items {
                    for inline in &item.inlines {
                        self.visit_inline(inline);
                    }
                    for child in &item.children {
                        self.visit_node(child);
                    }
                }
            }
            Node::Table { headers, rows, .. } => {
                for row in headers.iter().chain(rows.iter()) {
                    for cell in &row.cells {
                        for inline in &cell.content {
                            self.visit_inline(inline);
                        }
                    }
                }
            }
            Node::Image { .. } => {}
            Node::Footnote { content, .. } => {
                for inline in content {
                    self.visit_inline(inline);
                }
            }
            Node::Caption { text, .. } => {
                for inline in text {
                    self.visit_inline(inline);
                }
            }
            Node::Formula { .. } => {}
            Node::PageBreak { .. } => {}
            Node::ThematicBreak => {}
            Node::Unknown { .. } => {}
        }
    }

    fn visit_inline(&mut self, inline: &InlineNode) {
        match inline {
            InlineNode::Text(_) => {}
            InlineNode::Emphasis(children)
            | InlineNode::Strong(children)
            | InlineNode::Strikethrough(children) => {
                for c in children {
                    self.visit_inline(c);
                }
            }
            InlineNode::InlineCode(_) => {}
            InlineNode::Link { text, .. } => {
                for c in text {
                    self.visit_inline(c);
                }
            }
            InlineNode::FootnoteRef(_) => {}
            InlineNode::InlineFormula(_) => {}
        }
    }
}
