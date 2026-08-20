use crate::language::Language;
use crate::script::Script;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum NumeralSystem {
    /// Preserve original numerals as written in source PDF (Default)
    #[default]
    PreserveAsIs,

    /// Eastern Arabic-Indic: ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩
    EasternArabicIndic,

    /// Western Arabic (European standard): 0 1 2 3 4 5 6 7 8 9
    WesternArabic,

    /// Perso-Arabic / Urdu: ۰ ۱ ۲ ۳ ۴ ۵ ۶ ۷ ۸ ۹
    PersoArabic,

    /// Mixed numerals inside same document
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ArabicShapingMode {
    /// Convert all Presentation Forms A & B back to base Unicode logical characters (Default)
    #[default]
    UnshapeToUnicode,

    /// Preserve glyph shaping as-is
    Preserve,

    /// Normalize presentation forms while preserving ligatures
    NormalizePresentationForms,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ArabicNormalizationMode {
    /// Full NFC Unicode normalization
    #[default]
    StandardNFC,

    /// Strip Harakat / Tashkeel
    TashkeelStripped,

    /// Normalize all Alif variants (أ, إ, آ -> ا)
    AlifNormalized,

    /// Normalize Ta Marbuta (ة -> ه) and Ya (ى -> ي)
    OrthographicNormalized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DiacriticMode {
    /// Preserve all vowel marks (Fatha, Damma, Kasra, Shadda, Sukun, Tanween)
    #[default]
    PreserveHarakat,

    /// Strip short vowels but preserve Shadda (gemination)
    PreserveShaddaOnly,

    /// Strip all diacritical marks
    StripHarakat,

    /// Normalize Quranic / classical annotation marks
    NormalizeQuranic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PunctuationMode {
    /// Use Arabic mirrored punctuation: ؟ (question), ؛ (semicolon), ، (comma), « » (quotes)
    #[default]
    MirroredPunctuation,

    /// Preserve standard ASCII punctuation
    Preserve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BidiMode {
    /// PDF text stream is visual order; reorder to logical standard order
    VisualToLogicalReorder,

    /// PDF text stream is already stored in logical standard order
    LogicalNative,

    /// Auto-detect based on character cluster directions
    #[default]
    AutoDetect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArabicProcessingContext {
    pub script: Script,
    pub language: Language,
    pub direction: pdf2md_ast::geometry::WritingDirection,
    pub numeral_system: NumeralSystem,
    pub shaping_mode: ArabicShapingMode,
    pub normalization_mode: ArabicNormalizationMode,
    pub punctuation_mode: PunctuationMode,
    pub diacritic_mode: DiacriticMode,
    pub bidi_mode: BidiMode,
    pub confidence: f32,
}

impl Default for ArabicProcessingContext {
    fn default() -> Self {
        Self {
            script: Script::Arabic,
            language: Language::Arabic,
            direction: pdf2md_ast::geometry::WritingDirection::RightToLeft,
            numeral_system: NumeralSystem::PreserveAsIs,
            shaping_mode: ArabicShapingMode::UnshapeToUnicode,
            normalization_mode: ArabicNormalizationMode::StandardNFC,
            punctuation_mode: PunctuationMode::MirroredPunctuation,
            diacritic_mode: DiacriticMode::PreserveHarakat,
            bidi_mode: BidiMode::AutoDetect,
            confidence: 0.98,
        }
    }
}

impl ArabicProcessingContext {
    pub fn new_arabic() -> Self {
        Self::default()
    }

    pub fn new_persian() -> Self {
        Self {
            script: Script::Arabic,
            language: Language::Persian,
            numeral_system: NumeralSystem::PersoArabic,
            ..Default::default()
        }
    }

    pub fn new_urdu() -> Self {
        Self {
            script: Script::Arabic,
            language: Language::Urdu,
            numeral_system: NumeralSystem::PersoArabic,
            ..Default::default()
        }
    }

    pub fn new_kurdish() -> Self {
        Self {
            script: Script::Arabic,
            language: Language::Kurdish,
            numeral_system: NumeralSystem::EasternArabicIndic,
            ..Default::default()
        }
    }
}
