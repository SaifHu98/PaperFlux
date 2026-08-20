use pdf2md_ast::Section;
use pdf2md_pdf::elements::{RawPage, TextSpan};
use crate::footnotes::FootnoteDetector;
use crate::headers_footers::HeaderFooterFilter;
use crate::headings::HeadingClassifier;
use crate::paragraphs::ParagraphReconstructor;
use crate::xy_cut::StagedReadingOrderEngine;

pub struct LayoutEngine {
    pub reading_order_engine: StagedReadingOrderEngine,
    pub hf_filter: HeaderFooterFilter,
    pub para_reconstructor: ParagraphReconstructor,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            reading_order_engine: StagedReadingOrderEngine::default(),
            hf_filter: HeaderFooterFilter::default(),
            para_reconstructor: ParagraphReconstructor::default(),
        }
    }
}

impl LayoutEngine {
    pub fn analyze_page(&self, page: &RawPage) -> Section {
        let mut section = Section::new(page.page_number);

        if page.text_spans.is_empty() {
            return section;
        }

        // 1. Filter headers and footers
        let content_spans: Vec<TextSpan> = page
            .text_spans
            .iter()
            .filter(|s| !self.hf_filter.is_header_or_footer(s, page.height))
            .cloned()
            .collect();

        if content_spans.is_empty() {
            return section;
        }

        // 2. Statistical font-size hierarchy clustering
        let heading_classifier = HeadingClassifier::from_spans(&content_spans);

        // 3. Footnote detector for bottom margin
        let footnote_detector = FootnoteDetector::new(page.height, heading_classifier.base_body_font_size);

        // 4. Staged Reading Order decomposition (handling multi-column, sidebars, full-width headers)
        let layout_blocks = self.reading_order_engine.compute_reading_order(&content_spans, page.width);

        // 5. Reconstruct semantic AST elements for each block
        for block in layout_blocks {
            let block_nodes = self.para_reconstructor.reconstruct_nodes(
                &block.spans,
                &heading_classifier,
                Some(&footnote_detector),
            );
            section.elements.extend(block_nodes);
        }

        section
    }
}
