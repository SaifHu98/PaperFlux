use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct FontMap {
    pub name: String,
    pub base_font: String,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_monospace: bool,
    pub to_unicode: HashMap<u32, String>,
    pub widths: HashMap<u32, f32>,
    pub default_width: f32,
    glyph_cache: Arc<RwLock<HashMap<u32, String>>>,
}

impl FontMap {
    pub fn new(name: String, base_font: String) -> Self {
        let lower = base_font.to_lowercase();
        let is_bold = lower.contains("bold")
            || lower.contains("black")
            || lower.contains("heavy")
            || lower.contains("b-");
        let is_italic =
            lower.contains("italic") || lower.contains("oblique") || lower.contains("it-");
        let is_monospace = lower.contains("courier")
            || lower.contains("mono")
            || lower.contains("consolas")
            || lower.contains("code");

        Self {
            name,
            base_font,
            is_bold,
            is_italic,
            is_monospace,
            to_unicode: HashMap::new(),
            widths: HashMap::new(),
            default_width: 500.0,
            glyph_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Decodes a glyph code or byte into a UTF-8 character string using multi-stage Arabic font recovery with caching.
    pub fn decode_code(&self, code: u32) -> String {
        if let Ok(cache) = self.glyph_cache.read() {
            if let Some(decoded) = cache.get(&code) {
                return decoded.clone();
            }
        }

        let decoded = crate::arabic_font_recovery::ArabicFontDecoder::recover_glyph(
            code,
            None,
            &self.to_unicode,
        );

        if let Ok(mut cache) = self.glyph_cache.write() {
            cache.insert(code, decoded.clone());
        }

        decoded
    }

    /// Parses a ToUnicode CMap stream content into this font map.
    pub fn parse_to_unicode_cmap(&mut self, cmap_data: &str) {
        let lines: Vec<&str> = cmap_data.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            if line.ends_with("beginbfchar") {
                let count = line
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                i += 1;
                let mut processed = 0;
                while i < lines.len()
                    && !lines[i].trim().ends_with("endbfchar")
                    && (count == 0 || processed < count)
                {
                    let parts: Vec<&str> = lines[i].split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let (Some(code), Some(uni)) =
                            (parse_hex_code(parts[0]), parse_hex_unicode(parts[1]))
                        {
                            self.to_unicode.insert(code, uni);
                            processed += 1;
                        }
                    }
                    i += 1;
                }
            } else if line.ends_with("beginbfrange") {
                let count = line
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                i += 1;
                let mut processed = 0;
                while i < lines.len()
                    && !lines[i].trim().ends_with("endbfrange")
                    && (count == 0 || processed < count)
                {
                    let parts: Vec<&str> = lines[i].split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let (Some(start_code), Some(end_code)) =
                            (parse_hex_code(parts[0]), parse_hex_code(parts[1]))
                        {
                            if parts[2].starts_with('<') {
                                if let Some(start_uni_code) = parse_hex_code(parts[2]) {
                                    for offset in 0..=(end_code.saturating_sub(start_code)) {
                                        if let Some(ch) = char::from_u32(start_uni_code + offset) {
                                            self.to_unicode
                                                .insert(start_code + offset, ch.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        processed += 1;
                    }
                    i += 1;
                }
            }
            i += 1;
        }
    }
}

fn parse_hex_code(hex_str: &str) -> Option<u32> {
    let clean = hex_str.trim().trim_start_matches('<').trim_end_matches('>');
    u32::from_str_radix(clean, 16).ok()
}

fn parse_hex_unicode(hex_str: &str) -> Option<String> {
    let clean = hex_str.trim().trim_start_matches('<').trim_end_matches('>');
    if clean.len().is_multiple_of(4) {
        let mut s = String::new();
        for chunk in clean.as_bytes().chunks(4) {
            if let Ok(chunk_str) = std::str::from_utf8(chunk) {
                if let Ok(code) = u32::from_str_radix(chunk_str, 16) {
                    if let Some(ch) = char::from_u32(code) {
                        s.push(ch);
                    }
                }
            }
        }
        if !s.is_empty() {
            return Some(s);
        }
    }

    parse_hex_code(hex_str)
        .and_then(char::from_u32)
        .map(|c| c.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_properties() {
        let font = FontMap::new("F1".to_string(), "Helvetica-Bold".to_string());
        assert!(font.is_bold);
        assert!(!font.is_italic);

        let code_font = FontMap::new("F2".to_string(), "CourierNewPSMT".to_string());
        assert!(code_font.is_monospace);
    }

    #[test]
    fn test_cmap_parsing() {
        let mut font = FontMap::new("F1".to_string(), "CustomFont".to_string());
        let cmap = r#"
        1 beginbfchar
        <0001> <0041>
        endbfchar
        1 beginbfrange
        <0002> <0004> <0042>
        endbfrange
        "#;
        font.parse_to_unicode_cmap(cmap);
        assert_eq!(font.decode_code(1), "A");
        assert_eq!(font.decode_code(2), "B");
        assert_eq!(font.decode_code(3), "C");
        assert_eq!(font.decode_code(4), "D");
    }
}
