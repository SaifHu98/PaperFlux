use unicode_normalization::UnicodeNormalization;
use crate::bidi::process_bidi_text;
use crate::hyphenation::clean_soft_hyphens;
use crate::ligatures::unfold_ligatures;

pub struct TextNormalizer;

impl TextNormalizer {
    /// Full normalization pipeline for raw text extracted from PDF content streams.
    pub fn normalize(input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }

        // 1. Remove soft hyphens & zero-width artifacts
        let step1 = clean_soft_hyphens(input);

        // 2. Unfold ligatures (fi, fl, ffi, etc.)
        let step2 = unfold_ligatures(&step1);

        // 3. Process BiDi & RTL reordering/presentation forms if needed
        let step3 = process_bidi_text(&step2);

        // 4. Unicode NFC Normalization (canonical decomposition followed by canonical composition)
        let step4: String = step3.nfc().collect();

        // 5. Clean control characters while preserving essential newlines and tabs
        let mut clean = String::with_capacity(step4.len());
        for ch in step4.chars() {
            if ch == '\n' || ch == '\r' || ch == '\t' || !ch.is_control() {
                clean.push(ch);
            }
        }

        clean
    }

    /// Strips Arabic diacritical marks (Tashkeel / Harakat).
    pub fn strip_arabic_diacritics(input: &str) -> String {
        input
            .chars()
            .filter(|c| !matches!(*c as u32, 0x064B..=0x065F | 0x0670))
            .collect()
    }

    /// Normalizes inline whitespace (collapsing multiple consecutive spaces into a single space).
    pub fn collapse_whitespace(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut in_space = false;
        for ch in input.chars() {
            if ch.is_whitespace() && ch != '\n' {
                if !in_space {
                    result.push(' ');
                    in_space = true;
                }
            } else {
                result.push(ch);
                in_space = false;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization_pipeline() {
        let raw = "The ef\u{FB01}cient pr\u{00AD}ocess";
        let normalized = TextNormalizer::normalize(raw);
        assert_eq!(normalized, "The efficient process");
    }

    #[test]
    fn test_arabic_diacritic_stripping() {
        let raw = "بِسْمِ اللَّهِ";
        let stripped = TextNormalizer::strip_arabic_diacritics(raw);
        assert!(!stripped.contains('\u{064E}'));
    }
}
