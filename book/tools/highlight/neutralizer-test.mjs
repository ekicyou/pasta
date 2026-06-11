// neutralizer-test.mjs — ClientNeutralizer の jsdom build-time ユニットテスト
// （タスク 4.2 / 要件 3.1, 3.2）。
//
// 目的:
//   正準モジュール `neutralizer.mjs` の `installHljsNeutralizer(win)` を、実 DOM
//   （jsdom）上で book.js の `codeSnippets()` IIFE 挙動を忠実に模擬して検証する。
//
//   book.js（research §8.3・`book-c22b7243.js:173-202`）は次のように振る舞う:
//     - `hljs.configure({ languages: [] })` で自動言語検出を無効化したのち、
//       ヘッダ内 inline code を除く全 `<code>` を収集し、各要素へ
//       `hljs.highlightBlock(block)` を無条件適用する（`language-pasta` を
//       スキップする分岐は存在しない）。
//     - 未登録言語 `pasta` は plaintext 扱いとなり、`highlightBlock` は要素の
//       `textContent` を読み直して `innerHTML` をエスケープ再生成する。これにより
//       build-time に焼き込んだ事前 span（`<span class="hljs-...">`）が破壊される。
//
//   本テストは:
//     - 実 hljs の「未中和なら事前 span を破壊する」挙動を `fakeHljs` で忠実に模擬し
//       （`el.innerHTML = escapeHtml(el.textContent)`）、呼び出し回数も spy で記録する。
//     - `installHljsNeutralizer` を `window.hljs` 代入前に仕込み、代入の瞬間に
//       `highlightBlock` / `highlightElement` がラップされることを利用する。
//     - 中和が効いた状態で、pasta ブロックの事前 span が生存（3.1）し、
//       非 pasta ブロックは原処理へ委譲されて既存挙動どおり再生成される（3.2）ことを
//       実 DOM 上で証明する。
//
//   公開サイトや実 `book/book` を一切叩かない build-time の純粋単体検証であり、
//   恒常ゲートではない（要件 8.4 / 6.3 非抵触）。
//
// 実行: `node book/tools/highlight/neutralizer-test.mjs`（exit 0 = 全 PASS）。
// jsdom は book/node_modules に導入済み（タスク 1.1）。

import { JSDOM } from 'jsdom';
import { installHljsNeutralizer } from './neutralizer.mjs';

// --- 最小 assert ハーネス（依存ゼロ・bigram build-index-test.mjs / tokenizer-test.mjs に倣う） ---
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

