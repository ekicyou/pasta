// Real-module unit tests (review-improvement-loop cell 3.55 / G1).
//
// This file imports the REAL modules and runs them against the vscode mock in
// ./mock/vscode.ts, wired in via
//   esbuild --alias:vscode=./src/test/mock/vscode.ts
// so drift between production code and test expectations cannot go unnoticed.
// (Cell 3.57 / G4 removed the historical wasmBridge.test.ts /
// integration.test.ts files, which exercised inline COPIES of the production
// logic — one such mirror had already drifted: 15 vs 17 token types. Their
// assertions are all subsumed by the real-module tests below.)
//
// Covered:
//   - semanticTokensProvider.ts : PASTA_TOKEN_TYPES/MODIFIERS contracts,
//     delta→absolute conversion, cache behaviour, error paths
//   - diagnosticsManager.ts     : LSP→VSCode severity/range conversion, clear
//   - documentSync.ts           : open/change/close lifecycle, 200ms debounce,
//     analysis-failure fallback, dispose
//   - wasmBridge.ts             : init/dispose lifecycle, error wrapping,
//     wasm-boundary result-shape validation
//   - debugAdapterFactory.ts    : PastaDebugAdapterFactory descriptor and
//     PastaDebugConfigurationProvider only-when-explicit normalisation
//   - wordRefDecorator.ts       : @word-ref decoration ranges (regex behaviour)
//   - extension.ts              : activate (real WASM load), source-presentation
//     toggle wiring (status bar, push-event-driven display, error surfaces)

import * as assert from 'assert';
import * as fs from 'fs';
import * as path from 'path';
import * as mock from './mock/vscode';
import {
  PASTA_TOKEN_TYPES,
  PASTA_TOKEN_MODIFIERS,
  PASTA_TOKENS_LEGEND,
  SemanticTokensProvider,
} from '../semanticTokensProvider';
import { DiagnosticsManager } from '../diagnosticsManager';
import { WasmBridge } from '../wasmBridge';
import { DocumentSync } from '../documentSync';
import {
  PastaDebugAdapterFactory,
  PastaDebugConfigurationProvider,
} from '../debugAdapterFactory';
import { activateWordRefDecorator } from '../wordRefDecorator';
import { activate, deactivate, getActivationState } from '../extension';

const ROOT = path.resolve(__dirname, '..', '..');

// =============================================================================
// Async-aware sequential test runner
// =============================================================================

let passed = 0;
let failed = 0;
const queue: Array<{ name: string; fn: () => void | Promise<void> }> = [];

function test(name: string, fn: () => void | Promise<void>): void {
  queue.push({ name, fn });
}

const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

// =============================================================================
// Shared fakes
// =============================================================================

interface FakeAnalysisResult {
  tokens: Array<{
    deltaLine: number;
    deltaStartCharacter: number;
    length: number;
    tokenType: number;
    tokenModifiers: number;
  }>;
  diagnostics: Array<{
    range: { start: { line: number; character: number }; end: { line: number; character: number } };
    message: string;
    severity: number;
  }>;
}

/** Duck-typed WasmBridge double (real WasmBridge requires the vscode runtime). */
function fakeBridge(opts: { ready?: boolean; result?: FakeAnalysisResult; fail?: boolean } = {}) {
  const bridge = {
    calls: [] as string[],
    isReady: () => opts.ready !== false,
    analyzeDocument(text: string): FakeAnalysisResult {
      bridge.calls.push(text);
      if (opts.fail) {
        throw new Error('simulated analysis failure');
      }
      return opts.result ?? { tokens: [], diagnostics: [] };
    },
  };
  return bridge;
}

/** Minimal pasta TextDocument double for DocumentSync / providers. */
function fakeDocument(uriStr: string, languageId: string, text: string) {
  return {
    uri: mock.Uri.parse(uriStr),
    languageId,
    getText: () => text,
    positionAt: (offset: number) => new mock.Position(0, offset),
  };
}

const dummyToken = {} as never;
const dummyContext = () => ({ subscriptions: [] as Array<{ dispose(): void }> });

// =============================================================================
// Token type / modifier contracts (real PASTA_TOKEN_TYPES)
// =============================================================================

test('PASTA_TOKEN_TYPES has the full 17-entry order (incl. cueMarker/cueCommand)', () => {
  assert.deepStrictEqual(
    [...PASTA_TOKEN_TYPES],
    [
      'comment', 'namespace', 'scene', 'decorator', 'word', 'variable',
      'call', 'actor', 'actorName', 'codeBlock', 'talk', 'sakuraScript',
      'escape', 'operator', 'number', 'cueMarker', 'cueCommand',
    ]
  );
});

