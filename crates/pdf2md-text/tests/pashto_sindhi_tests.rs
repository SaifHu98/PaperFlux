use pdf2md_text::arabic::context::{ArabicShapingMode, DiacriticMode};
use pdf2md_text::arabic::shaping::ArabicShaper;
use pdf2md_text::arabic::bidi_engine::{BidiTokenizer, BidiTokenKind};
use pdf2md_text::language::{Language, LanguageClassifier};

#[test]
fn test_pashto_language_detection() {
    let pashto_sample1 = "دا یو پښتو متن دی چې د افغانستان او پښتونخوا د خلکو ژبه ده.";
    let (lang1, conf1) = LanguageClassifier::classify(pashto_sample1);
    assert_eq!(lang1, Language::Pashto);
    assert!(conf1 >= 0.90);

    let pashto_sample2 = "په کتاب کې د ژبې او ادبياتو په هکله ډېر مالومات شته دي.";
    let (lang2, conf2) = LanguageClassifier::classify(pashto_sample2);
    assert_eq!(lang2, Language::Pashto);
    assert!(conf2 >= 0.90);
}

#[test]
fn test_sindhi_language_detection() {
    let sindhi_sample1 = "سنڌي ٻولي هڪ قديم ۽ خوبصورت ٻولي آهي جيڪا سنڌ ۾ ڳالهائي وڃي ٿي.";
    let (lang1, conf1) = LanguageClassifier::classify(sindhi_sample1);
    assert_eq!(lang1, Language::Sindhi);
    assert!(conf1 >= 0.90);

    let sindhi_sample2 = "هن دستاويز ۾ سنڌي تعليم ۽ ادبي ترقيءَ جو ذڪر آهي.";
    let (lang2, conf2) = LanguageClassifier::classify(sindhi_sample2);
    assert_eq!(lang2, Language::Sindhi);
    assert!(conf2 >= 0.90);
}

#[test]
fn test_pashto_presentation_forms_unshaping() {
    // String with Pashto presentation forms A:
    // \u{FB62} (ټ), \u{FB6E} (ځ), \u{FB76} (څ), \u{FB9A} (ګ), \u{FBE4} (ې), \u{FBFC} (ۍ)
    let input = "\u{FB62}\u{FB6E}\u{FB76}\u{FB9A}\u{FBE4}\u{FBFC}";
    let unshaped = ArabicShaper::unshape(input, ArabicShapingMode::UnshapeToUnicode, DiacriticMode::PreserveHarakat);

    assert_eq!(unshaped, "ټځڅګېۍ");
}

#[test]
fn test_sindhi_presentation_forms_unshaping() {
    // String with Sindhi presentation forms A:
    // \u{FB52} (ٻ), \u{FB5A} (ڀ), \u{FB5E} (ٽ), \u{FB72} (ڄ), \u{FB96} (ڦ), \u{FB9E} (ڪ), \u{FBA2} (ڳ)
    let input = "\u{FB52}\u{FB5A}\u{FB5E}\u{FB72}\u{FB96}\u{FB9E}\u{FBA2}";
    let unshaped = ArabicShaper::unshape(input, ArabicShapingMode::UnshapeToUnicode, DiacriticMode::PreserveHarakat);

    assert_eq!(unshaped, "ٻڀٽڄڦڪڳ");
}

#[test]
fn test_pashto_sindhi_bidi_tokenization() {
    let mixed = "د پښتو او سنڌي ملاتړ په PaperFlux 2.0 کې بشپړ شو";
    let tokens = BidiTokenizer::tokenize(mixed);

    assert!(!tokens.is_empty());
    let brand_token = tokens.iter().find(|t| t.text == "PaperFlux");
    assert!(brand_token.is_some());
    assert_eq!(brand_token.unwrap().kind, BidiTokenKind::LatinText);

    let pashto_token = tokens.iter().find(|t| t.text.contains("پښتو"));
    assert!(pashto_token.is_some());
    assert!(pashto_token.unwrap().is_rtl);
}
