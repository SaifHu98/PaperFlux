pub fn is_rtl_char(ch: char) -> bool {
    matches!(ch,
        '\u{0590}'..='\u{05FF}' // Hebrew
        | '\u{0600}'..='\u{06FF}' // Arabic
        | '\u{0750}'..='\u{077F}' // Arabic Supplement
        | '\u{08A0}'..='\u{08FF}' // Arabic Extended-A
        | '\u{FB50}'..='\u{FDFF}' // Arabic Presentation Forms-A
        | '\u{FE70}'..='\u{FEFF}' // Arabic Presentation Forms-B
        | '\u{0700}'..='\u{074F}' // Syriac
        | '\u{0780}'..='\u{07BF}' // Thaana
        | '\u{0840}'..='\u{085F}' // Mandaic
    )
}

pub fn contains_rtl(text: &str) -> bool {
    text.chars().any(is_rtl_char)
}

pub fn is_final_presentation_form(ch: char) -> bool {
    matches!(
        ch as u32,
        0xFE8E
            | 0xFE90
            | 0xFE94
            | 0xFE96
            | 0xFE9A
            | 0xFE9E
            | 0xFEA2
            | 0xFEA6
            | 0xFEAA
            | 0xFEAC
            | 0xFEAE
            | 0xFEB0
            | 0xFEB2
            | 0xFEB6
            | 0xFEBA
            | 0xFEBE
            | 0xFEC2
            | 0xFEC6
            | 0xFECA
            | 0xFECE
            | 0xFED2
            | 0xFED6
            | 0xFEDA
            | 0xFEDE
            | 0xFEE2
            | 0xFEE6
            | 0xFEEA
            | 0xFEEE
            | 0xFEF0
            | 0xFEF2
    )
}

pub fn is_initial_presentation_form(ch: char) -> bool {
    matches!(
        ch as u32,
        0xFE8B
            | 0xFE91
            | 0xFE97
            | 0xFE9B
            | 0xFE9F
            | 0xFEA3
            | 0xFEA7
            | 0xFEB3
            | 0xFEB7
            | 0xFEBB
            | 0xFEBF
            | 0xFEC3
            | 0xFEC7
            | 0xFECB
            | 0xFECF
            | 0xFED3
            | 0xFED7
            | 0xFEDB
            | 0xFEDF
            | 0xFEE3
            | 0xFEE7
            | 0xFEEB
            | 0xFEF3
    )
}

