use pdf2md_ast::geometry::BoundingBox;
use pdf2md_ocr::arabic_ocr::{ArabicOcrDecision, ArabicOcrDecisionEngine, ArabicOcrFusionEngine};
use pdf2md_ocr::provider::{MockOCRProvider, OCRProvider};
use pdf2md_pdf::elements::{ImageObject, RawPage, TextSpan};

#[test]
fn test_ocr_stream_fusion_integration_scanned_document() {
    // 1. Create a scanned image page with partial native text (simulating dual stream)
    let images = vec![ImageObject {
        id: "scan_doc_1".to_string(),
        bbox: BoundingBox::new(0.0, 0.0, 600.0, 800.0),
        width: 1200,
        height: 1600,
        mime_type: "image/png".to_string(),
        data: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
    }];

    let corrupted_spans = vec![
        TextSpan::new(
            "تقرير \u{FFFD}\u{FFFD}ل سنوي".to_string(),
            BoundingBox::new(50.0, 50.0, 200.0, 20.0),
            "Font1".to_string(),
            14.0,
            false,
            false,
            false,
        ),
        TextSpan::new(
            "المملكة \u{E001}\u{E002} السعودية".to_string(),
            BoundingBox::new(50.0, 80.0, 250.0, 20.0),
            "Font1".to_string(),
            12.0,
            false,
            false,
            false,
        ),
    ];

    let page = RawPage {
        page_number: 1,
        width: 600.0,
        height: 800.0,
        rotation: 0,
        text_spans: corrupted_spans,
        paths: Vec::new(),
        vector_graphics: Vec::new(),
        images,
        has_usable_text: true,
        is_scanned: false,
    };

    // 2. Mock OCR provider with high accuracy ground truth
    let ocr_mock = MockOCRProvider::new("تقرير مالي سنوي\nالمملكة العربية السعودية");
    let ocr_result = ocr_mock
        .recognize(&page.images[0].data, Some("ara"))
        .expect("Mock OCR recognition should succeed");

    assert_eq!(ocr_result.text, "تقرير مالي سنوي\nالمملكة العربية السعودية");
    assert!(ocr_result.confidence >= 0.90);

    // 3. Evaluate preflight decision
    let decision = ArabicOcrDecisionEngine::evaluate_page(
        &page,
        pdf2md_ocr::arabic_ocr::ArabicDialectHint::SaudiArabia,
    );

    match decision {
        ArabicOcrDecision::FuseStreams {
            native_quality,
            expected_ocr_weight,
        } => {
            assert!(native_quality < 0.88);
            assert!(expected_ocr_weight > 0.0);
        }
        ArabicOcrDecision::RequireOcr { preflight, .. } => {
            assert!(preflight.native_text_quality < 0.88);
        }
        other => panic!("Expected FuseStreams or RequireOcr, got {:?}", other),
    }

    // 4. Perform character-by-character stream fusion
    let native_text = "تقرير \u{FFFD}\u{FFFD}ل سنوي\nالمملكة \u{E001}\u{E002} السعودية";
    let fusion_output = ArabicOcrFusionEngine::fuse_character_by_character(
        native_text,
        &ocr_result.text,
        0.35,
        ocr_result.confidence,
    );

    assert_eq!(
        fusion_output.fused_text,
        "تقرير مالي سنوي\nالمملكة العربية السعودية"
    );
    assert!(
        fusion_output.fusion_confidence >= 0.85,
        "Fusion confidence ({}) must exceed minimum quality threshold 0.85",
        fusion_output.fusion_confidence
    );
}
