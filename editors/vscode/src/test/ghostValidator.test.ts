// ゴースト全体の検査（GhostValidator）のテスト。
//
// vscode のモック（./mock/vscode.ts、esbuild --alias:vscode=./src/test/mock/vscode.ts）
// の上で走る。前半は GhostValidator が使うモックの部品
// （workspace.fs のメモリ上の偽のファイル、保存の通知、ファイルの見張り、
// FileType、Uri.joinPath、Diagnostic の source・code）が想定どおり動くことを、
// 後半は偽の bridge で GhostValidator の集め方と診断の出し方を確かめる。

import * as assert from 'assert';
import * as vscode from 'vscode';
import * as mock from './mock/vscode';
import { GhostValidator, findGhostMaster } from '../ghostValidator';
import type { FileText, GhostFinding, GhostInput, GhostReport, GhostStats } from '../wasmBridge';

let passed = 0;
let failed = 0;
const queue: Array<{ name: string; fn: () => void | Promise<void> }> = [];

function test(name: string, fn: () => void | Promise<void>): void {
  queue.push({ name, fn });
}

const MASTER = vscode.Uri.file('/ws/ghost/master');

async function rejects(p: Thenable<unknown>): Promise<void> {
  let threw = false;
  try {
    await p;
  } catch {
    threw = true;
  }
  assert.ok(threw, 'reject されるはず');
}

test('Uri.joinPath は区切りを 1 つでつなぐ', () => {
  const u = vscode.Uri.joinPath(MASTER, 'dic', 'a.pasta');
  assert.strictEqual(u.path, '/ws/ghost/master/dic/a.pasta');
  assert.strictEqual(u.toString(), vscode.Uri.file('/ws/ghost/master/dic/a.pasta').toString());
});

test('workspace.fs.readFile は置いた中身を返し、無いファイル・読めないファイルは reject', async () => {
  mock.resetFakeFs();
  mock.setFakeFile('/ws/ghost/master/pasta.toml', '[loader]\n');
  mock.setFakeFile('/ws/ghost/master/dic/bad.pasta', new Uint8Array([0xff, 0xfe]));
  mock.setFakeUnreadable('/ws/ghost/master/dic/locked.pasta');

  const toml = await vscode.workspace.fs.readFile(vscode.Uri.joinPath(MASTER, 'pasta.toml'));
  assert.strictEqual(new TextDecoder().decode(toml), '[loader]\n');
  const bad = await vscode.workspace.fs.readFile(vscode.Uri.joinPath(MASTER, 'dic', 'bad.pasta'));
  assert.deepStrictEqual(Array.from(bad), [0xff, 0xfe]);

  await rejects(vscode.workspace.fs.readFile(vscode.Uri.joinPath(MASTER, 'nope.pasta')));
  await rejects(vscode.workspace.fs.readFile(vscode.Uri.joinPath(MASTER, 'dic', 'locked.pasta')));
});

test('workspace.fs.readDirectory は直下の名前と FileType を返す（フォルダは暗黙・リンクは SymbolicLink 付き）', async () => {
  mock.resetFakeFs();
  mock.setFakeFile('/ws/ghost/master/pasta.toml', '');
  mock.setFakeFile('/ws/ghost/master/dic/a.pasta', '');
  mock.setFakeFile('/ws/ghost/master/dic/sub/b.lua', '');
  mock.setFakeSymlink('/ws/ghost/master/dic/link', vscode.FileType.Directory);

  const top = await vscode.workspace.fs.readDirectory(MASTER);
  assert.deepStrictEqual(top, [
    ['dic', vscode.FileType.Directory],
    ['pasta.toml', vscode.FileType.File],
  ]);
  const dic = await vscode.workspace.fs.readDirectory(vscode.Uri.joinPath(MASTER, 'dic'));
  assert.deepStrictEqual(dic, [
    ['a.pasta', vscode.FileType.File],
    ['link', vscode.FileType.Directory | vscode.FileType.SymbolicLink],
    ['sub', vscode.FileType.Directory],
  ]);
  assert.ok((dic[1][1] & vscode.FileType.SymbolicLink) !== 0);

  await rejects(vscode.workspace.fs.readDirectory(vscode.Uri.file('/ws/ghost/other')));
});

