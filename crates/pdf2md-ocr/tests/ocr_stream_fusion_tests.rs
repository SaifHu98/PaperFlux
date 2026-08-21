use pdf2md_ocr::arabic_ocr::ArabicOcrFusionEngine;
use pdf2md_ocr::evaluator::OcrFusionEngine;

#[test]
fn test_fusion_high_confidence_native_selected() {
    let native_text = "تقرير الأداء المالي والتشغيلي لشركة ألفا لعام 2026";
    let ocr_text = "تقرير الاداء المالى والتشغيلى لشركة الفا لعام 2026";

    let result =
        ArabicOcrFusionEngine::fuse_character_by_character(native_text, ocr_text, 0.98, 0.82);

    assert_eq!(result.stream_source, "native");
    assert_eq!(result.fused_text, native_text);
    assert_eq!(result.fusion_confidence, 0.98);
    assert_eq!(result.ocr_chosen_chars, 0);
    assert!(result.native_chosen_chars > 0);
}

#[test]
fn test_fusion_high_confidence_ocr_replaces_corrupted_native() {
    let native_text = "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD} \u{E001}\u{E002}\u{E003} \u{FFFD}\u{FFFD}";
    let ocr_text = "المملكة العربية السعودية";

    let result =
        ArabicOcrFusionEngine::fuse_character_by_character(native_text, ocr_text, 0.15, 0.95);

    assert_eq!(result.stream_source, "ocr");
    assert_eq!(result.fused_text, ocr_text);
    assert_eq!(result.fusion_confidence, 0.95);
    assert_eq!(result.native_chosen_chars, 0);
    assert!(result.ocr_chosen_chars > 0);
}

#[test]
fn test_character_by_character_corrupted_glyph_repair() {
    // Native has corrupted characters in the second token
    let native_text = "التقرير ال\u{FFFD}\u{FFFD}وي لعام 2026";
    let ocr_text = "التقرير السنوي لعام 2026";

    let result =
        ArabicOcrFusionEngine::fuse_character_by_character(native_text, ocr_text, 0.65, 0.90);

    assert_eq!(result.stream_source, "fused");
    assert_eq!(result.fused_text, "التقرير السنوي لعام 2026");
    assert!(
        result.ocr_chosen_chars > 0,
        "OCR should repair corrupted glyphs"
    );
    assert!(
        result.native_chosen_chars > 0,
        "Native clean characters should be preserved"
    );
    assert!(result.fusion_confidence >= 0.70);
}

#[test]
fn test_fusion_edge_cases_empty_streams() {
    // Both empty
    let empty_res = ArabicOcrFusionEngine::fuse_character_by_character("", "", 0.0, 0.0);
    assert_eq!(empty_res.stream_source, "empty");
    assert_eq!(empty_res.fused_text, "");
    assert_eq!(empty_res.fusion_confidence, 1.0);

    // Native empty, OCR present
    let ocr_only = ArabicOcrFusionEngine::fuse_character_by_character(
        "",
        "نص مستخرج عبر التعرف الضوئي",
        0.0,
        0.88,
    );
    assert_eq!(ocr_only.stream_source, "ocr");
    assert_eq!(ocr_only.fused_text, "نص مستخرج عبر التعرف الضوئي");
    assert_eq!(ocr_only.fusion_confidence, 0.88);

    // OCR empty, Native present
    let native_only =
        ArabicOcrFusionEngine::fuse_character_by_character("نص رقمي أصلي مباشر", "", 0.91, 0.0);
    assert_eq!(native_only.stream_source, "native");
    assert_eq!(native_only.fused_text, "نص رقمي أصلي مباشر");
    assert_eq!(native_only.fusion_confidence, 0.91);
}

#[test]
fn test_fusion_edge_cases_identical_streams() {
    let text = "جامعة الملك سعود - كلية علوم الحاسب والمعلومات";
    let result = ArabicOcrFusionEngine::fuse_character_by_character(text, text, 0.85, 0.88);

    assert_eq!(result.stream_source, "identical");
    assert_eq!(result.fused_text, text);
    assert_eq!(result.fusion_confidence, 0.88);
}

#[test]
fn test_fusion_edge_cases_length_and_formatting_mismatches() {
    let native_text = "ملخص البحث التنفيذي";
    let ocr_text = "ملخص البحث التنفيذي مع ملحقات إضافية مفصلة";

    let result =
        ArabicOcrFusionEngine::fuse_character_by_character(native_text, ocr_text, 0.70, 0.85);

    assert!(!result.fused_text.is_empty());
    assert!(result.fused_text.starts_with("ملخص البحث التنفيذي"));
}

#[test]
fn test_bilingual_code_and_arabic_fusion() {
    let native_text = "استخدام الدالة fn calculate_tax() بنجاح";
    let ocr_text = "استخدام الدالة fn calculate_tax() بنجاح";

    let result = OcrFusionEngine::fuse_character_by_character(native_text, 0.90, ocr_text, 0.90);

    assert_eq!(result.fused_text, native_text);
    assert!(result.fused_text.contains("fn calculate_tax()"));
}
