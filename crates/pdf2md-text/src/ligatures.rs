pub fn unfold_ligatures(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\u{FB00}' => result.push_str("ff"),
            '\u{FB01}' => result.push_str("fi"),
            '\u{FB02}' => result.push_str("fl"),
            '\u{FB03}' => result.push_str("ffi"),
            '\u{FB04}' => result.push_str("ffl"),
            '\u{FB05}' => result.push_str("ft"),
            '\u{FB06}' => result.push_str("st"),
            '\u{0132}' => result.push_str("IJ"),
            '\u{0133}' => result.push_str("ij"),
            '\u{0152}' => result.push_str("OE"),
            '\u{0153}' => result.push_str("oe"),
            '\u{00C6}' => result.push_str("AE"),
            '\u{00E6}' => result.push_str("ae"),
            // Non-breaking spaces and special PDF space variants
            '\u{00A0}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => {
                result.push(' ')
            }
            // Soft hyphen
            '\u{00AD}' => {}
            // Zero-width spaces & marks that are safe to drop in standard text
            '\u{200B}' | '\u{FEFF}' => {}
            _ => result.push(ch),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unfold_ligatures() {
        assert_eq!(unfold_ligatures("e\u{FB00}ect"), "effect");
        assert_eq!(unfold_ligatures("o\u{FB03}ce"), "office");
        assert_eq!(unfold_ligatures("\u{FB01}le"), "file");
        assert_eq!(unfold_ligatures("\u{FB02}ow"), "flow");
        assert_eq!(unfold_ligatures("a\u{00AD}pple"), "apple");
    }
}
