use pdf2md_core::{Config, Converter};

fn create_full_arabic_audit_pdf() -> Vec<u8> {
    let content = "BT\n/F1 18 Tf\n72 700 Td\n(المملكة العربية السعودية) Tj\n0 -25 Td\n/F1 14 Tf\n(عقد استشاري رقم 1024 لعام 2026) Tj\n0 -20 Td\n/F1 12 Tf\n(نظام PaperFlux الذكي) Tj\nET\n";
    let stream_len = content.len();

    let pdf = format!(
        "%PDF-1.4\n\
        1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
        2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
        3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n\
        4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n\
        xref\n0 5\n0000000000 65535 f \n\
        trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n300\n%%EOF\n",
        stream_len, content
    );
    pdf.into_bytes()
}

#[test]
fn test_arabic_end_to_end_pipeline_and_utf8_integrity() {
    let pdf_bytes = create_full_arabic_audit_pdf();
    let converter = Converter::new(Config::default());
    let result = converter.convert_bytes(&pdf_bytes);

    assert!(result.is_ok(), "Conversion failed: {:?}", result.err());
    let conversion_res = result.unwrap();

    let md = conversion_res.markdown;
    println!("DEBUG MD OUTPUT:\n{}", md);
    // 1. Verify exact Arabic Unicode strings without mojibake
    assert!(md.contains("المملكة العربية السعودية"));
    assert!(md.contains("عقد استشاري"));
    assert!(md.contains("PaperFlux"));
    assert!(md.contains("2026"));

    // 2. Verify JSON AST serialization preserves UTF-8
    let json_ast = serde_json::to_string(&conversion_res.document).unwrap();
    assert!(json_ast.contains("المملكة العربية السعودية"));

    // 3. Verify total confidence score is high
    assert!(conversion_res.diagnostics.overall_confidence >= 0.90);
}
