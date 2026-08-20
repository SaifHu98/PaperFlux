import init, { PDFMarkdown as WasmEngine } from './pdf2md_wasm.js';

let wasmReady: Promise<void> | null = null;

async function ensureWasm() {
  if (!wasmReady) {
    wasmReady = init();
  }
  await wasmReady;
}

self.onmessage = async (event: MessageEvent) => {
  const { id, type, payload } = event.data;

  try {
    await ensureWasm();

    if (type === 'ASSESS_CAPABILITIES') {
      const { bytes } = payload;
      const engine = new WasmEngine();
      const capabilities = engine.assess_capabilities(new Uint8Array(bytes));
      self.postMessage({ id, success: true, capabilities });
      return;
    }

    if (type === 'CONVERT_PROGRESSIVE') {
      const { bytes, options } = payload;
      const engine = new WasmEngine(options?.dialect, options?.detectTables);
      const uint8 = new Uint8Array(bytes);

      const caps = engine.assess_capabilities(uint8);
      const totalPages = caps.pageCount || 1;

      let fullMarkdown = '';

      for (let pageNum = 1; pageNum <= totalPages; pageNum++) {
        // Send page start
        self.postMessage({
          id,
          event: 'PAGE_START',
          pageNumber: pageNum,
          totalPages,
        });

        const pageRes = engine.convert_page(uint8, pageNum);
        fullMarkdown += (pageNum > 1 ? '\n<!-- pagebreak -->\n\n' : '') + pageRes.markdown;

        // Send page complete
        self.postMessage({
          id,
          event: 'PAGE_COMPLETE',
          pageNumber: pageNum,
          totalPages,
          pageMarkdown: pageRes.markdown,
          percent: Math.round((pageNum / totalPages) * 100),
        });
      }

      // Convert full document for final diagnostics and metadata
      const fullRes = engine.convert(uint8);

      self.postMessage({
        id,
        success: true,
        data: {
          markdown: fullMarkdown || fullRes.markdown,
          metadata: JSON.parse(fullRes.metadata),
          diagnostics: JSON.parse(fullRes.diagnostics),
          confidence: fullRes.overallConfidence,
          totalPages: fullRes.totalPages,
        },
      });
      return;
    }

    if (type === 'CONVERT_DIRECT') {
      const { bytes, options } = payload;
      const engine = new WasmEngine(options?.dialect, options?.detectTables);
      const fullRes = engine.convert(new Uint8Array(bytes));

      self.postMessage({
        id,
        success: true,
        data: {
          markdown: fullRes.markdown,
          metadata: JSON.parse(fullRes.metadata),
          diagnostics: JSON.parse(fullRes.diagnostics),
          confidence: fullRes.overallConfidence,
          totalPages: fullRes.totalPages,
        },
      });
      return;
    }
  } catch (err: any) {
    self.postMessage({
      id,
      success: false,
      error: err.message || String(err),
    });
  }
};
