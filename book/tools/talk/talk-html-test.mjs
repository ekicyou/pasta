// talk-html-test.mjs — 出力 HTML の台詞を吹き出しに変換するツールの自己テスト
// （manual-claudia-theme タスク 3.1 / 要件 3.1, 3.3, 3.6, 3.9, 3.10, 3.11, 8.4, 8.6, 10.7 / design「TalkHtmlTransform」）。
//
// 観測する完了条件（design「Testing Strategy / Unit Tests」）:
//   - 出力の構造が「出力 HTML 契約」どおり（クラス・alt・名札・本文の強調・コード・リンクの保持）。
//   - 深さ 0・1・2 のページと print.html で、顔画像の相対パスが正しい。
//   - 2 回実行しても結果が変わらない。
//   - 通常の引用ブロック・mdBook の注記（blockquote-tag）はバイト不変。
//   - 未知のタグ・入れ子の台詞で失敗し、そのときどのファイルにも書き込まない。
//   - 属性の値がエスケープされる。
//   フィクスチャの HTML は mdBook 0.5.4 の実際の出力の形（スクラッチのビルドで確認した形）を写している。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { renderTalk, transformHtml, transformDir } from './talk-html.mjs';
import { SPEAKERS } from './talk.mjs';

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
const show = (s) => JSON.stringify(s);

// mdBook 0.5.4 が `> 【…】…` から出す形
const TALK_SIMPLE = '<blockquote>\n<p>【高笑い】さあ、<strong>熱く</strong>参りましょう！</p>\n</blockquote>';
const TALK_RICH = '<blockquote>\n<p>【アンソニー】執事の<code>code</code>と<a href="grammar/markers.html">リンク</a>でございます。\n継続行です。</p>\n<p>二段落目です。</p>\n</blockquote>';
const PLAIN_QUOTE = '<blockquote>\n<p>将来変更あり</p>\n</blockquote>';
const ADMONITION = '<blockquote class="blockquote-tag blockquote-tag-note">\n<p class="blockquote-tag-title"><svg viewbox="0 0 16 16" width="18" height="18"><path d="M0 8"></path></svg>Note</p>\n<p>注記の本文</p>\n</blockquote>';
const page = (main) => `<!DOCTYPE html>\n<html>\n<body>\n<main>\n<h1 id="t"><a class="header" href="#t">T</a></h1>\n${main}\n\n</main>\n</body>\n</html>\n`;

