// link-check-test.mjs — link-check の自動検証（manual-ssot-authority タスク 3.1・3.2 / 要件 8.4, 5.3, 6.1, 6.4, 6.5, 10.2, 2.4）。
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
//   - headingSlug が GitHub 方式の slug を返し、重複見出しを -1・-2 と付番する（3.2）。
//   - checkSkillSelfContained が 2 スキルの規則 (a)〜(d) 違反を種別付きで検出し、
//     https・実在アンカー・明示アンカー（#s6-6）は許可する（3.2）。
//   実リポジトリのスキル検査は P4（タスク 4.x）完了まで違反ありでよいため、
//   (A)・(B-13) は book 部分（internal-md / github-repo-path / internals-path / readme-manual-url）が
//   0 件であることだけを確かめる。
//   - checkInternalsPaths が内部設計章のフェンス外インラインコードのリポジトリ内パスについて、
//     実在しない・`:行番号` 付き・トラバーサル・末尾 `/` なのにディレクトリでないものを
//     internals-path として行番号付きで検出し、フェンス内・接頭辞外・内部設計章以外は対象外とする
//     （pasta-runtime-internals-doc タスク 1.1 / 要件 3.2, 3.7, 1.6）。
//   - checkReadmeManualLinks が crates/*/README.md のフェンス外のリンクと裸の URL のうち公開マニュアルを
//     指すものについて、写像先の章・見出し（日本語見出し・パーセントエンコード・明示アンカー）の実在を照合し、
//     存在しない章・アンカー・トラバーサル・章に写像できない URL を readme-manual-url として行番号付きで検出する。
//     インラインコード内は対象外（対象の文書は下のタスク 3.2 の項）。manual.yml は crates/*/README.md の変更でも起動する
//     （pasta-runtime-internals-doc タスク 1.2 / 要件 7.5, 7.8, 1.6）。
//   - detectBrokenLinks が本の中のリンク（`章.md#見出し`・`#見出し`）の見出しをリンク先の章のアンカーの集合と照合し、
//     無ければ internal-anchor として報告する（行番号なし）。章のファイルが無いときは internal-md の 1 件だけ
//     （manual-link-anchor-check タスク 2.1 / 要件 1.1〜1.5, 2.1, 2.2, 3.1〜3.6, 5.1, 5.5, 7.1〜7.3）。
//   - checkReadmeManualLinks がクレートの README に加えて、ルートの README.md・SOUL.md・2 スキルの Markdown の
//     公開マニュアルの URL も、同じ区分（readme-manual-url）・同じ文面で検出する。対象の順は固定。
//     .kiro/steering/roadmap.md は対象外（manual-link-anchor-check タスク 3.2 / 要件 4.4〜4.6, 5.5, 7.1）。
//   - 自己テストの中の compareHeadingIds が、章ごとに headingSlugs の導く id の並びと、ビルドした出力（book/book/）の
//     見出しの id の並びを順番まで比べる。出力のフォルダが無ければ null、出力が無い章・出力が章より古い章は飛ばす。
//     見本は一時フォルダに置く（I-1）。link-check.mjs は book/book/ を読まず、出力の有無で結果が変わらない
//     （manual-link-anchor-check タスク 4.1 / 要件 6.2, 7.5, 7.6, 7.7）。
//   - I-2 が、実リポジトリに compareHeadingIds をかける。CI の外では、出力が無ければ飛ばして確かめの数に入れず、
//     在れば食い違い 0 を確かめて、突き合わせた章の数と飛ばした章を表示する。CI の中（環境変数 CI）では、
//     出力が無いか、飛ばした章が 1 つでも在れば失敗にする。食い違いはどちらでも失敗。判定の枝は見本の結果で確かめる
//     （manual-link-anchor-check タスク 4.2 / 要件 3.7, 7.6, 7.7, 7.8）。

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
  listMarkdownFiles,
  extractLinks,
  headingSlug,
  headingSlugs,
  checkSkillSelfContained,
  checkInternalsPaths,
  checkReadmeManualLinks,
  MANUAL_URL,
  INTERNALS_DIR,
  REPO_PATH_PREFIXES,
  CHECKED_SKILLS,
  FORBIDDEN_SKILL_TOKENS,
  BOOK_KINDS,
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
  const bookBroken = result.broken.filter((b) => BOOK_KINDS.has(b.kind));
  check('実リポジトリで book 内リンク切れ 0 件', bookBroken.length === 0,
    `broken=${JSON.stringify(bookBroken)}`);
  check('failed はいずれかの違反の有無と一致', result.failed === (result.broken.length > 0));
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
    writeFile(root, 'book/src/grammar/block-structure.md', '# block\n\n## sec\n');

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
  const expected = runLinkCheck(REPO_ROOT).failed ? 1 : 0;
  check(`CLI: 違反の有無どおりの exit code（期待 ${expected}）`, r.status === expected,
    `status=${r.status} stderr=${(r.stderr || '').slice(0, 300)}`);
  check('CLI: book 部分は 0 件', /\[1\][^\n]*: 0 件/.test(r.stdout || ''),
    (r.stdout || '').slice(0, 300));
  check('CLI: スキル検査の節を出力', /\[2\] スキル自己完結/.test(r.stdout || ''),
    (r.stdout || '').slice(0, 300));
  check('CLI: RESULT 行を出力', /RESULT: (OK|FAIL)/.test(r.stdout || ''),
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
log('\n== (B-15) 台詞の引用ブロック内のリンク（manual-claudia-theme タスク 3.3 / 要件 10.6, 10.7） ==');
// 台詞部品（`> 【話し手：表情】本文`）の本文中のリンクも、地の文と同じ規則で検証される。
// 壊れた相対 .md は internal-md、存在しない blob URL は github-repo-path で検出し、
// 実在するリンク（アンカー付きを含む）は台詞の中でも誤検出しない。
{
  const root = makeSandbox();
  try {
    writeFile(root, 'doc/spec/02-markers.md', '# 02\n');
    writeFile(root, 'book/src/grammar/block-structure.md', '# block\n\n## sec\n');
    writeFile(root, 'book/src/introduction.md', [
      '# はじめに',
      '',
      // (a) アンソニーの 1 行の台詞に、壊れた相対リンク。
      '> 【アンソニー：刮目】詳しくは [文法](grammar/missing.md) を。',
      '',
      // (b) クローディアの複数行の台詞。継続行に存在しない blob URL。
      '> 【クローディア：したり顔】仕様はこちら。',
      `> [仕様](https://github.com/${REPO_SLUG}/blob/main/doc/spec/99-nope.md) をどうぞ。`,
      '',
      // (c) 健全なリンクだけを含む台詞（相対・アンカー付き・実在 blob URL）。
      '> 【クローディア】[ブロック](grammar/block-structure.md#sec) と '
        + `[仕様](https://github.com/${REPO_SLUG}/blob/main/doc/spec/02-markers.md) は大丈夫。`,
      '',
    ].join('\n'));

    const result = runLinkCheck(root);
    const talkMd = result.broken.find((b) => b.kind === 'internal-md'
      && b.file === 'book/src/introduction.md' && b.target === 'grammar/missing.md');
    const talkBlob = result.broken.find((b) => b.kind === 'github-repo-path'
      && b.file === 'book/src/introduction.md' && /99-nope\.md$/.test(b.target));
    check('台詞内の壊れた相対 .md を internal-md で検出', !!talkMd, JSON.stringify(result.broken));
    check('internal-md の詳細は従来の文言', !!talkMd && /book 内リンク先が存在しない: grammar\/missing\.md/.test(talkMd.detail),
      JSON.stringify(talkMd));
    check('台詞の継続行の壊れた blob URL を github-repo-path で検出', !!talkBlob,
      JSON.stringify(result.broken));
    check('台詞内の健全リンクは誤検出しない（検出は 2 件のみ）', result.broken.length === 2,
      JSON.stringify(result.broken));
    check('台詞内のリンク切れで failed=true（exit 1 相当）', result.failed === true);
    check('失敗レポートに台詞の章の BROKEN 行',
      reportLinkCheck(result).includes('BROKEN  book/src/introduction.md'), reportLinkCheck(result));
  } finally {
    rmrf(root);
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
log('\n== (D) headingSlug / headingSlugs（GitHub 方式） ==');
{
  const cases = [
    ['## Overview', 'overview'],
    ['## 単語定義', '単語定義'],
    ['## [package] 予約注記', 'package-予約注記'],
    ['### 予約グローバル変数（pasta_ で始まる名前）', '予約グローバル変数pasta_-で始まる名前'],
    ['### set_scene_selector(...) / set_word_selector(...)', 'set_scene_selector--set_word_selector'],
    ['### `ACT:talk(text)` の使い方', 'acttalktext-の使い方'],
    ['## [リンク](other.md#x) 付き', 'リンク-付き'],
    ['## A-B_c 2', 'a-b_c-2'],
    ['同一スポット共有時の外見の復旧', '同一スポット共有時の外見の復旧'],
    ['  ## 前後空白  ', '前後空白'],
  ];
  for (const [h, want] of cases) {
    const got = headingSlug(h);
    check(`headingSlug(${JSON.stringify(h)}) = ${want}`, got === want, `got=${got}`);
  }

  const md = ['# T', '## 例', '```', '## 例', '```', '## 例', '### 例', '## 別'].join('\n');
  const slugs = headingSlugs(md);
  check('headingSlugs: フェンス内を除き重複に -1・-2 を付番',
    JSON.stringify(slugs) === JSON.stringify(['t', '例', '例-1', '例-2', '別']), JSON.stringify(slugs));
}

// ============================================================
log('\n== (E) checkSkillSelfContained（規則 a〜d） ==');
{
  check('CHECKED_SKILLS は 2 スキル',
    JSON.stringify(CHECKED_SKILLS) === JSON.stringify(['pasta-ghost-authoring', 'pasta-lua-coding']));
  check('FORBIDDEN_SKILL_TOKENS は 4 語',
    JSON.stringify(FORBIDDEN_SKILL_TOKENS) === JSON.stringify(['doc/spec', 'GRAMMAR.md', 'book/src', 'crates/']));

  const G = '.claude/skills/pasta-ghost-authoring';
  const L = '.claude/skills/pasta-lua-coding';

  // E-1: 許可ケースのみ（違反 0）。
  {
    const root = makeSandbox();
    try {
      writeFile(root, `${G}/SKILL.md`, [
        '# skill',
        '- [a](references/a.md)',
        '- [b](references/b.md#package-予約注記)',
        '- [p](references/patterns.md#s6-6)',
        '- [ext](https://ekicyou.github.io/pasta/grammar/index.html#x)',
        '- [self](#skill)',
        '',
      ].join('\n'));
      writeFile(root, `${G}/references/a.md`, [
        '# A',
        '## 同名', '## 同名',
        '[dup](#同名-1) [sib](b.md#予約グローバル変数pasta_-で始まる名前) [enc](b.md#%E4%BE%8B)',
        '```markdown',
        '[フェンス内は無視](../../../escape.md) doc/x',
        '```',
      ].join('\n'));
      writeFile(root, `${G}/references/b.md`, [
        '# B', '## [package] 予約注記', '### 予約グローバル変数（pasta_ で始まる名前）', '## 例',
      ].join('\n'));
      writeFile(root, `${G}/references/patterns.md`, '# P\n\n<a id="s6-6"></a>\n### 6.6 x\n');
      writeFile(root, `${L}/SKILL.md`, '# lua\n');
      // [ext] の公開 URL が指す章。スキルの Markdown も公開 URL の検査の対象なので、章と見出しを見本に置く。
      writeFile(root, 'book/src/grammar/index.md', '# 文法\n\n## x\n');
      const broken = checkSkillSelfContained(root);
      check('E-1 許可ケース（https・実在アンカー・重複付番・#s6-6・フェンス内）: 違反 0',
        broken.length === 0, JSON.stringify(broken));
      check('E-1 runLinkCheck も failed=false', runLinkCheck(root).failed === false);
    } finally {
      rmrf(root);
    }
  }

  // E-2: 各違反の検出。
  {
    const root = makeSandbox();
    try {
      writeFile(root, 'outside.md', '# outside\n');
      writeFile(root, `${G}/SKILL.md`, [
        '# skill',
        '- [esc](../../../outside.md)',
        '- [man](../../../book/src/grammar/index.md)',
        '- [miss](references/nope.md)',
        '- [a](references/a.md#no-such-heading)',
        '- [self](#missing-self)',
        '',
      ].join('\n'));
      writeFile(root, `${G}/references/a.md`,
        '<!-- source: doc/spec/02-markers.md -->\n# A\n\nGRAMMAR.md を参照。\n[x](only-from-ref.md)\n');
      writeFile(root, `${G}/references/orphan.md`, '# orphan\n');
      writeFile(root, `${G}/references/only-from-ref.md`, '# only from ref\n');
      writeFile(root, `${L}/SKILL.md`, '# lua\n[t](references/t.md)\n');
      writeFile(root, `${L}/references/t.md`, '# T\n\n```text\ncrates/pasta_lua/src/x.rs\n```\n');

      const broken = checkSkillSelfContained(root);
      const has = (kind, re) => broken.some((b) => b.kind === kind && re.test(`${b.file} ${b.target} ${b.detail}`));
      check('a: ../ でスキル外へ脱出 → skill-escape', has('skill-escape', /outside\.md/), JSON.stringify(broken));
      check('a: book/src を指すリンクも skill-escape', has('skill-escape', /book\/src\/grammar/), JSON.stringify(broken));
      check('a: 実在しない references/nope.md → skill-missing', has('skill-missing', /nope\.md/), JSON.stringify(broken));
      check('b: 存在しない見出しへのアンカー → skill-anchor', has('skill-anchor', /no-such-heading/), JSON.stringify(broken));
      check('b: 同一ファイル内 #missing-self → skill-anchor', has('skill-anchor', /missing-self/), JSON.stringify(broken));
      check('c: HTML コメント内 doc/spec → skill-forbidden-ref', has('skill-forbidden-ref', /a\.md.*doc\/spec/), JSON.stringify(broken));
      check('c: 本文 GRAMMAR.md → skill-forbidden-ref', has('skill-forbidden-ref', /a\.md.*GRAMMAR\.md/), JSON.stringify(broken));
      check('c: フェンス内の crates/ も検出（全文検査）', has('skill-forbidden-ref', /t\.md.*crates\//), JSON.stringify(broken));
      check('c: SKILL.md 本文の book/src も検出', has('skill-forbidden-ref', /SKILL\.md.*book\/src/), JSON.stringify(broken));
      check('d: SKILL.md 未リンクの references/orphan.md → skill-unlisted', has('skill-unlisted', /orphan\.md/), JSON.stringify(broken));
      check('d: references 内からのみリンクされ SKILL.md から未リンク → skill-unlisted',
        has('skill-unlisted', /only-from-ref\.md/), JSON.stringify(broken));
      check('d: リンク済みの a.md・t.md は unlisted にならない',
        !broken.some((b) => b.kind === 'skill-unlisted' && /\/(a|t)\.md/.test(b.file)), JSON.stringify(broken));
      check('file はリポジトリ相対', broken.every((b) => b.file.startsWith('.claude/skills/')), JSON.stringify(broken));

      const result = runLinkCheck(root);
      check('E-2 runLinkCheck: スキル違反で failed=true', result.failed === true);
      const rep = reportLinkCheck(result);
      check('E-2 レポートに種別が出る',
        ['skill-escape', 'skill-missing', 'skill-anchor', 'skill-forbidden-ref', 'skill-unlisted']
          .every((k) => rep.includes(k)), rep);
    } finally {
      rmrf(root);
    }
  }
}

// ============================================================
log('\n== (F) checkInternalsPaths（内部設計章のリポジトリ内パス実在） ==');
{
  check('INTERNALS_DIR は book/src/internals', INTERNALS_DIR === 'book/src/internals');
  check('REPO_PATH_PREFIXES は 6 接頭辞',
    JSON.stringify(REPO_PATH_PREFIXES)
    === JSON.stringify(['crates/', 'book/', '.github/', '.cargo/', '.kiro/', '.claude/']),
    JSON.stringify(REPO_PATH_PREFIXES));

  // F-1: 内部設計章が無ければ検査しない（違反 0）。
  {
    const root = makeSandbox();
    try {
      check('F-1 内部設計章が無ければ違反 0', checkInternalsPaths(root).length === 0);
    } finally {
      rmrf(root);
    }
  }

  // F-2: 合格・違反・対象外の混在。repoRoot 外に実在ファイルを置き、トラバーサルを確かめる。
  {
    const outer = fs.mkdtempSync(path.join(os.tmpdir(), 'link-check-'));
    const root = path.join(outer, 'repo');
    try {
      writeFile(outer, 'outside.md', '# repoRoot の外に実在\n');
      writeFile(root, 'crates/pasta_lua/src/lib.rs', '// lib\n');
      writeFile(root, '.claude/skills/x/SKILL.md', '# x\n');
      writeFile(root, 'book/src/internals/index.md', [
        '# 概要', // L1
        '実在ファイル `crates/pasta_lua/src/lib.rs` と実在ディレクトリ `crates/pasta_lua/src/`。', // L2
        '二重バッククォート `` .claude/skills/x/SKILL.md `` も合格。', // L3
        '欠落 `crates/pasta_lua/src/missing.rs`。', // L4
        '行番号付き `crates/pasta_lua/src/lib.rs:12`。', // L5
        'トラバーサル `crates/../../outside.md`。', // L6
        'ファイルに末尾スラッシュ `crates/pasta_lua/src/lib.rs/`。', // L7
        '二重バッククォートの欠落 `` book/nope.md ``。', // L8
        '接頭辞外 `scripts/main.lua` と `pasta.store`、空白入り `crates/a b`。', // L9
        '```text', // L10
        'フェンス内 `crates/fenced-nope.rs`', // L11
        '```', // L12
        '',
      ].join('\n'));
      writeFile(root, 'book/src/internals/sub/deep.md', '# 深い章\n\n`.github/workflows/nope.yml`\n');
      // 内部設計章以外は対象外。
      writeFile(root, 'book/src/grammar/markers.md', '`crates/not-checked.rs`\n');

      const broken = checkInternalsPaths(root);
      const targets = broken.map((b) => b.target).sort();
      const want = [
        '.github/workflows/nope.yml',
        'book/nope.md',
        'crates/../../outside.md',
        'crates/pasta_lua/src/lib.rs/',
        'crates/pasta_lua/src/lib.rs:12',
        'crates/pasta_lua/src/missing.rs',
      ];
      check('F-2 違反は欠落・行番号付き・トラバーサル・非ディレクトリ末尾 / ・二重 ` 内欠落・下位章の 6 件',
        JSON.stringify(targets) === JSON.stringify(want), JSON.stringify(broken));
      check('F-2 種別はすべて internals-path', broken.every((b) => b.kind === 'internals-path'),
        JSON.stringify(broken));
      const byTarget = (t) => broken.find((b) => b.target === t) || {};
      check('F-2 detail に行番号（欠落=L4・行番号付き=L5・トラバーサル=L6・下位章=L3）',
        /\bL4\b/.test(byTarget('crates/pasta_lua/src/missing.rs').detail)
        && /\bL5\b/.test(byTarget('crates/pasta_lua/src/lib.rs:12').detail)
        && /\bL6\b/.test(byTarget('crates/../../outside.md').detail)
        && /\bL3\b/.test(byTarget('.github/workflows/nope.yml').detail),
        JSON.stringify(broken));
      check('F-2 トラバーサルは実在してもトラバーサルとして報告',
        /トラバーサル/.test(byTarget('crates/../../outside.md').detail), JSON.stringify(broken));
      check('F-2 file はリポジトリ相対の章パス',
        byTarget('crates/pasta_lua/src/missing.rs').file === 'book/src/internals/index.md'
        && byTarget('.github/workflows/nope.yml').file === 'book/src/internals/sub/deep.md',
        JSON.stringify(broken));
      check('F-2 フェンス内・接頭辞外・空白入り・内部設計章以外は対象外',
        !broken.some((b) => /fenced-nope|scripts\/|pasta\.store|a b|not-checked/.test(b.target)),
        JSON.stringify(broken));

      const result = runLinkCheck(root);
      check('F-2 runLinkCheck に結合され failed=true',
        result.failed === true && result.broken.filter((b) => b.kind === 'internals-path').length === 6,
        JSON.stringify(result.broken));
      const rep = reportLinkCheck(result);
      check('F-2 レポートの [1] 区分に internals-path 6 件が出る',
        /\[1\][^\n]*: 6 件/.test(rep) && rep.includes('[internals-path]'), rep);
    } finally {
      rmrf(outer);
    }
  }

  // F-3: 実在パスだけなら合格。
  {
    const root = makeSandbox();
    try {
      writeFile(root, 'crates/pasta_lua/src/lib.rs', '// lib\n');
      writeFile(root, 'book/src/internals/index.md',
        '# 概要\n\n`crates/pasta_lua/src/lib.rs`・`crates/pasta_lua/`・`book/src/internals/index.md`\n');
      const broken = checkInternalsPaths(root);
      check('F-3 実在パスのみ: 違反 0', broken.length === 0, JSON.stringify(broken));
      check('F-3 runLinkCheck も failed=false', runLinkCheck(root).failed === false);
    } finally {
      rmrf(root);
    }
  }

  // F-4: 実リポジトリで internals-path 0 件。
  {
    const broken = checkInternalsPaths(REPO_ROOT);
    check('F-4 実リポジトリで internals-path 0 件', broken.length === 0, JSON.stringify(broken));
  }
}

// ============================================================
log('\n== (G) checkReadmeManualLinks（クレート README の公開マニュアル URL 実在） ==');
{
  check('MANUAL_URL は公開マニュアルのルート', MANUAL_URL === 'https://ekicyou.github.io/pasta/');
  const U = MANUAL_URL;

  // G-1: クレート README が無ければ違反 0。
  {
    const root = makeSandbox();
    try {
      check('G-1 クレート README が無ければ違反 0', checkReadmeManualLinks(root).length === 0);
    } finally {
      rmrf(root);
    }
  }

  // G-2: 合格・違反・対象外の混在。repoRoot 外に実在ファイルを置き、トラバーサルを確かめる。
  {
    const outer = fs.mkdtempSync(path.join(os.tmpdir(), 'link-check-'));
    const root = path.join(outer, 'repo');
    try {
      writeFile(outer, 'outside.md', '# repoRoot の外に実在\n');
      writeFile(root, 'book/src/introduction.md', '# はじめに\n');
      writeFile(root, 'book/src/debug/index.md', '# デバッグ\n\n## VSCode から接続する\n');
      writeFile(root, 'book/src/lua/modules/index.md',
        '# モジュール\n\n## pasta.store モジュール\n\n<a id="s6-6"></a>\n\n### 6.6 永続化\n');
      const enc = encodeURIComponent('pastastore-モジュール');
      writeFile(root, 'crates/pasta_lua/README.md', [
        '# pasta_lua', // L1
        `[ルート](${U}) と裸のルート ${U}index.html、`, // L2
        `[章](${U}lua/modules/index.html) と [日本語見出し](${U}lua/modules/index.html#pastastore-モジュール)。`, // L3
        `[エンコード済み](${U}lua/modules/index.html#${enc}) と [明示アンカー](<${U}lua/modules/index.html#s6-6>)。`, // L4
        `自動リンク <${U}debug/index.html#vscode-から接続する> とクエリ付き [q](${U}debug/index.html?x=1)。`, // L5
        `欠落章 [x](${U}nope.html)。`, // L6
        `裸の欠落アンカー ${U}lua/modules/index.html#no-such。`, // L7
        `トラバーサル [t](${U}../../outside.html)。`, // L8
        `章に写像できない [p](${U}book.pdf)。`, // L9
        `インラインコード \`${U}code-nope.html\` と他サイト https://example.com/nope.html。`, // L10
        '```text', // L11
        `${U}fenced-nope.html`, // L12
        '```', // L13
        '',
      ].join('\n'));
      writeFile(root, 'crates/pasta_core/README.md', `# core\n\n[欠落アンカー](${U}debug/index.html#無い見出し)\n`);
      // 対象外: book/src・入れ子の README・README 以外（リポジトリ直下の README は対象。G-6 で確かめる）。
      writeFile(root, 'book/src/grammar/markers.md', `[x](${U}book-nope.html)\n`);
      writeFile(root, 'crates/pasta_lua/docs/README.md', `[x](${U}nested-nope.html)\n`);
      writeFile(root, 'crates/pasta_lua/OTHER.md', `[x](${U}other-nope.html)\n`);

      const broken = checkReadmeManualLinks(root);
      const targets = broken.map((b) => b.target).sort();
      const want = [
        `${U}../../outside.html`,
        `${U}book.pdf`,
        `${U}debug/index.html#無い見出し`,
        `${U}lua/modules/index.html#no-such`,
        `${U}nope.html`,
      ].sort();
      check('G-2 違反は欠落章・欠落アンカー（裸・別クレート）・トラバーサル・写像不能の 5 件（重複なし）',
        JSON.stringify(targets) === JSON.stringify(want), JSON.stringify(broken));
      check('G-2 種別はすべて readme-manual-url', broken.every((b) => b.kind === 'readme-manual-url'),
        JSON.stringify(broken));
      const byTarget = (t) => broken.find((b) => b.target === t) || {};
      check('G-2 detail に行番号（欠落章=L6・裸の欠落アンカー=L7・トラバーサル=L8）',
        /\bL6\b/.test(byTarget(`${U}nope.html`).detail)
        && /\bL7\b/.test(byTarget(`${U}lua/modules/index.html#no-such`).detail)
        && /\bL8\b/.test(byTarget(`${U}../../outside.html`).detail),
        JSON.stringify(broken));
      check('G-2 トラバーサルは実在してもトラバーサルとして報告',
        /トラバーサル/.test(byTarget(`${U}../../outside.html`).detail), JSON.stringify(broken));
      check('G-2 file はリポジトリ相対の README パス',
        byTarget(`${U}nope.html`).file === 'crates/pasta_lua/README.md'
        && byTarget(`${U}debug/index.html#無い見出し`).file === 'crates/pasta_core/README.md',
        JSON.stringify(broken));
      check('G-2 インラインコード内・フェンス内・他サイト・README 以外は対象外',
        !broken.some((b) => /code-nope|fenced-nope|example\.com|book-nope|nested-nope|other-nope/.test(b.target)),
        JSON.stringify(broken));

      const result = runLinkCheck(root);
      check('G-2 runLinkCheck に結合され failed=true',
        result.failed === true && result.broken.filter((b) => b.kind === 'readme-manual-url').length === 5,
        JSON.stringify(result.broken));
      const rep = reportLinkCheck(result);
      check('G-2 レポートの [1] 区分に readme-manual-url 5 件が出る',
        /\[1\][^\n]*: 5 件/.test(rep) && rep.includes('[readme-manual-url]'), rep);
    } finally {
      rmrf(outer);
    }
  }

  // G-3: 実在章・実在アンカー（パーセントエンコード）だけなら合格。
  {
    const root = makeSandbox();
    try {
      writeFile(root, 'book/src/introduction.md', '# はじめに\n');
      writeFile(root, 'book/src/lua/index.md', '# Lua\n\n## 起動の流れ\n');
      writeFile(root, 'crates/pasta_shiori/README.md',
        `# shiori\n\n${U}\n\n[起動](${U}lua/index.html#${encodeURIComponent('起動の流れ')})\n`);
      const broken = checkReadmeManualLinks(root);
      check('G-3 実在章・実在アンカーのみ: 違反 0', broken.length === 0, JSON.stringify(broken));
      check('G-3 runLinkCheck も failed=false', runLinkCheck(root).failed === false);
    } finally {
      rmrf(root);
    }
  }

  // G-3b: 全角の括弧で囲んだ裸の URL は、括弧の手前で終える（括弧を URL に含めない）。
  {
    const root = makeSandbox();
    try {
      writeFile(root, 'book/src/introduction.md', '# はじめに\n');
      writeFile(root, 'book/src/lua/index.md', '# Lua\n\n## 起動の流れ\n');
      const enc = encodeURIComponent('起動の流れ');
      writeFile(root, 'crates/pasta_shiori/README.md', [
        '# shiori', // L1
        `マニュアル（公開: ${U}）を参照。`, // L2
        `<!-- このファイルは pasta マニュアル「Lua」（${U}lua/index.html）から自動生成されたものです。手で編集しないでください。 -->`, // L3
        `この節は（${U}lua/index.html#起動の流れ）から自動生成されたものです`, // L4
        `「${U}lua/index.html」『${U}lua/index.html#${enc}』【${U}index.html】`, // L5
        '',
      ].join('\n'));
      const broken = checkReadmeManualLinks(root);
      check('G-3b 全角の括弧で囲んだ裸の URL（（公開: URL）・（URL）から自動生成…・「」『』【】）: 違反 0',
        broken.length === 0, JSON.stringify(broken));
      // 切れた URL は、括弧を含まない URL のまま報告する（括弧の手前で終えている）。
      writeFile(root, 'crates/pasta_core/README.md',
        `# core\n\n（${U}lua/index.html#no-such）と「${U}nope.html」\n`);
      const targets = checkReadmeManualLinks(root).map((b) => b.target).sort();
      check('G-3b 全角の括弧で囲んだ切れた URL は括弧を含めずに報告',
        JSON.stringify(targets) === JSON.stringify([`${U}lua/index.html#no-such`, `${U}nope.html`]),
        JSON.stringify(targets));
    } finally {
      rmrf(root);
    }
  }

  // G-4: 実リポジトリで readme-manual-url 0 件。
  {
    const broken = checkReadmeManualLinks(REPO_ROOT);
    check('G-4 実リポジトリで readme-manual-url 0 件', broken.length === 0, JSON.stringify(broken));
  }

  // G-5: manual.yml は push・pull_request の両方で crates/*/README.md の変更でも起動する（7.8）。
  {
    const yml = fs.readFileSync(path.join(REPO_ROOT, '.github/workflows/manual.yml'), 'utf8');
    const n = (yml.match(/^\s*-\s*'crates\/\*\/README\.md'\s*$/gm) || []).length;
    check('G-5 manual.yml の paths（push・pull_request）に crates/*/README.md', n === 2, `count=${n}`);
  }

  // G-6: 広げた対象（ルートの README・SOUL.md・2 スキルの Markdown）の公開 URL
  // （manual-link-anchor-check タスク 3.2 / 要件 4.4〜4.6, 5.5, 7.1）。
  {
    const root = makeSandbox();
    try {
      const SK = '.claude/skills';
      writeFile(root, 'book/src/lua/index.md', '# Lua\n\n## 起動の流れ\n');
      // どの対象にも同じ本文を置き、行番号と文面をクレートの README のときと比べる。
      const body = [
        '# doc', // L1
        `欠落章 [x](${U}nope.html)。`, // L2
        `裸の欠落見出し ${U}lua/index.html#no-such。`, // L3
        `在る章と見出し [ok](${U}lua/index.html#起動の流れ)。`, // L4
        `マニュアル（公開: ${U}）を参照。`, // L5
        `<!-- このファイルは pasta マニュアル「Lua」（${U}lua/index.html）から自動生成されたものです。手で編集しないでください。 -->`, // L6
        '',
      ].join('\n');
      const CRATE = 'crates/pasta_core/README.md';
      // 対象の順（固定）: クレートの README（名前順）→ ルートの README → SOUL.md → 2 スキル（CHECKED_SKILLS の順・パス順）。
      const order = [
        CRATE,
        'crates/pasta_lua/README.md',
        'README.md',
        'SOUL.md',
        `${SK}/pasta-ghost-authoring/SKILL.md`,
        `${SK}/pasta-ghost-authoring/references/dsl.md`,
        `${SK}/pasta-lua-coding/SKILL.md`,
        `${SK}/pasta-lua-coding/references/api.md`,
      ];
      for (const f of [...order].reverse()) writeFile(root, f, body);
      // 対象外: 内部の作業メモ・2 スキルのほかのスキル。
      writeFile(root, '.kiro/steering/roadmap.md', `[x](${U}roadmap-nope.html)\n`);
      writeFile(root, `${SK}/other-skill/SKILL.md`, `[x](${U}other-skill-nope.html)\n`);

      const broken = checkReadmeManualLinks(root);
      const of = (f) => JSON.stringify(broken.filter((b) => b.file === f)
        .map(({ target, kind, detail }) => ({ target, kind, detail })));
      const want = JSON.stringify([
        { target: `${U}nope.html`, kind: 'readme-manual-url', detail: 'L2: 写像先の章が存在しない: book/src/nope.md' },
        { target: `${U}lua/index.html#no-such`, kind: 'readme-manual-url',
          detail: 'L3: 写像先 book/src/lua/index.md に見出し／明示アンカー "no-such" が無い' },
      ]);
      check('G-6 クレートの README: 無い章（L2）・無い見出し（L3）の 2 件', of(CRATE) === want, of(CRATE));
      for (const f of order.slice(2)) {
        check(`G-6 ${f}: 無い章・無い見出しが readme-manual-url で行番号つき、文面はクレートの README と同じ`,
          of(f) === want && of(f) === of(CRATE), of(f));
      }
      check('G-6 違反の並びは対象の順・行の順（2 回走らせて同じ）',
        JSON.stringify(broken.map((b) => b.file)) === JSON.stringify(order.flatMap((f) => [f, f]))
        && JSON.stringify(checkReadmeManualLinks(root)) === JSON.stringify(broken),
        JSON.stringify(broken.map((b) => b.file)));
      check('G-6 全角の括弧で囲んだ裸の URL（（公開: URL）・（URL）から自動生成…）は違反にならない',
        broken.length === order.length * 2 && !broken.some((b) => /[（）]/.test(b.target)),
        JSON.stringify(broken.map((b) => b.target)));
      check('G-6 .kiro/steering/roadmap.md・2 スキルのほかのスキルは対象外',
        !broken.some((b) => /roadmap-nope|other-skill-nope/.test(b.target)), JSON.stringify(broken));

      const result = runLinkCheck(root);
      const rep = reportLinkCheck(result);
      check('G-6 runLinkCheck に結合され、レポートの [1] 区分に 16 件が出る',
        result.broken.filter((b) => b.kind === 'readme-manual-url').length === 16
        && /\[1\][^\n]*: 16 件/.test(rep) && rep.includes('BROKEN  SOUL.md'), rep);
    } finally {
      rmrf(root);
    }
  }

  // G-6: crates/ が無くても、ルートの README だけで検査する（無い対象は飛ばす）。
  {
    const root = makeSandbox();
    try {
      writeFile(root, 'README.md', `# root\n\n[x](${U}nope.html)\n`);
      const broken = checkReadmeManualLinks(root);
      check('G-6 crates/ が無くてもルートの README を検査する',
        broken.length === 1 && broken[0].file === 'README.md' && broken[0].kind === 'readme-manual-url'
        && broken[0].detail === 'L3: 写像先の章が存在しない: book/src/nope.md',
        JSON.stringify(broken));
    } finally {
      rmrf(root);
    }
  }
}

// ============================================================
log('\n== (H) 本の中のリンクの見出し（internal-anchor） ==');
{
  const SRC = 'book/src/grammar/source.md';
  const DST = 'book/src/grammar/target.md';
  const noAnchor = (chapter, heading) => `リンク先 ${chapter} に見出し／明示アンカー "${heading}" が無い`;

  // H-1: 在る見出しを指すリンクだけなら違反 0（誤検出なし）。
  {
    const root = makeSandbox();
    try {
      writeFile(root, DST, [
        '# 対象',
        '## 単語定義',
        '## [package] 予約注記',
        '### `ACT:talk(text)` の使い方',
        '## 例',
        '## 例',
        '<a id="s6-6"></a>',
        '### 6.6 永続化',
        '',
      ].join('\n'));
      writeFile(root, SRC, [
        '# 元',
        '## 節 A',
        '[日本語](target.md#単語定義) [記号](target.md#package-予約注記)',
        '[インラインコード](target.md#acttalktext-の使い方) [重複の付番](target.md#例-1)',
        `[パーセントエンコード](target.md#${encodeURIComponent('単語定義')}) [明示のアンカー](target.md#s6-6)`,
        '[空のフラグメント](target.md#) [クエリつき](target.md?x=1#例) [同じ章の中](#節-a)',
        '',
        '> 【クローディア：したり顔】[他の章](target.md#単語定義) も [同じ章](#節-a) も在りますわ。',
        '',
      ].join('\n'));
      const result = runLinkCheck(root);
      check('H-1 在る見出し（日本語・記号・インラインコード・重複の付番・パーセントエンコード・明示のアンカー・'
        + '空のフラグメント・クエリつき・同じ章の中・台詞部品の中）: 違反 0',
        result.broken.length === 0, JSON.stringify(result.broken));
      check('H-1 failed=false', result.failed === false);
    } finally {
      rmrf(root);
    }
  }

  // H-2: 無い見出しの検出。違反はリンクの出現順に並ぶ。
  {
    const root = makeSandbox();
    try {
      const target = ['# 対象', '## Overview', '```markdown', '## フェンスの中だけ', '```', ''];
      writeFile(root, DST, target.join('\n'));
      const enc = encodeURIComponent('無い');
      writeFile(root, SRC, [
        '# 元',
        '[他の章](target.md#無い見出し) [同じ章](#無い節)',
        '[大文字と小文字](target.md#Overview) [フェンスの中](target.md#フェンスの中だけ)',
        '',
        '> 【アンソニー：刮目】詳しくは [台詞](target.md#台詞の無い見出し) を。',
        '',
        `[無いファイル](missing.md#sec) [エンコード](target.md#${enc}) [復号できない](target.md#%E3%8)`,
        '[在る見出し](target.md#overview) [在る同じ章](#元)',
        '',
      ].join('\n'));

      const result = runLinkCheck(root);
      const broken = result.broken;
      const anchorOf = (t) => broken.filter((b) => b.kind === 'internal-anchor' && b.target === t);
      const dump = JSON.stringify(broken);
      check('H-2 他の章の無い見出し → internal-anchor', anchorOf('target.md#無い見出し').length === 1, dump);
      check('H-2 同じ章の無い見出し → internal-anchor', anchorOf('#無い節').length === 1, dump);
      check('H-2 大文字と小文字の違い（## Overview に #Overview）→ internal-anchor',
        anchorOf('target.md#Overview').length === 1, dump);
      check('H-2 フェンスの中にしか無い見出し → internal-anchor',
        anchorOf('target.md#フェンスの中だけ').length === 1, dump);
      check('H-2 台詞部品の中の無い見出し → internal-anchor',
        anchorOf('target.md#台詞の無い見出し').length === 1, dump);
      check('H-2 復号できないフラグメントは例外にせず internal-anchor（見出しは書かれたとおり）',
        anchorOf('target.md#%E3%8').length === 1
        && anchorOf('target.md#%E3%8')[0].detail === noAnchor(DST, '%E3%8'), dump);

      const missing = broken.filter((b) => b.target === 'missing.md#sec');
      check('H-2 無いファイルにフラグメントが付いたリンクは internal-md の 1 件だけ（文面は今のまま）',
        missing.length === 1 && missing[0].kind === 'internal-md'
        && missing[0].detail === 'book 内リンク先が存在しない: missing.md', JSON.stringify(missing));

      const encoded = anchorOf(`target.md#${enc}`)[0] || {};
      check('H-2 違反の欄: file はリンク元・target は書かれたとおり（復号しない）・detail は章のパスと復号した見出し',
        encoded.file === SRC && encoded.target === `target.md#${enc}` && encoded.detail === noAnchor(DST, '無い'),
        JSON.stringify(encoded));
      check('H-2 同じ章の中のリンクは、detail の章がリンク元と同じパス',
        (anchorOf('#無い節')[0] || {}).detail === noAnchor(SRC, '無い節'), dump);

      const want = [
        ['internal-anchor', 'target.md#無い見出し'],
        ['internal-anchor', '#無い節'],
        ['internal-anchor', 'target.md#Overview'],
        ['internal-anchor', 'target.md#フェンスの中だけ'],
        ['internal-anchor', 'target.md#台詞の無い見出し'],
        ['internal-md', 'missing.md#sec'],
        ['internal-anchor', `target.md#${enc}`],
        ['internal-anchor', 'target.md#%E3%8'],
      ];
      check('H-2 違反は 8 件・リンクの出現順（在る見出しは出ない）',
        JSON.stringify(broken.map((b) => [b.kind, b.target])) === JSON.stringify(want), dump);
      check('H-2 2 回走らせて同じ結果', JSON.stringify(runLinkCheck(root).broken) === dump);

      // 報告: 見出しの違反は [1]（リンク切れ）に数えられて行が出る。[2] には回らない。
      const rep = reportLinkCheck(result);
      check('H-2 レポートの [1] 区分に 8 件（internal-anchor 7 行・internal-md 1 行）が出て、[2] は 0 件',
        /\[1\][^\n]*: 8 件/.test(rep) && /\[2\][^\n]*: 0 件/.test(rep)
        && rep.split('[internal-anchor]').length - 1 === 7 && rep.split('[internal-md]').length - 1 === 1, rep);
      check('H-2 failed=true（終了コード 1）で、レポートの結びも FAIL',
        result.failed === true && rep.includes('RESULT: FAIL'), rep);

      // 見本の章に見出しを足すと、その 1 件だけが消える（前の走りの集合を持ち越さない）。
      writeFile(root, DST, [...target, '## 無い見出し', ''].join('\n'));
      const after = runLinkCheck(root).broken;
      check('H-2 見本を書き換えた後の走りは、今の章と照合する',
        after.length === 7 && !after.some((b) => b.target === 'target.md#無い見出し'), JSON.stringify(after));
    } finally {
      rmrf(root);
    }
  }

  // H-3: 実リポジトリで internal-anchor 0 件。
  {
    const broken = runLinkCheck(REPO_ROOT).broken.filter((b) => b.kind === 'internal-anchor');
    check('H-3 実リポジトリで internal-anchor 0 件', broken.length === 0, JSON.stringify(broken));
  }
}

// ============================================================
// 見出しの id の突き合わせ（置き場所は自己テストの中だけ。link-check.mjs は book/book/ を読まない）。
// 章（book/src/**/*.md から SUMMARY.md を除いたもの・パス順）ごとに、headingSlugs が導く id の並びと、
// ビルドした出力（book/book/ の下の同じ相対パスの .html）の <h1>〜<h6> の開始タグの id の並びを、順番まで比べる。
// 章から写像して読むので、print.html・classic/・リダイレクトのページ・先頭章の写し（index.html）は読まない。
// id の無い見出しのタグ（枠の本の題）は、出力の並びに入れない。
// book/book が無ければ null。出力が無い章・出力の更新時刻が章より古い章は、飛ばして名前を残す。
// 食い違いの index は、その章の何番目の見出しか（1 始まり）。並びの長さが違えば、足りない側は undefined。
// 読むだけで、何も書かない。環境変数も見ない（CI の中での判定は I-2 の側に置く）。
const HEADING_ID_RE = /<h[1-6]\b[^>]*?\sid="([^"]*)"/g;
function compareHeadingIds(repoRoot) {
  const srcDir = path.join(repoRoot, 'book', 'src');
  const outDir = path.join(repoRoot, 'book', 'book');
  if (!fs.existsSync(outDir)) return null;
  const report = { checked: 0, skipped: [], mismatches: [] };
  const rels = listMarkdownFiles(srcDir)
    .map((f) => path.relative(srcDir, f).split(path.sep).join('/'))
    .filter((rel) => rel !== 'SUMMARY.md')
    .sort();
  for (const rel of rels) {
    const chapter = `book/src/${rel}`;
    const src = path.join(srcDir, rel);
    const out = path.join(outDir, rel.replace(/\.md$/, '.html'));
    if (!fs.existsSync(out) || fs.statSync(out).mtimeMs < fs.statSync(src).mtimeMs) {
      report.skipped.push(chapter);
      continue;
    }
    report.checked++;
    const expected = headingSlugs(fs.readFileSync(src, 'utf8'));
    const actual = [...fs.readFileSync(out, 'utf8').matchAll(HEADING_ID_RE)].map((m) => m[1]);
    for (let i = 0; i < Math.max(expected.length, actual.length); i++) {
      if (expected[i] !== actual[i]) {
        report.mismatches.push({ chapter, index: i + 1, expected: expected[i], actual: actual[i] });
      }
    }
  }
  return report;
}

// I-2 の判定。突き合わせの結果（compareHeadingIds の戻り値）と、CI の中かどうかから、合否・表題・続けて出す行を決める。
// ok が null のときは、確かめの数に入れない（CI の外で、ビルドした出力が無い）。
// CI の中では、出力が在り、飛ばした章が 0 であることまで求める（出力が無いときの N は 0。飛ばした章は無く、理由は次の行に出す）。
// 食い違いは、CI の中でも外でも失敗にする。環境変数は見ない（見本でも同じ判定を通すため、呼ぶ側が渡す）。
function judgeHeadingIds(report, inCI) {
  if (!report) {
    return inCI
      ? { ok: false, name: 'I-2 CI では全章の突き合わせが要る（出力が無い、または飛ばし 0 章）',
        lines: ['        book/book が無い（ビルドした出力が無い）'] }
      : { ok: null, name: 'I-2 ビルドした出力が無いので飛ばす', lines: [] };
  }
  const { checked, skipped, mismatches } = report;
  const id = (v) => v ?? '（無し）';
  const lines = [
    ...mismatches.map((m) => `        DIFF  ${m.chapter} の ${m.index} 番目の見出し: 導いた id ${id(m.expected)} / 出力の id ${id(m.actual)}`),
    ...skipped.map((c) => `        SKIP  ${c}（出力が無いか、章より古い）`),
  ];
  if (inCI && skipped.length > 0) {
    return { ok: false, name: `I-2 CI では全章の突き合わせが要る（出力が無い、または飛ばし ${skipped.length} 章）`, lines };
  }
  return {
    ok: mismatches.length === 0,
    name: `I-2 実リポジトリ: 食い違い ${mismatches.length}（突き合わせ ${checked} 章・飛ばし ${skipped.length} 章）`,
    lines,
  };
}

log('\n== (I) 見出しの id の突き合わせ（ビルドした出力が在るときだけ） ==');
{
  // 見本の更新時刻は utimesSync で決める（書いた順や時計の粒度に頼らない）。T は章の時刻（秒）。
  const T = 1700000000;
  const put = (root, rel, content, t) => fs.utimesSync(writeFile(root, rel, content), t, t);
  // mdBook の出力に似せたページ。本文の見出しは id つき。枠の <header> と、id の無い見出し（本の題）は並びに入らない。
  const h = (n, id) => `<h${n} id="${id}"><a class="header" href="#${id}">${id}</a></h${n}>\n`;
  const page = (body) => '<!DOCTYPE HTML>\n<html><head><title>t</title></head><body>\n'
    + `<header id="mdbook-menu-bar"><h1 class="menu-title">本</h1></header>\n<main>\n${body}</main>\n</body></html>\n`;

  // I-1: id が合う出力。目次・印刷用ページ・classic/・先頭章の写し・章に無いページは読まない。
  {
    const root = makeSandbox();
    try {
      put(root, 'book/src/SUMMARY.md', '# Summary\n\n- [はじめに](introduction.md)\n', T);
      put(root, 'book/src/introduction.md',
        '# はじめに\n\n## 例\n\n## 例\n\n### `ACT:talk(text)` の使い方\n\n#### 四\n\n##### 五\n\n###### 六\n', T);
      put(root, 'book/book/introduction.html', page(h(1, 'はじめに') + h(2, '例') + h(2, '例-1')
        + h(3, 'acttalktext-の使い方') + h(4, '四') + h(5, '五') + h(6, '六')), T + 10);
      put(root, 'book/src/grammar/markers.md', '# マーカー\n\n```markdown\n## フェンスの中\n```\n\n## 節\n', T);
      // 更新時刻が章と同じ出力は、古くない（突き合わせる）。
      put(root, 'book/book/grammar/markers.html', page(h(1, 'マーカー') + h(2, '節')), T);
      for (const stray of ['print.html', 'index.html', '404.html', 'classic/introduction.html']) {
        put(root, `book/book/${stray}`, page(h(1, '読まないページ')), T + 10);
      }

      const r = compareHeadingIds(root);
      check('I-1 id が合う出力（重複の付番・インラインコード・フェンスの中・h1〜h6）: 食い違い 0',
        r.mismatches.length === 0, JSON.stringify(r));
      check('I-1 突き合わせは 2 章・飛ばし 0（目次は章に数えない。印刷用ページ・classic/・先頭章の写しは読まない）',
        r.checked === 2 && r.skipped.length === 0, JSON.stringify(r));
    } finally {
      rmrf(root);
    }
  }

  // I-1: id が合わない出力。全角の空白は、headingSlug は落とし、mdBook は `-` にする。
  {
    const root = makeSandbox();
    try {
      put(root, 'book/src/grammar/space.md', '# 空白\n\n## 全角　空白\n', T);
      put(root, 'book/book/grammar/space.html', page(h(1, '空白') + h(2, '全角-空白')), T + 10);
      const r = compareHeadingIds(root);
      const want = [{ chapter: 'book/src/grammar/space.md', index: 2, expected: '全角空白', actual: '全角-空白' }];
      check('I-1 合わない出力（## 全角　空白 に id 全角-空白）: 章・何番目の見出しか・導いた id・出力の id つきで食い違い 1',
        r.checked === 1 && JSON.stringify(r.mismatches) === JSON.stringify(want), JSON.stringify(r));
    } finally {
      rmrf(root);
    }
  }

  // I-1: 見出しの数が違う出力。短いほうに合わせて切らず、足りない側を undefined にして食い違いに数える。
  {
    const root = makeSandbox();
    try {
      put(root, 'book/src/fewer.md', '# 章\n\n## 節\n', T);
      put(root, 'book/book/fewer.html', page(h(1, '章')), T + 10);
      put(root, 'book/src/more.md', '# 章\n', T);
      put(root, 'book/book/more.html', page(h(1, '章') + h(2, '余分')), T + 10);
      const r = compareHeadingIds(root);
      const [fewer, more] = r.mismatches;
      check('I-1 出力の見出しが少ない・多い: 章ごとに食い違い 1（足りない側は undefined）',
        r.checked === 2 && r.mismatches.length === 2
        && fewer.chapter === 'book/src/fewer.md' && fewer.index === 2
        && fewer.expected === '節' && fewer.actual === undefined
        && more.chapter === 'book/src/more.md' && more.index === 2
        && more.expected === undefined && more.actual === '余分', JSON.stringify(r));
    } finally {
      rmrf(root);
    }
  }

  // I-1: 章より古い出力・出力の無い章は、飛ばして名前を残す（id が合わなくても比べない）。
  {
    const root = makeSandbox();
    try {
      put(root, 'book/src/fresh.md', '# 新しい\n', T);
      put(root, 'book/book/fresh.html', page(h(1, '新しい')), T + 10);
      put(root, 'book/src/stale.md', '# 直した見出し\n', T);
      put(root, 'book/book/stale.html', page(h(1, '直す前の見出し')), T - 10);
      put(root, 'book/src/grammar/missing.md', '# 出力なし\n', T);
      const r = compareHeadingIds(root);
      check('I-1 章より古い出力・出力の無い章: 飛ばして名前を残す（パス順）。ほかの章は突き合わせる',
        r.checked === 1 && r.mismatches.length === 0
        && JSON.stringify(r.skipped) === JSON.stringify(['book/src/grammar/missing.md', 'book/src/stale.md']),
        JSON.stringify(r));
    } finally {
      rmrf(root);
    }
  }

  // I-1: 出力のフォルダが無ければ null。出力のフォルダの有無で、リンク検証の結果は変わらない（6.2）。
  {
    const root = makeSandbox();
    try {
      put(root, 'book/src/grammar/source.md', '# 元\n\n[無い見出し](#無い節)\n', T);
      check('I-1 book/book が無い: null', compareHeadingIds(root) === null, JSON.stringify(compareHeadingIds(root)));
      const before = runLinkCheck(root);

      fs.mkdirSync(path.join(root, 'book', 'book'));
      check('I-1 book/book が空: null でなく、全章を飛ばす',
        JSON.stringify(compareHeadingIds(root))
        === JSON.stringify({ checked: 0, skipped: ['book/src/grammar/source.md'], mismatches: [] }),
        JSON.stringify(compareHeadingIds(root)));

      // 出力の側に、id の合わないページと、切れたリンクを持つ Markdown を置く。リンク検証はどちらも読まない。
      put(root, 'book/book/grammar/source.html', page(h(1, '別の見出し')), T + 10);
      put(root, 'book/book/grammar/stray.md', '[x](nope.md)\n', T + 10);
      check('I-1 book/book の有無で runLinkCheck の結果が変わらない（違反は章の 1 件のまま）',
        before.broken.length === 1 && JSON.stringify(runLinkCheck(root)) === JSON.stringify(before),
        JSON.stringify(runLinkCheck(root)));
    } finally {
      rmrf(root);
    }
  }

  // I-2: 判定の枝（見本の結果 × CI の外・中）。環境変数 CI には頼らない。
  {
    const clean = { checked: 60, skipped: [], mismatches: [] };
    const stale = { checked: 59, skipped: ['book/src/getting-started/09-choice.md'], mismatches: [] };
    const diff = {
      checked: 60,
      skipped: [],
      mismatches: [
        { chapter: 'book/src/grammar/space.md', index: 2, expected: '全角空白', actual: '全角-空白' },
        { chapter: 'book/src/more.md', index: 3, expected: undefined, actual: '余分' },
      ],
    };
    const skipLine = '        SKIP  book/src/getting-started/09-choice.md（出力が無いか、章より古い）';
    const diffLines = [
      '        DIFF  book/src/grammar/space.md の 2 番目の見出し: 導いた id 全角空白 / 出力の id 全角-空白',
      '        DIFF  book/src/more.md の 3 番目の見出し: 導いた id （無し） / 出力の id 余分',
    ];
    const ciName = (n) => `I-2 CI では全章の突き合わせが要る（出力が無い、または飛ばし ${n} 章）`;
    const cases = [
      ['CI の外・出力が無い: 飛ばす（確かめの数に入れない）', null, false,
        { ok: null, name: 'I-2 ビルドした出力が無いので飛ばす', lines: [] }],
      ['CI の中・出力が無い: 失敗', null, true,
        { ok: false, name: ciName(0), lines: ['        book/book が無い（ビルドした出力が無い）'] }],
      ['CI の外・飛ばし 1 章: 通して、飛ばした章を出す', stale, false,
        { ok: true, name: 'I-2 実リポジトリ: 食い違い 0（突き合わせ 59 章・飛ばし 1 章）', lines: [skipLine] }],
      ['CI の中・飛ばし 1 章: 失敗', stale, true, { ok: false, name: ciName(1), lines: [skipLine] }],
      ['CI の外・食い違い 2: 失敗（章・何番目の見出しか・導いた id・出力の id）', diff, false,
        { ok: false, name: 'I-2 実リポジトリ: 食い違い 2（突き合わせ 60 章・飛ばし 0 章）', lines: diffLines }],
      ['CI の中・食い違い 2: 失敗', diff, true,
        { ok: false, name: 'I-2 実リポジトリ: 食い違い 2（突き合わせ 60 章・飛ばし 0 章）', lines: diffLines }],
      ['CI の外・全章が合う: 通す', clean, false,
        { ok: true, name: 'I-2 実リポジトリ: 食い違い 0（突き合わせ 60 章・飛ばし 0 章）', lines: [] }],
      ['CI の中・全章が合う: 通す', clean, true,
        { ok: true, name: 'I-2 実リポジトリ: 食い違い 0（突き合わせ 60 章・飛ばし 0 章）', lines: [] }],
    ];
    for (const [name, report, inCI, want] of cases) {
      const got = judgeHeadingIds(report, inCI);
      check(`I-2 判定 ${name}`, JSON.stringify(got) === JSON.stringify(want), JSON.stringify(got));
    }
    // CI の中で、飛ばしと食い違いが重なったら、CI の表題で失敗にし、食い違いの行も出す。
    const both = judgeHeadingIds({ ...diff, checked: 59, skipped: stale.skipped }, true);
    check('I-2 判定 CI の中・飛ばし 1 章と食い違い 2: CI の表題で失敗し、食い違いと飛ばした章の行を出す',
      both.ok === false && both.name === ciName(1)
      && JSON.stringify(both.lines) === JSON.stringify([...diffLines, skipLine]), JSON.stringify(both));
  }

  // I-2: 実リポジトリ。CI の中かどうか（環境変数 CI）を見るのは、ここだけ。
  {
    const { ok, name, lines } = judgeHeadingIds(compareHeadingIds(REPO_ROOT), Boolean(process.env.CI));
    if (ok === null) log(`  SKIP  ${name}`);
    else check(name, ok);
    for (const line of lines) log(line);
  }
}

// ============================================================
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
