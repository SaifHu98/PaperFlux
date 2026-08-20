use crate::provider::{OCRProvider, OcrError, OcrResult, OcrWord};
use pdf2md_ast::geometry::BoundingBox;

pub struct MockOcrProvider {
    pub mock_text: String,
    pub confidence_score: f32,
}

impl Default for MockOcrProvider {
    fn default() -> Self {
        Self {
            mock_text: "Recognized OCR text content".to_string(),
            confidence_score: 0.92,
        }
    }
}

impl MockOcrProvider {
    pub fn new(mock_text: &str, confidence_score: f32) -> Self {
        Self {
            mock_text: mock_text.to_string(),
            confidence_score,
        }
    }
}

impl OCRProvider for MockOcrProvider {
    fn detect_language(&self, _image_data: &[u8]) -> Result<String, OcrError> {
        Ok("en".to_string())
    }

    fn recognize(&self, _image_data: &[u8], _lang: Option<&str>) -> Result<OcrResult, OcrError> {
        let words = self
            .mock_text
            .split_whitespace()
            .enumerate()
            .map(|(i, w)| OcrWord {
                text: w.to_string(),
                bbox: BoundingBox::new((i as f32) * 50.0, 100.0, 45.0, 12.0),
                confidence: self.confidence_score,
            })
            .collect();

        Ok(OcrResult {
            text: self.mock_text.clone(),
            words,
            confidence: self.confidence_score,
            detected_language: Some("en".to_string()),
        })
    }

    fn detect_orientation(&self, _image_data: &[u8]) -> Result<i32, OcrError> {
        Ok(0)
    }

    fn confidence(&self) -> f32 {
        self.confidence_score
    }

    fn available_languages(&self) -> Vec<String> {
        vec!["en".to_string(), "ar".to_string(), "de".to_string(), "fr".to_string(), "es".to_string()]
    }
}
