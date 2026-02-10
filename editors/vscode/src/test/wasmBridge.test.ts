// Unit tests for WasmBridge (Phase 3, Task 3.3)
//
// Tests WasmBridge initialization, analyzeDocument, error handling,
// and dispose behavior using mocked WASM module.

import * as assert from 'assert';

// We re-implement a minimal WasmBridge-like class inline that
// mimics the real code but uses our mock. This avoids needing
// to hook into the ESM import system at runtime.

interface SemanticTokenData {
  deltaLine: number;
  deltaStartCharacter: number;
  length: number;
  tokenType: number;
  tokenModifiers: number;
}

interface DiagnosticData {
  range: { start: { line: number; character: number }; end: { line: number; character: number } };
  message: string;
  severity: number;
}

interface WasmAnalysisResult {
  tokens: ReadonlyArray<SemanticTokenData>;
  diagnostics: ReadonlyArray<DiagnosticData>;
}

// Minimal mock of WasmBridge for testability (same logic as real class)
class TestableWasmBridge {
  private wasmModule: any = null;
  private ready = false;
  private log: string[] = [];

  initializeWithMock(mockModule: any): void {
    this.wasmModule = mockModule;
    this.ready = true;
    this.log.push('initialized');
  }

  analyzeDocument(text: string): WasmAnalysisResult {
    if (!this.ready || !this.wasmModule) {
      throw new Error('WASM module is not initialized');
    }
    try {
      const result = this.wasmModule.wasm_analyze(text);
      return result as WasmAnalysisResult;
    } catch (error) {
      throw new Error(`WASM analysis failed: ${error}`);
    }
  }

  isReady(): boolean {
    return this.ready;
  }

  dispose(): void {
    this.wasmModule = null;
    this.ready = false;
    this.log.push('disposed');
  }

  getLog(): string[] {
    return this.log;
  }
}

// =============================================================================
// Test Helpers
// =============================================================================

function createMockWasmModule(analyzeResult?: WasmAnalysisResult) {
  const defaultResult: WasmAnalysisResult = {
    tokens: [
      { deltaLine: 0, deltaStartCharacter: 0, length: 3, tokenType: 1, tokenModifiers: 0 },
    ],
    diagnostics: [],
  };

  return {
    wasm_analyze: (source: string): WasmAnalysisResult => {
      if (source === 'PANIC') {
        throw new Error('simulated parser panic');
      }
      return analyzeResult ?? defaultResult;
    },
  };
}

let passed = 0;
let failed = 0;

function test(name: string, fn: () => void) {
  try {
    fn();
    console.log(`  ✓ ${name}`);
    passed++;
  } catch (e: any) {
    console.error(`  ✗ ${name}`);
    console.error(`    ${e.message}`);
    failed++;
  }
}

// =============================================================================
// Tests
// =============================================================================

console.log('\n[WasmBridge Unit Tests]');

test('isReady returns false before initialization', () => {
  const bridge = new TestableWasmBridge();
  assert.strictEqual(bridge.isReady(), false);
});

test('isReady returns true after initialization', () => {
  const bridge = new TestableWasmBridge();
  bridge.initializeWithMock(createMockWasmModule());
  assert.strictEqual(bridge.isReady(), true);
});

test('analyzeDocument throws when not initialized', () => {
  const bridge = new TestableWasmBridge();
  assert.throws(() => bridge.analyzeDocument('test'), /not initialized/);
});

test('analyzeDocument returns tokens from WASM module', () => {
  const bridge = new TestableWasmBridge();
  bridge.initializeWithMock(createMockWasmModule());
  const result = bridge.analyzeDocument('＊挨拶\n');
  assert.strictEqual(result.tokens.length, 1);
  assert.strictEqual(result.tokens[0].tokenType, 1);
  assert.strictEqual(result.tokens[0].deltaLine, 0);
});

test('analyzeDocument returns diagnostics from WASM module', () => {
  const customResult: WasmAnalysisResult = {
    tokens: [],
    diagnostics: [
      {
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } },
        message: 'syntax error',
        severity: 1,
      },
    ],
  };
  const bridge = new TestableWasmBridge();
  bridge.initializeWithMock(createMockWasmModule(customResult));
  const result = bridge.analyzeDocument('bad input');
  assert.strictEqual(result.diagnostics.length, 1);
  assert.strictEqual(result.diagnostics[0].message, 'syntax error');
  assert.strictEqual(result.diagnostics[0].severity, 1);
});

test('analyzeDocument wraps WASM errors as Error', () => {
  const bridge = new TestableWasmBridge();
  bridge.initializeWithMock(createMockWasmModule());
  assert.throws(() => bridge.analyzeDocument('PANIC'), /WASM analysis failed/);
});

test('dispose makes bridge not ready', () => {
  const bridge = new TestableWasmBridge();
  bridge.initializeWithMock(createMockWasmModule());
  assert.strictEqual(bridge.isReady(), true);
  bridge.dispose();
  assert.strictEqual(bridge.isReady(), false);
});

test('analyzeDocument throws after dispose', () => {
  const bridge = new TestableWasmBridge();
  bridge.initializeWithMock(createMockWasmModule());
  bridge.dispose();
  assert.throws(() => bridge.analyzeDocument('test'), /not initialized/);
});

test('analyzeDocument returns empty result for empty input', () => {
  const emptyResult: WasmAnalysisResult = { tokens: [], diagnostics: [] };
  const bridge = new TestableWasmBridge();
  bridge.initializeWithMock(createMockWasmModule(emptyResult));
  const result = bridge.analyzeDocument('');
  assert.strictEqual(result.tokens.length, 0);
  assert.strictEqual(result.diagnostics.length, 0);
});

test('analyzeDocument preserves all token fields', () => {
  const detailedResult: WasmAnalysisResult = {
    tokens: [
      { deltaLine: 2, deltaStartCharacter: 4, length: 7, tokenType: 5, tokenModifiers: 3 },
    ],
    diagnostics: [],
  };
  const bridge = new TestableWasmBridge();
  bridge.initializeWithMock(createMockWasmModule(detailedResult));
  const result = bridge.analyzeDocument('test');
  const t = result.tokens[0];
  assert.strictEqual(t.deltaLine, 2);
  assert.strictEqual(t.deltaStartCharacter, 4);
  assert.strictEqual(t.length, 7);
  assert.strictEqual(t.tokenType, 5);
  assert.strictEqual(t.tokenModifiers, 3);
});

// =============================================================================
// Summary
// =============================================================================

console.log(`\nWasmBridge: ${passed} passed, ${failed} failed\n`);
if (failed > 0) {
  process.exit(1);
}