// ============================================================
log('\n== (S) 出力 HTML 契約の構造 ==');
{
  const r = transformHtml(TALK_SIMPLE, '../');
  const want = '<div class="talk talk-claudia talk-left">\n'
    + '<img class="talk-face" src="../img/claudia/f5.png" alt="クローディア（高笑い）" width="56" height="56">\n'
    + '<div class="talk-bubble">\n'
    + '<span class="talk-name" aria-hidden="true">Claudia</span>\n'
    + '<p>さあ、<strong>熱く</strong>参りましょう！</p>\n'
    + '</div>\n'
    + '</div>';
  check('S-1 契約の例とバイト一致（既定の話し手・強調の保持）', r.html === want, show(r.html));
  check('S-2 変換数 1・エラーなし', r.converted === 1 && r.errors.length === 0, show(r));
  check('S-3 loading="lazy" を付けない', !/loading=/.test(r.html));

  const a = transformHtml(TALK_RICH, '');
  check('S-4 アンソニーは talk-anthony talk-right・素 f10', a.html.startsWith('<div class="talk talk-anthony talk-right">\n<img class="talk-face" src="img/claudia/f10.png" alt="アンソニー（素）" width="56" height="56">\n'), show(a.html));
  check('S-5 名札は Anthony・aria-hidden', a.html.includes('<span class="talk-name" aria-hidden="true">Anthony</span>\n'));
  check('S-6 インラインコード・リンク・継続行・2 段落目をそのまま移す',
    a.html.includes('<p>執事の<code>code</code>と<a href="grammar/markers.html">リンク</a>でございます。\n継続行です。</p>\n<p>二段落目です。</p>\n</div>\n</div>'), show(a.html));
  check('S-7 タグの文字列は本文に残らない', !a.html.includes('【'));

  const k = transformHtml('<blockquote>\n<p>【アンソニー：刮目】おや。</p>\n</blockquote>', '');
  check('S-8 話し手：表情（刮目 f11）', k.html.includes('src="img/claudia/f11.png" alt="アンソニー（刮目）"'), show(k.html));

  // タグだけの最初の段落（`> 【驚き】` の後に空の `>` を置いた形）は空の <p> を残さない
  const e = transformHtml('<blockquote>\n<p>【驚き】</p>\n<p>本文です。</p>\n</blockquote>', '');
  check('S-9 タグだけの段落は空の <p> を残さない', e.html.includes('aria-hidden="true">Claudia</span>\n<p>本文です。</p>\n</div>') && !e.html.includes('<p></p>'), show(e.html));
  // タグの直後で改行した形（`> 【驚き】\n> 本文`）
  const n = transformHtml('<blockquote>\n<p>【驚き】\n本文です。</p>\n</blockquote>', '');
  check('S-10 タグ直後の改行は落として本文を段落に残す', n.html.includes('</span>\n<p>本文です。</p>\n</div>'), show(n.html));
  // 全 16 表情が変換できる
  let all = true;
  for (const sp of SPEAKERS) {
    for (const [face, file] of Object.entries(sp.faces)) {
      const t = transformHtml(`<blockquote>\n<p>【${sp.name}：${face}】x</p>\n</blockquote>`, '');
      if (!t.html.includes(`src="img/claudia/${file}.png" alt="${sp.name}（${face}）"`) || !t.html.includes(`talk-${sp.id} talk-${sp.side}`)) all = false;
    }
  }
  check('S-11 全 16 表情を変換できる', all);
}

// ============================================================
log('\n== (U) 対象外はバイト不変 ==');
{
  const src = page(`${PLAIN_QUOTE}\n${ADMONITION}\n<p>地の文の【括弧】</p>\n<pre><code>&gt; 【高笑い】コードの中</code></pre>`);
  const r = transformHtml(src, '');
  check('U-1 通常の引用・注記・地の文・コードは 1 バイトも変えない', r.html === src && r.converted === 0 && r.errors.length === 0);
  const mixed = page(`${PLAIN_QUOTE}\n${TALK_SIMPLE}\n${ADMONITION}`);
  const m = transformHtml(mixed, '');
  const [pre, post] = mixed.split(TALK_SIMPLE);
  check('U-2 台詞の前後はバイト不変', m.html.startsWith(pre) && m.html.endsWith(post) && m.converted === 1);
}

// ============================================================
log('\n== (I) 冪等 ==');
{
  const src = page(`${TALK_SIMPLE}\n${PLAIN_QUOTE}\n${TALK_RICH}`);
  const once = transformHtml(src, '../');
  const twice = transformHtml(once.html, '../');
  check('I-1 1 回目は 2 件変換', once.converted === 2);
  check('I-2 2 回目は 0 件・結果不変', twice.converted === 0 && twice.html === once.html && twice.errors.length === 0);
}

