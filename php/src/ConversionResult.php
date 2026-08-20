<?php

declare(strict_types=1);

namespace Pdf2Md;

use JsonSerializable;
use Stringable;

class ConversionResult implements JsonSerializable, Stringable
{
    private string $markdown;
    private array $diagnostics;
    private array $metadata;

    public function __construct(string $markdown, array $diagnostics = [], array $metadata = [])
    {
        $this->markdown = $markdown;
        $this->diagnostics = $diagnostics;
        $this->metadata = $metadata;
    }

    public function getMarkdown(): string
    {
        return $this->markdown;
    }

    public function getDiagnostics(): array
    {
        return $this->diagnostics;
    }

    public function statistics(): array
    {
        return $this->diagnostics['stats'] ?? [
            'parse_time_ms' => 0,
            'layout_time_ms' => 0,
            'render_time_ms' => 0,
            'total_time_ms' => 0,
            'memory_peak_bytes' => 0,
        ];
    }

    public function warnings(): array
    {
        return $this->diagnostics['warnings'] ?? [];
    }

    public function confidence(): float
    {
        return (float)($this->diagnostics['overall_confidence'] ?? 1.0);
    }

    public function getOverallConfidence(): float
    {
        return $this->confidence();
    }

    public function metadata(): array
    {
        return $this->metadata;
    }

    public function totalPages(): int
    {
        return (int)($this->diagnostics['total_pages'] ?? $this->metadata['total_pages'] ?? 1);
    }

    public function getTotalPages(): int
    {
        return $this->totalPages();
    }

    public function getTablesDetected(): int
    {
        return (int)($this->diagnostics['tables_detected'] ?? 0);
    }

    public function getImagesExtracted(): int
    {
        return (int)($this->diagnostics['images_extracted'] ?? 0);
    }

    public function toArray(): array
    {
        return [
            'markdown' => $this->markdown,
            'metadata' => $this->metadata,
            'diagnostics' => $this->diagnostics,
            'confidence' => $this->confidence(),
            'total_pages' => $this->totalPages(),
        ];
    }

    public function jsonSerialize(): array
    {
        return $this->toArray();
    }

    public function __toString(): string
    {
        return $this->markdown;
    }
}
