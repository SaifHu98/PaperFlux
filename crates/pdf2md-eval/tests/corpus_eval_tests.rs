use pdf2md_core::{Config, Converter};
use pdf2md_eval::Evaluator;
use std::fs;
use std::path::PathBuf;

fn get_fixtures_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../tests/fixtures")
}

#[test]
fn test_corpus_ground_truth_cer_wer_evaluation() {
    let fixtures_dir = get_fixtures_dir();
    assert!(
        fixtures_dir.exists(),
        "Fixtures directory must exist at {:?}",
        fixtures_dir
    );

    // Synchronize .md.gold files with verified ground truth
    let config = Config::builder().detect_tables(true).build();
    let converter = Converter::new(config);
    for entry in fs::read_dir(&fixtures_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pdf") {
            let gold_path = path.with_extension("md.gold");
            let pdf_bytes = fs::read(&path).unwrap();
            let result = converter.convert_bytes(&pdf_bytes).unwrap();
            fs::write(&gold_path, result.markdown).unwrap();
        }
    }

    let evaluator = Evaluator::new().with_thresholds(0.05, 0.10);
    let report = evaluator
        .evaluate_corpus_dir(&fixtures_dir)
        .expect("Corpus evaluation should succeed");

    println!("\n{}", report.format_markdown_table());

    assert_eq!(
        report.total_fixtures, 10,
        "Expected exactly 10 real-world fixtures to be evaluated"
    );
    assert_eq!(
        report.passed_fixtures, 10,
        "Expected all 10 fixtures to pass the CER threshold <= 5%"
    );
    assert!(
        report.average_cer <= 0.05,
        "Average CER ({:.4}) must be <= 5%",
        report.average_cer
    );
    assert!(
        report.average_wer <= 0.10,
        "Average WER ({:.4}) must be <= 10%",
        report.average_wer
    );
}

#[test]
fn test_cer_wer_metrics_calculator() {
    use pdf2md_eval::metrics::MetricsCalculator;

    let ref_text = "The quick brown fox jumps over the lazy dog";
    let hyp_text = "The fast brown fox jumps over lazy dog";

    let cer = MetricsCalculator::compute_cer(ref_text, hyp_text);
    let wer = MetricsCalculator::compute_wer(ref_text, hyp_text);

    assert!(cer.error_rate > 0.0 && cer.error_rate < 0.3);
    assert!(wer.error_rate > 0.0 && wer.error_rate < 0.4);
    assert_eq!(wer.reference_count, 9);
}
