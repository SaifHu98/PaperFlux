<?php

declare(strict_types=1);

namespace Pdf2Md;

use InvalidArgumentException;
use Pdf2Md\Exceptions\SecurityException;

class Validation
{
    /**
     * Validates that a file exists, is readable, has a valid PDF magic header, and fits size limits.
     */
    public static function validatePdfFile(string $filePath, ?int $maxSizeBytes = null): void
    {
        if (!file_exists($filePath)) {
            throw new InvalidArgumentException("PDF file not found: {$filePath}");
        }

        if (!is_readable($filePath)) {
            throw new InvalidArgumentException("PDF file is not readable: {$filePath}");
        }

        if ($maxSizeBytes !== null) {
            $fileSize = filesize($filePath);
            if ($fileSize !== false && $fileSize > $maxSizeBytes) {
                throw new SecurityException("PDF file size ({$fileSize} bytes) exceeds limit of {$maxSizeBytes} bytes");
            }
        }

        // Validate PDF magic bytes (%PDF-)
        $handle = fopen($filePath, 'rb');
        if ($handle === false) {
            throw new InvalidArgumentException("Unable to open PDF file: {$filePath}");
        }

        $header = fread($handle, 8);
        fclose($handle);

        if ($header === false || !str_starts_with($header, '%PDF-')) {
            throw new InvalidArgumentException("File is not a valid PDF document (missing %PDF- header)");
        }
    }

    /**
     * Validates raw PDF bytes in memory.
     */
    public static function validatePdfContent(string $content, ?int $maxSizeBytes = null): void
    {
        if (empty($content)) {
            throw new InvalidArgumentException("PDF content cannot be empty");
        }

        if ($maxSizeBytes !== null && strlen($content) > $maxSizeBytes) {
            throw new SecurityException("PDF content length (" . strlen($content) . " bytes) exceeds limit of {$maxSizeBytes} bytes");
        }

        if (!str_starts_with($content, '%PDF-')) {
            throw new InvalidArgumentException("Content is not a valid PDF document (missing %PDF- header)");
        }
    }
}
