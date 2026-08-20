use pdf2md_ast::geometry::{BoundingBox, Point, Rect};
use pdf2md_ast::{CellAlignment, InlineNode, Node, TableCell, TableRow};
use pdf2md_pdf::elements::{PathSegment, TextSpan};
use pdf2md_text::normalizer::TextNormalizer;

pub fn extract_lattice_tables(
    paths: &[PathSegment],
    spans: &[TextSpan],
) -> Vec<(Node, Vec<usize>)> {
    let mut tables = Vec::new();
    let mut horizontal_lines = Vec::new();
    let mut vertical_lines = Vec::new();

    for path in paths {
        if let Some(rect) = &path.rect {
            if rect.width > 20.0 && rect.height <= 3.0 {
                horizontal_lines.push((rect.y, rect.x_min(), rect.x_max()));
            } else if rect.height > 20.0 && rect.width <= 3.0 {
                vertical_lines.push((rect.x, rect.y_min(), rect.y_max()));
            }
        } else if path.points.len() >= 2 {
            for window in path.points.windows(2) {
                let p1 = window[0];
                let p2 = window[1];
                let dx = (p1.x - p2.x).abs();
                let dy = (p1.y - p2.y).abs();

                if dy <= 2.0 && dx > 20.0 {
                    horizontal_lines.push((p1.y.min(p2.y), p1.x.min(p2.x), p1.x.max(p2.x)));
                } else if dx <= 2.0 && dy > 15.0 {
                    vertical_lines.push((p1.x.min(p2.x), p1.y.min(p2.y), p1.y.max(p2.y)));
                }
            }
        }
    }

    // Need at least 2 horizontal and 2 vertical lines to form a grid table
    if horizontal_lines.len() < 2 || vertical_lines.len() < 2 {
        return tables;
    }

    // Cluster Y baselines and X columns
    let mut y_coords: Vec<f32> = horizontal_lines.iter().map(|(y, _, _)| *y).collect();
    y_coords.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let clustered_y = cluster_1d_coords(&y_coords, 4.0);

    let mut x_coords: Vec<f32> = vertical_lines.iter().map(|(x, _, _)| *x).collect();
    x_coords.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let clustered_x = cluster_1d_coords(&x_coords, 4.0);

    if clustered_y.len() < 2 || clustered_x.len() < 2 {
        return tables;
    }

    // Build grid cells
    let mut grid_rows = Vec::new();
    let mut consumed_span_indices = Vec::new();

    for y_win in clustered_y.windows(2) {
        let top_y = y_win[0];
        let bot_y = y_win[1];
        let mut row_cells = Vec::new();

        for x_win in clustered_x.windows(2) {
            let left_x = x_win[0];
            let right_x = x_win[1];
            let cell_rect = Rect::new(left_x, top_y, right_x - left_x, bot_y - top_y);

            // Find all text spans inside this cell
            let mut cell_text = String::new();
            for (idx, span) in spans.iter().enumerate() {
                if cell_rect.contains_point(Point::new(span.bbox.center_x(), span.bbox.center_y())) {
                    if !cell_text.is_empty() {
                        cell_text.push(' ');
                    }
                    cell_text.push_str(&span.text);
                    if !consumed_span_indices.contains(&idx) {
                        consumed_span_indices.push(idx);
                    }
                }
            }

            let normalized = TextNormalizer::normalize(&cell_text);
            row_cells.push(TableCell {
                content: vec![InlineNode::Text(normalized)],
                colspan: 1,
                rowspan: 1,
                align: CellAlignment::Left,
                bbox: Some(cell_rect),
            });
        }

        grid_rows.push(TableRow {
            cells: row_cells,
            is_header: false,
        });
    }

    if !grid_rows.is_empty() {
        // Mark first row as header
        let headers = vec![grid_rows.remove(0)];
        let rows = grid_rows;

        let table_bbox = BoundingBox::new(
            clustered_x.first().copied().unwrap_or(0.0),
            clustered_y.first().copied().unwrap_or(0.0),
            clustered_x.last().copied().unwrap_or(0.0) - clustered_x.first().copied().unwrap_or(0.0),
            clustered_y.last().copied().unwrap_or(0.0) - clustered_y.first().copied().unwrap_or(0.0),
        );

        tables.push((
            Node::Table {
                headers,
                rows,
                caption: None,
                confidence: 0.95,
                has_borders: true,
                bbox: Some(table_bbox),
            },
            consumed_span_indices,
        ));
    }

    tables
}

fn cluster_1d_coords(coords: &[f32], tolerance: f32) -> Vec<f32> {
    let mut clustered: Vec<f32> = Vec::new();
    for &c in coords {
        if let Some(last) = clustered.last_mut() {
            let diff: f32 = (c - *last).abs();
            if diff <= tolerance {
                *last = (*last + c) / 2.0;
                continue;
            }
        }
        clustered.push(c);
    }
    clustered
}
