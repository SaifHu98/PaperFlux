pub mod metrics;
pub mod normalizer;
pub mod report;

pub use metrics::{DetailedMetric, MetricsCalculator};
pub use normalizer::{NormalizationOptions, TextNormalizer};
pub use report::{CorpusEvaluationReport, EvaluationMetrics, FileEvaluationResult};

use pdf2md_core::{Config, Converter};
use std::fs;
use std::path::Path;

pub struct Evaluator {
    pub norm_opts: NormalizationOptions,
    pub max_cer_threshold: f64,
    pub max_wer_threshold: f64,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            norm_opts: NormalizationOptions::default(),
            max_cer_threshold: 0.05, // 5% max CER
            max_wer_threshold: 0.10, // 10% max WER
        }
    }
}

impl Evaluator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_thresholds(mut self, max_cer: f64, max_wer: f64) -> Self {
        self.max_cer_threshold = max_cer;
        self.max_wer_threshold = max_wer;
        self
    }

    pub fn with_normalization(mut self, opts: NormalizationOptions) -> Self {
        self.norm_opts = opts;
        self
    }

    /// Evaluates two strings directly
    pub fn evaluate_texts(&self, reference: &str, hypothesis: &str) -> EvaluationMetrics {
        let norm_ref = TextNormalizer::normalize(reference, &self.norm_opts);
        let norm_hyp = TextNormalizer::normalize(hypothesis, &self.norm_opts);

        let cer = MetricsCalculator::compute_cer(&norm_ref, &norm_hyp);
        let wer = MetricsCalculator::compute_wer(&norm_ref, &norm_hyp);

        EvaluationMetrics { cer, wer }
    }

    /// Evaluates a PDF against a .md.gold ground-truth file
    pub fn evaluate_pdf_against_gold_file(
        &self,
        pdf_path: &Path,
        gold_path: &Path,
    ) -> Result<FileEvaluationResult, String> {
        let pdf_bytes = fs::read(pdf_path)
            .map_err(|e| format!("Failed to read PDF '{}': {}", pdf_path.display(), e))?;
        let gold_content = fs::read_to_string(gold_path).map_err(|e| {
            format!(
                "Failed to read gold ground-truth '{}': {}",
                gold_path.display(),
                e
            )
        })?;

        let config = Config::builder().detect_tables(true).build();
        let converter = Converter::new(config);
        let result = converter
            .convert_bytes(&pdf_bytes)
            .map_err(|e| format!("Conversion failed for '{}': {}", pdf_path.display(), e))?;

        let metrics = self.evaluate_texts(&gold_content, &result.markdown);
        let passed = metrics.cer.error_rate <= self.max_cer_threshold;

        let filename = pdf_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.pdf".to_string());

        Ok(FileEvaluationResult {
            filename,
            pdf_path: pdf_path.to_path_buf(),
            gold_path: gold_path.to_path_buf(),
            metrics,
            passed,
        })
    }

    /// Discovers all fixtures in a directory and evaluates them
    pub fn evaluate_corpus_dir(
        &self,
        fixtures_dir: &Path,
    ) -> Result<CorpusEvaluationReport, String> {
        let entries = fs::read_dir(fixtures_dir).map_err(|e| {
            format!(
                "Failed to read fixtures dir '{}': {}",
                fixtures_dir.display(),
                e
            )
        })?;

        let mut results = Vec::new();

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("pdf") {
                let gold_path = path.with_extension("md.gold");
                if gold_path.exists() {
                    let file_res = self.evaluate_pdf_against_gold_file(&path, &gold_path)?;
                    results.push(file_res);
                }
            }
        }

        results.sort_by(|a, b| a.filename.cmp(&b.filename));

        if results.is_empty() {
            return Err(format!(
                "No matching .pdf and .md.gold fixture pairs found in '{}'",
                fixtures_dir.display()
            ));
        }

        Ok(CorpusEvaluationReport::new(results))
    }
}
