use std::time::Instant;
use pdf2md_core::{Config, Converter, ExecutionProfile};

fn generate_benchmark_pdf(page_count: usize) -> Vec<u8> {
    let mut pages_objects = String::new();
    let mut kids = Vec::new();

    let mut obj_id = 3;

    for i in 1..=page_count {
        let page_obj_id = obj_id;
        let content_obj_id = obj_id + 1;
        obj_id += 2;

        kids.push(format!("{} 0 R", page_obj_id));

        let stream_content = format!(
            "BT\n/F1 16 Tf\n72 700 Td\n(Section Page {}) Tj\n0 -25 Td\n/F1 11 Tf\n(Benchmarking high-throughput layout analysis and Markdown rendering performance across multiple execution profiles.) Tj\n0 -18 Td\n(• Feature 1: High speed zero-copy stream processing) Tj\n0 -15 Td\n(• Feature 2: Bounded memory and adaptive scheduling) Tj\nET\n",
            i
        );
        let stream_len = stream_content.len();

        pages_objects.push_str(&format!(
            "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents {} 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n\
            {} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            page_obj_id, content_obj_id, content_obj_id, stream_len, stream_content
        ));
    }

    let kids_str = kids.join(" ");

    let pdf = format!(
        "%PDF-1.4\n\
        1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
        2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n\
        {}\
        xref\n0 {}\n0000000000 65535 f \n\
        trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n500\n%%EOF\n",
        kids_str, page_count, pages_objects, obj_id, obj_id
    );

    pdf.into_bytes()
}

#[test]
fn test_benchmark_throughput_and_profiles() {
    let page_count = 20;
    let pdf_bytes = generate_benchmark_pdf(page_count);
    let size_mb = (pdf_bytes.len() as f64) / (1024.0 * 1024.0);

    println!("\n================ BENCHMARK RESULTS ================");
    println!("Document: {} pages ({:.3} MB)", page_count, size_mb);

    // 1. Benchmark FAST profile
    let fast_config = Config::builder().profile(ExecutionProfile::Fast).build();
    let fast_conv = Converter::new(fast_config);
    let start_fast = Instant::now();
    let fast_res = fast_conv.convert_bytes(&pdf_bytes).unwrap();
    let fast_duration = start_fast.elapsed();
    let fast_pages_per_sec = (page_count as f64) / fast_duration.as_secs_f64();
    let fast_mb_per_sec = size_mb / fast_duration.as_secs_f64();

    println!("\n[PROFILE: FAST]");
    println!("  Total Time:       {:.2?}", fast_duration);
    println!("  Throughput:       {:.1} pages/sec", fast_pages_per_sec);
    println!("  Bandwidth:        {:.2} MB/sec", fast_mb_per_sec);
    println!("  Confidence:       {:.2}", fast_res.diagnostics.overall_confidence);

    // 2. Benchmark BALANCED profile
    let bal_config = Config::builder().profile(ExecutionProfile::Balanced).build();
    let bal_conv = Converter::new(bal_config);
    let start_bal = Instant::now();
    let bal_res = bal_conv.convert_bytes(&pdf_bytes).unwrap();
    let bal_duration = start_bal.elapsed();
    let bal_pages_per_sec = (page_count as f64) / bal_duration.as_secs_f64();

    println!("\n[PROFILE: BALANCED]");
    println!("  Total Time:       {:.2?}", bal_duration);
    println!("  Throughput:       {:.1} pages/sec", bal_pages_per_sec);
    println!("  Confidence:       {:.2}", bal_res.diagnostics.overall_confidence);

    // 3. Benchmark LOW_MEMORY profile
    let low_config = Config::builder().profile(ExecutionProfile::LowMemory).build();
    let low_conv = Converter::new(low_config);
    let start_low = Instant::now();
    let low_res = low_conv.convert_bytes(&pdf_bytes).unwrap();
    let low_duration = start_low.elapsed();
    let low_pages_per_sec = (page_count as f64) / low_duration.as_secs_f64();

    println!("\n[PROFILE: LOW_MEMORY]");
    println!("  Total Time:       {:.2?}", low_duration);
    println!("  Throughput:       {:.1} pages/sec", low_pages_per_sec);
    println!("  Confidence:       {:.2}", low_res.diagnostics.overall_confidence);

    // 4. Benchmark Cache Hit vs Cold Parse
    let start_cache = Instant::now();
    let _ = fast_conv.convert_bytes(&pdf_bytes).unwrap();
    let cache_duration = start_cache.elapsed();

    println!("\n[CACHE PERFORMANCE]");
    println!("  Cold Parse:       {:.2?}", fast_duration);
    println!("  Cache Hit Parse:  {:.2?}", cache_duration);
    println!("===================================================\n");

    assert_eq!(fast_res.diagnostics.total_pages, page_count);
    assert_eq!(bal_res.diagnostics.total_pages, page_count);
    assert_eq!(low_res.diagnostics.total_pages, page_count);
}
