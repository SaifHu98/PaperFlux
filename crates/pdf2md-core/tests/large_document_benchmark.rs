use std::time::Instant;
use pdf2md_core::{Config, Converter, ExecutionProfile};

/// Generates a valid multi-page PDF document with `num_pages` pages
fn generate_large_multipage_pdf(num_pages: usize) -> Vec<u8> {
    let mut page_objs = Vec::new();
    let mut kids_refs = Vec::new();
    let mut contents_objs = Vec::new();

    // Base object numbers:
    // 1: Catalog
    // 2: Pages tree
    // Pages will be: 3 + i * 2 (Page obj), 4 + i * 2 (Contents obj)
    for i in 0..num_pages {
        let page_obj_id = 3 + i * 2;
        let contents_obj_id = 4 + i * 2;
        kids_refs.push(format!("{} 0 R", page_obj_id));

        let page_text = format!(
            "BT\n/F1 14 Tf\n72 720 Td\n(Section Header for Page {}) Tj\n/F1 12 Tf\n0 -24 Td\n(This is paragraph one of page {} with scientific analysis.) Tj\n0 -18 Td\n(Arabic text sample: \\xd9\\x87\\xd8\\xb0\\xd8\\xa7 \\xd9\\x86\\xd8\\xb5 \\xd8\\xb9\\xd8\\xb1\\xd8\\xa8\\xd9\\x8a \\xd9\\x84\\xd9\\x84\\xd8\\xb5\\xd9\\x81\\xd8\\xad\\xd8\\xa9 {}) Tj\nET\n",
            i + 1, i + 1, i + 1
        );

        let stream_len = page_text.len();
        let contents_obj = format!(
            "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            contents_obj_id, stream_len, page_text
        );
        contents_objs.push(contents_obj);

        let page_obj = format!(
            "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents {} 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n",
            page_obj_id, contents_obj_id
        );
        page_objs.push(page_obj);
    }

    let mut pdf = String::new();
    pdf.push_str("%PDF-1.4\n");
    pdf.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    pdf.push_str(&format!(
        "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
        kids_refs.join(" "),
        num_pages
    ));

    for i in 0..num_pages {
        pdf.push_str(&page_objs[i]);
        pdf.push_str(&contents_objs[i]);
    }

    pdf.push_str("xref\n0 1\n0000000000 65535 f \n");
    pdf.push_str("trailer\n<< /Size ");
    pdf.push_str(&(3 + num_pages * 2).to_string());
    pdf.push_str(" /Root 1 0 R >>\nstartxref\n500\n%%EOF\n");

    pdf.into_bytes()
}

#[test]
fn test_large_100_page_pdf_parallel_conversion_benchmark() {
    let num_pages = 100;
    let pdf_bytes = generate_large_multipage_pdf(num_pages);
    assert!(!pdf_bytes.is_empty());

    // 1. Convert with Fast profile (Rayon parallel processing enabled)
    let parallel_config = Config::builder()
        .profile(ExecutionProfile::Fast)
        .build();
    let converter = Converter::new(parallel_config);

    let start_parallel = Instant::now();
    let result_parallel = converter.convert_bytes(&pdf_bytes).expect("Parallel 100-page conversion should succeed");
    let parallel_duration = start_parallel.elapsed();

    println!(
        "\n⚡ 100-Page Parallel Benchmark Time: {:?} ({:.2} pages/sec)",
        parallel_duration,
        num_pages as f64 / parallel_duration.as_secs_f64()
    );

    assert_eq!(result_parallel.diagnostics.total_pages, num_pages);
    assert_eq!(result_parallel.document.sections.len(), num_pages);
    assert!(result_parallel.markdown.contains("Section Header for Page 1"));
    assert!(result_parallel.markdown.contains("Section Header for Page 50"));
    assert!(result_parallel.markdown.contains("Section Header for Page 100"));

    // 2. Convert with LowMemory profile (Sequential processing)
    let seq_config = Config::builder()
        .profile(ExecutionProfile::LowMemory)
        .build();
    let seq_converter = Converter::new(seq_config);

    let start_seq = Instant::now();
    let result_seq = seq_converter.convert_bytes(&pdf_bytes).expect("Sequential 100-page conversion should succeed");
    let seq_duration = start_seq.elapsed();

    println!(
        "🐢 100-Page Sequential Benchmark Time: {:?} ({:.2} pages/sec)",
        seq_duration,
        num_pages as f64 / seq_duration.as_secs_f64()
    );

    assert_eq!(result_seq.diagnostics.total_pages, num_pages);
    assert_eq!(result_seq.document.sections.len(), num_pages);
    assert_eq!(result_parallel.document.sections.len(), result_seq.document.sections.len());
}