test('PASTA_TOKEN_MODIFIERS has the 3-entry order', () => {
  assert.deepStrictEqual([...PASTA_TOKEN_MODIFIERS], ['declaration', 'definition', 'global']);
});

test('PASTA_TOKENS_LEGEND mirrors the type/modifier arrays', () => {
  assert.deepStrictEqual(PASTA_TOKENS_LEGEND.tokenTypes, [...PASTA_TOKEN_TYPES]);
  assert.deepStrictEqual(PASTA_TOKENS_LEGEND.tokenModifiers, [...PASTA_TOKEN_MODIFIERS]);
});

test('package.json semanticTokenScopes covers exactly PASTA_TOKEN_TYPES', () => {
  const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, 'package.json'), 'utf-8'));
  const scopes = pkg.contributes.semanticTokenScopes[0].scopes as Record<string, string[]>;
  for (const t of PASTA_TOKEN_TYPES) {
    assert.ok(scopes[t], `package.json should map a TextMate scope for token type '${t}'`);
  }
  for (const key of Object.keys(scopes)) {
    assert.ok(
      (PASTA_TOKEN_TYPES as readonly string[]).includes(key),
      `package.json maps unknown token type '${key}' (drift against PASTA_TOKEN_TYPES)`
    );
  }
});

test('package.json custom semanticTokenTypes are all members of PASTA_TOKEN_TYPES', () => {
  const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, 'package.json'), 'utf-8'));
  const ids = pkg.contributes.semanticTokenTypes.map((t: { id: string }) => t.id) as string[];
  for (const id of ids) {
    assert.ok(
      (PASTA_TOKEN_TYPES as readonly string[]).includes(id),
      `declared custom token type '${id}' is not in PASTA_TOKEN_TYPES`
    );
  }
});

// Anti-drift check against the Rust source of truth. Registered only when the
// pasta_lsp sources are present (keeps the test portable if the extension is
// ever split out of the monorepo).
const RUST_TOKEN_TYPES_PATH = path.resolve(
  ROOT, '..', '..', 'crates', 'pasta_lsp', 'src', 'analysis', 'token_types.rs'
);
if (fs.existsSync(RUST_TOKEN_TYPES_PATH)) {
  test('PASTA_TOKEN_TYPES matches pasta_lsp token_types.rs TOKEN_TYPES order', () => {
    const src = fs.readFileSync(RUST_TOKEN_TYPES_PATH, 'utf-8');
    const block = src.match(/pub const TOKEN_TYPES[^=]*=\s*&\[([\s\S]*?)\];/);
    assert.ok(block, 'TOKEN_TYPES const block should be found in token_types.rs');
    const builtin: Record<string, string> = {
      COMMENT: 'comment', NAMESPACE: 'namespace', DECORATOR: 'decorator',
      VARIABLE: 'variable', OPERATOR: 'operator', NUMBER: 'number',
    };
    const names: string[] = [];
    const re = /SemanticTokenType::(?:new\("([^"]+)"\)|([A-Z_]+))/g;
    let m: RegExpExecArray | null;
    while ((m = re.exec(block![1])) !== null) {
      names.push(m[1] ?? builtin[m[2]] ?? `<unmapped:${m[2]}>`);
    }
    assert.deepStrictEqual([...PASTA_TOKEN_TYPES], names,
      'TS PASTA_TOKEN_TYPES must stay index-aligned with Rust TOKEN_TYPES');
  });

  test('PASTA_TOKEN_MODIFIERS matches pasta_lsp token_types.rs TOKEN_MODIFIERS order', () => {
    const src = fs.readFileSync(RUST_TOKEN_TYPES_PATH, 'utf-8');
    const block = src.match(/pub const TOKEN_MODIFIERS[^=]*=\s*&\[([\s\S]*?)\];/);
    assert.ok(block, 'TOKEN_MODIFIERS const block should be found in token_types.rs');
    const builtin: Record<string, string> = { DECLARATION: 'declaration', DEFINITION: 'definition' };
    const names: string[] = [];
    const re = /SemanticTokenModifier::(?:new\("([^"]+)"\)|([A-Z_]+))/g;
    let m: RegExpExecArray | null;
    while ((m = re.exec(block![1])) !== null) {
      names.push(m[1] ?? builtin[m[2]] ?? `<unmapped:${m[2]}>`);
    }
    assert.deepStrictEqual([...PASTA_TOKEN_MODIFIERS], names);
  });
}

// =============================================================================
// SemanticTokensProvider (real)
// =============================================================================

test('provider returns null when the bridge is not ready', () => {
  const provider = new SemanticTokensProvider(fakeBridge({ ready: false }) as never);
  const doc = fakeDocument('file:///a.pasta', 'pasta', '＊挨拶');
  assert.strictEqual(provider.provideDocumentSemanticTokens(doc as never, dummyToken), null);
});

