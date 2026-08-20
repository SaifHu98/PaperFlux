use thiserror::Error;
use pdf2md_ast::geometry::BoundingBox;
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("OCR engine unavailable: {0}")]
    Unavailable(String),

    #[error("OCR recognition failure: {0}")]
    RecognitionFailed(String),

    #[error("Invalid image for OCR: {0}")]
    InvalidImage(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OcrOrientation {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}

impl Default for OcrOrientation {
    fn default() -> Self {
        Self::Rot0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrWord {
    pub text: String,
    pub bbox: BoundingBox,
    pub confidence: f32,
    pub detected_script: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrLine {
    pub text: String,
    pub bbox: BoundingBox,
    pub confidence: f32,
    pub words: Vec<OcrWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrRequest {
    pub image_bytes: Vec<u8>,
    pub language_hints: Vec<String>,
    pub detect_orientation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
    pub orientation: OcrOrientation,
    pub lines: Vec<OcrLine>,
    pub detected_language: Option<String>,
}

pub trait OCRProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn recognize(&self, image_bytes: &[u8], lang_hint: Option<&str>) -> Result<OcrResult, String>;
    fn recognize_advanced(&self, request: OcrRequest) -> Result<OcrResult, String> {
        let hint = request.language_hints.first().map(|s| s.as_str());
        self.recognize(&request.image_bytes, hint)
    }
}

pub struct MockOCRProvider {
    pub dummy_text: String,
    pub confidence: f32,
}

impl Default for MockOCRProvider {
    fn default() -> Self {
        Self {
            dummy_text: "Mock OCR extracted text".to_string(),
            confidence: 0.95,
        }
    }
}

impl OCRProvider for MockOCRProvider {
    fn name(&self) -> &str {
        "MockOCRProvider"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn recognize(&self, _image_bytes: &[u8], _lang_hint: Option<&str>) -> Result<OcrResult, String> {
        Ok(OcrResult {
            text: self.dummy_text.clone(),
            confidence: self.confidence,
            orientation: OcrOrientation::Rot0,
            lines: vec![OcrLine {
                text: self.dummy_text.clone(),
                bbox: BoundingBox::new(72.0, 72.0, 400.0, 20.0),
                confidence: self.confidence,
                words: vec![OcrWord {
                    text: self.dummy_text.clone(),
                    bbox: BoundingBox::new(72.0, 72.0, 400.0, 20.0),
                    confidence: self.confidence,
                    detected_script: Some("Latin".into()),
                }],
            }],
            detected_language: Some("English".into()),
        })
    }
}
