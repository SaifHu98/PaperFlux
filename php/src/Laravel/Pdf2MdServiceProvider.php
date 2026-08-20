<?php

declare(strict_types=1);

namespace Pdf2Md\Laravel;

use Pdf2Md\Config;
use Pdf2Md\PDFMarkdown;

/**
 * Laravel Service Provider for pdf2md.
 */
class Pdf2MdServiceProvider
{
    protected $app;

    public function __construct($app = null)
    {
        $this->app = $app;
    }

    public function register(): void
    {
        if ($this->app && method_exists($this->app, 'singleton')) {
            $this->app->singleton('pdf2md', function ($app) {
                $config = new Config();
                if (function_exists('config')) {
                    $config
                        ->setDialect(config('pdf2md.dialect', 'gfm'))
                        ->setTimeoutSeconds(config('pdf2md.timeout', 60))
                        ->setMemoryLimitMB(config('pdf2md.memory_limit_mb', 256))
                        ->setMaxPages(config('pdf2md.max_pages', 5000))
                        ->setDetectTables(config('pdf2md.detect_tables', true))
                        ->setOcrMode(config('pdf2md.ocr_mode', 'auto'))
                        ->setExtractImages(config('pdf2md.extract_images', false))
                        ->setImagesDir(config('pdf2md.images_dir', null))
                        ->setBinaryPath(config('pdf2md.binary_path', null))
                        ->setHttpEndpoint(config('pdf2md.http_endpoint', null));
                }
                return new PDFMarkdown(null, $config);
            });
        }
    }

    public function boot(): void
    {
        // Publish config when running in console
        if (function_exists('config_path')) {
            $configPath = __DIR__ . '/../../config/pdf2md.php';
            if (file_exists($configPath) && is_object($this->app) && method_exists($this->app, 'runningInConsole') && $this->app->runningInConsole()) {
                // If Laravel publishes method exists
            }
        }
    }
}
