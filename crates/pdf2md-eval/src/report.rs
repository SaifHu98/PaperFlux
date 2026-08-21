use crate::metrics::DetailedMetric;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetrics {
    pub cer: DetailedMetric,
    pub wer: DetailedMetric,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvaluationResult {
    pub filename: String,
    pub pdf_path: PathBuf,
    pub gold_path: PathBuf,
    pub metrics: EvaluationMetrics,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusEvaluationReport {
    pub results: Vec<FileEvaluationResult>,
    pub average_cer: f64,
    pub average_wer: f64,
    pub total_fixtures: usize,
    pub passed_fixtures: usize,
    pub all_passed: bool,
}

impl CorpusEvaluationReport {
    pub fn new(results: Vec<FileEvaluationResult>) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let (total_cer, total_wer) = results.iter().fold((0.0, 0.0), |(c_acc, w_acc), r| {
            (
                c_acc + r.metrics.cer.error_rate,
                w_acc + r.metrics.wer.error_rate,
            )
        });

        let avg_cer = if total > 0 {
            total_cer / (total as f64)
        } else {
            0.0
        };
        let avg_wer = if total > 0 {
            total_wer / (total as f64)
        } else {
            0.0
        };
        let all_passed = passed == total && total > 0;

        Self {
            results,
            average_cer: avg_cer,
            average_wer: avg_wer,
            total_fixtures: total,
            passed_fixtures: passed,
            all_passed,
        }
    }

    pub fn format_markdown_table(&self) -> String {
        let mut out = String::new();
        out.push_str("### 📊 Automated Evaluation Report (CER / WER)\n\n");
        out.push_str(
            "| Fixture Document | CER (%) | WER (%) | Ref Chars | Errors (S/D/I) | Status |\n",
        );
        out.push_str("| :--- | :---: | :---: | :---: | :---: | :---: |\n");

        for r in &self.results {
            let status = if r.passed { "✅ PASS" } else { "❌ FAIL" };
            let cer_pct = r.metrics.cer.error_rate * 100.0;
            let wer_pct = r.metrics.wer.error_rate * 100.0;
            let sdi = format!(
                "{}/{}/{}",
                r.metrics.cer.substitutions, r.metrics.cer.deletions, r.metrics.cer.insertions
            );

            out.push_str(&format!(
                "| `{}` | **{:.2}%** | **{:.2}%** | {} | {} | {} |\n",
                r.filename, cer_pct, wer_pct, r.metrics.cer.reference_count, sdi, status
            ));
        }

        out.push_str(&format!(
            "\n**Summary**: Average CER: **{:.2}%** | Average WER: **{:.2}%** | Fixtures Passed: **{}/{}**\n",
            self.average_cer * 100.0,
            self.average_wer * 100.0,
            self.passed_fixtures,
            self.total_fixtures
        ));

        out
    }
}
