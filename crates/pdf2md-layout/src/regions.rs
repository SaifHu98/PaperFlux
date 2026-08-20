use serde::{Deserialize, Serialize};
use pdf2md_ast::geometry::BoundingBox;
use pdf2md_pdf::elements::TextSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionKind {
    Header,
    Footer,
    Body,
    Column(usize), // Column index (0, 1, 2...)
    Sidebar,
    FloatingFigure,
    Footnote,
    Table,
}

#[derive(Debug, Clone)]
pub struct PageRegion {
    pub id: usize,
    pub kind: RegionKind,
    pub bbox: BoundingBox,
    pub spans: Vec<TextSpan>,
    pub confidence: f32,
}

impl PageRegion {
    pub fn new(id: usize, kind: RegionKind, bbox: BoundingBox, spans: Vec<TextSpan>, confidence: f32) -> Self {
        Self {
            id,
            kind,
            bbox,
            spans,
            confidence,
        }
    }
}

pub struct RegionClassifier {
    pub page_width: f32,
    pub page_height: f32,
    pub header_margin_pt: f32,
    pub footer_margin_pt: f32,
}

impl RegionClassifier {
    pub fn new(page_width: f32, page_height: f32) -> Self {
        Self {
            page_width,
            page_height,
            header_margin_pt: 45.0,
            footer_margin_pt: 45.0,
        }
    }

    pub fn classify_span_region(&self, span: &TextSpan) -> RegionKind {
        if span.bbox.y_min() <= self.header_margin_pt {
            return RegionKind::Header;
        }
        if span.bbox.y_max() >= (self.page_height - self.footer_margin_pt) {
            return RegionKind::Footer;
        }

        // Check for sidebar: located in the left or right 25% of page with small width
        let is_left_margin = span.bbox.x_max() <= self.page_width * 0.28;
        let is_right_margin = span.bbox.x_min() >= self.page_width * 0.72;
        if (is_left_margin || is_right_margin) && span.bbox.width < self.page_width * 0.25 {
            return RegionKind::Sidebar;
        }

        RegionKind::Body
    }
}