/// Normalizes Arabic Presentation Forms (A & B) back to standard base Arabic Unicode characters.
pub fn normalize_arabic_presentation_forms(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let normalized = match ch {
            // Presentation Forms-B (common Arabic letters in isolated/initial/medial/final forms)
            '\u{FE80}' => '\u{0621}',              // HAMZA
            '\u{FE81}' | '\u{FE82}' => '\u{0622}', // ALEF WITH MADDA ABOVE
            '\u{FE83}' | '\u{FE84}' => '\u{0623}', // ALEF WITH HAMZA ABOVE
            '\u{FE85}' | '\u{FE86}' => '\u{0624}', // WAW WITH HAMZA ABOVE
            '\u{FE87}' | '\u{FE88}' => '\u{0625}', // ALEF WITH HAMZA BELOW
            '\u{FE89}' | '\u{FE8A}' | '\u{FE8B}' | '\u{FE8C}' => '\u{0626}', // YEH WITH HAMZA ABOVE
            '\u{FE8D}' | '\u{FE8E}' => '\u{0627}', // ALEF
            '\u{FE8F}' | '\u{FE90}' | '\u{FE91}' | '\u{FE92}' => '\u{0628}', // BEH
            '\u{FE93}' | '\u{FE94}' => '\u{0629}', // TEH MARBUTA
            '\u{FE95}' | '\u{FE96}' | '\u{FE97}' | '\u{FE98}' => '\u{062A}', // TEH
            '\u{FE99}' | '\u{FE9A}' | '\u{FE9B}' | '\u{FE9C}' => '\u{062B}', // THEH
            '\u{FE9D}' | '\u{FE9E}' | '\u{FE9F}' | '\u{FEA0}' => '\u{062C}', // JEEM
            '\u{FEA1}' | '\u{FEA2}' | '\u{FEA3}' | '\u{FEA4}' => '\u{062D}', // HAH
            '\u{FEA5}' | '\u{FEA6}' | '\u{FEA7}' | '\u{FEA8}' => '\u{062E}', // KHAH
            '\u{FEA9}' | '\u{FEAA}' => '\u{062F}', // DAL
            '\u{FEAB}' | '\u{FEAC}' => '\u{0630}', // THAL
            '\u{FEAD}' | '\u{FEAE}' => '\u{0631}', // REH
            '\u{FEAF}' | '\u{FEB0}' => '\u{0632}', // ZAIN
            '\u{FEB1}' | '\u{FEB2}' | '\u{FEB3}' | '\u{FEB4}' => '\u{0633}', // SEEN
            '\u{FEB5}' | '\u{FEB6}' | '\u{FEB7}' | '\u{FEB8}' => '\u{0634}', // SHEEN
            '\u{FEB9}' | '\u{FEBA}' | '\u{FEBB}' | '\u{FEBC}' => '\u{0635}', // SAD
            '\u{FEBD}' | '\u{FEBE}' | '\u{FEBF}' | '\u{FEC0}' => '\u{0636}', // DAD
            '\u{FEC1}' | '\u{FEC2}' | '\u{FEC3}' | '\u{FEC4}' => '\u{0637}', // TAH
            '\u{FEC5}' | '\u{FEC6}' | '\u{FEC7}' | '\u{FEC8}' => '\u{0638}', // ZAH
            '\u{FEC9}' | '\u{FECA}' | '\u{FECB}' | '\u{FECC}' => '\u{0639}', // AIN
            '\u{FECD}' | '\u{FECE}' | '\u{FECF}' | '\u{FED0}' => '\u{063A}', // GHAIN
            '\u{FED1}' | '\u{FED2}' | '\u{FED3}' | '\u{FED4}' => '\u{0641}', // FEH
            '\u{FED5}' | '\u{FED6}' | '\u{FED7}' | '\u{FED8}' => '\u{0642}', // QAF
            '\u{FED9}' | '\u{FEDA}' | '\u{FEDB}' | '\u{FEDC}' => '\u{0643}', // KAF
            '\u{FEDD}' | '\u{FEDE}' | '\u{FEDF}' | '\u{FEE0}' => '\u{0644}', // LAM
            '\u{FEE1}' | '\u{FEE2}' | '\u{FEE3}' | '\u{FEE4}' => '\u{0645}', // MEEM
            '\u{FEE5}' | '\u{FEE6}' | '\u{FEE7}' | '\u{FEE8}' => '\u{0646}', // NOON
            '\u{FEE9}' | '\u{FEEA}' | '\u{FEEB}' | '\u{FEEC}' => '\u{0647}', // HEH
            '\u{FEED}' | '\u{FEEE}' => '\u{0648}', // WAW
            '\u{FEEF}' | '\u{FEF0}' => '\u{0649}', // ALEF MAKSURA
            '\u{FEF1}' | '\u{FEF2}' | '\u{FEF3}' | '\u{FEF4}' => '\u{064A}', // YEH
            // Lam-Alef ligatures
            '\u{FEF5}' | '\u{FEF6}' => {
                out.push('\u{0644}');
                out.push('\u{0622}');
                continue;
            }
            '\u{FEF7}' | '\u{FEF8}' => {
                out.push('\u{0644}');
                out.push('\u{0623}');
                continue;
            }
            '\u{FEF9}' | '\u{FEFA}' => {
                out.push('\u{0644}');
                out.push('\u{0625}');
                continue;
            }
            '\u{FEFB}' | '\u{FEFC}' => {
                out.push('\u{0644}');
                out.push('\u{0627}');
                continue;
            }
            // Persian / Urdu additions
            '\u{FB56}' | '\u{FB57}' | '\u{FB58}' | '\u{FB59}' => '\u{067E}', // PEH
            '\u{FB7A}' | '\u{FB7B}' | '\u{FB7C}' | '\u{FB7D}' => '\u{0686}', // TCHEH
            '\u{FB8A}' | '\u{FB8B}' => '\u{0698}',                           // JEH
            '\u{FB8E}' | '\u{FB8F}' | '\u{FB90}' | '\u{FB91}' => '\u{06A9}', // KEHEH
            '\u{FB92}' | '\u{FB93}' | '\u{FB94}' | '\u{FB95}' => '\u{06AF}', // GAF
            '\u{FBA6}' | '\u{FBA7}' | '\u{FBA8}' | '\u{FBA9}' => '\u{06BA}', // NOON GHUNNA
            '\u{FBF6}' | '\u{FBF7}' | '\u{FBF8}' => '\u{06CC}',              // FARSI YEH
            _ => ch,
        };
        out.push(normalized);
    }
    out
}

/// Reorders visual-reversed RTL text into logical standard order for Markdown.
pub fn process_bidi_text(input: &str) -> String {
    if !contains_rtl(input) {
        return input.to_string();
    }

    // Check if input was stored in visual reversed order (final presentation form first)
    let first_rtl = input.chars().find(|c| is_rtl_char(*c));
    let last_rtl = input.chars().rev().find(|c| is_rtl_char(*c));

    let is_visual_reversed = match (first_rtl, last_rtl) {
        (Some(f), Some(l)) => is_final_presentation_form(f) && is_initial_presentation_form(l),
        _ => false,
    };

    let normalized = normalize_arabic_presentation_forms(input);

    if is_visual_reversed {
        // Reverse characters to restore logical order
        normalized.chars().rev().collect()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic_presentation_forms() {
        let raw = "\u{FE8D}\u{FEDF}\u{FEB3}\u{FE8E}\u{FEDF}\u{FEE2}";
        let normalized = normalize_arabic_presentation_forms(raw);
        assert!(!normalized.contains('\u{FE8D}'));
    }

    #[test]
    fn test_contains_rtl() {
        assert!(contains_rtl("مرحبا بالعالم"));
        assert!(!contains_rtl("Hello World"));
    }
}
