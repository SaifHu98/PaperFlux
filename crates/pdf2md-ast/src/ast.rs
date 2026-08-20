use serde::{Deserialize, Serialize};
use crate::geometry::BoundingBox;
use crate::diagnostics::ConversionDiagnostics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub metadata: DocumentMetadata,
    pub sections: Vec<Section>,
    pub diagnostics: ConversionDiagnostics,
}

impl Document {
    pub fn new(metadata: DocumentMetadata) -> Self {
        Self {
            metadata,
            sections: Vec::new(),
            diagnostics: ConversionDiagnostics::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sections.is_empty() || self.sections.iter().all(|s| s.elements.is_empty())
    }

    pub fn total_sections(&self) -> usize {
        self.sections.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Vec<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub mod_date: Option<String>,
    pub total_pages: usize,
    pub language: Option<String>,
    pub is_encrypted: bool,
    pub pdf_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub page_number: usize,
    pub elements: Vec<Node>,
    pub bbox: Option<BoundingBox>,
}

impl Section {
    pub fn new(page_number: usize) -> Self {
        Self {
            page_number,
            elements: Vec::new(),
            bbox: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Node {
    Heading {
        level: u8,
        text: Vec<InlineNode>,
        confidence: f32,
        id: Option<String>,
        bbox: Option<BoundingBox>,
    },
    Paragraph {
        inlines: Vec<InlineNode>,
        confidence: f32,
        bbox: Option<BoundingBox>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
        bbox: Option<BoundingBox>,
    },
    BlockQuote {
        children: Vec<Node>,
        bbox: Option<BoundingBox>,
    },
    List {
        ordered: bool,
        start: Option<u64>,
        items: Vec<ListItem>,
        bbox: Option<BoundingBox>,
    },
    Table {
        headers: Vec<TableRow>,
        rows: Vec<TableRow>,
        caption: Option<String>,
        confidence: f32,
        has_borders: bool,
        bbox: Option<BoundingBox>,
    },
    Image {
        alt_text: String,
        src: String,
        title: Option<String>,
        width: Option<f32>,
        height: Option<f32>,
        bbox: Option<BoundingBox>,
        mime_type: Option<String>,
    },
    Footnote {
        id: String,
        content: Vec<InlineNode>,
    },
    Caption {
        target_type: CaptionTarget,
        text: Vec<InlineNode>,
        bbox: Option<BoundingBox>,
    },
    Formula {
        latex: String,
        inline: bool,
        bbox: Option<BoundingBox>,
    },
    PageBreak {
        page_number: usize,
    },
    ThematicBreak,
    Unknown {
        raw_hint: String,
        bbox: Option<BoundingBox>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub inlines: Vec<InlineNode>,
    pub children: Vec<Node>,
    pub bullet: Option<String>,
    pub level: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
    pub is_header: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: Vec<InlineNode>,
    pub colspan: usize,
    pub rowspan: usize,
    pub align: CellAlignment,
    pub bbox: Option<BoundingBox>,
}

impl TableCell {
    pub fn new(content: Vec<InlineNode>) -> Self {
        Self {
            content,
            colspan: 1,
            rowspan: 1,
            align: CellAlignment::None,
            bbox: None,
        }
    }

    pub fn text_content(&self) -> String {
        let mut out = String::new();
        for inline in &self.content {
            append_inline_text(inline, &mut out);
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellAlignment {
    Left,
    Center,
    Right,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptionTarget {
    Figure,
    Table,
    Code,
    Equation,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum InlineNode {
    Text(String),
    Emphasis(Vec<InlineNode>),
    Strong(Vec<InlineNode>),
    Strikethrough(Vec<InlineNode>),
    InlineCode(String),
    Link {
        url: String,
        text: Vec<InlineNode>,
        title: Option<String>,
    },
    FootnoteRef(String),
    InlineFormula(String),
}

impl InlineNode {
    pub fn plain_text(&self) -> String {
        let mut s = String::new();
        append_inline_text(self, &mut s);
        s
    }
}

fn append_inline_text(inline: &InlineNode, out: &mut String) {
    match inline {
        InlineNode::Text(t) => out.push_str(t),
        InlineNode::Emphasis(children)
        | InlineNode::Strong(children)
        | InlineNode::Strikethrough(children) => {
            for c in children {
                append_inline_text(c, out);
            }
        }
        InlineNode::InlineCode(c) => out.push_str(c),
        InlineNode::Link { text, .. } => {
            for c in text {
                append_inline_text(c, out);
            }
        }
        InlineNode::FootnoteRef(id) => {
            out.push('[');
            out.push('^');
            out.push_str(id);
            out.push(']');
        }
        InlineNode::InlineFormula(f) => {
            out.push('$');
            out.push_str(f);
            out.push('$');
        }
    }
}
