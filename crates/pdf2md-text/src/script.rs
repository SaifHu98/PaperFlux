use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Script {
    Latin,
    Arabic,
    Hebrew,
    Cyrillic,
    Cjk,
    Devanagari,
    Greek,
    Math,
    Mixed,
    Unknown,
}

pub struct ScriptDetector;

impl ScriptDetector {
    pub fn char_script(c: char) -> Script {
        let u = c as u32;

        match u {
            // Latin
            0x0041..=0x005A | 0x0061..=0x007A | 0x00C0..=0x024F | 0x1E00..=0x1EFF => Script::Latin,

            // Arabic (Arabic, Persian, Urdu, Kurdish, Arabic Supplement, Presentation Forms)
            0x0600..=0x06FF
            | 0x0750..=0x077F
            | 0x08A0..=0x08FF
            | 0xFB50..=0xFDFF
            | 0xFE70..=0xFEFF => Script::Arabic,

            // Hebrew
            0x0590..=0x05FF | 0xFB1D..=0xFB4F => Script::Hebrew,

            // Cyrillic
            0x0400..=0x04FF | 0x0500..=0x052F | 0x2DE0..=0x2DFF => Script::Cyrillic,

            // CJK (Hanzi/Kanji, Hiragana, Katakana, Hangul, Bopomofo)
            0x4E00..=0x9FFF
            | 0x3400..=0x4DBF
            | 0x3040..=0x309F
            | 0x30A0..=0x30FF
            | 0xAC00..=0xD7AF
            | 0x1100..=0x11FF => Script::Cjk,

            // Devanagari (Hindi, Sanskrit, Marathi, Nepali)
            0x0900..=0x097F => Script::Devanagari,

            // Greek
            0x0370..=0x03FF | 0x1F00..=0x1FFF => Script::Greek,

            // Mathematical Symbols & Operators
            0x2200..=0x22FF
            | 0x2A00..=0x2AFF
            | 0x2190..=0x21FF
            | 0x27C0..=0x27EF
            | 0x2980..=0x29FF => Script::Math,

            _ => Script::Unknown,
        }
    }

    pub fn detect_script(text: &str) -> Script {
        let counts = Self::detect_script_distribution(text);
        if counts.is_empty() {
            return Script::Unknown;
        }

        let mut dominant_script = Script::Unknown;
        let mut max_ratio = 0.0;
        let mut non_zero_scripts = 0;

        for (script, ratio) in &counts {
            if *ratio > 0.15 {
                non_zero_scripts += 1;
            }
            if *ratio > max_ratio {
                max_ratio = *ratio;
                dominant_script = *script;
            }
        }

        if non_zero_scripts > 1 && max_ratio < 0.70 {
            Script::Mixed
        } else {
            dominant_script
        }
    }

    pub fn detect_script_distribution(text: &str) -> HashMap<Script, f32> {
        let mut counts: HashMap<Script, usize> = HashMap::new();
        let mut total_alphabetic = 0;

        for c in text.chars() {
            if c.is_whitespace() || c.is_ascii_punctuation() || c.is_ascii_digit() {
                continue;
            }
            let script = Self::char_script(c);
            if script != Script::Unknown {
                *counts.entry(script).or_insert(0) += 1;
                total_alphabetic += 1;
            }
        }

        let mut distribution = HashMap::new();
        if total_alphabetic > 0 {
            for (script, count) in counts {
                distribution.insert(script, (count as f32) / (total_alphabetic as f32));
            }
        }

        distribution
    }
}
