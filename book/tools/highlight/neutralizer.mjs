// neutralizer.mjs — ClientNeutralizer 正準ソース（タスク 4.1 / 要件 3.1, 3.2, 4.2）。
//
// mdBook の `book.js`（`codeSnippets()` 即時 IIFE）は全 `<code>`（ヘッダ除く）に
// `hljs.highlightBlock(block)` を無条件適用し、未登録言語 `pasta` を plaintext として
// innerHTML を再生成するため、build-time に焼き込んだ事前 span を破壊する（research §8.3）。
//
// 本モジュールは `window` 相当オブジェクトに `Object.defineProperty(win,'hljs',…)` の
// アクセサを仕込み、highlight-*.js が `window.hljs` を代入した瞬間に `highlightBlock`
// （旧名）と `highlightElement`（新名）の両方をラップする。ラップ関数は引数要素が
// `language-pasta` クラスを持つ場合に原処理を呼ばずスキップし（3.1）、それ以外は原処理へ
// そのまま委譲して戻り値も返す（3.2・既存挙動不変）。
//
// ネットワークやサーバには一切依存せず、純粋に win 上のメソッドラップのみを行うため
// `file://` オフライン閲覧でも同様に動作する（4.2）。defineProperty 不可な環境では
// `window.hljs` の出現をポーリングで待ってラップする（既存 elasticlunr 中和と同方式）。
//
// === 正準 ↔ head.hbs ミラー規約 ===
// ブラウザは ESM import 不可ゆえ、`theme/head.hbs` は下記 `installHljsNeutralizer` の
// 関数本体を `export` を除いて逐語ミラーした `<script>(function(){…})();</script>` ブロックを
// インライン同梱する（既存 `book/tools/bigram-index/tokenize.mjs`↔head.hbs と同方式）。
// 本ファイルを変更した場合は head.hbs の中和ブロックも必ず同期すること（関数本体は逐語一致）。

/**
 * win 上に hljs アクセサを仕込み、highlightBlock/highlightElement をラップして
 * language-pasta 要素の再ハイライトをスキップする。
 *
 * 純ロジック（DOM API は要素の classList/className 参照のみ）。win はブラウザの
 * window でも、テスト用のプレーンオブジェクトでもよい。
 *
 * @param {object} win window 相当オブジェクト（hljs プロパティを監視・ラップする対象）
 */
export function installHljsNeutralizer(win) {
  if (!win) {
    return;
  }

  // 引数要素が language-pasta クラスを持つか判定（3.1 のスキップ条件）。
  // classList.contains を優先し、無い環境向けに className 文字列 fallback も用意。
  function isPastaElement(el) {
    if (!el) {
      return false;
    }
    if (el.classList && typeof el.classList.contains === 'function') {
      return el.classList.contains('language-pasta');
    }
    var cls = el.className;
    if (typeof cls === 'string') {
      return (' ' + cls + ' ').indexOf(' language-pasta ') !== -1;
    }
    return false;
  }

  // 原処理を language-pasta スキップ付きでラップする。二重ラップは避ける（冪等）。
  function wrap(fn) {
    if (typeof fn !== 'function' || fn.__pastaNeutralized) {
      return fn;
    }
    var wrapped = function (el) {
      // language-pasta は原処理を呼ばずスキップ（事前 span を破壊しない・3.1）。
      if (isPastaElement(el)) {
        return undefined;
      }
      // それ以外は原処理へそのまま委譲し戻り値も返す（既存挙動不変・3.2）。
      return fn.apply(this, arguments);
    };
    wrapped.__pastaNeutralized = true;
    return wrapped;
  }

  // hljs オブジェクトの highlightBlock / highlightElement を両方ラップする。
  // 既にラップ済みなら何もしない（冪等・二重 wrap 防止）。
  function neutralize(hljs) {
    if (!hljs || hljs.__pastaNeutralized) {
      return;
    }
    if (typeof hljs.highlightBlock === 'function') {
      hljs.highlightBlock = wrap(hljs.highlightBlock);
    }
    if (typeof hljs.highlightElement === 'function') {
      hljs.highlightElement = wrap(hljs.highlightElement);
    }
    try {
      hljs.__pastaNeutralized = true;
    } catch (e) {
      // 凍結オブジェクト等でマーカーを置けなくてもラップ自体は済んでいる。
    }
  }

  // highlight-*.js は <head> 後（body 末尾）で読み込まれ `window.hljs = …` を実行する。
  // そのタイミングを捕捉して確実にラップするため、代入を監視するアクセサを先に仕込む。
  if (Object.prototype.hasOwnProperty.call(win, 'hljs') && win.hljs) {
    // 既に読み込まれていれば即ラップ（読み込み順が変わっても安全側に倒す）。
    neutralize(win.hljs);
  } else {
    try {
      var holder;
      Object.defineProperty(win, 'hljs', {
        configurable: true,
        enumerable: true,
        get: function () {
          return holder;
        },
        set: function (v) {
          holder = v;
          neutralize(v);
        },
      });
    } catch (e) {
      // defineProperty 不可な環境ではポーリングでフォールバック。
      var tries = 0;
      var timer = setInterval(function () {
        if (win.hljs || tries++ > 200) {
          clearInterval(timer);
          neutralize(win.hljs);
        }
      }, 25);
    }
  }
}
