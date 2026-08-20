use pdf2md_text::bidi::contains_rtl;
use pdf2md_text::cjk::join_lines_cjk_aware;
use pdf2md_text::language::{Language, LanguageClassifier};
use pdf2md_text::normalizer::TextNormalizer;
use pdf2md_text::quality::TextQualityAssessor;
use pdf2md_text::script::{Script, ScriptDetector};

#[test]
fn test_corpus_arabic_persian_urdu_kurdish() {
    // 1. Standard Arabic with Tashkeel
    let arabic = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ - قَامَ النِّظَامُ بِمُعَالَجَةِ النُّصُوصِ.";
    let (lang, conf) = LanguageClassifier::classify(arabic);
    assert_eq!(lang, Language::Arabic);
    assert!(conf >= 0.90);
    assert!(contains_rtl(arabic));

    // Tashkeel normalization & stripping
    let stripped_ar = TextNormalizer::strip_arabic_diacritics(arabic);
    assert!(!stripped_ar.contains('\u{064E}')); // Fatha stripped

    // 2. Persian
    let persian = "این یک گزارش جامع درباره عملکرد سیستم است که با دقت بالا بررسی شده است.";
    let (lang_fa, _) = LanguageClassifier::classify(persian);
    assert_eq!(lang_fa, Language::Persian);

    // 3. Urdu
    let urdu = "یہ ایک جامع رپورٹ ہے جس میں سسٹم کی کارکردگی کا جائزہ لیا گیا ہے۔";
    let (lang_ur, _) = LanguageClassifier::classify(urdu);
    assert_eq!(lang_ur, Language::Urdu);

    // 4. Kurdish (Sorani)
    let kurdish = "ئەمە ڕاپۆرتێکی گشتگیرە دەربارەی کارکردنی سیستەمەکە.";
    let (lang_ku, _) = LanguageClassifier::classify(kurdish);
    assert_eq!(lang_ku, Language::Kurdish);
}

#[test]
fn test_corpus_hebrew_and_turkish() {
    // Hebrew
    let hebrew = "דוח ביצועי המערכת מציג תוצאות מדויקות עבור כל המסמכים.";
    let (lang_he, _) = LanguageClassifier::classify(hebrew);
    assert_eq!(lang_he, Language::Hebrew);
    assert!(contains_rtl(hebrew));

    // Turkish with special characters
    let turkish = "Gelişmiş metin işleme motoru, Türkçe karakterleri başarıyla ayrıştırır.";
    let (lang_tr, _) = LanguageClassifier::classify(turkish);
    assert_eq!(lang_tr, Language::Turkish);
}

#[test]
fn test_corpus_cjk_languages() {
    // Chinese (Simplified)
    let chinese = "本系统采用先进的神经网络模型进行文档解析与结构化提取。";
    let (lang_zh, _) = LanguageClassifier::classify(chinese);
    assert_eq!(lang_zh, Language::Chinese);
    assert_eq!(ScriptDetector::detect_script(chinese), Script::Cjk);

    // Japanese (with Hiragana/Katakana)
    let japanese = "このドキュメントは、システムアーキテクチャの概要を説明しています。";
    let (lang_ja, _) = LanguageClassifier::classify(japanese);
    assert_eq!(lang_ja, Language::Japanese);

    // CJK-aware line wrapping (no false space insertion)
    let line1 = "このドキュメントは、";
    let line2 = "システムアーキテクチャの概要です。";
    let joined = join_lines_cjk_aware(line1, line2);
    assert_eq!(joined, "このドキュメントは、システムアーキテクチャの概要です。");

    // Korean (Hangul)
    let korean = "이 문서는 고성능 문서 변환 엔진의 아키텍처를 설명합니다.";
    let (lang_ko, _) = LanguageClassifier::classify(korean);
    assert_eq!(lang_ko, Language::Korean);
}

#[test]
fn test_corpus_cyrillic_and_indic() {
    // Russian
    let russian = "Система выполняет высокоточный анализ структуры и верстки документов.";
    let (lang_ru, _) = LanguageClassifier::classify(russian);
    assert_eq!(lang_ru, Language::Russian);

    // Ukrainian (contains 'і', 'ї', 'є', 'ґ')
    let ukrainian = "Ця система забезпечує надійне розпізнавання та конвертацію звітів.";
    let (lang_uk, _) = LanguageClassifier::classify(ukrainian);
    assert_eq!(lang_uk, Language::Ukrainian);

    // Hindi (Devanagari)
    let hindi = "यह प्रणाली बहुभाषी दस्तावेजों का सटीक और कुशल विश्लेषण करती है।";
    let (lang_hi, _) = LanguageClassifier::classify(hindi);
    assert_eq!(lang_hi, Language::Hindi);
    assert_eq!(ScriptDetector::detect_script(hindi), Script::Devanagari);
}

#[test]
fn test_corpus_mathematical_notation() {
    let math = "∀ x ∈ ℝ, ∫ f(x) dx = ∑_{i=1}^n a_i x^i + C where ∇ · E = ρ / ε_0";
    let script = ScriptDetector::detect_script(math);
    assert!(matches!(script, Script::Math | Script::Latin | Script::Mixed));
}

#[test]
fn test_corpus_mixed_script_page() {
    let mixed = "English Specification: تقرير الأداء الفني - システム仕様書";
    let script = ScriptDetector::detect_script(mixed);
    assert_eq!(script, Script::Mixed);

    let distribution = ScriptDetector::detect_script_distribution(mixed);
    assert!(distribution.contains_key(&Script::Latin));
    assert!(distribution.contains_key(&Script::Arabic));
    assert!(distribution.contains_key(&Script::Cjk));
}

#[test]
fn test_corpus_corrupted_font_quality_assessment() {
    // Text with missing glyphs and unmapped PUA fonts
    let corrupted = "Th\u{FFFD}s is a c\u{FFFD}rr\u{FFFD}pted d\u{E001}\u{E002}\u{E003}cument str\u{FFFD}am.";
    let quality = TextQualityAssessor::assess(corrupted);

    assert!(quality.is_corrupted, "Should flag corrupted font stream");
    assert!(quality.replacement_char_count >= 4);
    assert!(quality.pua_char_count >= 3);
    assert!(quality.quality_score < 0.60);
}
