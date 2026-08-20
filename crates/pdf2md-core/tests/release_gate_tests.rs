use pdf2md_core::{Config, Converter, ExecutionProfile};

fn create_sample_pdf() -> Vec<u8> {
    let content = "BT\n/F1 16 Tf\n72 700 Td\n(Release Gate Validation Document) Tj\n0 -25 Td\n/F1 11 Tf\n(Deterministic, secure, and production-ready.) Tj\nET\n";
    let len = content.len();
    format!(
        "%PDF-1.4\n\
        1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
        2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
        3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n\
        4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n\
        xref\n0 5\n0000000000 65535 f \n\
        trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n300\n%%EOF\n",
        len, content
    ).into_bytes()
}

#[test]
fn test_release_gate_determinism() {
    let pdf_bytes = create_sample_pdf();
    let config = Config::builder().deterministic(true).build();
    let converter = Converter::new(config);

    let res1 = converter.convert_bytes(&pdf_bytes).unwrap();
    let res2 = converter.convert_bytes(&pdf_bytes).unwrap();
    let res3 = converter.convert_bytes(&pdf_bytes).unwrap();

    assert_eq!(res1.markdown, res2.markdown, "Output must be strictly deterministic across run 1 & 2");
    assert_eq!(res2.markdown, res3.markdown, "Output must be strictly deterministic across run 2 & 3");
    assert_eq!(res1.diagnostics.overall_confidence, res2.diagnostics.overall_confidence);
}

#[test]
fn test_release_gate_profiles_validation() {
    let pdf_bytes = create_sample_pdf();

    for profile in [ExecutionProfile::Fast, ExecutionProfile::Balanced, ExecutionProfile::LowMemory] {
        let config = Config::builder().profile(profile).build();
        let converter = Converter::new(config);
        let res = converter.convert_bytes(&pdf_bytes);
        assert!(res.is_ok(), "Profile {:?} must succeed", profile);
        let output = res.unwrap();
        assert!(output.markdown.contains("Release Gate Validation Document"));
    }
}

#[test]
fn test_release_gate_resource_limits_enforced() {
    let pdf_bytes = create_sample_pdf();

    // Set page limit to 0 to test enforcement
    let config = Config::builder().max_pages(0).build();
    let converter = Converter::new(config);

    let res = converter.convert_bytes(&pdf_bytes);
    assert!(res.is_err(), "Exceeding max_pages limit must return Err");
}
