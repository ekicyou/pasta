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
//   (A)・(B-13) は book 部分（internal-md / github-repo-path / internals-path）が 0 件であることだけを確かめる。
//   - checkInternalsPaths が内部設計章のフェンス外インラインコードのリポジトリ内パスについて、
//     実在しない・`:行番号` 付き・トラバーサル・末尾 `/` なのにディレクトリでないものを
//     internals-path として行番号付きで検出し、フェンス内・接頭辞外・内部設計章以外は対象外とする
//     （pasta-runtime-internals-doc タスク 1.1 / 要件 3.2, 3.7, 1.6）。

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
  headingSlug,
  headingSlugs,
  checkSkillSelfContained,
  checkInternalsPaths,
  INTERNALS_DIR,
  REPO_PATH_PREFIXES,
  CHECKED_SKILLS,
  FORBIDDEN_SKILL_TOKENS,
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
const BOOK_KINDS = new Set(['internal-md', 'github-repo-path', 'internals-path']);
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
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