// ============================================================
log('\n== (E) 不正（未知のタグ・入れ子） ==');
{
  const u = transformHtml('<blockquote>\n<p>【大笑い】x</p>\n</blockquote>', '');
  check('E-1 未知の表情は unknown-expression・タグを示す', u.errors.length === 1 && u.errors[0].kind === 'unknown-expression' && u.errors[0].tag === '大笑い', show(u.errors));
  const s = transformHtml('<blockquote>\n<p>【セバスチャン：素】x</p>\n</blockquote>', '');
  check('E-2 未知の話し手は unknown-speaker', s.errors[0]?.kind === 'unknown-speaker', show(s.errors));
  const b = transformHtml('<blockquote>\n<p>【高笑い さあ</p>\n</blockquote>', '');
  check('E-3 】の欠落は malformed-tag', b.errors[0]?.kind === 'malformed-tag', show(b.errors));
  const nestIn = transformHtml('<blockquote>\n<blockquote>\n<p>【驚き】入れ子</p>\n</blockquote>\n</blockquote>', '');
  check('E-4 引用の中の台詞は nested-talk', nestIn.errors.length === 1 && nestIn.errors[0].kind === 'nested-talk' && nestIn.errors[0].tag === '驚き', show(nestIn.errors));
  const nestOut = transformHtml('<blockquote>\n<p>【驚き】外</p>\n<blockquote>\n<p>中の引用</p>\n</blockquote>\n</blockquote>', '');
  check('E-5 入れ子の引用を含む台詞は nested-talk', nestOut.errors.length === 1 && nestOut.errors[0].kind === 'nested-talk', show(nestOut.errors));
  const both = transformHtml('<blockquote>\n<p>【驚き】外</p>\n<blockquote>\n<p>【照れ】中</p>\n</blockquote>\n</blockquote>', '');
  check('E-6 台詞の中の台詞は 1 件として報告', both.errors.length === 1 && both.errors[0].kind === 'nested-talk', show(both.errors));
  const li = transformHtml('<ul>\n<li>\n<blockquote>\n<p>【驚き】リスト内</p>\n</blockquote>\n</li>\n</ul>', '');
  check('E-7 リストの中の台詞は nested-talk', li.errors.length === 1 && li.errors[0].kind === 'nested-talk', show(li.errors));
  const two = transformHtml(`<blockquote>\n<p>【大笑い】x</p>\n</blockquote>\n${TALK_SIMPLE}\n<blockquote>\n<p>【泣き】y</p>\n</blockquote>`, '');
  check('E-8 不正は最初の 1 件で止めず全件返す', two.errors.length === 2 && two.errors.map((x) => x.tag).join() === '大笑い,泣き', show(two.errors));
}

// ============================================================
log('\n== (X) 属性値のエスケープ ==');
{
  const evil = { id: 'x"y', name: 'A<B', latin: 'L&"<>\'', side: 'left', faces: { '"q\'': 'f"0' } };
  const html = renderTalk({ speaker: evil, face: '"q\'', file: 'f"0' }, '<p>本文</p>\n', '../');
  check('X-1 class の値をエスケープ', html.startsWith('<div class="talk talk-x&quot;y talk-left">'), show(html));
  check('X-2 src・alt の値をエスケープ', html.includes('src="../img/claudia/f&quot;0.png" alt="A&lt;B（&quot;q&#39;）"'), show(html));
  check('X-3 名札の文字をエスケープ', html.includes('aria-hidden="true">L&amp;&quot;&lt;&gt;&#39;</span>'), show(html));
  check('X-4 本文はそのまま移す', html.includes('</span>\n<p>本文</p>\n</div>\n</div>'), show(html));
}

