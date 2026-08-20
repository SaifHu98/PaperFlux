use pdf2md_ast::Node;
use pdf2md_pdf::elements::{PathSegment, TextSpan};
use crate::borderless::extract_borderless_tables;
use crate::lattice::extract_lattice_tables;
use crate::cell_merging::balance_table_columns;

pub struct TableDetector;

impl TableDetector {
    pub fn detect_tables(
        paths: &[PathSegment],
        spans: &[TextSpan],
    ) -> (Vec<Node>, Vec<usize>) {
        let mut final_tables = Vec::new();
        let mut all_consumed_indices = Vec::new();

        // 1. Try lattice table detection first (highest precision)
        let lattice_tables = extract_lattice_tables(paths, spans);
        for (table_node, consumed) in lattice_tables {
            all_consumed_indices.extend(consumed);
            final_tables.push(table_node);
        }

        // 2. Try borderless table detection on remaining spans
        let borderless_tables = extract_borderless_tables(spans, &all_consumed_indices);
        for (table_node, consumed) in borderless_tables {
            all_consumed_indices.extend(consumed);
            final_tables.push(table_node);
        }

        // Balance table columns for each table
        for table in &mut final_tables {
            if let Node::Table { headers, rows, .. } = table {
                balance_table_columns(headers, rows);
            }
        }

        (final_tables, all_consumed_indices)
    }
}
