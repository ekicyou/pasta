// theme-test.mjs — Claudia テーマ（book/theme/claudia.css）の配色トークンの対比テスト
// （manual-claudia-theme タスク 2.2 / 要件 1.1, 2.2, 2.4, 3.5, 8.3、design「ThemeTest」「テーマトークン契約」）。
//
// 観測する完了条件:
//   - claudia.css の最上位ブロック `.light, html:not(.js)` と `.navy` から `--name: #rrggbb;` を読む。
//   - 両ブロックに mdBook のテーマ変数 40 個と Claudia のトークンがすべてある（欠けたら失敗）。
//   - 設計の「テーマトークン契約」で決めた文字色と背景色の組ごとに、WCAG の対比が 4.5 以上。
//     対象のトークンが欠けている、または 16 進色（#rrggbb）でない場合も失敗。
//   - 部品の層（タスク 2.3 / 要件 1.2, 1.4, 1.8, 9.3、design「ClaudiaTheme」の意匠）:
//       * 色は直書きしない（カスタムプロパティの定義を除く宣言に 16 進色・rgb() 等・色名が無い）。
//       * 角丸は 14px・10px・4px の 3 段（円の 50% と 0・inherit は可）。影は --claudia-shadow-1/-2 だけ。
//       * 影のトークンが :root・light・navy にある。本文と見出しに字体トークンを当てている。
//       * サイドバーと上部バーの規則は色と字体だけを変える（幅・開閉・折りたたみ・切り替え点に触れない）。
//       * highlight.js の兄弟クラス（mdBook の highlight.css の 6 群）を、light・navy・JS 無効の
//         3 つの範囲で同じ群の --claudia-hl-* で塗る（実行時に着色する lua・toml・json 等の対比を守る）。
//   - Node 標準のみ・ビルド不要。問題は全件列挙してから exit 1。

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

export const CSS_PATH = fileURLToPath(new URL('../theme/claudia.css', import.meta.url));
export const MIN_CONTRAST = 4.5;

// テーマごとのブロックのセレクタ（design「ClaudiaTheme / State Management」の 2・3）
export const THEME_SELECTORS = {
  light: '.light, html:not(.js)',
  navy: '.navy',
};

// mdBook 0.5 の variables.css がテーマごとに持つ 40 変数（research.md RN6 の一覧。0.5.4 の出力で実測）
export const MDBOOK_VARS = [
  '--bg', '--fg', '--sidebar-bg', '--sidebar-fg', '--sidebar-non-existant', '--sidebar-active',
  '--sidebar-spacer', '--scrollbar', '--icons', '--icons-hover', '--links', '--inline-code-color',
  '--theme-popup-bg', '--theme-popup-border', '--theme-hover', '--quote-bg', '--quote-border',
  '--warning-border', '--table-border-color', '--table-header-bg', '--table-alternate-bg',
  '--searchbar-border-color', '--searchbar-bg', '--searchbar-fg', '--searchbar-shadow-color',
  '--searchresults-header-fg', '--searchresults-border-color', '--searchresults-li-bg',
  '--search-mark-bg', '--color-scheme', '--copy-button-filter', '--copy-button-filter-hover',
  '--footnote-highlight', '--overlay-bg', '--blockquote-note-color', '--blockquote-tip-color',
  '--blockquote-important-color', '--blockquote-warning-color', '--blockquote-caution-color',
  '--sidebar-header-border-color',
];

// pasta コードの着色 6 群（scope-map.mjs の hljs クラスの群ごとに 1 トークン）
export const HL_TOKENS = [
  '--claudia-hl-comment', '--claudia-hl-variable', '--claudia-hl-number',
  '--claudia-hl-string', '--claudia-hl-title', '--claudia-hl-keyword',
];

// 「テーマトークン契約」の Claudia のトークン
export const CLAUDIA_TOKENS = [
  '--claudia-paper', '--claudia-ink', '--claudia-ink2', '--claudia-paper2', '--claudia-edge',
  '--claudia-accent', '--claudia-gold', '--claudia-wax', '--claudia-code-bg', '--talk-bubble',
  '--talk-claudia-ink', '--talk-claudia-face', '--talk-claudia-ring', '--talk-claudia-name',
  '--talk-anthony-ink', '--talk-anthony-face', '--talk-anthony-ring', '--talk-anthony-name',
  ...HL_TOKENS,
];

