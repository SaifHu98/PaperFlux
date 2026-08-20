pub fn merge_hyphenated_lines(prev_line: &str, next_line: &str) -> (String, bool) {
    let trimmed_prev = prev_line.trim_end();
    let trimmed_next = next_line.trim_start();

    if trimmed_prev.is_empty() {
        return (next_line.to_string(), false);
    }
    if trimmed_next.is_empty() {
        return (prev_line.to_string(), false);
    }

    // Check if previous line ends with a hyphenation character
    let hyphen_chars = ['-', '\u{2010}', '\u{00AD}'];
    if let Some(last_char) = trimmed_prev.chars().last() {
        if hyphen_chars.contains(&last_char) {
            // Get character before the hyphen
            let before_hyphen = &trimmed_prev[..trimmed_prev.len() - last_char.len_utf8()];
            let first_char_next = trimmed_next.chars().next().unwrap_or(' ');

            // If the next word starts with lowercase, it is almost certainly a split word
            if first_char_next.is_alphabetic() && first_char_next.is_lowercase() {
                let merged = format!("{}{}", before_hyphen, trimmed_next);
                return (merged, true);
            }
        }
    }

    (format!("{} {}", trimmed_prev, trimmed_next), false)
}

pub fn clean_soft_hyphens(input: &str) -> String {
    input.replace('\u{00AD}', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_hyphenated_lines() {
        let (merged, was_hyphen) = merge_hyphenated_lines("The docu-", "mentation is clear.");
        assert!(was_hyphen);
        assert_eq!(merged, "The documentation is clear.");

        let (merged2, was_hyphen2) = merge_hyphenated_lines("This is a well-", "Known method.");
        assert!(!was_hyphen2); // Capital letter -> compound word preserved
        assert_eq!(merged2, "This is a well- Known method.");
    }
}
