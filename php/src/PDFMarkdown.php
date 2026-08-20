<?php

declare(strict_types=1);

namespace Pdf2Md;

use BadMethodCallException;
use InvalidArgumentException;
use Pdf2Md\Exceptions\ConversionException;

class PDFMarkdown
{
    private ?string $filePath = null;
    private ?string $pdfContent = null;
    private Config $config;

    public function __construct(?string $pdfContent = null, ?Config $config = null)
    {
        $this->pdfContent = $pdfContent;
        $this->config = $config ?? new Config();
    }

    /**
     * Intercept static calls like PDFMarkdown::convert($path, $config)
     */
    public static function __callStatic(string $name, array $arguments)
    {
        if ($name === 'convert') {
            $filePath = $arguments[0] ?? '';
            $config = $arguments[1] ?? null;
            return self::fromFile((string)$filePath, $config)->convert();
        }

        throw new BadMethodCallException("Static method {$name} does not exist on " . static::class);
    }

    /**
     * Initialize conversion from local file path without buffering whole file into memory.
     */
    public static function fromFile(string $filePath, ?Config $config = null): self
    {
        Validation::validatePdfFile($filePath);

        $instance = new self(null, $config);
        $instance->filePath = $filePath;
        return $instance;
    }

    /**
     * Initialize conversion from in-memory string.
     */
    public static function fromString(string $pdfContent, ?Config $config = null): self
    {
        Validation::validatePdfContent($pdfContent);
        return new self($pdfContent, $config);
    }

    /**
     * Initialize conversion from a stream resource.
     *
     * @param resource $stream
     */
    public static function fromStream($stream, ?Config $config = null): self
    {
        if (!is_resource($stream)) {
            throw new InvalidArgumentException('Expected a valid stream resource');
        }

        $content = stream_get_contents($stream);
        if ($content === false) {
            throw new ConversionException('Failed to read from stream');
        }

        return self::fromString($content, $config);
    }

    /**
     * Initialize conversion from Laravel UploadedFile or PSR-7 UploadedFileInterface.
     */
    public static function fromUploadedFile($uploadedFile, ?Config $config = null): self
    {
        if (is_object($uploadedFile)) {
            if (method_exists($uploadedFile, 'getRealPath')) {
                return self::fromFile($uploadedFile->getRealPath(), $config);
            }
            if (method_exists($uploadedFile, 'getPathname')) {
                return self::fromFile($uploadedFile->getPathname(), $config);
            }
            if (method_exists($uploadedFile, 'getStream')) {
                $stream = $uploadedFile->getStream();
                if (is_resource($stream)) {
                    return self::fromStream($stream, $config);
                }
            }
        }

        throw new InvalidArgumentException('Invalid uploaded file object');
    }

    // --- Fluent Configuration Modifiers ---

    public function ocr(string $mode): self
    {
        $this->config->ocr($mode);
        return $this;
    }

    public function tables(bool $enable = true): self
    {
        $this->config->tables($enable);
        return $this;
    }

    public function images(bool $enable = true, ?string $dir = null): self
    {
        $this->config->images($enable);
        if ($dir !== null) {
            $this->config->setImagesDir($dir);
        }
        return $this;
    }

    public function metadata(bool $enable = true): self
    {
        $this->config->metadata($enable);
        return $this;
    }

    public function dialect(string $dialect): self
    {
        $this->config->setDialect($dialect);
        return $this;
    }

    public function timeout(int $seconds): self
    {
        $this->config->timeout($seconds);
        return $this;
    }

    public function memoryLimit(int $mb): self
    {
        $this->config->memoryLimit($mb);
        return $this;
    }

    public function maxPages(int $pages): self
    {
        $this->config->maxPages($pages);
        return $this;
    }

    public function tempDir(string $dir): self
    {
        $this->config->tempDir($dir);
        return $this;
    }

    public function setConfig(Config $config): self
    {
        $this->config = $config;
        return $this;
    }

    public function getConfig(): Config
    {
        return $this->config;
    }

    /**
     * Executes the conversion and returns the structured ConversionResult.
     */
    public function convert(): ConversionResult
    {
        // 1. Offload to remote HTTP microservice if configured
        if ($this->config->getHttpEndpoint() !== null) {
            $client = new HttpClient($this->config->getHttpEndpoint(), $this->config->getTimeoutSeconds());
            if ($this->filePath !== null) {
                $content = file_get_contents($this->filePath);
                if ($content === false) {
                    throw new ConversionException("Failed to read file: {$this->filePath}");
                }
                return $client->convert($content);
            }
            return $client->convert($this->pdfContent ?? '');
        }

        // 2. Execute native binary
        $runner = new ProcessRunner($this->config);
        if ($this->filePath !== null) {
            return $runner->runFile($this->filePath);
        }

        return $runner->runContent($this->pdfContent ?? '');
    }
}
