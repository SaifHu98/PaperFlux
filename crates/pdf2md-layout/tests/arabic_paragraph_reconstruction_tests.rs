use pdf2md_ast::geometry::BoundingBox;
use pdf2md_layout::arabic_paragraph::{
    ArabicLineReconstructor, ArabicParagraphReconstructor, ArabicWordBoundaryDetector,
};
use pdf2md_pdf::elements::TextSpan;

fn make_span(text: &str, x: f32, y: f32, w: f32, h: f32, size: f32) -> TextSpan {
    TextSpan::new(
        text.to_string(),
        BoundingBox::new(x, y, w, h),
        "Amiri".to_string(),
        size,
        false,
        false,
        false,
    )
}

#[test]
fn test_fragmented_glyph_runs_merging() {
    // 5 separate glyph spans forming "تقرير" with tiny gaps (1.0pt)
    let spans = vec![
        make_span("ت", 300.0, 700.0, 8.0, 14.0, 12.0),
        make_span("ق", 291.0, 700.0, 8.0, 14.0, 12.0),
        make_span("ر", 282.0, 700.0, 8.0, 14.0, 12.0),
        make_span("ي", 273.0, 700.0, 8.0, 14.0, 12.0),
        make_span("ر", 264.0, 700.0, 8.0, 14.0, 12.0),
    ];

    let reconstructed = ArabicLineReconstructor::reconstruct_line(&spans);
    assert_eq!(reconstructed, "تقرير");
}

#[test]
fn test_true_inter_word_space_preservation() {
    // Two distinct words: "نظام" (at x: 300..260) and "متقدم" (at x: 245..190) with 15pt gap
    let spans = vec![
        make_span("نظام", 260.0, 700.0, 40.0, 14.0, 12.0),
        make_span("متقدم", 195.0, 700.0, 50.0, 14.0, 12.0),
    ];

    let reconstructed = ArabicLineReconstructor::reconstruct_line(&spans);
    assert_eq!(reconstructed, "نظام متقدم");
}

#[test]
fn test_dual_joining_and_right_joining_rules() {
    assert!(ArabicWordBoundaryDetector::is_dual_joining('ب'));
    assert!(ArabicWordBoundaryDetector::is_dual_joining('س'));
    assert!(ArabicWordBoundaryDetector::is_dual_joining('ل'));
    assert!(ArabicWordBoundaryDetector::is_dual_joining('م'));
    assert!(ArabicWordBoundaryDetector::is_dual_joining('پ')); // Persian Peh
    assert!(ArabicWordBoundaryDetector::is_dual_joining('ڵ')); // Kurdish Lam

    assert!(ArabicWordBoundaryDetector::is_right_joining('ا'));
    assert!(ArabicWordBoundaryDetector::is_right_joining('د'));
    assert!(ArabicWordBoundaryDetector::is_right_joining('ر'));
    assert!(ArabicWordBoundaryDetector::is_right_joining('و'));
    assert!(ArabicWordBoundaryDetector::is_right_joining('ژ')); // Persian Jeh
}

#[test]
fn test_arabic_tatweel_justification_normalization() {
    let spans = vec![
        make_span("كـتـاب", 200.0, 700.0, 80.0, 14.0, 12.0),
        make_span("مـمـتـاز", 100.0, 700.0, 90.0, 14.0, 12.0),
    ];

    let reconstructed = ArabicLineReconstructor::reconstruct_line(&spans);
    // Tatweel \u0640 should be normalized/stripped
    assert_eq!(reconstructed, "كتاب ممتاز");
}

#[test]
fn test_arabic_paragraph_continuation_and_boundaries() {
    let lines = vec![
        "بدأت الثورة الرقمية في تغيير ملامح الاقتصاد العالمي بشكل متسارع.".to_string(),
        "وقد شمل هذا التحول كافة القطاعات الصناعية والخدمية دون استثناء.".to_string(),
        "".to_string(),
        "المادة الأولى: تسري هذه اللائحة على جميع الشركات المعتمدة.".to_string(),
        "وتلزم جميع الأطراف بتطبيق معايير الشفافية المطلوبة.".to_string(),
    ];

    let paragraphs = ArabicParagraphReconstructor::reconstruct_paragraphs(&lines);

    assert_eq!(paragraphs.len(), 2);
    assert!(paragraphs[0].contains("بدأت الثورة الرقمية"));
    assert!(paragraphs[0].contains("دون استثناء."));
    assert!(paragraphs[1].contains("المادة الأولى:"));
    assert!(paragraphs[1].contains("معايير الشفافية المطلوبة."));
}

#[test]
fn test_statistical_line_gap_clustering() {
    // 2 paragraphs:
    // Para 1: 3 lines with normal intra-gap 14pt (y: 100, 114, 128)
    // Inter-paragraph gap: 30pt (y: 158)
    // Para 2: 2 lines with normal intra-gap 14pt (y: 158, 172)
    let lines_with_geo = vec![
        ("السطر الأول من الفقرة الأولى.".to_string(), 100.0, 12.0),
        ("السطر الثاني التابع لنفس الفقرة.".to_string(), 114.0, 12.0),
        ("السطر الثالث المكمل للفقرة الأولى.".to_string(), 128.0, 12.0),
        (
            "بداية الفقرة الثانية بعد مسافة عمودية واضحة.".to_string(),
            158.0,
            12.0,
        ),
        (
            "تكملة السطر الأخير في الفقرة الثانية.".to_string(),
            172.0,
            12.0,
        ),
    ];

    let paragraphs =
        ArabicParagraphReconstructor::reconstruct_paragraphs_with_geometry(&lines_with_geo, None);

    assert_eq!(
        paragraphs.len(),
        2,
        "Statistical clustering should detect 2 distinct paragraphs"
    );
    assert!(paragraphs[0].contains("السطر الأول"));
    assert!(paragraphs[0].contains("السطر الثالث"));
    assert!(paragraphs[1].contains("بداية الفقرة الثانية"));
    assert!(paragraphs[1].contains("تكملة السطر الأخير"));
}
