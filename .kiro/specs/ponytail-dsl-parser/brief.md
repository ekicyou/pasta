# Brief: ponytail-dsl-parser

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。版には縛らない。0.4.2 を出した後、0.5.0 の spec が並べられずに空いた所へ入れる（2026-10-22 開発者の決定。2026-10-19 の「0.4.3 に入れる」から改めた）。同じ監査から起こした 10 本（roadmap の「ponytail 監査の指摘の修正（隙間の spec）」）と同じウェーブで並走できる。

## 進め方（2026-10-19 開発者の決定）

- 下の「Current State」は、2026-10-19 の監査の控え（出発点）である。着手するまでに、0.4.2 の直しでコードが動いている。
- **ギャップ分析**（`/kiro-validate-gap`）で、この spec の領域に `/ponytail:ponytail-audit`（プラグインのスキル）をやり直す。控えと突き合わせ、消えた指摘・新しい指摘・動いた行番号を、ギャップ分析の報告に書く。
- 要件は「その領域の監査の指摘が、残らず直っている」ことと、挙動を変える直しの決めごとにする。
- **修正は設計以降**（設計 → タスク → 実装）で行う。

## Problem

ゴーストの作者と、辞書を読むすべての道具（pasta.dll・`pasta_check`・VSCode 拡張）が困っている。

- 大きい辞書ほど、読み込みが 2 乗で遅くなる。1 シーン 4 行の辞書を release で測ると、次のとおりだった。
  - 2,000 行で 0.62 秒
  - 8,000 行で 10.6 秒
  - 32,000 行で 151 秒
- 型に入らない大きな数を書くと、黙って誤った値になる。誤り方は書いた場所ごとに違い、pasta.dll も `pasta_check` も何も言わない。
- 括弧を深く入れ子にした辞書（1000 段）は、pest の再帰でスタックがあふれ、SSP ごと落ちる。`review-improvement-loop` の 3.4 が申し送ったまま、まだ直っていない。

## Current State

2026-10-19 の監査で確かめた。不具合は、スクラッチの probe クレートで再現した。

1. **解析が 2 乗に遅い**（Must fix）
   - 場所：`crates/pasta_dsl/src/parser/ast/span.rs` L120-137 の `impl From<&pest::Span> for Span`。`parse_scene.rs`・`parse_action.rs`・`parse_elements.rs` の 18 か所から呼ばれる。
   - 原因：節ごとに `pest::Position::line_col()` を 2 回呼び、それが入力の先頭から数え直す。
   - 8,000 行の内訳：pest 自体は 45 ms で、`Position::line_col` が 13.5 秒かかる。`Pair::line_col()`（pest の行の索引を使う）なら 45 ms で済む。
2. **大きな数の字句が化ける**（Should fix）
   - 場所と、今の扱い：
     - `parse_action.rs` L351-358（`try_parse_expr`）：`parse().ok()?` が項を落とす。
     - `parse_elements.rs` L40-47（属性）：`unwrap_or(0)`。
     - `parse_scene.rs` L404-410（キュー）：`unwrap_or(0)`。
     - `parse_scene.rs` L160-163（アクターの番号）：`.ok()` で自動の番号へ戻る。
   - 実例：
     - `＄x＝10000000000000000000` → x = `""`。
     - `＄y＝1＋10000000000000000000` → y = 1（演算子が zip で消える）。
     - `＞t（10000000000000000000、2）` → 引数が 1 つになり、`＄０` が 2 にずれる。
     - `!wait(99999999999999999999)` → `Integer(0)`。
     - `％a、b＝4294967296` → 番号が [0, 1]。
   - マニュアルの `book/src/grammar/literals.md` は「i64 / f64」とだけ書いている。
3. **作られない誤りの型**（Nice to have）
   - 場所：`crates/pasta_dsl/src/error.rs` L10-71。
   - 作られるのは `SyntaxError` と `From<io::Error>` だけで、次の型と関数は作られない。
     - 型：`PestError`・`MultipleErrors`・`ParseErrorInfo`・`ParseResult`
     - コンストラクタ：`syntax_error`・`pest_error`・`io_error`
4. **`PastaFile.span` が誤って計算され、テストだけが読む**（Nice to have）
   - 場所：`crates/pasta_dsl/src/parser/mod.rs` L163-166。
   - `end_col` が文字数でなくバイト長で、日本語では誤る。読むのは `error_api_test.rs` と `span_byte_offset_test.rs` だけ。
