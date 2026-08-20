use pdf2md_ast::geometry::{Baseline, BoundingBox, Color, Matrix, WritingDirection};
use pdf2md_ocr::evaluator::{OcrFusionEngine, OcrNecessityEvaluator};
use pdf2md_ocr::provider::{MockOCRProvider, OCRProvider, OcrOrientation, OcrRequest};
use pdf2md_pdf::elements::{ImageObject, RawPage, TextSpan};

fn create_span(text: &str) -> TextSpan {
    let bbox = BoundingBox::new(72.0, 72.0, 400.0, 14.0);
    TextSpan {
        text: text.to_string(),
        bbox,
        baseline: Baseline::new(86.0, 72.0, 472.0),
        font_name: "Helvetica".into(),
        font_size: 11.0,
        is_bold: false,
        is_italic: false,
        is_monospace: false,
        color: Color::BLACK,
        matrix: Matrix::IDENTITY,
        char_spacing: 0.0,
        word_spacing: 0.0,
        leading: 0.0,
        direction: WritingDirection::LeftToRight,
    }
}

#[test]
fn test_ocr_decision_clean_digital_page() {
    let mut page = RawPage::new(1, 612.0, 792.0);
    page.text_spans.push(create_span("This is a clean, crisp digitally generated PDF page with well-formed fonts and standard Unicode characters."));

    let decision = OcrNecessityEvaluator::evaluate(&page);

    assert!(!decision.should_ocr, "Clean digital page must skip OCR");
    assert_eq!(decision.ocr_necessity_score, 0.0);
    assert!(!decision.is_font_corrupted);
}

#[test]
fn test_ocr_decision_scanned_image_page() {
    let mut page = RawPage::new(1, 612.0, 792.0);
    page.is_scanned = true;
    page.images.push(ImageObject {
        id: "Img1".into(),
        bbox: BoundingBox::new(0.0, 0.0, 612.0, 792.0),
        width: 1200,
        height: 1600,
        mime_type: "image/jpeg".into(),
        data: vec![0xFF, 0xD8, 0xFF, 0xE0],
    });

    let decision = OcrNecessityEvaluator::evaluate(&page);

    assert!(decision.should_ocr, "Scanned image page must trigger OCR");
    assert!(decision.is_image_only);
    assert_eq!(decision.ocr_necessity_score, 1.0);
}

#[test]
fn test_ocr_decision_corrupted_font_stream() {
    let mut page = RawPage::new(1, 612.0, 792.0);
    page.text_spans.push(create_span(
        "Th\u{FFFD}s is c\u{FFFD}rr\u{FFFD}pt\u{FFFD}d \u{E001}\u{E002}\u{E003} f\u{FFFD}nt",
    ));
    page.images.push(ImageObject {
        id: "Img1".into(),
        bbox: BoundingBox::new(72.0, 72.0, 300.0, 200.0),
        width: 600,
        height: 400,
        mime_type: "image/png".into(),
        data: vec![0x89, 0x50, 0x4E, 0x47],
    });

    let decision = OcrNecessityEvaluator::evaluate(&page);

    assert!(
        decision.should_ocr,
        "Corrupted font stream must trigger OCR"
    );
    assert!(decision.is_font_corrupted);
    assert!(decision.ocr_necessity_score >= 0.85);
}

#[test]
fn test_ocr_fusion_selection() {
    // 1. Native text is superior
    let (text1, conf1, source1) = OcrFusionEngine::select_best_stream(
        "High quality native text.",
        0.95,
        "Slightly garbled ocr text",
        0.80,
    );
    assert_eq!(source1, "native");
    assert_eq!(text1, "High quality native text.");
    assert_eq!(conf1, 0.95);

    // 2. OCR text is superior when native text is corrupted
    let (text2, conf2, source2) = OcrFusionEngine::select_best_stream(
        "C\u{FFFD}rr\u{FFFD}pted n\u{E001}t\u{E002}ve",
        0.40,
        "Corrected high confidence OCR text.",
        0.92,
    );
    assert_eq!(source2, "ocr");
    assert_eq!(text2, "Corrected high confidence OCR text.");
    assert_eq!(conf2, 0.92);
}

#[test]
fn test_pluggable_ocr_provider() {
    let provider = MockOCRProvider::default();
    assert!(provider.is_available());

    let request = OcrRequest {
        image_bytes: vec![0x01, 0x02, 0x03],
        language_hints: vec!["en".into(), "ar".into()],
        detect_orientation: true,
    };

    let result = provider.recognize_advanced(request).unwrap();
    assert_eq!(result.orientation, OcrOrientation::Rot0);
    assert_eq!(result.lines.len(), 1);
    assert_eq!(result.lines[0].words.len(), 1);
    assert!(result.confidence >= 0.90);
}

#[test]
fn test_system_tesseract_ocr_provider_configuration() {
    use pdf2md_ocr::provider::SystemTesseractOCRProvider;
    let provider = SystemTesseractOCRProvider::with_languages("ara+eng");
    assert_eq!(provider.name(), "SystemTesseractOCRProvider");
    assert_eq!(provider.languages, "ara+eng");

    // If tesseract binary is not installed on system, recognize() returns graceful error
    if !provider.is_available() {
        let err = provider.recognize(b"fake_image", None).unwrap_err();
        assert!(err.contains("Tesseract binary not found"));
    }
}