test('workspace.fs.stat はファイル・フォルダの種類を返し、無ければ reject', async () => {
  mock.resetFakeFs();
  mock.setFakeFile('/ws/ghost/master/dic/a.pasta', 'abc');

  const f = await vscode.workspace.fs.stat(vscode.Uri.joinPath(MASTER, 'dic', 'a.pasta'));
  assert.strictEqual(f.type, vscode.FileType.File);
  assert.strictEqual(f.size, 3);
  const d = await vscode.workspace.fs.stat(MASTER);
  assert.strictEqual(d.type, vscode.FileType.Directory);
  await rejects(vscode.workspace.fs.stat(vscode.Uri.joinPath(MASTER, 'pasta.toml')));
});

test('onDidSaveTextDocument はテストから発火できる', () => {
  const seen: unknown[] = [];
  const sub = vscode.workspace.onDidSaveTextDocument((d) => seen.push(d));
  const doc = { uri: vscode.Uri.joinPath(MASTER, 'dic', 'a.pasta') };
  mock.didSaveTextDocumentEmitter.fire(doc);
  sub.dispose();
  mock.didSaveTextDocumentEmitter.fire(doc);
  assert.deepStrictEqual(seen, [doc]);
});

test('createFileSystemWatcher はパターンを記録し、作成・削除をテストから発火できる', () => {
  const watcher = vscode.workspace.createFileSystemWatcher('**/ghost/master/**/*.{pasta,lua}');
  const w = mock.createdFileSystemWatchers[mock.createdFileSystemWatchers.length - 1];
  assert.strictEqual(w.globPattern, '**/ghost/master/**/*.{pasta,lua}');

  const created: string[] = [];
  const deleted: string[] = [];
  watcher.onDidCreate((u) => created.push(u.path));
  watcher.onDidDelete((u) => deleted.push(u.path));
  w.createEmitter.fire(vscode.Uri.joinPath(MASTER, 'dic', 'new.lua'));
  w.deleteEmitter.fire(vscode.Uri.joinPath(MASTER, 'dic', 'old.pasta'));
  assert.deepStrictEqual(created, ['/ws/ghost/master/dic/new.lua']);
  assert.deepStrictEqual(deleted, ['/ws/ghost/master/dic/old.pasta']);

  watcher.dispose();
  assert.strictEqual(w.disposed, true);
  w.createEmitter.fire(vscode.Uri.joinPath(MASTER, 'dic', 'late.lua'));
  assert.strictEqual(created.length, 1);
});

test('Diagnostic は source と code を持てる', () => {
  const r = new vscode.Range(new vscode.Position(0, 0), new vscode.Position(0, 0));
  const d = new vscode.Diagnostic(r, 'm', vscode.DiagnosticSeverity.Warning);
  d.source = 'pasta validate';
  d.code = 'scene-name-normalized';
  assert.strictEqual(d.source, 'pasta validate');
  assert.strictEqual(d.code, 'scene-name-normalized');
});

test('出力チャンネルに書いた行はテストから見える', () => {
  const ch = vscode.window.createOutputChannel('Pasta');
  ch.appendLine('検査に失敗しました');
  assert.deepStrictEqual(mock.outputLines[mock.outputLines.length - 1], {
    channel: 'Pasta',
    message: '検査に失敗しました',
  });
});

// --- GhostValidator（集める・検査する・診断を出す） ------------------------------

const EMPTY_STATS: GhostStats = {
  dictionaryFiles: 0,
  errors: 0,
  warnings: 0,
  uncheckedDynamicCalls: 0,
  referencesSkipped: false,
};

/** 偽の bridge。選び出しの結果と報告をテストが決め、呼び出しを記録する */
class FakeBridge {
  selected: string[] = [];
  report: GhostReport = { findings: [], stats: EMPTY_STATS };
  throwOnValidate = false;
  readonly selectCalls: GhostInput[] = [];
  readonly validateCalls: Array<{ input: GhostInput; files: Readonly<Record<string, FileText>> }> = [];

