use pdf2md_ast::{Document, Node, Section, TableRow};

pub struct CrossPageTableStitcher {
    pub header_similarity_threshold: f32,
    pub allow_headerless_continuation: bool,
}

impl Default for CrossPageTableStitcher {
    fn default() -> Self {
        Self {
            header_similarity_threshold: 0.70,
            allow_headerless_continuation: true,
        }
    }
}

impl CrossPageTableStitcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Determines if two AST Table nodes are compatible for cross-page stitching
    pub fn can_stitch(&self, t1: &Node, t2: &Node) -> bool {
        match (t1, t2) {
            (
                Node::Table {
                    headers: h1,
                    rows: r1,
                    ..
                },
                Node::Table {
                    headers: h2,
                    rows: r2,
                    ..
                },
            ) => {
                let c1 = Self::get_column_count(h1, r1);
                let c2 = Self::get_column_count(h2, r2);

                if c1 == 0 || c2 == 0 || c1 != c2 {
                    return false;
                }

                // Check for repeated header match or valid headerless continuation
                if !h2.is_empty() {
                    self.is_header_repeated(h1, h2) || (h1.len() == h2.len())
                } else if !r2.is_empty() {
                    // Page 2 has no explicit header but has matching data columns
                    self.allow_headerless_continuation
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Stitches two compatible Table nodes into a single logical Table node
    pub fn stitch_two_tables(&self, t1: &Node, t2: &Node) -> Option<Node> {
        if !self.can_stitch(t1, t2) {
            return None;
        }

        if let (
            Node::Table {
                headers: h1,
                rows: r1,
                caption: cap1,
                confidence: conf1,
                has_borders: b1,
                bbox: bbox1,
            },
            Node::Table {
                headers: h2,
                rows: r2,
                caption: cap2,
                confidence: conf2,
                has_borders: b2,
                bbox: bbox2,
            },
        ) = (t1, t2)
        {
            let mut stitched_rows = r1.clone();

            // Handle rows from Table 2:
            // If Table 2 has headers that repeat Table 1's header, ignore h2.
            // If Table 2 has no headers, check if its first data row is accidentally a repeated header row.
            let mut t2_data_rows = r2.clone();
            if h2.is_empty()
                && !t2_data_rows.is_empty()
                && !h1.is_empty()
                && self.is_row_header_duplicate(&h1[0], &t2_data_rows[0])
            {
                t2_data_rows.remove(0);
            }

            stitched_rows.extend(t2_data_rows);

            let caption = cap1.clone().or_else(|| cap2.clone());
            let confidence = (*conf1 + *conf2) / 2.0;
            let has_borders = *b1 || *b2;

            // Combine bounding boxes if present
            let bbox = match (bbox1, bbox2) {
                (Some(b1), Some(b2)) => Some(pdf2md_ast::geometry::BoundingBox::new(
                    b1.x.min(b2.x),
                    b1.y.min(b2.y),
                    b1.width.max(b2.width),
                    b1.height + b2.height,
                )),
                (Some(b1), None) => Some(*b1),
                (None, Some(b2)) => Some(*b2),
                (None, None) => None,
            };

            Some(Node::Table {
                headers: h1.clone(),
                rows: stitched_rows,
                caption,
                confidence,
                has_borders,
                bbox,
            })
        } else {
            None
        }
    }

    /// Stitches all consecutive compatible tables within a list of AST nodes
    pub fn stitch_table_nodes(&self, nodes: &[Node]) -> Vec<Node> {
        if nodes.is_empty() {
            return Vec::new();
        }

        let mut result: Vec<Node> = Vec::new();

        for node in nodes {
            if let Some(last) = result.last_mut() {
                if let Some(stitched) = self.stitch_two_tables(last, node) {
                    *last = stitched;
                    continue;
                }
            }
            result.push(node.clone());
        }

        result
    }

    /// Stitches tables across consecutive Sections in a Document
    pub fn stitch_sections(&self, sections: &mut [Section]) {
        if sections.len() < 2 {
            return;
        }

        for i in 0..sections.len() - 1 {
            // Check if the last element of section[i] is a Table and the first element of section[i+1] is a Table
            let can_merge = if let (Some(last_elem), Some(first_elem)) = (
                sections[i].elements.last(),
                sections[i + 1].elements.first(),
            ) {
                self.can_stitch(last_elem, first_elem)
            } else {
                false
            };

            if can_merge {
                let last_elem = sections[i].elements.pop().unwrap();
                let first_elem = sections[i + 1].elements.remove(0);

                if let Some(stitched_table) = self.stitch_two_tables(&last_elem, &first_elem) {
                    sections[i].elements.push(stitched_table);
                } else {
                    // Fallback in case stitching returned None
                    sections[i].elements.push(last_elem);
                    sections[i + 1].elements.insert(0, first_elem);
                }
            }
        }
    }

    /// Stitches cross-page tables across the entire document
    pub fn stitch_document(&self, doc: &mut Document) {
        self.stitch_sections(&mut doc.sections);
    }

    fn get_column_count(headers: &[TableRow], rows: &[TableRow]) -> usize {
        headers
            .first()
            .map(|r| r.cells.len())
            .or_else(|| rows.first().map(|r| r.cells.len()))
            .unwrap_or(0)
    }

    fn is_header_repeated(&self, h1: &[TableRow], h2: &[TableRow]) -> bool {
        if h1.is_empty() || h2.is_empty() {
            return false;
        }

        if h1[0].cells.len() != h2[0].cells.len() {
            return false;
        }

        let total_cells = h1[0].cells.len();
        if total_cells == 0 {
            return false;
        }

        let mut matches = 0;
        for (c1, c2) in h1[0].cells.iter().zip(h2[0].cells.iter()) {
            let t1 = c1.text_content().trim().to_lowercase();
            let t2 = c2.text_content().trim().to_lowercase();

            if t1 == t2 || t1.contains(&t2) || t2.contains(&t1) {
                matches += 1;
            }
        }

        (matches as f32 / total_cells as f32) >= self.header_similarity_threshold
    }

    fn is_row_header_duplicate(&self, header_row: &TableRow, data_row: &TableRow) -> bool {
        if header_row.cells.len() != data_row.cells.len() {
            return false;
        }

        let total_cells = header_row.cells.len();
        if total_cells == 0 {
            return false;
        }

        let mut matches = 0;
        for (c1, c2) in header_row.cells.iter().zip(data_row.cells.iter()) {
            let t1 = c1.text_content().trim().to_lowercase();
            let t2 = c2.text_content().trim().to_lowercase();

            if t1 == t2 && !t1.is_empty() {
                matches += 1;
            }
        }

        (matches as f32 / total_cells as f32) >= 0.70
    }
}
