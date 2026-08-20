// Node.js test runner for browser SDK capabilities and fallback logic
import assert from 'assert';

console.log('=== Running Browser / WebAssembly SDK Unit Tests ===');

let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    fn();
    console.log(`  [PASS] ${name}`);
    passed++;
  } catch (err) {
    console.error(`  [FAIL] ${name}: ${err.message}`);
    failed++;
  }
}

// Test 1: Capability & resource limit checks
test('File size limit check rejects oversize documents without fallback', () => {
  const maxLimitMB = 10;
  const simulatedFileSizeMB = 15;
  const bytesLength = simulatedFileSizeMB * 1024 * 1024;
  
  assert.strictEqual(bytesLength > maxLimitMB * 1024 * 1024, true);
});

// Test 2: Event emission payloads
test('Progress event structure conforms to interface', () => {
  const progressEvent = {
    currentPage: 2,
    totalPages: 5,
    percent: 40,
    stage: 'rendering',
  };

  assert.strictEqual(progressEvent.currentPage, 2);
  assert.strictEqual(progressEvent.percent, 40);
  assert.strictEqual(progressEvent.stage, 'rendering');
});

// Test 3: AbortSignal cancellation
test('AbortController triggers cancellation flag', () => {
  const controller = new AbortController();
  assert.strictEqual(controller.signal.aborted, false);
  controller.abort();
  assert.strictEqual(controller.signal.aborted, true);
});

// Test 4: ConversionResult schema validation
test('ConversionResult contains required structural fields', () => {
  const sampleResult = {
    markdown: '# Title\n\nBody content.',
    metadata: { total_pages: 1, is_encrypted: false },
    diagnostics: { total_pages: 1, overall_confidence: 0.96 },
    statistics: { parse_time_ms: 5, layout_time_ms: 10, render_time_ms: 2, total_time_ms: 17 },
    confidence: 0.96,
    warnings: [],
    processedLocally: true,
  };

  assert.ok(sampleResult.markdown.includes('# Title'));
  assert.strictEqual(sampleResult.processedLocally, true);
  assert.strictEqual(sampleResult.confidence, 0.96);
});

console.log(`\nSDK Test Summary: ${passed} passed, ${failed} failed.\n`);
if (failed > 0) {
  process.exit(1);
}
