# PHP 8.2+ & Laravel Integration Guide (`ecouni/pdf2md`)

The `ecouni/pdf2md` package provides first-class, memory-efficient, high-performance PDF-to-Markdown conversion for PHP 8.2+, 8.3+, 8.4+, 8.5+, Laravel, and standalone PHP applications.

---

## 1. Installation

```bash
composer require ecouni/pdf2md
```

Ensure the compiled `pdf2md` native binary is placed in your `$PATH` or configure the executable path explicitly.

---

## 2. Fluent PHP API

### Simple Conversion

```php
use Pdf2Md\PDFMarkdown;

// One-liner static conversion
$result = PDFMarkdown::convert('/path/to/document.pdf');

echo $result->getMarkdown();
echo "Pages: " . $result->totalPages();
echo "Confidence: " . $result->confidence();
```

### Fluent Configuration Chaining

```php
use Pdf2Md\PDFMarkdown;

$result = PDFMarkdown::fromFile('/path/to/document.pdf')
    ->ocr('auto')            // 'auto', 'always', 'never'
    ->tables(true)           // Extract vector & borderless tables
    ->images(true, '/path/to/assets') // Extract images to directory
    ->metadata(true)         // Emit YAML frontmatter
    ->timeout(45)            // Seconds
    ->memoryLimit(512)       // MB budget
    ->convert();

echo $result->getMarkdown();

// Access rich diagnostics
$stats = $result->statistics();
$warnings = $result->warnings();
$meta = $result->metadata();
```

### Input Sources

```php
// Local file (Zero PHP memory overhead - path passed directly to native binary)
$pdf = PDFMarkdown::fromFile('/path/to/large_book.pdf');

// Uploaded file (Laravel UploadedFile or PSR-7)
$pdf = PDFMarkdown::fromUploadedFile($request->file('document'));

// In-memory string (safely streamed via STDIN)
$pdf = PDFMarkdown::fromString($rawPdfBinary);

// Stream resource
$stream = fopen('https://example.com/sample.pdf', 'rb');
$pdf = PDFMarkdown::fromStream($stream);
```

---

## 3. Laravel Integration

### Service Provider & Configuration

In Laravel, package discovery registers `Pdf2MdServiceProvider` and the `PDFMarkdown` Facade automatically.

Publish the configuration file:

```bash
php artisan vendor:publish --provider="Pdf2Md\Laravel\Pdf2MdServiceProvider"
```

Configure via `.env`:

```env
PDF2MD_BINARY_PATH=/usr/local/bin/pdf2md
PDF2MD_DIALECT=gfm
PDF2MD_TIMEOUT=60
PDF2MD_MEMORY_LIMIT_MB=256
PDF2MD_OCR_MODE=auto
PDF2MD_DETECT_TABLES=true
```

### Laravel Facade

```php
use Pdf2Md\Laravel\Facades\PDFMarkdown;

class DocumentController extends Controller
{
    public function convert(Request $request)
    {
        $request->validate([
            'file' => 'required|file|mimes:pdf|max:51200', // 50MB
        ]);

        $result = PDFMarkdown::fromUploadedFile($request->file('file'))
            ->tables(true)
            ->convert();

        return response()->json([
            'markdown' => $result->getMarkdown(),
            'confidence' => $result->confidence(),
            'statistics' => $result->statistics(),
        ]);
    }
}
```

### Async Queue Job (`PDFMarkdownJob`)

For processing large documents asynchronously in Laravel background workers:

```php
use Pdf2Md\Laravel\Jobs\PDFMarkdownJob;
use Pdf2Md\Config;

$config = (new Config())
    ->setDialect('gfm')
    ->setTimeoutSeconds(180);

// Dispatch to queue
dispatch(new PDFMarkdownJob(
    filePath: storage_path('app/uploads/annual_report.pdf'),
    destinationPath: storage_path('app/markdown/annual_report.md'),
    config: $config,
    idempotencyKey: 'doc_' . $document->id,
    deleteSourceOnComplete: true
));
```

---

## 4. Remote Microservice Offloading

For high-traffic or containerized environments, point conversions to a remote `pdf2md-http` worker:

```php
use Pdf2Md\PDFMarkdown;

$result = PDFMarkdown::fromFile($path)
    ->httpEndpoint('http://pdf2md-worker.internal:8080')
    ->timeout(60)
    ->convert();
```

---

## 5. Security & Isolation

- **Zero Shell Concatenation**: Executed strictly with array argument vectors via `proc_open`.
- **Low Memory Footprint**: File paths are passed directly to the binary without holding large byte buffers in PHP RAM.
- **Strict Timeouts**: Built-in non-blocking watchdog terminates stalled processes cleanly.
- **Automated Cleanup**: Temporary diagnostics and intermediate files are scrubbed in `finally` blocks.
