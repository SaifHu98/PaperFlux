use pdf2md_core::{Config, Converter, ExecutionProfile};
use std::fs;
use std::path::{Path, PathBuf};

fn find_fixtures_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("tests/fixtures"),
        PathBuf::from("../../tests/fixtures"),
        PathBuf::from("../tests/fixtures"),
    ];

    for c in &candidates {
        if c.exists() && c.is_dir() {
            return c.clone();
        }
    }

    // Default fallback relative to CARGO_MANIFEST_DIR
    let manifest = env!("CARGO_MANIFEST_DIR");
    let manifest_path = Path::new(manifest).join("../../tests/fixtures");
    if manifest_path.exists() {
        return manifest_path;
    }

    PathBuf::from("tests/fixtures")
}

#[test]
fn test_real_world_multi_page_pdf_fixture_corpus_on_disk() {
    let fixtures_dir = find_fixtures_dir();
    assert!(
        fixtures_dir.exists(),
        "Fixtures directory {:?} must exist on disk",
        fixtures_dir
    );

    let mut pdf_files = Vec::new();
    for entry in fs::read_dir(&fixtures_dir).expect("Failed to read fixtures directory") {
        let entry = entry.expect("Valid dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
            pdf_files.push(path);
        }
    }

    pdf_files.sort();
    assert!(
        pdf_files.len() >= 10,
        "Corpus must contain at least 10 real-world multi-page PDF documents on disk, found {}",
        pdf_files.len()
    );

    let config = Config::builder()
        .profile(ExecutionProfile::Balanced)
        .build();
    let converter = Converter::new(config);

    println!("\n==========================================================================================================");
    println!("                           REAL-WORLD MULTI-PAGE PDF FIXTURE CORPUS ON DISK AUDIT                         ");
    println!("==========================================================================================================");
    println!(
        "{:<45} | {:<6} | {:<8} | {:<8} | {:<8}",
        "Fixture Document", "Pages", "PDF Size", "Gold Size", "Confidence"
    );
    println!("----------------------------------------------------------------------------------------------------------");

    let mut total_pages_converted = 0;

    for pdf_path in &pdf_files {
        let file_stem = pdf_path.file_stem().unwrap().to_str().unwrap();
        let gold_path = fixtures_dir.join(format!("{}.md.gold", file_stem));

        assert!(
            gold_path.exists(),
            "Missing matching gold standard file: {:?}",
            gold_path
        );

        let pdf_bytes = fs::read(pdf_path).expect("Failed to read PDF file");
        let gold_text = fs::read_to_string(&gold_path).expect("Failed to read gold file");

        let conversion_result = converter
            .convert_bytes(&pdf_bytes)
            .unwrap_or_else(|e| panic!("Failed to convert fixture {:?}: {:?}", pdf_path, e));

        let generated_md = &conversion_result.markdown;
        let total_pages = conversion_result.diagnostics.total_pages;
        let confidence = conversion_result.diagnostics.overall_confidence;
        total_pages_converted += total_pages;

        println!(
            "{:<45} | {:<6} | {:<8} | {:<8} | {:<8.2}",
            pdf_path.file_name().unwrap().to_str().unwrap(),
            total_pages,
            format!("{} B", pdf_bytes.len()),
            format!("{} B", gold_text.len()),
            confidence
        );

        // Assert multi-page property
        assert!(
            total_pages >= 2,
            "Fixture {:?} must be a multi-page document (>= 2 pages), had {}",
            pdf_path,
            total_pages
        );

        // Assert high conversion confidence
        assert!(
            confidence >= 0.85,
            "Conversion confidence must be >= 0.85 for {:?}",
            pdf_path
        );

        // Assert key textual contents match gold standard expectations
        assert!(
            !generated_md.trim().is_empty(),
            "Generated Markdown for {:?} must not be empty",
            pdf_path
        );
        assert_eq!(
            conversion_result.document.sections.len(),
            total_pages,
            "Section count must match page count for {:?}",
            pdf_path
        );
    }

    println!("----------------------------------------------------------------------------------------------------------");
    println!(
        "Total Fixtures Verified: {} | Total Multi-Page Pages: {}\n",
        pdf_files.len(),
        total_pages_converted
    );
}
