<?php

declare(strict_types=1);

use Pdf2Md\Laravel\Pdf2MdController;

/**
 * Route definition for Laravel:
 * In your routes/api.php, add:
 *
 * Route::post('/pdf2md/convert', [Pdf2MdController::class, 'convert'])
 *     ->middleware(['auth:sanctum', 'throttle:60,1']);
 */
return static function ($router) {
    $router->post('/api/pdf2md/convert', [Pdf2MdController::class, 'convert']);
};
