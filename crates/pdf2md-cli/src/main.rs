use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;
use clap::{Parser, ValueEnum};
use pdf2md_core::{Config, Converter, MarkdownDialect, OcrMode, PageBreakStyle};

#[derive(Parser, Debug)]
#[command(
    name = "pdf2md",
    author = "EcoUni Systems <dev@ecouni.org>",
    version = "0.1.0",
    about = "Converts PDF documents into clean, structurally accurate Markdown"
)]
struct Cli {
    /// Input PDF file path, or "-" to read from standard input
    #[arg(value_name = "INPUT")]
    input: String,

    /// Output Markdown file path, or "-" for standard output
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Markdown dialect to emit
    #[arg(short = 'd', long = "dialect", default_value = "gfm")]
    dialect: DialectArg,

    /// Extract images embedded in the PDF
    #[arg(long = "extract-images")]
    extract_images: bool,

    /// Directory where extracted images will be saved
    #[arg(long = "images-dir")]
    images_dir: Option<PathBuf>,

    /// Disable table detection
    #[arg(long = "no-tables")]
    no_tables: bool,

    /// OCR mode (auto, always, never)
    #[arg(long = "ocr", default_value = "auto")]
    ocr: OcrArg,

    /// Output machine-readable diagnostics JSON to the specified file
    #[arg(long = "diagnostics-json")]
    diagnostics_json: Option<PathBuf>,

    /// Maximum memory limit in MB
    #[arg(long = "memory-limit-mb", default_value = "256")]
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

#[derive(ValueEnum, Clone, Copy, Debug)]
enum DialectArg {
    Commonmark,
    Gfm,
    Extended,
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

    // 1. Read input bytes
    let input_bytes = if cli.input == "-" {
        let mut buffer = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut buffer) {
            eprintln!("Error reading from stdin: {}", e);
            process::exit(1);
        }
        buffer
    } else {
        match fs::read(&cli.input) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", cli.input, e);
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

    let mut builder = Config::builder()
        .dialect(dialect)
        .emit_frontmatter(!cli.no_frontmatter)
        .page_breaks(page_breaks)
        .extract_images(cli.extract_images)
        .detect_tables(!cli.no_tables)
        .ocr_mode(ocr_mode)
        .memory_limit_mb(cli.memory_limit_mb)
        .max_pages(cli.max_pages)
        .timeout_seconds(cli.timeout);

    if let Some(img_dir) = cli.images_dir {
        builder = builder.images_dir(img_dir);
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
