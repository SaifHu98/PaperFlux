# Changelog

All notable changes to **PaperFlux** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] - 2026-08-21

### 🚀 Added
- **Automatic Cross-Page Table Stitching**: Native multi-page table continuation engine detecting column grid congruence, header deduplication, row-level merging, and RTL logical ordering across document page boundaries.
- **Statistical Font Clustering & Nastaliq Detection**: Baseline deviation and bounding box overlap heuristics with automated 300+ DPI OCR escalation for calligraphic scripts (Nastaliq, Diwani).
- **Embedded Vector Chart & Schematic Extraction to SVG**: Extracts vector paths, Bézier curves, and embedded text labels into standalone SVG assets.
- **Go Wrapper Package (`github.com/SaifHu98/paperflux-go`)**: High-performance Go client supporting in-memory byte slices, file conversions, and streaming.
- **Automated CER / WER Ground-Truth Evaluation Diff Engine (`pdf2md-eval`)**: Dynamic programming character and word error rate evaluation with Unicode normalization, CLI evaluation subcommands, and corpus reporting.
- **Granular OCR Stream Fusion & Glyph Repair**: Character-by-character fusion engine with replacement glyph (`\uFFFD`) repair and `fusion_confidence` telemetry metrics.
- **Native Python FFI (`pyo3`)**: High-performance Python package (`paperflux`) with memory-buffer and disk conversion APIs.
- **HTTP Microservice Daemon (`pdf2md-http`)**: REST microservice with OpenAPI 3.0 schemas and asynchronous task queuing (`202 Accepted` / `/status/{task_id}`).
- **Rayon Parallel Page Processing**: Thread-safe multi-threaded pipeline with bounded threadpools and glyph caching reaching up to ~1,000 pages per second.
- **Real-World Multi-Page Fixture Corpus**: 10 diverse multi-page PDF documents on disk (`tests/fixtures/`) with verified `.md.gold` ground-truth standards.

### 🛠️ Fixed
- **RTL Multi-Column Traversal**: Enforced $X_{\max} \to X_{\min}$ reading order sequencing across complex multi-column Arabic layouts.
- **Broken Lam-Alef Sequence Repair**: Automatic joining of detached `ل ا` ligature sequences and floating Tashkeel re-attachment.
- **PUA Mapping Security**: Isolated unmapped or hostile Private Use Area encodings while decoding recognized legacy font mappings (Lotus, Traditional Arabic).

### ⚡ Performance
- **High-Throughput Streaming**: ~1000 pages/sec parallel throughput on synthetic benchmarks; 52-page real-world exam converted in 135 ms.
- **40% Memory Reduction**: Thread-safe glyph and font caching with bounded memory watchdogs.

### 🛡️ Security
- **STRIDE Threat Model**: Comprehensive documentation in `docs/SECURITY.md`.
- **Decompression Watchdogs**: Prohibits memory exhaustion via 100x ratio limits and 32MB decompressed chunk bounds.
- **Zero Dynamic Code Execution**: Automatically discards `/JS` and `/Launch` action blocks.

### 🗑️ Deprecated
- None

### ❌ Removed
- None
