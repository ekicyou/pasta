// link-check-test.mjs — link-check の自動検証（manual-ssot-authority タスク 3.1 / 要件 8.4）。
//
// 検証方針:
//   book/src を恒久変更しないため、検証は
//   (A) 実リポジトリ現状でのクリーン判定（exit 0 相当）と、
//   (B) 一時サンドボックス（tmp ディレクトリへ最小フィクスチャを構築）への
//       リンク切れ注入で行う。
//   サンドボックスは runLinkCheck(repoRoot) の repoRoot を差し替えて使う
//   （本物の book/src には一切書き込まない）。
//
// 観測する完了条件（design「LinkCheck」Validation）:
//   - クリーン（リンク健全）→ failed=false。
//   - book 内 .md リンク切れ / 存在しない GitHub blob URL / トラバーサル → 検出＆ failed=true。
//   - 同一入力 → 同一結果（決定性）。
//   - maskFences が CommonMark 準拠でフェンス内の行を空行に置換する（行数は保つ）。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import {
  REPO_ROOT,
  REPO_SLUG,
  LINK_RE,
  maskFences,
  extractLinks,
  githubUrlToRepoPath,
  runLinkCheck,
  reportLinkCheck,
} from './link-check.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));

// --- 最小 assert ハーネス（依存ゼロ） ---
let passed = 0;
let failed = 0;
const log = (...a) => console.log(...a);
function check(name, cond, detail) {
  if (cond) {
    passed++;
    log(`  PASS  ${name}`);
  } else {
    failed++;
    log(`  FAIL  ${name}${detail ? '  -- ' + detail : ''}`);
  }
}

// 一時サンドボックスを作る（book/src/grammar を最小構成で配置）。戻り値はルートパス。
function makeSandbox() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'link-check-'));
  fs.mkdirSync(path.join(root, 'doc', 'spec'), { recursive: true });
  fs.mkdirSync(path.join(root, 'book', 'src', 'grammar'), { recursive: true });
  return root;
}

function writeFile(root, rel, content) {
  const abs = path.join(root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, content, 'utf8');
  return abs;
}

function rmrf(root) {
  fs.rmSync(root, { recursive: true, force: true });
}

// ============================================================
log('\n== (A) 実リポジトリ現状: クリーン判定 ==');
{
  const result = runLinkCheck(REPO_ROOT);
  check('実リポジトリでリンク切れ 0 件', result.broken.length === 0,
    `broken=${JSON.stringify(result.broken)}`);
  check('実リポジトリで failed=false（exit 0 相当）', result.failed === false);
  check('reportLinkCheck が文字列を返す', typeof reportLinkCheck(result) === 'string');
}

