use pdf2md_text::arabic::context::{ArabicProcessingContext, NumeralSystem};
use pdf2md_text::arabic::pipeline::ArabicTextPipeline;
use pdf2md_text::arabic::semantic_normalizer::{
    ArabicNumericExpression, ArabicScholarlyDetector, ArabicScholarlySectionKind,
    ArabicSemanticNormalizer,
};

#[test]
fn test_default_numeral_preservation() {
    let ctx = ArabicProcessingContext::default();
    assert_eq!(ctx.numeral_system, NumeralSystem::PreserveAsIs);

    let eastern = "صدر التقرير بتاريخ ٢٠٢٦/٠٨/٢٠ بمعدل ٩٨٫٥٪";
    let (proc_eastern, _) = ArabicTextPipeline::process(eastern, &ctx);
    // Preserves Eastern Arabic-Indic digits by default
    assert!(proc_eastern.contains("٢٠٢٦/٠٨/٢٠"));
    assert!(proc_eastern.contains("٩٨٫٥٪"));

    let western = "صدر التقرير بتاريخ 2026/08/20 بمعدل 98.5%";
    let (proc_western, _) = ArabicTextPipeline::process(western, &ctx);
    // Preserves Western Arabic digits by default
    assert!(proc_western.contains("2026/08/20"));
    assert!(proc_western.contains("98.5%"));
}

#[test]
fn test_complex_numerical_expressions_detection() {
    let text =
        "في تاريخ ٢٠٢٦/٠٨/٢٠ عند الساعة ١٤:٣٠ بلغت النسبة ٩٨٫٥٪ بقيمة ١٥٠ ر.س حسب [1] في ص. 24";
    let expressions = ArabicSemanticNormalizer::detect_expressions(text);

    assert!(expressions.contains(&ArabicNumericExpression::Date("٢٠٢٦/٠٨/٢٠".to_string())));
    assert!(expressions.contains(&ArabicNumericExpression::Time("١٤:٣٠".to_string())));
    assert!(expressions.contains(&ArabicNumericExpression::Percentage("٩٨٫٥٪".to_string())));
    assert!(expressions.contains(&ArabicNumericExpression::Currency {
        value: "١٥٠".to_string(),
        unit: "ر.س".to_string()
    }));
    assert!(expressions.contains(&ArabicNumericExpression::Citation("[1]".to_string())));
    assert!(expressions.contains(&ArabicNumericExpression::PageReference("ص. 24".to_string())));
}

#[test]
fn test_statistical_scholarly_section_detection() {
    assert_eq!(
        ArabicScholarlyDetector::classify_heading("المستخلص").map(|(k, _)| k),
        Some(ArabicScholarlySectionKind::Abstract)
    );
    assert_eq!(
        ArabicScholarlyDetector::classify_heading("مدخل وتمهيد عام").map(|(k, _)| k),
        Some(ArabicScholarlySectionKind::Introduction)
    );
    assert_eq!(
        ArabicScholarlyDetector::classify_heading("إجراءات الدراسة والمنهجية").map(|(k, _)| k),
        Some(ArabicScholarlySectionKind::Methodology)
    );
    assert_eq!(
        ArabicScholarlyDetector::classify_heading("معطيات الدراسة والنتائج").map(|(k, _)| k),
        Some(ArabicScholarlySectionKind::Results)
    );
    assert_eq!(
        ArabicScholarlyDetector::classify_heading("قائمة المصادر والمراجع").map(|(k, _)| k),
        Some(ArabicScholarlySectionKind::References)
    );
}

#[test]
fn test_scholarly_markdown_formatting() {
    let heading = ArabicScholarlyDetector::format_scholarly_heading("المقدمة", 2);
    assert_eq!(heading, "## المقدمة");

    let citation = ArabicScholarlyDetector::format_citation("(1)");
    assert_eq!(citation, "[^1]");
}
