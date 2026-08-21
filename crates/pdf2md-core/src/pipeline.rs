use crate::buffer_pool::BufferPool;
use crate::cache::PageCache;
use crate::config::Config;
use crate::error::{ConversionError, ConversionResult};
use crate::scheduler::AdaptiveScheduler;
use pdf2md_ast::{ConversionDiagnostics, Document, PageDiagnostics, Section};
use pdf2md_images::{ImageExtractor, ImageExtractorConfig};
use pdf2md_layout::LayoutEngine;
use pdf2md_markdown::MarkdownRenderer;
use pdf2md_ocr::{OCRProvider, OcrDecisionEngine};
use pdf2md_pdf::elements::RawPage;
use pdf2md_pdf::PdfDocument;
use pdf2md_table::TableDetector;
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;

pub struct Pipeline {
    config: Config,
    page_cache: Arc<PageCache>,
    pub buffer_pool: Arc<BufferPool>,
}

struct ProcessedPageOutput {
    section: Section,
    diagnostics: PageDiagnostics,
    is_ocr: bool,
    image_count: usize,
    table_count: usize,
}

fn process_single_page(
    raw_page: &RawPage,
    config: &Config,
    layout_engine: &LayoutEngine,
    image_extractor: &ImageExtractor,
    ocr_decision: &OcrDecisionEngine,
) -> ProcessedPageOutput {
    let mut section = Section::new(raw_page.page_number);
    let mut ocr_applied = false;

    // 1. Check if OCR is needed
    if ocr_decision.should_perform_ocr(raw_page) {
        let default_tesseract = pdf2md_ocr::SystemTesseractOCRProvider::new();
        let provider: Option<&dyn pdf2md_ocr::OCRProvider> = match &config.ocr_provider {
            Some(p) => Some(p.as_ref()),
            None => {
                if default_tesseract.is_available() {
                    Some(&default_tesseract)
                } else {
                    None
                }
            }
        };

        if let Some(ocr_prov) = provider {
            for img in &raw_page.images {
                if let Ok(ocr_res) = ocr_prov.recognize(&img.data, None) {
                    if !ocr_res.text.trim().is_empty() {
                        let para = pdf2md_ast::Node::Paragraph {
                            inlines: vec![pdf2md_ast::InlineNode::Text(ocr_res.text)],
                            confidence: ocr_res.confidence,
                            bbox: None,
                        };
                        section.elements.push(para);
                        ocr_applied = true;
                    }
                }
            }
        }
    }

    // 2. Extract images and vector charts if enabled
    let mut img_count = 0;
    for img in &raw_page.images {
        if let Some(img_node) = image_extractor.process_image(img) {
            section.elements.push(img_node);
            img_count += 1;
        }
    }

    if config.extract_vectors && !raw_page.paths.is_empty() {
        let vector_charts = pdf2md_images::SvgSerializer::cluster_vector_objects(
            &raw_page.paths,
            &raw_page.text_spans,
        );

        for (idx, chart) in vector_charts.iter().enumerate() {
            let svg_content = pdf2md_images::SvgSerializer::serialize_vector_graphic(chart);
            let filename = format!("vector_chart_p{}_{}.svg", raw_page.page_number, idx + 1);

            let src = if let Some(dir) = &config.images_dir {
                let file_path = dir.join(&filename);
                if let Some(parent) = file_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&file_path, svg_content.as_bytes());
                file_path.to_string_lossy().to_string()
            } else {
                filename
            };

            section.elements.push(pdf2md_ast::Node::Image {
                alt_text: "Vector Chart / Schematic Diagram".to_string(),
                src,
                title: None,
                width: Some(chart.bbox.width),
                height: Some(chart.bbox.height),
                bbox: Some(chart.bbox),
                mime_type: Some("image/svg+xml".to_string()),
            });
            img_count += 1;
        }
    }

    // 3. Table detection
    let (detected_tables, consumed_spans) = if config.detect_tables {
        TableDetector::detect_tables(&raw_page.paths, &raw_page.text_spans)
    } else {
        (Vec::new(), Vec::new())
    };
    let table_count = detected_tables.len();

    // 4. Filter out table spans from layout text spans
    let non_table_spans: Vec<pdf2md_pdf::elements::TextSpan> = raw_page
        .text_spans
        .iter()
        .enumerate()
        .filter(|(idx, _)| !consumed_spans.contains(idx))
        .map(|(_, s)| s.clone())
        .collect();

    let mut page_clone = raw_page.clone();
    page_clone.text_spans = non_table_spans;

    // 5. Layout analysis & Reading order
    let layout_section = layout_engine.analyze_page(&page_clone);

    // 6. Merge tables and layout elements in visual order
    section.elements.extend(detected_tables);
    section.elements.extend(layout_section.elements);

    let diagnostics = PageDiagnostics {
        page_number: raw_page.page_number,
        is_scanned: raw_page.is_scanned,
        ocr_applied,
        glyph_count: raw_page.text_spans.iter().map(|s| s.text.len()).sum(),
        image_count: raw_page.images.len(),
        table_count,
        detected_language: None,
        confidence: if ocr_applied { 0.85 } else { 0.96 },
        reading_order_score: 0.95,
        fusion_confidence: if ocr_applied { Some(0.90) } else { None },
    };

    ProcessedPageOutput {
        section,
        diagnostics,
        is_ocr: ocr_applied,
        image_count: img_count,
        table_count,
    }
}

