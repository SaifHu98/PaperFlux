use std::collections::HashMap;
use pdf2md_pdf::arabic_font_recovery::{
    AdobeArabicGlyphMap, ArabicCorruptionDetector, ArabicFontDecoder,
};
use pdf2md_text::arabic::joining::ArabicJoiningReconstructor;

#[test]
fn test_adobe_arabic_glyph_names_and_afii_recovery() {
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("afii57414").unwrap(), "ا");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("afii57415").unwrap(), "ب");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("afii57442").unwrap(), "و");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("lam_alef").unwrap(), "لا");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("allah").unwrap(), "الله");
    assert_eq!(
        AdobeArabicGlyphMap::glyph_name_to_unicode("sallallahou_alayhe_wasallam").unwrap(),
        "صلى الله عليه وسلم"
    );
    assert_eq!(
        AdobeArabicGlyphMap::glyph_name_to_unicode("bismillah").unwrap(),
        "بسم الله الرحمن الرحيم"
    );
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("rial").unwrap(), "ريال");
}

#[test]
fn test_uni_and_hex_pattern_recovery() {
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("uni0627").unwrap(), "ا");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("uni0628").unwrap(), "ب");
    // uniFE8D is Alef isolated presentation form -> unshaped to \u0627
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("uniFE8D").unwrap(), "ا");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("u0645").unwrap(), "م");
}

#[test]
fn test_pua_private_use_area_code_remapping() {
    // 0xF0C7 -> 'ا', 0xF0C8 -> 'ب'
    assert_eq!(ArabicFontDecoder::decode_pua_code(0xF0C7).unwrap(), "ا");
    assert_eq!(ArabicFontDecoder::decode_pua_code(0xF0C8).unwrap(), "ب");
    assert_eq!(ArabicFontDecoder::decode_pua_code(0xF0ED).unwrap(), "ي");
}

#[test]
fn test_multistage_glyph_recovery_pipeline() {
    let mut to_unicode = HashMap::new();
    to_unicode.insert(101, "\u{FE8D}".to_string()); // Maps to Presentation form
    to_unicode.insert(102, "\u{E050}".to_string()); // PUA leak

    // Stage 1: ToUnicode unshapes presentation form
    let res1 = ArabicFontDecoder::recover_glyph(101, None, &to_unicode);
    assert_eq!(res1, "ا");

    // Stage 2: Falls back to glyph name when ToUnicode is PUA
    let res2 = ArabicFontDecoder::recover_glyph(102, Some("beh"), &to_unicode);
    assert_eq!(res2, "ب");

    // Stage 3: PUA code resolution
    let res3 = ArabicFontDecoder::recover_glyph(0xF0C7, None, &HashMap::new());
    assert_eq!(res3, "ا");
}

#[test]
fn test_arabic_joining_and_isolated_letter_reconstruction() {
    let broken_text = "ت ق ر ي ر  ح و ل  ج ا م ع ة  د م ش ق";
    let repaired = ArabicJoiningReconstructor::reconstruct_isolated_words(broken_text);
    assert_eq!(repaired, "تقرير حول جامعة دمشق");
}

#[test]
fn test_broken_lam_alef_repair() {
    let broken = "المملكة العربية السعودية ل ا تنسى فضل العلم و ل أصحاب الهمم";
    let repaired = ArabicJoiningReconstructor::repair_broken_ligatures(broken);
    assert!(repaired.contains("لا تنسى"));
    assert!(repaired.contains("لأصحاب"));
}

#[test]
fn test_arabic_corruption_detector() {
    assert!(ArabicCorruptionDetector::detect_isolated_glyph_corruption("ت ق ر ي ر"));
    assert!(!ArabicCorruptionDetector::detect_isolated_glyph_corruption("تقرير كامل عن النظام"));

    assert!(ArabicCorruptionDetector::detect_broken_lam_alef("ل ا يجوز ذلك"));
    assert!(!ArabicCorruptionDetector::detect_broken_lam_alef("لا يجوز ذلك"));

    assert!(ArabicCorruptionDetector::detect_pua_leakage("نص يحتوي على رمز \u{E001} خاص"));
    assert!(!ArabicCorruptionDetector::detect_pua_leakage("نص عربي سليم"));
}

#[test]
fn test_persian_urdu_kurdish_glyph_names() {
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("peh").unwrap(), "پ");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("tcheh").unwrap(), "چ");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("jeh").unwrap(), "ژ");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("gaf").unwrap(), "گ");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("tteh").unwrap(), "ٹ");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("lam_kurdish").unwrap(), "ڵ");
    assert_eq!(AdobeArabicGlyphMap::glyph_name_to_unicode("oe_kurdish").unwrap(), "ۆ");
}
