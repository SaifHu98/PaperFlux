use pdf2md_ast::geometry::{Baseline, BoundingBox, Color, Matrix, WritingDirection};
use pdf2md_ast::{CaptionTarget, Node};
use pdf2md_layout::LayoutEngine;
use pdf2md_pdf::elements::{ImageObject, RawPage, TextSpan};

#[allow(clippy::too_many_arguments)]
fn create_span(
    text: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    font_size: f32,
    is_bold: bool,
    is_italic: bool,
    is_monospace: bool,
) -> TextSpan {
    let bbox = BoundingBox::new(x, y, w, h);
    let baseline = Baseline::new(y + h, x, x + w);
    TextSpan {
        text: text.to_string(),
        bbox,
        baseline,
        font_name: if is_monospace {
            "Courier".into()
        } else if is_bold {
            "Helvetica-Bold".into()
        } else {
            "Helvetica".into()
        },
        font_size,
        is_bold,
        is_italic,
        is_monospace,
        color: Color::BLACK,
        matrix: Matrix::IDENTITY,
        char_spacing: 0.0,
        word_spacing: 0.0,
        leading: 0.0,
        direction: if pdf2md_text::bidi::contains_rtl(text) {
            WritingDirection::RightToLeft
        } else {
            WritingDirection::LeftToRight
        },
    }
}

#[test]
fn test_golden_academic_two_column_paper() {
    let mut page = RawPage::new(1, 612.0, 792.0);

    // Title (full width at top)
    page.text_spans.push(create_span(
        "Deep Residual Learning for Document Synthesis",
        72.0,
        60.0,
        468.0,
        24.0,
        20.0,
        true,
        false,
        false,
    ));
    page.text_spans.push(create_span(
        "Authors: Jane Doe, John Smith",
        72.0,
        95.0,
        300.0,
        14.0,
        11.0,
        false,
        true,
        false,
    ));

    // Abstract (full width)
    page.text_spans.push(create_span(
        "Abstract", 72.0, 125.0, 100.0, 14.0, 12.0, true, false, false,
    ));
    page.text_spans.push(create_span(
        "We present a novel neural architecture capable of parsing multi-column documents accurately.",
        72.0,
        145.0,
        468.0,
        12.0,
        10.0,
        false,
        false,
        false,
    ));

    // Left Column (x: 72..280)
    page.text_spans.push(create_span(
        "1. Introduction",
        72.0,
        180.0,
        150.0,
        16.0,
        14.0,
        true,
        false,
        false,
    ));
    page.text_spans.push(create_span(
        "Document layout analysis is a central problem in computer vision and information retrieval.",
        72.0,
        205.0,
        200.0,
        12.0,
        10.0,
        false,
        false,
        false,
    ));
    page.text_spans.push(create_span(
        "Earlier heuristics relied on handcrafted projection profiles.",
        72.0,
        225.0,
        200.0,
        12.0,
        10.0,
        false,
        false,
        false,
    ));

    // Right Column (x: 330..540)
    page.text_spans.push(create_span(
        "2. Related Work",
        330.0,
        180.0,
        150.0,
        16.0,
        14.0,
        true,
        false,
        false,
    ));
    page.text_spans.push(create_span(
        "Recent work leverages graph neural networks to reconstruct reading order across multi-column pages.",
        330.0,
        205.0,
        200.0,
        12.0,
        10.0,
        false,
        false,
        false,
    ));

    // Caption for Figure
    page.text_spans.push(create_span(
        "Figure 1: Architectural diagram of the pipeline.",
        72.0,
        320.0,
        250.0,
        12.0,
        9.0,
        false,
        true,
        false,
    ));

    // Footnote (at bottom margin)
    page.text_spans.push(create_span(
        "1. Correspondence should be addressed to dev@ecouni.org",
        72.0,
        720.0,
        350.0,
        10.0,
        8.0,
        false,
        false,
        false,
    ));

    let layout_engine = LayoutEngine::default();
    let section = layout_engine.analyze_page(&page);

    assert!(!section.elements.is_empty());
    let has_title = section
        .elements
        .iter()
        .any(|n| matches!(n, Node::Heading { level: 1, .. }));
    let has_sections = section
        .elements
        .iter()
        .any(|n| matches!(n, Node::Heading { level: 2 | 3, .. }));
    let has_caption = section.elements.iter().any(|n| {
        matches!(
            n,
            Node::Caption {
                target_type: CaptionTarget::Figure,
                ..
            }
        )
    });
    let has_footnote = section
        .elements
        .iter()
        .any(|n| matches!(n, Node::Footnote { id, .. } if id == "1"));

    assert!(has_title, "Should extract main title as H1");
    assert!(has_sections, "Should extract numbered sections as H2/H3");
    assert!(has_caption, "Should extract figure caption");
    assert!(has_footnote, "Should extract bottom footnote");
}

