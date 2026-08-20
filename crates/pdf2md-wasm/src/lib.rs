use wasm_bindgen::prelude::*;
use pdf2md_core::{Config, Converter, MarkdownDialect, OcrMode, PageBreakStyle};
use pdf2md_markdown::MarkdownRenderer;
use pdf2md_pdf::PdfDocument;

#[wasm_bindgen]
pub struct PDFMarkdown {
    config: Config,
}

#[wasm_bindgen]
impl PDFMarkdown {
    #[wasm_bindgen(constructor)]
    pub fn new(dialect: Option<String>, detect_tables: Option<bool>) -> Self {
        let md_dialect = match dialect.as_deref() {
            Some("commonmark") => MarkdownDialect::CommonMark,
            Some("extended") => MarkdownDialect::Extended,
            _ => MarkdownDialect::GitHubFlavored,
        };

        let config = Config::builder()
            .dialect(md_dialect)
            .detect_tables(detect_tables.unwrap_or(true))
            .ocr_mode(OcrMode::Auto)
            .page_breaks(PageBreakStyle::HtmlComment)
            .build();

        Self { config }
    }

    /// Fast capability assessment to check if document requires server fallback or OCR
    #[wasm_bindgen]
    pub fn assess_capabilities(&self, pdf_bytes: &[u8]) -> Result<JsValue, JsValue> {
        let doc = PdfDocument::parse(pdf_bytes, self.config.security_limits.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let total_pages = doc.pages.len();
        let scanned_pages = doc.pages.iter().filter(|p| p.is_scanned).count();
        let digital_pages = doc.pages.iter().filter(|p| p.has_usable_text).count();
        let total_images: usize = doc.pages.iter().map(|p| p.images.len()).sum();

        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("pageCount"), &JsValue::from_f64(total_pages as f64))?;
        js_sys::Reflect::set(&obj, &JsValue::from_str("scannedPages"), &JsValue::from_f64(scanned_pages as f64))?;
        js_sys::Reflect::set(&obj, &JsValue::from_str("digitalPages"), &JsValue::from_f64(digital_pages as f64))?;
        js_sys::Reflect::set(&obj, &JsValue::from_str("totalImages"), &JsValue::from_f64(total_images as f64))?;
        js_sys::Reflect::set(&obj, &JsValue::from_str("requiresOcr"), &JsValue::from_bool(scanned_pages > 0))?;
        js_sys::Reflect::set(&obj, &JsValue::from_str("isEncrypted"), &JsValue::from_bool(doc.metadata.is_encrypted))?;

        Ok(obj.into())
    }

    /// Converts entire PDF bytes in a single pass
    #[wasm_bindgen]
    pub fn convert(&self, pdf_bytes: &[u8]) -> Result<JsValue, JsValue> {
        let converter = Converter::new(self.config.clone());
        let result = converter
            .convert_bytes(pdf_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let json_diag = serde_json::to_string(&result.diagnostics)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let json_meta = serde_json::to_string(&result.document.metadata)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let out = js_sys::Object::new();
        js_sys::Reflect::set(&out, &JsValue::from_str("markdown"), &JsValue::from_str(&result.markdown))?;
        js_sys::Reflect::set(&out, &JsValue::from_str("metadata"), &JsValue::from_str(&json_meta))?;
        js_sys::Reflect::set(&out, &JsValue::from_str("diagnostics"), &JsValue::from_str(&json_diag))?;
        js_sys::Reflect::set(&out, &JsValue::from_str("overallConfidence"), &JsValue::from_f64(result.diagnostics.overall_confidence as f64))?;
        js_sys::Reflect::set(&out, &JsValue::from_str("totalPages"), &JsValue::from_f64(result.diagnostics.total_pages as f64))?;

        Ok(out.into())
    }

    /// Converts a single page for progressive streaming
    #[wasm_bindgen]
    pub fn convert_page(&self, pdf_bytes: &[u8], page_num: usize) -> Result<JsValue, JsValue> {
        let doc = PdfDocument::parse(pdf_bytes, self.config.security_limits.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let raw_page = doc.pages.iter().find(|p| p.page_number == page_num)
            .ok_or_else(|| JsValue::from_str(&format!("Page {} not found", page_num)))?;

        let layout_engine = pdf2md_layout::LayoutEngine::default();
        let section = layout_engine.analyze_page(raw_page);

        let mut single_doc = pdf2md_ast::Document::new(doc.metadata);
        single_doc.sections.push(section);

        let renderer = MarkdownRenderer::new(self.config.to_render_options());
        let page_markdown = renderer.render(&single_doc);

        let out = js_sys::Object::new();
        js_sys::Reflect::set(&out, &JsValue::from_str("pageNumber"), &JsValue::from_f64(page_num as f64))?;
        js_sys::Reflect::set(&out, &JsValue::from_str("markdown"), &JsValue::from_str(&page_markdown))?;
        js_sys::Reflect::set(&out, &JsValue::from_str("isScanned"), &JsValue::from_bool(raw_page.is_scanned))?;

        Ok(out.into())
    }
}
