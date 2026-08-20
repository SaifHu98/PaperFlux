<?php

declare(strict_types=1);

namespace Pdf2Md;

use Pdf2Md\Exceptions\ConversionException;

class HttpClient
{
    private string $endpoint;
    private int $timeout;

    public function __construct(string $endpoint, int $timeout = 60)
    {
        $this->endpoint = rtrim($endpoint, '/');
        $this->timeout = $timeout;
    }

    public function convert(string $pdfContent): ConversionResult
    {
        $url = "{$this->endpoint}/convert";

        $context = stream_context_create([
            'http' => [
                'method' => 'POST',
                'header' => "Content-Type: application/pdf\r\nContent-Length: " . strlen($pdfContent) . "\r\n",
                'content' => $pdfContent,
                'timeout' => $this->timeout,
                'ignore_errors' => true,
            ],
        ]);

        $response = @file_get_contents($url, false, $context);

        if ($response === false) {
            throw new ConversionException("Failed to connect to pdf2md HTTP microservice at {$url}");
        }

        $data = json_decode($response, true);
        if (!is_array($data) || !isset($data['markdown'])) {
            $err = $data['error'] ?? 'Unknown remote error';
            throw new ConversionException("Remote conversion error: {$err}");
        }

        return new ConversionResult($data['markdown'], $data['diagnostics'] ?? []);
    }
}
