<?php

declare(strict_types=1);

namespace Pdf2Md\Laravel\Jobs;

use Exception;
use Pdf2Md\Config;
use Pdf2Md\ConversionResult;
use Pdf2Md\PDFMarkdown;
use Throwable;

/**
 * Async Queue Job for high-performance off-thread PDF conversion in Laravel applications.
 */
class PDFMarkdownJob
{
    public int $tries = 3;
    public int $timeout = 180;
    public array $backoff = [10, 30, 60];

    protected string $filePath;
    protected ?string $destinationPath;
    protected ?Config $config;
    protected ?string $idempotencyKey;
    protected bool $deleteSourceOnComplete;

    public function __construct(
        string $filePath,
        ?string $destinationPath = null,
        ?Config $config = null,
        ?string $idempotencyKey = null,
        bool $deleteSourceOnComplete = false
    ) {
        $this->filePath = $filePath;
        $this->destinationPath = $destinationPath;
        $this->config = $config;
        $this->idempotencyKey = $idempotencyKey;
        $this->deleteSourceOnComplete = $deleteSourceOnComplete;
    }

    /**
     * Execute the job.
     */
    public function handle(): ConversionResult
    {
        try {
            $pdf = PDFMarkdown::fromFile($this->filePath, $this->config);
            $result = $pdf->convert();

            // If destination path is provided, save Markdown output
            if ($this->destinationPath !== null) {
                file_put_contents($this->destinationPath, $result->getMarkdown());
            }

            return $result;
        } finally {
            if ($this->deleteSourceOnComplete && file_exists($this->filePath)) {
                @unlink($this->filePath);
            }
        }
    }

    /**
     * Handle job failure.
     */
    public function failed(Throwable $exception): void
    {
        if ($this->deleteSourceOnComplete && file_exists($this->filePath)) {
            @unlink($this->filePath);
        }
    }

    public function getIdempotencyKey(): ?string
    {
        return $this->idempotencyKey;
    }
}
