use pdf2md_ast::geometry::BoundingBox;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("OCR engine unavailable: {0}")]
    Unavailable(String),

    #[error("OCR recognition failure: {0}")]
    RecognitionFailed(String),

    #[error("Invalid image for OCR: {0}")]
    InvalidImage(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OcrOrientation {
    #[default]
    Rot0,
    Rot90,
    Rot180,
    Rot270,
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

impl MockOCRProvider {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            dummy_text: text.into(),
            confidence: 0.95,
        }
    }

    pub fn with_confidence(text: impl Into<String>, confidence: f32) -> Self {
        Self {
            dummy_text: text.into(),
            confidence,
        }
    }
}

impl Default for MockOCRProvider {
    fn default() -> Self {
        Self::new("Mock OCR extracted text")
    }
}

impl OCRProvider for MockOCRProvider {
    fn name(&self) -> &str {
        "MockOCRProvider"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn recognize(
        &self,
        _image_bytes: &[u8],
        _lang_hint: Option<&str>,
    ) -> Result<OcrResult, String> {
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

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Production OCR provider utilizing the system Tesseract OCR binary
#[derive(Debug, Clone)]
pub struct SystemTesseractOCRProvider {
    pub binary_path: Option<PathBuf>,
    pub languages: String,
}

impl Default for SystemTesseractOCRProvider {
    fn default() -> Self {
        Self {
            binary_path: None,
            languages: "ara+eng".to_string(),
        }
    }
}

impl SystemTesseractOCRProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_languages(languages: &str) -> Self {
        Self {
            binary_path: None,
            languages: languages.to_string(),
        }
    }

    pub fn with_binary(binary: PathBuf, languages: &str) -> Self {
        Self {
            binary_path: Some(binary),
            languages: languages.to_string(),
        }
    }

    pub fn resolve_binary(&self) -> Option<PathBuf> {
        if let Some(b) = &self.binary_path {
            if b.exists() {
                return Some(b.clone());
            }
        }

        if let Ok(env_path) = std::env::var("TESSERACT_PATH") {
            let p = PathBuf::from(env_path);
            if p.exists() {
                return Some(p);
            }
        }

        let candidates = [
            "tesseract",
            "tesseract.exe",
            r"C:\Program Files\Tesseract-OCR\tesseract.exe",
            r"C:\Program Files (x86)\Tesseract-OCR\tesseract.exe",
            "/usr/bin/tesseract",
            "/usr/local/bin/tesseract",
            "/opt/homebrew/bin/tesseract",
        ];

        for c in candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                return Some(p);
            }
            if let Ok(out) = Command::new(c).arg("--version").output() {
                if out.status.success() {
                    return Some(PathBuf::from(c));
                }
            }
        }

        None
    }
}

impl OCRProvider for SystemTesseractOCRProvider {
    fn name(&self) -> &str {
        "SystemTesseractOCRProvider"
    }

    fn is_available(&self) -> bool {
        self.resolve_binary().is_some()
    }

    fn recognize(&self, image_bytes: &[u8], lang_hint: Option<&str>) -> Result<OcrResult, String> {
        let binary = self.resolve_binary().ok_or_else(|| {
            "Tesseract binary not found on host system (set TESSERACT_PATH)".to_string()
        })?;

        let lang = lang_hint.unwrap_or(&self.languages);

        // Write image bytes to a temp file
        let temp_dir = std::env::temp_dir();
        let file_id = format!(
            "tess_ocr_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_img = temp_dir.join(format!("{}.jpg", file_id));

        fs::write(&temp_img, image_bytes)
            .map_err(|e| format!("Failed to write temporary OCR image: {}", e))?;

        let output = Command::new(binary)
            .arg(&temp_img)
            .arg("stdout")
            .arg("-l")
            .arg(lang)
            .output();

        let _ = fs::remove_file(&temp_img);

        match output {
            Ok(out) => {
                if !out.status.success() {
                    let err = String::from_utf8_lossy(&out.stderr);
                    return Err(format!("Tesseract OCR failed: {}", err));
                }
                let recognized_text = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let lines: Vec<OcrLine> = recognized_text
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|line| OcrLine {
                        text: line.to_string(),
                        bbox: BoundingBox::new(0.0, 0.0, 595.0, 20.0),
                        confidence: 0.90,
                        words: line
                            .split_whitespace()
                            .map(|w| OcrWord {
                                text: w.to_string(),
                                bbox: BoundingBox::new(0.0, 0.0, 50.0, 20.0),
                                confidence: 0.90,
                                detected_script: Some("Arabic".into()),
                            })
                            .collect(),
                    })
                    .collect();

                Ok(OcrResult {
                    text: recognized_text,
                    confidence: 0.90,
                    orientation: OcrOrientation::Rot0,
                    lines,
                    detected_language: Some(lang.to_string()),
                })
            }
            Err(e) => Err(format!("Failed to execute Tesseract: {}", e)),
        }
    }
}
