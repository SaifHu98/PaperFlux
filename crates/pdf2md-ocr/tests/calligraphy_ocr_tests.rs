use pdf2md_ast::geometry::BoundingBox;
use pdf2md_ocr::arabic_ocr::{ArabicDialectHint, ArabicOcrDecision, ArabicOcrDecisionEngine};
use pdf2md_ocr::calligraphy::{CalligraphicScriptType, CalligraphyDetector};
use pdf2md_pdf::elements::{ImageObject, RawPage, TextSpan};

fn make_span(text: &str, font: &str, bbox: BoundingBox) -> TextSpan {
    TextSpan::new(
        text.to_string(),
        bbox,
        font.to_string(),
        14.0,
        false,
        false,
        false,
    )
}

#[test]
fn test_nastaliq_font_detection_and_300_dpi_boost() {
    let mut page = RawPage::new(1, 612.0, 792.0);
    page.text_spans.push(make_span(
        "نستعلیق خط میں لکھا ہوا کلام",
        "Jameel Noori Nastaleeq",
        BoundingBox::new(100.0, 100.0, 300.0, 30.0),
    ));
    page.images.push(ImageObject {
        id: "img1".to_string(),
        data: vec![0u8; 100],
        mime_type: "image/png".to_string(),
        width: 300,
        height: 200,
        bbox: BoundingBox::new(100.0, 150.0, 300.0, 200.0),
    });

    let det = CalligraphyDetector::detect(&page);
    assert!(det.is_calligraphic);
    assert_eq!(det.script_type, Some(CalligraphicScriptType::Nastaliq));
    assert_eq!(det.recommended_dpi, 300);

    let decision = ArabicOcrDecisionEngine::evaluate_page(&page, ArabicDialectHint::GeneralArabic);
    if let ArabicOcrDecision::RequireOcr { preflight, reason } = decision {
        assert!(preflight.is_calligraphic);
        assert_eq!(preflight.calligraphic_script, Some("Nastaliq".to_string()));
        assert_eq!(preflight.estimated_dpi, 300);
        assert!(reason.contains("300 DPI"));
    } else {
        panic!("Expected RequireOcr for Nastaliq with image / non-pure digital");
    }
}

#[test]
fn test_diwani_font_detection_and_300_dpi_boost() {
    let mut page = RawPage::new(1, 612.0, 792.0);
    page.text_spans.push(make_span(
        "الخط الديواني الجلي الأصيل",
        "Diwan Letter Calligraphic",
        BoundingBox::new(100.0, 100.0, 300.0, 35.0),
    ));

    let det = CalligraphyDetector::detect(&page);
    assert!(det.is_calligraphic);
    assert_eq!(det.script_type, Some(CalligraphicScriptType::Diwani));
    assert_eq!(det.recommended_dpi, 300);
}

#[test]
fn test_geometric_cascading_baseline_and_overlap_detection() {
    let mut page = RawPage::new(1, 612.0, 792.0);
    // 4 overlapping spans with vertical cascading step (slanted baseline)
    page.text_spans.push(make_span(
        "کلمہ اول",
        "CustomFont1",
        BoundingBox::new(100.0, 100.0, 80.0, 20.0),
    ));
    page.text_spans.push(make_span(
        "کلمہ دوم",
        "CustomFont1",
        BoundingBox::new(120.0, 108.0, 80.0, 20.0),
    ));
    page.text_spans.push(make_span(
        "کلمہ سوم",
        "CustomFont1",
        BoundingBox::new(140.0, 116.0, 80.0, 20.0),
    ));
    page.text_spans.push(make_span(
        "کلمہ چہارم",
        "CustomFont1",
        BoundingBox::new(160.0, 124.0, 80.0, 20.0),
    ));

    let det = CalligraphyDetector::detect(&page);
    assert!(det.is_calligraphic);
    assert_eq!(det.script_type, Some(CalligraphicScriptType::Nastaliq));
    assert_eq!(det.recommended_dpi, 300);
}

#[test]
fn test_standard_naskh_horizontal_typography_normal_dpi() {
    let mut page = RawPage::new(1, 612.0, 792.0);
    page.text_spans.push(make_span(
        "النص العربي القياسي بخط النسخ",
        "Traditional Arabic",
        BoundingBox::new(100.0, 100.0, 300.0, 18.0),
    ));

    let det = CalligraphyDetector::detect(&page);
    assert!(!det.is_calligraphic);
    assert_eq!(det.recommended_dpi, 150);
}

#[test]
fn test_statistical_font_clustering_synthetic_nastaliq_fixture() {
    let mut page = RawPage::new(1, 612.0, 792.0);
    // Generic font name with NO Nastaliq keywords in font name
    let font_name = "EmbeddedCustomType3Font_ABC123";

    // 6 glyph spans with cascading diagonal baselines (slope ~ 0.5) and 50% horizontal overlap
    for i in 0..6 {
        let x = 100.0 + (i as f32 * 14.0);
        let y = 200.0 + (i as f32 * 7.5);
        page.text_spans.push(make_span(
            &format!("حرف_{}", i),
            font_name,
            BoundingBox::new(x, y, 28.0, 22.0),
        ));
    }

    let det = CalligraphyDetector::detect(&page);
    assert!(det.is_calligraphic);
    assert_eq!(det.script_type, Some(CalligraphicScriptType::Nastaliq));
    assert!(
        det.confidence >= 0.75,
        "Expected confidence >= 0.75, got {}",
        det.confidence
    );
    assert_eq!(det.recommended_dpi, 300);

    let metrics = det
        .metrics
        .expect("Expected StatisticalGlyphMetrics to be populated");
    assert!(metrics.overlap_density >= 0.50);
    assert!(metrics.diagonal_slope_ratio >= 0.50);
    assert!(metrics.baseline_deviation > 0.0);

    let decision = ArabicOcrDecisionEngine::evaluate_page(&page, ArabicDialectHint::GeneralArabic);
    if let ArabicOcrDecision::RequireOcr { preflight, reason } = decision {
        assert!(preflight.is_calligraphic);
        assert_eq!(preflight.calligraphic_script, Some("Nastaliq".to_string()));
        assert_eq!(preflight.estimated_dpi, 300);
        assert!(reason.contains("300 DPI"));
    } else {
        panic!("Expected RequireOcr with 300 DPI escalation for synthetic Nastaliq clustering");
    }
}
