use pdf2md_ast::geometry::{Baseline, BoundingBox, Color, Matrix, Point, Rect, WritingDirection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSpan {
    pub text: String,
    pub bbox: BoundingBox,
    pub baseline: Baseline,
    pub font_name: String,
    pub font_size: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_monospace: bool,
    pub color: Color,
    pub matrix: Matrix,
    pub char_spacing: f32,
    pub word_spacing: f32,
    pub leading: f32,
    pub direction: WritingDirection,
}

impl TextSpan {
    pub fn new(
        text: String,
        bbox: BoundingBox,
        font_name: String,
        font_size: f32,
        is_bold: bool,
        is_italic: bool,
        is_monospace: bool,
    ) -> Self {
        let baseline = Baseline::new(bbox.y_max(), bbox.x_min(), bbox.x_max());
        Self {
            text,
            bbox,
            baseline,
            font_name,
            font_size,
            is_bold,
            is_italic,
            is_monospace,
            color: Color::BLACK,
            matrix: Matrix::IDENTITY,
            char_spacing: 0.0,
            word_spacing: 0.0,
            leading: 0.0,
            direction: WritingDirection::LeftToRight,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub ch: char,
    pub bbox: BoundingBox,
    pub font_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSegment {
    pub rect: Option<Rect>,
    pub points: Vec<Point>,
    pub is_stroke: bool,
    pub is_fill: bool,
    pub stroke_width: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    pub bbox: BoundingBox,
    pub width: usize,
    pub height: usize,
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPage {
    pub page_number: usize,
    pub width: f32,
    pub height: f32,
    pub rotation: i32,
    pub text_spans: Vec<TextSpan>,
    pub paths: Vec<PathSegment>,
    pub images: Vec<ImageObject>,
    pub has_usable_text: bool,
    pub is_scanned: bool,
}

impl RawPage {
    pub fn new(page_number: usize, width: f32, height: f32) -> Self {
        Self {
            page_number,
            width,
            height,
            rotation: 0,
            text_spans: Vec::new(),
            paths: Vec::new(),
            images: Vec::new(),
            has_usable_text: false,
            is_scanned: false,
        }
    }

    pub fn assess_capabilities(&mut self) {
        let total_chars: usize = self.text_spans.iter().map(|s| s.text.len()).sum();
        self.has_usable_text = total_chars > 20;
        self.is_scanned = !self.has_usable_text && !self.images.is_empty();
    }
}

#[derive(Debug, Clone)]
pub struct GraphicsState {
    pub ctm: Matrix,
    pub text_matrix: Matrix,
    pub text_line_matrix: Matrix,
    pub font_name: String,
    pub font_size: f32,
    pub char_spacing: f32,
    pub word_spacing: f32,
    pub leading: f32,
    pub text_rise: f32,
    pub stroke_color: Color,
    pub fill_color: Color,
    pub line_width: f32,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix::IDENTITY,
            text_matrix: Matrix::IDENTITY,
            text_line_matrix: Matrix::IDENTITY,
            font_name: "Helvetica".to_string(),
            font_size: 12.0,
            char_spacing: 0.0,
            word_spacing: 0.0,
            leading: 0.0,
            text_rise: 0.0,
            stroke_color: Color::BLACK,
            fill_color: Color::BLACK,
            line_width: 1.0,
        }
    }
}
