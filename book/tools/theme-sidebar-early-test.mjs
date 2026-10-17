// theme-sidebar-early-test.mjs — 目次を描く前に開き直すスクリプト（book/theme/head.hbs の claudiaSidebarEarly）の jsdom ユニットテスト
// （getting-started-screenshots の完了時の棚卸・目次の持ち越しのちらつきの修正）。
//
// 直す不具合:
//   幅 620〜1080 px で目次を開いたまま項目を押すと、移った先のページで目次が一瞬閉じて本文の幅が変わり、ちらつく。
//   mdBook 0.5.4 の本文の頭の埋め込みスクリプトが毎回目次を閉じ、claudia.js の claudiaSidebarKeep が
//   本文の最後（読み込みの後）で開き直すまでの間に、閉じた画面が一度描かれるため。
//
// 観測する完了条件:
//   - 幅 620 以上 1080 未満・保存値 mdbook-sidebar が visible のとき、mdBook の埋め込みスクリプトが閉じた目次を、
//     ページの読み込みが終わるより前（DOMContentLoaded の前）に開いた状態へ戻す。
//     html の sidebar-visible・チェックボックス・ARIA 属性・リンクの tabIndex・目次の display がそろう。
//   - 開くときのアニメーションを出さない（開くときに html に sidebar-resizing を付け、描いた後に外す）。
//   - 保存値が hidden・無し、幅が 620 未満・1080 以上なら何もしない（mdBook のふるまいのまま）。
//   - 保存値を書かない。例外を外へ出さない。
//   - claudia.js の claudiaSidebarKeep と一緒に動いても、change を送らない（二重に開かない）。
//
// テストの模型:
//   - ページは mdBook 0.5.4 の出力（book/book の章のページ）の順を写す: head の中に head.hbs の該当ブロック、
//     body の頭にテーマの埋め込み（html のクラスを替える）、チェックボックス、目次を閉じる埋め込み、目次、ARIA の埋め込み。
//   - jsdom は描画をしないので、body の clientWidth は場合ごとの値を返す形に差し替える（beforeParse）。
//   - 実際のブラウザーでは、MutationObserver の知らせは目次を閉じる埋め込みの直後（パーサーが続きを読む前）に届く。
//     jsdom では知らせがパースの後に届くので、ARIA の埋め込みが閉じた状態を書いた後に開き直す形になる。
//     どちらの順でも最後の状態がそろうことを確かめる。
//
// 実行: `node book/tools/theme-sidebar-early-test.mjs`（exit 0 = 全 PASS）。

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { JSDOM } from 'jsdom';

export const HEAD_PATH = fileURLToPath(new URL('../theme/head.hbs', import.meta.url));
export const JS_PATH = fileURLToPath(new URL('../theme/claudia.js', import.meta.url));

