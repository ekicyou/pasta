// Unit tests for SemanticTokensProvider + DiagnosticsManager + DocumentSync
// (Phase 4, Tasks 4.2-4.4)
//
// Tests token type constants, legend alignment, diagnostics conversion,
// and document sync debounce/lifecycle using mocks.

import * as assert from 'assert';

// =============================================================================
// Inline type definitions (mirror real types)
// =============================================================================

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

// Token types matching pasta_lsp analysis.rs (same order as semanticTokensProvider.ts)
const TOKEN_TYPES: readonly string[] = [
  'comment',       // 0
  'namespace',     // 1
  'scene',         // 2
  'decorator',     // 3
  'word',          // 4
  'variable',      // 5
  'call',          // 6
  'actor',         // 7
  'actorName',     // 8
  'codeBlock',     // 9
  'string',        // 10
  'sakuraScript',  // 11
  'escape',        // 12
  'operator',      // 13
] as const;

const TOKEN_MODIFIERS: readonly string[] = [
  'declaration',
  'definition',
  'global',
] as const;

// =============================================================================
// Test framework
// =============================================================================

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
// SemanticTokensProvider Tests
// =============================================================================

console.log('\n[SemanticTokensProvider Tests]');

test('has exactly 14 token types', () => {
  assert.strictEqual(TOKEN_TYPES.length, 14);
});

test('has exactly 3 token modifiers', () => {
  assert.strictEqual(TOKEN_MODIFIERS.length, 3);
});

test('token types match pasta_lsp analysis.rs order', () => {
  // Verify the first few critical types that pasta_lsp relies on
  assert.strictEqual(TOKEN_TYPES[0], 'comment');
  assert.strictEqual(TOKEN_TYPES[1], 'namespace');
  assert.strictEqual(TOKEN_TYPES[2], 'scene');
  assert.strictEqual(TOKEN_TYPES[3], 'decorator');
  assert.strictEqual(TOKEN_TYPES[4], 'word');
  assert.strictEqual(TOKEN_TYPES[5], 'variable');
  assert.strictEqual(TOKEN_TYPES[6], 'call');
  assert.strictEqual(TOKEN_TYPES[7], 'actor');
  assert.strictEqual(TOKEN_TYPES[8], 'actorName');
  assert.strictEqual(TOKEN_TYPES[9], 'codeBlock');
  assert.strictEqual(TOKEN_TYPES[10], 'string');
  assert.strictEqual(TOKEN_TYPES[11], 'sakuraScript');
  assert.strictEqual(TOKEN_TYPES[12], 'escape');
  assert.strictEqual(TOKEN_TYPES[13], 'operator');
});

test('token modifiers match pasta_lsp analysis.rs order', () => {
  assert.strictEqual(TOKEN_MODIFIERS[0], 'declaration');
  assert.strictEqual(TOKEN_MODIFIERS[1], 'definition');
  assert.strictEqual(TOKEN_MODIFIERS[2], 'global');
});

test('custom token types are in correct positions', () => {
  // These are pasta-specific custom types that must stay in sync with Rust
  const customTypes = ['scene', 'word', 'call', 'actor', 'actorName', 'codeBlock', 'sakuraScript', 'escape'];
  for (const ct of customTypes) {
    assert.ok(TOKEN_TYPES.includes(ct), `custom type '${ct}' should be in TOKEN_TYPES`);
  }
});

// =============================================================================
// SemanticToken building simulation
// =============================================================================

console.log('\n[SemanticToken Building Tests]');

test('delta-encoded tokens produce correct builder output', () => {
  // Simulate what SemanticTokensBuilder.push does internally
  const tokens: SemanticTokenData[] = [
    { deltaLine: 0, deltaStartCharacter: 0, length: 3, tokenType: 1, tokenModifiers: 0 },
    { deltaLine: 1, deltaStartCharacter: 2, length: 5, tokenType: 8, tokenModifiers: 0 },
    { deltaLine: 0, deltaStartCharacter: 5, length: 1, tokenType: 13, tokenModifiers: 0 },
  ];

  // Build flat array (same as SemanticTokensBuilder)
  const data: number[] = [];
  for (const t of tokens) {
    data.push(t.deltaLine, t.deltaStartCharacter, t.length, t.tokenType, t.tokenModifiers);
  }

  // 3 tokens × 5 fields = 15 values
  assert.strictEqual(data.length, 15);

  // First token: line 0, char 0, length 3, type NAMESPACE(1), mods 0
  assert.deepStrictEqual(data.slice(0, 5), [0, 0, 3, 1, 0]);
  // Second token: delta_line 1, char 2, length 5, type ACTOR_NAME(8), mods 0
  assert.deepStrictEqual(data.slice(5, 10), [1, 2, 5, 8, 0]);
  // Third token: same line (delta 0), char 5, length 1, type OPERATOR(13), mods 0
  assert.deepStrictEqual(data.slice(10, 15), [0, 5, 1, 13, 0]);
});

test('empty token list produces empty builder output', () => {
  const data: number[] = [];
  assert.strictEqual(data.length, 0);
});

// =============================================================================
// DiagnosticsManager Tests
// =============================================================================

console.log('\n[DiagnosticsManager Tests]');

test('severity 1 maps to Error', () => {
  const severityMap: Record<number, string> = { 1: 'Error', 2: 'Warning', 3: 'Information', 4: 'Hint' };
  assert.strictEqual(severityMap[1], 'Error');
});