// ============================================================
log('\n== (B-1) クリーン サンドボックス ==');
{
  const root = makeSandbox();
  try {
    writeFile(root, 'doc/spec/02-markers.md', '# 02 markers\n');
    writeFile(root, 'book/src/grammar/markers.md',
      'マーカー解説。[block](block-structure.md) '
      + `[spec](https://github.com/${REPO_SLUG}/blob/main/doc/spec/02-markers.md)\n`);
    writeFile(root, 'book/src/grammar/block-structure.md', '# block\n');

    const result = runLinkCheck(root);
    check('クリーン: リンク切れ 0', result.broken.length === 0, JSON.stringify(result.broken));
    check('クリーン: failed=false', result.failed === false);
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-4) リンク切れ注入（book 内 .md / GitHub blob URL） ==');
{
  const root = makeSandbox();
  try {
    // (a) 存在しない book 内 .md リンク + (b) リポジトリ内に存在しない blob URL。
    writeFile(root, 'book/src/grammar/markers.md',
      '壊れた相対リンク [x](does-not-exist.md) '
      + `壊れた blob [y](https://github.com/${REPO_SLUG}/blob/main/doc/spec/99-nope.md) `
      + '外部リンク [ok](https://example.com/page) ' // 外部はスキップ（健全扱い）
      + '健全相対 [ok2](block-structure.md)\n');
    writeFile(root, 'book/src/grammar/block-structure.md', '# block\n');

    const result = runLinkCheck(root);
    const kinds = result.broken.map((b) => b.kind).sort();
    check('リンク切れ 2 件検出', result.broken.length === 2, JSON.stringify(result.broken));
    check('内訳: internal-md と github-repo-path',
      JSON.stringify(kinds) === JSON.stringify(['github-repo-path', 'internal-md']),
      JSON.stringify(kinds));
    check('外部リンク・健全相対は誤検出しない',
      !result.broken.some((b) => /example\.com|block-structure/.test(b.target)));
    check('リンク切れで failed=true（exit 1 相当）', result.failed === true);
    const rep = reportLinkCheck(result);
    check('失敗レポートに RESULT: FAIL と BROKEN 行',
      rep.includes('RESULT: FAIL') && rep.includes('BROKEN  book/src/grammar/markers.md'), rep);
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-5) ユニット: extractLinks / LINK_RE / githubUrlToRepoPath ==');
{
  const links = extractLinks('[a](foo.md) ![img](pic.png) [b](<spaced url.md>) [c](http://x/y "t")');
  check('extractLinks: 相対/画像/山括弧/タイトル付きを抽出',
    links.includes('foo.md') && links.includes('pic.png')
    && links.includes('spaced url.md') && links.includes('http://x/y'),
    JSON.stringify(links));

  check('LINK_RE は global な RegExp', LINK_RE instanceof RegExp && LINK_RE.global);
  const viaRe = [...'[a](foo.md) [b](<x y.md>)'.matchAll(LINK_RE)].map((m) => m[1]);
  check('LINK_RE: 先頭キャプチャがリンク先（山括弧は保持）',
    JSON.stringify(viaRe) === JSON.stringify(['foo.md', '<x y.md>']), JSON.stringify(viaRe));

  check('githubUrlToRepoPath: 自リポ blob → 相対パス',
    githubUrlToRepoPath(`https://github.com/${REPO_SLUG}/blob/main/doc/spec/02-markers.md`)
    === 'doc/spec/02-markers.md');
  check('githubUrlToRepoPath: 自リポ tree → 相対パス',
    githubUrlToRepoPath(`https://github.com/${REPO_SLUG}/tree/main/doc/spec`)
    === 'doc/spec');
  check('githubUrlToRepoPath: 別リポは null',
    githubUrlToRepoPath('https://github.com/other/repo/blob/main/x.md') === null);
  check('githubUrlToRepoPath: 外部 URL は null',
    githubUrlToRepoPath('https://example.com/doc/spec/02.md') === null);
}

// ============================================================
log('\n== (B-6) 決定論性（同一入力 → 同一結果） ==');
{
  const r1 = runLinkCheck(REPO_ROOT);
  const r2 = runLinkCheck(REPO_ROOT);
  check('2 回実行で同一レポート', reportLinkCheck(r1) === reportLinkCheck(r2));
}

// ============================================================
log('\n== (B-11) リンクのフラグメント/クエリ許容（誤検出なし） ==');
{
  const root = makeSandbox();
  try {
    writeFile(root, 'doc/spec/02-markers.md', '# 02\n');
    writeFile(root, 'book/src/grammar/markers.md',
      'アンカー付き [a](block-structure.md#sec) '
      + 'クエリ付き [b](block-structure.md?x=1) '
      + `blob フラグメント [c](https://github.com/${REPO_SLUG}/blob/main/doc/spec/02-markers.md#L10)\n`);
    writeFile(root, 'book/src/grammar/block-structure.md', '# block\n');

    const result = runLinkCheck(root);
    check('フラグメント/クエリ付き実在リンクは壊れ扱いしない',
      result.broken.length === 0, JSON.stringify(result.broken));
    check('failed=false', result.failed === false);
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-13) CLI 結線（exit code / RESULT 出力） ==');
{
  const cliPath = path.join(here, 'link-check.mjs');
  const r = spawnSync(process.execPath, [cliPath], { encoding: 'utf8', timeout: 60000 });
  check('CLI: クリーンな実リポジトリで exit 0', r.status === 0,
    `status=${r.status} stderr=${(r.stderr || '').slice(0, 300)}`);
  check('CLI: RESULT: OK を出力', /RESULT: OK/.test(r.stdout || ''),
    (r.stdout || '').slice(-200));
}

// ============================================================
log('\n== (B-14) パストラバーサル拒否（repoRoot 外を指すリンクは実在しても broken） ==');
// ハードニング境界（cell 3.59 / R3.2, R3.3）:
//   `..` 等で repoRoot の外へ脱出するリンク（blob URL・相対 .md とも）は、
//   解決先が実在しても「リンク切れ」として報告する（リポジトリ外の存在プローブ防止）。
//   一方、repoRoot 「内」に留まる `..` 相対参照は従来どおり実在チェックのみ（正常系不変）。
{
  // サンドボックスを入れ子にし、repoRoot（= outer/repo）の外に実在ファイルを置く。
  const outer = fs.mkdtempSync(path.join(os.tmpdir(), 'link-check-'));
  const root = path.join(outer, 'repo');
  try {
    fs.mkdirSync(path.join(root, 'doc', 'spec'), { recursive: true });
    fs.mkdirSync(path.join(root, 'book', 'src', 'grammar'), { recursive: true });
    writeFile(outer, 'outside.md', '# repoRoot の外に実在するファイル\n');

    writeFile(root, 'doc/spec/02-markers.md', '# 02\n');
    writeFile(root, 'book/src/grammar/markers.md',
      // (a) blob URL の `..` 脱出（解決先は実在する）。
      `[esc-blob](https://github.com/${REPO_SLUG}/blob/main/../outside.md) `
      // (b) 相対 .md リンクの `..` 脱出（grammar→src→book→repo→outer。解決先は実在する）。
      + '[esc-rel](../../../../outside.md) '
      // (c) repoRoot 内に留まる `..` 相対参照（実在）→ 従来どおり健全。
      + '[ok-up](../../../doc/spec/02-markers.md)\n');

    const result = runLinkCheck(root);
    const escBlob = result.broken.find(
      (b) => b.kind === 'github-repo-path' && /トラバーサル/.test(b.detail));
    const escRel = result.broken.find(
      (b) => b.kind === 'internal-md' && /トラバーサル/.test(b.detail));
    check('blob URL の repoRoot 脱出を実在でも broken 扱い', !!escBlob,
      JSON.stringify(result.broken));
    check('相対 .md の repoRoot 脱出を実在でも broken 扱い', !!escRel,
      JSON.stringify(result.broken));
    check('repoRoot 内の .. 相対参照（実在）は誤検出しない（境界の内側は不変）',
      !result.broken.some((b) => /ok-up|02-markers/.test(b.target)),
      JSON.stringify(result.broken));
    check('トラバーサル検出 2 件のみ', result.broken.length === 2,
      JSON.stringify(result.broken));
    check('トラバーサルで failed=true（exit 1 相当）', result.failed === true);
  } finally {
    rmrf(outer);
  }
}

// ============================================================
log('\n== (C) maskFences（CommonMark 準拠のフェンス判定・行数保持） ==');
{
  const lines = (s) => s.split('\n');

  // 3 連バッククォート: 開閉行と内側を空行に、外側は不変、行数は保つ。
  {
    const src = ['前 [a](x.md)', '```lua', '[b](y.md)', '```', '後'].join('\n');
    const out = maskFences(src);
    check('3 連: 行数を保つ', lines(out).length === lines(src).length, JSON.stringify(out));
    check('3 連: フェンス内（開閉行含む）を空行に',
      JSON.stringify(lines(out)) === JSON.stringify(['前 [a](x.md)', '', '', '', '後']),
      JSON.stringify(out));
  }

  // 4 連フェンス内の 3 連行はフェンス内のまま（閉じない）。
  {
    const src = ['````pasta', '＊シーン', '```lua', 'x = 1 -- ですわ', '```', '---', '````', '外'].join('\n');
    const out = maskFences(src);
    check('4 連フェンス内の 3 連行はフェンスを閉じない（内側全部が空行）',
      JSON.stringify(lines(out)) === JSON.stringify(['', '', '', '', '', '', '', '外']),
      JSON.stringify(out));
  }

  // ~~~ フェンス、および ``` では閉じない。
  {
    const src = ['~~~', '```', 'code', '~~~', '外'].join('\n');
    const out = maskFences(src);
    check('~~~ フェンスは ``` で閉じず ~~~ で閉じる',
      JSON.stringify(lines(out)) === JSON.stringify(['', '', '', '', '外']), JSON.stringify(out));
  }

  // 開始より多い個数の閉じフェンスは閉じる。
  {
    const src = ['```', 'code', '`````', '外'].join('\n');
    const out = maskFences(src);
    check('開始以上の個数（5 連）で閉じる',
      JSON.stringify(lines(out)) === JSON.stringify(['', '', '', '外']), JSON.stringify(out));
  }

  // 情報文字列つきの行は閉じフェンスにならない。
  {
    const src = ['```', '```lua', 'code', '```', '外'].join('\n');
    const out = maskFences(src);
    check('情報文字列つき行（```lua）は閉じない',
      JSON.stringify(lines(out)) === JSON.stringify(['', '', '', '', '外']), JSON.stringify(out));
  }

  // 末尾の空白は情報文字列ではない（閉じる）。
  {
    const src = ['```', 'code', '```   ', '外'].join('\n');
    const out = maskFences(src);
    check('閉じフェンス末尾の空白は許容',
      JSON.stringify(lines(out)) === JSON.stringify(['', '', '', '外']), JSON.stringify(out));
  }

  // 2 連はフェンスでない／閉じないフェンスは文書末尾まで続く。
  {
    check('2 連バッククォートはフェンスでない', maskFences('``\n[a](x.md)\n``') === '``\n[a](x.md)\n``');
    const out = maskFences('外\n```\ncode\n[a](x.md)');
    check('閉じないフェンスは文書末尾まで', out === '外\n\n\n', JSON.stringify(out));
  }

  // 3 スペースまでのインデントはフェンス、4 スペースはフェンスでない。
  {
    check('3 スペースインデントの開閉はフェンス',
      maskFences('   ```\ncode\n   ```\n外') === '\n\n\n外');
    check('4 スペースインデントはフェンスでない',
      maskFences('    ```\ncode') === '    ```\ncode');
  }

  // バッククォートを含む情報文字列はフェンスでない（インラインコード）。
  check('情報文字列にバッククォートを含む行はフェンスでない',
    maskFences('``` a`b\ncode') === '``` a`b\ncode');

  // CRLF 入力でも行数を保つ。
  {
    const out = maskFences('a\r\n```\r\ncode\r\n```\r\nb');
    check('CRLF 入力: LF 正規化して行数を保つ', out === 'a\n\n\n\nb', JSON.stringify(out));
  }
}

// ============================================================
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
