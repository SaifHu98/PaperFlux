import { ConversionOptions, ConversionResult, FallbackConfig } from './types';

export class ServerFallbackClient {
  private config: FallbackConfig;

  constructor(config: FallbackConfig) {
    this.config = config;
  }

  public async uploadAndConvert(
    bytes: Uint8Array,
    options?: ConversionOptions
  ): Promise<ConversionResult> {
    const endpoint = this.config.endpoint.replace(/\/$/, '') + '/convert';
    const timeoutMs = this.config.timeoutMs || 60000;

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

    if (options?.signal) {
      options.signal.addEventListener('abort', () => controller.abort());
    }

    const headers: Record<string, string> = {
      'Content-Type': 'application/pdf',
      ...(this.config.headers || {}),
    };

    if (this.config.authToken) {
      headers['Authorization'] = `Bearer ${this.config.authToken}`;
    }
    if (this.config.csrfToken) {
      headers['X-CSRF-TOKEN'] = this.config.csrfToken;
      headers['X-XSRF-TOKEN'] = this.config.csrfToken;
    }

    try {
      const response = await fetch(endpoint, {
        method: 'POST',
        headers,
        body: bytes,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const errorText = await response.text().catch(() => 'Unknown error');
        throw new Error(`Server fallback conversion failed (${response.status}): ${errorText}`);
      }

      const data = await response.json();

      const diagnostics = data.diagnostics || {};
      const metadata = data.metadata || { total_pages: diagnostics.total_pages || 0, is_encrypted: false };

      return {
        markdown: data.markdown || '',
        metadata,
        diagnostics,
        statistics: diagnostics.stats || {
          parse_time_ms: 0,
          layout_time_ms: 0,
          render_time_ms: 0,
          total_time_ms: 0,
          memory_peak_bytes: 0,
        },
        confidence: diagnostics.overall_confidence ?? 0.95,
        warnings: diagnostics.warnings || [],
        processedLocally: false,
      };
    } catch (err: any) {
      clearTimeout(timeoutId);
      if (err.name === 'AbortError') {
        throw new Error('Server fallback conversion timed out or was aborted');
      }
      throw err;
    }
  }
}
