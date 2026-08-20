use serde::{Deserialize, Serialize};
use crate::script::{Script, ScriptDetector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Arabic,
    Persian,
    Urdu,
    Pashto,
    Sindhi,
    Kurdish,
    Hebrew,
    Turkish,
    Chinese,
    Japanese,
    Korean,
    Russian,
    Ukrainian,
    Hindi,
    English,
    German,
    French,
    Spanish,
    Unknown,
}

pub struct LanguageClassifier;

impl LanguageClassifier {
    pub fn classify(text: &str) -> (Language, f32) {
        let script = ScriptDetector::detect_script(text);

        match script {
            Script::Arabic => Self::classify_arabic_family(text),
            Script::Hebrew => (Language::Hebrew, 0.98),
            Script::Cjk => Self::classify_cjk_family(text),
            Script::Cyrillic => Self::classify_cyrillic_family(text),
            Script::Devanagari => (Language::Hindi, 0.95),
            Script::Latin => Self::classify_latin_family(text),
            Script::Math | Script::Greek => (Language::Unknown, 0.50),
            Script::Mixed | Script::Unknown => (Language::Unknown, 0.30),
        }
    }

    fn classify_arabic_family(text: &str) -> (Language, f32) {
        // Sindhi specific characters: ٻ, ٿ, ڀ, ٽ, ڄ, ڃ, ڇ, ڌ, ڍ, ڊ, ڙ, ڪ, ڳ, ڱ, ڦ, ڻ, ڏ
        if text.chars().any(|c| matches!(c, 'ٻ' | 'ٿ' | 'ڀ' | 'ٽ' | 'ڄ' | 'ڃ' | 'ڇ' | 'ڌ' | 'ڍ' | 'ڊ' | 'ڙ' | 'ڪ' | 'ڳ' | 'ڱ' | 'ڦ' | 'ڻ' | 'ڏ')) {
            return (Language::Sindhi, 0.96);
        }

        // Pashto specific characters: ټ, ځ, څ, ډ, ړ, ږ, ښ, ګ, ڼ, ې, ۍ, ۀ
        if text.chars().any(|c| matches!(c, 'ټ' | 'ځ' | 'څ' | 'ډ' | 'ړ' | 'ږ' | 'ښ' | 'ګ' | 'ڼ' | 'ې' | 'ۍ' | 'ۀ')) {
            return (Language::Pashto, 0.96);
        }

        // Kurdish Sorani specific characters: ڵ, ۆ, ێ, ڕ, ە
        if text.chars().any(|c| matches!(c, 'ڵ' | 'ۆ' | 'ێ' | 'ڕ' | 'ە')) {
            return (Language::Kurdish, 0.95);
        }

        // Urdu specific characters: ٹ, ڈ, ڑ, ے, ں, ہ, ۂ
        if text.chars().any(|c| matches!(c, 'ٹ' | 'ڈ' | 'ڑ' | 'ے' | 'ں' | 'ہ')) {
            return (Language::Urdu, 0.95);
        }

        // Persian specific characters: گ, چ, پ, ژ
        if text.chars().any(|c| matches!(c, 'گ' | 'چ' | 'پ' | 'ژ')) {
            return (Language::Persian, 0.95);
        }

        (Language::Arabic, 0.92)
    }

    fn classify_cjk_family(text: &str) -> (Language, f32) {
        // Japanese Hiragana: 0x3040..=0x309F or Katakana: 0x30A0..=0x30FF
        if text.chars().any(|c| matches!(c as u32, 0x3040..=0x309F | 0x30A0..=0x30FF)) {
            return (Language::Japanese, 0.98);
        }

        // Korean Hangul: 0xAC00..=0xD7AF | 0x1100..=0x11FF
        if text.chars().any(|c| matches!(c as u32, 0xAC00..=0xD7AF | 0x1100..=0x11FF)) {
            return (Language::Korean, 0.98);
        }

        (Language::Chinese, 0.92)
    }

    fn classify_cyrillic_family(text: &str) -> (Language, f32) {
        // Ukrainian specific: і, ї, є, ґ, І, Ї, Є, Ґ
        if text.chars().any(|c| matches!(c, 'і' | 'ї' | 'є' | 'ґ' | 'І' | 'Ї' | 'Є' | 'Ґ')) {
            return (Language::Ukrainian, 0.95);
        }

        (Language::Russian, 0.90)
    }

    fn classify_latin_family(text: &str) -> (Language, f32) {
        // Turkish: ğ, ı, ş, ç, ö, ü, Ğ, İ, Ş, Ç, Ö, Ü
        if text.chars().any(|c| matches!(c, 'ğ' | 'ı' | 'ş' | 'Ğ' | 'İ' | 'Ş')) {
            return (Language::Turkish, 0.95);
        }

        // German: ä, ö, ü, ß
        if text.chars().any(|c| matches!(c, 'ä' | 'Ä' | 'ß')) {
            return (Language::German, 0.90);
        }

        // French: é, è, ê, ë, à, â, ç, ù, û, ô, î, ï, œ, æ
        if text.chars().any(|c| matches!(c, 'é' | 'è' | 'ê' | 'ë' | 'à' | 'â' | 'ù' | 'û' | 'ô' | 'î' | 'ï' | 'œ' | 'æ')) {
            return (Language::French, 0.90);
        }

        // Spanish: ñ, á, é, í, ó, ú, ¿, ¡
        if text.chars().any(|c| matches!(c, 'ñ' | 'Ñ' | '¿' | '¡')) {
            return (Language::Spanish, 0.90);
        }

        (Language::English, 0.85)
    }
}