test('provider converts delta-encoded tokens to absolute builder positions', () => {
  // pasta_lsp emits LSP delta encoding; vscode SemanticTokensBuilder.push()
  // expects ABSOLUTE positions. deltaLine>0 must reset the character base,
  // deltaLine==0 must accumulate it.
  const result: FakeAnalysisResult = {
    tokens: [
      { deltaLine: 0, deltaStartCharacter: 2, length: 3, tokenType: 1, tokenModifiers: 0 },
      { deltaLine: 1, deltaStartCharacter: 4, length: 5, tokenType: 8, tokenModifiers: 2 },
      { deltaLine: 0, deltaStartCharacter: 3, length: 1, tokenType: 13, tokenModifiers: 0 },
    ],
    diagnostics: [],
  };
  const provider = new SemanticTokensProvider(fakeBridge({ result }) as never);
  const doc = fakeDocument('file:///delta.pasta', 'pasta', 'dummy');
  const tokens = provider.provideDocumentSemanticTokens(doc as never, dummyToken) as mock.SemanticTokens;
  assert.ok(tokens, 'should build tokens');
  assert.deepStrictEqual(
    Array.from(tokens.data),
    [
      0, 2, 3, 1, 0,  // line 0, char 2 (absolute)
      1, 4, 5, 8, 2,  // line 1, char 4 (deltaLine>0 resets char base)
      1, 7, 1, 13, 0, // line 1, char 4+3=7 (same line accumulates)
    ]
  );
});

test('provider caches on-demand analysis (no re-analysis for same uri)', () => {
  const bridge = fakeBridge({ result: { tokens: [], diagnostics: [] } });
  const provider = new SemanticTokensProvider(bridge as never);
  const doc = fakeDocument('file:///cache.pasta', 'pasta', 'text');
  provider.provideDocumentSemanticTokens(doc as never, dummyToken);
  provider.provideDocumentSemanticTokens(doc as never, dummyToken);
  assert.strictEqual(bridge.calls.length, 1, 'second call must hit the cache');
});

test('clearTokens drops the cache and forces re-analysis', () => {
  const bridge = fakeBridge({ result: { tokens: [], diagnostics: [] } });
  const provider = new SemanticTokensProvider(bridge as never);
  const doc = fakeDocument('file:///clear.pasta', 'pasta', 'text');
  provider.provideDocumentSemanticTokens(doc as never, dummyToken);
  provider.clearTokens(doc.uri.toString());
  provider.provideDocumentSemanticTokens(doc as never, dummyToken);
  assert.strictEqual(bridge.calls.length, 2);
});

test('updateTokens replaces the cache and fires onDidChangeSemanticTokens', () => {
  const bridge = fakeBridge({ result: { tokens: [], diagnostics: [] } });
  const provider = new SemanticTokensProvider(bridge as never);
  let fired = 0;
  provider.onDidChangeSemanticTokens(() => { fired++; });
  const doc = fakeDocument('file:///update.pasta', 'pasta', 'text');
  provider.updateTokens(doc.uri.toString(), [
    { deltaLine: 0, deltaStartCharacter: 1, length: 2, tokenType: 4, tokenModifiers: 0 },
  ]);
  assert.strictEqual(fired, 1, 'event must fire on updateTokens');
  const tokens = provider.provideDocumentSemanticTokens(doc as never, dummyToken) as mock.SemanticTokens;
  assert.deepStrictEqual(Array.from(tokens.data), [0, 1, 2, 4, 0]);
  assert.strictEqual(bridge.calls.length, 0, 'cached tokens must be served without analysis');
});

test('provider returns null when analysis throws (TextMate fallback)', () => {
  const provider = new SemanticTokensProvider(fakeBridge({ fail: true }) as never);
  const doc = fakeDocument('file:///fail.pasta', 'pasta', 'text');
  assert.strictEqual(provider.provideDocumentSemanticTokens(doc as never, dummyToken), null);
});

// =============================================================================
// DiagnosticsManager (real)
// =============================================================================

function collectionOf(dm: DiagnosticsManager): mock.MockDiagnosticCollection {
  return (dm as unknown as { collection: mock.MockDiagnosticCollection }).collection;
}

test('setDiagnostics maps LSP severities 1..4 to VSCode severities', () => {
  const dm = new DiagnosticsManager();
  const uri = mock.Uri.parse('file:///sev.pasta');
  const mk = (severity: number) => ({
    range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
    message: `sev ${severity}`,
    severity,
  });
  dm.setDiagnostics(uri as never, [mk(1), mk(2), mk(3), mk(4)]);
  const got = collectionOf(dm).get(uri)!;
  assert.strictEqual(got.length, 4);
  assert.strictEqual(got[0].severity, mock.DiagnosticSeverity.Error);
  assert.strictEqual(got[1].severity, mock.DiagnosticSeverity.Warning);
  assert.strictEqual(got[2].severity, mock.DiagnosticSeverity.Information);
  assert.strictEqual(got[3].severity, mock.DiagnosticSeverity.Hint);
  dm.dispose();
});