#[test]
fn test_golden_book_chapter_with_quotes() {
    let mut page = RawPage::new(1, 612.0, 792.0);

    // Header & Page Number (should be stripped)
    page.text_spans.push(create_span(
        "CHAPTER 4. SYSTEMS ARCHITECTURE",
        200.0,
        30.0,
        250.0,
        10.0,
        9.0,
        false,
        true,
        false,
    ));
    page.text_spans.push(create_span(
        "73", 520.0, 30.0, 20.0, 10.0, 9.0, false, false, false,
    ));

    // Chapter Title
    page.text_spans.push(create_span(
        "4. The Microkernel Philosophy",
        72.0,
        80.0,
        400.0,
        22.0,
        18.0,
        true,
        false,
        false,
    ));

    // Paragraph
    page.text_spans.push(create_span(
        "A microkernel provides only the minimal mechanisms required to implement an operating system.",
        72.0,
        120.0,
        450.0,
        14.0,
        11.0,
        false,
        false,
        false,
    ));

    // Blockquote (indented by > 30pt, italic)
    page.text_spans.push(create_span(
        "Perfection is achieved not when there is nothing more to add, but when there is nothing left to take away.",
        120.0,
        160.0,
        380.0,
        14.0,
        11.0,
        false,
        true,
        false,
    ));

    let layout_engine = LayoutEngine::default();
    let section = layout_engine.analyze_page(&page);

    let has_running_header = section.elements.iter().any(|n| match n {
        Node::Paragraph { inlines, .. } => {
            inlines.iter().any(|i| i.plain_text().contains("CHAPTER 4"))
        }
        _ => false,
    });
    assert!(!has_running_header, "Running header must be filtered out");

    let has_blockquote = section
        .elements
        .iter()
        .any(|n| matches!(n, Node::BlockQuote { .. }));
    assert!(
        has_blockquote,
        "Indented italicized passage should become a BlockQuote"
    );
}

#[test]
fn test_golden_technical_manual_code_blocks() {
    let mut page = RawPage::new(1, 612.0, 792.0);

    page.text_spans.push(create_span(
        "Configuration Reference",
        72.0,
        60.0,
        300.0,
        20.0,
        16.0,
        true,
        false,
        false,
    ));
    page.text_spans.push(create_span(
        "To initialize the converter, run the following code:",
        72.0,
        95.0,
        400.0,
        14.0,
        11.0,
        false,
        false,
        false,
    ));

    // Monospace Code Block
    page.text_spans.push(create_span(
        "let config = Config::builder()",
        72.0,
        130.0,
        250.0,
        12.0,
        10.0,
        false,
        false,
        true,
    ));
    page.text_spans.push(create_span(
        "    .dialect(MarkdownDialect::GitHubFlavored)",
        72.0,
        145.0,
        320.0,
        12.0,
        10.0,
        false,
        false,
        true,
    ));
    page.text_spans.push(create_span(
        "    .build();",
        72.0,
        160.0,
        100.0,
        12.0,
        10.0,
        false,
        false,
        true,
    ));

    // Nested numbered list
    page.text_spans.push(create_span(
        "1. Install Rust toolchain",
        72.0,
        200.0,
        200.0,
        12.0,
        10.0,
        false,
        false,
        false,
    ));
    page.text_spans.push(create_span(
        "2. Run cargo build --release",
        72.0,
        220.0,
        220.0,
        12.0,
        10.0,
        false,
        false,
        false,
    ));

    let layout_engine = LayoutEngine::default();
    let section = layout_engine.analyze_page(&page);

    let has_code = section
        .elements
        .iter()
        .any(|n| matches!(n, Node::CodeBlock { .. }));
    let has_list = section
        .elements
        .iter()
        .any(|n| matches!(n, Node::List { ordered: true, .. }));

    assert!(has_code, "Monospaced lines should be detected as CodeBlock");
    assert!(has_list, "Numbered steps should be detected as List");
}

#[test]
fn test_golden_arabic_rtl_document() {
    let mut page = RawPage::new(1, 612.0, 792.0);

    // Arabic Heading
    page.text_spans.push(create_span(
        "تقرير الأداء الفني",
        72.0,
        60.0,
        300.0,
        22.0,
        18.0,
        true,
        false,
        false,
    ));

    // Mixed Arabic/English Paragraph
    page.text_spans.push(create_span(
        "تم تصميم نظام pdf2md ليدعم اللغة العربية بدقة عالية وسرعة فائقة.",
        72.0,
        100.0,
        450.0,
        14.0,
        11.0,
        false,
        false,
        false,
    ));

    // Arabic bullet points
    page.text_spans.push(create_span(
        "• دعم كامل لليونيكود",
        72.0,
        130.0,
        200.0,
        12.0,
        10.0,
        false,
        false,
        false,
    ));
    page.text_spans.push(create_span(
        "• معالجة الجداول متعددة الأعمدة",
        72.0,
        150.0,
        250.0,
        12.0,
        10.0,
        false,
        false,
        false,
    ));

    let layout_engine = LayoutEngine::default();
    let section = layout_engine.analyze_page(&page);

    let has_arabic_heading = section
        .elements
        .iter()
        .any(|n| matches!(n, Node::Heading { .. }));
    let has_arabic_list = section
        .elements
        .iter()
        .any(|n| matches!(n, Node::List { .. }));

    assert!(
        has_arabic_heading,
        "Arabic title should be recognized as Heading"
    );
    assert!(
        has_arabic_list,
        "Arabic bullet points should be recognized as List"
    );
}

