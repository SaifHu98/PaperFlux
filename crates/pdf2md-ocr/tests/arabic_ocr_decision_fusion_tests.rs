use pdf2md_ast::geometry::BoundingBox;
use pdf2md_ocr::arabic_ocr::{
    ArabicDialectHint, ArabicOcrDecision, ArabicOcrDecisionEngine, ArabicOcrFusionEngine,
};
use pdf2md_pdf::elements::{ImageObject, RawPage, TextSpan};

fn make_text_span(text: &str, x: f32, y: f32) -> TextSpan {
    TextSpan::new(
        text.to_string(),
        BoundingBox::new(x, y, 100.0, 14.0),
        "Amiri".to_string(),
        12.0,
        false,
        false,
        false,
    )
}

#[test]
fn test_arabic_clean_digital_page_skips_ocr() {
    let spans = vec![
        make_text_span("المملكة العربية السعودية", 200.0, 700.0),
        make_text_span("تقرير الأداء المالي والتشغيلي لعام ٢٠٢٦", 200.0, 680.0),
        make_text_span(
            "حققت الشركة نمواً قياسياً بنسبة ٢٨٪ في الأرباح الصافية.",
            200.0,
            660.0,
        ),
    ];

    let page = RawPage {
        page_number: 1,
        width: 600.0,
        height: 800.0,
        rotation: 0,
        text_spans: spans,
        paths: Vec::new(),
        images: Vec::new(),
        has_usable_text: true,
        is_scanned: false,
    };

    let decision = ArabicOcrDecisionEngine::evaluate_page(&page, ArabicDialectHint::SaudiArabia);
    match decision {
        ArabicOcrDecision::SkipOcr { reason } => {
            assert!(reason.contains("High-fidelity Arabic digital text"));
        }
        other => panic!("Expected SkipOcr, got {:?}", other),
    }
}

#[test]
fn test_arabic_scanned_image_page_triggers_ocr() {
    let images = vec![ImageObject {
        id: "scan_1".to_string(),
        bbox: BoundingBox::new(0.0, 0.0, 600.0, 800.0),
        width: 1200,
        height: 1600,
        mime_type: "image/png".to_string(),
        data: vec![0x89, 0x50, 0x4E, 0x47],
    }];

    let page = RawPage {
        page_number: 1,
        width: 600.0,
        height: 800.0,
        rotation: 0,
        text_spans: Vec::new(), // No native text spans -> Scanned
        paths: Vec::new(),
        images,
        has_usable_text: false,
        is_scanned: true,
    };

    let decision = ArabicOcrDecisionEngine::evaluate_page(&page, ArabicDialectHint::GeneralArabic);
    match decision {
        ArabicOcrDecision::RequireOcr { preflight, reason } => {
            assert_eq!(preflight.arabic_char_count, 0);
            assert!(reason.contains("Scanned image-only Arabic page"));
        }
        other => panic!("Expected RequireOcr, got {:?}", other),
    }
}

#[test]
fn test_arabic_corrupted_font_stream_triggers_ocr() {
    let corrupted_spans = vec![
        make_text_span(
            "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD} \u{E001}\u{E002}\u{E003}",
            200.0,
            700.0,
        ),
        make_text_span("\u{E010}\u{E011}\u{E012} \u{FFFD}\u{FFFD}", 200.0, 680.0),
    ];

    let page = RawPage {
        page_number: 1,
        width: 600.0,
        height: 800.0,
        rotation: 0,
        text_spans: corrupted_spans,
        paths: Vec::new(),
        images: Vec::new(),
        has_usable_text: true,
        is_scanned: false,
    };

    let decision = ArabicOcrDecisionEngine::evaluate_page(&page, ArabicDialectHint::Egypt);
    match decision {
        ArabicOcrDecision::RequireOcr { preflight, reason } => {
            assert!(preflight.native_text_quality < 0.50);
            assert!(reason.contains("Corrupted font encoding"));
        }
        other => panic!("Expected RequireOcr, got {:?}", other),
    }
}

#[test]
fn test_arabic_dialect_hints_parsing() {
    assert_eq!(
        ArabicDialectHint::parse("ar-SA"),
        ArabicDialectHint::SaudiArabia
    );
    assert_eq!(ArabicDialectHint::parse("ar-EG"), ArabicDialectHint::Egypt);
    assert_eq!(ArabicDialectHint::parse("ar-IQ"), ArabicDialectHint::Iraq);
    assert_eq!(ArabicDialectHint::parse("ar-AE"), ArabicDialectHint::UAE);
    assert_eq!(
        ArabicDialectHint::parse("ar-MA"),
        ArabicDialectHint::Morocco
    );
    assert_eq!(
        ArabicDialectHint::parse("ar-DZ"),
        ArabicDialectHint::Algeria
    );
    assert_eq!(
        ArabicDialectHint::parse("ar"),
        ArabicDialectHint::GeneralArabic
    );
}

#[test]
fn test_arabic_stream_fusion_engine() {
    let native_text = "تقرير مالي لعام \u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}";
    let ocr_text = "تقرير مالي لعام 2026";

    let (fused, conf) = ArabicOcrFusionEngine::fuse_streams(native_text, ocr_text, 0.40, 0.95);
    assert_eq!(fused, "تقرير مالي لعام 2026");
    assert!(conf >= 0.60);
}