  selectGhostFiles(input: GhostInput): string[] {
    this.selectCalls.push(input);
    return this.selected;
  }

  validateGhost(input: GhostInput, files: Readonly<Record<string, FileText>>): GhostReport {
    this.validateCalls.push({ input, files });
    if (this.throwOnValidate) {
      throw new Error('WASM validateGhost failed: boom');
    }
    return this.report;
  }
}

function finding(
  code: string,
  severity: 'error' | 'warning',
  path: string,
  line: number | null,
  column: number | null,
  message: string
): GhostFinding {
  return { code, severity, path, line, column, message };
}

function newValidator(bridge: FakeBridge): { gv: GhostValidator; col: mock.MockDiagnosticCollection } {
  const gv = new GhostValidator(bridge, vscode.window.createOutputChannel('Pasta Language'));
  const col = mock.createdDiagnosticCollections[mock.createdDiagnosticCollections.length - 1];
  return { gv, col };
}

function uriOf(rel: string): vscode.Uri {
  return vscode.Uri.joinPath(MASTER, ...rel.split('/'));
}

function setOpenDocuments(rels: string[]): void {
  mock.workspace.textDocuments.length = 0;
  for (const rel of rels) {
    mock.workspace.textDocuments.push({ uri: uriOf(rel) });
  }
}

test('findGhostMaster は祖先の ghost/master を返し、無ければ undefined', () => {
  assert.strictEqual(findGhostMaster(vscode.Uri.file('/ws/ghost/master/dic/a.pasta'))?.path, '/ws/ghost/master');
  assert.strictEqual(findGhostMaster(vscode.Uri.file('/ws/ghost/master/pasta.toml'))?.path, '/ws/ghost/master');
  assert.strictEqual(
    findGhostMaster(vscode.Uri.file('/ws/ghost/master/dic/ghost/master/x.pasta'))?.path,
    '/ws/ghost/master/dic/ghost/master'
  );
  assert.strictEqual(findGhostMaster(vscode.Uri.file('/ws/dic/a.pasta')), undefined);
  assert.strictEqual(findGhostMaster(vscode.Uri.file('/ws/master/a.pasta')), undefined);
  assert.strictEqual(findGhostMaster(vscode.Uri.file('/ws/ghost/master')), undefined);
});

test('一覧は再帰・最上位の profile に入らず・リンクを含めず、中身は厳格な UTF-8 で読んで区別する', async () => {
  mock.resetFakeFs();
  setOpenDocuments([]);
  mock.setFakeFile('/ws/ghost/master/pasta.toml', '[loader]\n');
  mock.setFakeFile('/ws/ghost/master/dic/a.pasta', '﻿＊会話\n');
  mock.setFakeFile('/ws/ghost/master/dic/sub/b.lua', 'return 1');
  mock.setFakeFile('/ws/ghost/master/dic/bad.pasta', new Uint8Array([0xe3, 0x81]));
  mock.setFakeUnreadable('/ws/ghost/master/dic/locked.pasta');
  mock.setFakeFile('/ws/ghost/master/dic/profile/c.pasta', '');
  mock.setFakeFile('/ws/ghost/master/profile/x.pasta', '');
  mock.setFakeSymlink('/ws/ghost/master/dic/link.pasta');
  mock.setFakeSymlink('/ws/ghost/master/linkdir/d.pasta', vscode.FileType.Directory);
  const bridge = new FakeBridge();
  bridge.selected = ['dic/a.pasta', 'dic/bad.pasta', 'dic/locked.pasta', 'dic/gone.pasta'];
  const { gv } = newValidator(bridge);

  await gv.validate(MASTER);

  assert.strictEqual(bridge.selectCalls.length, 1);
  const input = bridge.selectCalls[0];
  assert.deepStrictEqual(input.pastaToml, { kind: 'text', text: '[loader]\n' });
  assert.deepStrictEqual([...input.entries].sort(), [
    'dic/a.pasta',
    'dic/bad.pasta',
    'dic/locked.pasta',
    'dic/profile/c.pasta',
    'dic/sub/b.lua',
    'pasta.toml',
  ]);
  assert.strictEqual(bridge.validateCalls.length, 1);
  const { input: vInput, files } = bridge.validateCalls[0];
  assert.deepStrictEqual(vInput, input);
  // 選ばれたファイルだけを読む（b.lua・c.pasta は読まない）
  assert.deepStrictEqual(Object.keys(files).sort(), ['dic/a.pasta', 'dic/bad.pasta', 'dic/gone.pasta', 'dic/locked.pasta']);
  // BOM は残す（CLI の String::from_utf8 と同じ）
  assert.deepStrictEqual(files['dic/a.pasta'], { kind: 'text', text: '﻿＊会話\n' });
  assert.deepStrictEqual(files['dic/bad.pasta'], { kind: 'notUtf8' });
  assert.strictEqual(files['dic/locked.pasta'].kind, 'unreadable');
  assert.deepStrictEqual(files['dic/gone.pasta'], { kind: 'unreadable', reason: '一覧にありません' });
  gv.dispose();
});

