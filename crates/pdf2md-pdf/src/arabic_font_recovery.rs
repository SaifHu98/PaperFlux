use std::collections::HashMap;

pub struct AdobeArabicGlyphMap;

impl AdobeArabicGlyphMap {
    /// Maps Adobe Standard / Extended Arabic glyph names to canonical Unicode
    pub fn glyph_name_to_unicode(name: &str) -> Option<String> {
        let clean = name.trim();

        // 1. Unicode pattern: uniXXXX (e.g. uni0627 -> \u{0627}, uniFE8D -> \u{0627})
        if (clean.starts_with("uni") || clean.starts_with("Uni")) && clean.len() == 7 {
            if let Ok(code) = u32::from_str_radix(&clean[3..], 16) {
                return decode_single_or_presentation_form(code);
            }
        }

        // 2. Unicode pattern: uXXXX (e.g. u0627)
        if clean.starts_with('u') && (clean.len() == 5 || clean.len() == 6) {
            if let Ok(code) = u32::from_str_radix(&clean[1..], 16) {
                return decode_single_or_presentation_form(code);
            }
        }

        // 3. AFII Glyph Names (Adobe / Linotype Presentation forms)
        if let Some(ch) = match clean {
            "afii57414" => Some('ا'),
            "afii57415" => Some('ب'),
            "afii57416" => Some('ة'),
            "afii57417" => Some('ت'),
            "afii57418" => Some('ث'),
            "afii57419" => Some('ج'),
            "afii57420" => Some('ح'),
            "afii57421" => Some('خ'),
            "afii57422" => Some('د'),
            "afii57423" => Some('ذ'),
            "afii57424" => Some('ر'),
            "afii57425" => Some('ز'),
            "afii57426" => Some('س'),
            "afii57427" => Some('ش'),
            "afii57428" => Some('ص'),
            "afii57429" => Some('ض'),
            "afii57430" => Some('ط'),
            "afii57431" => Some('ظ'),
            "afii57432" => Some('ع'),
            "afii57433" => Some('غ'),
            "afii57434" => Some('ـ'), // Tatweel
            "afii57435" => Some('ف'),
            "afii57436" => Some('ق'),
            "afii57437" => Some('ك'),
            "afii57438" => Some('ل'),
            "afii57439" => Some('م'),
            "afii57440" => Some('ن'),
            "afii57441" => Some('ه'),
            "afii57442" => Some('و'),
            "afii57443" => Some('ى'),
            "afii57444" => Some('ي'),
            _ => None,
        } {
            return Some(ch.to_string());
        }

        // 4. Standard Descriptive Adobe Glyph Names
        match clean {
            // Basic Arabic Alphabet
            "hamza" => Some("ء".to_string()),
            "alefmadda" | "alefmaddah" => Some("آ".to_string()),
            "alefhamza" | "alefhamzaabove" => Some("أ".to_string()),
            "wawhamza" => Some("ؤ".to_string()),
            "alefhamzabelow" => Some("إ".to_string()),
            "yehhamza" => Some("ئ".to_string()),
            "alef" => Some("ا".to_string()),
            "beh" | "ba" => Some("ب".to_string()),
            "tehmarbuta" => Some("ة".to_string()),
            "teh" | "ta" => Some("ت".to_string()),
            "theh" | "tha" => Some("ث".to_string()),
            "jeem" | "jim" => Some("ج".to_string()),
            "hah" | "ha" => Some("ح".to_string()),
            "khah" | "kha" => Some("خ".to_string()),
            "dal" => Some("د".to_string()),
            "thal" => Some("ذ".to_string()),
            "reh" | "ra" => Some("ر".to_string()),
            "zain" | "zay" => Some("ز".to_string()),
            "seen" | "sin" => Some("س".to_string()),
            "sheen" | "shin" => Some("ش".to_string()),
            "sad" => Some("ص".to_string()),
            "dad" => Some("ض".to_string()),
            "tah" => Some("ط".to_string()),
            "zah" => Some("ظ".to_string()),
            "ain" | "ayn" => Some("ع".to_string()),
            "ghain" | "ghayn" => Some("غ".to_string()),
            "tatweel" | "kashida" => Some("".to_string()),
            "feh" | "fa" => Some("ف".to_string()),
            "qaf" => Some("ق".to_string()),
            "kaf" => Some("ك".to_string()),
            "lam" => Some("ل".to_string()),
            "meem" | "mim" => Some("م".to_string()),
            "noon" | "nun" => Some("ن".to_string()),
            "heh" | "he" => Some("ه".to_string()),
            "waw" => Some("و".to_string()),
            "alefmaksura" => Some("ى".to_string()),
            "yeh" | "ya" => Some("ي".to_string()),

            // Common Ligatures
            "lam_alef" | "lamalef" => Some("لا".to_string()),
            "lam_alefhamza" | "lamalefhamzaabove" => Some("لأ".to_string()),
            "lam_alefhamzabelow" => Some("لإ".to_string()),
            "lam_alefmadda" => Some("لآ".to_string()),
            "allah" => Some("الله".to_string()),
            "sallallahou_alayhe_wasallam" | "saw" => Some("صلى الله عليه وسلم".to_string()),
            "jalajalalouhou" => Some("جل جلاله".to_string()),
            "bismillah" => Some("بسم الله الرحمن الرحيم".to_string()),
            "rial" | "riyal" => Some("ريال".to_string()),

            // Persian / Urdu / Kurdish Extensions
            "peh" => Some("پ".to_string()),
            "tcheh" | "cheh" => Some("چ".to_string()),
            "jeh" | "zheh" => Some("ژ".to_string()),
            "gaf" => Some("گ".to_string()),
            "keheh" => Some("ک".to_string()),
            "tteh" => Some("ٹ".to_string()),
            "ddal" => Some("ڈ".to_string()),
            "rreh" => Some("ڑ".to_string()),
            "noonghunna" => Some("ں".to_string()),
            "yeh_barree" => Some("ے".to_string()),
            "heh_goal" => Some("ہ".to_string()),
            "lam_kurdish" => Some("ڵ".to_string()),
            "oe_kurdish" => Some("ۆ".to_string()),
            "yeh_kurdish" => Some("ێ".to_string()),
            "reh_kurdish" => Some("ڕ".to_string()),
            "e_kurdish" => Some("ە".to_string()),

            // Arabic-Indic Digits
            "zero_arabic" | "uni0660" => Some("٠".to_string()),
            "one_arabic" | "uni0661" => Some("١".to_string()),
            "two_arabic" | "uni0662" => Some("٢".to_string()),
            "three_arabic" | "uni0663" => Some("٣".to_string()),
            "four_arabic" | "uni0664" => Some("٤".to_string()),
            "five_arabic" | "uni0665" => Some("٥".to_string()),
            "six_arabic" | "uni0666" => Some("٦".to_string()),
            "seven_arabic" | "uni0667" => Some("٧".to_string()),
            "eight_arabic" | "uni0668" => Some("٨".to_string()),
            "nine_arabic" | "uni0669" => Some("٩".to_string()),

            // Arabic Punctuation
            "question_arabic" => Some("؟".to_string()),
            "comma_arabic" => Some("،".to_string()),
            "semicolon_arabic" => Some("؛".to_string()),
            "percent_arabic" => Some("٪".to_string()),

            _ => None,
        }
    }
}

