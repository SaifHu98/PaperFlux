use crate::arabic::context::NumeralSystem;

pub struct ArabicNumerals;

impl ArabicNumerals {
    /// Converts any Eastern or Persian numeral characters to Western Arabic numerals (0-9)
    pub fn to_western(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                // Eastern Arabic-Indic (0x0660..=0x0669)
                '٠' => '0',
                '١' => '1',
                '٢' => '2',
                '٣' => '3',
                '٤' => '4',
                '٥' => '5',
                '٦' => '6',
                '٧' => '7',
                '٨' => '8',
                '٩' => '9',

                // Perso-Arabic / Extended (0x06F0..=0x06F9)
                '۰' => '0',
                '۱' => '1',
                '۲' => '2',
                '۳' => '3',
                '۴' => '4',
                '۵' => '5',
                '۶' => '6',
                '۷' => '7',
                '۸' => '8',
                '۹' => '9',

                // Arabic separators
                '٫' => '.',
                '٬' => ',',
                '٪' => '%',

                _ => c,
            })
            .collect()
    }

    /// Converts Western Arabic numerals (0-9) to Eastern Arabic-Indic numerals (٠-٩)
    pub fn to_eastern_indic(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                '0' => '٠',
                '1' => '١',
                '2' => '٢',
                '3' => '٣',
                '4' => '٤',
                '5' => '٥',
                '6' => '٦',
                '7' => '٧',
                '8' => '٨',
                '9' => '٩',
                '.' => '٫',
                ',' => '٬',
                '%' => '٪',
                _ => c,
            })
            .collect()
    }

    /// Converts Western Arabic numerals (0-9) to Perso-Arabic numerals (۰-۹)
    pub fn to_perso_arabic(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                '0' => '۰',
                '1' => '۱',
                '2' => '۲',
                '3' => '۳',
                '4' => '۴',
                '5' => '۵',
                '6' => '۶',
                '7' => '۷',
                '8' => '۸',
                '9' => '۹',
                _ => c,
            })
            .collect()
    }

    /// Detects the dominant numeral system in the text
    pub fn detect_numeral_system(input: &str) -> NumeralSystem {
        let mut eastern_count = 0;
        let mut western_count = 0;
        let mut perso_count = 0;

        for c in input.chars() {
            match c {
                '٠'..='٩' => eastern_count += 1,
                '0'..='9' => western_count += 1,
                '۰'..='۹' => perso_count += 1,
                _ => {}
            }
        }

        if eastern_count > western_count && eastern_count > perso_count {
            NumeralSystem::EasternArabicIndic
        } else if western_count > eastern_count && western_count > perso_count {
            NumeralSystem::WesternArabic
        } else if perso_count > 0 {
            NumeralSystem::PersoArabic
        } else {
            NumeralSystem::WesternArabic
        }
    }
}
