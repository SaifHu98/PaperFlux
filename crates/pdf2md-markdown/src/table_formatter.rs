use pdf2md_ast::{CellAlignment, TableCell, TableRow};
use crate::options::RenderOptions;

pub fn format_table(
    headers: &[TableRow],
    rows: &[TableRow],
    caption: Option<&str>,
    options: &RenderOptions,
) -> String {
    let has_complex_spans = headers
        .iter()
        .chain(rows.iter())
        .any(|r| r.cells.iter().any(|c| c.colspan > 1 || c.rowspan > 1));

    if has_complex_spans && options.allow_html_tables_for_spans {
        return format_html_table(headers, rows, caption);
    }

    format_gfm_table(headers, rows, caption)
}

fn format_gfm_table(
    headers: &[TableRow],
    rows: &[TableRow],
    caption: Option<&str>,
) -> String {
    let mut out = String::new();

    if let Some(cap) = caption {
        out.push_str(&format!("*Table: {}*\n\n", cap));
    }

    // Determine number of columns
    let num_cols = headers
        .first()
        .map(|h| h.cells.len())
        .or_else(|| rows.first().map(|r| r.cells.len()))
        .unwrap_or(0);

    if num_cols == 0 {
        return out;
    }

    // Write headers
    if let Some(header_row) = headers.first() {
        out.push('|');
        for cell in &header_row.cells {
            out.push(' ');
            out.push_str(&escape_gfm_cell(&cell.text_content()));
            out.push_str(" |");
        }
        out.push('\n');

        // Write separator
        out.push('|');
        for cell in &header_row.cells {
            match cell.align {
                CellAlignment::Left => out.push_str(":---|"),
                CellAlignment::Center => out.push_str(":---:|"),
                CellAlignment::Right => out.push_str("---:|"),
                CellAlignment::None => out.push_str("---|"),
            }
        }
        out.push('\n');
    } else {
        // Synthesize default header row if none exists
        out.push('|');
        for i in 1..=num_cols {
            out.push_str(&format!(" Col {} |", i));
        }
        out.push('\n');
        out.push('|');
        for _ in 0..num_cols {
            out.push_str("---|");
        }
        out.push('\n');
    }

    // Write rows
    for row in rows {
        out.push('|');
        for (i, cell) in row.cells.iter().enumerate() {
            if i >= num_cols {
                break;
            }
            out.push(' ');
            out.push_str(&escape_gfm_cell(&cell.text_content()));
            out.push_str(" |");
        }
        // Pad missing cells
        for _ in row.cells.len()..num_cols {
            out.push_str("  |");
        }
        out.push('\n');
    }

    out
}

fn format_html_table(
    headers: &[TableRow],
    rows: &[TableRow],
    caption: Option<&str>,
) -> String {
    let mut out = String::from("<table>\n");

    if let Some(cap) = caption {
        out.push_str(&format!("  <caption>{}</caption>\n", cap));
    }

    if !headers.is_empty() {
        out.push_str("  <thead>\n");
        for row in headers {
            out.push_str("    <tr>\n");
            for cell in &row.cells {
                out.push_str(&format_html_cell(cell, true));
            }
            out.push_str("    </tr>\n");
        }
        out.push_str("  </thead>\n");
    }

    if !rows.is_empty() {
        out.push_str("  <tbody>\n");
        for row in rows {
            out.push_str("    <tr>\n");
            for cell in &row.cells {
                out.push_str(&format_html_cell(cell, false));
            }
            out.push_str("    </tr>\n");
        }
        out.push_str("  </tbody>\n");
    }

    out.push_str("</table>\n");
    out
}

fn format_html_cell(cell: &TableCell, is_header: bool) -> String {
    let tag = if is_header { "th" } else { "td" };
    let mut attrs = String::new();

    if cell.colspan > 1 {
        attrs.push_str(&format!(" colspan=\"{}\"", cell.colspan));
    }
    if cell.rowspan > 1 {
        attrs.push_str(&format!(" rowspan=\"{}\"", cell.rowspan));
    }
    match cell.align {
        CellAlignment::Left => attrs.push_str(" align=\"left\""),
        CellAlignment::Center => attrs.push_str(" align=\"center\""),
        CellAlignment::Right => attrs.push_str(" align=\"right\""),
        CellAlignment::None => {}
    }

    format!("      <{}{}>{}</{}>\n", tag, attrs, cell.text_content(), tag)
}

fn escape_gfm_cell(text: &str) -> String {
    let clean = text.replace('|', "\\|").replace('\n', "<br>").replace('\r', "");
    clean.trim().to_string()
}
