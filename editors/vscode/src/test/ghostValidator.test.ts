// ゴースト全体の検査（GhostValidator）のテスト。
//
// vscode のモック（./mock/vscode.ts、esbuild --alias:vscode=./src/test/mock/vscode.ts）
// の上で走る。ここではまず、GhostValidator が使うモックの部品
// （workspace.fs のメモリ上の偽のファイル、保存の通知、ファイルの見張り、
// FileType、Uri.joinPath、Diagnostic の source・code）が想定どおり動くことを確かめる。

import * as assert from 'assert';
import * as vscode from 'vscode';
import * as mock from './mock/vscode';

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
