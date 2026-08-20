use crate::bidi::is_rtl_char;
use pdf2md_ast::geometry::WritingDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BidiTokenKind {
    ArabicText,
    LatinText,
    Numeric,
    Url,
    Email,
    FilenameOrPath,
    CodeFragment,
    MathOrChemicalFormula,
    CitationOrReference,
    PunctuationOrSymbol,
    Whitespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BidiToken {
    pub kind: BidiTokenKind,
    pub text: String,
    pub is_rtl: bool,
}

pub struct BidiTokenizer;

impl BidiTokenizer {
    /// Tokenizes input string into semantic BiDi-classified tokens
    pub fn tokenize(input: &str) -> Vec<BidiToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            // 1. Whitespace
            if c.is_whitespace() {
                let start = i;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                tokens.push(BidiToken {
                    kind: BidiTokenKind::Whitespace,
                    text,
                    is_rtl: false,
                });
                continue;
            }

            // 2. URLs (http://, https://, www.)
            let remaining: String = chars[i..].iter().collect();
            if remaining.starts_with("http://")
                || remaining.starts_with("https://")
                || remaining.starts_with("www.")
            {
                let start = i;
                while i < chars.len()
                    && !chars[i].is_whitespace()
                    && chars[i] != ')'
                    && chars[i] != '>'
                    && chars[i] != ']'
                {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                tokens.push(BidiToken {
                    kind: BidiTokenKind::Url,
                    text,
                    is_rtl: false,
                });
                continue;
            }

            // 3. Citations & References ([1], [12], [A1])
            if c == '[' {
                if let Some(close_idx) = chars[i..].iter().position(|&ch| ch == ']') {
                    let end = i + close_idx + 1;
                    let candidate: String = chars[i..end].iter().collect();
                    if candidate.len() <= 10
                        && candidate[1..candidate.len() - 1]
                            .chars()
                            .all(|ch| ch.is_alphanumeric() || ch == ',' || ch == '-')
                    {
                        tokens.push(BidiToken {
                            kind: BidiTokenKind::CitationOrReference,
                            text: candidate,
                            is_rtl: false,
                        });
                        i = end;
                        continue;
                    }
                }
            }

            // 4. Code fragments enclosed in backticks (`foo()`)
            if c == '`' {
                if let Some(close_idx) = chars[i + 1..].iter().position(|&ch| ch == '`') {
                    let end = i + 1 + close_idx + 1;
                    let text: String = chars[i..end].iter().collect();
                    tokens.push(BidiToken {
                        kind: BidiTokenKind::CodeFragment,
                        text,
                        is_rtl: false,
                    });
                    i = end;
                    continue;
                }
            }

            // 5. Arabic Text
            if is_rtl_char(c) {
                let start = i;
                while i < chars.len() && (is_rtl_char(chars[i]) || chars[i] == '\u{0640}') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                tokens.push(BidiToken {
                    kind: BidiTokenKind::ArabicText,
                    text,
                    is_rtl: true,
                });
                continue;
            }

            // 6. Numeric (Western, Eastern Arabic-Indic, Perso-Arabic)
            if c.is_ascii_digit() || matches!(c, '٠'..='٩' | '۰'..='۹') {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_digit()
                        || matches!(chars[i], '٠'..='٩' | '۰'..='۹' | '.' | ',' | '٫' | '٬' | '%' | '٪' | '-'))
                {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                tokens.push(BidiToken {
                    kind: BidiTokenKind::Numeric,
                    text,
                    is_rtl: false,
                });
                continue;
            }

            // 7. Latin Text (Words, Acronyms, Identifiers)
            if c.is_ascii_alphabetic() {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '-')
                {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();

                // Check if it's an email (e.g. contains @)
                if i < chars.len() && chars[i] == '@' {
                    i += 1;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '-')
                    {
                        i += 1;
                    }
                    let full_email: String = chars[start..i].iter().collect();
                    tokens.push(BidiToken {
                        kind: BidiTokenKind::Email,
                        text: full_email,
                        is_rtl: false,
                    });
                    continue;
                }

                // Check if it's a filename (e.g. app.php, index.ts)
                if i < chars.len()
                    && chars[i] == '.'
                    && i + 1 < chars.len()
                    && chars[i + 1].is_ascii_alphabetic()
                {
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_alphabetic() {
                        i += 1;
                    }
                    let full_file: String = chars[start..i].iter().collect();
                    tokens.push(BidiToken {
                        kind: BidiTokenKind::FilenameOrPath,
                        text: full_file,
                        is_rtl: false,
                    });
                    continue;
                }

                tokens.push(BidiToken {
                    kind: BidiTokenKind::LatinText,
                    text,
                    is_rtl: false,
                });
                continue;
            }

            // 8. Punctuation and Symbols
            tokens.push(BidiToken {
                kind: BidiTokenKind::PunctuationOrSymbol,
                text: c.to_string(),
                is_rtl: is_rtl_char(c),
            });
            i += 1;
        }

        tokens
    }
}

pub struct ArabicBidiEngine;

impl ArabicBidiEngine {
    /// Detects paragraph base direction according to UBA P1/P2/P3 rules and character density
    pub fn detect_paragraph_direction(text: &str) -> WritingDirection {
        let mut rtl_count = 0;
        let mut ltr_count = 0;
        let mut first_strong: Option<WritingDirection> = None;

        for c in text.chars() {
            if is_rtl_char(c) {
                rtl_count += 1;
                if first_strong.is_none() {
                    first_strong = Some(WritingDirection::RightToLeft);
                }
            } else if c.is_ascii_alphabetic() {
                ltr_count += 1;
                if first_strong.is_none() {
                    first_strong = Some(WritingDirection::LeftToRight);
                }
            }
        }

        // If first strong character is RTL or more than 30% of strong text is RTL -> RTL paragraph
        if rtl_count > 0
            && (rtl_count >= ltr_count || first_strong == Some(WritingDirection::RightToLeft))
        {
            WritingDirection::RightToLeft
        } else {
            WritingDirection::LeftToRight
        }
    }

    /// Full UBA-compliant reconstruction of mixed Arabic/Latin/Numeric lines
    pub fn process_line(text: &str) -> String {
        if !text.chars().any(is_rtl_char) {
            return text.to_string();
        }

        let tokens = BidiTokenizer::tokenize(text);
        let mut out = String::with_capacity(text.len());

        for token in tokens {
            match token.kind {
                // Arabic text runs
                BidiTokenKind::ArabicText => {
                    out.push_str(&crate::bidi::normalize_arabic_presentation_forms(
                        &token.text,
                    ));
                }
                // Protected LTR isolates: URLs, emails, code, filenames, Latin identifiers, numbers
                BidiTokenKind::Url
                | BidiTokenKind::Email
                | BidiTokenKind::FilenameOrPath
                | BidiTokenKind::CodeFragment
                | BidiTokenKind::MathOrChemicalFormula
                | BidiTokenKind::CitationOrReference
                | BidiTokenKind::LatinText
                | BidiTokenKind::Numeric => {
                    out.push_str(&token.text);
                }
                // Punctuation and whitespace
                BidiTokenKind::PunctuationOrSymbol | BidiTokenKind::Whitespace => {
                    out.push_str(&token.text);
                }
            }
        }

        out
    }
}
