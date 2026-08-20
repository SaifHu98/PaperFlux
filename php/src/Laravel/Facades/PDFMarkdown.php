<?php

declare(strict_types=1);

namespace Pdf2Md\Laravel\Facades;

use Pdf2Md\Config;
use Pdf2Md\ConversionResult;
use Pdf2Md\PDFMarkdown as BasePDFMarkdown;

/**
 * Laravel Facade for PDFMarkdown.
 *
 * @method static ConversionResult convert(string $filePath, ?Config $config = null)
 * @method static BasePDFMarkdown fromFile(string $filePath, ?Config $config = null)
 * @method static BasePDFMarkdown fromString(string $pdfContent, ?Config $config = null)
 * @method static BasePDFMarkdown fromStream($stream, ?Config $config = null)
 * @method static BasePDFMarkdown fromUploadedFile($uploadedFile, ?Config $config = null)
 */
class PDFMarkdown
{
    /**
     * Get the registered name of the component.
     */
    protected static function getFacadeAccessor(): string
    {
        return 'pdf2md';
    }

    /**
     * Handle dynamic static calls against the underlying PDFMarkdown instance.
     */
    public static function __callStatic(string $method, array $args)
    {
        return BasePDFMarkdown::$method(...$args);
    }
}
