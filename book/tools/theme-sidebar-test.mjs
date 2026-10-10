// theme-sidebar-test.mjs — 目次の持ち越しのスクリプト（book/theme/claudia.js の claudiaSidebarKeep）の jsdom ユニットテスト
// （getting-started-story-guide タスク 2.4 / 要件 13.1, 13.2, 13.3, 13.4、design「SidebarKeeper」）。
//
// 観測する完了条件:
//   - 幅 620 以上 1080 未満・保存値 mdbook-sidebar が visible・目次が閉じている、のときだけ目次を開く。
//     開く手順は book.js に任せる（目次の display を空に戻し、チェックボックスを入れて change を 1 回送る）。
//   - 開くときのアニメーションを出さない（change の間は html に sidebar-resizing が付いていて、終わると外れている）。
//   - 保存値が hidden・無し、幅が 620 未満・1080 以上、すでに開いている、のどれかなら何もしない。
//   - 目次の要素が無いページでは保存値を読まない。保存値を読むと例外になるときは何もせず、例外を外へ出さない。
//   - 保存値を書かない。外部への通信（fetch・XMLHttpRequest・WebSocket・EventSource・sendBeacon）をしない。
//     要素を足さず、消さない。
//
// テストの模型:
//   - 目次の DOM は mdBook 0.5.4 の出力（book/book/index.html）の写し。目次のリンクは toc.js が入れるものの代わり。
//   - 開閉の処理は mdBook 0.5.4 の book.js（sidebar() の初期化と change のリスナー）の写し。
//     実ページと同じく、claudia.js より先に登録する（additional-js は book.js の後に読まれる）。
//   - 実ページの先頭のインラインスクリプトは、幅 1080 未満では保存値を見ずに毎回閉じ、1080 以上では保存値どおりにする。
//     ここでは条件を 1 つずつ確かめるために、開閉の状態を場合ごとに直接作る（幅 1080 以上で閉じた状態も作る）。
//   - jsdom は描画をしないので、body の clientWidth は場合ごとの値を返す形に差し替える。
//
// jsdom は book/node_modules に導入済み（theme-menu-test.mjs と同じ）。ビルド不要。
// 実行: `node book/tools/theme-sidebar-test.mjs`（exit 0 = 全 PASS）。

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { JSDOM } from 'jsdom';

export const JS_PATH = fileURLToPath(new URL('../theme/claudia.js', import.meta.url));

