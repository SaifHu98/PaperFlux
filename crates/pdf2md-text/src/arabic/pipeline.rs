use crate::arabic::context::{ArabicNormalizationMode, ArabicProcessingContext, PunctuationMode};
use crate::arabic::joining::ArabicJoiningReconstructor;
use crate::arabic::numerals::ArabicNumerals;
use crate::arabic::shaping::ArabicShaper;
use crate::bidi::process_bidi_text;
use crate::hyphenation::clean_soft_hyphens;
use unicode_normalization::UnicodeNormalization;

pub struct ArabicTextPipeline;

impl ArabicTextPipeline {
    /// Full Arabic text recovery pipeline
    pub fn process(text: &str, ctx: &ArabicProcessingContext) -> (String, ArabicProcessingContext) {
        if text.is_empty() {
            return (String::new(), ctx.clone());
        }

        // 1. Strip soft hyphens and zero-width artifacts
        let step1 = clean_soft_hyphens(text);

        // 2. Reconnect isolated space-separated character sequences and repair broken Lam-Alef
        let step2 = ArabicJoiningReconstructor::reconstruct_isolated_words(&step1);

        // 3. Attach detached floating diacritics
        let step3 = ArabicJoiningReconstructor::attach_floating_diacritics(&step2);

        // 4. Unshape Presentation Forms A & B and decompose complex ligatures
        let step4 = ArabicShaper::unshape(&step3, ctx.shaping_mode, ctx.diacritic_mode);

        // 5. Orthographic and Alif normalization if requested
        let step5 = match ctx.normalization_mode {
            ArabicNormalizationMode::AlifNormalized => normalize_alif(&step4),
            ArabicNormalizationMode::OrthographicNormalized => {
                normalize_orthography(&normalize_alif(&step4))
            }
            _ => step4,
        };

        // 6. Punctuation mirroring for Arabic flow (respecting numeric commas)
        let step6 = if ctx.punctuation_mode == PunctuationMode::MirroredPunctuation {
            mirror_arabic_punctuation(&step5)
        } else {
            step5
        };

        // 7. Logical BiDi reordering (ensuring embedded Latin terms and numbers remain intact)
        let step7 = process_bidi_text(&step6);

        // 8. Unicode NFC Canonical Composition
        let step8: String = step7.nfc().collect();

        // 9. Update context with detected numerals
        let mut updated_ctx = ctx.clone();
        updated_ctx.numeral_system = ArabicNumerals::detect_numeral_system(&step8);

        (step8, updated_ctx)
    }
}

fn normalize_alif(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'أ' | 'إ' | 'آ' | 'ٱ' => 'ا',
            _ => c,
        })
        .collect()
}

fn normalize_orthography(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'ة' => 'ه',
            'ى' => 'ي',
            'ئ' => 'ي',
            _ => c,
        })
        .collect()
}

fn mirror_arabic_punctuation(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut in_quote = false;

    for i in 0..chars.len() {
        let c = chars[i];
        match c {
            '?' => out.push('؟'),
            ';' => out.push('؛'),
            ',' => {
                // If comma is between digits (e.g. 15,450), preserve as numeric comma
                let is_numeric = i > 0
                    && i + 1 < chars.len()
                    && chars[i - 1].is_ascii_digit()
                    && chars[i + 1].is_ascii_digit();
                if is_numeric {
                    out.push(',');
                } else {
                    out.push('،');
                }
            }
            '"' => {
                if !in_quote {
                    out.push('«');
                    in_quote = true;
                } else {
                    out.push('»');
                    in_quote = false;
                }
            }
            _ => out.push(c),
        }
    }

    out
}