// ============================================================
log('\n== (D) 出力フォルダの変換（深さ 0・1・2・print.html・書き込みなしの失敗・CLI） ==');
const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'talk-html-test-'));
const write = (root, rel, text) => {
  const f = path.join(root, ...rel.split('/'));
  fs.mkdirSync(path.dirname(f), { recursive: true });
  fs.writeFileSync(f, text, 'utf8');
};
const read = (root, rel) => fs.readFileSync(path.join(root, ...rel.split('/')), 'utf8');
const snapshot = (root) => {
  const out = {};
  for (const rel of fs.readdirSync(root, { recursive: true }).map((p) => p.split(path.sep).join('/')).sort()) {
    const f = path.join(root, rel);
    if (fs.statSync(f).isFile()) out[rel] = fs.readFileSync(f, 'utf8');
  }
  return out;
};
try {
  const good = path.join(tmpRoot, 'good');
  write(good, 'introduction.html', page(TALK_SIMPLE));
  write(good, 'print.html', page(`${TALK_SIMPLE}\n${PLAIN_QUOTE}\n${TALK_RICH}`));
  write(good, 'grammar/markers.html', page(TALK_SIMPLE));
  write(good, 'grammar/deep/x.html', page(TALK_SIMPLE));
  write(good, 'plain.html', page(PLAIN_QUOTE));
  write(good, 'searchindex.js', TALK_SIMPLE); // HTML 以外は読まない
  const plainBefore = read(good, 'plain.html');

  const r = transformDir(good);
  check('D-1 HTML を名前順にすべて読む', r.files.join() === 'grammar/deep/x.html,grammar/markers.html,introduction.html,plain.html,print.html', show(r.files));
  check('D-2 変換数 5・変更ファイル 4', r.converted === 5 && r.filesChanged === 4, show(r));
  check('D-3 深さ 0（introduction.html）は img/claudia/', read(good, 'introduction.html').includes('src="img/claudia/f5.png"'));
  check('D-4 深さ 1（grammar/markers.html）は ../img/claudia/', read(good, 'grammar/markers.html').includes('src="../img/claudia/f5.png"'));
  check('D-5 深さ 2（grammar/deep/x.html）は ../../img/claudia/', read(good, 'grammar/deep/x.html').includes('src="../../img/claudia/f5.png"'));
  const pr = read(good, 'print.html');
  check('D-6 print.html も変換し img/claudia/', pr.includes('src="img/claudia/f5.png"') && pr.includes('src="img/claudia/f10.png"') && !pr.includes('<p>【'));
  check('D-7 台詞の無いファイルはバイト不変', read(good, 'plain.html') === plainBefore);
  check('D-8 HTML 以外は触らない', read(good, 'searchindex.js') === TALK_SIMPLE);
  const before2 = snapshot(good);
  const r2 = transformDir(good);
  check('D-9 2 回目は変換 0・全ファイル不変', r2.converted === 0 && r2.filesChanged === 0 && JSON.stringify(snapshot(good)) === JSON.stringify(before2));

  // 1 つでも不正があれば、正しいファイルも含めて何も書き込まない
  const bad = path.join(tmpRoot, 'bad');
  write(bad, 'a.html', page(TALK_SIMPLE));
  write(bad, 'sub/b.html', page('<blockquote>\n<p>【大笑い】x</p>\n</blockquote>'));
  write(bad, 'z.html', page('<blockquote>\n<blockquote>\n<p>【驚き】y</p>\n</blockquote>\n</blockquote>'));
  const badBefore = snapshot(bad);
  let thrown = null;
  try { transformDir(bad); } catch (e) { thrown = e; }
  check('D-10 不正があると例外', thrown !== null);
  check('D-11 例外は出力ファイルとタグを全件示す', thrown && /sub\/b\.html/.test(thrown.message) && /【大笑い】/.test(thrown.message)
    && /z\.html/.test(thrown.message) && /【驚き】/.test(thrown.message) && /nested-talk/.test(thrown.message), thrown && thrown.message);
  check('D-12 失敗時はどのファイルにも書き込まない', JSON.stringify(snapshot(bad)) === JSON.stringify(badBefore));

  let missing = null;
  try { transformDir(path.join(tmpRoot, 'nope')); } catch (e) { missing = e; }
  check('D-13 入力フォルダが無ければ例外', missing !== null);

  // CLI
  const cli = path.join(path.dirname(fileURLToPath(import.meta.url)), 'talk-html.mjs');
  const ok = spawnSync(process.execPath, [cli, good], { encoding: 'utf8' });
  check('D-14 CLI 成功は exit 0・読んだ数と変換数を出す', ok.status === 0 && /scanned 5 html file/.test(ok.stdout) && /converted 0 talk block/.test(ok.stdout), ok.stdout + ok.stderr);
  const ng = spawnSync(process.execPath, [cli, bad], { encoding: 'utf8' });
  check('D-15 CLI 失敗は exit 1・ファイルとタグを stderr に出す・書き込みなし',
    ng.status === 1 && /sub\/b\.html/.test(ng.stderr) && /【大笑い】/.test(ng.stderr) && JSON.stringify(snapshot(bad)) === JSON.stringify(badBefore), ng.stdout + ng.stderr);
} finally {
  fs.rmSync(tmpRoot, { recursive: true, force: true });
}

log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