test('setDiagnostics defaults unknown LSP severity to Error', () => {
  const dm = new DiagnosticsManager();
  const uri = mock.Uri.parse('file:///sev0.pasta');
  dm.setDiagnostics(uri as never, [{
    range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
    message: 'weird severity',
    severity: 99,
  }]);
  assert.strictEqual(collectionOf(dm).get(uri)![0].severity, mock.DiagnosticSeverity.Error);
  dm.dispose();
});

test('setDiagnostics preserves range and message', () => {
  const dm = new DiagnosticsManager();
  const uri = mock.Uri.parse('file:///range.pasta');
  dm.setDiagnostics(uri as never, [{
    range: { start: { line: 3, character: 5 }, end: { line: 4, character: 2 } },
    message: 'unexpected token',
    severity: 1,
  }]);
  const d = collectionOf(dm).get(uri)![0];
  assert.strictEqual(d.message, 'unexpected token');
  assert.strictEqual(d.range.start.line, 3);
  assert.strictEqual(d.range.start.character, 5);
  assert.strictEqual(d.range.end.line, 4);
  assert.strictEqual(d.range.end.character, 2);
  dm.dispose();
});

test('setDiagnostics replaces, clear deletes, clearAll empties', () => {
  const dm = new DiagnosticsManager();
  const a = mock.Uri.parse('file:///a.pasta');
  const b = mock.Uri.parse('file:///b.pasta');
  const diag = (msg: string) => ({
    range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
    message: msg,
    severity: 1,
  });
  dm.setDiagnostics(a as never, [diag('one'), diag('two')]);
  dm.setDiagnostics(a as never, [diag('three')]);
  assert.strictEqual(collectionOf(dm).get(a)!.length, 1, 'set must replace, not append');
  dm.setDiagnostics(b as never, [diag('four')]);
  dm.clear(a as never);
  assert.strictEqual(collectionOf(dm).get(a), undefined);
  assert.strictEqual(collectionOf(dm).get(b)!.length, 1);
  dm.clearAll();
  assert.strictEqual(collectionOf(dm).size, 0);
  dm.dispose();
});

// =============================================================================
// DocumentSync (real) — lifecycle + debounce
// =============================================================================

function makeSyncHarness(bridgeOpts: Parameters<typeof fakeBridge>[0] = {}) {
  const bridge = fakeBridge({
    result: {
      tokens: [{ deltaLine: 0, deltaStartCharacter: 0, length: 1, tokenType: 1, tokenModifiers: 0 }],
      diagnostics: [{
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
        message: 'diag',
        severity: 1,
      }],
    },
    ...bridgeOpts,
  });
  const provider = new SemanticTokensProvider(bridge as never);
  const dm = new DiagnosticsManager();
  const logged: string[] = [];
  const channel = { name: 'test', appendLine: (msg: string) => { logged.push(msg); }, dispose: () => {} };
  const sync = new DocumentSync(bridge as never, provider, dm, channel as never);
  return { bridge, provider, dm, logged, sync };
}

test('activate analyzes already-open pasta documents and skips other languages', () => {
  const h = makeSyncHarness();
  mock.workspace.textDocuments = [
    fakeDocument('file:///open.pasta', 'pasta', 'pasta text'),
    fakeDocument('file:///open.lua', 'lua', 'lua text'),
  ];
  h.sync.activate(dummyContext() as never);
  assert.deepStrictEqual(h.bridge.calls, ['pasta text']);
  h.sync.dispose();
  h.dm.dispose();
  mock.workspace.textDocuments = [];
});

test('document open triggers immediate analysis; non-pasta open is ignored', () => {
  const h = makeSyncHarness();
  h.sync.activate(dummyContext() as never);
  mock.didOpenTextDocumentEmitter.fire(fakeDocument('file:///x.pasta', 'pasta', 'opened'));
  mock.didOpenTextDocumentEmitter.fire(fakeDocument('file:///y.txt', 'plaintext', 'nope'));
  assert.deepStrictEqual(h.bridge.calls, ['opened']);
  // analysis results must reach both consumers
  assert.ok(collectionOf(h.dm).get(mock.Uri.parse('file:///x.pasta')), 'diagnostics must be set');
  h.sync.dispose();
  h.dm.dispose();
});

