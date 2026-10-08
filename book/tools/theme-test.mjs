// theme-test.mjs — Claudia テーマ（book/theme/claudia.css）の配色トークンの対比テスト
// （manual-claudia-theme タスク 2.2 / 要件 1.1, 2.2, 2.4, 3.5, 8.3、design「ThemeTest」「テーマトークン契約」）。
//
// 観測する完了条件:
//   - claudia.css の最上位ブロック `.light, html:not(.js)` と `.navy` から `--name: #rrggbb;` を読む。
//   - 両ブロックに mdBook のテーマ変数 40 個と Claudia のトークンがすべてある（欠けたら失敗）。
//   - 設計の「テーマトークン契約」で決めた文字色と背景色の組ごとに、WCAG の対比が 4.5 以上。
//     対象のトークンが欠けている、または 16 進色（#rrggbb）でない場合も失敗。
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
  }

  console.log(`\n結果: ${passed} passed, ${failed} failed`);
  process.exit(failed === 0 ? 0 : 1);
}