#[test]
fn test_golden_hebrew_rtl_document() {
    let mut page = RawPage::new(1, 612.0, 792.0);

    // Hebrew Heading
    page.text_spans.push(create_span(
        "דו\"ח ביצועי מערכת",
        72.0,
        60.0,
        300.0,
        22.0,
        18.0,
        true,
        false,
        false,
    ));

    // Hebrew Paragraph
    page.text_spans.push(create_span(
        "מערכת pdf2md מספקת דיוק מקסימלי בהמרת קבצי PDF למסמכי Markdown.",
        72.0,
        100.0,
        450.0,
        14.0,
        11.0,
        false,
        false,
        false,
    ));

    let layout_engine = LayoutEngine::default();
    let section = layout_engine.analyze_page(&page);

    let has_hebrew_heading = section
        .elements
        .iter()
        .any(|n| matches!(n, Node::Heading { .. }));
    assert!(
        has_hebrew_heading,
        "Hebrew title should be recognized as Heading"
    );
}

#[test]
fn test_golden_cjk_document() {
    let mut page = RawPage::new(1, 612.0, 792.0);

    page.text_spans.push(create_span(
        "システムアーキテクチャの概要",
        72.0,
        60.0,
        300.0,
        22.0,
        18.0,
        true,
        false,
        false,
    ));

    page.text_spans.push(create_span(
        "本システムは、PDFドキュメントを解析し、",
        72.0,
        100.0,
        350.0,
        14.0,
        11.0,
        false,
        false,
        false,
    ));
    page.text_spans.push(create_span(
        "構造化されたMarkdownを高速に生成します。",
        72.0,
        118.0,
        350.0,
        14.0,
        11.0,
        false,
        false,
        false,
    ));

    let layout_engine = LayoutEngine::default();
    let section = layout_engine.analyze_page(&page);

    let para = section
        .elements
        .iter()
        .find(|n| matches!(n, Node::Paragraph { .. }));
    assert!(para.is_some(), "Japanese paragraph should be generated");

    if let Some(Node::Paragraph { inlines, .. }) = para {
        let text = inlines[0].plain_text();
        assert!(text.contains(
            "本システムは、PDFドキュメントを解析し、構造化されたMarkdownを高速に生成します。"
        ));
    }
}

#[test]
fn test_golden_legal_contract_clauses() {
    let mut page = RawPage::new(1, 612.0, 792.0);

    page.text_spans.push(create_span(
        "MASTER SERVICES AGREEMENT",
        72.0,
        60.0,
        400.0,
        22.0,
        18.0,
        true,
        false,
        false,
    ));

    // Hierarchical clauses
    page.text_spans.push(create_span(
        "1. DEFINITIONS AND INTERPRETATION",
        72.0,
        100.0,
        350.0,
        14.0,
        12.0,
        true,
        false,
        false,
    ));
    page.text_spans.push(create_span(
        "1.1. Confidential Information shall mean all non-public proprietary data.",
        72.0,
        125.0,
        450.0,
        12.0,
        10.0,
        false,
        false,
        false,
    ));
    page.text_spans.push(create_span(
        "1.2. Effective Date means the date of final signature.",
        72.0,
        145.0,
        400.0,
        12.0,
        10.0,
        false,
        false,
        false,
    ));

    let layout_engine = LayoutEngine::default();
    let section = layout_engine.analyze_page(&page);

    let has_main_title = section
        .elements
        .iter()
        .any(|n| matches!(n, Node::Heading { level: 1, .. }));
    let has_clauses = section.elements.iter().any(|n| {
        matches!(
            n,
            Node::List { ordered: true, .. } | Node::Heading { level: 2..=4, .. }
        )
    });

    assert!(has_main_title, "Legal title should be H1");
    assert!(
        has_clauses,
        "Legal clauses should be recognized structurally"
    );
}

#[test]
fn test_golden_scanned_pdf_capability_assessment() {
    let mut page = RawPage::new(1, 612.0, 792.0);

    page.images.push(ImageObject {
        id: "Img1".into(),
        bbox: BoundingBox::new(0.0, 0.0, 612.0, 792.0),
        width: 1200,
        height: 1600,
        mime_type: "image/jpeg".into(),
        data: vec![0xFF, 0xD8, 0xFF, 0xE0],
    });

    page.assess_capabilities();

    assert!(!page.has_usable_text, "Should have no digital text");
    assert!(
        page.is_scanned,
        "Should be classified as scanned/image-only page"
    );
}
