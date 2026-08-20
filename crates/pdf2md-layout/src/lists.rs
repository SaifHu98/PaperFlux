use pdf2md_ast::{InlineNode, ListItem, Node};
use pdf2md_text::normalizer::TextNormalizer;

pub fn detect_list_item(text: &str, indent_pt: f32) -> Option<(bool, Option<String>, String, usize)> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let indent_level = ((indent_pt / 16.0).floor() as usize).min(6);

    // Bullet characters
    let bullet_chars = [
        '•', '-', '*', '+', '▪', '▫', '–', '—', '◦', '‣', '⁃', '\u{25CF}', '\u{25CB}', '\u{25AA}',
        '\u{25AB}',
    ];
    if let Some(first_ch) = trimmed.chars().next() {
        if bullet_chars.contains(&first_ch) {
            let after_bullet = trimmed[first_ch.len_utf8()..].trim_start();
            return Some((
                false,
                Some(first_ch.to_string()),
                after_bullet.to_string(),
                indent_level,
            ));
        }
    }

    // Numbered / Lettered / Clause lists: "1.", "1)", "(1)", "[1]", "a.", "i.", "1.1."
    if let Some(pos) = trimmed.find(|c| c == '.' || c == ')' || c == ']') {
        let prefix = &trimmed[..pos].trim_start_matches('(').trim_start_matches('[');
        if !prefix.is_empty() && prefix.len() <= 8 {
            if prefix.parse::<u64>().is_ok()
                || is_roman_numeral(prefix)
                || is_single_letter(prefix)
                || is_hierarchical_clause(prefix)
            {
                let after = trimmed[pos + 1..].trim_start();
                return Some((
                    true,
                    Some(prefix.to_string()),
                    after.to_string(),
                    indent_level,
                ));
            }
        }
    }

    None
}

fn is_hierarchical_clause(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() >= 2 && parts.iter().all(|p| p.parse::<u32>().is_ok())
}

fn is_roman_numeral(s: &str) -> bool {
    let lower = s.to_lowercase();
    let romans = [
        "i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x", "xi", "xii",
    ];
    romans.contains(&lower.as_str())
}

fn is_single_letter(s: &str) -> bool {
    s.len() == 1 && s.chars().next().unwrap().is_alphabetic()
}

pub fn create_list_node(items: Vec<(bool, Option<String>, String, usize)>) -> Node {
    let is_ordered = items.first().map(|(ord, _, _, _)| *ord).unwrap_or(false);
    let mut list_items = Vec::new();

    for (_ordered, bullet, content, level) in items {
        let normalized = TextNormalizer::normalize(&content);
        list_items.push(ListItem {
            inlines: vec![InlineNode::Text(normalized)],
            children: Vec::new(),
            bullet,
            level,
        });
    }

    Node::List {
        ordered: is_ordered,
        start: if is_ordered { Some(1) } else { None },
        items: list_items,
        bbox: None,
    }
}
