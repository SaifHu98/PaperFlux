pub fn is_cjk_char(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}'   // CJK Unified Ideographs Extension A
        | '\u{20000}'..='\u{2A6DF}' // CJK Unified Ideographs Extension B
        | '\u{3040}'..='\u{309F}'   // Hiragana
        | '\u{30A0}'..='\u{30FF}'   // Katakana
        | '\u{AC00}'..='\u{D7AF}'   // Hangul Syllables
        | '\u{1100}'..='\u{11FF}'   // Hangul Jamo
        | '\u{3000}'..='\u{303F}'   // CJK Symbols and Punctuation
        | '\u{FF00}'..='\u{FFEF}'   // Halfwidth and Fullwidth Forms
    )
}

pub fn contains_cjk(text: &str) -> bool {
    text.chars().any(is_cjk_char)
}

/// Joins two consecutive lines of text while intelligently avoiding unwanted spaces between CJK characters.
pub fn join_lines_cjk_aware(line1: &str, line2: &str) -> String {
    let t1 = line1.trim_end();
    let t2 = line2.trim_start();

    if t1.is_empty() {
        return t2.to_string();
    }
    if t2.is_empty() {
        return t1.to_string();
    }

    let last_ch1 = t1.chars().last().unwrap();
    let first_ch2 = t2.chars().next().unwrap();

    // In CJK typography, line breaks within CJK text do not introduce spaces
    if is_cjk_char(last_ch1) && is_cjk_char(first_ch2) {
        format!("{}{}", t1, t2)
    } else {
        format!("{} {}", t1, t2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cjk_join() {
        assert_eq!(
            join_lines_cjk_aware("これは日本語の", "文章です。"),
            "これは日本語の文章です。"
        );
        assert_eq!(
            join_lines_cjk_aware("This is an English", "sentence."),
            "This is an English sentence."
        );
    }
}