fn decode_single_or_presentation_form(code: u32) -> Option<String> {
    // Check if it's already a base Unicode character
    if (0x0600..=0x06FF).contains(&code) || (0x0750..=0x077F).contains(&code) || (0x08A0..=0x08FF).contains(&code) {
        if let Some(ch) = char::from_u32(code) {
            return Some(ch.to_string());
        }
    }

    // Presentation Forms-A and B un-shaping
    if (0xFB50..=0xFDFF).contains(&code) || (0xFE70..=0xFEFF).contains(&code) {
        if let Some(ch) = char::from_u32(code) {
            let s = ch.to_string();
            return Some(pdf2md_text::bidi::normalize_arabic_presentation_forms(&s));
        }
    }

    None
}

pub struct ArabicFontDecoder;

impl ArabicFontDecoder {
    /// Multi-stage recovery resolving code, glyph name, and custom CMap
    pub fn recover_glyph(
        code: u32,
        glyph_name: Option<&str>,
        to_unicode_map: &HashMap<u32, String>,
    ) -> String {
        // Stage 1: Check explicit ToUnicode map
        if let Some(mapped) = to_unicode_map.get(&code) {
            if !mapped.is_empty() && !mapped.chars().all(|c| (0xE000..=0xF8FF).contains(&(c as u32))) {
                return pdf2md_text::bidi::normalize_arabic_presentation_forms(mapped);
            }
        }

        // Stage 2: Check Glyph Name Mapping (AGL + AFII + uniXXXX)
        if let Some(name) = glyph_name {
            if let Some(recovered) = AdobeArabicGlyphMap::glyph_name_to_unicode(name) {
                return recovered;
            }
        }

        // Stage 3: PUA (Private Use Area) Remapping Heuristic for Subsetted Arabic Fonts
        if (0xE000..=0xF8FF).contains(&code) {
            if let Some(pua_mapped) = Self::decode_pua_code(code) {
                return pua_mapped;
            }
        }

        // Stage 4: Direct Unicode or Presentation Form Recovery
        if let Some(recovered) = decode_single_or_presentation_form(code) {
            return recovered;
        }

        // Stage 5: Fallback to standard char if printable ASCII / Latin
        if code < 256 {
            if let Some(ch) = char::from_u32(code) {
                return ch.to_string();
            }
        }

        char::from_u32(code).map(|c| c.to_string()).unwrap_or_default()
    }

