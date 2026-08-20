use pdf2md_pdf::elements::RawPage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OcrMode {
    #[default]
    Auto,
    Always,
    Never,
}

pub struct OcrDecisionEngine {
    pub mode: OcrMode,
    pub min_text_chars_threshold: usize,
}

impl Default for OcrDecisionEngine {
    fn default() -> Self {
        Self {
            mode: OcrMode::Auto,
            min_text_chars_threshold: 20,
        }
    }
}

impl OcrDecisionEngine {
    pub fn should_perform_ocr(&self, page: &RawPage) -> bool {
        match self.mode {
            OcrMode::Never => false,
            OcrMode::Always => true,
            OcrMode::Auto => {
                let total_chars: usize = page.text_spans.iter().map(|s| s.text.len()).sum();
                // If page has virtually no usable digital text but has images or is scanned
                total_chars < self.min_text_chars_threshold
                    && (page.is_scanned || !page.images.is_empty())
            }
        }
    }
}