// 対比 4.5 以上を検査する [文字色, 背景色] の組（「テーマトークン契約」の箇条）
export const CONTRAST_PAIRS = [
  ['--fg', '--bg'],
  ['--fg', '--claudia-paper2'],
  ['--sidebar-fg', '--sidebar-bg'],
  ['--sidebar-active', '--sidebar-bg'],
  ['--links', '--bg'],
  ['--links', '--claudia-paper2'],
  ['--claudia-ink2', '--bg'],
  ['--talk-claudia-ink', '--talk-bubble'],
  ['--talk-claudia-name', '--talk-bubble'],
  ['--talk-anthony-ink', '--talk-bubble'],
  ['--talk-anthony-name', '--talk-bubble'],
  ...HL_TOKENS.map((t) => [t, '--claudia-code-bg']),
  ['--fg', '--claudia-code-bg'],
  ['--inline-code-color', '--claudia-code-bg'],
];

// --- CSS の最上位ブロックを読む（@media 等の入れ子の中は対象外） ---
export function topLevelBlocks(css) {
  const src = css.replace(/\/\*[\s\S]*?\*\//g, '');
  const blocks = [];
  let depth = 0;
  let head = 0;
  let selector = '';
  let bodyStart = 0;
  for (let i = 0; i < src.length; i++) {
    const c = src[i];
    if (c === '{') {
      if (depth === 0) {
        selector = src.slice(head, i).trim().replace(/\s+/g, ' ');
        bodyStart = i + 1;
      }
      depth++;
    } else if (c === '}') {
      depth--;
      if (depth === 0) {
        blocks.push({ selector, body: src.slice(bodyStart, i) });
        head = i + 1;
      }
    }
  }
  return blocks;
}

// セレクタが一致する最上位ブロックの宣言を Map にする（同じセレクタが複数あれば後勝ち）
export function readTokens(css, selector) {
  const tokens = new Map();
  let found = false;
  for (const b of topLevelBlocks(css)) {
    if (b.selector !== selector) continue;
    found = true;
    for (const m of b.body.matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) tokens.set(m[1], m[2].trim());
  }
  return found ? tokens : null;
}

// --- WCAG 2.x の相対輝度と対比 ---
function luminance(hex) {
  const ch = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
    .map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * ch[0] + 0.7152 * ch[1] + 0.0722 * ch[2];
}
export function contrast(a, b) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}
const HEX_RE = /^#[0-9a-fA-F]{6}$/;

// CSS 全文を検査し、[{ theme, ok, name, detail }] を返す
export function checkTheme(css) {
  const results = [];
  const add = (theme, ok, name, detail) => results.push({ theme, ok, name, detail });
  for (const [theme, selector] of Object.entries(THEME_SELECTORS)) {
    const tokens = readTokens(css, selector);
    if (!tokens) {
      add(theme, false, `ブロック ${selector} がある`, 'claudia.css の最上位に見つからない');
      continue;
    }
    for (const name of [...MDBOOK_VARS, ...CLAUDIA_TOKENS]) {
      add(theme, tokens.has(name), `トークン ${name} がある`, '欠けている');
    }
    for (const [fg, bg] of CONTRAST_PAIRS) {
      const name = `対比 ${fg} / ${bg}`;
      const a = tokens.get(fg);
      const b = tokens.get(bg);
      const bad = [[fg, a], [bg, b]].filter(([, v]) => !HEX_RE.test(v ?? ''));
      if (bad.length) {
        add(theme, false, name, bad.map(([n, v]) => `${n} が 16 進色でない（${v ?? '欠落'}）`).join('、'));
        continue;
      }
      const r = contrast(a, b);
      add(theme, r >= MIN_CONTRAST, name, `${a} / ${b} = ${r.toFixed(2)}（${MIN_CONTRAST} 未満）`);
    }
  }
  return results;
}

// --- 部品の層の検査（タスク 2.3） ---

// 影のトークン（墨色の淡い 2 段）。:root（rust・coal・ayu 用の既定値）・light・navy の 3 か所に置く
export const SHADOW_TOKENS = ['--claudia-shadow-1', '--claudia-shadow-2'];