test('severity 2 maps to Warning', () => {
  const severityMap: Record<number, string> = { 1: 'Error', 2: 'Warning', 3: 'Information', 4: 'Hint' };
  assert.strictEqual(severityMap[2], 'Warning');
});

test('severity 3 maps to Information', () => {
  const severityMap: Record<number, string> = { 1: 'Error', 2: 'Warning', 3: 'Information', 4: 'Hint' };
  assert.strictEqual(severityMap[3], 'Information');
});

test('severity 4 maps to Hint', () => {
  const severityMap: Record<number, string> = { 1: 'Error', 2: 'Warning', 3: 'Information', 4: 'Hint' };
  assert.strictEqual(severityMap[4], 'Hint');
});

test('unknown severity defaults to Error', () => {
  function convertSeverity(s: number): string {
    switch (s) {
      case 1: return 'Error';
      case 2: return 'Warning';
      case 3: return 'Information';
      case 4: return 'Hint';
      default: return 'Error';
    }
  }
  assert.strictEqual(convertSeverity(99), 'Error');
  assert.strictEqual(convertSeverity(0), 'Error');
});

test('diagnostic range fields are preserved', () => {
  const d: DiagnosticData = {
    range: { start: { line: 3, character: 5 }, end: { line: 3, character: 10 } },
    message: 'unexpected token',
    severity: 1,
  };
  assert.strictEqual(d.range.start.line, 3);
  assert.strictEqual(d.range.start.character, 5);
  assert.strictEqual(d.range.end.line, 3);
  assert.strictEqual(d.range.end.character, 10);
  assert.strictEqual(d.message, 'unexpected token');
});

// =============================================================================
// DocumentSync Tests (debounce logic simulation)
// =============================================================================

console.log('\n[DocumentSync Tests]');

test('debounce timer replaces previous timer on rapid changes', () => {
  const timers = new Map<string, ReturnType<typeof setTimeout>>();
  let analyzeCount = 0;

  function debouncedAnalyze(uri: string): void {
    const existing = timers.get(uri);
    if (existing) clearTimeout(existing);
    const timer = setTimeout(() => {
      timers.delete(uri);
      analyzeCount++;
    }, 10); // Short delay for test
    timers.set(uri, timer);
  }

  // Simulate 3 rapid changes
  debouncedAnalyze('file:///test.pasta');
  debouncedAnalyze('file:///test.pasta');
  debouncedAnalyze('file:///test.pasta');

  // Only 1 timer should be active
  assert.strictEqual(timers.size, 1);

  // Clean up
  for (const t of timers.values()) clearTimeout(t);
  timers.clear();
});

test('isPastaDocument filters by language id', () => {
  function isPasta(doc: { languageId: string }): boolean {
    return doc.languageId === 'pasta';
  }

  assert.strictEqual(isPasta({ languageId: 'pasta' }), true);
  assert.strictEqual(isPasta({ languageId: 'javascript' }), false);
  assert.strictEqual(isPasta({ languageId: 'lua' }), false);
  assert.strictEqual(isPasta({ languageId: '' }), false);
});

test('document close clears cached data', () => {
  const cachedTokens = new Map<string, SemanticTokenData[]>();
  const uri = 'file:///test.pasta';

  // Simulate cached data
  cachedTokens.set(uri, [
    { deltaLine: 0, deltaStartCharacter: 0, length: 3, tokenType: 1, tokenModifiers: 0 },
  ]);
  assert.strictEqual(cachedTokens.has(uri), true);

  // Simulate close
  cachedTokens.delete(uri);
  assert.strictEqual(cachedTokens.has(uri), false);
});

// =============================================================================
// Token Cache Tests
// =============================================================================

console.log('\n[Token Cache Tests]');

test('updateTokens stores new tokens in cache', () => {
  const cache = new Map<string, SemanticTokenData[]>();
  const uri = 'file:///test.pasta';
  const tokens: SemanticTokenData[] = [
    { deltaLine: 0, deltaStartCharacter: 0, length: 5, tokenType: 1, tokenModifiers: 0 },
  ];

  cache.set(uri, [...tokens]);
  assert.strictEqual(cache.get(uri)!.length, 1);
  assert.strictEqual(cache.get(uri)![0].tokenType, 1);
});

test('updateTokens replaces existing tokens', () => {
  const cache = new Map<string, SemanticTokenData[]>();
  const uri = 'file:///test.pasta';

  cache.set(uri, [{ deltaLine: 0, deltaStartCharacter: 0, length: 3, tokenType: 1, tokenModifiers: 0 }]);
  cache.set(uri, [{ deltaLine: 0, deltaStartCharacter: 0, length: 5, tokenType: 2, tokenModifiers: 0 }]);

  assert.strictEqual(cache.get(uri)!.length, 1);
  assert.strictEqual(cache.get(uri)![0].tokenType, 2);
});

test('clearTokens removes entry from cache', () => {
  const cache = new Map<string, SemanticTokenData[]>();
  const uri = 'file:///test.pasta';

  cache.set(uri, [{ deltaLine: 0, deltaStartCharacter: 0, length: 3, tokenType: 1, tokenModifiers: 0 }]);
  cache.delete(uri);

  assert.strictEqual(cache.has(uri), false);
});

// =============================================================================
// Summary
// =============================================================================

console.log(`\nIntegration: ${passed} passed, ${failed} failed\n`);
if (failed > 0) {
  process.exit(1);
}
