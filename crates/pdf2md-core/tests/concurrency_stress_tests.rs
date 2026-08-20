use std::sync::Arc;
use std::thread;
use std::time::Instant;
use pdf2md_core::{Config, Converter, ExecutionProfile};

#[test]
fn test_high_concurrency_stress_100_threads() {
    let pdf_content = "BT\n/F1 16 Tf\n72 700 Td\n(Concurrent Stress Test Document) Tj\n0 -25 Td\n/F1 11 Tf\n(Validating multi-threaded safety, zero data races, and bounded heap allocations.) Tj\nET\n";
    let len = pdf_content.len();
    let pdf = format!(
        "%PDF-1.4\n\
        1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
        2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
        3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n\
        4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n\
        xref\n0 5\n0000000000 65535 f \n\
        trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n300\n%%EOF\n",
        len, pdf_content
    ).into_bytes();

    let config = Config::builder().profile(ExecutionProfile::Fast).build();
    let converter = Arc::new(Converter::new(config));
    let pdf_bytes = Arc::new(pdf);

    let start_time = Instant::now();
    let concurrency_count = 100;
    let mut handles = Vec::with_capacity(concurrency_count);

    for i in 0..concurrency_count {
        let conv_clone = Arc::clone(&converter);
        let bytes_clone = Arc::clone(&pdf_bytes);

        let handle = thread::spawn(move || {
            let res = conv_clone.convert_bytes(&bytes_clone);
            assert!(res.is_ok(), "Thread {} failed conversion", i);
            let conv_res = res.unwrap();
            assert!(conv_res.markdown.contains("Concurrent Stress Test Document"));
            assert_eq!(conv_res.diagnostics.total_pages, 1);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread joined successfully");
    }

    let elapsed = start_time.elapsed();
    println!("\n=== Concurrency Stress Test ===");
    println!("Completed 100 concurrent conversions in {:.2?}", elapsed);
    println!("Average latency per concurrent request: {:.2?}", elapsed / (concurrency_count as u32));
    println!("Throughput: {:.1} docs/sec", (concurrency_count as f64) / elapsed.as_secs_f64());
    println!("===============================\n");
}