// 角丸の許容値（design「意匠」: 紙 14px・カード 10px・部品 4px。顔の円の 50%、角を立てる 0、継承）
export const ALLOWED_RADII = new Set(['14px', '10px', '4px', '50%', '0', 'inherit']);

// mdBook 0.5.x の highlight.css（light 用）の色の群。実行時に hljs 10.1.1 が付けるクラスのうち
// 色が付くものはすべてここに入る。群の名前は --claudia-hl-<群> に対応する。
export const HLJS_GROUPS = {
  comment: ['comment', 'quote'],
  variable: ['variable', 'template-variable', 'attribute', 'attr', 'tag', 'name', 'regexp', 'link',
    'selector-id', 'selector-class'],
  number: ['number', 'meta', 'built_in', 'builtin-name', 'literal', 'type', 'params'],
  string: ['string', 'symbol', 'bullet'],
  title: ['title', 'section'],
  keyword: ['keyword', 'selector-tag'],
};
// 着色を上書きする範囲（light・navy と、JS 無効時の既定）
export const HLJS_SCOPES = ['.light', '.navy', 'html:not(.js)'];

// サイドバーと上部バーの規則（色と字体だけ。要件 1.8・9.3）
// サイドバーの開閉で本文を押し出す仕組み（.page-wrapper・#mdbook-body-container）も同じ扱い
const CHROME_SELECTOR_RE = /\.sidebar|#mdbook-sidebar|\.chapter|#mdbook-menu-bar|\.menu-bar|\.menu-title|\.page-wrapper|#mdbook-page-wrapper|#mdbook-body-container/;
const CHROME_PROP_RE = /^(color|background-color|font-family|font-weight|font-style|letter-spacing|border(-block-end|-block-start|-inline-start|-inline-end)?-color)$/;

// 色の直書きの検出（カスタムプロパティ名を取り除いてから見る。--claudia-gold の gold を拾わない）
const COLOR_NAMES = ['white', 'black', 'red', 'green', 'blue', 'gray', 'grey', 'silver', 'maroon',
  'purple', 'fuchsia', 'lime', 'olive', 'yellow', 'navy', 'teal', 'aqua', 'orange', 'brown', 'pink',
  'gold', 'beige', 'ivory', 'tan', 'wheat', 'crimson', 'indigo', 'violet'];
const COLOR_LITERAL_RE = new RegExp(
  `#[0-9a-fA-F]{3,8}\\b|\\b(rgba?|hsla?|hwb|lab|lch|oklab|oklch|color)\\(|\\b(${COLOR_NAMES.join('|')})\\b`, 'i');

