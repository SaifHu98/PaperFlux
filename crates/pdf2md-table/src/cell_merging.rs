use pdf2md_ast::TableRow;

/// Resolves column span adjustments for table rows to maintain uniform grid column counts.
pub fn balance_table_columns(headers: &mut [TableRow], rows: &mut [TableRow]) {
    let max_cols = headers
        .iter()
        .chain(rows.iter())
        .map(|r| r.cells.iter().map(|c| c.colspan).sum::<usize>())
        .max()
        .unwrap_or(0);

    if max_cols == 0 {
        return;
    }

    for row in headers.iter_mut().chain(rows.iter_mut()) {
        let current_cols: usize = row.cells.iter().map(|c| c.colspan).sum();
        if current_cols < max_cols && !row.cells.is_empty() {
            let diff = max_cols - current_cols;
            if let Some(last_cell) = row.cells.last_mut() {
                last_cell.colspan += diff;
            }
        }
    }
}
