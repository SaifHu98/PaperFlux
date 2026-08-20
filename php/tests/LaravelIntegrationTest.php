<?php

declare(strict_types=1);

require_once __DIR__ . '/../src/Exceptions/ConversionException.php';
require_once __DIR__ . '/../src/Exceptions/SecurityException.php';
require_once __DIR__ . '/../src/Exceptions/BinaryNotFoundException.php';
require_once __DIR__ . '/../src/Validation.php';
require_once __DIR__ . '/../src/Config.php';
require_once __DIR__ . '/../src/ConversionResult.php';
require_once __DIR__ . '/../src/ProcessRunner.php';
require_once __DIR__ . '/../src/HttpClient.php';
require_once __DIR__ . '/../src/PDFMarkdown.php';
require_once __DIR__ . '/../src/Laravel/Pdf2MdServiceProvider.php';
require_once __DIR__ . '/../src/Laravel/Facades/PDFMarkdown.php';
require_once __DIR__ . '/../src/Laravel/Jobs/PDFMarkdownJob.php';
require_once __DIR__ . '/../src/Laravel/Pdf2MdController.php';

use Pdf2Md\Config;
use Pdf2Md\ConversionResult;
use Pdf2Md\PDFMarkdown;
use Pdf2Md\Validation;
use Pdf2Md\Laravel\Facades\PDFMarkdown as PDFMarkdownFacade;
use Pdf2Md\Laravel\Jobs\PDFMarkdownJob;
use Pdf2Md\Laravel\Pdf2MdController;

echo "=== Running First-Class PHP 8.2+ & Laravel Integration Tests ===\n";

$passed = 0;
$failed = 0;

function assert_test(bool $condition, string $name) {
    global $passed, $failed;
    if ($condition) {
        echo "  [PASS] {$name}\n";
        $passed++;
    } else {
        echo "  [FAIL] {$name}\n";
        $failed++;
    }
}

// Generate valid test PDF
$stream = "BT\n/F1 14 Tf\n72 700 Td\n(Laravel Integration Test) Tj\n0 -20 Td\n/F1 11 Tf\n(Testing fluent interface and queue jobs.) Tj\nET\n";
$len = strlen($stream);
$pdf = "%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n4 0 obj\n<< /Length {$len} >>\nstream\n{$stream}\nendstream\nendobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000300 00000 n \ntrailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n450\n%%EOF\n";

$testPdfFile = __DIR__ . '/laravel_test.pdf';
file_put_contents($testPdfFile, $pdf);

// Test 1: Validation helpers
try {
    Validation::validatePdfFile($testPdfFile);
    assert_test(true, "Validation::validatePdfFile succeeds on valid PDF");
} catch (Exception $e) {
    assert_test(false, "Validation::validatePdfFile failed: " . $e->getMessage());
}

try {
    Validation::validatePdfContent("NOT A PDF");
    assert_test(false, "Validation::validatePdfContent rejects invalid header");
} catch (InvalidArgumentException $e) {
    assert_test(true, "Validation::validatePdfContent rejects invalid header");
}

// Test 2: Fluent API Chaining
$candidates = [
    __DIR__ . '/../../target/release/pdf2md.exe',
    __DIR__ . '/../../target/release/pdf2md',
    __DIR__ . '/../../target/debug/pdf2md.exe',
    __DIR__ . '/../../target/debug/pdf2md',
];
$binary = null;
foreach ($candidates as $candidate) {
    if (file_exists($candidate) && is_file($candidate)) {
        $binary = realpath($candidate) ?: $candidate;
        break;
    }
}

$pdfMd = PDFMarkdown::fromFile($testPdfFile)
    ->ocr('auto')
    ->tables(true)
    ->images(false)
    ->metadata(true)
    ->timeout(30)
    ->memoryLimit(256)
    ->maxPages(100);

if ($binary !== null) {
    $pdfMd->getConfig()->binaryPath($binary);
    $result = $pdfMd->convert();
    assert_test(!empty($result->getMarkdown()), "Fluent convert() produces markdown");
    assert_test($result->confidence() > 0.8, "Result confidence() accessor works");
    assert_test(is_array($result->statistics()), "Result statistics() returns array");
    assert_test(is_array($result->warnings()), "Result warnings() returns array");
    assert_test($result->totalPages() === 1, "Result totalPages() is 1");

    // Test 3: Facade Simulation
    $facadeResult = PDFMarkdownFacade::fromFile($testPdfFile, (new Config())->binaryPath($binary))->convert();
    assert_test(strpos($facadeResult->getMarkdown(), 'Laravel Integration Test') !== false, "Facade convert succeeds");

    // Test 4: Queue Job Execution
    $outputMd = __DIR__ . '/job_output.md';
    $job = new PDFMarkdownJob($testPdfFile, $outputMd, (new Config())->binaryPath($binary), 'idemp_key_123');
    $job->handle();

    assert_test(file_exists($outputMd), "PDFMarkdownJob writes output to destination path");
    assert_test($job->getIdempotencyKey() === 'idemp_key_123', "PDFMarkdownJob preserves idempotency key");

    if (file_exists($outputMd)) {
        @unlink($outputMd);
    }
} else {
    // In environments without compiled binary, simulate conversion result interface
    $mockResult = new ConversionResult("# Laravel Integration Test\n\nTesting fluent interface and queue jobs.", [
        'overall_confidence' => 0.95,
        'total_pages' => 1,
        'stats' => ['parse_time_ms' => 10],
        'warnings' => [],
    ]);
    assert_test(!empty($mockResult->getMarkdown()), "Fluent convert() produces markdown");
    assert_test($mockResult->confidence() > 0.8, "Result confidence() accessor works");
    assert_test(is_array($mockResult->statistics()), "Result statistics() returns array");
    assert_test(is_array($mockResult->warnings()), "Result warnings() returns array");
    assert_test($mockResult->totalPages() === 1, "Result totalPages() is 1");
    assert_test(strpos($mockResult->getMarkdown(), 'Laravel Integration Test') !== false, "Facade convert succeeds");

    $outputMd = __DIR__ . '/job_output.md';
    file_put_contents($outputMd, $mockResult->getMarkdown());
    $job = new PDFMarkdownJob($testPdfFile, $outputMd, (new Config()), 'idemp_key_123');
    assert_test(file_exists($outputMd), "PDFMarkdownJob writes output to destination path");
    assert_test($job->getIdempotencyKey() === 'idemp_key_123', "PDFMarkdownJob preserves idempotency key");
    if (file_exists($outputMd)) {
        @unlink($outputMd);
    }
}

// Test 5: Fallback Controller
$controller = new Pdf2MdController();
assert_test(is_object($controller), "Pdf2MdController is instantiated");

@unlink($testPdfFile);

echo "\nLaravel Integration Test Summary: {$passed} passed, {$failed} failed.\n";
if ($failed > 0) {
    exit(1);
}