// @media 等の入れ子をたどり、葉の規則を { selector, decls: [[prop, value]], nested } で返す
export function leafRules(css) {
  const out = [];
  const walk = (src, nested) => {
    for (const b of topLevelBlocks(src)) {
      if (/^@(media|supports|layer)\b/.test(b.selector)) {
        walk(b.body, true);
        continue;
      }
      const decls = [];
      for (const part of b.body.split(';')) {
        const i = part.indexOf(':');
        if (i < 0) continue;
        decls.push([part.slice(0, i).trim(), part.slice(i + 1).trim()]);
      }
      out.push({ selector: b.selector, decls, nested });
    }
  };
  walk(css.replace(/\/\*[\s\S]*?\*\//g, ''), false);
  return out;
}

// CSS 全文の部品の層を検査し、[{ ok, name, detail }] を返す
export function checkComponents(css) {
  const results = [];
  const add = (ok, name, detail) => results.push({ ok, name, detail });
  const rules = leafRules(css);
  const each = function* (pred) {
    for (const r of rules) for (const [prop, value] of r.decls) if (pred(prop, value, r)) yield `${r.selector} { ${prop}: ${value} }`;
  };

  // 色の直書き（カスタムプロパティの定義は除く）
  const literal = [...each((p, v) => !p.startsWith('--') && COLOR_LITERAL_RE.test(v.replace(/--[\w-]+/g, '')))];
  add(literal.length === 0, 'C-1 部品の層に色の直書きが無い', literal.join(' / '));

  // 角丸の 3 段
  const radii = [...each((p, v) => /^border(-[a-z-]+)?-radius$/.test(p) && v.split(/[\s/]+/).some((x) => !ALLOWED_RADII.has(x)))];
  add(radii.length === 0, 'C-2 角丸は 14px・10px・4px の 3 段', radii.join(' / '));

  // 影は 2 段のトークンだけ
  const SHADOW_RE = /^var\(--claudia-shadow-[12]\)$/;
  const shadows = [...each((p, v) => p === 'box-shadow' && v !== 'none' && !SHADOW_RE.test(v))];
  add(shadows.length === 0, 'C-3 影は --claudia-shadow-1/-2 だけ', shadows.join(' / '));
  const shadowUsed = [...each((p, v) => p === 'box-shadow' && SHADOW_RE.test(v))].length > 0;
  add(shadowUsed, 'C-4 影のトークンを部品に当てている', '--claudia-shadow-1/-2 を参照する box-shadow が無い');

  // 影のトークンの定義（:root・light・navy）
  for (const [where, selector] of [[':root', ':root'], ['light', THEME_SELECTORS.light], ['navy', THEME_SELECTORS.navy]]) {
    const tokens = readTokens(css, selector) ?? new Map();
    for (const t of SHADOW_TOKENS) add(tokens.has(t), `C-5 ${where} にトークン ${t} がある`, '欠けている');
  }

  // 字体トークンの適用（本文ゴシック・見出し明朝。要件 1.3・1.4）。
  // 本文は html か body の規則に、見出しは h1〜h6 をすべて含む規則に当てていること
  const fontRule = (token, covers) => rules.some((r) => !r.nested
    && r.decls.some(([p, v]) => p === 'font-family' && v === `var(${token})`)
    && covers(r.selector.split(',').map((s) => s.trim())));
  add(fontRule('--claudia-font-body', (sels) => sels.includes('html') || sels.includes('body')),
    'C-6 本文の字体トークンを html か body に当てている', 'html・body の規則に font-family: var(--claudia-font-body) が無い');
  add(fontRule('--claudia-font-heading', (sels) => [1, 2, 3, 4, 5, 6].every((n) => sels.some((s) => new RegExp(`(^|[\\s>])h${n}$`).test(s)))),
    'C-6 見出しの字体トークンを h1〜h6 に当てている', 'h1〜h6 をすべて含む規則に font-family: var(--claudia-font-heading) が無い');

  // サイドバーと上部バーは色と字体だけ
  const chrome = [...each((p, _v, r) => CHROME_SELECTOR_RE.test(r.selector) && !CHROME_PROP_RE.test(p))];
  add(chrome.length === 0, 'C-7 サイドバーと上部バーは色と字体だけを変える', chrome.join(' / '));

  // highlight.js の兄弟クラス（最上位の規則だけを数える）
  const painted = new Map(); // "<範囲> .hljs-<クラス>" -> 群
  for (const r of rules) {
    if (r.nested) continue;
    const hl = r.decls.find(([p, v]) => p === 'color' && /^var\(--claudia-hl-\w+\)$/.test(v));
    if (!hl) continue;
    const group = hl[1].match(/--claudia-hl-(\w+)/)[1];
    for (const sel of r.selector.split(',')) painted.set(sel.trim(), group);
  }
  for (const [group, classes] of Object.entries(HLJS_GROUPS)) {
    for (const cls of classes) {
      const missing = HLJS_SCOPES.filter((scope) => painted.get(`${scope} .hljs-${cls}`) !== group);
      add(missing.length === 0, `C-8 .hljs-${cls} を --claudia-hl-${group} で塗る`, `範囲 ${missing.join('・')} で未設定または群が違う`);
    }
  }
  return results;
}

// --- 実行 ---
if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  let passed = 0;
  let failed = 0;
  const check = (name, ok, detail) => {
    if (ok) passed++;
    else failed++;
    console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${ok || !detail ? '' : '  -- ' + detail}`);
  };

  console.log('\n== (S) 検査器の自己確認 ==');
  check('S-1 黒と白の対比は 21', Math.abs(contrast('#000000', '#ffffff') - 21) < 1e-9);
  check('S-2 同色の対比は 1', contrast('#FAF5EA', '#FAF5EA') === 1);
  {
    const r = checkTheme('.light, html:not(.js) { --fg: #777777; --bg: #888888; }');
    const miss = r.find((x) => x.name === 'トークン --links がある');
    const low = r.find((x) => x.name === '対比 --fg / --bg');
    const absent = r.find((x) => x.theme === 'navy');
    check('S-3 欠けたトークンを失敗にする', miss && !miss.ok);
    check('S-4 対比の不足を失敗にする', low && !low.ok);
    check('S-5 ブロックが無いテーマを失敗にする', absent && !absent.ok);
    const nested = checkTheme('@media print { .navy { --fg: #000000; } }').find((x) => x.theme === 'navy');
    check('S-6 @media の中のブロックは数えない', nested && !nested.ok);
    const named = checkTheme('.light, html:not(.js) { --fg: black; }').find((x) => x.name === '対比 --fg / --bg');
    check('S-7 16 進色でない値を失敗にする', named && !named.ok && named.detail.includes('16 進色でない'));
    const comp = (src, prefix) => checkComponents(src).find((x) => x.name.startsWith(prefix));
    const lit = comp('.x { color: #fff; } .y { border-color: rgba(0,0,0,.1); }', 'C-1');
    check('S-8 部品の色の直書きを失敗にする', lit && !lit.ok);
    const tok = comp('.x { color: var(--claudia-gold); --y: #fff; }', 'C-1');
    check('S-9 トークンの参照と定義は直書きに数えない', tok && tok.ok);
    const rad = comp('.x { border-radius: 6px; }', 'C-2');
    check('S-10 3 段以外の角丸を失敗にする', rad && !rad.ok);
    const sh = comp('.x { box-shadow: 0 1px 2px var(--fg); }', 'C-3');
    check('S-11 トークン以外の影を失敗にする', sh && !sh.ok);
    const side = comp('.sidebar { width: 300px; color: var(--fg); }', 'C-7');
    check('S-12 サイドバーの幅の上書きを失敗にする', side && !side.ok);
    const hl = comp('@media print { .light .hljs-symbol, .navy .hljs-symbol, html:not(.js) .hljs-symbol { color: var(--claudia-hl-string); } }', 'C-8 .hljs-symbol');
    check('S-13 @media の中の着色は数えない', hl && !hl.ok);
    const hlok = comp('.light .hljs-symbol, .navy .hljs-symbol, html:not(.js) .hljs-symbol { color: var(--claudia-hl-string); }', 'C-8 .hljs-symbol');
    check('S-14 3 つの範囲を同じ群で塗れば合格', hlok && hlok.ok);
    const elsewhere = comp('.menu-title { font-family: var(--claudia-font-body); }', 'C-6 本文');
    check('S-15 本文の字体トークンを html・body 以外にだけ当てても失敗', elsewhere && !elsewhere.ok);
    const partial = comp('.content h1, .content h2 { font-family: var(--claudia-font-heading); }', 'C-6 見出し');
    check('S-16 見出しの字体トークンが h1〜h6 の一部だけなら失敗', partial && !partial.ok);
    const push = comp('#mdbook-sidebar-toggle-anchor:checked ~ .page-wrapper { margin-inline-start: 0; }', 'C-7');
    check('S-17 本文を押し出す仕組みの上書きを失敗にする', push && !push.ok);
  }

  let css = null;
  try {
    css = readFileSync(CSS_PATH, 'utf8');
  } catch (e) {
    check('T-0 claudia.css を読める', false, `${CSS_PATH}: ${e.code ?? e.message}`);
  }
  if (css !== null) {
    for (const theme of Object.keys(THEME_SELECTORS)) {
      console.log(`\n== (${theme}) トークンと対比 ==`);
      for (const r of checkTheme(css).filter((x) => x.theme === theme)) check(r.name, r.ok, r.detail);
    }
    console.log('\n== (C) 部品の層 ==');
    for (const r of checkComponents(css)) check(r.name, r.ok, r.detail);
  }

  console.log(`\n結果: ${passed} passed, ${failed} failed`);
  process.exit(failed === 0 ? 0 : 1);
}