test('rapid document changes are debounced to a single analysis (200ms)', async () => {
  const h = makeSyncHarness();
  h.sync.activate(dummyContext() as never);
  const doc = fakeDocument('file:///debounce.pasta', 'pasta', 'changed');
  mock.didChangeTextDocumentEmitter.fire({ document: doc });
  mock.didChangeTextDocumentEmitter.fire({ document: doc });
  mock.didChangeTextDocumentEmitter.fire({ document: doc });
  assert.strictEqual(h.bridge.calls.length, 0, 'analysis must not run synchronously');
  await sleep(320);
  assert.strictEqual(h.bridge.calls.length, 1, 'exactly one analysis after the debounce window');
  h.sync.dispose();
  h.dm.dispose();
});

test('document close cancels the pending debounce and clears cached state', async () => {
  const h = makeSyncHarness();
  h.sync.activate(dummyContext() as never);
  const doc = fakeDocument('file:///close.pasta', 'pasta', 'text');
  // Seed cached state via an open analysis
  mock.didOpenTextDocumentEmitter.fire(doc);
  assert.strictEqual(h.bridge.calls.length, 1);
  assert.ok(collectionOf(h.dm).get(doc.uri), 'diagnostics seeded');
  // Pending change, then close before the debounce fires
  mock.didChangeTextDocumentEmitter.fire({ document: doc });
  mock.didCloseTextDocumentEmitter.fire(doc);
  await sleep(320);
  assert.strictEqual(h.bridge.calls.length, 1, 'close must cancel the pending analysis');
  assert.strictEqual(collectionOf(h.dm).get(doc.uri), undefined, 'close must clear diagnostics');
  // Token cache cleared → next provide re-analyzes
  h.provider.provideDocumentSemanticTokens(doc as never, dummyToken);
  assert.strictEqual(h.bridge.calls.length, 2, 'token cache must have been cleared on close');
  h.sync.dispose();
  h.dm.dispose();
});

test('analysis failure logs to the output channel and clears diagnostics', () => {
  const h = makeSyncHarness({ fail: true });
  h.sync.activate(dummyContext() as never);
  const doc = fakeDocument('file:///err.pasta', 'pasta', 'bad');
  mock.didOpenTextDocumentEmitter.fire(doc);
  assert.ok(
    h.logged.some((l) => l.includes('Document analysis failed')),
    'failure must be logged'
  );
  assert.strictEqual(collectionOf(h.dm).get(doc.uri), undefined, 'diagnostics must be cleared on failure');
  h.sync.dispose();
  h.dm.dispose();
});

test('no analysis is attempted while the bridge is not ready', () => {
  const h = makeSyncHarness({ ready: false });
  h.sync.activate(dummyContext() as never);
  mock.didOpenTextDocumentEmitter.fire(fakeDocument('file:///nr.pasta', 'pasta', 'text'));
  assert.strictEqual(h.bridge.calls.length, 0);
  h.sync.dispose();
  h.dm.dispose();
});

test('dispose cancels pending timers and unsubscribes listeners', async () => {
  const h = makeSyncHarness();
  h.sync.activate(dummyContext() as never);
  const doc = fakeDocument('file:///disp.pasta', 'pasta', 'text');
  mock.didChangeTextDocumentEmitter.fire({ document: doc });
  h.sync.dispose();
  await sleep(320);
  assert.strictEqual(h.bridge.calls.length, 0, 'pending debounce must be cancelled by dispose');
  mock.didOpenTextDocumentEmitter.fire(doc);
  assert.strictEqual(h.bridge.calls.length, 0, 'listeners must be disposed');
  h.dm.dispose();
});

// =============================================================================
// WasmBridge (real class) — lifecycle, error wrapping (3.57 / G4 — replaces the
// deleted inline-copy wasmBridge.test.ts) and wasm-boundary result-shape
// validation (3.56 / G3)
// =============================================================================
// The wasm module is an out-of-process artifact from the TS bundle's point of
// view: a malformed `wasm_analyze` result must be rejected AT the bridge
// boundary (the callers' established `WASM analysis failed` exception path),
// not escape as a distant TypeError when the result is spread/iterated later.

/** Real WasmBridge with an injected fake wasm module (bypasses initialize()). */
function bridgeWithWasmModule(wasmModule: { wasm_analyze(text: string): unknown }): WasmBridge {
  const bridge = new WasmBridge({ appendLine: () => {} } as never);
  const internals = bridge as unknown as { wasmModule: unknown; ready: boolean };
  internals.wasmModule = wasmModule;
  internals.ready = true;
  return bridge;
}

function bridgeWithWasmResult(result: unknown): WasmBridge {
  return bridgeWithWasmModule({ wasm_analyze: () => result });
}