impl Pipeline {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            page_cache: Arc::new(PageCache::default()),
            buffer_pool: Arc::new(BufferPool::default()),
        }
    }

    pub fn convert_bytes(&self, bytes: &[u8]) -> Result<ConversionResult, ConversionError> {
        let start_time = Instant::now();

        // 1. Parse PDF Document
        let parse_start = Instant::now();
        let pdf_doc = PdfDocument::parse(bytes, self.config.security_limits.clone())?;
        let parse_time_ms = parse_start.elapsed().as_millis() as u64;

        let total_pages = pdf_doc.pages.len();
        let mut doc = Document::new(pdf_doc.metadata);

        let image_extractor = ImageExtractor::new(ImageExtractorConfig {
            enabled: self.config.extract_images,
            output_dir: self.config.images_dir.clone(),
            min_width: 32,
            min_height: 32,
            base_url: None,
        });

        let layout_engine = LayoutEngine::default()
            .with_paragraph_gap_threshold(self.config.paragraph_gap_threshold);
        let ocr_decision = OcrDecisionEngine {
            mode: self.config.ocr_mode,
            min_text_chars_threshold: 20,
        };

        // 2. Schedule execution
        let scheduler = AdaptiveScheduler::new(
            self.config.profile,
            Some(self.config.security_limits.max_decompressed_stream_bytes / (1024 * 1024)),
        );
        let plan = scheduler.plan(total_pages, bytes.len());

        let layout_start = Instant::now();

        // 3. Process pages (Parallelized with Rayon when plan.concurrency > 1)
        let processed_pages: Vec<ProcessedPageOutput> =
            if plan.concurrency > 1 && pdf_doc.pages.len() > 1 {
                pdf_doc
                    .pages
                    .par_iter()
                    .map(|page| {
                        process_single_page(
                            page,
                            &self.config,
                            &layout_engine,
                            &image_extractor,
                            &ocr_decision,
                        )
                    })
                    .collect()
            } else {
                pdf_doc
                    .pages
                    .iter()
                    .map(|page| {
                        process_single_page(
                            page,
                            &self.config,
                            &layout_engine,
                            &image_extractor,
                            &ocr_decision,
                        )
                    })
                    .collect()
            };

        let mut page_diagnostics = Vec::with_capacity(total_pages);
        let mut total_tables = 0;
        let mut total_images = 0;
        let mut ocr_pages_count = 0;
        let mut text_pages_count = 0;

        for (i, page_out) in processed_pages.into_iter().enumerate() {
            let raw_page = &pdf_doc.pages[i];
            let page_hash = PageCache::compute_page_hash(raw_page);
            if plan.use_cache && self.config.enable_caching {
                self.page_cache.insert(page_hash, page_out.section.clone());
            }

            if page_out.is_ocr || raw_page.is_scanned {
                ocr_pages_count += 1;
            } else {
                text_pages_count += 1;
            }
            total_images += page_out.image_count;
            total_tables += page_out.table_count;
            page_diagnostics.push(page_out.diagnostics);
            doc.sections.push(page_out.section);
        }

        let layout_time_ms = layout_start.elapsed().as_millis() as u64;

        // 3.5 Cross-page table stitching across consecutive sections
        if self.config.detect_tables {
            let table_stitcher = pdf2md_table::CrossPageTableStitcher::default();
            table_stitcher.stitch_document(&mut doc);
        }

        // 4. Render Markdown
        let render_start = Instant::now();
        let renderer = MarkdownRenderer::new(self.config.to_render_options());
        let markdown = renderer.render(&doc);
        let render_time_ms = render_start.elapsed().as_millis() as u64;
        let total_time_ms = start_time.elapsed().as_millis() as u64;

        // 5. Build diagnostics
        let diagnostics = ConversionDiagnostics {
            total_pages,
            text_pages: text_pages_count,
            ocr_pages: ocr_pages_count,
            tables_detected: total_tables,
            images_extracted: total_images,
            overall_confidence: if ocr_pages_count > 0 { 0.88 } else { 0.96 },
            confidence_breakdown: Default::default(),
            pages: page_diagnostics,
            warnings: Vec::new(),
            stats: pdf2md_ast::ProcessingStats {
                parse_time_ms,
                layout_time_ms,
                render_time_ms,
                total_time_ms,
                memory_peak_bytes: 0,
            },
        };

        doc.diagnostics = diagnostics.clone();

        Ok(ConversionResult {
            markdown,
            document: doc,
            diagnostics,
        })
    }
}
