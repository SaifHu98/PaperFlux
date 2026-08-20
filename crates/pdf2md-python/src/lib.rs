#![allow(clippy::useless_conversion, clippy::too_many_arguments)]

use pdf2md_core::{Config, Converter, ExecutionProfile};
use pdf2md_markdown::MarkdownDialect;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use std::fs;
use std::path::PathBuf;

#[pyclass]
#[derive(Clone)]
pub struct ConversionResult {
    #[pyo3(get)]
    pub markdown: String,
    #[pyo3(get)]
    pub confidence: f32,
    #[pyo3(get)]
    pub total_pages: usize,
    #[pyo3(get)]
    pub text_pages: usize,
    #[pyo3(get)]
    pub ocr_pages: usize,
    #[pyo3(get)]
    pub tables_detected: usize,
    #[pyo3(get)]
    pub images_extracted: usize,
    diagnostics_json_str: String,
}

#[pymethods]
impl ConversionResult {
    pub fn diagnostics_json(&self) -> String {
        self.diagnostics_json_str.clone()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "<ConversionResult total_pages={} confidence={:.2} tables={} images={}>",
            self.total_pages, self.confidence, self.tables_detected, self.images_extracted
        )
    }

    pub fn __str__(&self) -> String {
        self.markdown.clone()
    }
}

fn parse_dialect(dialect: &str) -> PyResult<MarkdownDialect> {
    match dialect.to_lowercase().as_str() {
        "gfm" | "github" | "githubflavored" => Ok(MarkdownDialect::GitHubFlavored),
        "commonmark" => Ok(MarkdownDialect::CommonMark),
        "extended" | "scholarly" | "obsidian" | "hugo" => Ok(MarkdownDialect::Extended),
        _ => Err(PyValueError::new_err(format!(
            "Unknown markdown dialect: {}",
            dialect
        ))),
    }
}

fn parse_profile(profile: &str) -> PyResult<ExecutionProfile> {
    match profile.to_lowercase().as_str() {
        "fast" => Ok(ExecutionProfile::Fast),
        "balanced" => Ok(ExecutionProfile::Balanced),
        "low_memory" | "lowmemory" => Ok(ExecutionProfile::LowMemory),
        _ => Err(PyValueError::new_err(format!(
            "Unknown execution profile: {}",
            profile
        ))),
    }
}

#[pyfunction]
#[pyo3(signature = (
    pdf_path,
    dialect = "gfm",
    detect_tables = true,
    extract_images = false,
    images_dir = None,
    profile = "balanced",
    paragraph_gap_threshold = None,
    ocr_dpi = None
))]
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
pub fn convert(
    pdf_path: &str,
    dialect: &str,
    detect_tables: bool,
    extract_images: bool,
    images_dir: Option<&str>,
    profile: &str,
    paragraph_gap_threshold: Option<f32>,
    ocr_dpi: Option<u32>,
) -> PyResult<ConversionResult> {
    let bytes = fs::read(pdf_path).map_err(|e| {
        PyIOError::new_err(format!("Failed to read PDF file '{}': {}", pdf_path, e))
    })?;
    convert_bytes(
        &bytes,
        dialect,
        detect_tables,
        extract_images,
        images_dir,
        profile,
        paragraph_gap_threshold,
        ocr_dpi,
    )
}

#[pyfunction]
#[pyo3(signature = (
    data,
    dialect = "gfm",
    detect_tables = true,
    extract_images = false,
    images_dir = None,
    profile = "balanced",
    paragraph_gap_threshold = None,
    ocr_dpi = None
))]
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
pub fn convert_bytes(
    data: &[u8],
    dialect: &str,
    detect_tables: bool,
    extract_images: bool,
    images_dir: Option<&str>,
    profile: &str,
    paragraph_gap_threshold: Option<f32>,
    ocr_dpi: Option<u32>,
) -> PyResult<ConversionResult> {
    let md_dialect = parse_dialect(dialect)?;
    let exec_profile = parse_profile(profile)?;

    let mut builder = Config::builder()
        .dialect(md_dialect)
        .profile(exec_profile)
        .detect_tables(detect_tables)
        .extract_images(extract_images);

    if let Some(dir) = images_dir {
        builder = builder.images_dir(PathBuf::from(dir));
    }
    if let Some(threshold) = paragraph_gap_threshold {
        builder = builder.paragraph_gap_threshold(threshold);
    }
    if let Some(dpi) = ocr_dpi {
        builder = builder.ocr_dpi(dpi);
    }

    let config = builder.build();
    let converter = Converter::new(config);

    let res = converter
        .convert_bytes(data)
        .map_err(|e| PyValueError::new_err(format!("Conversion failed: {}", e)))?;

    let diag_json = serde_json::to_string_pretty(&res.diagnostics).unwrap_or_default();

    Ok(ConversionResult {
        markdown: res.markdown,
        confidence: res.diagnostics.overall_confidence,
        total_pages: res.diagnostics.total_pages,
        text_pages: res.diagnostics.text_pages,
        ocr_pages: res.diagnostics.ocr_pages,
        tables_detected: res.diagnostics.tables_detected,
        images_extracted: res.diagnostics.images_extracted,
        diagnostics_json_str: diag_json,
    })
}

#[pymodule]
fn paperflux(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ConversionResult>()?;
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    m.add_function(wrap_pyfunction!(convert_bytes, m)?)?;
    Ok(())
}