test('WasmBridge is not ready before initialize and analyzeDocument throws', () => {
  const bridge = new WasmBridge({ appendLine: () => {} } as never);
  assert.strictEqual(bridge.isReady(), false);
  assert.throws(() => bridge.analyzeDocument('x'), /not initialized/);
});

test('WasmBridge.dispose makes the bridge not ready and analyzeDocument throw', () => {
  const bridge = bridgeWithWasmResult({ tokens: [], diagnostics: [] });
  assert.strictEqual(bridge.isReady(), true);
  bridge.dispose();
  assert.strictEqual(bridge.isReady(), false);
  assert.throws(() => bridge.analyzeDocument('x'), /not initialized/);
});

test('analyzeDocument wraps a throwing wasm_analyze as "WASM analysis failed"', () => {
  const bridge = bridgeWithWasmModule({
    wasm_analyze: () => {
      throw new Error('simulated parser panic');
    },
  });
  assert.throws(() => bridge.analyzeDocument('x'), /WASM analysis failed.*simulated parser panic/);
});

test('analyzeDocument passes an empty (but well-formed) wasm result through', () => {
  const result = bridgeWithWasmResult({ tokens: [], diagnostics: [] }).analyzeDocument('');
  assert.deepStrictEqual(result, { tokens: [], diagnostics: [] });
});

test('analyzeDocument rejects a null/non-object wasm result at the boundary', () => {
  assert.throws(() => bridgeWithWasmResult(null).analyzeDocument('x'), /WASM analysis failed/);
  assert.throws(() => bridgeWithWasmResult(undefined).analyzeDocument('x'), /WASM analysis failed/);
  assert.throws(() => bridgeWithWasmResult(42).analyzeDocument('x'), /WASM analysis failed/);
});

test('analyzeDocument rejects a wasm result with non-array tokens/diagnostics', () => {
  assert.throws(
    () => bridgeWithWasmResult({ tokens: 'oops', diagnostics: [] }).analyzeDocument('x'),
    /WASM analysis failed/
  );
  assert.throws(
    () => bridgeWithWasmResult({ tokens: [], diagnostics: null }).analyzeDocument('x'),
    /WASM analysis failed/
  );
  assert.throws(
    () => bridgeWithWasmResult({ tokens: [] }).analyzeDocument('x'),
    /WASM analysis failed/
  );
});

test('analyzeDocument passes a well-formed wasm result through unchanged', () => {
  const wellFormed = {
    tokens: [{ deltaLine: 0, deltaStartCharacter: 1, length: 2, tokenType: 3, tokenModifiers: 0 }],
    diagnostics: [{
      range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } },
      message: 'syntax error',
      severity: 1,
    }],
  };
  const result = bridgeWithWasmResult(wellFormed).analyzeDocument('x');
  assert.deepStrictEqual(result, wellFormed);
});

// =============================================================================
// Debug adapter factory + configuration provider (real classes)
// =============================================================================

test('PastaDebugAdapterFactory resolves the session config into a DebugAdapterServer', () => {
  const factory = new PastaDebugAdapterFactory();
  const session = { configuration: { type: 'pasta', request: 'attach', host: '10.0.0.5', port: '4321' } };
  const desc = factory.createDebugAdapterDescriptor(session as never, undefined) as mock.DebugAdapterServer;
  assert.ok(desc instanceof mock.DebugAdapterServer);
  assert.strictEqual(desc.host, '10.0.0.5');
  assert.strictEqual(desc.port, 4321);
});

test('PastaDebugAdapterFactory falls back to 127.0.0.1:9276 for an empty config', () => {
  const factory = new PastaDebugAdapterFactory();
  const desc = factory.createDebugAdapterDescriptor(
    { configuration: {} } as never, undefined
  ) as mock.DebugAdapterServer;
  assert.strictEqual(desc.host, '127.0.0.1');
  assert.strictEqual(desc.port, 9276);
});

test('config provider lower-cases and forwards an explicit sourcePresentation', () => {
  const provider = new PastaDebugConfigurationProvider();
  const config = { type: 'pasta', request: 'attach', name: 'x', sourcePresentation: 'LUA' };
  const resolved = provider.resolveDebugConfiguration(undefined, config as never) as Record<string, unknown>;
  assert.strictEqual(resolved, config, 'must return the same config object');
  assert.strictEqual(resolved.sourcePresentation, 'lua');
});

test('config provider removes an absent/invalid sourcePresentation (no client default)', () => {
  const provider = new PastaDebugConfigurationProvider();
  const absent = { type: 'pasta', request: 'attach', name: 'x' };
  const resolvedAbsent = provider.resolveDebugConfiguration(undefined, absent as never) as Record<string, unknown>;
  assert.ok(!('sourcePresentation' in resolvedAbsent), 'absent key must stay absent');
  const invalid = { type: 'pasta', request: 'attach', name: 'x', sourcePresentation: 'banana' };
  const resolvedInvalid = provider.resolveDebugConfiguration(undefined, invalid as never) as Record<string, unknown>;
  assert.ok(!('sourcePresentation' in resolvedInvalid), 'invalid value must be removed so the server decides');
});

