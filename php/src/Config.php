<?php

declare(strict_types=1);

namespace Pdf2Md;

class Config
{
    private string $dialect = 'gfm';
    private bool $extractImages = false;
    private ?string $imagesDir = null;
    private bool $detectTables = true;
    private string $ocrMode = 'auto';
    private int $memoryLimitMB = 256;
    private int $maxPages = 5000;
    private int $timeoutSeconds = 60;
    private string $pageBreaks = 'html';
    private bool $emitFrontmatter = true;
    private ?string $tempDir = null;
    private ?string $binaryPath = null;
    private ?string $httpEndpoint = null;

    public function getDialect(): string
    {
        return $this->dialect;
    }

    public function setDialect(string $dialect): self
    {
        $this->dialect = $dialect;
        return $this;
    }

    public function isExtractImages(): bool
    {
        return $this->extractImages;
    }

    public function setExtractImages(bool $extractImages): self
    {
        $this->extractImages = $extractImages;
        return $this;
    }

    public function images(bool $extract = true): self
    {
        return $this->setExtractImages($extract);
    }

    public function getImagesDir(): ?string
    {
        return $this->imagesDir;
    }

    public function setImagesDir(?string $imagesDir): self
    {
        $this->imagesDir = $imagesDir;
        return $this;
    }

    public function isDetectTables(): bool
    {
        return $this->detectTables;
    }

    public function setDetectTables(bool $detectTables): self
    {
        $this->detectTables = $detectTables;
        return $this;
    }

    public function tables(bool $detect = true): self
    {
        return $this->setDetectTables($detect);
    }

    public function getOcrMode(): string
    {
        return $this->ocrMode;
    }

    public function setOcrMode(string $ocrMode): self
    {
        $this->ocrMode = $ocrMode;
        return $this;
    }

    public function ocr(string $mode): self
    {
        return $this->setOcrMode($mode);
    }

    public function getMemoryLimitMB(): int
    {
        return $this->memoryLimitMB;
    }

    public function setMemoryLimitMB(int $memoryLimitMB): self
    {
        $this->memoryLimitMB = $memoryLimitMB;
        return $this;
    }

    public function memoryLimit(int $mb): self
    {
        return $this->setMemoryLimitMB($mb);
    }

    public function getMaxPages(): int
    {
        return $this->maxPages;
    }

    public function setMaxPages(int $maxPages): self
    {
        $this->maxPages = $maxPages;
        return $this;
    }

    public function maxPages(int $pages): self
    {
        return $this->setMaxPages($pages);
    }

    public function getTimeoutSeconds(): int
    {
        return $this->timeoutSeconds;
    }

    public function setTimeoutSeconds(int $timeoutSeconds): self
    {
        $this->timeoutSeconds = $timeoutSeconds;
        return $this;
    }

    public function timeout(int $seconds): self
    {
        return $this->setTimeoutSeconds($seconds);
    }

    public function getPageBreaks(): string
    {
        return $this->pageBreaks;
    }

    public function setPageBreaks(string $pageBreaks): self
    {
        $this->pageBreaks = $pageBreaks;
        return $this;
    }

    public function isEmitFrontmatter(): bool
    {
        return $this->emitFrontmatter;
    }

    public function setEmitFrontmatter(bool $emitFrontmatter): self
    {
        $this->emitFrontmatter = $emitFrontmatter;
        return $this;
    }

    public function metadata(bool $emit = true): self
    {
        return $this->setEmitFrontmatter($emit);
    }

    public function getTempDir(): string
    {
        return $this->tempDir ?? sys_get_temp_dir();
    }

    public function setTempDir(string $tempDir): self
    {
        $this->tempDir = $tempDir;
        return $this;
    }

    public function tempDir(string $dir): self
    {
        return $this->setTempDir($dir);
    }

    public function getBinaryPath(): ?string
    {
        return $this->binaryPath;
    }

    public function setBinaryPath(?string $binaryPath): self
    {
        $this->binaryPath = $binaryPath;
        return $this;
    }

    public function binaryPath(string $path): self
    {
        return $this->setBinaryPath($path);
    }

    public function getHttpEndpoint(): ?string
    {
        return $this->httpEndpoint;
    }

    public function setHttpEndpoint(?string $httpEndpoint): self
    {
        $this->httpEndpoint = $httpEndpoint;
        return $this;
    }

    public function httpEndpoint(string $url): self
    {
        return $this->setHttpEndpoint($url);
    }
}
