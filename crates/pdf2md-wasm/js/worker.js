// Web Worker for asynchronous off-thread PDF-to-Markdown processing in the browser
import init, { PDFMarkdown } from './pdf2md_wasm.js';

let wasmInitialized = false;

self.onmessage = async (event) => {
  const { id, type, payload } = event.data;

  try {
    if (!wasmInitialized) {
      await init();
      wasmInitialized = true;
    }

    if (type === 'CONVERT') {
      const { bytes, options } = payload;
      const converter = new PDFMarkdown(options?.dialect, options?.detectTables);
      const result = converter.convert(new Uint8Array(bytes));
      
      self.postMessage({
        id,
        success: true,
        data: {
          markdown: result.markdown,
          diagnostics: JSON.parse(result.diagnostics),
        },
      });
    }
  } catch (err) {
    self.postMessage({
      id,
      success: false,
      error: err.message || String(err),
    });
  }
};