// HTML エンティティ最小エスケープ（実 hljs の plaintext 再生成と同型）。
function escapeHtml(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

// pasta ブロックの実 HTML（事前 span 焼き込み済み）。
// `language-pasta` クラスを持ち、内部に hljs span を内包する（タスク 3.x の出力相当）。
// textContent は "＊シーン　こんにちは"。未中和の highlightBlock が textContent から
// innerHTML を再生成すると、これらの span は失われる（= 3.1 が破られる）。
const PASTA_INNER_HTML =
  '<span class="hljs-keyword">＊</span>' +
  '<span class="hljs-title">シーン</span>' +
  '　こんにちは';

// 非 pasta（他言語）ブロック。事前 span は無い純テキスト（`<` を含み再生成を観測可能に）。
const RUST_TEXT = 'fn main() { let x = a < b; }';

// 無指定ブロック（任意・他言語委譲側に分類される）。
const PLAIN_TEXT = 'plain text';

// book.js codeSnippets の本体ループを忠実に模擬する。
// - ヘッダ（`.header`）内の inline code を除く全 `<code>` を集める。
// - 各要素へ `win.hljs.highlightBlock(code)` を無条件適用する（言語分岐なし）。
// `useElementApi` が true のときは新名 `highlightElement` 経路を使う。
function runCodeSnippets(doc, win, useElementApi) {
  const codes = Array.prototype.slice.call(doc.querySelectorAll('code'));
  // book.js はヘッダ内 inline code を除外する（`.header code` を弾く）。
  const targets = codes.filter((c) => !c.closest('.header'));
  for (const code of targets) {
    if (useElementApi) {
      win.hljs.highlightElement(code);
    } else {
      win.hljs.highlightBlock(code);
    }
  }
  return targets;
}

// 実 hljs を模した fakeHljs を構築する。
// `highlightBlock` / `highlightElement` は未中和なら事前 span を破壊する実挙動を模擬:
// 要素の textContent を読み直して innerHTML をエスケープ再生成する（plaintext 扱い）。
// 呼び出された要素の className を spy 配列へ記録する。
function makeFakeHljs() {
  const spy = { block: [], element: [] };
  function rehighlight(el) {
    // 実 hljs と同じく textContent から innerHTML を作り直す（span 消滅の根本原因）。
    el.innerHTML = escapeHtml(el.textContent);
  }
  return {
    spy,
    configure() {
      /* 自動検出オフ相当。挙動には影響しない。 */
    },
    highlightBlock(el) {
      spy.block.push(el.className);
      rehighlight(el);
    },
    highlightElement(el) {
      spy.element.push(el.className);
      rehighlight(el);
    },
  };
}

// jsdom ドキュメントを構築する。`url` は file:// 相当の検証にも使える。
function buildDom(url) {
  const html = `<!DOCTYPE html><html><head></head><body>
    <div class="header"><code>inline-in-header</code></div>
    <pre><code class="language-pasta">${PASTA_INNER_HTML}</code></pre>
    <pre><code class="language-rust">${escapeHtml(RUST_TEXT)}</code></pre>
    <pre><code>${escapeHtml(PLAIN_TEXT)}</code></pre>
  </body></html>`;
  const opts = url ? { url } : {};
  return new JSDOM(html, opts);
}

// pasta / rust / plain の各 <code> を取り出すヘルパ。
function pick(doc) {
  return {
    pasta: doc.querySelector('code.language-pasta'),
    rust: doc.querySelector('code.language-rust'),
    plain: doc.querySelector('pre > code:not([class])'),
    headerCode: doc.querySelector('.header code'),
  };
}

// ポーリングフォールバック検証用の待機ヘルパ（ケース J）。
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function run() {
  log('ClientNeutralizer (neutralizer.mjs) jsdom unit test');
  log('');

  // =========================================================================
  // ケース A: highlightBlock 経路（旧名・book.js が実際に呼ぶ API）。
  //   3.1 事前 span 生存 / 3.2 他言語委譲を検証。
  // =========================================================================
  {
    const dom = buildDom();
    const { window } = dom;
    const doc = window.document;
    const el = pick(doc);

    // 事前条件: pasta ブロックは焼き込み span を持つ（中和前のベースライン）。
    check(
      'A-前提: pasta ブロックが事前 hljs span を内包する',
      el.pasta.querySelectorAll('span.hljs-keyword, span.hljs-title').length === 2,
      el.pasta.innerHTML,
    );
    const pastaTextBefore = el.pasta.textContent;
    const rustHtmlBefore = el.rust.innerHTML;

    // 中和を window.hljs 代入「前」に仕込む（head.hbs 発火順序の模擬）。
    installHljsNeutralizer(window);

    // highlight-*.js が window.hljs を代入する瞬間にラップが効く。
    const fake = makeFakeHljs();
    window.hljs = fake;

    // book.js codeSnippets 模倣（全 code へ無条件 highlightBlock）。
    const targets = runCodeSnippets(doc, window, /* useElementApi */ false);

    // ヘッダ内 inline code は対象外（book.js の挙動模倣の健全性確認）。
    check(
      'A: ヘッダ内 inline code は codeSnippets 対象から除外される',
      !targets.includes(el.headerCode),
    );

    // --- 3.1: pasta ブロックの事前 span が生存する ---
    check(
      '3.1: pasta ブロックの事前 hljs span が中和後も残存する',
      el.pasta.querySelectorAll('span.hljs-keyword, span.hljs-title').length === 2,
      el.pasta.innerHTML,
    );
    check(
      '3.1: pasta ブロックの innerHTML が破壊されていない（テキストも保持）',
      el.pasta.querySelector('span.hljs-keyword') &&
        el.pasta.querySelector('span.hljs-keyword').textContent === '＊' &&
        el.pasta.textContent === pastaTextBefore,
      el.pasta.innerHTML,
    );
    // 原 highlightBlock が pasta 要素に対し呼ばれていない（spy count 0）。
    check(
      '3.1: 原 highlightBlock が pasta 要素に対し呼ばれていない（spy=0）',
      fake.spy.block.every((cls) => !/\blanguage-pasta\b/.test(cls)),
      `spy.block=${JSON.stringify(fake.spy.block)}`,
    );

    // --- 3.2: 非 pasta ブロックは原処理へ委譲される（既存挙動不変） ---
    check(
      '3.2: rust 要素に対し原 highlightBlock が呼ばれている',
      fake.spy.block.some((cls) => /\blanguage-rust\b/.test(cls)),
      `spy.block=${JSON.stringify(fake.spy.block)}`,
    );
    // 原処理どおり innerHTML が（模擬）再生成されている（textContent から作り直された）。
    check(
      '3.2: rust 要素の innerHTML が原処理で再生成されている（既存挙動）',
      el.rust.innerHTML === escapeHtml(el.rust.textContent) &&
        el.rust.innerHTML === rustHtmlBefore, // RUST_TEXT は元から escape 済みなので等価
      el.rust.innerHTML,
    );
    check(
      '3.2: rust 要素の textContent が保持されている（委譲後も内容不変）',
      el.rust.textContent === RUST_TEXT,
      el.rust.textContent,
    );
    // 無指定ブロックも委譲側（pasta でないため原処理が走る）。
    check(
      '3.2: 無指定ブロックにも原 highlightBlock が委譲される',
      fake.spy.block.some((cls) => cls === ''),
      `spy.block=${JSON.stringify(fake.spy.block)}`,
    );

    dom.window.close();
  }
  log('');

  // =========================================================================
  // ケース B: highlightElement 経路（新名・将来の hljs API）。
  //   同様に pasta スキップ・非 pasta 委譲を検証。
  // =========================================================================
  {
    const dom = buildDom();
    const { window } = dom;
    const doc = window.document;
    const el = pick(doc);

    installHljsNeutralizer(window);
    const fake = makeFakeHljs();
    window.hljs = fake;

    runCodeSnippets(doc, window, /* useElementApi */ true);

    check(
      '3.1(element経路): pasta ブロックの事前 span が残存する',
      el.pasta.querySelectorAll('span.hljs-keyword, span.hljs-title').length === 2,
      el.pasta.innerHTML,
    );
    check(
      '3.1(element経路): 原 highlightElement が pasta 要素に対し呼ばれていない',
      fake.spy.element.every((cls) => !/\blanguage-pasta\b/.test(cls)),
      `spy.element=${JSON.stringify(fake.spy.element)}`,
    );
    check(
      '3.2(element経路): rust 要素に対し原 highlightElement が呼ばれ再生成される',
      fake.spy.element.some((cls) => /\blanguage-rust\b/.test(cls)) &&
        el.rust.innerHTML === escapeHtml(el.rust.textContent),
      `spy.element=${JSON.stringify(fake.spy.element)} html=${el.rust.innerHTML}`,
    );

    dom.window.close();
  }
  log('');

  // =========================================================================
  // ケース C: file:// 相当 URL でも同様に動作する（要件 4.2 オフライン閲覧の整合）。
  // =========================================================================
  {
    const dom = buildDom('file:///C:/home/maz/git/pasta/book/book/index.html');
    const { window } = dom;
    const doc = window.document;
    const el = pick(doc);

    installHljsNeutralizer(window);
    const fake = makeFakeHljs();
    window.hljs = fake;
    runCodeSnippets(doc, window, false);

    check(
      'C(file://): pasta ブロックの事前 span が残存する',
      el.pasta.querySelectorAll('span.hljs-keyword, span.hljs-title').length === 2,
      el.pasta.innerHTML,
    );
    check(
      'C(file://): rust 要素は原処理へ委譲され再生成される',
      fake.spy.block.some((cls) => /\blanguage-rust\b/.test(cls)),
      `spy.block=${JSON.stringify(fake.spy.block)}`,
    );

    dom.window.close();
  }
  log('');

  // =========================================================================
  // ケース D: 二重 install しても二重ラップしない（冪等・neutralizer.mjs の __pastaNeutralized）。
  //   pasta スキップは保たれ、非 pasta は「ちょうど一度だけ」委譲される。
  // =========================================================================
  {
    const dom = buildDom();
    const { window } = dom;
    const doc = window.document;
    const el = pick(doc);

    installHljsNeutralizer(window);
    installHljsNeutralizer(window); // 二度目（二重ラップしないこと）。
    const fake = makeFakeHljs();
    window.hljs = fake;

    runCodeSnippets(doc, window, false);

    check(
      'D(冪等): 二重 install 後も pasta 事前 span が残存する',
      el.pasta.querySelectorAll('span.hljs-keyword, span.hljs-title').length === 2,
      el.pasta.innerHTML,
    );
    // 非 pasta（rust）への原 highlightBlock 呼び出しは「ちょうど 1 回」（二重ラップなら 0 回や
    // 再帰で挙動が崩れる）。codeSnippets ループ自体は rust を 1 回しか回さないため、
    // spy に rust が 1 件だけ記録されることで二重ラップが無いことを示す。
    const rustCalls = fake.spy.block.filter((cls) => /\blanguage-rust\b/.test(cls)).length;
    check(
      'D(冪等): rust への原 highlightBlock 呼び出しがちょうど 1 回',
      rustCalls === 1,
      `rustCalls=${rustCalls} spy.block=${JSON.stringify(fake.spy.block)}`,
    );

    dom.window.close();
  }
  log('');

  // =========================================================================
  // ケース E: install 前に window.hljs が既に存在する（読み込み順逆転）。
  //   neutralizer.mjs L91-93 の即時 neutralize 分岐。highlight-*.js が先に
  //   読み込まれても安全側に倒れることを検証する。
  // =========================================================================
  {
    const dom = buildDom();
    const { window } = dom;
    const doc = window.document;
    const el = pick(doc);

    // 先に hljs を代入してから install する（既存テストと逆順）。
    const fake = makeFakeHljs();
    window.hljs = fake;
    installHljsNeutralizer(window);

    runCodeSnippets(doc, window, false);

    check(
      'E(既存hljs): install 前に hljs が存在しても pasta 事前 span が残存する',
      el.pasta.querySelectorAll('span.hljs-keyword, span.hljs-title').length === 2,
      el.pasta.innerHTML,
    );
    check(
      'E(既存hljs): rust 要素は原処理へ委譲される',
      fake.spy.block.some((cls) => /\blanguage-rust\b/.test(cls)),
      `spy.block=${JSON.stringify(fake.spy.block)}`,
    );

    dom.window.close();
  }
  log('');

  // =========================================================================
  // ケース F: win が null/undefined でも例外を出さない（早期 return）。
  // =========================================================================
  {
    let threw = false;
    try {
      installHljsNeutralizer(null);
      installHljsNeutralizer(undefined);
    } catch (e) {
      threw = true;
    }
    check('F(nullガード): win=null/undefined で例外を出さない', !threw);
  }
  log('');

  // =========================================================================
  // ケース G: classList の無い環境（プレーンオブジェクト win + className 文字列
  //   fallback）。neutralizer.mjs は DOM API 非依存（classList/className 参照のみ）
  //   を謳うため、jsdom なしで検証する。
  // =========================================================================
  {
    const win = {};
    installHljsNeutralizer(win);
    const fake = makeFakeHljs();
    win.hljs = fake;

    // className 文字列のみを持つ偽要素（classList なし）。
    const pastaEl = { className: 'language-pasta hljs', textContent: 'x', innerHTML: 'x' };
    const rustEl = { className: 'language-rust', textContent: 'y', innerHTML: 'y' };
    const noClassEl = { textContent: 'z', innerHTML: 'z' }; // className も classList も無い

    win.hljs.highlightBlock(pastaEl);
    win.hljs.highlightBlock(rustEl);
    win.hljs.highlightBlock(noClassEl);

    check(
      'G(className fallback): language-pasta（className 文字列判定）はスキップされる',
      fake.spy.block.every((cls) => !/\blanguage-pasta\b/.test(cls)),
      `spy.block=${JSON.stringify(fake.spy.block)}`,
    );
    check(
      'G(className fallback): 非 pasta（rust/クラス無し）は委譲される',
      fake.spy.block.length === 2,
      `spy.block=${JSON.stringify(fake.spy.block)}`,
    );
    // 部分一致の誤判定がない: language-pastafoo はスキップされない。
    const fake2 = makeFakeHljs();
    win.hljs = fake2; // setter 再発火 → 新オブジェクトも neutralize される
    win.hljs.highlightBlock({ className: 'language-pastafoo', textContent: 'w', innerHTML: 'w' });
    check(
      'G(className fallback): language-pastafoo（部分一致）はスキップされず委譲される',
      fake2.spy.block.length === 1,
      `spy.block=${JSON.stringify(fake2.spy.block)}`,
    );
    check(
      'G(再代入): win.hljs 再代入でも setter が新オブジェクトを neutralize する',
      fake2.highlightBlock.__pastaNeutralized === true,
    );
  }
  log('');

  // =========================================================================
  // ケース H: API が片方しか無い hljs（highlightBlock のみ / highlightElement のみ /
  //   どちらも無い / null）でも例外を出さず、存在する側だけラップする。
  // =========================================================================
  {
    const win = {};
    installHljsNeutralizer(win);

    let threw = false;
    try {
      win.hljs = null; // neutralize(!hljs) ガード
      win.hljs = {}; // メソッド無し
    } catch (e) {
      threw = true;
    }
    check('H(部分API): hljs=null/メソッド無しオブジェクトでも例外を出さない', !threw);

    // highlightBlock のみ。
    const win2 = {};
    installHljsNeutralizer(win2);
    const onlyBlock = {
      calls: 0,
      highlightBlock(el) {
        this.calls++;
      },
    };
    win2.hljs = onlyBlock;
    win2.hljs.highlightBlock({ className: 'language-pasta' });
    win2.hljs.highlightBlock({ className: 'language-rust' });
    check(
      'H(部分API): highlightBlock のみの hljs でも pasta スキップ・非 pasta 委譲が機能する',
      onlyBlock.calls === 1 && typeof win2.hljs.highlightElement === 'undefined',
      `calls=${onlyBlock.calls}`,
    );

    // highlightElement のみ。
    const win3 = {};
    installHljsNeutralizer(win3);
    const onlyElement = {
      calls: 0,
      highlightElement(el) {
        this.calls++;
      },
    };
    win3.hljs = onlyElement;
    win3.hljs.highlightElement({ className: 'language-pasta' });
    win3.hljs.highlightElement({ className: 'language-rust' });
    check(
      'H(部分API): highlightElement のみの hljs でも pasta スキップ・非 pasta 委譲が機能する',
      onlyElement.calls === 1,
      `calls=${onlyElement.calls}`,
    );
  }
  log('');

  // =========================================================================
  // ケース I: 拡張不可（preventExtensions）の hljs — マーカー __pastaNeutralized を
  //   置けなくても try/catch で吸収し、ラップ自体は機能する（L82-86）。
  // =========================================================================
  {
    const win = {};
    installHljsNeutralizer(win);
    const spy = { calls: [] };
    const rigid = {
      highlightBlock(el) {
        spy.calls.push(el.className);
      },
    };
    Object.preventExtensions(rigid); // 既存プロパティは書換可・新規追加は不可

    let threw = false;
    try {
      win.hljs = rigid; // neutralize 中のマーカー追加が strict mode で TypeError → catch
    } catch (e) {
      threw = true;
    }
    check('I(拡張不可): preventExtensions された hljs でも例外を出さない', !threw);
    win.hljs.highlightBlock({ className: 'language-pasta' });
    win.hljs.highlightBlock({ className: 'language-rust' });
    check(
      'I(拡張不可): マーカー無しでもラップは機能する（pasta スキップ・rust 委譲）',
      spy.calls.length === 1 && /\blanguage-rust\b/.test(spy.calls[0]),
      `calls=${JSON.stringify(spy.calls)}`,
    );
  }
  log('');

  // =========================================================================
  // ケース J: defineProperty 不可な win → ポーリングフォールバック（L108-117）。
  //   'hljs' を非 configurable な書込可データプロパティとして事前定義すると、
  //   アクセサへの再定義が TypeError となり setInterval ポーリングへ落ちる。
  //   その後 hljs を代入し、25ms 周期のポーリングが検出・neutralize することを待つ。
  // =========================================================================
  {
    const win = {};
    Object.defineProperty(win, 'hljs', {
      configurable: false, // defineProperty(アクセサ化)を不可にする
      writable: true,
      enumerable: true,
      value: undefined,
    });

    let threw = false;
    try {
      installHljsNeutralizer(win); // defineProperty が TypeError → ポーリング開始
    } catch (e) {
      threw = true;
    }
    check('J(ポーリング): defineProperty 不可でも install が例外を出さない', !threw);

    const fake = makeFakeHljs();
    win.hljs = fake; // データプロパティへの素の代入（setter なし）

    // ポーリング周期 25ms を確実に跨ぐまで待機（余裕を持って最大 ~500ms リトライ）。
    let wrapped = false;
    for (let i = 0; i < 20; i++) {
      await sleep(25);
      if (win.hljs.highlightBlock && win.hljs.highlightBlock.__pastaNeutralized) {
        wrapped = true;
        break;
      }
    }
    check('J(ポーリング): ポーリングが hljs 出現を検出してラップする', wrapped);

    win.hljs.highlightBlock({ className: 'language-pasta' });
    win.hljs.highlightBlock({ className: 'language-rust' });
    check(
      'J(ポーリング): ラップ後は pasta スキップ・非 pasta 委譲が機能する',
      fake.spy.block.length === 1 && /\blanguage-rust\b/.test(fake.spy.block[0]),
      `spy.block=${JSON.stringify(fake.spy.block)}`,
    );
  }
  log('');

  log(`RESULT: ${passed} passed, ${failed} failed`);
  // tokenizer-test.mjs と同様、jsdom の async ハンドル close 中の libuv assertion を避けるため
  // process.exit() を即時に呼ばず exitCode を設定してイベントループを自然に drain させる。
  process.exitCode = failed === 0 ? 0 : 1;
}

run().catch((e) => {
  console.error('UNEXPECTED ERROR:', e && e.stack ? e.stack : e);
  process.exitCode = 1;
});
