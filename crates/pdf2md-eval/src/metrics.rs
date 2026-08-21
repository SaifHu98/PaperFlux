use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailedMetric {
    pub error_rate: f64,
    pub errors: usize,
    pub substitutions: usize,
    pub deletions: usize,
    pub insertions: usize,
    pub reference_count: usize,
    pub hypothesis_count: usize,
}

pub struct MetricsCalculator;

impl MetricsCalculator {
    /// Computes Character Error Rate (CER) using Levenshtein distance on Unicode characters
    pub fn compute_cer(reference: &str, hypothesis: &str) -> DetailedMetric {
        let ref_chars: Vec<char> = reference.chars().collect();
        let hyp_chars: Vec<char> = hypothesis.chars().collect();

        Self::compute_levenshtein_details(&ref_chars, &hyp_chars)
    }

    /// Computes Word Error Rate (WER) using Levenshtein distance on words
    pub fn compute_wer(reference: &str, hypothesis: &str) -> DetailedMetric {
        let ref_words: Vec<&str> = reference.split_whitespace().collect();
        let hyp_words: Vec<&str> = hypothesis.split_whitespace().collect();

        Self::compute_levenshtein_details(&ref_words, &hyp_words)
    }

    /// Generic Levenshtein distance and edit operation decomposition
    fn compute_levenshtein_details<T: PartialEq>(ref_seq: &[T], hyp_seq: &[T]) -> DetailedMetric {
        let m = ref_seq.len();
        let n = hyp_seq.len();

        if m == 0 {
            let err_rate = if n == 0 { 0.0 } else { 1.0 };
            return DetailedMetric {
                error_rate: err_rate,
                errors: n,
                substitutions: 0,
                deletions: 0,
                insertions: n,
                reference_count: 0,
                hypothesis_count: n,
            };
        }

        if n == 0 {
            return DetailedMetric {
                error_rate: 1.0,
                errors: m,
                substitutions: 0,
                deletions: m,
                insertions: 0,
                reference_count: m,
                hypothesis_count: 0,
            };
        }

        // DP matrix: dp[i][j] = (cost, substitutions, deletions, insertions)
        #[derive(Clone, Copy, Default)]
        struct Cell {
            cost: usize,
            sub: usize,
            del: usize,
            ins: usize,
        }

        let mut prev_row = vec![Cell::default(); n + 1];
        for j in 0..=n {
            prev_row[j] = Cell {
                cost: j,
                sub: 0,
                del: 0,
                ins: j,
            };
        }

        for i in 1..=m {
            let mut curr_row = vec![Cell::default(); n + 1];
            curr_row[0] = Cell {
                cost: i,
                sub: 0,
                del: i,
                ins: 0,
            };

            for j in 1..=n {
                if ref_seq[i - 1] == hyp_seq[j - 1] {
                    curr_row[j] = prev_row[j - 1];
                } else {
                    let sub_cost = prev_row[j - 1].cost + 1;
                    let del_cost = prev_row[j].cost + 1;
                    let ins_cost = curr_row[j - 1].cost + 1;

                    let min_cost = sub_cost.min(del_cost).min(ins_cost);

                    if min_cost == sub_cost {
                        curr_row[j] = Cell {
                            cost: sub_cost,
                            sub: prev_row[j - 1].sub + 1,
                            del: prev_row[j - 1].del,
                            ins: prev_row[j - 1].ins,
                        };
                    } else if min_cost == del_cost {
                        curr_row[j] = Cell {
                            cost: del_cost,
                            sub: prev_row[j].sub,
                            del: prev_row[j].del + 1,
                            ins: prev_row[j].ins,
                        };
                    } else {
                        curr_row[j] = Cell {
                            cost: ins_cost,
                            sub: curr_row[j - 1].sub,
                            del: curr_row[j - 1].del,
                            ins: curr_row[j - 1].ins + 1,
                        };
                    }
                }
            }
            prev_row = curr_row;
        }

        let final_cell = prev_row[n];
        let total_errors = final_cell.cost;
        let error_rate = (total_errors as f64) / (m as f64);

        DetailedMetric {
            error_rate,
            errors: total_errors,
            substitutions: final_cell.sub,
            deletions: final_cell.del,
            insertions: final_cell.ins,
            reference_count: m,
            hypothesis_count: n,
        }
    }
}
