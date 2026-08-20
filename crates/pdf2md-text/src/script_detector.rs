use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptType {
    Latin,
    Arabic,
    Hebrew,
    Cyrillic,
    Greek,
    Devanagari,
    CJK,
    Other,
}

pub fn detect_script(ch: char) -> ScriptType {
    match ch {
        'a'..='z' | 'A'..='Z' | '\u{00C0}'..='\u{024F}' => ScriptType::Latin,
        '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}' | '\u{FB50}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}' => ScriptType::Arabic,
        '\u{0590}'..='\u{05FF}' => ScriptType::Hebrew,
        '\u{0400}'..='\u{04FF}' | '\u{0500}'..='\u{052F}' => ScriptType::Cyrillic,
        '\u{0370}'..='\u{03FF}' | '\u{1F00}'..='\u{1FFF}' => ScriptType::Greek,
        '\u{0900}'..='\u{097F}' => ScriptType::Devanagari,
        '\u{4E00}'..='\u{9FFF}' | '\u{3040}'..='\u{30FF}' | '\u{AC00}'..='\u{D7AF}' => ScriptType::CJK,
        _ => ScriptType::Other,
    }
}

pub fn detect_primary_script(text: &str) -> ScriptType {
    let mut counts: HashMap<ScriptType, usize> = HashMap::new();
    for ch in text.chars() {
        if ch.is_alphabetic() {
            let script = detect_script(ch);
            *counts.entry(script).or_insert(0) += 1;
        }
    }

    counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(script, _)| script)
        .unwrap_or(ScriptType::Latin)
}
