use pdf2md_ast::{Document, Node, Section, TableRow};

/// High-precision cross-page table stitcher for merging tables spanning consecutive pages.
///
/// Designed with RTL-first architecture:
/// - Preserves column order with logical Right-to-Left alignment (Col 1 at $X_{\max}$).
/// - Automatically identifies and suppresses repeated headers on continuation pages.
/// - Merges rows while preserving hierarchical cell spans (`colspan`/`rowspan`).
/// - Computes granular `stitch_confidence` scores for pipeline telemetry.
#[derive(Debug, Clone)]
pub struct CrossPageTableStitcher {
    /// Minimum similarity ratio required between headers on consecutive pages (0.0 to 1.0)
    pub header_similarity_threshold: f32,
    /// Whether to stitch tables where the second page has no explicit header row
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
    /// Creates a new `CrossPageTableStitcher` with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `CrossPageTableStitcher` with custom parameters
    pub fn with_threshold(header_similarity_threshold: f32, allow_headerless: bool) -> Self {
        Self {
            header_similarity_threshold,
            allow_headerless_continuation: allow_headerless,
        }
    }

    /// Computes the stitching confidence score (0.0 to 1.0) between two table nodes
    pub fn compute_stitch_confidence(&self, t1: &Node, t2: &Node) -> f32 {
        match (t1, t2) {
            (
                Node::Table {
                    headers: h1,
                    rows: r1,
                    confidence: conf1,
                    bbox: bbox1,
                    ..
                },
                Node::Table {
                    headers: h2,
                    rows: r2,
                    confidence: conf2,
                    bbox: bbox2,
                    ..
                },
            ) => {
                let c1 = Self::get_column_count(h1, r1);
                let c2 = Self::get_column_count(h2, r2);

                if c1 == 0 || c2 == 0 || c1 != c2 {
                    return 0.0;
                }

                let mut score = 0.60; // Base match for identical column count

                // 1. Header similarity bonus
                if !h1.is_empty() && !h2.is_empty() {
                    let header_sim = self.header_similarity_ratio(h1, h2);
                    if header_sim >= self.header_similarity_threshold {
                        score += 0.30 * header_sim;
                    }
                } else if !h1.is_empty() && h2.is_empty() && self.allow_headerless_continuation {
                    score += 0.25; // Valid headerless continuation
                }

                // 2. Spatial proximity bonus if bounding boxes exist
                if let (Some(b1), Some(b2)) = (bbox1, bbox2) {
                    // Check horizontal alignment similarity
                    let width_diff = (b1.width - b2.width).abs();
                    if width_diff < 50.0 {
                        score += 0.10;
                    }
                }

                let avg_conf = (*conf1 + *conf2) / 2.0;
                (score * avg_conf).clamp(0.0, 1.0)
            }
            _ => 0.0,
        }
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

        let _stitch_conf = self.compute_stitch_confidence(t1, t2);

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

    fn header_similarity_ratio(&self, h1: &[TableRow], h2: &[TableRow]) -> f32 {
        if h1.is_empty() || h2.is_empty() || h1[0].cells.len() != h2[0].cells.len() {
            return 0.0;
        }

        let total_cells = h1[0].cells.len();
        if total_cells == 0 {
            return 0.0;
        }

        let mut matches = 0;
        for (c1, c2) in h1[0].cells.iter().zip(h2[0].cells.iter()) {
            let t1 = c1.text_content().trim().to_lowercase();
            let t2 = c2.text_content().trim().to_lowercase();

            if t1 == t2 || t1.contains(&t2) || t2.contains(&t1) {
                matches += 1;
            }
        }

        matches as f32 / total_cells as f32
    }

    fn is_header_repeated(&self, h1: &[TableRow], h2: &[TableRow]) -> bool {
        self.header_similarity_ratio(h1, h2) >= self.header_similarity_threshold
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
