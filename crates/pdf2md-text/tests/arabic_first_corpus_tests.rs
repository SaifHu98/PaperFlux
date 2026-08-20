use pdf2md_text::arabic::context::{
    ArabicNormalizationMode, ArabicProcessingContext, ArabicShapingMode, DiacriticMode,
    NumeralSystem, PunctuationMode,
};
use pdf2md_text::arabic::numerals::ArabicNumerals;
use pdf2md_text::arabic::pipeline::ArabicTextPipeline;

#[test]
fn test_arabic_classical_tashkeel_and_diacritic_modes() {
    let classical = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ - قُلْ أَعُوذُ بِرَبِّ النَّاسِ";

    // 1. Preserve Harakat
    let ctx_preserve = ArabicProcessingContext {
        diacritic_mode: DiacriticMode::PreserveHarakat,
        ..Default::default()
    };
    let (processed_pres, _) = ArabicTextPipeline::process(classical, &ctx_preserve);
    assert!(processed_pres.contains('\u{0650}')); // Contains Kasra

    // 2. Strip Harakat
    let ctx_strip = ArabicProcessingContext {
        diacritic_mode: DiacriticMode::StripHarakat,
        ..Default::default()
    };
    let (processed_strip, _) = ArabicTextPipeline::process(classical, &ctx_strip);
    assert!(!processed_strip.contains('\u{064E}')); // No Fatha
    assert!(!processed_strip.contains('\u{0650}')); // No Kasra
    assert!(processed_strip.contains("بسم الله"));

    // 3. Preserve Shadda only
    let ctx_shadda = ArabicProcessingContext {
        diacritic_mode: DiacriticMode::PreserveShaddaOnly,
        ..Default::default()
    };
    let (processed_shadda, _) = ArabicTextPipeline::process(classical, &ctx_shadda);
    assert!(processed_shadda.contains('\u{0651}')); // Shadda preserved
    assert!(!processed_shadda.contains('\u{064E}')); // Fatha stripped
}

#[test]
fn test_arabic_presentation_forms_and_honorific_ligatures() {
    // String containing presentation forms B and honorific ligatures
    // \uFDFA = ﷺ, \uFDFD = ﷽, \uFDFC = ﷼, \uFE8D\uFEDF\uFEE7\uFE91\uFEED\uFE93 = النبوة
    let input = "\u{FDFD} - قال النبي \u{FDFA}: المبلغ هو 500 \u{FDFC}. \u{FE8D}\u{FEDF}\u{FEE7}\u{FE91}\u{FEED}\u{FE93}";
    let ctx = ArabicProcessingContext::default();

    let (recovered, _) = ArabicTextPipeline::process(input, &ctx);

    assert!(recovered.contains("بسم الله الرحمن الرحيم"));
    assert!(recovered.contains("صلى الله عليه وسلم"));
    assert!(recovered.contains("ريال"));
    assert!(recovered.contains("النبوة"));
}

#[test]
fn test_arabic_numeral_systems_and_separators() {
    let eastern = "تم تسجيل ١٥٬٤٥٠ طالب بنسبة نجاح ٩٨٫٥٪ في عام ٢٠٢٦";

    // 1. Detect numeral system
    let system = ArabicNumerals::detect_numeral_system(eastern);
    assert_eq!(system, NumeralSystem::EasternArabicIndic);

    // 2. Convert to Western numerals
    let western = ArabicNumerals::to_western(eastern);
    assert!(western.contains("15,450"));
    assert!(western.contains("98.5%"));
    assert!(western.contains("2026"));

    // 3. Convert back to Eastern Arabic-Indic
    let back_to_eastern = ArabicNumerals::to_eastern_indic(&western);
    assert!(back_to_eastern.contains("١٥٬٤٥٠"));
    assert!(back_to_eastern.contains("٩٨٫٥٪"));
}

#[test]
fn test_arabic_persian_urdu_kurdish_overlap() {
    // Persian text with 'گ چ پ ژ'
    let persian = "گزارش عملکرد پایگاه داده با چهارچوب چابک و پروژه به روز.";
    let ctx_fa = ArabicProcessingContext::new_persian();
    let (processed_fa, _) = ArabicTextPipeline::process(persian, &ctx_fa);
    assert!(processed_fa.contains('گ'));
    assert!(processed_fa.contains('چ'));
    assert!(processed_fa.contains('پ'));

    // Urdu text with 'ٹ ڈ ڑ ے ں'
    let urdu = "یہ دستاویز اردو زبان میں لکھی گئی ہے جس میں بڑے ڈیٹا کا تجزیہ کیا گیا ہے۔";
    let ctx_ur = ArabicProcessingContext::new_urdu();
    let (processed_ur, _) = ArabicTextPipeline::process(urdu, &ctx_ur);
    assert!(processed_ur.contains('ٹ'));
    assert!(processed_ur.contains('ڑ'));
    assert!(processed_ur.contains('ے'));

    // Kurdish Sorani text with 'ڵ ۆ ێ ڕ ە'
    let kurdish = "ئەم ڕاپۆرتە دەربارەی کۆمەڵگەی زیرەک و فێرکاری بە زمانی کوردی نووسراوە.";
    let ctx_ku = ArabicProcessingContext::new_kurdish();
    let (processed_ku, _) = ArabicTextPipeline::process(kurdish, &ctx_ku);
    assert!(processed_ku.contains('ڵ'));
    assert!(processed_ku.contains('ۆ'));
    assert!(processed_ku.contains('ێ'));
    assert!(processed_ku.contains('ڕ'));
}

#[test]
fn test_arabic_punctuation_mirroring() {
    let input = "هل يعمل المحرك بكفاءة؟ نعم، بالتأكيد؛ لقد تم اختباره: \"نتائج ممتازة\"";
    let ctx = ArabicProcessingContext {
        punctuation_mode: PunctuationMode::MirroredPunctuation,
        ..Default::default()
    };

    let (processed, _) = ArabicTextPipeline::process(input, &ctx);
    assert!(processed.contains('؟'));
    assert!(processed.contains('؛'));
    assert!(processed.contains('،'));
    assert!(processed.contains('«'));
    assert!(processed.contains('»'));
}

#[test]
fn test_mixed_arabic_english_inline_coexistence() {
    let mixed = "تم تطوير مكتبة PaperFlux باستخدام لغة Rust لدعم بروتوكول OAuth 2.0 والتحويل إلى Markdown.";
    let ctx = ArabicProcessingContext::default();

    let (processed, _) = ArabicTextPipeline::process(mixed, &ctx);

    assert!(processed.contains("PaperFlux"));
    assert!(processed.contains("Rust"));
    assert!(processed.contains("OAuth 2.0"));
    assert!(processed.contains("Markdown"));
}
