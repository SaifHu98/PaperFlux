use pdf2md_ast::geometry::BoundingBox;
use pdf2md_pdf::elements::{PathCommand, PathSegment, TextSpan, VectorGraphic};

pub struct SvgSerializer;

impl SvgSerializer {
    /// Escapes XML special characters
    pub fn escape_xml(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Serializes a `VectorGraphic` containing paths and embedded text to a valid standalone SVG document
    pub fn serialize_vector_graphic(graphic: &VectorGraphic) -> String {
        let width = graphic.bbox.width.max(1.0);
        let height = graphic.bbox.height.max(1.0);
        Self::serialize_paths(
            &graphic.paths,
            &graphic.texts,
            graphic.bbox.x_min(),
            graphic.bbox.y_min(),
            width,
            height,
        )
    }

    /// Serializes a set of vector paths and text spans into a standalone SVG document
    pub fn serialize_paths(
        paths: &[PathSegment],
        texts: &[TextSpan],
        offset_x: f32,
        offset_y: f32,
        width: f32,
        height: f32,
    ) -> String {
        let mut svg = String::new();
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.2} {:.2}" width="{:.2}" height="{:.2}">"#,
            width, height, width, height
        ));
        svg.push('\n');

        // 1. Render paths & shapes
        for path in paths {
            if let Some(rect) = &path.rect {
                let rx = (rect.x - offset_x).max(0.0);
                let ry = (rect.y - offset_y).max(0.0);
                let stroke_color = if path.is_stroke {
                    path.color.to_hex()
                } else {
                    "none".to_string()
                };
                let fill_color = if path.is_fill {
                    path.fill_color.unwrap_or(path.color).to_hex()
                } else {
                    "none".to_string()
                };
                let stroke_width = if path.is_stroke {
                    path.stroke_width.max(0.5)
                } else {
                    0.0
                };

                svg.push_str(&format!(
                    r#"  <rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" stroke="{}" stroke-width="{:.2}" fill="{}" />"#,
                    rx, ry, rect.width, rect.height, stroke_color, stroke_width, fill_color
                ));
                svg.push('\n');
            } else {
                let mut d = String::new();

                if !path.commands.is_empty() {
                    for cmd in &path.commands {
                        match cmd {
                            PathCommand::MoveTo(p) => {
                                d.push_str(&format!(
                                    "M {:.2} {:.2} ",
                                    p.x - offset_x,
                                    p.y - offset_y
                                ));
                            }
                            PathCommand::LineTo(p) => {
                                d.push_str(&format!(
                                    "L {:.2} {:.2} ",
                                    p.x - offset_x,
                                    p.y - offset_y
                                ));
                            }
                            PathCommand::CurveTo { p1, p2, p3 } => {
                                d.push_str(&format!(
                                    "C {:.2} {:.2}, {:.2} {:.2}, {:.2} {:.2} ",
                                    p1.x - offset_x,
                                    p1.y - offset_y,
                                    p2.x - offset_x,
                                    p2.y - offset_y,
                                    p3.x - offset_x,
                                    p3.y - offset_y
                                ));
                            }
                            PathCommand::Rectangle(r) => {
                                let rx = r.x - offset_x;
                                let ry = r.y - offset_y;
                                d.push_str(&format!(
                                    "M {:.2} {:.2} H {:.2} V {:.2} H {:.2} Z ",
                                    rx,
                                    ry,
                                    rx + r.width,
                                    ry + r.height,
                                    rx
                                ));
                            }
                            PathCommand::ClosePath => {
                                d.push_str("Z ");
                            }
                        }
                    }
                } else if !path.points.is_empty() {
                    for (i, p) in path.points.iter().enumerate() {
                        let px = p.x - offset_x;
                        let py = p.y - offset_y;
                        if i == 0 {
                            d.push_str(&format!("M {:.2} {:.2} ", px, py));
                        } else {
                            d.push_str(&format!("L {:.2} {:.2} ", px, py));
                        }
                    }
                }

                if !d.trim().is_empty() {
                    let stroke_color = if path.is_stroke {
                        path.color.to_hex()
                    } else {
                        "none".to_string()
                    };
                    let fill_color = if path.is_fill {
                        path.fill_color.unwrap_or(path.color).to_hex()
                    } else {
                        "none".to_string()
                    };
                    let stroke_width = if path.is_stroke {
                        path.stroke_width.max(0.5)
                    } else {
                        0.0
                    };

                    svg.push_str(&format!(
                        r#"  <path d="{}" stroke="{}" stroke-width="{:.2}" fill="{}" />"#,
                        d.trim_end(),
                        stroke_color,
                        stroke_width,
                        fill_color
                    ));
                    svg.push('\n');
                }
            }
        }

        // 2. Render embedded Arabic / Latin text labels
        for text in texts {
            let tx = (text.bbox.x_min() - offset_x).max(0.0);
            let ty = (text.bbox.y_max() - offset_y).max(0.0);
            let escaped = Self::escape_xml(&text.text);
            let is_rtl = text.text.chars().any(|c| {
                ('\u{0600}'..='\u{06FF}').contains(&c)
                    || ('\u{0750}'..='\u{077F}').contains(&c)
                    || ('\u{FB50}'..='\u{FDFF}').contains(&c)
                    || ('\u{FE70}'..='\u{FEFF}').contains(&c)
                    || ('\u{0590}'..='\u{05FF}').contains(&c)
            });

            let mut style_attrs = Vec::new();
            if is_rtl {
                style_attrs.push(r#"direction="rtl" unicode-bidi="bidi-override""#.to_string());
            }
            if text.is_bold {
                style_attrs.push(r#"font-weight="bold""#.to_string());
            }
            if text.is_italic {
                style_attrs.push(r#"font-style="italic""#.to_string());
            }

            let style_str = if style_attrs.is_empty() {
                String::new()
            } else {
                format!(" {}", style_attrs.join(" "))
            };

            svg.push_str(&format!(
                r#"  <text x="{:.2}" y="{:.2}" font-size="{:.1}px" font-family="sans-serif" fill="{}"{}>{}</text>"#,
                tx,
                ty,
                text.font_size.max(6.0),
                text.color.to_hex(),
                style_str,
                escaped
            ));
            svg.push('\n');
        }

        svg.push_str("</svg>\n");
        svg
    }

    /// Clusters loose paths and intersecting text spans into discrete vector diagrams / charts
    pub fn cluster_vector_objects(paths: &[PathSegment], texts: &[TextSpan]) -> Vec<VectorGraphic> {
        if paths.is_empty() {
            return Vec::new();
        }

        // Compute overall bounding box of drawing paths
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        let mut valid_paths = Vec::new();
        for path in paths {
            if let Some(rect) = &path.rect {
                // Ignore whole-page backgrounds or thin 1px table separator lines
                if rect.width > 10.0 && rect.height > 10.0 {
                    min_x = min_x.min(rect.x);
                    min_y = min_y.min(rect.y);
                    max_x = max_x.max(rect.x + rect.width);
                    max_y = max_y.max(rect.y + rect.height);
                    valid_paths.push(path.clone());
                }
            } else if path.points.len() >= 2 {
                for p in &path.points {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
                valid_paths.push(path.clone());
            }
        }

        if valid_paths.is_empty() || min_x >= max_x || min_y >= max_y {
            return Vec::new();
        }

        let chart_bbox = BoundingBox::new(min_x, min_y, max_x - min_x, max_y - min_y);

        // Associate text spans located inside or immediately adjacent to the chart bbox
        let mut chart_texts = Vec::new();
        for text in texts {
            if text.bbox.x_min() >= min_x - 30.0
                && text.bbox.x_max() <= max_x + 30.0
                && text.bbox.y_min() >= min_y - 30.0
                && text.bbox.y_max() <= max_y + 30.0
            {
                chart_texts.push(text.clone());
            }
        }

        vec![VectorGraphic {
            id: "chart_1".to_string(),
            bbox: chart_bbox,
            paths: valid_paths,
            texts: chart_texts,
        }]
    }
}
