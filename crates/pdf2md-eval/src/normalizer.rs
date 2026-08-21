use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
pub struct NormalizationOptions {
    pub normalize_unicode_nfc: bool,
    pub collapse_whitespace: bool,
    pub strip_frontmatter: bool,
    pub strip_html_comments: bool,
    pub strip_markdown_decorations: bool,
    pub strip_tashkeel: bool,
    pub ignore_case: bool,
}

impl Default for NormalizationOptions {
    fn default() -> Self {
        Self {
            normalize_unicode_nfc: true,
            collapse_whitespace: true,
            strip_frontmatter: true,
            strip_html_comments: true,
            strip_markdown_decorations: true,
            strip_tashkeel: false,
            ignore_case: false,
        }
    }
}

pub struct TextNormalizer;

impl TextNormalizer {
    pub fn normalize(text: &str, opts: &NormalizationOptions) -> String {
        let mut s = text.trim().to_string();

        if opts.strip_frontmatter && s.starts_with("---") {
            if let Some(end_pos) = s[3..].find("---") {
                s = s[3 + end_pos + 3..].trim_start().to_string();
            }
        }

        if opts.strip_html_comments {
            while let Some(start) = s.find("<!--") {
                if let Some(end) = s[start..].find("-->") {
                    s.replace_range(start..start + end + 3, " ");
                } else {
                    break;
                }
            }
        }

        if opts.normalize_unicode_nfc {
            s = s.nfc().collect::<String>();
        }

        if opts.strip_tashkeel {
            s = s
                .chars()
                .filter(|&c| !('\u{064B}'..='\u{0652}').contains(&c) && c != '\u{0670}')
                .collect();
        }

        if opts.strip_markdown_decorations {
            s = s
                .replace('#', " ")
                .replace('*', " ")
                .replace('_', " ")
                .replace('`', " ")
                .replace('|', " ")
                .replace("- ", " ")
                .replace("---", " ");
        }

        if opts.ignore_case {
            s = s.to_lowercase();
        }

        if opts.collapse_whitespace {
            let mut out = String::with_capacity(s.len());
            let mut in_whitespace = false;
            for c in s.chars() {
                if c.is_whitespace() {
                    if !in_whitespace {
                        out.push(' ');
                        in_whitespace = true;
                    }
                } else {
                    out.push(c);
                    in_whitespace = false;
                }
            }
            s = out.trim().to_string();
        }

        s
    }
}
