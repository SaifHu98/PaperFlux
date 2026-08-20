use pdf2md_pdf::elements::TextSpan;

#[derive(Debug, Clone)]
pub struct ArabicPageZones {
    pub header_spans: Vec<TextSpan>,
    pub spanning_title_spans: Vec<TextSpan>,
    pub author_abstract_spans: Vec<TextSpan>,
    pub columns: Vec<Vec<TextSpan>>,
    pub sidebar_spans: Vec<TextSpan>,
    pub footnote_spans: Vec<TextSpan>,
    pub footer_spans: Vec<TextSpan>,
}

pub struct ArabicReadingOrderEngine;

impl ArabicReadingOrderEngine {
    /// Sequences spans on an Arabic page in true right-to-left, multi-stage reading order
    pub fn sequence_arabic_page(
        spans: &[TextSpan],
        page_width: f32,
        page_height: f32,
    ) -> (Vec<TextSpan>, f32, Vec<String>) {
        if spans.is_empty() {
            return (Vec::new(), 1.0, Vec::new());
        }

        let mut warnings = Vec::new();
        let mut confidence: f32 = 0.98;

        // 1. Separate running header (top 40pt) and running footer (bottom 40pt)
        let mut header_spans = Vec::new();
        let mut footer_spans = Vec::new();
        let mut footnote_spans = Vec::new();
        let mut body_candidates = Vec::new();

        for span in spans {
            let y_top = span.bbox.y + span.bbox.height;
            let y_bottom = span.bbox.y;

            if y_bottom >= page_height - 40.0 {
                header_spans.push(span.clone());
            } else if y_top <= 40.0 {
                footer_spans.push(span.clone());
            } else if y_top <= 120.0 && is_likely_footnote(span) {
                footnote_spans.push(span.clone());
            } else {
                body_candidates.push(span.clone());
            }
        }

        // 2. Identify top spanning banners (Title, Document Title)
        let mut spanning_title_spans = Vec::new();
        let mut author_abstract_spans = Vec::new();
        let mut remaining_body = Vec::new();

        let max_body_y = body_candidates
            .iter()
            .map(|s| s.bbox.y + s.bbox.height)
            .fold(0.0f32, f32::max);

        for span in body_candidates {
            let is_high_y = (span.bbox.y + span.bbox.height) >= max_body_y - 100.0;
            let is_title_banner =
                is_high_y && (span.font_size >= 16.0 || (span.font_size >= 13.0 && span.is_bold));

            if is_title_banner {
                spanning_title_spans.push(span);
            } else if is_high_y
                && (span.text.contains("المستخلص")
                    || span.text.contains("ملخص")
                    || span.text.contains("Abstract"))
            {
                author_abstract_spans.push(span);
            } else {
                remaining_body.push(span);
            }
        }

        // 3. Cluster remaining body into Right-to-Left columns
        let (columns, sidebar_spans, col_conf, col_warnings) =
            cluster_rtl_columns(&remaining_body, page_width);

        confidence = confidence.min(col_conf);
        warnings.extend(col_warnings);

        // 4. Assemble in strict Arabic reading order:
        // Spanning Title -> Author/Abstract -> Right Column 1 -> Col 2 ... -> Col N -> Sidebar -> Footnotes -> Footer
        let mut ordered_spans = Vec::with_capacity(spans.len());

        // A. Header
        ordered_spans.extend(sort_rtl_spans(&header_spans));

        // B. Spanning Title
        ordered_spans.extend(sort_rtl_spans(&spanning_title_spans));

        // C. Author & Abstract
        ordered_spans.extend(sort_rtl_spans(&author_abstract_spans));

        // D. Body Columns from Rightmost to Leftmost
        for col in columns {
            ordered_spans.extend(sort_rtl_spans(&col));
        }

        // E. Sidebars
        ordered_spans.extend(sort_rtl_spans(&sidebar_spans));

        // F. Footnotes
        ordered_spans.extend(sort_rtl_spans(&footnote_spans));

        // G. Footer
        ordered_spans.extend(sort_rtl_spans(&footer_spans));

        (ordered_spans, confidence, warnings)
    }
}

/// Clusters body spans into columns and orders them Right-to-Left
fn cluster_rtl_columns(
    spans: &[TextSpan],
    page_width: f32,
) -> (Vec<Vec<TextSpan>>, Vec<TextSpan>, f32, Vec<String>) {
    if spans.is_empty() {
        return (Vec::new(), Vec::new(), 1.0, Vec::new());
    }

    let mut warnings = Vec::new();
    let mut confidence = 0.98;

    // Detect if page has multiple columns using X-center histogram
    let x_mid = page_width / 2.0;
    let mut right_col = Vec::new();
    let mut left_col = Vec::new();
    let mut center_col = Vec::new();
    let mut sidebars = Vec::new();

    // Check if document is 2-column or 3-column
    let is_multi_col = spans
        .iter()
        .any(|s| s.bbox.width <= page_width * 0.48 && s.bbox.width >= 50.0);

    if !is_multi_col {
        // Single column document
        return (vec![spans.to_vec()], Vec::new(), 0.99, Vec::new());
    }

    for span in spans {
        let x_center = span.bbox.x + span.bbox.width / 2.0;

        if span.bbox.width < page_width * 0.22
            && (span.bbox.x > page_width * 0.78 || span.bbox.x < page_width * 0.22)
        {
            sidebars.push(span.clone());
        } else if x_center > x_mid + 20.0 {
            // Right column (Arabic Column 1)
            right_col.push(span.clone());
        } else if x_center < x_mid - 20.0 {
            // Left column (Arabic Column 2)
            left_col.push(span.clone());
        } else {
            // Center spanning element inside columns
            center_col.push(span.clone());
        }
    }

    // Check column balance
    if right_col.is_empty() && !left_col.is_empty() {
        confidence = 0.88;
        warnings.push("Asymmetrical single left column detected in RTL document".to_string());
    }

    let mut columns = Vec::new();

    // In Arabic RTL, Right column is read first!
    if !right_col.is_empty() {
        columns.push(right_col);
    }
    if !center_col.is_empty() {
        columns.push(center_col);
    }
    if !left_col.is_empty() {
        columns.push(left_col);
    }

    (columns, sidebars, confidence, warnings)
}

/// Sorts spans top-to-bottom (Y descending), and right-to-left within same line (X descending)
fn sort_rtl_spans(spans: &[TextSpan]) -> Vec<TextSpan> {
    let mut sorted = spans.to_vec();
    sorted.sort_by(|a, b| {
        // Quantize Y baseline to 6pt tolerance to group lines
        let y_a = (a.bbox.y / 6.0).round();
        let y_b = (b.bbox.y / 6.0).round();

        if (y_a - y_b).abs() < f32::EPSILON {
            // Same line in RTL -> Rightmost span comes first (X descending)
            b.bbox
                .x
                .partial_cmp(&a.bbox.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        } else {
            // Top to bottom (Y descending in PDF coordinate space)
            y_b.partial_cmp(&y_a).unwrap_or(std::cmp::Ordering::Equal)
        }
    });
    sorted
}

fn is_likely_footnote(span: &TextSpan) -> bool {
    span.font_size <= 10.0
        && (span.text.starts_with('(')
            || span.text.starts_with('[')
            || span.text.starts_with('١')
            || span.text.starts_with('1')
            || span.text.contains("انظر")
            || span.text.contains("المرجع"))
}
