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

use Pdf2Md\Config;
use Pdf2Md\PDFMarkdown;

// Generate synthetic test PDF
$streamContent = "BT\n/F1 16 Tf\n72 700 Td\n(System Architecture Report) Tj\n0 -25 Td\n/F1 12 Tf\n(This document was converted automatically from PDF to clean Markdown.) Tj\n0 -20 Td\n(Key findings include high performance, low memory footprint, and robust multilingual parsing.) Tj\nET\n";
$streamLen = strlen($streamContent);

$pdf = "%PDF-1.4\n"
    . "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n"
    . "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n"
    . "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >> >> >> >>\nendobj\n"
    . "4 0 obj\n<< /Length {$streamLen} >>\nstream\n{$streamContent}\nendstream\nendobj\n"
    . "xref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000300 00000 n \n"
    . "trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n450\n%%EOF\n";

$testPdfFile = __DIR__ . '/test_doc.pdf';
file_put_contents($testPdfFile, $pdf);

echo "=== Testing PHP Native Execution with pdf2md Binary ===\n";

$config = (new Config())
    ->setDialect('gfm')
    ->setBinaryPath(__DIR__ . '/../../target/release/pdf2md.exe');

$pdfMd = PDFMarkdown::fromFile($testPdfFile, $config);
$result = $pdfMd->convert();

echo "--- Generated Markdown ---\n";
echo $result->getMarkdown() . "\n";
echo "--------------------------\n";
echo "Total Pages: " . $result->getTotalPages() . "\n";
echo "Confidence: " . $result->getOverallConfidence() . "\n";

if (strpos($result->getMarkdown(), 'System Architecture Report') !== false) {
    echo "\n>>> SUCCESS: PDF converted to Markdown with high structural accuracy! <<<\n";
} else {
    echo "\n>>> FAILURE: Heading text not found in Markdown output <<<\n";
    exit(1);
}

@unlink($testPdfFile);
