use pdf2md_pdf::elements::TextSpan;

pub struct HeaderFooterFilter {
    pub header_margin_pt: f32,
    pub footer_margin_pt: f32,
}

impl Default for HeaderFooterFilter {
    fn default() -> Self {
        Self {
            header_margin_pt: 45.0,
            footer_margin_pt: 45.0,
        }
    }
}

impl HeaderFooterFilter {
    pub fn is_header_or_footer(&self, span: &TextSpan, page_height: f32) -> bool {
        let is_top = span.bbox.y_min() <= self.header_margin_pt;
        let is_bottom = span.bbox.y_max() >= (page_height - self.footer_margin_pt);

        if is_top || is_bottom {
            let text = span.text.trim();
            // Check if it's a page number or trivial header/footer
            if is_page_number_pattern(text) {
                return true;
            }
            if text.len() < 40 && (is_top || is_bottom) {
                return true;
            }
        }

        false
    }
}

fn is_page_number_pattern(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.starts_with("page ") || lower.starts_with("p. ") || lower.starts_with("- ") {
        return true;
    }
    // Standalone number or "1 of 10" / "1 / 10"
    if text.parse::<u32>().is_ok() {
        return true;
    }
    if text.contains(" of ") || text.contains('/') {
        let parts: Vec<&str> = text.split([' ', '/']).filter(|s| !s.is_empty()).collect();
        if parts.len() <= 3 && parts.iter().all(|p| p.parse::<u32>().is_ok() || *p == "of") {
            return true;
        }
    }
    false
}