5. **深い括弧の入れ子でスタックがあふれる**
   - 場所：`.kiro/specs/review-improvement-loop/matrix.md` の 3.4 の申し送り。
   - 深さ 100 は大丈夫で、1000 で `STATUS_STACK_OVERFLOW` になる。pest 2.x には深さを制限する API が無い。

## Desired Outcome

- 解析の時間が、辞書の大きさに比例する。8,000 行が 1 秒未満で済み、結果（span の値）は今と同じ。
- 大きな数の字句は、どこに書いても同じ規則で値になる。i64 に入らなければ f64（Lua と同じ）。引数や演算子は消えない。
- 深い入れ子は、プロセスを落とさず、位置つきの構文の誤りになる。
- 作られない誤りの型と、誤った `PastaFile.span` が無くなる。

## Approach

- 監査が示した最小の直しを採る。
  - `Span::from_pair(&Pair)` を足す。始まりは `pair.line_col()` から取り、終わりは `pair.as_str()` の中だけを数える（`'\n'` の数と、最後の `'\n'` の後の文字数＋1。pest の決まりに合わせる）。
  - 18 か所を置き換え、`From<&pest::Span>` を消す。
  - 別案は、`parse_str` で行頭の索引を 1 つ作って二分探索すること。
- 数は、`parse_elements.rs` の 1 つの補助 `number_value(s) -> Expr` に集める（i64 を試し、だめなら f64）。4 か所でそれを使う。アクターの番号は、`u32::MAX` に飽和させるか構文の誤りにするかを、要件で決める。
- 深さの守りは、`parse_str` の前に括弧の深さを数えるだけの走査を置く。正しい辞書を誤検知しない上限は、要件で決める。

## Scope

- **In**: 上の 1〜5、回帰テスト、`book/src/grammar/literals.md` の大きい整数の 1 行。
- **Out**:
  - 部分解析（`crates/pasta_dsl/src/partial.rs`）。これは `ponytail-editor-tooling` が持つ。
  - pasta_core の型。これは `ponytail-core-registry` が持つ。

## Boundary Candidates

- span の計算（`ast/span.rs` と 18 か所の呼び出し）
- 数の字句の値（`parse_elements.rs` の補助と 4 か所）
- 入れ子の深さの守り（`parse_str` の入口）
- 誤りの型の整理（`error.rs`・`lib.rs` の再輸出）

## Out of Boundary

- `partial.rs` とその誤りの出し方（`ponytail-editor-tooling`）
- 文法（`.pest`）の意味の変更
- `review-improvement-loop` の 3.3 の既知のバグ（`paren_expr` の第 1 項）。拾うかは要件で決める。

## Upstream / Downstream

- **Upstream**: pest 2.x
- **Downstream**:
  - `pasta_lua` のトランスパイラ（span を使ってソースの対応表を作る）
  - `pasta_check`
  - `pasta_lsp`（`ponytail-editor-tooling`）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-editor-tooling`（`partial.rs` を持つ。`partial.rs` は今、`error.rs` の型も `Span::from` も使っていない。2026-10-19 に grep で確かめた）
  - `talk-grapheme-clusters`（ファイルは重ならない）

## Constraints

- 公開クレートの pub API を消してよい（2026-10-19 開発者の決定）。`README` と `.kiro/steering/tech.md` に「pasta の部品で、外部から使う安定した面ではない」と書くのは、10 本のどれか 1 本で 1 回だけ行う。この spec が先に入るなら、この spec が書く。
- 挙動を変える直しは、同じ変更でマニュアルを直し、`node book/tools/gen-skill-refs.mjs` で references を作り直す。
- リファクタリングは安全で、戻せる形にする。振る舞いを固めるテストを先に書き、1 つ直すごとに確かめて 1 コミットにする。
- 品質の関門：`cargo fmt --check`、`cargo clippy --all-targets`（警告 0）、`cargo test -p pasta_dsl -p pasta_lua -p pasta_check -p pasta_lsp`。
- cargo の前に、環境変数 `NoDefaultCurrentDirectoryInExePath` を外す。外さないと、LuaJIT のビルドが exit 101 で落ちる。
