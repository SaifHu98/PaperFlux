use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextQualityScore {
    pub quality_score: f32,
    pub is_corrupted: bool,
    pub printable_ratio: f32,
    pub replacement_char_count: usize,
    pub pua_char_count: usize,
    pub reasons: Vec<String>,
}

pub struct TextQualityAssessor;

impl TextQualityAssessor {
    pub fn assess(text: &str) -> TextQualityScore {
        if text.trim().is_empty() {
            return TextQualityScore {
                quality_score: 0.0,
                is_corrupted: false,
                printable_ratio: 1.0,
                replacement_char_count: 0,
                pua_char_count: 0,
                reasons: vec!["Empty text".into()],
            };
        }

        let total_chars = text.chars().count();
        let mut printable_chars = 0;
        let mut replacement_char_count = 0;
        let mut pua_char_count = 0;
        let mut control_chars = 0;

        for c in text.chars() {
            if c == '\u{FFFD}' {
                replacement_char_count += 1;
            } else if matches!(c as u32, 0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD)
            {
                pua_char_count += 1;
            } else if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                control_chars += 1;
            } else if !c.is_control() {
                printable_chars += 1;
            }
        }

        let printable_ratio = (printable_chars as f32) / (total_chars as f32);
        let mut reasons = Vec::new();
        let mut is_corrupted = false;

        if replacement_char_count > 0 {
            reasons.push(format!(
                "Found {} Unicode replacement characters (\\uFFFD)",
                replacement_char_count
            ));
            if (replacement_char_count as f32) / (total_chars as f32) > 0.05 {
                is_corrupted = true;
            }
        }

        if pua_char_count > 0 {
            reasons.push(format!(
                "Found {} Private Use Area characters (unmapped font glyphs)",
                pua_char_count
            ));
            if (pua_char_count as f32) / (total_chars as f32) > 0.10 {
                is_corrupted = true;
            }
        }

        if printable_ratio < 0.70 {
            reasons.push(format!(
                "Low printable character ratio ({:.1}%)",
                printable_ratio * 100.0
            ));
            is_corrupted = true;
        }

        let penalty = (replacement_char_count as f32 * 0.05)
            + (pua_char_count as f32 * 0.02)
            + (control_chars as f32 * 0.05);
        let quality_score = (printable_ratio - penalty).clamp(0.0, 1.0);

        TextQualityScore {
            quality_score,
            is_corrupted,
            printable_ratio,
            replacement_char_count,
            pua_char_count,
            reasons,
        }
    }
}
