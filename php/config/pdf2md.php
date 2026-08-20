<?php

declare(strict_types=1);

return [
    /*
    |--------------------------------------------------------------------------
    | Binary Path
    |--------------------------------------------------------------------------
    |
    | Path to the compiled `pdf2md` native binary. If null, the package
    | will search in the target build directories or system PATH.
    |
    */
    'binary_path' => env('PDF2MD_BINARY_PATH', null),

    /*
    |--------------------------------------------------------------------------
    | Default Markdown Dialect
    |--------------------------------------------------------------------------
    |
    | Supported dialects: 'gfm' (GitHub Flavored), 'commonmark', 'extended'.
    |
    */
    'dialect' => env('PDF2MD_DIALECT', 'gfm'),

    /*
    |--------------------------------------------------------------------------
    | Resource Budgets
    |--------------------------------------------------------------------------
    */
    'timeout' => (int)env('PDF2MD_TIMEOUT', 60),
    'memory_limit_mb' => (int)env('PDF2MD_MEMORY_LIMIT_MB', 256),
    'max_pages' => (int)env('PDF2MD_MAX_PAGES', 5000),

    /*
    |--------------------------------------------------------------------------
    | Features
    |--------------------------------------------------------------------------
    */
    'detect_tables' => (bool)env('PDF2MD_DETECT_TABLES', true),
    'ocr_mode' => env('PDF2MD_OCR_MODE', 'auto'), // 'auto', 'always', 'never'
    'extract_images' => (bool)env('PDF2MD_EXTRACT_IMAGES', false),
    'images_dir' => env('PDF2MD_IMAGES_DIR', null),

    /*
    |--------------------------------------------------------------------------
    | Remote Microservice Worker (Optional)
    |--------------------------------------------------------------------------
    |
    | If configured, conversions will be offloaded to this HTTP daemon.
    |
    */
    'http_endpoint' => env('PDF2MD_HTTP_ENDPOINT', null),
];
