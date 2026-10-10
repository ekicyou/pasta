// theme-menu-test.mjs — テーマメニューのスクリプト（book/theme/claudia.js）の jsdom ユニットテスト
// （manual-claudia-theme タスク 2.5 / 要件 2.1, 2.4, 2.5, 2.6、design「ThemeMenu」「Security Considerations」）。
//
// 観測する完了条件:
//   - claudia.js を実行すると、隠した rust・coal・ayu の項目がメニューの末尾へ移り、
//     先頭から Auto・Light・Navy の順に並ぶ。
//   - mdBook 0.5.x の book.js と同じ矢印キーの規則（隣の li のボタンへ focus()）で、
//     Auto・Light・Navy の 3 項目の間を上下に移動でき、隠した項目へは進まない。
//     Home は Auto、End は Navy へ移る。
//   - 対照として、claudia.js が無いと Light から Navy へ矢印キーで進めない（テストが問題を検出できる）。
//   - 外部への通信（fetch・XMLHttpRequest・WebSocket・EventSource・sendBeacon）をしない。
//     テーマメニューの関数は、保存してある設定値（localStorage・sessionStorage）を読み書きしない。要素を足さず、html のテーマのクラスも変えない。
//     同じファイルの目次の持ち越し（claudiaSidebarKeep）は、目次のあるページで localStorage の mdbook-sidebar を読むだけで、書かない（検査は theme-sidebar-test.mjs。このテストの模型には目次の要素が無いので、保存値に触れない）。
//   - メニューが無いページでは何もしない（例外を出さない）。2 回実行しても並びは変わらない。
//
// テストの模型:
//   - メニューの DOM は mdBook 0.5.4 の出力（book/book/index.html の #mdbook-theme-list）の写し。
//   - 矢印キー等の処理は mdBook 0.5.4 の book.js（themes() の keydown ハンドラ）の写し。
//     実ページと同じく、claudia.js より先に document へ登録する（additional-js は book.js の後に読まれる）。
//   - ブラウザは display: none の要素へフォーカスを移さない。jsdom は描画をしないので、
//     claudia.css の隠し規則（theme-test.mjs の checkMenu と同じ読み方）で隠れるボタンの focus() を何もしない形にする。
//
// jsdom は book/node_modules に導入済み（highlight/neutralizer-test.mjs と同じ）。ビルド不要。
// 実行: `node book/tools/theme-menu-test.mjs`（exit 0 = 全 PASS）。

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { JSDOM } from 'jsdom';
import { CSS_PATH, leafRules } from './theme-test.mjs';

export const JS_PATH = fileURLToPath(new URL('../theme/claudia.js', import.meta.url));

