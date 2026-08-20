use pdf2md_core::{Config, Converter};

fn generate_multi_page_table_pdf() -> Vec<u8> {
    // Page 1: Arabic Financial Table (Header + Row 1 + Row 2)
    let page1_stream = "\
BT\n\
/F1 14 Tf\n\
72 720 Td\n\
(تقرير مالي سنوي) Tj\n\
0 -30 Td\n\
/F1 11 Tf\n\
(التاريخ | البيان | المبلغ) Tj\n\
0 -20 Td\n\
(2026/01/01 | مبيعات الربع الأول | 50,000) Tj\n\
0 -20 Td\n\
(2026/04/01 | مبيعات الربع الثاني | 75,000) Tj\n\
ET\n";

    // Page 2: Repeated Header + Row 3 + Row 4
    let page2_stream = "\
BT\n\
/F1 11 Tf\n\
72 720 Td\n\
(التاريخ | البيان | المبلغ) Tj\n\
0 -20 Td\n\
(2026/07/01 | مبيعات الربع الثالث | 90,000) Tj\n\
0 -20 Td\n\
(2026/10/01 | مبيعات الربع الرابع | 110,000) Tj\n\
ET\n";

    let len1 = page1_stream.len();
    let len2 = page2_stream.len();

    format!(
        "%PDF-1.4\n\
        1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
        2 0 obj\n<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>\nendobj\n\
        3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n\
        4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n\
        5 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 6 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n\
        6 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n\
        xref\n0 7\n0000000000 65535 f \n\
        trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n500\n%%EOF\n",
        len1, page1_stream, len2, page2_stream
    ).into_bytes()
}

#[test]
fn test_e2e_arabic_financial_table_cross_page_stitching() {
    let config = Config::builder().detect_tables(true).build();
    let converter = Converter::new(config);
    let pdf_bytes = generate_multi_page_table_pdf();

    let result = converter
        .convert_bytes(&pdf_bytes)
        .expect("Conversion should succeed");

    assert_eq!(result.diagnostics.total_pages, 2);
    assert!(result.markdown.contains("تقرير مالي"));
    assert!(result.markdown.contains("مبيعات الربع الأول"));
    assert!(result.markdown.contains("مبيعات الربع الرابع"));
}
