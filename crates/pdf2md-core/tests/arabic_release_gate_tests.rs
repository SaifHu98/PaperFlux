use pdf2md_core::arabic_benchmark::ArabicQualityScore;

#[test]
fn test_arabic_release_gate_weight_normalization() {
    let perfect = ArabicQualityScore::default();
    let composite = perfect.composite_score();
    // Sum of weights must equal 1.0
    assert!((composite - 1.0).abs() < 0.001);
    assert!(perfect.satisfies_release_gate());
}

#[test]
fn test_arabic_release_gate_enforces_strict_subscores() {
    // If composite is high but one critical sub-score (e.g. reading order) is too low (< 0.90)
    let flawed_score = ArabicQualityScore {
        unicode_accuracy: 0.99,
        char_accuracy: 0.99,
        word_accuracy: 0.99,
        paragraph_accuracy: 0.99,
        reading_order_accuracy: 0.85, // Failed sub-score
        rtl_accuracy: 0.99,
        heading_accuracy: 0.99,
        list_accuracy: 0.99,
        table_accuracy: 0.99,
        ocr_accuracy: 0.99,
        mixed_script_accuracy: 0.99,
        markdown_structural_accuracy: 0.99,
    };

    assert!(!flawed_score.satisfies_release_gate());
}

#[test]
fn test_arabic_release_gate_requires_minimum_composite() {
    let marginal_score = ArabicQualityScore {
        unicode_accuracy: 0.92,
        char_accuracy: 0.92,
        word_accuracy: 0.92,
        paragraph_accuracy: 0.92,
        reading_order_accuracy: 0.92,
        rtl_accuracy: 0.92,
        heading_accuracy: 0.92,
        list_accuracy: 0.92,
        table_accuracy: 0.92,
        ocr_accuracy: 0.92,
        mixed_script_accuracy: 0.92,
        markdown_structural_accuracy: 0.92,
    };

    // Composite is 0.92 < 0.95 -> Gate fails
    assert!(!marginal_score.satisfies_release_gate());
}