test('pasta.toml は無い・UTF-8 でない・読めないを区別する', async () => {
  const bridge = new FakeBridge();
  const { gv } = newValidator(bridge);

  mock.resetFakeFs();
  mock.setFakeFile('/ws/ghost/master/dic/a.pasta', '');
  await gv.validate(MASTER);
  assert.deepStrictEqual(bridge.selectCalls[0].pastaToml, { kind: 'missing' });

  mock.setFakeFile('/ws/ghost/master/pasta.toml', new Uint8Array([0xff]));
  await gv.validate(MASTER);
  assert.deepStrictEqual(bridge.selectCalls[1].pastaToml, { kind: 'unreadable', reason: 'UTF-8 として読めません' });

  mock.setFakeUnreadable('/ws/ghost/master/pasta.toml');
  await gv.validate(MASTER);
  assert.strictEqual(bridge.selectCalls[2].pastaToml.kind, 'unreadable');
  gv.dispose();
});

test('開いていないファイルと pasta.toml に Error・Warning を source・code つきで出す', async () => {
  mock.resetFakeFs();
  setOpenDocuments([]);
  mock.setFakeFile('/ws/ghost/master/dic/b.pasta', '');
  const bridge = new FakeBridge();
  bridge.report = {
    findings: [
      finding('syntax', 'error', 'dic/b.pasta', 3, 5, '構文の誤り'),
      finding('config-missing', 'warning', 'pasta.toml', null, null, 'pasta.toml がありません'),
    ],
    stats: { ...EMPTY_STATS, errors: 1, warnings: 1 },
  };
  const { gv, col } = newValidator(bridge);
  assert.strictEqual(col.name, 'pasta-ghost');

  await gv.validate(MASTER);

  const b = col.get(uriOf('dic/b.pasta'));
  assert.ok(b);
  assert.strictEqual(b.length, 1);
  assert.strictEqual(b[0].severity, vscode.DiagnosticSeverity.Error);
  assert.strictEqual(b[0].message, '構文の誤り');
  assert.strictEqual(b[0].source, 'pasta validate');
  assert.strictEqual(b[0].code, 'syntax');
  assert.deepStrictEqual([b[0].range.start.line, b[0].range.start.character], [2, 4]);

  const t = col.get(uriOf('pasta.toml'));
  assert.ok(t);
  assert.strictEqual(t.length, 1);
  assert.strictEqual(t[0].severity, vscode.DiagnosticSeverity.Warning);
  assert.strictEqual(t[0].code, 'config-missing');
  assert.deepStrictEqual([t[0].range.start.line, t[0].range.start.character], [0, 0]);
  gv.dispose();
});

test('前回あって今回無い URI の診断を消す', async () => {
  mock.resetFakeFs();
  setOpenDocuments([]);
  mock.setFakeFile('/ws/ghost/master/dic/b.pasta', '');
  const bridge = new FakeBridge();
  bridge.report = {
    findings: [
      finding('syntax', 'error', 'dic/b.pasta', 1, 1, 'x'),
      finding('call-target-missing', 'warning', 'dic/c.pasta', 2, null, 'y'),
    ],
    stats: EMPTY_STATS,
  };
  const { gv, col } = newValidator(bridge);
  await gv.validate(MASTER);
  assert.strictEqual(col.size, 2);

  bridge.report = { findings: [finding('call-target-missing', 'warning', 'dic/c.pasta', 2, null, 'y')], stats: EMPTY_STATS };
  await gv.validate(MASTER);
  assert.strictEqual(col.get(uriOf('dic/b.pasta')), undefined);
  assert.strictEqual(col.get(uriOf('dic/c.pasta'))?.length, 1);
  assert.strictEqual(col.size, 1);
  gv.dispose();
});

