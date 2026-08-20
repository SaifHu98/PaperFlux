use serde::{Deserialize, Serialize};
use crate::arabic::context::{ArabicProcessingContext, NumeralSystem};
use crate::arabic::numerals::ArabicNumerals;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArabicNumericExpression {
    Date(String),
    Time(String),
    Percentage(String),
    Currency { value: String, unit: String },
    DecimalNumber(String),
    Citation(String),
    FootnoteMarker(String),
    PageReference(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArabicScholarlySectionKind {
    Abstract,
    Introduction,
    Methodology,
    Results,
    Discussion,
    Conclusion,
    References,
    Footnotes,
}

pub struct ArabicSemanticNormalizer;

impl ArabicSemanticNormalizer {
    /// Detects Arabic numeric, date, time, percentage, and currency expressions
    pub fn detect_expressions(text: &str) -> Vec<ArabicNumericExpression> {
        let mut expressions = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        let mut i = 0;
        while i < words.len() {
            let w = words[i];

            // 1. Percentage: ends with % or ٪
            if w.ends_with('%') || w.ends_with('٪') {
                expressions.push(ArabicNumericExpression::Percentage(w.to_string()));
                i += 1;
                continue;
            }

            // 2. Date pattern: YYYY/MM/DD or DD/MM/YYYY with / or -
            if (w.contains('/') || w.contains('-')) && w.chars().any(|c| c.is_ascii_digit() || ('٠'..='٩').contains(&c) || ('۰'..='۹').contains(&c)) {
                let parts: Vec<&str> = if w.contains('/') { w.split('/').collect() } else { w.split('-').collect() };
                if parts.len() == 3 {
                    expressions.push(ArabicNumericExpression::Date(w.to_string()));
                    i += 1;
                    continue;
                }
            }

            // 3. Time pattern: HH:MM with : or ٫
            if w.contains(':') && w.chars().any(|c| c.is_ascii_digit() || ('٠'..='٩').contains(&c)) {
                expressions.push(ArabicNumericExpression::Time(w.to_string()));
                i += 1;
                continue;
            }

            // 4. Currency: value followed by currency unit (ر.س, SAR, ج.م, د.إ, $, €)
            if i + 1 < words.len() {
                let next = words[i + 1];
                let is_curr = matches!(next, "ر.س" | "SAR" | "ج.م" | "د.إ" | "د.ك" | "$" | "€" | "ريال" | "جنيه" | "درهم");
                let is_val = w.chars().all(|c| c.is_ascii_digit() || ('٠'..='٩').contains(&c) || ('۰'..='۹').contains(&c) || c == '.' || c == '٫' || c == ',');

                if is_curr && is_val {
                    expressions.push(ArabicNumericExpression::Currency {
                        value: w.to_string(),
                        unit: next.to_string(),
                    });
                    i += 2;
                    continue;
                }
            }

            // 5. Citations: [1], [ابن خلدون، 1980]
            if w.starts_with('[') && w.ends_with(']') {
                expressions.push(ArabicNumericExpression::Citation(w.to_string()));
                i += 1;
                continue;
            }

            // 6. Footnote markers: (1), (١), [1]
            if (w.starts_with('(') && w.ends_with(')')) && w.len() <= 5 {
                expressions.push(ArabicNumericExpression::FootnoteMarker(w.to_string()));
                i += 1;
                continue;
            }

            // 7. Page references: ص. 12, ص 25-30
            if (w == "ص." || w == "ص") && i + 1 < words.len() {
                let next = words[i + 1];
                expressions.push(ArabicNumericExpression::PageReference(format!("{} {}", w, next)));
                i += 2;
                continue;
            }

            i += 1;
        }

        expressions
    }

    /// Preserves original numerals by default unless user explicitly requested conversion
    pub fn process_numerals(text: &str, ctx: &ArabicProcessingContext) -> String {
        match ctx.numeral_system {
            NumeralSystem::PreserveAsIs | NumeralSystem::Mixed => text.to_string(),
            NumeralSystem::EasternArabicIndic => ArabicNumerals::to_eastern_indic(text),
            NumeralSystem::PersoArabic => ArabicNumerals::to_perso_arabic(text),
            NumeralSystem::WesternArabic => ArabicNumerals::to_western(text),
        }
    }
}

pub struct ArabicScholarlyDetector;

impl ArabicScholarlyDetector {
    /// Statistically classifies an Arabic heading into a standardized academic section kind
    pub fn classify_heading(heading_text: &str) -> Option<(ArabicScholarlySectionKind, u8)> {
        let clean = heading_text.trim().to_lowercase();

        // 1. Abstract
        if clean.contains("المستخلص") || clean.contains("ملخص") || clean.contains("خلاصة") || clean.contains("abstract") {
            return Some((ArabicScholarlySectionKind::Abstract, 2));
        }

        // 2. Introduction
        if clean.contains("المقدمة") || clean.contains("مقدمة") || clean.contains("توطئة") || clean.contains("تمهيد") || clean.contains("مدخل") || clean.contains("introduction") {
            return Some((ArabicScholarlySectionKind::Introduction, 2));
        }

        // 3. Methodology
        if clean.contains("المنهجية") || clean.contains("منهج البحث") || clean.contains("طريقة البحث") || clean.contains("إجراءات الدراسة") || clean.contains("المواد والطرائق") || clean.contains("methodology") {
            return Some((ArabicScholarlySectionKind::Methodology, 2));
        }

        // 4. Results
        if clean.contains("النتائج") || clean.contains("معطيات الدراسة") || clean.contains("findings") || clean.contains("results") {
            return Some((ArabicScholarlySectionKind::Results, 2));
        }

        // 5. Discussion
        if clean.contains("المناقشة") || clean.contains("مناقشة النتائج") || clean.contains("تحليل النتائج") || clean.contains("discussion") {
            return Some((ArabicScholarlySectionKind::Discussion, 2));
        }

        // 6. Conclusion
        if clean.contains("الخاتمة") || clean.contains("الاستنتاج") || clean.contains("خاتمة البحث") || clean.contains("التوصيات") || clean.contains("conclusion") {
            return Some((ArabicScholarlySectionKind::Conclusion, 2));
        }

        // 7. References
        if clean.contains("المراجع") || clean.contains("المصادر") || clean.contains("قائمة المراجع") || clean.contains("ثبت المراجع") || clean.contains("references") {
            return Some((ArabicScholarlySectionKind::References, 2));
        }

        // 8. Footnotes
        if clean.contains("الهوامش") || clean.contains("الحواشي") || clean.contains("شروح وتوضيحات") || clean.contains("footnotes") {
            return Some((ArabicScholarlySectionKind::Footnotes, 2));
        }

        None
    }

    /// Formats academic headings with appropriate Markdown header syntax
    pub fn format_scholarly_heading(heading: &str, level: usize) -> String {
        let prefix = "#".repeat(level.max(1).min(6));
        format!("{} {}", prefix, heading.trim())
    }

    /// Converts raw Arabic citation / footnote markers into clean Markdown syntax
    pub fn format_citation(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            let inner = &trimmed[1..trimmed.len() - 1];
            format!("[^{}]", inner.trim())
        } else {
            trimmed.to_string()
        }
    }
}
