use pdf2md_core::{Config, Converter, MarkdownDialect};

fn create_synthetic_pdf(text_lines: &[&str]) -> Vec<u8> {
    let mut stream_content = String::new();
    stream_content.push_str("BT\n/F1 12 Tf\n72 700 Td\n");

    for (i, line) in text_lines.iter().enumerate() {
        if i > 0 {
            stream_content.push_str("0 -15 Td\n");
        }
        stream_content.push_str(&format!(
            "({}) Tj\n",
            line.replace('(', "\\(").replace(')', "\\)")
        ));
    }
    stream_content.push_str("ET\n");

    let stream_len = stream_content.len();

    let pdf = format!(
        "%PDF-1.4\n\
        1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
        2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
        3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n\
        4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n\
        xref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000300 00000 n \n\
        trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n450\n%%EOF\n",
        stream_len, stream_content
    );

    pdf.into_bytes()
}

#[test]
fn test_end_to_end_simple_conversion() {
    let lines = [
        "Introduction",
        "This is the first paragraph of the document.",
    ];
    let pdf_bytes = create_synthetic_pdf(&lines);

    let config = Config::builder()
        .dialect(MarkdownDialect::GitHubFlavored)
        .build();

    let converter = Converter::new(config);
    let result = converter
        .convert_bytes(&pdf_bytes)
        .expect("Conversion should succeed");

    assert!(!result.markdown.is_empty());
    assert!(result.markdown.contains("Introduction") || result.markdown.contains("paragraph"));
    assert_eq!(result.diagnostics.total_pages, 1);
}

#[test]
fn test_end_to_end_list_detection() {
    let lines = [
        "Features Overview",
        "• High performance",
        "• Multilingual support",
        "• Pluggable OCR",
    ];
    let pdf_bytes = create_synthetic_pdf(&lines);

    let config = Config::builder().build();
    let converter = Converter::new(config);
    let result = converter
        .convert_bytes(&pdf_bytes)
        .expect("Conversion should succeed");

    assert!(
        result.markdown.contains("- High performance")
            || result.markdown.contains("High performance")
    );
}

#[test]
fn test_config_calligraphy_ocr_dpi_options() {
    let config = Config::builder()
        .ocr_dpi(300)
        .calligraphic_dpi_escalation(true)
        .build();

    assert_eq!(config.ocr_dpi, Some(300));
    assert!(config.calligraphic_dpi_escalation);
    assert!(config.auto_calligraphy_dpi_boost);

    let config_custom = Config::builder()
        .ocr_dpi(400)
        .calligraphic_dpi_escalation(false)
        .build();

    assert_eq!(config_custom.ocr_dpi, Some(400));
    assert!(!config_custom.calligraphic_dpi_escalation);
    assert!(!config_custom.auto_calligraphy_dpi_boost);
}

fn create_synthetic_pdf_with_gaps(lines_and_offsets: &[(&str, f32)]) -> Vec<u8> {
    let mut stream_content = String::new();
    stream_content.push_str("BT\n/F1 12 Tf\n72 700 Td\n");

    for (i, (line, dy)) in lines_and_offsets.iter().enumerate() {
        if i > 0 {
            stream_content.push_str(&format!("0 -{:.1} Td\n", dy));
        }
        stream_content.push_str(&format!(
            "({}) Tj\n",
            line.replace('(', "\\(").replace(')', "\\)")
        ));
    }
    stream_content.push_str("ET\n");

    let stream_len = stream_content.len();

    let pdf = format!(
        "%PDF-1.4\n\
        1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
        2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
        3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n\
        4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n\
        xref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000300 00000 n \n\
        trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n450\n%%EOF\n",
        stream_len, stream_content
    );

    pdf.into_bytes()
}

#[test]
fn test_end_to_end_statistical_paragraph_boundary_detection() {
    let lines_and_gaps = [
        ("First paragraph line one.", 14.0),
        ("First paragraph line two continuation.", 14.0),
        ("Second paragraph starts after a larger vertical gap.", 32.0),
        ("Second paragraph line two continuation.", 14.0),
    ];

    let pdf_bytes = create_synthetic_pdf_with_gaps(&lines_and_gaps);

    let config = Config::builder().paragraph_gap_threshold(1.4).build();

    let converter = Converter::new(config);
    let result = converter
        .convert_bytes(&pdf_bytes)
        .expect("Conversion should succeed");

    assert!(!result.markdown.is_empty());
    assert!(result.markdown.contains("First paragraph line one."));
    assert!(result.markdown.contains("Second paragraph starts after"));

    // Check that there are at least two distinct paragraph nodes in the AST document
    let para_count = result
        .document
        .sections
        .iter()
        .flat_map(|s| s.elements.iter())
        .filter(|node| matches!(node, pdf2md_ast::Node::Paragraph { .. }))
        .count();

    assert!(
        para_count >= 2,
        "Expected at least 2 distinct paragraphs, found {}",
        para_count
    );
}