let passed = 0;
let failed = 0;
function check(name, ok, detail) {
  if (ok) passed++;
  else failed++;
  console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${ok || !detail ? '' : '  -- ' + detail}`);
}

// head.hbs から claudiaSidebarEarly の <script> を取り出す
function extractEarly(head) {
  const m = head.match(/<script>\s*\(function claudiaSidebarEarly\(\) \{[\s\S]*?\}\)\(\);\s*<\/script>/);
  return m ? m[0] : null;
}

// mdBook 0.5.4 の章のページの埋め込み（book/book/getting-started/01-boot.html の写し）
const THEME_INLINE = `<script>
    const default_theme = 'light';
    let theme = default_theme;
    const html = document.documentElement;
    html.classList.remove('light')
    html.classList.add(theme);
    html.classList.add("js");
</script>`;
const CHECKBOX_HTML = '<input type="checkbox" id="mdbook-sidebar-toggle-anchor" class="hidden">';
const CLOSE_INLINE = `<script>
    let sidebar = null;
    const sidebar_toggle = document.getElementById("mdbook-sidebar-toggle-anchor");
    if (document.body.clientWidth >= 1080) {
        try { sidebar = localStorage.getItem('mdbook-sidebar'); } catch(e) { }
        sidebar = sidebar || 'visible';
    } else {
        sidebar = 'hidden';
        sidebar_toggle.checked = false;
    }
    if (sidebar === 'visible') {
        sidebar_toggle.checked = true;
    } else {
        html.classList.remove('sidebar-visible');
    }
</script>`;
const NAV_HTML = `<nav id="mdbook-sidebar" class="sidebar" aria-label="Table of contents">
    <mdbook-sidebar-scrollbox class="sidebar-scrollbox"><ol class="chapter"><li><a href="index.html">はじめに</a></li><li><a href="setup.html">準備</a></li></ol></mdbook-sidebar-scrollbox>
</nav>`;
const ARIA_INLINE = `<script>
    document.getElementById('mdbook-sidebar-toggle').setAttribute('aria-expanded', sidebar === 'visible');
    document.getElementById('mdbook-sidebar').setAttribute('aria-hidden', sidebar !== 'visible');
    Array.from(document.querySelectorAll('#mdbook-sidebar a')).forEach(function(link) {
        link.setAttribute('tabIndex', sidebar === 'visible' ? 0 : -1);
    });
</script>`;
const PAGE_HTML = `<div id="mdbook-page-wrapper" class="page-wrapper"><div class="page">
<label id="mdbook-sidebar-toggle" class="icon-button" for="mdbook-sidebar-toggle-anchor" aria-controls="mdbook-sidebar">M</label>
<main><p>本文</p></main></div></div>`;

// 1 つの場合を組み立ててページを読み込ませ、観測した結果を返す。
//   width: body の clientWidth、stored: 保存値（null = 無し）、broken: 読むと例外、keep: claudia.js も最後に読む
async function run(early, { width = 800, stored = 'visible', broken = false, keep = false, js = '' } = {}) {
  const log = [];
  const changes = [];
  const page = `<!doctype html><html lang="ja" class="light sidebar-visible"><head>${early}</head><body>
${THEME_INLINE}
${CHECKBOX_HTML}
${CLOSE_INLINE}
${PAGE_HTML}
${NAV_HTML}
${ARIA_INLINE}
</body></html>`;
  let errors = [];
  const dom = new JSDOM(page, {
    runScripts: 'dangerously',
    pretendToBeVisual: true,
    url: 'http://localhost/getting-started/01-boot.html',
    beforeParse(win) {
      Object.defineProperty(win.HTMLBodyElement.prototype, 'clientWidth', { configurable: true, get: () => width });
      const storage = {
        getItem(key) { log.push(`getItem:${key}`); if (broken) throw new Error('blocked'); return stored; },
        setItem(key, value) { log.push(`setItem:${key}=${value}`); },
        removeItem(key) { log.push(`removeItem:${key}`); },
        clear() { log.push('clear'); },
      };
      Object.defineProperty(win, 'localStorage', { configurable: true, get: () => storage });
      win.addEventListener('error', (e) => errors.push(e.message));
      win.document.addEventListener('DOMContentLoaded', () => {
        const d = win.document;
        log.push(`dcl:${d.documentElement.classList.contains('sidebar-visible')}:${d.getElementById('mdbook-sidebar-toggle-anchor').checked}`);
      });
      win.document.addEventListener('change', (e) => changes.push(e.target.id), true);
    },
  });
  const win = dom.window;
  // 描いた後（2 フレーム）を待つ
  await new Promise((resolve) => setTimeout(resolve, 120));
  // claudia.js は本文の最後に読まれる。実際のブラウザーでは見張りの知らせがそれより先に届くので、
  // jsdom でも開き直しが済んだ後に走らせて、実際の順を写す。
  if (keep) {
    try { win.eval(js); } catch (e) { errors.push(e.message); }
  }
  const d = win.document;
  const nav = d.getElementById('mdbook-sidebar');
  const result = {
    html: d.documentElement,
    checked: d.getElementById('mdbook-sidebar-toggle-anchor').checked,
    ariaHidden: nav.getAttribute('aria-hidden'),
    ariaExpanded: d.getElementById('mdbook-sidebar-toggle').getAttribute('aria-expanded'),
    tab: [...nav.querySelectorAll('a')].map((a) => a.getAttribute('tabIndex')).join(','),
    display: nav.style.display,
    dcl: log.find((x) => x.startsWith('dcl:')),
    writes: log.filter((x) => /^(setItem|removeItem|clear)/.test(x)),
    changes,
    errors,
  };
  win.close();
  return result;
}

function isOpen(r) {
  return r.html.classList.contains('sidebar-visible') && r.checked && r.ariaHidden === 'false'
    && r.ariaExpanded === 'true' && r.tab === '0,0' && r.display === '';
}
function state(r) {
  return `class=${r.html.className} checked=${r.checked} aria-hidden=${r.ariaHidden} aria-expanded=${r.ariaExpanded} tab=${r.tab} display=${JSON.stringify(r.display)} ${r.dcl}`;
}

let head = null;
let js = null;
try {
  head = readFileSync(HEAD_PATH, 'utf8');
  js = readFileSync(JS_PATH, 'utf8');
} catch (e) {
  check('T-0 head.hbs と claudia.js を読める', false, e.message);
}

const early = head === null ? null : extractEarly(head);
check('T-1 head.hbs に claudiaSidebarEarly の <script> がある', early !== null);

if (early !== null) {
  console.log('\n== (S) 幅 800・保存値 visible: 読み込みの終わる前に開いた状態へ戻す ==');
  {
    const r = await run(early);
    check('S-1 最後の状態が開いている（class・checkbox・ARIA・tabIndex・display）', isOpen(r), state(r));
    check('S-2 DOMContentLoaded の時点ですでに開いている', r.dcl === 'dcl:true:true', r.dcl);
    check('S-3 描いた後に sidebar-resizing が残らない', !r.html.classList.contains('sidebar-resizing'), r.html.className);
    check('S-4 保存値を書かず、change を送らず、例外を出さない', r.writes.length === 0 && r.changes.length === 0 && r.errors.length === 0,
      `writes=${r.writes} changes=${r.changes} errors=${r.errors}`);
  }

  console.log('\n== (W) 幅（620 以上 1080 未満だけ開き直す） ==');
  for (const [id, width, opens] of [['W-1', 500, false], ['W-2', 619, false], ['W-3', 620, true], ['W-4', 1079, true]]) {
    const r = await run(early, { width });
    if (opens) check(`${id} 幅 ${width}: 開いた状態へ戻す`, isOpen(r) && r.dcl === 'dcl:true:true', state(r));
    else check(`${id} 幅 ${width}: 閉じたまま（mdBook のまま）`, !r.html.classList.contains('sidebar-visible') && !r.checked && r.ariaHidden === 'true', state(r));
    check(`${id}c 例外を出さず、保存値を書かない`, r.errors.length === 0 && r.writes.length === 0, `${r.errors} ${r.writes}`);
  }
  {
    const r = await run(early, { width: 1200 });
    check('W-5 幅 1200・保存値 visible: mdBook が開いたまま（そのまま）', isOpen(r), state(r));
    const r2 = await run(early, { width: 1200, stored: 'hidden' });
    check('W-6 幅 1200・保存値 hidden: mdBook が閉じたまま（開き直さない）', !r2.html.classList.contains('sidebar-visible') && !r2.checked, state(r2));
  }

  console.log('\n== (V) 幅 800・保存値が visible でない・読めない ==');
  for (const [id, label, opts] of [['V-1', '保存値 hidden', { stored: 'hidden' }], ['V-2', '保存値 無し', { stored: null }], ['V-3', '保存値を読むと例外', { broken: true }]]) {
    const r = await run(early, opts);
    check(`${id} ${label}: 閉じたまま`, !r.html.classList.contains('sidebar-visible') && !r.checked && r.ariaHidden === 'true', state(r));
    check(`${id}c 例外を外へ出さず、保存値を書かない`, r.errors.length === 0 && r.writes.length === 0, `${r.errors} ${r.writes}`);
  }

  console.log('\n== (K) claudia.js の claudiaSidebarKeep と一緒に動く ==');
  if (js !== null) {
    const r = await run(early, { keep: true, js });
    check('K-1 開いた状態のまま、change を 1 回も送らない（二重に開かない）', isOpen(r) && r.changes.length === 0, `${state(r)} changes=${r.changes}`);
    check('K-2 保存値を書かず、例外を出さない', r.writes.length === 0 && r.errors.length === 0, `${r.writes} ${r.errors}`);
  }

  console.log('\n== (SRC) ソース（コメントを除く） ==');
  {
    const src = early.replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
    const banned = ['sessionStorage', 'setItem', 'removeItem', '.clear(', 'fetch', 'XMLHttpRequest', 'WebSocket', 'EventSource', 'sendBeacon', 'http:', 'https:',
      'createElement', 'innerHTML', 'removeChild', 'dispatchEvent'].filter((w) => src.includes(w));
    check('SRC-1 保存値を書く API・通信の API・要素を足す／消す API・change を送る API の名前が無い', banned.length === 0, banned.join(','));
    const touches = src.match(/localStorage[^;\n]*/g) ?? [];
    check("SRC-2 保存値に触れるのは localStorage.getItem('mdbook-sidebar') の 1 か所だけ",
      touches.length === 1 && touches[0] === "localStorage.getItem('mdbook-sidebar')", touches.join(' | ') || '無し');
  }
}

console.log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
