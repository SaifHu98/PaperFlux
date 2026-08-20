<?php

declare(strict_types=1);

namespace Pdf2Md;

use Pdf2Md\Exceptions\BinaryNotFoundException;
use Pdf2Md\Exceptions\ConversionException;
use Pdf2Md\Exceptions\SecurityException;

class ProcessRunner
{
    private Config $config;

    public function __construct(Config $config)
    {
        $this->config = $config;
    }

    /**
     * Executes conversion passing file path directly to binary to minimize PHP memory overhead.
     */
    public function runFile(string $filePath): ConversionResult
    {
        return $this->executeCommand($filePath, null);
    }

    /**
     * Executes conversion streaming raw PDF bytes through standard input.
     */
    public function runContent(string $pdfContent): ConversionResult
    {
        return $this->executeCommand('-', $pdfContent);
    }

    private function executeCommand(string $inputArg, ?string $stdinData): ConversionResult
    {
        $binary = $this->resolveBinaryPath();
        $diagFile = tempnam($this->config->getTempDir(), 'pdf2md_diag_');

        $cmd = [
            $binary,
            $inputArg,
            '--dialect', $this->config->getDialect(),
            '--ocr', $this->config->getOcrMode(),
            '--memory-limit-mb', (string)$this->config->getMemoryLimitMB(),
            '--max-pages', (string)$this->config->getMaxPages(),
            '--timeout', (string)$this->config->getTimeoutSeconds(),
            '--page-breaks', $this->config->getPageBreaks(),
            '--diagnostics-json', $diagFile,
        ];

        if (!$this->config->isDetectTables()) {
            $cmd[] = '--no-tables';
        }

        if ($this->config->isExtractImages()) {
            $cmd[] = '--extract-images';
            if ($this->config->getImagesDir() !== null) {
                $cmd[] = '--images-dir';
                $cmd[] = $this->config->getImagesDir();
            }
        }

        if (!$this->config->isEmitFrontmatter()) {
            $cmd[] = '--no-frontmatter';
        }

        $descriptors = [
            0 => ['pipe', 'r'], // STDIN
            1 => ['pipe', 'w'], // STDOUT
            2 => ['pipe', 'w'], // STDERR
        ];

        $process = proc_open($cmd, $descriptors, $pipes);

        if (!is_resource($process)) {
            if (file_exists($diagFile)) {
                @unlink($diagFile);
            }
            throw new ConversionException('Failed to spawn pdf2md process');
        }

        // Stream stdin data if provided
        if ($stdinData !== null) {
            $len = strlen($stdinData);
            $offset = 0;
            $chunkSize = 8192;
            while ($offset < $len) {
                $chunk = substr($stdinData, $offset, $chunkSize);
                $written = fwrite($pipes[0], $chunk);
                if ($written === false || $written === 0) {
                    break;
                }
                $offset += $written;
            }
        }
        fclose($pipes[0]);

        // Non-blocking read with timeout
        $timeout = $this->config->getTimeoutSeconds();
        $startTime = time();
        $stdout = '';
        $stderr = '';

        stream_set_blocking($pipes[1], false);
        stream_set_blocking($pipes[2], false);

        while (true) {
            $read = [$pipes[1], $pipes[2]];
            $write = null;
            $except = null;

            $numChanged = @stream_select($read, $write, $except, 1);

            if ($numChanged > 0) {
                foreach ($read as $r) {
                    if ($r === $pipes[1]) {
                        $chunk = fread($pipes[1], 8192);
                        if ($chunk !== false) {
                            $stdout .= $chunk;
                        }
                    } elseif ($r === $pipes[2]) {
                        $chunk = fread($pipes[2], 8192);
                        if ($chunk !== false) {
                            $stderr .= $chunk;
                        }
                    }
                }
            }

            $status = proc_get_status($process);
            if (!$status['running']) {
                break;
            }

            if ((time() - $startTime) > $timeout) {
                proc_terminate($process, 9);
                fclose($pipes[1]);
                fclose($pipes[2]);
                proc_close($process);
                if (file_exists($diagFile)) {
                    @unlink($diagFile);
                }
                throw new SecurityException("Conversion timed out after {$timeout} seconds");
            }
        }

        $stdout .= stream_get_contents($pipes[1]);
        $stderr .= stream_get_contents($pipes[2]);
        fclose($pipes[1]);
        fclose($pipes[2]);

        $exitCode = proc_close($process);

        // Read diagnostics JSON
        $diagnostics = [];
        if (file_exists($diagFile)) {
            $diagJson = @file_get_contents($diagFile);
            if ($diagJson !== false && !empty($diagJson)) {
                $diagnostics = json_decode($diagJson, true) ?? [];
            }
            @unlink($diagFile);
        }

        if ($exitCode !== 0) {
            throw new ConversionException("pdf2md conversion failed (exit code {$exitCode}): {$stderr}");
        }

        return new ConversionResult($stdout, $diagnostics, $diagnostics['metadata'] ?? []);
    }

    private function resolveBinaryPath(): string
    {
        if ($this->config->getBinaryPath() !== null && file_exists($this->config->getBinaryPath())) {
            return $this->config->getBinaryPath();
        }

        $candidates = [
            __DIR__ . '/../../target/release/pdf2md',
            __DIR__ . '/../../target/release/pdf2md.exe',
            __DIR__ . '/../../target/debug/pdf2md',
            __DIR__ . '/../../target/debug/pdf2md.exe',
            'pdf2md',
            'pdf2md.exe',
        ];

        foreach ($candidates as $candidate) {
            if (file_exists($candidate)) {
                return realpath($candidate) ?: $candidate;
            }
        }

        return 'pdf2md';
    }
}