let passed = 0;
let failed = 0;
function check(name, ok, detail) {
  if (ok) passed++;
  else failed++;
  console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${ok || !detail ? '' : '  -- ' + detail}`);
}

// mdBook 0.5.4 のテーマメニュー（index.html の写し）
const MENU_HTML = `
<div id="mdbook-menu-bar"><div class="left-buttons">
<button id="mdbook-theme-toggle" class="icon-button" type="button" title="Change theme" aria-label="Change theme" aria-haspopup="true" aria-expanded="false" aria-controls="mdbook-theme-list">T</button>
<ul id="mdbook-theme-list" class="theme-popup" aria-label="Themes" role="menu">
    <li role="none"><button role="menuitem" class="theme" id="mdbook-theme-default_theme">Auto</button></li>
    <li role="none"><button role="menuitem" class="theme" id="mdbook-theme-light">Light</button></li>
    <li role="none"><button role="menuitem" class="theme" id="mdbook-theme-rust">Rust</button></li>
    <li role="none"><button role="menuitem" class="theme" id="mdbook-theme-coal">Coal</button></li>
    <li role="none"><button role="menuitem" class="theme" id="mdbook-theme-navy">Navy</button></li>
    <li role="none"><button role="menuitem" class="theme" id="mdbook-theme-ayu">Ayu</button></li>
</ul>
</div></div>
<main><p>本文</p></main>`;

// mdBook 0.5.4 の book.js（themes() の keydown ハンドラ）の写し
function installBookJsKeys(win) {
  const document = win.document;
  const themePopup = document.getElementById('mdbook-theme-list');
  document.addEventListener('keydown', function (e) {
    if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) {
      return;
    }
    if (!themePopup.contains(e.target)) {
      return;
    }
    let li;
    switch (e.key) {
    case 'ArrowUp':
      e.preventDefault();
      li = document.activeElement.parentElement;
      if (li && li.previousElementSibling) {
        li.previousElementSibling.querySelector('button').focus();
      }
      break;
    case 'ArrowDown':
      e.preventDefault();
      li = document.activeElement.parentElement;
      if (li && li.nextElementSibling) {
        li.nextElementSibling.querySelector('button').focus();
      }
      break;
    case 'Home':
      e.preventDefault();
      themePopup.querySelector('li:first-child button').focus();
      break;
    case 'End':
      e.preventDefault();
      themePopup.querySelector('li:last-child button').focus();
      break;
    }
  });
}

// claudia.css の最上位の規則で display: none になる #id の一覧
function hiddenIds(css) {
  const ids = new Set();
  for (const r of leafRules(css)) {
    if (r.nested || !r.decls.some(([p, v]) => p === 'display' && v === 'none')) continue;
    for (const s of r.selector.split(',')) {
      const m = s.trim().match(/^#([\w-]+)$/);
      if (m) ids.add(m[1]);
    }
  }
  return ids;
}

// 隠れたボタンへはフォーカスが移らない（ブラウザの挙動の模型）
function modelHiddenFocus(win, ids) {
  const original = win.HTMLElement.prototype.focus;
  win.HTMLElement.prototype.focus = function (...args) {
    if (ids.has(this.id)) return;
    return original.apply(this, args);
  };
}

// 外部通信と保存値への接触を数える
function installSpies(win) {
  const touched = [];
  for (const name of ['localStorage', 'sessionStorage']) {
    Object.defineProperty(win, name, { configurable: true, get() { touched.push(name); return null; } });
  }
  win.fetch = () => { touched.push('fetch'); return Promise.reject(new Error('blocked')); };
  win.XMLHttpRequest.prototype.open = function () { touched.push('XMLHttpRequest'); };
  win.WebSocket = function () { touched.push('WebSocket'); };
  win.EventSource = function () { touched.push('EventSource'); };
  win.navigator.sendBeacon = () => { touched.push('sendBeacon'); return false; };
  return touched;
}

function makeWindow(html, { bookJs = true } = {}) {
  const dom = new JSDOM(`<!doctype html><html lang="ja" class="light js"><head></head><body>${html}</body></html>`, {
    runScripts: 'outside-only',
    url: 'http://localhost/index.html',
  });
  const win = dom.window;
  if (bookJs && win.document.getElementById('mdbook-theme-list')) installBookJsKeys(win);
  return win;
}

const ids = (win) => [...win.document.querySelectorAll('#mdbook-theme-list > li > button')].map((b) => b.id.replace(/^mdbook-theme-/, ''));
const active = (win) => (win.document.activeElement?.id ?? '').replace(/^mdbook-theme-/, '');
function press(win, key) {
  const target = win.document.activeElement;
  target.dispatchEvent(new win.KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
  return active(win);
}
function focus(win, name) {
  win.document.getElementById(`mdbook-theme-${name}`).focus();
}

// --- 実行 ---
let css = null;
let js = null;
try {
  css = readFileSync(CSS_PATH, 'utf8');
} catch (e) {
  check('T-0 claudia.css を読める', false, `${CSS_PATH}: ${e.code ?? e.message}`);
}
try {
  js = readFileSync(JS_PATH, 'utf8');
} catch (e) {
  check('T-0 claudia.js を読める', false, `${JS_PATH}: ${e.code ?? e.message}`);
}

if (css !== null) {
  const hidden = hiddenIds(css);

  console.log('\n== (K0) 対照: claudia.js が無いときの book.js の矢印キー ==');
  {
    const win = makeWindow(MENU_HTML);
    modelHiddenFocus(win, hidden);
    focus(win, 'light');
    check('K0-1 claudia.js が無いと Light から Navy へ進めない（隠れた Rust で止まる）', press(win, 'ArrowDown') === 'light', `実際: ${active(win)}`);
  }

  if (js !== null) {
    console.log('\n== (TM) 並べ替え ==');
    const win = makeWindow(MENU_HTML);
    modelHiddenFocus(win, hidden);
    const touched = installSpies(win);
    const before = win.document.getElementsByTagName('*').length;
    const htmlClass = win.document.documentElement.className;
    let error = null;
    try {
      win.eval(js);
    } catch (e) {
      error = e;
    }
    check('TM-1 claudia.js が例外なく動く', error === null, error?.message);
    const order = ids(win).join(',');
    check('TM-2 並びが Auto・Light・Navy・Rust・Coal・Ayu になる', order === 'default_theme,light,navy,rust,coal,ayu', `実際: ${order}`);
    check('TM-3 claudia.css で隠れるのは末尾の 3 項目だけ', ids(win).map((n) => hidden.has(`mdbook-theme-${n}`)).join(',') === 'false,false,false,true,true,true',
      `隠れる id: ${[...hidden].join(',')}`);

    console.log('\n== (K) 矢印キー・Home・End（book.js の規則） ==');
    focus(win, 'default_theme');
    const down = [press(win, 'ArrowDown'), press(win, 'ArrowDown'), press(win, 'ArrowDown')].join(',');
    check('K-1 ArrowDown で Auto → Light → Navy、Navy で止まる', down === 'light,navy,navy', `実際: ${down}`);
    const up = [press(win, 'ArrowUp'), press(win, 'ArrowUp'), press(win, 'ArrowUp')].join(',');
    check('K-2 ArrowUp で Navy → Light → Auto、Auto で止まる', up === 'light,default_theme,default_theme', `実際: ${up}`);
    focus(win, 'light');
    check('K-3 End で Navy（見えている最後の項目）へ移る', press(win, 'End') === 'navy', `実際: ${active(win)}`);
    check('K-4 Home で Auto へ移る', press(win, 'Home') === 'default_theme', `実際: ${active(win)}`);
    focus(win, 'light');
    win.document.activeElement.dispatchEvent(new win.KeyboardEvent('keydown', { key: 'End', shiftKey: true, bubbles: true, cancelable: true }));
    check('K-5 修飾キー付きの End では動かない（book.js と同じ）', active(win) === 'light', `実際: ${active(win)}`);
    win.document.querySelector('main').dispatchEvent(new win.KeyboardEvent('keydown', { key: 'End', bubbles: true, cancelable: true }));
    check('K-6 メニューの外の End ではフォーカスを動かさない', active(win) === 'light', `実際: ${active(win)}`);

    console.log('\n== (N) 通信・保存値・他の DOM に触れない ==');
    check('N-1 外部通信も保存値の読み書きもしない', touched.length === 0, `触れたもの: ${[...new Set(touched)].join(',')}`);
    check('N-2 要素を足さない', win.document.getElementsByTagName('*').length === before, `前 ${before} → 後 ${win.document.getElementsByTagName('*').length}`);
    check('N-3 html のテーマのクラスを変えない', win.document.documentElement.className === htmlClass, `実際: ${win.document.documentElement.className}`);
    const src = js.replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
    const banned = ['sessionStorage', 'fetch', 'XMLHttpRequest', 'WebSocket', 'EventSource', 'sendBeacon', 'http:', 'https:', 'import(', 'setItem', 'removeItem']
      .filter((w) => src.includes(w));
    check('N-4 ソースに通信・保存値の書き込みの API の名前が無い（コメントを除く。localStorage を読む箇所は theme-sidebar-test.mjs の SRC-2 が検査する）', banned.length === 0, banned.join(','));

    console.log('\n== (E) 冪等・メニューが無いページ ==');
    win.eval(js);
    check('E-1 2 回実行しても並びは変わらない', ids(win).join(',') === order, `実際: ${ids(win).join(',')}`);
    const bare = makeWindow('<main><p>メニューの無いページ</p></main>');
    const bareTouched = installSpies(bare);
    let bareError = null;
    try {
      bare.eval(js);
    } catch (e) {
      bareError = e;
    }
    check('E-2 メニューが無いページでは例外を出さない', bareError === null, bareError?.message);
    check('E-3 メニューが無いページでも通信・保存値に触れない', bareTouched.length === 0, bareTouched.join(','));
  }
}

console.log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