test('開いている文書の syntax は出さず、閉じると出し、開くとまた外す', async () => {
  mock.resetFakeFs();
  mock.setFakeFile('/ws/ghost/master/dic/b.pasta', '');
  setOpenDocuments(['dic/b.pasta']);
  const bridge = new FakeBridge();
  bridge.report = {
    findings: [
      finding('syntax', 'error', 'dic/b.pasta', 1, 1, '構文'),
      finding('module-name-invalid', 'error', 'dic/b.pasta', null, null, 'モジュール名'),
      finding('syntax', 'error', 'dic/c.pasta', 4, 2, '閉じている文書の構文'),
    ],
    stats: EMPTY_STATS,
  };
  const { gv, col } = newValidator(bridge);
  gv.register();
  await gv.validate(MASTER);

  const codes = (rel: string) => (col.get(uriOf(rel)) ?? []).map((d) => d.code);
  assert.deepStrictEqual(codes('dic/b.pasta'), ['module-name-invalid']);
  assert.deepStrictEqual(codes('dic/c.pasta'), ['syntax']);

  // 閉じる: 検査し直さず、前回の結果から syntax を含めて出し直す
  setOpenDocuments([]);
  mock.didCloseTextDocumentEmitter.fire({ uri: uriOf('dic/b.pasta') });
  assert.deepStrictEqual(codes('dic/b.pasta'), ['syntax', 'module-name-invalid']);

  // 開く: syntax を外して出し直す
  setOpenDocuments(['dic/b.pasta']);
  mock.didOpenTextDocumentEmitter.fire({ uri: uriOf('dic/b.pasta') });
  assert.deepStrictEqual(codes('dic/b.pasta'), ['module-name-invalid']);
  assert.strictEqual(bridge.validateCalls.length, 1);

  // syntax だけの文書を開くと、その URI の診断は消える
  setOpenDocuments(['dic/b.pasta', 'dic/c.pasta']);
  mock.didOpenTextDocumentEmitter.fire({ uri: uriOf('dic/c.pasta') });
  assert.strictEqual(col.get(uriOf('dic/c.pasta')), undefined);

  gv.dispose();
  // dispose の後は購読が外れている
  setOpenDocuments([]);
  mock.didCloseTextDocumentEmitter.fire({ uri: uriOf('dic/c.pasta') });
  assert.strictEqual(col.get(uriOf('dic/c.pasta')), undefined);
});

test('WASM の呼び出しが throw したらそのゴーストの診断を消して出力チャンネルに 1 行残す', async () => {
  mock.resetFakeFs();
  setOpenDocuments([]);
  mock.setFakeFile('/ws/ghost/master/dic/b.pasta', '');
  const bridge = new FakeBridge();
  bridge.report = { findings: [finding('syntax', 'error', 'dic/b.pasta', 1, 1, 'x')], stats: EMPTY_STATS };
  const { gv, col } = newValidator(bridge);
  await gv.validate(MASTER);
  assert.strictEqual(col.size, 1);

  const before = mock.outputLines.length;
  bridge.throwOnValidate = true;
  await gv.validate(MASTER);
  assert.strictEqual(col.size, 0);
  const added = mock.outputLines.slice(before);
  assert.strictEqual(added.length, 1);
  assert.strictEqual(added[0].channel, 'Pasta Language');
  assert.ok(added[0].message.includes('boom'), added[0].message);
  gv.dispose();
});

(async () => {
  console.log('\n[ghostValidator Unit Tests (vscode mock alias)]');
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
  console.log(`\nghostValidator: ${passed} passed, ${failed} failed\n`);
  if (failed > 0) {
    process.exit(1);
  }
})();
