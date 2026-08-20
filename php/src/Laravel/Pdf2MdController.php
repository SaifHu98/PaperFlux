<?php

declare(strict_types=1);

namespace Pdf2Md\Laravel;

use Exception;
use Pdf2Md\Config;
use Pdf2Md\PDFMarkdown;

/**
 * Production-ready server fallback controller for Laravel applications.
 */
class Pdf2MdController
{
    /**
     * Handle incoming PDF conversion request from browser fallback or API client.
     */
    public function convert($request = null)
    {
        $startTime = microtime(true);

        try {
            // Read raw body or uploaded file safely
            $pdfContent = $this->extractPdfContent($request);

            if (empty($pdfContent)) {
                return $this->jsonResponse([
                    'error' => 'No PDF content received or file is empty',
                ], 400);
            }

            // Enforce maximum server document size limit (100MB default)
            if (strlen($pdfContent) > 100 * 1024 * 1024) {
                return $this->jsonResponse([
                    'error' => 'PDF document exceeds maximum server limit of 100MB',
                ], 413);
            }

            // Configure conversion
            $config = (new Config())
                ->setDialect('gfm')
                ->setDetectTables(true)
                ->setTimeoutSeconds(60)
                ->setMemoryLimitMB(512);

            $pdf = PDFMarkdown::fromString($pdfContent, $config);
            $result = $pdf->convert();

            $elapsedMs = (int)((microtime(true) - $startTime) * 1000);

            $diagnostics = $result->getDiagnostics();
            if (!isset($diagnostics['stats'])) {
                $diagnostics['stats'] = [
                    'parse_time_ms' => 0,
                    'layout_time_ms' => 0,
                    'render_time_ms' => 0,
                    'total_time_ms' => $elapsedMs,
                    'memory_peak_bytes' => memory_get_peak_usage(true),
                ];
            }

            return $this->jsonResponse([
                'markdown' => $result->getMarkdown(),
                'metadata' => [
                    'total_pages' => $result->getTotalPages(),
                    'is_encrypted' => false,
                ],
                'diagnostics' => $diagnostics,
                'overall_confidence' => $result->getOverallConfidence(),
                'total_pages' => $result->getTotalPages(),
            ], 200);
        } catch (Exception $e) {
            return $this->jsonResponse([
                'error' => 'Conversion failed: ' . $e->getMessage(),
            ], 500);
        } finally {
            // Scrub memory buffers
            unset($pdfContent);
        }
    }

    private function extractPdfContent($request): string
    {
        // 1. Direct raw body (HTTP POST with application/pdf)
        $raw = file_get_contents('php://input');
        if (!empty($raw) && str_starts_with($raw, '%PDF-')) {
            return $raw;
        }

        // 2. Laravel Request file support if available
        if (is_object($request) && method_exists($request, 'file')) {
            $file = $request->file('pdf') ?? $request->file('file');
            if ($file && method_exists($file, 'getRealPath')) {
                $path = $file->getRealPath();
                if (file_exists($path)) {
                    return file_get_contents($path) ?: '';
                }
            }
        }

        // 3. Fallback to $_FILES
        if (!empty($_FILES['file']['tmp_name']) && file_exists($_FILES['file']['tmp_name'])) {
            return file_get_contents($_FILES['file']['tmp_name']) ?: '';
        }
        if (!empty($_FILES['pdf']['tmp_name']) && file_exists($_FILES['pdf']['tmp_name'])) {
            return file_get_contents($_FILES['pdf']['tmp_name']) ?: '';
        }

        return $raw ?: '';
    }

    private function jsonResponse(array $data, int $statusCode): array
    {
        http_response_code($statusCode);
        header('Content-Type: application/json');
        echo json_encode($data);
        return $data;
    }
}
