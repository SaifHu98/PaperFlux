use pdf2md_ast::geometry::BoundingBox;
use pdf2md_pdf::elements::TextSpan;

#[derive(Debug, Clone)]
pub struct LayoutBlock {
    pub id: usize,
    pub bbox: BoundingBox,
    pub spans: Vec<TextSpan>,
    pub column_index: Option<usize>,
    pub is_full_width: bool,
    pub confidence: f32,
}

pub struct StagedReadingOrderEngine {
    pub min_column_gap_pt: f32,
    pub min_block_gap_pt: f32,
}

impl Default for StagedReadingOrderEngine {
    fn default() -> Self {
        Self {
            min_column_gap_pt: 12.0,
            min_block_gap_pt: 7.0,
        }
    }
}

impl StagedReadingOrderEngine {
    pub fn compute_reading_order(&self, spans: &[TextSpan], page_width: f32) -> Vec<LayoutBlock> {
        if spans.is_empty() {
            return Vec::new();
        }

        // 1. Detect column structure of the page
        let column_count = self.detect_column_count(spans, page_width);

        // 2. Separate full-width elements (e.g. Title, Abstract header) from column elements
        let mut full_width_top = Vec::new();
        let mut column_spans = Vec::new();
        let mut full_width_bottom = Vec::new();

        let mid_x = page_width / 2.0;

        for span in spans {
            if span.bbox.width > page_width * 0.65 {
                if span.bbox.center_y() < 250.0 {
                    full_width_top.push(span.clone());
                } else {
                    full_width_bottom.push(span.clone());
                }
            } else {
                column_spans.push(span.clone());
            }
        }

        let mut ordered_blocks = Vec::new();
        let mut next_id = 1;

        // Top full-width blocks
        if !full_width_top.is_empty() {
            let top_blocks = self.segment_vertical_blocks(&full_width_top, &mut next_id, true, None);
            ordered_blocks.extend(top_blocks);
        }

        // Multi-column or single-column body
        if !column_spans.is_empty() {
            if column_count == 2 {
                // 2-column flow: Left column (x < mid_x), Right column (x >= mid_x)
                let mut col_left = Vec::new();
                let mut col_right = Vec::new();

                for span in column_spans {
                    if span.bbox.center_x() < mid_x {
                        col_left.push(span);
                    } else {
                        col_right.push(span);
                    }
                }

                // Left column first, top-to-bottom
                let left_blocks = self.segment_vertical_blocks(&col_left, &mut next_id, false, Some(0));
                ordered_blocks.extend(left_blocks);

                // Right column second, top-to-bottom
                let right_blocks = self.segment_vertical_blocks(&col_right, &mut next_id, false, Some(1));
                ordered_blocks.extend(right_blocks);
            } else if column_count == 3 {
                // 3-column flow
                let col_w = page_width / 3.0;
                let mut col1 = Vec::new();
                let mut col2 = Vec::new();
                let mut col3 = Vec::new();

                for span in column_spans {
                    if span.bbox.center_x() < col_w {
                        col1.push(span);
                    } else if span.bbox.center_x() < col_w * 2.0 {
                        col2.push(span);
                    } else {
                        col3.push(span);
                    }
                }

                ordered_blocks.extend(self.segment_vertical_blocks(&col1, &mut next_id, false, Some(0)));
                ordered_blocks.extend(self.segment_vertical_blocks(&col2, &mut next_id, false, Some(1)));
                ordered_blocks.extend(self.segment_vertical_blocks(&col3, &mut next_id, false, Some(2)));
            } else {
                // Single column flow
                ordered_blocks.extend(self.segment_vertical_blocks(&column_spans, &mut next_id, false, None));
            }
        }

        // Bottom full-width blocks
        if !full_width_bottom.is_empty() {
            let bot_blocks = self.segment_vertical_blocks(&full_width_bottom, &mut next_id, true, None);
            ordered_blocks.extend(bot_blocks);
        }

        ordered_blocks
    }

    fn detect_column_count(&self, spans: &[TextSpan], page_width: f32) -> usize {
        let narrow_spans: Vec<&TextSpan> = spans
            .iter()
            .filter(|s| s.bbox.width < page_width * 0.55 && s.bbox.width > 20.0)
            .collect();

        if narrow_spans.len() < 4 {
            return 1;
        }

        let resolution = 2.0;
        let bins = ((page_width / resolution).ceil() as usize).max(1);
        let mut profile = vec![0usize; bins];

        for span in narrow_spans {
            let start = (((span.bbox.x_min()) / resolution).floor() as usize).min(bins - 1);
            let end = (((span.bbox.x_max()) / resolution).ceil() as usize).min(bins);
            for bin in &mut profile[start..end] {
                *bin += 1;
            }
        }

        // Count distinct vertical valleys
        let mut valleys = 0;
        let mut in_valley = false;
        let mut valley_width = 0.0;

        for &count in &profile[bins / 6..bins * 5 / 6] {
            if count == 0 {
                in_valley = true;
                valley_width += resolution;
            } else if in_valley {
                if valley_width >= self.min_column_gap_pt {
                    valleys += 1;
                }
                in_valley = false;
                valley_width = 0.0;
            }
        }

        if valleys >= 2 {
            3
        } else if valleys == 1 {
            2
        } else {
            1
        }
    }

    fn segment_vertical_blocks(
        &self,
        spans: &[TextSpan],
        next_id: &mut usize,
        is_full_width: bool,
        col_idx: Option<usize>,
    ) -> Vec<LayoutBlock> {
        if spans.is_empty() {
            return Vec::new();
        }

        let mut sorted_spans = spans.to_vec();
        sorted_spans.sort_by(|a, b| a.bbox.y.partial_cmp(&b.bbox.y).unwrap_or(std::cmp::Ordering::Equal));

        let mut blocks = Vec::new();
        let mut curr_group = Vec::new();

        for span in sorted_spans {
            if let Some(last) = curr_group.last() as Option<&TextSpan> {
                let vertical_gap = span.bbox.y_min() - last.bbox.y_max();
                if vertical_gap > self.min_block_gap_pt * 2.5 {
                    // Start new block
                    let bbox = compute_bounding_box(&curr_group);
                    blocks.push(LayoutBlock {
                        id: *next_id,
                        bbox,
                        spans: curr_group.clone(),
                        column_index: col_idx,
                        is_full_width,
                        confidence: 0.95,
                    });
                    *next_id += 1;
                    curr_group.clear();
                }
            }
            curr_group.push(span);
        }

        if !curr_group.is_empty() {
            let bbox = compute_bounding_box(&curr_group);
            blocks.push(LayoutBlock {
                id: *next_id,
                bbox,
                spans: curr_group,
                column_index: col_idx,
                is_full_width,
                confidence: 0.95,
            });
            *next_id += 1;
        }

        blocks
    }
}

pub fn compute_bounding_box(spans: &[TextSpan]) -> BoundingBox {
    if spans.is_empty() {
        return BoundingBox::new(0.0, 0.0, 0.0, 0.0);
    }
    let mut bbox = spans[0].bbox;
    for span in &spans[1..] {
        bbox = bbox.union(&span.bbox);
    }
    bbox
}
