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
//   - diagnosticsManager.ts     : range conversion (always Error), clear
//   - documentSync.ts           : open/change/close lifecycle, 200ms debounce,
//     analysis-failure fallback, dispose
//   - wasmBridge.ts             : init/dispose lifecycle, error wrapping,
//     wasm-boundary result-shape validation
//   - wordRefDecorator.ts       : @word-ref decoration ranges (TextMate の文法の規則)
//   - extension.ts              : activate (real WASM load), ghost validator wiring

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
import { SceneOutlineProvider } from '../sceneOutline';
import { DiagnosticsManager } from '../diagnosticsManager';
import { WasmBridge } from '../wasmBridge';
import { DocumentSync } from '../documentSync';
import { GhostValidator } from '../ghostValidator';
import { activateWordRefDecorator, findWordRefRanges } from '../wordRefDecorator';
import { WORD_REF_CASES } from './fixtures/wordRefCases';
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
  scenes: unknown[];
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
      return opts.result ?? { tokens: [], diagnostics: [], scenes: [] };
    },
  };
  return bridge;
}

/** Minimal pasta TextDocument double for DocumentSync / providers. */
function fakeDocument(uriStr: string, languageId: string, text: string, version = 1) {
  return {
    uri: mock.Uri.parse(uriStr),
    languageId,
    version,
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
    const names = [...block![1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);
    assert.deepStrictEqual([...PASTA_TOKEN_TYPES], names,
      'TS PASTA_TOKEN_TYPES must stay index-aligned with Rust TOKEN_TYPES');
  });

  test('PASTA_TOKEN_MODIFIERS matches pasta_lsp token_types.rs TOKEN_MODIFIERS order', () => {
    const src = fs.readFileSync(RUST_TOKEN_TYPES_PATH, 'utf-8');
    const block = src.match(/pub const TOKEN_MODIFIERS[^=]*=\s*&\[([\s\S]*?)\];/);
    assert.ok(block, 'TOKEN_MODIFIERS const block should be found in token_types.rs');
    const names = [...block![1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);
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
    scenes: [],
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
  const bridge = fakeBridge({ result: { tokens: [], diagnostics: [], scenes: [] } });
  const provider = new SemanticTokensProvider(bridge as never);
  const doc = fakeDocument('file:///cache.pasta', 'pasta', 'text');
  provider.provideDocumentSemanticTokens(doc as never, dummyToken);
  provider.provideDocumentSemanticTokens(doc as never, dummyToken);
  assert.strictEqual(bridge.calls.length, 1, 'second call must hit the cache');
});

test('clearTokens drops the cache and forces re-analysis', () => {
  const bridge = fakeBridge({ result: { tokens: [], diagnostics: [], scenes: [] } });
  const provider = new SemanticTokensProvider(bridge as never);
  const doc = fakeDocument('file:///clear.pasta', 'pasta', 'text');
  provider.provideDocumentSemanticTokens(doc as never, dummyToken);
  provider.clearTokens(doc.uri.toString());
  provider.provideDocumentSemanticTokens(doc as never, dummyToken);
  assert.strictEqual(bridge.calls.length, 2);
});

test('updateTokens replaces the cache and fires onDidChangeSemanticTokens', () => {
  const bridge = fakeBridge({ result: { tokens: [], diagnostics: [], scenes: [] } });
  const provider = new SemanticTokensProvider(bridge as never);
  let fired = 0;
  provider.onDidChangeSemanticTokens(() => { fired++; });
  const doc = fakeDocument('file:///update.pasta', 'pasta', 'text');
  provider.updateTokens(doc.uri.toString(), doc.version, [
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

// 要件 8.1: 控えは版つき。違う版の控えは返さず、その場で解き直す
test('provider does not serve tokens cached for a different document version', () => {
  const fresh = { deltaLine: 0, deltaStartCharacter: 0, length: 3, tokenType: 1, tokenModifiers: 0 };
  const bridge = fakeBridge({ result: { tokens: [fresh], diagnostics: [], scenes: [] } });
  const provider = new SemanticTokensProvider(bridge as never);
  const uri = 'file:///ver.pasta';
  provider.updateTokens(mock.Uri.parse(uri).toString(), 1, [
    { deltaLine: 0, deltaStartCharacter: 5, length: 1, tokenType: 4, tokenModifiers: 0 },
  ]);
  const v2 = fakeDocument(uri, 'pasta', 'new text', 2);
  const tokens = provider.provideDocumentSemanticTokens(v2 as never, dummyToken) as mock.SemanticTokens;
  assert.deepStrictEqual(bridge.calls, ['new text'], 'a stale version must be re-analyzed');
  assert.deepStrictEqual(Array.from(tokens.data), [0, 0, 3, 1, 0]);
  // 解き直した結果は v2 として控えるので、もう一度求められても解かない
  provider.provideDocumentSemanticTokens(v2 as never, dummyToken);
  assert.strictEqual(bridge.calls.length, 1, 'the re-analyzed result must be cached for v2');
});

// 要件 8.2: 今の版の解析に失敗したら、前の版の色を返さず控えも消す
test('provider failure on a new version returns null and drops the previous version', () => {
  const opts = { result: { tokens: [], diagnostics: [], scenes: [] }, fail: false };
  const bridge = fakeBridge(opts);
  const provider = new SemanticTokensProvider(bridge as never);
  const uri = 'file:///ver-fail.pasta';
  provider.provideDocumentSemanticTokens(fakeDocument(uri, 'pasta', 'v1', 1) as never, dummyToken);
  opts.fail = true;
  assert.strictEqual(
    provider.provideDocumentSemanticTokens(fakeDocument(uri, 'pasta', 'v2', 2) as never, dummyToken),
    null,
    'the previous version must not be returned when the current one fails'
  );
  opts.fail = false;
  provider.provideDocumentSemanticTokens(fakeDocument(uri, 'pasta', 'v1', 1) as never, dummyToken);
  assert.deepStrictEqual(bridge.calls, ['v1', 'v2', 'v1'], 'the failed analysis must drop the cache');
});

// =============================================================================
// SceneOutlineProvider (real)
// =============================================================================

const span = (l0: number, c0: number, l1: number, c1: number) => ({
  start: { line: l0, character: c0 }, end: { line: l1, character: c1 },
});

/** ＊挨拶（ローカル ・朝）／＊挨拶（続き）／＊挨拶（同じ名前の別の続き） */
const SCENES = [
  {
    name: '挨拶', kind: 'global', heading: span(0, 0, 0, 3), range: span(0, 0, 4, 0),
    children: [{ name: '朝', kind: 'local', heading: span(2, 2, 2, 4), range: span(2, 0, 4, 0), children: [] }],
  },
  { name: '挨拶', kind: 'global', heading: span(5, 0, 5, 3), range: span(5, 0, 6, 0), children: [] },
  { name: '挨拶', kind: 'global', heading: span(7, 0, 7, 3), range: span(7, 0, 8, 5), children: [] },
];

function sceneBridge(opts: { fail?: boolean } = {}) {
  return fakeBridge({ result: { tokens: [], diagnostics: [], scenes: SCENES }, fail: opts.fail });
}

const plainRange = (r: mock.Range) => span(r.start.line, r.start.character, r.end.line, r.end.character);

test('SceneOutlineProvider: DocumentSymbol は名前・アイコン・範囲・選ぶ位置・子を写す（2.1・2.3・2.5）', () => {
  const provider = new SceneOutlineProvider(sceneBridge() as never);
  const symbols = provider.provideDocumentSymbols(fakeDocument('file:///s.pasta', 'pasta', 'x') as never, dummyToken) as unknown as mock.DocumentSymbol[];
  assert.strictEqual(symbols.length, 3);
  const [first] = symbols;
  assert.ok(first instanceof mock.DocumentSymbol);
  assert.strictEqual(first.name, '挨拶');
  assert.strictEqual(first.detail, '');
  assert.strictEqual(first.kind, mock.SymbolKind.Namespace);
  assert.deepStrictEqual(plainRange(first.range), SCENES[0].range);
  assert.deepStrictEqual(plainRange(first.selectionRange), SCENES[0].heading);
  assert.strictEqual(first.children.length, 1);
  const child = first.children[0];
  assert.strictEqual(child.name, '朝');
  assert.strictEqual(child.kind, mock.SymbolKind.Function);
  assert.deepStrictEqual(plainRange(child.range), SCENES[0].children[0].range);
  assert.deepStrictEqual(plainRange(child.selectionRange), SCENES[0].children[0].heading);
  assert.deepStrictEqual(child.children, []);
});

test('SceneOutlineProvider: 同じ名前の 2 つのシーンは別の項目（2.4）', () => {
  const provider = new SceneOutlineProvider(sceneBridge() as never);
  const symbols = provider.provideDocumentSymbols(fakeDocument('file:///same.pasta', 'pasta', 'x') as never, dummyToken) as unknown as mock.DocumentSymbol[];
  assert.deepStrictEqual(symbols.map((s) => [s.name, s.selectionRange.start.line]), [['挨拶', 0], ['挨拶', 5], ['挨拶', 7]]);
});

test('SceneOutlineProvider: すべての見出しに CodeLens が 1 つ・題は package.json と同じ・引数は [uri, 見出しの行]（3.1・4.5）', () => {
  const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, 'package.json'), 'utf-8'));
  const title = pkg.contributes.commands.find((c: { command: string }) => c.command === 'pasta.runSceneAtCursor').title;
  const provider = new SceneOutlineProvider(sceneBridge() as never);
  const doc = fakeDocument('file:///lens.pasta', 'pasta', 'x');
  const lenses = provider.provideCodeLenses(doc as never, dummyToken) as unknown as mock.CodeLens[];
  // グローバル・ローカル・続き・続きの順（文書の順）
  assert.deepStrictEqual(lenses.map((l) => plainRange(l.range)), [SCENES[0].heading, SCENES[0].children[0].heading, SCENES[1].heading, SCENES[2].heading]);
  for (const lens of lenses) {
    assert.ok(lens instanceof mock.CodeLens);
    assert.strictEqual(lens.command?.title, title);
    assert.strictEqual(lens.command?.command, 'pasta.runSceneAtCursor');
    assert.strictEqual(lens.command?.arguments?.[0], doc.uri);
    assert.strictEqual(lens.command?.arguments?.[1], lens.range.start.line);
    assert.strictEqual(lens.command?.arguments?.length, 2);
  }
  assert.strictEqual((provider as unknown as Record<string, unknown>).resolveCodeLens, undefined);
  assert.strictEqual((provider as unknown as Record<string, unknown>).onDidChangeCodeLenses, undefined);
});

test('SceneOutlineProvider: 解析の例外なら両方とも空で、通知を出さない（5.3）', () => {
  const provider = new SceneOutlineProvider(sceneBridge({ fail: true }) as never);
  const doc = fakeDocument('file:///boom.pasta', 'pasta', 'x');
  const before = mock.shownMessages.length;
  const outputBefore = mock.outputLines.length;
  assert.deepStrictEqual(provider.provideDocumentSymbols(doc as never, dummyToken), []);
  assert.deepStrictEqual(provider.provideCodeLenses(doc as never, dummyToken), []);
  assert.strictEqual(mock.shownMessages.length, before);
  assert.strictEqual(mock.outputLines.length, outputBefore);
});

test('SceneOutlineProvider: 同じ版では解析は 1 回・版が変わると解き直す（2.7・3.6）', () => {
  const bridge = sceneBridge();
  const provider = new SceneOutlineProvider(bridge as never);
  const uri = 'file:///cache-scenes.pasta';
  const v1 = fakeDocument(uri, 'pasta', 'v1', 1);
  provider.provideDocumentSymbols(v1 as never, dummyToken);
  provider.provideCodeLenses(v1 as never, dummyToken);
  assert.deepStrictEqual(bridge.calls, ['v1']);
  provider.provideCodeLenses(fakeDocument(uri, 'pasta', 'v2', 2) as never, dummyToken);
  assert.deepStrictEqual(bridge.calls, ['v1', 'v2']);
  // 別の文書は同じ版の数でも解き直す
  provider.provideDocumentSymbols(fakeDocument('file:///other.pasta', 'pasta', 'o', 2) as never, dummyToken);
  assert.deepStrictEqual(bridge.calls, ['v1', 'v2', 'o']);
});

// =============================================================================
// DiagnosticsManager (real)
// =============================================================================

function collectionOf(dm: DiagnosticsManager): mock.MockDiagnosticCollection {
  return (dm as unknown as { collection: mock.MockDiagnosticCollection }).collection;
}

// 入力のたびの診断は構文の誤りだけなので、重さは常に Error（N9）
test('setDiagnostics always uses Error severity', () => {
  const dm = new DiagnosticsManager();
  const uri = mock.Uri.parse('file:///sev.pasta');
  const mk = (severity: number) => ({
    range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
    message: `sev ${severity}`,
    severity,
  });
  dm.setDiagnostics(uri as never, [mk(1), mk(2), mk(3), mk(4), mk(99)]);
  const got = collectionOf(dm).get(uri)!;
  assert.deepStrictEqual(got.map((d) => d.severity), Array(5).fill(mock.DiagnosticSeverity.Error));
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

test('setDiagnostics replaces, clear deletes', () => {
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
      scenes: [],
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

// 要件 8.2: 同期の解析に失敗したら、診断と一緒に色の控えも消す
test('analysis failure in DocumentSync also drops the cached tokens', () => {
  const opts = {
    result: { tokens: [{ deltaLine: 0, deltaStartCharacter: 0, length: 1, tokenType: 1, tokenModifiers: 0 }], diagnostics: [], scenes: [] },
    fail: false,
  };
  const h = makeSyncHarness(opts);
  h.sync.activate(dummyContext() as never);
  const v1 = fakeDocument('file:///sync-fail.pasta', 'pasta', 'v1', 1);
  mock.didOpenTextDocumentEmitter.fire(v1);
  opts.fail = true;
  mock.didOpenTextDocumentEmitter.fire(fakeDocument('file:///sync-fail.pasta', 'pasta', 'v2', 2));
  opts.fail = false;
  // 控えが残っていれば v1 の色を解かずに返してしまう
  h.provider.provideDocumentSemanticTokens(v1 as never, dummyToken);
  assert.deepStrictEqual(h.bridge.calls, ['v1', 'v2', 'v1'], 'the failed sync must clear the token cache');
  h.sync.dispose();
  h.dm.dispose();
});

// 要件 9.5: wasm32 の panic は trap として JS の例外になる。拡張は落ちず、理由を書き、診断と控えを消す
test('a wasm trap during sync is caught, logged, and clears diagnostics and tokens', () => {
  let trap = false;
  const bridge = bridgeWithWasmModule({
    wasm_analyze: () => {
      if (trap) {
        // 本物は WebAssembly.RuntimeError: unreachable（tsconfig の lib に型が無いので名前だけ写す）
        throw Object.assign(new Error('unreachable'), { name: 'RuntimeError' });
      }
      return {
        tokens: [{ deltaLine: 0, deltaStartCharacter: 0, length: 1, tokenType: 1, tokenModifiers: 0 }],
        diagnostics: [{
          range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
          message: 'diag',
          severity: 1,
        }],
        scenes: [],
      };
    },
  });
  const provider = new SemanticTokensProvider(bridge);
  const dm = new DiagnosticsManager();
  const logged: string[] = [];
  const channel = { name: 'test', appendLine: (msg: string) => { logged.push(msg); }, dispose: () => {} };
  const sync = new DocumentSync(bridge, provider, dm, channel as never);
  sync.activate(dummyContext() as never);
  const uri = 'file:///trap.pasta';
  mock.didOpenTextDocumentEmitter.fire(fakeDocument(uri, 'pasta', 'ok', 1));
  assert.ok(collectionOf(dm).get(mock.Uri.parse(uri)), 'diagnostics seeded');
  trap = true;
  assert.doesNotThrow(() => mock.didOpenTextDocumentEmitter.fire(fakeDocument(uri, 'pasta', 'boom', 1)));
  assert.ok(
    logged.some((l) => l.includes('Document analysis failed') && l.includes('unreachable')),
    'the trap reason must reach the output channel'
  );
  assert.strictEqual(collectionOf(dm).get(mock.Uri.parse(uri)), undefined, 'diagnostics must be cleared');
  assert.strictEqual(
    (provider as unknown as { cachedTokens: Map<string, unknown> }).cachedTokens.has(mock.Uri.parse(uri).toString()),
    false,
    'token cache must be cleared'
  );
  sync.dispose();
  dm.dispose();
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
function bridgeWithWasmModule(wasmModule: object): WasmBridge {
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
  const bridge = bridgeWithWasmResult({ tokens: [], diagnostics: [], scenes: [] });
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
  const result = bridgeWithWasmResult({ tokens: [], diagnostics: [], scenes: [] }).analyzeDocument('');
  assert.deepStrictEqual(result, { tokens: [], diagnostics: [], scenes: [] });
});

test('analyzeDocument rejects a null/non-object wasm result at the boundary', () => {
  assert.throws(() => bridgeWithWasmResult(null).analyzeDocument('x'), /WASM analysis failed/);
  assert.throws(() => bridgeWithWasmResult(undefined).analyzeDocument('x'), /WASM analysis failed/);
  assert.throws(() => bridgeWithWasmResult(42).analyzeDocument('x'), /WASM analysis failed/);
});

test('analyzeDocument rejects a wasm result with non-array tokens/diagnostics', () => {
  assert.throws(
    () => bridgeWithWasmResult({ tokens: 'oops', diagnostics: [], scenes: [] }).analyzeDocument('x'),
    /WASM analysis failed/
  );
  assert.throws(
    () => bridgeWithWasmResult({ tokens: [], diagnostics: null, scenes: [] }).analyzeDocument('x'),
    /WASM analysis failed/
  );
  assert.throws(
    () => bridgeWithWasmResult({ tokens: [], scenes: [] }).analyzeDocument('x'),
    /WASM analysis failed/
  );
});

test('analyzeDocument rejects a wasm result without a scenes array', () => {
  assert.throws(
    () => bridgeWithWasmResult({ tokens: [], diagnostics: [] }).analyzeDocument('x'),
    /WASM analysis failed/
  );
  assert.throws(
    () => bridgeWithWasmResult({ tokens: [], diagnostics: [], scenes: null }).analyzeDocument('x'),
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
    scenes: [{
      name: 'メイン',
      kind: 'global',
      heading: { start: { line: 0, character: 0 }, end: { line: 0, character: 4 } },
      range: { start: { line: 0, character: 0 }, end: { line: 2, character: 3 } },
      children: [{
        name: '続き',
        kind: 'local',
        heading: { start: { line: 1, character: 0 }, end: { line: 1, character: 3 } },
        range: { start: { line: 1, character: 0 }, end: { line: 2, character: 3 } },
        children: [],
      }],
    }],
  };
  const result = bridgeWithWasmResult(wellFormed).analyzeDocument('x');
  assert.deepStrictEqual(result, wellFormed);
});

// --- ゴースト全体の検査（pasta-check-dic-validate 7.1・要件 11.2 / 11.8） ---
// wasm_select_ghost_files / wasm_validate_ghost は失敗で null を返す。結果の形を
// 境界で確かめ、違えば throw する（核の結果はそのまま通す）。

const ghostInput = { pastaToml: { kind: 'missing' } as const, entries: ['dic/a.pasta'], dirs: ['dic'] };

const wellFormedReport = {
  findings: [
    { code: 'syntax', severity: 'error', path: 'dic/a.pasta', line: 3, column: 1, message: '構文の誤り' },
    { code: 'config-scene-alias', severity: 'warning', path: 'pasta.toml', line: null, column: null, message: '注意' },
  ],
  stats: { dictionaryFiles: 1, errors: 1, warnings: 1, uncheckedDynamicCalls: 0, skipReasons: ['syntax'] },
};

test('selectGhostFiles passes the wasm input through and returns the string list', () => {
  let seen: unknown;
  const bridge = bridgeWithWasmModule({
    wasm_select_ghost_files: (input: unknown) => {
      seen = input;
      return ['dic/a.pasta'];
    },
  });
  assert.deepStrictEqual(bridge.selectGhostFiles(ghostInput), ['dic/a.pasta']);
  assert.deepStrictEqual(seen, ghostInput);
});

test('selectGhostFiles throws on null / non-string-array wasm results', () => {
  for (const bad of [null, undefined, 'dic/a.pasta', [1], ['ok', null]]) {
    const bridge = bridgeWithWasmModule({ wasm_select_ghost_files: () => bad });
    assert.throws(() => bridge.selectGhostFiles(ghostInput), /WASM/, String(bad));
  }
});

test('selectGhostFiles throws when the bridge is not ready', () => {
  const bridge = new WasmBridge({ appendLine: () => {} } as never);
  assert.throws(() => bridge.selectGhostFiles(ghostInput), /not initialized/);
});

test('validateGhost passes input/files through and returns a well-formed report unchanged', () => {
  const files = { 'dic/a.pasta': { kind: 'text', text: '＊会話\n' } as const };
  let seen: unknown[] = [];
  const bridge = bridgeWithWasmModule({
    wasm_validate_ghost: (input: unknown, f: unknown) => {
      seen = [input, f];
      return wellFormedReport;
    },
  });
  assert.deepStrictEqual(bridge.validateGhost(ghostInput, files), wellFormedReport);
  assert.deepStrictEqual(seen, [ghostInput, files]);
});

test('validateGhost throws on null and on every malformed report shape', () => {
  const finding = wellFormedReport.findings[0];
  const stats = wellFormedReport.stats;
  const bads: unknown[] = [
    null,
    undefined,
    42,
    { findings: [] },
    { stats },
    { findings: 'x', stats },
    { findings: [], stats: null },
    { findings: [{ ...finding, severity: 'info' }], stats },
    { findings: [{ ...finding, code: 7 }], stats },
    { findings: [{ ...finding, path: undefined }], stats },
    { findings: [{ ...finding, line: undefined }], stats },
    { findings: [{ ...finding, column: '1' }], stats },
    { findings: [{ ...finding, message: null }], stats },
    { findings: [null], stats },
    { findings: [], stats: { ...stats, dictionaryFiles: '1' } },
    { findings: [], stats: { ...stats, errors: undefined } },
    { findings: [], stats: { ...stats, warnings: null } },
    { findings: [], stats: { ...stats, uncheckedDynamicCalls: true } },
    { findings: [], stats: { ...stats, skipReasons: undefined } },
    { findings: [], stats: { ...stats, skipReasons: 'syntax' } },
    { findings: [], stats: { ...stats, skipReasons: ['syntax', 'parse'] } },
    { findings: [], stats: { ...stats, skipReasons: [null] } },
    // 古い形（真偽値だけ）は受け付けない
    {
      findings: [],
      stats: { dictionaryFiles: 1, errors: 1, warnings: 1, uncheckedDynamicCalls: 0, referencesSkipped: true },
    },
  ];
  for (const bad of bads) {
    const bridge = bridgeWithWasmModule({ wasm_validate_ghost: () => bad });
    assert.throws(() => bridge.validateGhost(ghostInput, {}), /WASM/, JSON.stringify(bad));
  }
});

test('real WASM: selectGhostFiles / validateGhost results pass the shape checks', async () => {
  const bridge = new WasmBridge({ appendLine: () => {} } as never);
  await bridge.initialize(mock.Uri.joinPath(mock.Uri.file(ROOT), 'wasm', 'pasta_lsp_wasm_bg.wasm') as never);
  const input = { pastaToml: { kind: 'missing' } as const, entries: ['dic/a.pasta', 'readme.txt'], dirs: ['dic'] };
  assert.deepStrictEqual(bridge.selectGhostFiles(input), ['dic/a.pasta']);
  const report = bridge.validateGhost(input, { 'dic/a.pasta': { kind: 'notUtf8' } });
  // 読めないファイルの誤りは位置を持たない（None → null で届く）
  assert.ok(
    report.findings.some((f) => f.severity === 'error' && f.path === 'dic/a.pasta' && f.line === null && f.column === null),
    JSON.stringify(report)
  );
  assert.ok(report.stats.errors >= 1);
  assert.deepStrictEqual(report.stats.skipReasons, ['unreadable-file']);
  bridge.dispose();
});

// 拡張と本物の核をつなぐ（pasta-check-validate-edges 3.4・要件 4.1 / 4.2）。
// 大文字・小文字の混じった Ghost/Master の配下の中身の無い dic/x.pasta/ を、
// 本物の WASM がフォルダの誤りとして返し、フォルダの URI に Error で出る。
test('real WASM: Ghost/Master の配下の空の dic/x.pasta/ は folder-matches-pattern の Error になる', async () => {
  const bridge = new WasmBridge({ appendLine: () => {} } as never);
  await bridge.initialize(mock.Uri.joinPath(mock.Uri.file(ROOT), 'wasm', 'pasta_lsp_wasm_bg.wasm') as never);
  mock.resetFakeFs();
  mock.workspace.textDocuments.length = 0;
  mock.setFakeFile('/ws/Ghost/Master/dic/a.pasta', '＊会話\n');
  mock.setFakeDir('/ws/Ghost/Master/dic/x.pasta');
  const gv = new GhostValidator(bridge, mock.window.createOutputChannel('Pasta Language') as never);
  const col = mock.createdDiagnosticCollections[mock.createdDiagnosticCollections.length - 1];
  const master = mock.Uri.file('/ws/Ghost/Master');

  await gv.validate(master as never);

  const d = col.get(mock.Uri.joinPath(master, 'dic', 'x.pasta'));
  assert.ok(d, `フォルダの URI に診断が無い（診断のある URI は ${col.size} 個）`);
  assert.deepStrictEqual(
    d.map((x) => [x.code, x.severity]),
    [['folder-matches-pattern', mock.DiagnosticSeverity.Error]]
  );
  gv.dispose();
  bridge.dispose();
  mock.resetFakeFs();
});

test('validateGhost throws when the bridge is not ready', () => {
  const bridge = new WasmBridge({ appendLine: () => {} } as never);
  assert.throws(() => bridge.validateGhost(ghostInput, {}), /not initialized/);
});

// =============================================================================
// wordRefDecorator (real) — 箱の範囲は TextMate の文法の規則から（要件 5.1〜5.3）
// =============================================================================

for (const c of WORD_REF_CASES) {
  test(`word-ref box: ${c.name}`, () => {
    const spans = findWordRefRanges(c.text).map(([s, e]) => c.text.slice(s, e));
    assert.deepStrictEqual(spans, c.boxes);
  });
}

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

test('word-ref decorator boxes ＠/@ references in talk and stops at delimiters', () => {
  const text = '　さくら：＠笑顔　＠挨拶！ @hello world';
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
  const text = '　さくら：＠通常　やふぅ。';
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
// mock/vscode — 3 つのコマンドの流れで使う API が記録を残す（タスク 2.2・要件 8.1）
// =============================================================================

test('mock: showQuickPick は決めた選択を返し、次は閉じた（undefined）になる', async () => {
  const before = mock.quickPickCalls.length;
  const items = [{ label: 'SSP（127.0.0.1:9801）' }, { label: 'areka（127.0.0.1:9821）' }];
  mock.setNextQuickPickSelection('areka（127.0.0.1:9821）');
  assert.strictEqual(await mock.window.showQuickPick(items, { placeHolder: 'p' }), items[1]);
  assert.strictEqual(await mock.window.showQuickPick(items), undefined, 'selection is consumed once');
  assert.strictEqual(mock.quickPickCalls.length, before + 2);
  assert.deepStrictEqual(mock.quickPickCalls[before], { items: items.map((i) => i.label), options: { placeHolder: 'p' } });
});

test('mock: withProgress は記録して task の値を返す', async () => {
  const before = mock.progressCalls.length;
  const opts = { location: mock.ProgressLocation.Window, title: 'Pasta: x' };
  const value = await mock.window.withProgress(opts, async (progress) => { progress.report({ message: 'm' }); return 42; });
  assert.strictEqual(value, 42);
  assert.deepStrictEqual(mock.progressCalls.slice(before), [opts]);
  assert.strictEqual(mock.ProgressLocation.Window, 10);
});

test('mock: 出力のパネルの show と書いた行が残る', () => {
  const ch = mock.window.createOutputChannel('Pasta-2.2');
  const before = mock.shownOutputChannels.length;
  ch.appendLine('line');
  ch.show(true);
  assert.deepStrictEqual(mock.shownOutputChannels.slice(before), ['Pasta-2.2']);
  assert.deepStrictEqual(mock.outputLines[mock.outputLines.length - 1], { channel: 'Pasta-2.2', message: 'line' });
});

test('mock: 文書の save() は記録され、成否を試験が決められる。textDocuments は差し替えられる', async () => {
  const ok = mock.createTextDocument('/g/ghost/master/dic/a.pasta');
  const ng = mock.createTextDocument('/g/ghost/master/pasta.toml', { saveResult: false });
  mock.workspace.textDocuments = [ok, ng];
  assert.deepStrictEqual(mock.workspace.textDocuments.map((d) => d.fileName), [ok.fileName, ng.fileName]);
  const before = mock.savedDocuments.length;
  assert.strictEqual(ok.isDirty, true);
  assert.strictEqual(await ok.save(), true);
  assert.strictEqual(ok.isDirty, false);
  assert.strictEqual(await ng.save(), false);
  assert.strictEqual(ng.isDirty, true);
  assert.deepStrictEqual(mock.savedDocuments.slice(before), [
    { path: ok.uri.path, ok: true },
    { path: ng.uri.path, ok: false },
  ]);
  mock.workspace.textDocuments = [];
});

// =============================================================================
// extension.ts (real activate) — WASM init + ghost validator wiring
// =============================================================================
// Run LAST: activate registers persistent listeners on the shared mock emitters.

const GHOST_WATCH_GLOB = '**/*';

function ghostValidatorCreated(fromCollection: number, fromWatcher: number): { collections: number; watchers: number } {
  return {
    collections: mock.createdDiagnosticCollections.slice(fromCollection).filter((c) => c.name === 'pasta-ghost').length,
    watchers: mock.createdFileSystemWatchers.slice(fromWatcher).filter((w) => w.globPattern === GHOST_WATCH_GLOB).length,
  };
}

/** 要件（requirements.md の Introduction）の例の文書 */
const REQUIREMENT_EXAMPLE = '＊挨拶\n  さくら：やあ\n  ・朝\n    さくら：おはよう\n＊\n  さくら：またね\n';

// 3 つのコマンドは activate が 1 回で登録する（4.1・5.1・6.1）
const DEV_COMMANDS = ['pasta.runSceneAtCursor', 'pasta.reloadShiori', 'pasta.showSaveVariables'];
function assertDevCommandsRegistered(): void {
  for (const id of DEV_COMMANDS) {
    assert.ok(mock.registeredCommands.has(id), `${id} must be registered by activate`);
  }
}

test('activate loads the real WASM bridge and registers the ghost validator', async () => {
  mock.workspace.textDocuments = [];
  const context = { subscriptions: [] as Array<{ dispose(): void }>, extensionUri: mock.Uri.file(ROOT) };
  const collectionsBefore = mock.createdDiagnosticCollections.length;
  const watchersBefore = mock.createdFileSystemWatchers.length;
  const symbolsBefore = mock.registeredDocumentSymbolProviders.length;
  const lensesBefore = mock.registeredCodeLensProviders.length;
  mock.registeredCommands.clear();
  await activate(context as never);
  // WASM が読めたら GhostValidator を作って登録する（11.12）
  assert.deepStrictEqual(ghostValidatorCreated(collectionsBefore, watchersBefore), { collections: 1, watchers: 1 });
  const state = getActivationState();
  assert.strictEqual(state.wasmReady, true, 'real wasm/pasta_lsp_wasm_bg.wasm must load');
  // DAP の提示モードの切り替えはもう登録しない（要件 1.3）
  assert.ok(!mock.registeredCommands.has('pasta.debug.toggleSourcePresentation'), 'DAP toggle command must not be registered');
  assertDevCommandsRegistered();
  // 5.1・1.13: 2 つのプロバイダが pasta に登録され、本物の WASM で要件の例から 2 つのグローバルと 1 つのローカルが出る
  const symbols = mock.registeredDocumentSymbolProviders.slice(symbolsBefore);
  const lenses = mock.registeredCodeLensProviders.slice(lensesBefore);
  assert.deepStrictEqual(symbols.map((r) => r.selector), [{ language: 'pasta' }]);
  assert.deepStrictEqual(lenses.map((r) => r.selector), [{ language: 'pasta' }]);
  const doc = fakeDocument('file:///ghost/master/dic/a.pasta', 'pasta', REQUIREMENT_EXAMPLE);
  const tree = (symbols[0].provider.provideDocumentSymbols(doc, dummyToken) as mock.DocumentSymbol[])
    .map((s) => [s.name, s.children.map((c) => c.name)]);
  assert.deepStrictEqual(tree, [['挨拶', ['朝']], ['挨拶', []]]);
  const lensLines = (lenses[0].provider.provideCodeLenses(doc, dummyToken) as mock.CodeLens[])
    .map((l) => l.command!.arguments![1]);
  assert.deepStrictEqual(lensLines, [0, 2, 4]);
});

test('deactivate disposes cleanly', () => {
  assert.doesNotThrow(() => deactivate());
});

// 最後に走らせる: 失敗した activate はコマンド等を登録し直す
test('WASM が読めないときは GhostValidator を作らず、activate は失敗しない（11.12）', async () => {
  const context = { subscriptions: [] as Array<{ dispose(): void }>, extensionUri: mock.Uri.file(path.join(ROOT, 'no-such-dir')) };
  const collectionsBefore = mock.createdDiagnosticCollections.length;
  const watchersBefore = mock.createdFileSystemWatchers.length;
  const symbolsBefore = mock.registeredDocumentSymbolProviders.length;
  const lensesBefore = mock.registeredCodeLensProviders.length;
  mock.registeredCommands.clear();
  await activate(context as never);
  assert.strictEqual(getActivationState().wasmReady, false);
  // 5.1: アウトラインも見出しの上の ▶ も出さない
  assert.strictEqual(mock.registeredDocumentSymbolProviders.length, symbolsBefore);
  assert.strictEqual(mock.registeredCodeLensProviders.length, lensesBefore);
  assert.deepStrictEqual(ghostValidatorCreated(collectionsBefore, watchersBefore), { collections: 0, watchers: 0 });
  assertDevCommandsRegistered(); // WASM と関係なく登録する（1.5）
  deactivate();
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
