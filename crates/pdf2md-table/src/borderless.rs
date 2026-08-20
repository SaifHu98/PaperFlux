use pdf2md_ast::{CellAlignment, InlineNode, Node, TableCell, TableRow};
use pdf2md_pdf::elements::TextSpan;
use pdf2md_text::normalizer::TextNormalizer;

pub fn extract_borderless_tables(
    spans: &[TextSpan],
    consumed_indices: &[usize],
) -> Vec<(Node, Vec<usize>)> {
    let mut unconsumed_spans: Vec<(usize, TextSpan)> = spans
        .iter()
        .enumerate()
        .filter(|(idx, _)| !consumed_indices.contains(idx))
        .map(|(idx, s)| (idx, s.clone()))
        .collect();

    if unconsumed_spans.len() < 6 {
        return Vec::new();
    }

    // Group spans into lines by Y coordinate
    unconsumed_spans.sort_by(|(_, a), (_, b)| a.bbox.y.partial_cmp(&b.bbox.y).unwrap());

    let mut lines: Vec<Vec<(usize, TextSpan)>> = Vec::new();
    for item in unconsumed_spans {
        if let Some(last_line) = lines.last_mut() {
            if let Some((_, first_span)) = last_line.first() {
                if (item.1.bbox.y - first_span.bbox.y).abs() <= 4.0 {
                    last_line.push(item);
                    continue;
                }
            }
        }
        lines.push(vec![item]);
    }

    let mut tables = Vec::new();
    let mut current_table_lines = Vec::new();
    let mut table_consumed = Vec::new();

    for line in lines {
        // Sort spans in line by X coordinate
        let mut sorted_line = line;
        sorted_line.sort_by(|(_, a), (_, b)| a.bbox.x.partial_cmp(&b.bbox.x).unwrap());

        // A line belongs to a potential tabular structure if it has >= 2 horizontally spaced columns
        if sorted_line.len() >= 2 && has_distinct_columns(&sorted_line) {
            for (idx, _) in &sorted_line {
                table_consumed.push(*idx);
            }
            current_table_lines.push(sorted_line);
        } else {
            if current_table_lines.len() >= 3 {
                if let Some(table_node) = build_borderless_table(&current_table_lines) {
                    tables.push((table_node, table_consumed.clone()));
                }
            }
            current_table_lines.clear();
            table_consumed.clear();
        }
    }

    if current_table_lines.len() >= 3 {
        if let Some(table_node) = build_borderless_table(&current_table_lines) {
            tables.push((table_node, table_consumed));
        }
    }

    tables
}

fn has_distinct_columns(spans: &[(usize, TextSpan)]) -> bool {
    for window in spans.windows(2) {
        let gap = window[1].1.bbox.x_min() - window[0].1.bbox.x_max();
        if gap >= 12.0 {
            return true;
        }
    }
    false
}

fn build_borderless_table(lines: &[Vec<(usize, TextSpan)>]) -> Option<Node> {
    let mut all_rows = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        let mut cells = Vec::new();
        for (_, span) in line {
            let normalized = TextNormalizer::normalize(&span.text);
            cells.push(TableCell {
                content: vec![InlineNode::Text(normalized)],
                colspan: 1,
                rowspan: 1,
                align: CellAlignment::Left,
                bbox: Some(span.bbox),
            });
        }

        all_rows.push(TableRow {
            cells,
            is_header: line_idx == 0,
        });
    }

    if all_rows.is_empty() {
        return None;
    }

    let headers = vec![all_rows.remove(0)];
    let rows = all_rows;

    Some(Node::Table {
        headers,
        rows,
        caption: None,
        confidence: 0.82,
        has_borders: false,
        bbox: None,
    })
}
