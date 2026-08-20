use crate::arabic::context::{ArabicShapingMode, DiacriticMode};

pub struct ArabicShaper;

impl ArabicShaper {
    /// Converts all Arabic presentation forms (Forms A & B), ligatures, and extended glyphs back to base logical Unicode
    pub fn unshape(input: &str, mode: ArabicShapingMode, diacritic_mode: DiacriticMode) -> String {
        if mode == ArabicShapingMode::Preserve {
            return input.to_string();
        }

        let mut out = String::with_capacity(input.len());

        for ch in input.chars() {
            // Check diacritic filtering first
            if is_arabic_diacritic(ch) {
                match diacritic_mode {
                    DiacriticMode::StripHarakat => continue,
                    DiacriticMode::PreserveShaddaOnly => {
                        if ch != '\u{0651}' {
                            // \u0651 is Shadda
                            continue;
                        }
                    }
                    _ => {}
                }
            }

            // Unshape Presentation Forms A & B and complex ligatures
            match ch {
                // Common Presentation Forms-A Multi-character Ligatures
                '\u{FDFA}' => {
                    out.push_str("صلى الله عليه وسلم");
                    continue;
                }
                '\u{FDFB}' => {
                    out.push_str("جل جلاله");
                    continue;
                }
                '\u{FDFC}' => {
                    out.push_str("ريال");
                    continue;
                }
                '\u{FDFD}' => {
                    out.push_str("بسم الله الرحمن الرحيم");
                    continue;
                }

                // Lam-Alef Presentation Forms-B Ligatures
                '\u{FEF5}' | '\u{FEF6}' => {
                    out.push('ل');
                    out.push('آ');
                    continue;
                }
                '\u{FEF7}' | '\u{FEF8}' => {
                    out.push('ل');
                    out.push('أ');
                    continue;
                }
                '\u{FEF9}' | '\u{FEFA}' => {
                    out.push('ل');
                    out.push('إ');
                    continue;
                }
                '\u{FEFB}' | '\u{FEFC}' => {
                    out.push('ل');
                    out.push('ا');
                    continue;
                }

                // General Presentation Forms-B Unshaping (isolated/initial/medial/final -> base character)
                '\u{FE80}' => out.push('\u{0621}'), // HAMZA
                '\u{FE81}' | '\u{FE82}' => out.push('\u{0622}'), // ALEF WITH MADDA
                '\u{FE83}' | '\u{FE84}' => out.push('\u{0623}'), // ALEF WITH HAMZA ABOVE
                '\u{FE85}' | '\u{FE86}' => out.push('\u{0624}'), // WAW WITH HAMZA
                '\u{FE87}' | '\u{FE88}' => out.push('\u{0625}'), // ALEF WITH HAMZA BELOW
                '\u{FE89}' | '\u{FE8A}' | '\u{FE8B}' | '\u{FE8C}' => out.push('\u{0626}'), // YEH WITH HAMZA
                '\u{FE8D}' | '\u{FE8E}' => out.push('\u{0627}'), // ALEF
                '\u{FE8F}' | '\u{FE90}' | '\u{FE91}' | '\u{FE92}' => out.push('\u{0628}'), // BEH
                '\u{FE93}' | '\u{FE94}' => out.push('\u{0629}'), // TEH MARBUTA
                '\u{FE95}' | '\u{FE96}' | '\u{FE97}' | '\u{FE98}' => out.push('\u{062A}'), // TEH
                '\u{FE99}' | '\u{FE9A}' | '\u{FE9B}' | '\u{FE9C}' => out.push('\u{062B}'), // THEH
                '\u{FE9D}' | '\u{FE9E}' | '\u{FE9F}' | '\u{FEA0}' => out.push('\u{062C}'), // JEEM
                '\u{FEA1}' | '\u{FEA2}' | '\u{FEA3}' | '\u{FEA4}' => out.push('\u{062D}'), // HAH
                '\u{FEA5}' | '\u{FEA6}' | '\u{FEA7}' | '\u{FEA8}' => out.push('\u{062E}'), // KHAH
                '\u{FEA9}' | '\u{FEAA}' => out.push('\u{062F}'), // DAL
                '\u{FEAB}' | '\u{FEAC}' => out.push('\u{0630}'), // THAL
                '\u{FEAD}' | '\u{FEAE}' => out.push('\u{0631}'), // REH
                '\u{FEAF}' | '\u{FEB0}' => out.push('\u{0632}'), // ZAIN
                '\u{FEB1}' | '\u{FEB2}' | '\u{FEB3}' | '\u{FEB4}' => out.push('\u{0633}'), // SEEN
                '\u{FEB5}' | '\u{FEB6}' | '\u{FEB7}' | '\u{FEB8}' => out.push('\u{0634}'), // SHEEN
                '\u{FEB9}' | '\u{FEBA}' | '\u{FEBB}' | '\u{FEBC}' => out.push('\u{0635}'), // SAD
                '\u{FEBD}' | '\u{FEBE}' | '\u{FEBF}' | '\u{FEC0}' => out.push('\u{0636}'), // DAD
                '\u{FEC1}' | '\u{FEC2}' | '\u{FEC3}' | '\u{FEC4}' => out.push('\u{0637}'), // TAH
                '\u{FEC5}' | '\u{FEC6}' | '\u{FEC7}' | '\u{FEC8}' => out.push('\u{0638}'), // ZAH
                '\u{FEC9}' | '\u{FECA}' | '\u{FECB}' | '\u{FECC}' => out.push('\u{0639}'), // AIN
                '\u{FECD}' | '\u{FECE}' | '\u{FECF}' | '\u{FED0}' => out.push('\u{063A}'), // GHAIN
                '\u{FED1}' | '\u{FED2}' | '\u{FED3}' | '\u{FED4}' => out.push('\u{0641}'), // FEH
                '\u{FED5}' | '\u{FED6}' | '\u{FED7}' | '\u{FED8}' => out.push('\u{0642}'), // QAF
                '\u{FED9}' | '\u{FEDA}' | '\u{FEDB}' | '\u{FEDC}' => out.push('\u{0643}'), // KAF
                '\u{FEDD}' | '\u{FEDE}' | '\u{FEDF}' | '\u{FEE0}' => out.push('\u{0644}'), // LAM
                '\u{FEE1}' | '\u{FEE2}' | '\u{FEE3}' | '\u{FEE4}' => out.push('\u{0645}'), // MEEM
                '\u{FEE5}' | '\u{FEE6}' | '\u{FEE7}' | '\u{FEE8}' => out.push('\u{0646}'), // NOON
                '\u{FEE9}' | '\u{FEEA}' | '\u{FEEB}' | '\u{FEEC}' => out.push('\u{0647}'), // HEH
                '\u{FEED}' | '\u{FEEE}' => out.push('\u{0648}'), // WAW
                '\u{FEEF}' | '\u{FEF0}' => out.push('\u{0649}'), // ALEF MAKSURA
                '\u{FEF1}' | '\u{FEF2}' | '\u{FEF3}' | '\u{FEF4}' => out.push('\u{064A}'), // YEH

                // Persian / Urdu / Kurdish Extended Forms
                '\u{FB56}' | '\u{FB57}' | '\u{FB58}' | '\u{FB59}' => out.push('\u{067E}'), // PEH (پ)
                '\u{FB66}' | '\u{FB67}' | '\u{FB68}' | '\u{FB69}' => out.push('\u{0679}'), // TTEH (ٹ)
                '\u{FB7A}' | '\u{FB7B}' | '\u{FB7C}' | '\u{FB7D}' => out.push('\u{0686}'), // TCHEH (چ)
                '\u{FB88}' | '\u{FB89}' => out.push('\u{0688}'), // DDAL (ڈ)
                '\u{FB8A}' | '\u{FB8B}' => out.push('\u{0698}'), // JEH (ژ)
                '\u{FB8C}' | '\u{FB8D}' => out.push('\u{0691}'), // RREH (ڑ)
                '\u{FB8E}' | '\u{FB8F}' | '\u{FB90}' | '\u{FB91}' => out.push('\u{06A9}'), // KEHEH (ک)
                '\u{FB92}' | '\u{FB93}' | '\u{FB94}' | '\u{FB95}' => out.push('\u{06AF}'), // GAF (گ)
                '\u{FBA6}' | '\u{FBA7}' | '\u{FBA8}' | '\u{FBA9}' => out.push('\u{06BA}'), // NOON GHUNNA (ں)
                '\u{FBAA}' | '\u{FBAB}' | '\u{FBAC}' | '\u{FBAD}' => out.push('\u{06C1}'), // HEH GOAL (ہ)
                '\u{FBAE}' | '\u{FBAF}' => out.push('\u{06D2}'), // YEH BARREE (ے)
                '\u{FBF6}' | '\u{FBF7}' | '\u{FBF8}' => out.push('\u{06CC}'), // FARSI YEH (ی)

                // Pashto Specific Presentation Forms-A
                '\u{FB62}' | '\u{FB63}' | '\u{FB64}' | '\u{FB65}' => out.push('\u{067C}'), // TEH WITH RING (ټ)
                '\u{FB6E}' | '\u{FB6F}' | '\u{FB70}' | '\u{FB71}' => out.push('\u{0681}'), // HAH WITH HAMZA ABOVE (ځ)
                '\u{FB76}' | '\u{FB77}' | '\u{FB78}' | '\u{FB79}' => out.push('\u{0685}'), // HAH WITH THREE DOTS (څ)
                '\u{FB84}' | '\u{FB85}' => out.push('\u{0689}'), // DAL WITH RING (ډ)
                '\u{FB9A}' | '\u{FB9B}' | '\u{FB9C}' | '\u{FB9D}' => out.push('\u{06AB}'), // GAF WITH RING (ګ)
                '\u{FBE4}' | '\u{FBE5}' | '\u{FBE6}' | '\u{FBE7}' => out.push('\u{06D0}'), // E (ې)
                '\u{FBFC}' | '\u{FBFD}' | '\u{FBFE}' | '\u{FBFF}' => out.push('\u{06CD}'), // YEH WITH TAIL (ۍ)

                // Sindhi Specific Presentation Forms-A
                '\u{FB52}' | '\u{FB53}' | '\u{FB54}' | '\u{FB55}' => out.push('\u{067B}'), // BEEH (ٻ)
                '\u{FB5A}' | '\u{FB5B}' | '\u{FB5C}' | '\u{FB5D}' => out.push('\u{0680}'), // BHEH (ڀ)
                '\u{FB5E}' | '\u{FB5F}' | '\u{FB60}' | '\u{FB61}' => out.push('\u{067D}'), // TEH WITH FOUR DOTS (ٽ)
                '\u{FB72}' | '\u{FB73}' | '\u{FB74}' | '\u{FB75}' => out.push('\u{0684}'), // DYEH (ڄ)
                '\u{FB7E}' | '\u{FB7F}' | '\u{FB80}' | '\u{FB81}' => out.push('\u{0687}'), // TCHEHEH (ڇ)
                '\u{FB96}' | '\u{FB97}' | '\u{FB98}' | '\u{FB99}' => out.push('\u{06A6}'), // PEHEH (ڦ)
                '\u{FB9E}' | '\u{FB9F}' | '\u{FBA0}' | '\u{FBA1}' => out.push('\u{06AA}'), // SWASH KAF (ڪ)
                '\u{FBA2}' | '\u{FBA3}' => out.push('\u{06B3}'), // GUEH (ڳ)

                // Kurdish Sorani specific glyphs
                '\u{06B5}' => out.push('ڵ'),
                '\u{06C6}' => out.push('ۆ'),
                '\u{06CE}' => out.push('ێ'),
                '\u{0695}' => out.push('ڕ'),
                '\u{06D5}' => out.push('ە'),

                // Tatweel / Kashida (decorative elongation, usually dropped in Markdown)
                '\u{0640}' => continue,

                _ => out.push(ch),
            }
        }

        out
    }
}

pub fn is_arabic_diacritic(c: char) -> bool {
    matches!(
        c as u32,
        0x064B..=0x065F // Fathatan, Dammatan, Kasratan, Fatha, Damma, Kasra, Shadda, Sukun
        | 0x0670        // Superscript Alif (Dagger Alif)
        | 0x06D6..=0x06ED // Quranic annotation marks
    )
}