// =============================================================================
// wordRefDecorator (real) — regex/decoration behaviour
// =============================================================================

function fakeEditor(languageId: string, text: string) {
  const captured: { ranges: mock.Range[] | undefined; setCalls: number } = { ranges: undefined, setCalls: 0 };
  const editor = {
    document: fakeDocument(`file:///deco.${languageId}`, languageId, text),
    setDecorations: (_type: unknown, ranges: mock.Range[]) => {
      captured.ranges = ranges;
      captured.setCalls++;
    },
  };
  return { editor, captured };
}

// activateWordRefDecorator is called exactly ONCE for this whole section (each
// call would register an extra listener on the shared mock emitters and make
// the setDecorations call counts non-deterministic).

test('word-ref decorator boxes ＠/@ references and stops at delimiters', () => {
  const text = '＠笑顔　＠挨拶！ @hello world';
  const { editor, captured } = fakeEditor('pasta', text);
  mock.window.activeTextEditor = editor;
  activateWordRefDecorator(dummyContext() as never);
  mock.window.activeTextEditor = undefined;
  assert.ok(captured.ranges, 'decorations must be applied on activation');
  const spans = captured.ranges!.map((r) => text.substring(r.start.character, r.end.character));
  // ！/! are delimiters, whitespace ends a reference
  assert.deepStrictEqual(spans, ['＠笑顔', '＠挨拶', '@hello']);
});

test('word-ref decorator ignores non-pasta documents', () => {
  const { editor, captured } = fakeEditor('lua', '@word should not be decorated');
  mock.didChangeActiveTextEditorEmitter.fire(editor);
  assert.strictEqual(captured.setCalls, 0, 'setDecorations must not be called for non-pasta docs');
});

test('word-ref decorator re-applies on active-editor change and document edits', () => {
  const text = 'さくら：＠通常　やふぅ。';
  const { editor, captured } = fakeEditor('pasta', text);
  mock.didChangeActiveTextEditorEmitter.fire(editor);
  assert.strictEqual(captured.setCalls, 1, 'editor switch must decorate');
  const spans1 = captured.ranges!.map((r) => text.substring(r.start.character, r.end.character));
  assert.deepStrictEqual(spans1, ['＠通常']);
  // Document change event for the active editor re-decorates
  mock.window.activeTextEditor = editor;
  mock.didChangeTextDocumentEmitter.fire({ document: editor.document });
  assert.strictEqual(captured.setCalls, 2, 'document edit must re-decorate');
  mock.window.activeTextEditor = undefined;
});

// =============================================================================
// extension.ts (real activate) — WASM init + source-presentation toggle wiring
// =============================================================================
// Run LAST: activate registers persistent listeners on the shared mock emitters.

const TOGGLE_COMMAND = 'pasta.debug.toggleSourcePresentation';
const TOGGLE_EVENT = 'pasta/sourcePresentation';

test('activate loads the real WASM bridge and registers the debug + toggle wiring', async () => {
  mock.workspace.textDocuments = [];
  const context = { subscriptions: [] as Array<{ dispose(): void }>, extensionUri: mock.Uri.file(ROOT) };
  await activate(context as never);
  const state = getActivationState();
  assert.strictEqual(state.wasmReady, true, 'real wasm/pasta_lsp_wasm_bg.wasm must load');
  assert.strictEqual(state.fallbackMode, false);
  assert.ok(mock.registeredCommands.has(TOGGLE_COMMAND), 'toggle command must be registered');
  assert.strictEqual(mock.registeredDebugFactories.length, 1, 'debug adapter factory registered');
  assert.strictEqual(mock.registeredDebugConfigProviders.length, 1, 'debug config provider registered');
  assert.strictEqual(mock.createdStatusBarItems.length, 1, 'one status bar item created');
  assert.strictEqual(mock.createdStatusBarItems[0].command, TOGGLE_COMMAND, 'status bar click = 3rd entry point');
  assert.strictEqual(mock.createdStatusBarItems[0].visible, false, 'hidden until a pasta session pushes a mode');
});

test('toggle without an active pasta session warns and sends nothing', async () => {
  mock.debug.activeDebugSession = undefined;
  mock.shownMessages.length = 0;
  await mock.commands.executeCommand(TOGGLE_COMMAND);
  assert.strictEqual(mock.shownMessages.length, 1);
  assert.strictEqual(mock.shownMessages[0].kind, 'warning');
  assert.ok(mock.shownMessages[0].message.includes('Pasta デバッグセッション'));
});

