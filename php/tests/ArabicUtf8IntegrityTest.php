<?php

declare(strict_types=1);

require_once __DIR__ . '/../src/Exceptions/ConversionException.php';
require_once __DIR__ . '/../src/Exceptions/SecurityException.php';
require_once __DIR__ . '/../src/Exceptions/BinaryNotFoundException.php';
require_once __DIR__ . '/../src/Validation.php';
require_once __DIR__ . '/../src/Config.php';
require_once __DIR__ . '/../src/ConversionResult.php';
require_once __DIR__ . '/../src/PDFMarkdown.php';

use Pdf2Md\ConversionResult;

function assert_test(bool $condition, string $message) {
    if ($condition) {
        echo "  [PASS] $message\n";
    } else {
        echo "  [FAIL] $message\n";
        exit(1);
    }
}

echo "=== Running Arabic UTF-8 Multibyte Integrity Tests for PHP & Laravel ===\n";

// 1. Test Arabic String and Multibyte handling
$arabicText = "تقرير الأداء السنوي لشركة PaperFlux لعام ٢٠٢٦ - معدل الإنجاز ٩٨٫٥٪";
$expectedLen = mb_strlen($arabicText, 'UTF-8');
assert_test($expectedLen > 50 && $expectedLen < 80, "mb_strlen correctly computes Arabic UTF-8 length ($expectedLen chars)");

// 2. Test ConversionResult with Arabic Content
$res = new ConversionResult(
    markdown: "# $arabicText\n\nتم إنجاز كافة المهام بنجاح.",
    diagnostics: [
        'overall_confidence' => 0.98,
        'total_pages' => 1,
        'warnings' => [],
        'stats' => ['arabic_chars' => 85, 'latin_chars' => 9],
    ],
    metadata: ['title' => 'تقرير سنوي', 'author' => 'جامعة بغداد']
);

assert_test(str_contains($res->getMarkdown(), "تقرير الأداء السنوي"), "Markdown contains Arabic title");
assert_test(str_contains($res->getMarkdown(), "PaperFlux"), "Markdown preserves embedded Latin brand");
assert_test(str_contains($res->getMarkdown(), "٢٠٢٦"), "Markdown preserves Eastern Arabic numerals");
assert_test(str_contains($res->getMarkdown(), "٩٨٫٥٪"), "Markdown preserves Eastern Arabic percentage");

// 3. Test JSON Serialization and Deserialization with Arabic UTF-8
$json = json_encode($res->jsonSerialize(), JSON_UNESCAPED_UNICODE | JSON_PRETTY_PRINT);
assert_test(!str_contains($json, "\\u062a"), "JSON contains unescaped Arabic Unicode chars");
assert_test(str_contains($json, "تقرير سنوي"), "JSON metadata contains Arabic title");

$decoded = json_decode($json, true);
assert_test($decoded['metadata']['author'] === 'جامعة بغداد', "JSON decode preserves Arabic author name");

echo "\nArabic UTF-8 Integrity Test Summary: 8 passed, 0 failed.\n";
