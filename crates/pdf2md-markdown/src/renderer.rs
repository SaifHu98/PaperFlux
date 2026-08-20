use crate::options::{PageBreakStyle, RenderOptions};
use crate::table_formatter::format_table;
use pdf2md_ast::{Document, DocumentMetadata, InlineNode, ListItem, Node, Section};

pub struct MarkdownRenderer {
    pub options: RenderOptions,
}

impl MarkdownRenderer {
    pub fn new(options: RenderOptions) -> Self {
        Self { options }
    }

    pub fn render(&self, document: &Document) -> String {
        let mut out = String::new();

        // 1. Optional YAML frontmatter
        if self.options.emit_frontmatter && has_meaningful_metadata(&document.metadata) {
            out.push_str(&render_frontmatter(&document.metadata));
            out.push('\n');
        }

        // 2. Render sections (pages)
        for (i, section) in document.sections.iter().enumerate() {
            if i > 0 {
                self.render_page_break(section.page_number, &mut out);
            }
            self.render_section(section, &mut out);
        }

        out
    }

    fn render_page_break(&self, page_num: usize, out: &mut String) {
        match self.options.page_breaks {
            PageBreakStyle::None => {}
            PageBreakStyle::ThematicBreak => {
                out.push_str("\n---\n\n");
            }
            PageBreakStyle::HtmlComment => {
                out.push_str(&format!("\n<!-- pagebreak: page {} -->\n\n", page_num));
            }
            PageBreakStyle::CustomMarker => {
                out.push_str(&format!("\n[[Page {}]]\n\n", page_num));
            }
        }
    }

    fn render_section(&self, section: &Section, out: &mut String) {
        for element in &section.elements {
            self.render_node(element, out);
        }
    }

    fn render_node(&self, node: &Node, out: &mut String) {
        match node {
            Node::Heading { level, text, .. } => {
                let prefix = "#".repeat((*level as usize).clamp(1, 6));
                out.push_str(&format!("{} ", prefix));
                for inline in text {
                    self.render_inline(inline, out);
                }
                out.push_str("\n\n");
            }
            Node::Paragraph { inlines, .. } => {
                for inline in inlines {
                    self.render_inline(inline, out);
                }
                out.push_str("\n\n");
            }
            Node::CodeBlock { language, code, .. } => {
                let lang = language.as_deref().unwrap_or("");
                out.push_str(&format!("```{}\n{}\n```\n\n", lang, code.trim_end()));
            }
            Node::BlockQuote { children, .. } => {
                let mut inner = String::new();
                for child in children {
                    self.render_node(child, &mut inner);
                }
                for line in inner.lines() {
                    out.push_str("> ");
                    out.push_str(line);
                    out.push('\n');
                }
                out.push('\n');
            }
            Node::List {
                ordered,
                start,
                items,
                ..
            } => {
                self.render_list(*ordered, *start, items, 0, out);
                out.push('\n');
            }
            Node::Table {
                headers,
                rows,
                caption,
                ..
            } => {
                let table_md = format_table(headers, rows, caption.as_deref(), &self.options);
                out.push_str(&table_md);
                out.push('\n');
            }
            Node::Image {
                alt_text,
                src,
                title,
                ..
            } => {
                if let Some(t) = title {
                    out.push_str(&format!("![{}]({} \"{}\")\n\n", alt_text, src, t));
                } else {
                    out.push_str(&format!("![{}]({})\n\n", alt_text, src));
                }
            }
            Node::Footnote { id, content } => {
                out.push_str(&format!("[^{}]: ", id));
                for inline in content {
                    self.render_inline(inline, out);
                }
                out.push_str("\n\n");
            }
            Node::Caption {
                target_type: _,
                text,
                ..
            } => {
                out.push('*');
                for inline in text {
                    self.render_inline(inline, out);
                }
                out.push_str("*\n\n");
            }
            Node::Formula { latex, inline, .. } => {
                if *inline {
                    out.push_str(&format!("${}$\n\n", latex));
                } else {
                    out.push_str(&format!("$$\n{}\n$$\n\n", latex.trim()));
                }
            }
            Node::PageBreak { page_number } => {
                self.render_page_break(*page_number, out);
            }
            Node::ThematicBreak => {
                out.push_str("---\n\n");
            }
            Node::Unknown { raw_hint, .. } => {
                if !raw_hint.is_empty() {
                    out.push_str(raw_hint);
                    out.push_str("\n\n");
                }
            }
        }
    }

    fn render_list(
        &self,
        ordered: bool,
        start: Option<u64>,
        items: &[ListItem],
        indent: usize,
        out: &mut String,
    ) {
        let mut curr_num = start.unwrap_or(1);
        let indent_str = "  ".repeat(indent);

        for item in items {
            let marker = if ordered {
                let m = format!("{}{}. ", indent_str, curr_num);
                curr_num += 1;
                m
            } else {
                format!("{}- ", indent_str)
            };

            out.push_str(&marker);
            for inline in &item.inlines {
                self.render_inline(inline, out);
            }
            out.push('\n');

            for child in &item.children {
                self.render_node(child, out);
            }
        }
    }

    fn render_inline(&self, inline: &InlineNode, out: &mut String) {
        match inline {
            InlineNode::Text(t) => out.push_str(t),
            InlineNode::Emphasis(children) => {
                out.push('*');
                for c in children {
                    self.render_inline(c, out);
                }
                out.push('*');
            }
            InlineNode::Strong(children) => {
                out.push_str("**");
                for c in children {
                    self.render_inline(c, out);
                }
                out.push_str("**");
            }
            InlineNode::Strikethrough(children) => {
                out.push_str("~~");
                for c in children {
                    self.render_inline(c, out);
                }
                out.push_str("~~");
            }
            InlineNode::InlineCode(code) => {
                out.push('`');
                out.push_str(code);
                out.push('`');
            }
            InlineNode::Link { url, text, title } => {
                out.push('[');
                for c in text {
                    self.render_inline(c, out);
                }
                out.push(']');
                if let Some(t) = title {
                    out.push_str(&format!("({} \"{}\")", url, t));
                } else {
                    out.push_str(&format!("({})", url));
                }
            }
            InlineNode::FootnoteRef(id) => {
                out.push_str(&format!("[^{}]", id));
            }
            InlineNode::InlineFormula(latex) => {
                out.push('$');
                out.push_str(latex);
                out.push('$');
            }
        }
    }
}

fn has_meaningful_metadata(meta: &DocumentMetadata) -> bool {
    meta.title.is_some()
        || meta.author.is_some()
        || meta.subject.is_some()
        || !meta.keywords.is_empty()
}

fn render_frontmatter(meta: &DocumentMetadata) -> String {
    let mut s = String::from("---\n");
    if let Some(title) = &meta.title {
        s.push_str(&format!("title: \"{}\"\n", escape_yaml(title)));
    }
    if let Some(author) = &meta.author {
        s.push_str(&format!("author: \"{}\"\n", escape_yaml(author)));
    }
    if let Some(subject) = &meta.subject {
        s.push_str(&format!("subject: \"{}\"\n", escape_yaml(subject)));
    }
    if !meta.keywords.is_empty() {
        s.push_str("keywords: [");
        for (i, kw) in meta.keywords.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&format!("\"{}\"", escape_yaml(kw)));
        }
        s.push_str("]\n");
    }
    if meta.total_pages > 0 {
        s.push_str(&format!("pages: {}\n", meta.total_pages));
    }
    s.push_str("---\n");
    s
}

fn escape_yaml(val: &str) -> String {
    val.replace('\\', "\\\\").replace('"', "\\\"")
}
