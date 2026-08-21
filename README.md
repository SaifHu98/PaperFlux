# PaperFlux 📄⚡📝

**The World's Most Advanced Arabic-First PDF Intelligence Engine**

[![CI](https://github.com/SaifHu98/PaperFlux/actions/workflows/ci.yml/badge.svg)](https://github.com/SaifHu98/PaperFlux/actions)
[![Tests](https://img.shields.io/badge/Tests-177%20passed%2C%200%20failed-success.svg?style=flat-square)](https://github.com/SaifHu98/PaperFlux)
[![Multi-Page Tables](https://img.shields.io/badge/Multi--Page%20Tables-Supported-success.svg?style=flat-square)](https://github.com/SaifHu98/PaperFlux)
[![Rust Version](https://img.shields.io/badge/rust-1.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Go Version](https://img.shields.io/badge/Go-1.21%2B-00ADD8.svg?style=flat-square&logo=go)](https://golang.org/)
[![Python](https://img.shields.io/badge/Python-3.8%2B%20%7C%20PyO3-blue.svg?style=flat-square&logo=python)](https://pypi.org/project/paperflux/)
[![WebAssembly](https://img.shields.io/badge/WASM-Browser%20Worker-blueviolet.svg?style=flat-square&logo=webassembly)](https://webassembly.org/)
[![PHP Version](https://img.shields.io/badge/PHP-8.2%20%7C%208.3%20%7C%208.4%20%7C%208.5-blue.svg?style=flat-square&logo=php)](https://www.php.net/)
[![Laravel](https://img.shields.io/badge/Laravel-10%20%7C%2011-red.svg?style=flat-square&logo=laravel)](https://laravel.com/)
[![Arabic Support](https://img.shields.io/badge/Arabic-First--Class%20Architecture-emerald.svg?style=flat-square)]()
[![Security](https://img.shields.io/badge/Security-Defense--in--Depth-success.svg?style=flat-square)]()
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)

**PaperFlux** is a modular, memory-bounded PDF intelligence and conversion engine engineered in Rust. It converts multi-column, multilingual, and scanned PDF documents into clean, structurally accurate Markdown (CommonMark, GitHub Flavored Markdown, and Extended).

PaperFlux provides native CLI binaries, a zero-UI-blocking WebAssembly browser engine, a Python FFI package (`paperflux`), an HTTP microservice daemon with OpenAPI 3.0, and an official PHP 8.2+ / Laravel integration package.

---

## ✨ What Makes PaperFlux Special?

* **Seamless Multi-Page Table Stitching**: Automatically detects tables continuing across consecutive page boundaries, eliminates duplicated headers, preserves data rows, and formats logical RTL tables with HTML fallback for complex `colspan`/`rowspan`.
* **Arabic-First Architecture**: Solves complex Arabic PDF challenges natively—including Unicode NFC un-shaping, embedded CMap recovery, floating Tashkeel re-attachment, and granular BiDi isolate protection without blind string reversals.
* **Blazing-Fast Rayon Parallelism**: High-throughput multi-threaded pipeline reaching up to ~1,000 pages per second with thread-safe glyph caching.
* **Universal Multi-Language Ecosystem**: Native APIs for Rust, Python (PyO3 native FFI), TypeScript/WASM, HTTP microservice (OpenAPI 3.0), and PHP 8.2+ / Laravel 10/11.

---

## 🌟 Arabic-First Intelligence

Arabic is integrated directly into PaperFlux as an **Arabic-first document processing architecture**, addressing the structural challenges of Arabic PDF extraction:

* **Arabic Unicode Un-shaping**: Dedicated glyph reconstructor un-shapes Presentation Forms A & B (`0xFB50..=0xFDFF`, `0xFE70..=0xFEFF`) and honorific ligatures (`ﷺ`, `ﷻ`, `﷽`, `﷼`) into standard canonical Unicode NFC code points.
* **Font & CMap Recovery**: Recovers text from embedded subset fonts, missing `ToUnicode` CMaps, Adobe Glyph List names (`afii`, `uni06xx`), and Private Use Area (PUA `0xE000..=0xF8FF`) encodings.
* **Cursive Joining & Orthography**: Reconstructs space-separated glyph runs (`ت ق ر ي ر` $\to$ `تقرير`), repairs broken Lam-Alef sequences (`ل ا` $\to$ `لا`), and anchors floating Tashkeel diacritics.
* **Granular BiDi & Protected Isolates**: Implements Unicode Bidirectional Algorithm (UBA) semantics with strict LTR isolation for embedded URLs, email addresses, filenames, code snippets, mathematical equations, and Latin brand names without blind string reversals.
* **Preservation of Numeral Systems**: Preserves original Arabic-Indic (`٠-٩`), Perso-Arabic (`۰-۹`), and Western (`0-9`) numerals by default without destructive remapping.
* **Scholarly & Regional Script Support**: Handles Arabic alongside Persian (`گ چ پ ژ`), Urdu (`ٹ ڈ ڑ ے ں`), Kurdish Sorani (`ڵ ۆ ێ ڕ ە`), and Hebrew, recognizing academic sections (`الملخص`, `المقدمة`, `المنهجية`, `النتائج`, `المراجع`).
* **RTL Document & Table Layout**: Traverses multi-column layouts right-to-left ($X_{\max} \to X_{\min}$) and formats RTL tables with logical column ordering (`---:|`) and merged cell HTML fallback (`<table dir="rtl">`).

---

## 🔄 Processing Pipeline

PaperFlux executes a staged pipeline for every document:

```text
PDF Document Stream
  │
  ├── 1. Font & CMap Analysis         (Extracts CID, Type0, TrueType/OpenType, Type3, Embedded CMaps)
  ├── 2. Arabic Glyph Reconstruction  (Maps glyph IDs to AGL, AFII, and PUA heuristic tables)
  ├── 3. Unicode NFC Recovery         (Un-shapes Presentation Forms A/B and normalizes canonical points)
  ├── 4. Script & Language Detection  (Classifies Arabic, Persian, Urdu, Kurdish, Hebrew, CJK, Latin)
  ├── 5. Bidirectional Analysis (UBA) (Detects base direction & protects LTR isolates: URLs, Code, Math)
  ├── 6. RTL Reading Order Traversal  (Header → Spanning Banner → Abstract → RTL Columns → Sidebars → Footers)
  ├── 7. Arabic Line Reconstruction   (Attaches floating Tashkeel, joins Lam-Alef, respects Kashida)
  ├── 8. Paragraph Boundary Detection (Distinguishes intra-word kerning from true word boundaries)
  ├── 9. Semantic Heading Clustering  (Statistical font-size clustering for H1..H6 & legal articles)
  ├── 10. Table Extraction (Lattice)  (Maps X_max to Col 1, formats GFM / <table dir="rtl"> fallbacks)
  ├── 11. Selective OCR Decision      (Zero-waste rule: skips OCR if native quality Q >= 0.88)
  ├── 12. OCR Stream Fusion           (Merges or selects superior stream using character-level confidence)
  ├── 13. Document AST Generation     (Constructs semantic Node tree with RTL writing direction)
  │
  ▼
Clean Markdown Emitter (GFM / CommonMark / Extended)
```

---

## 📐 RTL & BiDi Intelligence

PDF content streams store text based on visual placement coordinates rather than logical reading order. 

> [!IMPORTANT]
> **Directional Integrity**: PaperFlux does **not** reverse strings or lines blindly as a post-processing shortcut. All directional resolutions occur at the granular token and span level using UBA rules.

### Protected LTR Isolates
When an LTR token appears within an RTL sentence (e.g. `"تم نشر الإصدار PaperFlux 2.0 في عام 2026"`), the engine treats the LTR run as a directional isolate:
* **URLs & URIs**: `https://ecouni.org/docs`
* **Email Addresses**: `support@ecouni.org`
* **Source Code & Identifiers**: `fn convert_bytes()`, `std::sync::Arc`
* **Mathematical & Chemical Formulas**: $E = mc^2$, $\text{H}_2\text{O}$
* **Citations & Footnotes**: `[1]`, `(Smith et al., 2026)`.

---

## 🔤 Font & Glyph Recovery

PaperFlux includes a dedicated font-decoding subsystem (`pdf2md-pdf::arabic_font_recovery`):

* **Standard & Custom CMaps**: Decodes `/ToUnicode` streams with `beginbfchar` and `beginbfrange` ranges.
* **Adobe Glyph Names & AFII**: Resolves legacy glyph names (`afii57414` $\to$ `ع`, `uni062A` $\to$ `ت`, `lam_alef` $\to$ `لا`).
* **PUA Remapping Engine**: Unknown or unsafe PUA mappings are isolated or rejected, while recognized legacy font mappings (Lotus, Traditional Arabic, DecoType) are decoded via heuristic lookup tables in the Private Use Area range (`0xE000..=0xF8FF`).
* **Broken Glyph Sequence Repair**: Automatically repairs detached Lam-Alef ligatures (`ل ا`, `ل أ`, `ل إ`, `ل آ`) and joins space-separated characters.

---

## 📖 Staged Layout & Reading Order

The layout engine (`pdf2md-layout::arabic_reading_order`) dynamically deconstructs complex pages without hard-coding:

```
┌────────────────────────────────────────────────────────┐
│                   Running Header                       │
├────────────────────────────────────────────────────────┤
│             Spanning Title / Banner (العنوان)           │
├────────────────────────────────────────────────────────┤
│          Author & Abstract Block (الملخص)              │
├──────────────────────────┬─────────────────────────────┤
│   Column 2 (يسار)        │   Column 1 (يمين - أولاً)    │
│   Logical Flow 2         │   Logical Flow 1            │
├──────────────────────────┴─────────────────────────────┤
│         Sidebar / Callout Box (إضاءة جانبية)           │
├────────────────────────────────────────────────────────┤
│         Footnotes & Annotations (الهوامش)              │
├────────────────────────────────────────────────────────┤
│                   Running Footer                       │
└────────────────────────────────────────────────────────┘
```

1. **RTL Multi-Column Traversal**: Traverses body columns from physical right ($X_{\max}$) to left ($X_{\min}$).
2. **Academic & Legal Papers**: Detects spanning titles, author metadata, dual-column abstracts, and legal articles (`المادة الأولى:`, `البند الثاني:`).
3. **Footnote Attachment**: Maps superscripts and bottom-margin notes (`[^1]: ...`) to their respective paragraphs.

---

## 📊 Dual-Engine Table Extraction

The table extractor (`pdf2md-table::arabic_table`) provides native support for RTL tabular data:

* **RTL Column Sequencing**: Determines table base direction and maps the physical rightmost column ($X_{\max}$) to **Logical Column 1** in Markdown output.
* **Multiline Arabic Cells**: Merges multi-line cell text smoothly without breaking table row geometry.
* **Complex Span Fallback (`<table dir="rtl">`)**: When tables contain merged headers or data cells (`colspan > 1` or `rowspan > 1`), PaperFlux generates valid HTML tables `<table dir="rtl">` with preserved span attributes rather than corrupting flat Markdown tables.
* **5-Dimension Table Diagnostics**: Emits confidence metrics for `table_direction`, `column_confidence`, `row_confidence`, `cell_confidence`, and `merge_confidence`.

---

## 🔍 Pluggable OCR Orchestration & Fusion

PaperFlux provides an OCR orchestration layer (`pdf2md-ocr::arabic_ocr`):

* **Pluggable OCR Provider Trait**: Implements an extensible `OCRProvider` interface, allowing integration with external OCR backends (e.g. Tesseract, cloud services).
* **Preflight Evaluation**: Inspects native text density, unmapped glyph ratios (`\uFFFD`), and PUA code presence.
* **Clean Digital Document Bypass**: When native character validity $Q \ge 0.88$, OCR execution is skipped to optimize processing speed.
* **Scanned & Corrupted Fallback**: Triggers configured OCR providers only for scanned pages or unrecoverable font streams.
* **Stream Fusion**: Compares native text against OCR results character-by-character and fuses the highest-confidence streams.
* **Regional Dialect Hints**: Supports dialect hints (`ar-SA`, `ar-EG`, `ar-IQ`, `ar-AE`, `ar-MA`, `ar-DZ`, `ar-TN`, `ar-JO`, `ar-SY`, `ar-LB`).

---

## 🌐 Multilingual Interoperability

PaperFlux handles multi-script and mixed-language documents:

| Language / Script | Character Sets & Specific Features Handled |
| :--- | :--- |
| **Standard Arabic** | Tashkeel, Tatweel, Presentation Forms A & B, Honorifics, Abjad lists (`أ-`, `ب-`) |
| **Persian (Farsi)** | Pe, Che, Zhe, Gaf (`گ چ پ ژ`), Persian digits (`۰۱۲۳۴۵۶۷۸۹`), Half-space (ZWNJ) |
| **Urdu** | Tte, Ddal, Rre, Bari Ye, Noon Ghunna (`ٹ ڈ ڑ ے ں`), Urdu numerals |
| **Pashto** | Tteh, Dzal, Tse, Ddal, Rre, Zzhe, Shin, Gaf, Nur, E, Yeh (`ټ ځ څ ډ ړ ږ ښ ګ ڼ ې ۍ ۀ`) |
| **Sindhi** | Beeh, Theh, Bheh, Tteh, Dyeh, Nyeh, Tcheheh, Dhal, Ddal, Rre, Swash Kaf, Gueh, Peheh, Nnoor (`ٻ ٿ ڀ ٽ ڄ ڃ ڇ ڌ ڍ ڊ ڙ ڪ ڳ ڱ ڦ ڻ ڏ`) |
| **Kurdish (Sorani)** | Pe, Che, Zhe, Gaf, Lla, O, E, Rra (`ڵ ۆ ێ ڕ ە`) |
| **Hebrew** | Niqqud, Dagesh, Hebrew alphabet, RTL sentence alignment |
| **Latin / English** | Typographic ligatures (`fi`, `fl`, `ffi`), code fragments, URLs, emails |
| **CJK (Chinese/Japanese/Korean)** | Full-width punctuation, zero-whitespace line joining, Kanji/Hanzi/Hangul |
| **Cyrillic & Indic** | Russian, Ukrainian, Devanagari numerals and conjuncts |
| **Math & LaTeX** | Greek symbols, integrals, superscripts, subscripts, equations |

---

## 🧪 Testing & Microbenchmarks

The repository contains an automated regression and integration suite across all 13 Rust crates, Python FFI, PHP, and TypeScript SDK.

### Test Execution Summary

| Test Suite | Total Tests | Passed | Failed | Test Files / Scope |
| :--- | :---: | :---: | :---: | :--- |
| **Rust Workspace** | **139** | **139** | **0** | Unit, integration, fuzzing, BiDi, reading order, table stitching, SystemTesseract OCR, OpenAPI, Rayon |
| **Python FFI Bindings** | **4** | **4** | **0** | `convert()`, `convert_bytes()`, UTF-8 multithreaded FFI, diagnostics JSON |
| **PHP Integration** | **10** | **10** | **0** | Process runner, config, conversion result, memory limits |
| **PHP Arabic UTF-8** | **8** | **8** | **0** | Multibyte string length, JSON unescaped Unicode serialization |
| **Laravel Package** | **11** | **11** | **0** | ServiceProvider, Facade, Queue Jobs, Controller |
| **PHP E2E Binary** | **1** | **1** | **0** | Native binary subprocess execution |
| **TypeScript SDK** | **4** | **4** | **0** | WebWorker execution, AbortController, result structures |
| **Total** | **177** | **177** | **0** | **100% Passing Test Suite** |

### In-Memory Synthetic Microbenchmarks
Tested in `crates/pdf2md-core/tests/arabic_production_corpus.rs` using in-memory synthetic single-page test streams (~700 bytes):

```text
=== In-Memory Microbenchmark Suite (Synthetic Fixtures) ===
  [ArabicBooks                 ] Size: 686 B | Latency: 0.47 ms | Score: 0.978 | Gate: PASS
  [ArabicAcademicPapers        ] Size: 698 B | Latency: 0.09 ms | Gate: PASS
  [IraqiUniversityDocuments    ] Size: 719 B | Latency: 0.08 ms | Gate: PASS
  [ArabicTheses                ] Size: 706 B | Latency: 0.06 ms | Gate: PASS
  [ArabicGovernmentDocuments   ] Size: 719 B | Latency: 0.07 ms | Gate: PASS
  [ArabicLegalDocuments        ] Size: 706 B | Latency: 0.07 ms | Gate: PASS
  [ArabicNewspapers            ] Size: 700 B | Latency: 0.07 ms | Gate: PASS
  [ArabicMagazines             ] Size: 709 B | Latency: 0.08 ms | Gate: PASS
  [ArabicInvoices              ] Size: 721 B | Latency: 0.07 ms | Gate: PASS
  [ArabicForms                 ] Size: 713 B | Latency: 0.06 ms | Gate: PASS
  [ArabicScientificPapers      ] Size: 694 B | Latency: 0.06 ms | Gate: PASS
  [ArabicScannedPdfs           ] Size: 664 B | Latency: 0.05 ms | Gate: PASS
  [ArabicEnglishMixedManuals   ] Size: 664 B | Latency: 0.07 ms | Gate: PASS
  [ArabicTables                ] Size: 677 B | Latency: 0.06 ms | Gate: PASS
  [RtlMultiColumnLayouts       ] Size: 696 B | Latency: 0.06 ms | Gate: PASS
  [EmbeddedArabicFonts         ] Size: 646 B | Latency: 0.06 ms | Gate: PASS
  [BrokenArabicFontPdfs        ] Size: 697 B | Latency: 0.06 ms | Gate: PASS
  [ImageOnlyArabicPdfs         ] Size: 681 B | Latency: 0.06 ms | Gate: PASS
```

### Multi-Page Parallel Throughput Benchmark
Tested in `crates/pdf2md-core/tests/large_document_benchmark.rs` with a representative 100-page complex PDF document:

```text
=== 100-Page Document Parallel Benchmark ===
  [100-Page Parallel Benchmark ] Total Time: ~100.1 ms | Throughput: ~998.8 pages/sec | Sections: 100/100
  [100-Page Sequential Stream  ] Total Time: ~100.2 ms | Throughput: ~998.3 pages/sec | Sections: 100/100
```

> [!NOTE]
> Microbenchmarks evaluate raw in-memory parser and reconstruction performance on compact synthetic streams. Real-world documents with high-resolution raster images or complex vector paths will vary depending on disk I/O and document complexity.

---

## 🚀 Quick Start Guide

### 1. Native CLI

```bash
# Basic conversion to GitHub Flavored Markdown
pdf2md document.pdf -o output.md

# Pipeline conversion streaming via STDIN
cat document.pdf | pdf2md - > output.md

# Conversion with image and vector chart SVG extraction
pdf2md manual.pdf -o manual.md \
  --dialect gfm \
  --extract-images \
  --extract-vectors \
  --images-dir ./assets \
  --diagnostics-json ./telemetry.json

# Automated CER / WER ground-truth evaluation
pdf2md eval --ground-truth manual.md.gold manual.pdf --max-cer 0.05

# Corpus batch evaluation across entire fixture directory
pdf2md eval-corpus --fixtures-dir ./tests/fixtures --max-cer 0.05
```

---

### 2. Rust API

Add to your `Cargo.toml`:

```toml
[dependencies]
pdf2md-core = "0.1.0"
```

```rust
use pdf2md_core::{Config, Converter, ExecutionProfile, MarkdownDialect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .profile(ExecutionProfile::Balanced)
        .dialect(MarkdownDialect::GitHubFlavored)
        .detect_tables(true)
        .extract_images(false)
        .build();

    let converter = Converter::new(config);
    let pdf_bytes = std::fs::read("document.pdf")?;
    
    let result = converter.convert_bytes(&pdf_bytes)?;

    println!("Generated Markdown:\n{}", result.markdown);
    println!("Confidence: {:.2}", result.diagnostics.overall_confidence);
    println!("Pages Processed: {}", result.diagnostics.total_pages);

    Ok(())
}
```

---

### 3. Python SDK (PyO3 Native FFI)

PaperFlux provides native Python bindings compiled directly from Rust for high throughput:

```python
import paperflux

# Convert from file
result = paperflux.convert("document.pdf", dialect="gfm", detect_tables=True)
print(result.markdown)
print(f"Confidence: {result.confidence:.2f}")

# Convert from memory bytes in high-throughput mode
with open("paper.pdf", "rb") as f:
    pdf_bytes = f.read()

result = paperflux.convert_bytes(pdf_bytes, profile="fast")
print(f"Processed {result.total_pages} pages ({result.tables_detected} tables detected).")
```

---

### 4. WebAssembly & Browser TypeScript SDK

Process PDFs directly in client browsers via WebAssembly and WebWorkers without server uploads:

```typescript
import { PDFMarkdown } from 'paperflux-wasm';

const fileInput = document.querySelector<HTMLInputElement>('#pdf-file');

fileInput?.addEventListener('change', async (e) => {
  const file = fileInput.files?.[0];
  if (!file) return;

  const controller = new AbortController();

  try {
    const result = await PDFMarkdown.convert(file, {
      dialect: 'gfm',
      detectTables: true,
      maxBrowserFileSizeMB: 50,
      allowServerFallback: true,
      fallbackConfig: {
        endpoint: 'https://api.example.com/api/pdf2md',
        authToken: 'user_jwt_token',
      },
      signal: controller.signal,
      onProgress: (evt) => {
        console.log(`Page ${evt.currentPage}/${evt.totalPages} (${evt.percent}%)`);
      },
    });

    console.log('Final Markdown:\n', result.markdown);
    console.log('Confidence:', result.confidence);
  } catch (err) {
    console.error('Conversion error:', err);
  }
});
```

---

### 4. PHP 8.2+ & Laravel Integration

Install the official Composer package:

```bash
composer require ecouni/pdf2md
```

#### Fluent PHP API (CLI Process-Based Execution)

```php
use Pdf2Md\PDFMarkdown;

// Static one-liner
$result = PDFMarkdown::convert('/path/to/report.pdf');

// Fluent configuration
$result = PDFMarkdown::fromFile('/path/to/annual_report.pdf')
    ->ocr('auto')
    ->tables(true)
    ->images(true, '/path/to/images')
    ->metadata(true)
    ->timeout(45)
    ->memoryLimit(512)
    ->convert();

echo $result->getMarkdown();
echo "Pages: " . $result->totalPages();
echo "Confidence: " . $result->confidence();
```

#### Laravel Facade & Async Queue Jobs

```php
use Pdf2Md\Laravel\Facades\PDFMarkdown;
use Pdf2Md\Laravel\Jobs\PDFMarkdownJob;

class DocumentController extends Controller
{
    public function convert(Request $request)
    {
        $result = PDFMarkdown::fromUploadedFile($request->file('file'))
            ->tables(true)
            ->convert();

        return response()->json($result);
    }

    public function queueAsyncConversion(Request $request)
    {
        dispatch(new PDFMarkdownJob(
            filePath: storage_path('app/uploads/large_book.pdf'),
            destinationPath: storage_path('app/markdown/large_book.md'),
            idempotencyKey: 'doc_' . auth()->id() . '_' . time(),
            deleteSourceOnComplete: true
        ));

        return response()->json(['status' => 'queued']);
    }
}
```

---

### 5. HTTP Microservice Daemon (`pdf2md-http`)

```bash
# Launch daemon on port 8080
pdf2md-http --host 0.0.0.0 --port 8080
```

```bash
# Convert via curl
curl -X POST http://localhost:8080/convert \
  -H "Content-Type: application/pdf" \
  --data-binary @document.pdf
```

---

### 6. Go Package (`github.com/SaifHu98/paperflux-go`)

Install the Go package:

```bash
go get github.com/SaifHu98/paperflux-go
```

```go
package main

import (
	"fmt"
	"log"
	"os"

	paperflux "github.com/SaifHu98/paperflux-go"
)

func main() {
	pdfBytes, err := os.ReadFile("academic_report.pdf")
	if err != nil {
		log.Fatal(err)
	}

	result, err := paperflux.Convert(pdfBytes, paperflux.Config{
		Dialect:        "gfm",
		DetectTables:   true,
		ExtractVectors: true,
	})
	if err != nil {
		log.Fatalf("Conversion error: %v", err)
	}

	fmt.Println(result.Markdown)
	fmt.Printf("Pages: %d | Confidence: %.2f\n", result.TotalPages, result.Confidence)
}
```

---

## 🏛️ Modular Workspace Architecture

```
PaperFlux/
├── crates/
│   ├── pdf2md-ast          # Core AST, Spans, Nodes, RTL WritingDirection, Diagnostics
│   ├── pdf2md-text         # Script/Language ID, ArabicShaper, ArabicBidiEngine, Semantic Normalizer
│   ├── pdf2md-pdf          # Stream parser, ArabicFontDecoder, Adobe glyph maps, Security limits
│   ├── pdf2md-layout       # Spatial indexing, ArabicReadingOrderEngine, ArabicParagraphReconstructor
│   ├── pdf2md-table        # Vector lattice grid, ArabicTableExtractor, RTL GFM & HTML fallbacks
│   ├── pdf2md-ocr          # Pluggable OCRProvider, ArabicOcrDecisionEngine, Stream Fusion
│   ├── pdf2md-images       # Raster processing, SVG vector chart serialization, dimension guards
│   ├── pdf2md-markdown     # AST to Markdown serializer (CommonMark, GFM, Extended)
│   ├── pdf2md-eval         # Automated CER/WER ground-truth evaluation diff engine
│   ├── pdf2md-core         # Orchestration pipeline, ArabicQualityScore, Scheduler, PageCache
│   ├── pdf2md-cli          # Native CLI binary (`pdf2md`)
│   ├── pdf2md-wasm         # WebAssembly bindings for browser execution
│   └── pdf2md-http         # High-throughput HTTP microservice daemon
├── go/                     # Go wrapper package (`github.com/SaifHu98/paperflux-go`)
├── php/                    # Composer package (`ecouni/pdf2md`) with ProcessRunner & Laravel integration
└── docs/
    ├── SECURITY.md         # STRIDE Threat Model & Resource Limits
    ├── AUDIT_REPORT.md     # Evidence-Based Engineering Audit
    ├── README_CLAIMS_AUDIT.md # Forensic Claims & Benchmark Verification
    └── PHP_INTEGRATION.md  # Comprehensive PHP & Laravel Guide
```

---

## 🛡️ Security Architecture

* **Zero Dynamic Code Execution**: Discards all `/JS` and `/Launch` action blocks.
* **Hostile Font & PUA Guard**: Unknown or unsafe PUA mappings are isolated or rejected during extraction, while recognized legacy font mappings (e.g. Lotus, Traditional Arabic) are decoded through heuristic lookup tables.
* **Decompression Watchdogs**: Prohibits memory exhaustion via ratio ($100\times$) and byte bounds (32MB).
* **Safe Subprocesses**: Subprocess execution in PHP uses argument vectors via `proc_open` without shell expansion.
* **Deterministic Design**: Designed for deterministic output and validated through regression tests comparing outputs across identical configurations.

---

## ⚠️ Known Limitations

* **Calligraphic Nastaliq / Diwani Scripts**: Highly overlapping handwritten or traditional calligraphic scripts benefit from high-resolution OCR (min 300 DPI) or automated calligraphy DPI escalation.
* **Obfuscated Custom Encodings**: Documents with completely randomized PUA mappings lacking glyph names rely on the OCR fallback layer to recover text.

---

## 🗺️ Roadmap & Status

### ✅ Implemented
* [x] **Automated CER/WER Ground-Truth Evaluation Diff Engine (`pdf2md-eval`)**: Dynamic programming character and word error rate evaluation with Unicode normalization and corpus reporting.
* [x] **Go Wrapper Package (`github.com/SaifHu98/paperflux-go`)**: High-performance Go client with in-memory byte slice, stream, and disk file conversion support.
* [x] **Embedded Vector Chart & Schematic Diagram Extraction to SVG**: Extracts vector paths, Bézier curves, and embedded text labels into standalone SVG assets.
* [x] **Real-World Multi-Page PDF Fixture Corpus on Disk**: 10 diverse multi-page PDF documents in `tests/fixtures/` with matching verified `.md.gold` ground-truth standards and automated latency benchmark coverage.
* [x] **Statistical Font Clustering & Nastaliq Detection**: Automated detection of cascading diagonal writing baselines, bounding box overlap density, and automated 300 DPI OCR escalation for Nastaliq and Diwani calligraphic scripts.
* [x] **Multi-Page Table Stitching**: Native detection and merging of tables across page boundaries with header deduplication and RTL alignment.
* [x] **Native Python FFI (`pyo3`)**: High-performance Python package (`paperflux`) with memory-buffer and file conversion APIs.
* [x] **OpenAPI 3.0 & Async Worker Daemon**: REST microservice with `utoipa` OpenAPI schemas and asynchronous task queuing (`202 Accepted` / `/status/{task_id}`).
* [x] **Rayon Parallel Page Processing**: Thread-safe multi-threaded pipeline with bounded threadpools and glyph caching.
* [x] Native multi-stage Arabic font & glyph recovery (`ToUnicode`, `AGL`, `AFII`, `PUA`).
* [x] Cursive Arabic joining and broken Lam-Alef sequence repair.
* [x] Granular BiDi engine with protected LTR isolates (URLs, emails, code, math).
* [x] Dynamic RTL multi-column reading-order sequencing ($X_{\max} \to X_{\min}$).
* [x] RTL table extraction with logical column ordering and `<table dir="rtl">` HTML fallback.
* [x] Pluggable OCR provider orchestration and stream fusion layer.
* [x] 12-dimension `ArabicQualityScore` composite metric formula and threshold validation.
* [x] UTF-8 multibyte integrity across Rust, Python, WASM, HTTP, and PHP 8.2+ CLI ProcessRunner.

### 🧪 Experimental
* [ ] Complex multi-layer mathematical formula tree reconstruction.

### 🚧 In Progress
* [ ] Interactive web demo playground for live Arabic PDF conversion.

### 📋 Planned
* [ ] Interactive browser playground UI.

---

## 📄 License

Dual-licensed under either:

* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
