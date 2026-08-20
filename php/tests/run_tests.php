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
use Pdf2Md\ConversionResult;
use Pdf2Md\PDFMarkdown;

echo "=== Running PHP Integration Tests ===\n";

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

// Test 1: Config builder defaults and setters
$config = (new Config())
    ->setDialect('gfm')
    ->setExtractImages(true)
    ->setImagesDir('/tmp/images')
    ->setMemoryLimitMB(128)
    ->setTimeoutSeconds(15)
    ->setDetectTables(false);

assert_test($config->getDialect() === 'gfm', "Config dialect is 'gfm'");
assert_test($config->isExtractImages() === true, "Config extractImages is true");
assert_test($config->getImagesDir() === '/tmp/images', "Config imagesDir is '/tmp/images'");
assert_test($config->getMemoryLimitMB() === 128, "Config memoryLimitMB is 128");
assert_test($config->isDetectTables() === false, "Config detectTables is false");

// Test 2: ConversionResult accessors
$diag = [
    'total_pages' => 5,
    'overall_confidence' => 0.98,
    'tables_detected' => 2,
    'images_extracted' => 3,
];
$result = new ConversionResult("# Sample Heading\n\nContent", $diag);
assert_test($result->getMarkdown() === "# Sample Heading\n\nContent", "Result markdown content");
assert_test($result->getTotalPages() === 5, "Result total pages is 5");
assert_test($result->getOverallConfidence() === 0.98, "Result overall confidence is 0.98");
assert_test($result->getTablesDetected() === 2, "Result tables detected is 2");

// Test 3: PDFMarkdown validation
try {
    PDFMarkdown::fromString("");
    assert_test(false, "Empty string throws exception");
} catch (InvalidArgumentException $e) {
    assert_test(true, "Empty string throws InvalidArgumentException");
}

echo "\nTest Summary: {$passed} passed, {$failed} failed.\n";
if ($failed > 0) {
    exit(1);
}
