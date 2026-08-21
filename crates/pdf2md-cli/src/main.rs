use clap::{Parser, Subcommand, ValueEnum};
use pdf2md_core::{Config, Converter, MarkdownDialect, OcrMode, PageBreakStyle};
use pdf2md_eval::Evaluator;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(
    name = "pdf2md",
    author = "EcoUni Systems <dev@ecouni.org>",
    version = "1.0.0",
    about = "Converts PDF documents into clean, structurally accurate Markdown"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input PDF file path, or "-" to read from standard input
    #[arg(value_name = "INPUT")]
    input: Option<String>,

    /// Output Markdown file path, or "-" for standard output
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Markdown dialect to emit
    #[arg(short = 'd', long = "dialect", default_value = "gfm")]
    dialect: DialectArg,

    /// Execution profile (fast, balanced, quality)
    #[arg(long = "profile", default_value = "balanced")]
    profile: ProfileArg,

    /// Extract images embedded in the PDF
    #[arg(long = "extract-images")]
    extract_images: bool,

    /// Extract vector charts and schematic diagrams to SVG
    #[arg(long = "extract-vectors")]
    extract_vectors: bool,

    /// Directory where extracted images will be saved
    #[arg(long = "images-dir")]
    images_dir: Option<PathBuf>,

    /// Detect tables (true/false)
    #[arg(long = "detect-tables", default_missing_value = "true", num_args = 0..=1)]
    detect_tables: Option<bool>,

    /// Disable table detection
    #[arg(long = "no-tables")]
    no_tables: bool,

    /// Parallel multi-page processing
    #[arg(long = "parallel-pages", default_missing_value = "true", num_args = 0..=1)]
    parallel_pages: Option<bool>,

    /// OCR mode (auto, always, never)
    #[arg(long = "ocr", default_value = "auto")]
    ocr: OcrArg,

    /// OCR provider (auto, tesseract, mock)
    #[arg(long = "ocr-provider", default_value = "auto")]
    ocr_provider: String,

    /// OCR languages to recognize (e.g. ara+eng, ara, fas, urd)
    #[arg(long = "ocr-lang", default_value = "ara+eng")]
    ocr_lang: String,

    /// Custom path to OCR binary (e.g. path to tesseract executable)
    #[arg(long = "ocr-binary")]
    ocr_binary: Option<PathBuf>,

    /// Output machine-readable diagnostics JSON to the specified file
    #[arg(long = "diagnostics-json")]
    diagnostics_json: Option<PathBuf>,

    /// Maximum memory limit in MB
    #[arg(long = "memory-limit", alias = "memory-limit-mb", default_value = "256")]
    memory_limit_mb: usize,

    /// Maximum number of pages to process
    #[arg(long = "max-pages", default_value = "5000")]
    max_pages: usize,

    /// Processing timeout in seconds
    #[arg(long = "timeout", default_value = "60")]
    timeout: u64,

    /// Page break style in markdown (none, thematic, html, marker)
    #[arg(long = "page-breaks", default_value = "html")]
    page_breaks: PageBreakArg,

    /// Do not emit YAML frontmatter metadata block
    #[arg(long = "no-frontmatter")]
    no_frontmatter: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Evaluate CER/WER against a ground truth .md file
    Eval {
        /// Ground truth Markdown file path
        #[arg(long = "ground-truth")]
        ground_truth: PathBuf,

        /// Input PDF file to evaluate
        #[arg(value_name = "PDF")]
        pdf: PathBuf,

        /// Maximum allowed CER threshold (e.g. 0.05 for 5%)
        #[arg(long = "max-cer", default_value = "0.05")]
        max_cer: f64,

        /// Maximum allowed WER threshold (e.g. 0.10 for 10%)
        #[arg(long = "max-wer", default_value = "0.10")]
        max_wer: f64,
    },

    /// Evaluate all fixtures in a directory against matching .md.gold files
    EvalCorpus {
        /// Directory containing PDF fixtures and .md.gold ground truths
        #[arg(long = "fixtures-dir", default_value = "tests/fixtures")]
        fixtures_dir: PathBuf,

        /// Maximum allowed average CER threshold
        #[arg(long = "max-cer", default_value = "0.05")]
        max_cer: f64,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum DialectArg {
    Commonmark,
    Gfm,
    Extended,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum ProfileArg {
    Fast,
    Balanced,
    LowMemory,
    Quality,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum OcrArg {
    Auto,
    Always,
    Never,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum PageBreakArg {
    None,
    Thematic,
    Html,
    Marker,
}

fn main() {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Eval {
                ground_truth,
                pdf,
                max_cer,
                max_wer,
            } => {
                let evaluator = Evaluator::new().with_thresholds(max_cer, max_wer);
                match evaluator.evaluate_pdf_against_gold_file(&pdf, &ground_truth) {
                    Ok(res) => {
                        let cer_pct = res.metrics.cer.error_rate * 100.0;
                        let wer_pct = res.metrics.wer.error_rate * 100.0;
                        println!("Evaluation Results for '{}':", res.filename);
                        println!(
                            "  CER: {:.2}% (Sub: {}, Del: {}, Ins: {}, Ref: {})",
                            cer_pct,
                            res.metrics.cer.substitutions,
                            res.metrics.cer.deletions,
                            res.metrics.cer.insertions,
                            res.metrics.cer.reference_count
                        );
                        println!(
                            "  WER: {:.2}% (Sub: {}, Del: {}, Ins: {}, Ref: {})",
                            wer_pct,
                            res.metrics.wer.substitutions,
                            res.metrics.wer.deletions,
                            res.metrics.wer.insertions,
                            res.metrics.wer.reference_count
                        );
                        if res.passed {
                            println!("Status: PASS (CER <= {:.1}%)", max_cer * 100.0);
                            process::exit(0);
                        } else {
                            eprintln!("Status: FAIL (CER > {:.1}%)", max_cer * 100.0);
                            process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("Evaluation error: {}", e);
                        process::exit(1);
                    }
                }
            }
            Commands::EvalCorpus {
                fixtures_dir,
                max_cer,
            } => {
                let evaluator = Evaluator::new().with_thresholds(max_cer, 0.10);
                match evaluator.evaluate_corpus_dir(&fixtures_dir) {
                    Ok(report) => {
                        println!("{}", report.format_markdown_table());
                        if report.all_passed {
                            println!("All fixtures passed CER threshold!");
                            process::exit(0);
                        } else {
                            eprintln!("Some fixtures exceeded CER threshold.");
                            process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("Corpus evaluation error: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
    }

    let input_path = match cli.input {
        Some(p) => p,
        None => {
            eprintln!("Error: No input PDF file specified. Run with --help for usage.");
            process::exit(1);
        }
    };

    // 1. Read input bytes
    let input_bytes = if input_path == "-" {
        let mut buffer = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut buffer) {
            eprintln!("Error reading from stdin: {}", e);
            process::exit(1);
        }
        buffer
    } else {
        match fs::read(&input_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", input_path, e);
                process::exit(1);
            }
        }
    };

    // 2. Build configuration
    let dialect = match cli.dialect {
        DialectArg::Commonmark => MarkdownDialect::CommonMark,
        DialectArg::Gfm => MarkdownDialect::GitHubFlavored,
        DialectArg::Extended => MarkdownDialect::Extended,
    };

    let ocr_mode = match cli.ocr {
        OcrArg::Auto => OcrMode::Auto,
        OcrArg::Always => OcrMode::Always,
        OcrArg::Never => OcrMode::Never,
    };

    let page_breaks = match cli.page_breaks {
        PageBreakArg::None => PageBreakStyle::None,
        PageBreakArg::Thematic => PageBreakStyle::ThematicBreak,
        PageBreakArg::Html => PageBreakStyle::HtmlComment,
        PageBreakArg::Marker => PageBreakStyle::CustomMarker,
    };

    let profile = match cli.profile {
        ProfileArg::Fast => pdf2md_core::ExecutionProfile::Fast,
        ProfileArg::Balanced | ProfileArg::Quality => pdf2md_core::ExecutionProfile::Balanced,
        ProfileArg::LowMemory => pdf2md_core::ExecutionProfile::LowMemory,
    };

    let should_detect_tables = cli.detect_tables.unwrap_or(true) && !cli.no_tables;

    let mut builder = Config::builder()
        .profile(profile)
        .dialect(dialect)
        .emit_frontmatter(!cli.no_frontmatter)
        .page_breaks(page_breaks)
        .extract_images(cli.extract_images)
        .extract_vectors(cli.extract_vectors)
        .detect_tables(should_detect_tables)
        .ocr_mode(ocr_mode)
        .memory_limit_mb(cli.memory_limit_mb)
        .max_pages(cli.max_pages)
        .timeout_seconds(cli.timeout);

    if let Some(img_dir) = cli.images_dir {
        builder = builder.images_dir(img_dir);
    }

    if let Some(ocr_bin) = cli.ocr_binary {
        let prov = std::sync::Arc::new(pdf2md_ocr::SystemTesseractOCRProvider::with_binary(
            ocr_bin,
            &cli.ocr_lang,
        ));
        builder = builder.ocr_provider(prov);
    } else if cli.ocr_provider == "tesseract" || (!cli.ocr_lang.is_empty() && cli.ocr_lang != "ara+eng") {
        let prov = std::sync::Arc::new(pdf2md_ocr::SystemTesseractOCRProvider::with_languages(
            &cli.ocr_lang,
        ));
        builder = builder.ocr_provider(prov);
    } else if cli.ocr_provider == "mock" {
        let prov = std::sync::Arc::new(pdf2md_ocr::MockOCRProvider::new("OCR Recognized Text"));
        builder = builder.ocr_provider(prov);
    }

    let config = builder.build();
    let converter = Converter::new(config);

    // 3. Perform conversion
    let result = match converter.convert_bytes(&input_bytes) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Conversion error: {}", e);
            process::exit(1);
        }
    };

    // 4. Output diagnostics JSON if requested
    if let Some(diag_path) = cli.diagnostics_json {
        if let Ok(json_str) = serde_json::to_string_pretty(&result.diagnostics) {
            let _ = fs::write(diag_path, json_str);
        }
    }

    // 5. Output Markdown
    match cli.output.as_deref() {
        None | Some("-") => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            let _ = handle.write_all(result.markdown.as_bytes());
        }
        Some(out_path) => {
            if let Err(e) = fs::write(out_path, result.markdown) {
                eprintln!("Error writing output to '{}': {}", out_path, e);
                process::exit(1);
            }
        }
    }
}
