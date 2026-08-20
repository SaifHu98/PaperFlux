pub struct ArabicJoiningReconstructor;

impl ArabicJoiningReconstructor {
    /// Reconnects space-separated isolated Arabic letters back into proper cursive words (e.g. "ت ق ر ي ر  ح و ل" -> "تقرير حول")
    pub fn reconstruct_isolated_words(text: &str) -> String {
        if text.contains("  ") {
            let words: Vec<String> = text
                .split("  ")
                .map(|word_chunk| {
                    let cleaned: String = word_chunk.split_whitespace().collect();
                    Self::repair_broken_ligatures(&cleaned)
                })
                .collect();
            words.join(" ")
        } else {
            Self::repair_broken_ligatures(text)
        }
    }

    /// Repairs broken Lam-Alef sequences ("ل ا" -> "لا", "ل أ" -> "لأ")
    pub fn repair_broken_ligatures(text: &str) -> String {
        text.replace("ل ا", "لا")
            .replace("ل أ", "لأ")
            .replace("ل إ", "لإ")
            .replace("ل آ", "لآ")
            .replace("لـا", "لا")
            .replace("لـأ", "لأ")
            .replace("لـإ", "لإ")
            .replace("لـآ", "لآ")
    }

    /// Attaches floating detached Harakat to preceding base consonant
    pub fn attach_floating_diacritics(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == ' ' {
                // If next character is a diacritic, skip the space so it attaches
                if let Some(&next_c) = chars.peek() {
                    if crate::arabic::shaping::is_arabic_diacritic(next_c) {
                        continue;
                    }
                }
            }
            out.push(c);
        }

        out
    }
}