let passed = 0;
let failed = 0;
function check(name, ok, detail) {
  if (ok) passed++;
  else failed++;
  console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${ok || !detail ? '' : '  -- ' + detail}`);
}

// mdBook 0.5.4 の目次まわり（index.html の写し）
const CHECKBOX_HTML = '<input type="checkbox" id="mdbook-sidebar-toggle-anchor" class="hidden">';
const NAV_HTML = `
<nav id="mdbook-sidebar" class="sidebar" aria-label="Table of contents">
    <mdbook-sidebar-scrollbox class="sidebar-scrollbox"><ol class="chapter"><li><a href="index.html">はじめに</a></li><li><a href="setup.html">準備</a></li></ol></mdbook-sidebar-scrollbox>
    <div id="mdbook-sidebar-resize-handle" class="sidebar-resize-handle"><div class="sidebar-resize-indicator"></div></div>
</nav>`;
const PAGE_HTML = `
<div id="mdbook-page-wrapper" class="page-wrapper"><div class="page">
<div id="mdbook-menu-bar" class="menu-bar sticky"><div class="left-buttons">
<label id="mdbook-sidebar-toggle" class="icon-button" for="mdbook-sidebar-toggle-anchor" title="Toggle Table of Contents" aria-label="Toggle Table of Contents" aria-controls="mdbook-sidebar">M</label>
</div></div>
<main><p>本文</p></main>
</div></div>`;

// mdBook 0.5.4 の book.js（sidebar() の初期化と change のリスナー）の写し
function installBookJsSidebar(win) {
  const document = win.document;
  const sidebar = document.getElementById('mdbook-sidebar');
  const sidebarLinks = document.querySelectorAll('#mdbook-sidebar a');
  const sidebarToggleButton = document.getElementById('mdbook-sidebar-toggle');
  const sidebarCheckbox = document.getElementById('mdbook-sidebar-toggle-anchor');

  if (!document.documentElement.classList.contains('sidebar-visible')) {
    sidebar.style.display = 'none';
  }

  function showSidebar() {
    document.documentElement.classList.add('sidebar-visible');
    Array.from(sidebarLinks).forEach(function (link) {
      link.setAttribute('tabIndex', 0);
    });
    sidebarToggleButton.setAttribute('aria-expanded', true);
    sidebar.setAttribute('aria-hidden', false);
    try {
      win.localStorage.setItem('mdbook-sidebar', 'visible');
    } catch {
      // Ignore error.
    }
  }

  function hideSidebar() {
    document.documentElement.classList.remove('sidebar-visible');
    Array.from(sidebarLinks).forEach(function (link) {
      link.setAttribute('tabIndex', -1);
    });
    sidebarToggleButton.setAttribute('aria-expanded', false);
    sidebar.setAttribute('aria-hidden', true);
    try {
      win.localStorage.setItem('mdbook-sidebar', 'hidden');
    } catch {
      // Ignore error.
    }
  }

  sidebarCheckbox.addEventListener('change', function sidebarToggle() {
    if (sidebarCheckbox.checked) {
      showSidebar();
    } else {
      hideSidebar();
    }
  });
}

// 保存値の模型。触れた順に log へ積む（get = localStorage を取り出した、getItem・setItem 等 = 呼んだ）。
// broken: 'get' は localStorage を取り出すだけで例外（保存が禁止のブラウザ）、'getItem' は読むと例外。
function installStorage(win, stored, broken) {
  const log = [];
  const storage = {
    getItem(key) {
      log.push(`getItem:${key}`);
      if (broken === 'getItem') throw new Error('blocked');
      return stored;
    },
    setItem(key, value) { log.push(`setItem:${key}=${value}`); },
    removeItem(key) { log.push(`removeItem:${key}`); },
    clear() { log.push('clear'); },
  };
  Object.defineProperty(win, 'localStorage', {
    configurable: true,
    get() {
      log.push('get');
      if (broken === 'get') throw new Error('blocked');
      return storage;
    },
  });
  Object.defineProperty(win, 'sessionStorage', { configurable: true, get() { log.push('sessionStorage'); return null; } });
  return log;
}

// 外部通信を数える
function installNetSpies(win) {
  const touched = [];
  win.fetch = () => { touched.push('fetch'); return Promise.reject(new Error('blocked')); };
  win.XMLHttpRequest.prototype.open = function () { touched.push('XMLHttpRequest'); };
  win.WebSocket = function () { touched.push('WebSocket'); };
  win.EventSource = function () { touched.push('EventSource'); };
  win.navigator.sendBeacon = () => { touched.push('sendBeacon'); return false; };
  return touched;
}

// 1 つの場合を組み立てて claudia.js を実行し、観測した結果を返す。
//   width: body の clientWidth、stored: 保存値（null = 無し）、open: 実行前に目次が開いているか
//   checkbox・nav: 目次の要素を置くか、bookJs: book.js の写しを登録するか、broken: 保存値の模型の壊し方
function run(js, { width = 800, stored = 'visible', open = false, checkbox = true, nav = true, bookJs = false, broken = null } = {}) {
  const dom = new JSDOM(
    `<!doctype html><html lang="ja" class="light js${open ? ' sidebar-visible' : ''}"><head></head><body>${checkbox ? CHECKBOX_HTML : ''}${nav ? NAV_HTML : ''}${PAGE_HTML}</body></html>`,
    { runScripts: 'outside-only', url: 'http://localhost/index.html' },
  );
  const win = dom.window;
  const document = win.document;
  const html = document.documentElement;
  const box = document.getElementById('mdbook-sidebar-toggle-anchor');
  const sidebar = document.getElementById('mdbook-sidebar');
  Object.defineProperty(document.body, 'clientWidth', { configurable: true, get: () => width });

  const log = installStorage(win, stored, broken);
  const net = installNetSpies(win);
  if (box) box.checked = open;
  if (sidebar && !open) sidebar.style.display = 'none'; // book.js が閉じた目次に付ける
  if (bookJs) installBookJsSidebar(win);

  // change が送られた時点の状態と、そのあとの位置の確定（強制リフロー。book.js と同じ offsetHeight の読み取り）を記録する
  const changes = [];
  const reflows = [];
  if (box) {
    box.addEventListener('change', (e) => {
      changes.push({ target: e.target === box, checked: box.checked, display: sidebar?.style.display, resizing: html.classList.contains('sidebar-resizing') });
    });
  }
  if (sidebar) {
    Object.defineProperty(sidebar, 'offsetHeight', {
      configurable: true,
      get() {
        reflows.push({ after: changes.length, resizing: html.classList.contains('sidebar-resizing') });
        return 0;
      },
    });
  }

  log.length = 0; // ここまでの準備（book.js の写し）が触れた分は数えない
  const before = document.getElementsByTagName('*').length;
  const htmlClass = html.className;
  let error = null;
  try {
    win.eval(js);
  } catch (e) {
    error = e;
  }
  return {
    win, box, sidebar, html, log, net, changes, reflows, error, htmlClass,
    reads: log.filter((x) => x.startsWith('getItem:')),
    writes: log.filter((x) => /^(setItem|removeItem|clear)/.test(x)),
    added: document.getElementsByTagName('*').length - before,
    resizingLeft: html.classList.contains('sidebar-resizing'),
  };
}

// どの場合にも共通の確かめ（設計の表の「全部の場合」）
function checkCommon(id, r, { writes = 0 } = {}) {
  const bad = [];
  if (r.error !== null) bad.push(`例外: ${r.error.message}`);
  if (r.writes.length !== writes) bad.push(`保存値への書き込み: ${r.writes.join(',') || '無し'}`);
  if (r.log.includes('sessionStorage')) bad.push('sessionStorage に触れた');
  if (r.net.length !== 0) bad.push(`通信: ${[...new Set(r.net)].join(',')}`);
  if (r.added !== 0) bad.push(`要素の増減: ${r.added}`);
  if (r.resizingLeft) bad.push('sidebar-resizing が残っている');
  check(`${id} 例外を出さず、保存値を書かず、通信せず、要素を増減させず、sidebar-resizing を残さない`, bad.length === 0, bad.join(' / '));
}

// 何もしない場合の確かめ
function checkUntouched(id, label, r, { checked = false, display = 'none' } = {}) {
  const actual = `checked=${r.box?.checked} change=${r.changes.length} display=${JSON.stringify(r.sidebar?.style.display)} class=${r.html.className}`;
  const ok = r.changes.length === 0
    && (!r.box || r.box.checked === checked)
    && (!r.sidebar || r.sidebar.style.display === display)
    && r.html.className === r.htmlClass;
  check(`${id} ${label}: 何もしない`, ok, `実際: ${actual}`);
}

// --- 実行 ---
let js = null;
try {
  js = readFileSync(JS_PATH, 'utf8');
} catch (e) {
  check('T-0 claudia.js を読める', false, `${JS_PATH}: ${e.code ?? e.message}`);
}

if (js !== null) {
  console.log('\n== (S) 幅 800・保存値 visible・閉じている: 開く ==');
  {
    const r = run(js);
    check('S-1 チェックボックスが入る', r.box.checked === true, `checked=${r.box.checked}`);
    check('S-2 change が 1 回、チェックボックスへ送られる', r.changes.length === 1 && r.changes[0].target, `回数: ${r.changes.length}`);
    check('S-3 目次の display が空になる', r.sidebar.style.display === '', `実際: ${JSON.stringify(r.sidebar.style.display)}`);
    check('S-4 change の時点で、チェックボックスが入っていて display が空になっている',
      r.changes.length === 1 && r.changes[0].checked === true && r.changes[0].display === '', JSON.stringify(r.changes));
    check('S-5 change の時点で html に sidebar-resizing が付いている（アニメーションを出さない）',
      r.changes.length === 1 && r.changes[0].resizing === true, JSON.stringify(r.changes));
    check('S-6 sidebar-resizing を外す前に、付けたまま位置を確定させる（change の後に目次の offsetHeight を読む）',
      r.reflows.some((x) => x.after === 1 && x.resizing), JSON.stringify(r.reflows));
    check('S-7 終わると html のクラスは元のまま（sidebar-resizing が残らない）', r.html.className === r.htmlClass, `実際: ${r.html.className}`);
    check('S-8 保存値は mdbook-sidebar を 1 回読むだけ', r.reads.join(',') === 'getItem:mdbook-sidebar', `実際: ${r.log.join(',')}`);
    checkCommon('S-9', r);
    r.win.eval(js);
    check('S-10 2 回実行しても change は 1 回のまま（開いた目次には何もしない）', r.changes.length === 1 && r.box.checked === true, `回数: ${r.changes.length}`);
  }

  console.log('\n== (BJ) book.js の写しと合わせる: 開いた状態がそろう ==');
  {
    const r = run(js, { bookJs: true });
    const links = [...r.sidebar.querySelectorAll('a')].map((a) => a.getAttribute('tabIndex')).join(',');
    check('BJ-1 html に sidebar-visible が付く', r.html.classList.contains('sidebar-visible'), `実際: ${r.html.className}`);
    check('BJ-2 ARIA 属性とリンクの tabIndex が開いた状態になる',
      r.win.document.getElementById('mdbook-sidebar-toggle').getAttribute('aria-expanded') === 'true'
      && r.sidebar.getAttribute('aria-hidden') === 'false' && links === '0,0',
      `aria-hidden=${r.sidebar.getAttribute('aria-hidden')} tabIndex=${links}`);
    check('BJ-3 保存値への書き込みは book.js の 1 回だけで、値は visible のまま', r.writes.join(',') === 'setItem:mdbook-sidebar=visible', `実際: ${r.writes.join(',')}`);
    check('BJ-4 sidebar-resizing が残らない', !r.resizingLeft, `実際: ${r.html.className}`);
    checkCommon('BJ-5', r, { writes: 1 });
  }

  console.log('\n== (V) 幅 800・保存値が visible でない ==');
  {
    const hidden = run(js, { stored: 'hidden' });
    checkUntouched('V-1', '保存値 hidden', hidden);
    checkCommon('V-2', hidden);
    const none = run(js, { stored: null });
    checkUntouched('V-3', '保存値 無し', none);
    checkCommon('V-4', none);
  }

  console.log('\n== (W) 幅（620 以上 1080 未満だけ開く） ==');
  for (const [id, width, opens] of [['W-1', 500, false], ['W-2', 619, false], ['W-3', 620, true], ['W-4', 1079, true], ['W-5', 1080, false], ['W-6', 1200, false]]) {
    const r = run(js, { width });
    if (opens) {
      check(`${id} 幅 ${width}: 開く`, r.box.checked === true && r.changes.length === 1 && r.sidebar.style.display === '',
        `checked=${r.box.checked} change=${r.changes.length} display=${JSON.stringify(r.sidebar.style.display)}`);
    } else {
      checkUntouched(id, `幅 ${width}（保存値 visible・閉じている）`, r);
    }
    checkCommon(`${id}c`, r);
  }

  console.log('\n== (O) すでに開いている ==');
  {
    const r = run(js, { open: true });
    checkUntouched('O-1', '幅 800・保存値 visible・開いている', r, { checked: true, display: '' });
    checkCommon('O-2', r);
  }

  console.log('\n== (N) 目次の要素が無いページ ==');
  for (const [id, label, opts] of [
    ['N-1', 'チェックボックスも目次も無い', { checkbox: false, nav: false }],
    ['N-2', 'チェックボックスだけが無い', { checkbox: false }],
    ['N-3', '目次だけが無い', { nav: false }],
  ]) {
    const r = run(js, opts);
    check(`${id} ${label}: 保存値を読まない（localStorage にも触れない）`, r.log.length === 0, `触れたもの: ${r.log.join(',')}`);
    checkUntouched(`${id}u`, label, r);
    checkCommon(`${id}c`, r);
  }

  console.log('\n== (X) 保存値を読むと例外になる ==');
  for (const [id, label, broken] of [['X-1', 'localStorage を取り出すと例外', 'get'], ['X-2', 'getItem が例外', 'getItem']]) {
    const r = run(js, { broken });
    check(`${id} ${label}: 保存値を読もうとして、例外を外へ出さない`, r.log.includes('get') && r.error === null, r.error?.message ?? `触れたもの: ${r.log.join(',') || '無し'}`);
    checkUntouched(`${id}u`, label, r);
    checkCommon(`${id}c`, r);
  }

  console.log('\n== (SRC) ソース（コメントを除く） ==');
  {
    const src = js.replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
    const banned = ['sessionStorage', 'setItem', 'removeItem', '.clear(', 'fetch', 'XMLHttpRequest', 'WebSocket', 'EventSource', 'sendBeacon', 'http:', 'https:', 'import(',
      'createElement', 'insertAdjacent', 'innerHTML', 'removeChild']
      .filter((w) => src.includes(w));
    check('SRC-1 保存値を書く API・通信の API・要素を足す／消す API の名前が無い', banned.length === 0, banned.join(','));
    const touches = src.match(/localStorage[^;\n]*/g) ?? [];
    check("SRC-2 保存値に触れるのは localStorage.getItem('mdbook-sidebar') の 1 か所だけ",
      touches.length === 1 && touches[0] === "localStorage.getItem('mdbook-sidebar')", touches.join(' | ') || '無し');
    check('SRC-3 目次の持ち越しは、テーマメニューとは別の即時関数 claudiaSidebarKeep',
      /\(function claudiaThemeMenu\(\) \{[\s\S]*?\n\}\)\(\);[\s\S]*\(function claudiaSidebarKeep\(\) \{[\s\S]*?\n\}\)\(\);/.test(src));
  }
}

console.log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