    /// PUA heuristic remapping for common Arabic fonts (e.g. Lotus, Traditional Arabic, DecoType)
    pub fn decode_pua_code(code: u32) -> Option<String> {
        // Offset mapping for WinAnsi / PUA Arabic font subsets
        let base_offset = code.wrapping_sub(0xF000);
        if (0x20..=0xFF).contains(&base_offset) {
            // Common Arabic CP1256 offset
            let mapped_char = match base_offset {
                0xC1 => 'ء',
                0xC2 => 'آ',
                0xC3 => 'أ',
                0xC4 => 'ؤ',
                0xC5 => 'إ',
                0xC6 => 'ئ',
                0xC7 => 'ا',
                0xC8 => 'ب',
                0xC9 => 'ة',
                0xCA => 'ت',
                0xCB => 'ث',
                0xCC => 'ج',
                0xCD => 'ح',
                0xCE => 'خ',
                0xCF => 'د',
                0xD0 => 'ذ',
                0xD1 => 'ر',
                0xD2 => 'ز',
                0xD3 => 'س',
                0xD4 => 'ش',
                0xD5 => 'ص',
                0xD6 => 'ض',
                0xD8 => 'ط',
                0xD9 => 'ظ',
                0xDA => 'ع',
                0xDB => 'غ',
                0xDC => 'ـ',
                0xDD => 'ف',
                0xDE => 'ق',
                0xDF => 'ك',
                0xE1 => 'ل',
                0xE3 => 'م',
                0xE4 => 'ن',
                0xE5 => 'ه',
                0xE6 => 'و',
                0xEC => 'ى',
                0xED => 'ي',
                _ => return None,
            };
            return Some(mapped_char.to_string());
        }
        None
    }
}

pub struct ArabicCorruptionDetector;

impl ArabicCorruptionDetector {
    /// Detects if text contains isolated Arabic character corruption (e.g. "ت ق ر ي ر")
    pub fn detect_isolated_glyph_corruption(text: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        let single_arabic_letters = words
            .iter()
            .filter(|w| w.chars().count() == 1 && pdf2md_text::bidi::is_rtl_char(w.chars().next().unwrap()))
            .count();

        single_arabic_letters >= 3 && single_arabic_letters > words.len() / 2
    }

    /// Detects broken Lam-Alef sequences (e.g. "ل ا" or "ل أ")
    pub fn detect_broken_lam_alef(text: &str) -> bool {
        text.contains("ل ا") || text.contains("ل أ") || text.contains("ل إ") || text.contains("ل آ")
    }

    /// Detects Private Use Area codes leaking into output
    pub fn detect_pua_leakage(text: &str) -> bool {
        text.chars().any(|c| (0xE000..=0xF8FF).contains(&(c as u32)))
    }
}
