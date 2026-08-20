use serde::{Deserialize, Serialize};
use crate::language::Language;
use crate::script::Script;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NumeralSystem {
    /// Preserve original numerals as written in source PDF (Default)
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

impl Default for NumeralSystem {
    fn default() -> Self {
        Self::PreserveAsIs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArabicShapingMode {
    /// Convert all Presentation Forms A & B back to base Unicode logical characters (Default)
    UnshapeToUnicode,

    /// Preserve glyph shaping as-is
    Preserve,

    /// Normalize presentation forms while preserving ligatures
    NormalizePresentationForms,
}

impl Default for ArabicShapingMode {
    fn default() -> Self {
        Self::UnshapeToUnicode
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArabicNormalizationMode {
    /// Full NFC Unicode normalization
    StandardNFC,

    /// Strip Harakat / Tashkeel
    TashkeelStripped,

    /// Normalize all Alif variants (أ, إ, آ -> ا)
    AlifNormalized,

    /// Normalize Ta Marbuta (ة -> ه) and Ya (ى -> ي)
    OrthographicNormalized,
}

impl Default for ArabicNormalizationMode {
    fn default() -> Self {
        Self::StandardNFC
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiacriticMode {
    /// Preserve all vowel marks (Fatha, Damma, Kasra, Shadda, Sukun, Tanween)
    PreserveHarakat,

    /// Strip short vowels but preserve Shadda (gemination)
    PreserveShaddaOnly,

    /// Strip all diacritical marks
    StripHarakat,

    /// Normalize Quranic / classical annotation marks
    NormalizeQuranic,
}

impl Default for DiacriticMode {
    fn default() -> Self {
        Self::PreserveHarakat
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PunctuationMode {
    /// Use Arabic mirrored punctuation: ؟ (question), ؛ (semicolon), ، (comma), « » (quotes)
    MirroredPunctuation,

    /// Preserve standard ASCII punctuation
    Preserve,
}

impl Default for PunctuationMode {
    fn default() -> Self {
        Self::MirroredPunctuation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BidiMode {
    /// PDF text stream is visual order; reorder to logical standard order
    VisualToLogicalReorder,

    /// PDF text stream is already stored in logical standard order
    LogicalNative,

    /// Auto-detect based on character cluster directions
    AutoDetect,
}

impl Default for BidiMode {
    fn default() -> Self {
        Self::AutoDetect
    }
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
