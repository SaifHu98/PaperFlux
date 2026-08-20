import {
  ConversionOptions,
  ConversionResult,
  DocumentCapabilities,
  ProgressEvent,
} from './types';
import { ServerFallbackClient } from './fallback';

export * from './types';
export { ServerFallbackClient } from './fallback';

export class PDFMarkdown {
  /**
   * Primary entrypoint: converts a user-selected PDF into Markdown directly in the browser
   * without uploading, falling back to secure server endpoint only when limits or OCR require it.
   */
  public static async convert(
    input: File | Blob | ArrayBuffer | Uint8Array,
    options: ConversionOptions = {}
  ): Promise<ConversionResult> {
    const bytes = await this.normalizeInputToBytes(input);
    const maxFileSizeMB = options.maxBrowserFileSizeMB ?? 50;
    const maxPages = options.maxBrowserPages ?? 200;

    // Check cancellation
    if (options.signal?.aborted) {
      throw new Error('Conversion aborted by user');
    }

    // 1. File size pre-check for browser resource limits
    const fileSizeMB = bytes.length / (1024 * 1024);
    if (fileSizeMB > maxFileSizeMB) {
      if (options.allowServerFallback && options.fallbackConfig) {
        options.onWarning?.({
          code: 'FILE_SIZE_LIMIT_EXCEEDED',
          message: `PDF size (${fileSizeMB.toFixed(1)}MB) exceeds browser limit (${maxFileSizeMB}MB). Routing to secure server.`,
          category: 'BrowserLimit',
        });
        const fallbackClient = new ServerFallbackClient(options.fallbackConfig);
        return fallbackClient.uploadAndConvert(bytes, options);
      }
      throw new Error(
        `PDF file size (${fileSizeMB.toFixed(1)}MB) exceeds browser memory limit of ${maxFileSizeMB}MB.`
      );
    }

    // 2. Perform conversion via Web Worker / WebAssembly
    return new Promise((resolve, reject) => {
      const workerUrl = new URL('./worker.js', import.meta.url);
      const worker = new Worker(workerUrl, { type: 'module' });
      const reqId = 'conv_' + Math.random().toString(36).slice(2, 9);

      let isCompleted = false;

      const cleanup = () => {
        worker.terminate();
      };

      if (options.signal) {
        options.signal.addEventListener('abort', () => {
          cleanup();
          reject(new Error('Conversion aborted by user'));
        });
      }

      worker.onmessage = async (event: MessageEvent) => {
        const msg = event.data;
        if (msg.id !== reqId) return;

        if (msg.event === 'PAGE_START') {
          options.onPageStart?.(msg.pageNumber, msg.totalPages);
          options.onProgress?.({
            currentPage: msg.pageNumber,
            totalPages: msg.totalPages,
            percent: Math.round(((msg.pageNumber - 1) / msg.totalPages) * 100),
            stage: 'layout',
          });
          return;
        }

        if (msg.event === 'PAGE_COMPLETE') {
          options.onPageComplete?.(msg.pageNumber, msg.totalPages, msg.pageMarkdown);
          options.onProgress?.({
            currentPage: msg.pageNumber,
            totalPages: msg.totalPages,
            percent: msg.percent,
            stage: 'rendering',
          });
          return;
        }

        if (msg.success) {
          isCompleted = true;
          cleanup();

          const data = msg.data;
          const result: ConversionResult = {
            markdown: data.markdown,
            metadata: data.metadata,
            diagnostics: data.diagnostics,
            statistics: data.diagnostics?.stats || {
              parse_time_ms: 0,
              layout_time_ms: 0,
              render_time_ms: 0,
              total_time_ms: 0,
              memory_peak_bytes: 0,
            },
            confidence: data.confidence ?? 0.95,
            warnings: data.diagnostics?.warnings || [],
            processedLocally: true,
          };

          options.onProgress?.({
            currentPage: data.totalPages,
            totalPages: data.totalPages,
            percent: 100,
            stage: 'completed',
          });

          options.onComplete?.(result);
          resolve(result);
        } else {
          cleanup();
          // If browser WASM execution failed and fallback is enabled, try server
          if (options.allowServerFallback && options.fallbackConfig) {
            try {
              options.onWarning?.({
                code: 'BROWSER_EXEC_ERROR',
                message: `Local conversion failed (${msg.error}). Falling back to secure server endpoint.`,
                category: 'Fallback',
              });
              const fallbackClient = new ServerFallbackClient(options.fallbackConfig);
              const fallbackResult = await fallbackClient.uploadAndConvert(bytes, options);
              resolve(fallbackResult);
              return;
            } catch (fallbackErr) {
              reject(fallbackErr);
              return;
            }
          }
          reject(new Error(msg.error || 'Conversion failed in Web Worker'));
        }
      };

      worker.onerror = (err) => {
        cleanup();
        reject(new Error(`Web Worker error: ${err.message}`));
      };

      // Launch progressive conversion
      worker.postMessage({
        id: reqId,
        type: 'CONVERT_PROGRESSIVE',
        payload: {
          bytes: bytes.buffer,
          options: {
            dialect: options.dialect,
            detectTables: options.detectTables,
          },
        },
      }, [bytes.buffer]);
    });
  }

  private static async normalizeInputToBytes(
    input: File | Blob | ArrayBuffer | Uint8Array
  ): Promise<Uint8Array> {
    if (input instanceof Uint8Array) {
      return input;
    }
    if (input instanceof ArrayBuffer) {
      return new Uint8Array(input);
    }
    if (input instanceof Blob || (typeof File !== 'undefined' && input instanceof File)) {
      const buffer = await input.arrayBuffer();
      return new Uint8Array(buffer);
    }
    throw new Error('Unsupported input type. Expected File, Blob, ArrayBuffer, or Uint8Array.');
  }
}