function makePastaSession() {
  const session = {
    type: 'pasta',
    requests: [] as Array<{ command: string; args: unknown }>,
    failNext: false,
    customRequest: async (command: string, args: unknown) => {
      if (session.failNext) {
        throw new Error('backend unavailable');
      }
      session.requests.push({ command, args });
      return {};
    },
  };
  return session;
}

test('backend push event drives the status bar (single source of truth)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  const statusBar = mock.createdStatusBarItems[0];

  // Attach-time initial push
  mock.debugCustomEventEmitter.fire({ event: TOGGLE_EVENT, session, body: { mode: 'pasta' } });
  assert.strictEqual(statusBar.visible, true, 'status bar shows for the active pasta session');
  assert.strictEqual(statusBar.text, '$(eye) 提示: .pasta');

  // Toggle command flips the tracked mode and sends the set request,
  // WITHOUT optimistically updating the display.
  await mock.commands.executeCommand(TOGGLE_COMMAND);
  assert.deepStrictEqual(session.requests, [{ command: TOGGLE_EVENT, args: { mode: 'lua' } }]);
  assert.strictEqual(statusBar.text, '$(eye) 提示: .pasta', 'display updates only via push events');

  // Post-toggle push updates the display
  mock.debugCustomEventEmitter.fire({ event: TOGGLE_EVENT, session, body: { mode: 'lua' } });
  assert.strictEqual(statusBar.text, '$(eye) 提示: .lua');

  // Malformed push bodies are ignored (requirement 1.4 client-side)
  mock.debugCustomEventEmitter.fire({ event: TOGGLE_EVENT, session, body: { mode: 'xml' } });
  mock.debugCustomEventEmitter.fire({ event: TOGGLE_EVENT, session, body: null });
  assert.strictEqual(statusBar.text, '$(eye) 提示: .lua', 'invalid bodies must not change the display');

  // Unrelated events are ignored
  mock.debugCustomEventEmitter.fire({ event: 'other/event', session, body: { mode: 'pasta' } });
  assert.strictEqual(statusBar.text, '$(eye) 提示: .lua');
});

test('failed customRequest surfaces an error message and keeps the display', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.debugCustomEventEmitter.fire({ event: TOGGLE_EVENT, session, body: { mode: 'lua' } });
  const statusBar = mock.createdStatusBarItems[0];
  session.failNext = true;
  mock.shownMessages.length = 0;
  await mock.commands.executeCommand(TOGGLE_COMMAND);
  assert.strictEqual(mock.shownMessages.length, 1);
  assert.strictEqual(mock.shownMessages[0].kind, 'error');
  assert.ok(mock.shownMessages[0].message.includes('backend unavailable'));
  assert.strictEqual(statusBar.text, '$(eye) 提示: .lua', 'no optimistic update on failure');
});

test('session termination resets the tracked mode and hides the status bar', () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.debugCustomEventEmitter.fire({ event: TOGGLE_EVENT, session, body: { mode: 'lua' } });
  const statusBar = mock.createdStatusBarItems[0];
  assert.strictEqual(statusBar.visible, true);
  mock.debug.activeDebugSession = undefined;
  mock.didTerminateDebugSessionEmitter.fire(session);
  assert.strictEqual(statusBar.visible, false, 'status bar hides when the session ends');
});

test('switching to a non-pasta session hides the status bar and resets the mode', () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.debugCustomEventEmitter.fire({ event: TOGGLE_EVENT, session, body: { mode: 'pasta' } });
  const statusBar = mock.createdStatusBarItems[0];
  assert.strictEqual(statusBar.visible, true);
  const nodeSession = { type: 'node' };
  mock.debug.activeDebugSession = nodeSession;
  mock.didChangeActiveDebugSessionEmitter.fire(nodeSession);
  assert.strictEqual(statusBar.visible, false);
  mock.debug.activeDebugSession = undefined;
});

test('deactivate disposes cleanly', () => {
  assert.doesNotThrow(() => deactivate());
});

// =============================================================================
// Runner
// =============================================================================

(async () => {
  console.log('\n[Real-module Unit Tests (vscode mock alias)]');
  for (const { name, fn } of queue) {
    try {
      await fn();
      console.log(`  ✓ ${name}`);
      passed++;
    } catch (e: unknown) {
      console.error(`  ✗ ${name}`);
      console.error(`    ${e instanceof Error ? e.stack ?? e.message : String(e)}`);
      failed++;
    }
  }
  console.log(`\nvscodeModules: ${passed} passed, ${failed} failed\n`);
  if (failed > 0) {
    process.exit(1);
  }
})();
