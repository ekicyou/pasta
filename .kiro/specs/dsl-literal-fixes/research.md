# ギャップ分析: dsl-literal-fixes

## Summary
- **Feature**: `dsl-literal-fixes`
- **Discovery Scope**: Extension（既存の文法・パーサ・文字列出力の局所修正）
- **分析日**: 2026-10-04（現行 main `8033b4d7` 相当のワークツリー）
- **Key Findings**:
  - 3 件とも原因は brief.md の指摘どおりで、修正点は `grammar.pest` の 2 規則・`parse_elements.rs` の 2 箇所・`string_literalizer.rs` の 1 関数に収まる。新しい部品は要らない。
  - U24 は「改行をまたぐ」と「`""` が囲みの開始として先に試される」の 2 つが重なって起きる。`string_contents` から改行を除くと、別々の行の `""` も同時に直る。同じ行の `""…""` の曖昧さは残る（PEG の順序付き選択の性質）。
  - 既存のテスト・スナップショット・サンプルゴースト・フィクスチャ（`.pasta` 39 ファイル）に、変更で期待値が変わるものは無い。非互換の実害は、現行のリポジトリ内では見つからない。
  - VSCode 拡張のハイライト文法は、引用文字列を（キューコマンドの引数以外で）色分けしておらず、全規則が 1 行の `match` である。要件 7.1 は現状で満たしている。
  - 改行入りの値を Lua の長い文字列 `[[…]]` で出すと、Lua が CR・CRLF を LF に正規化し、先頭の改行を捨てる。値を変えずに出す（要件 5.1）には、`"…"` 形式でエスケープする方式が合う。

## 現状の再現（パーサ出力で確認）

スクラッチの probe（`pasta_dsl::parse_str` を直接呼ぶ）で、現行の AST を確認した。

| 入力 | 現行の結果 | 要件 |
| ---- | ---------- | ---- |
| `＠w：あ、い ＃c` | 候補 `あ`・`い ＃c` | 4.1 |
| `＠w：「」、""` | 候補 `「」`・`""`（2 文字ずつ） | 3.1 |
| `＆k：「」` | `AttrValue::String("「」")` | 3.2 |
| `＠a：""`＋改行＋`＠b：""` | 単語 `a` の候補 `\n＠b：` の 1 つだけ。単語 `b` は消える | 2.1・2.2 |
| `＠w：""、""`（同じ行） | 候補 `、` の 1 つ | 2.3 |
| `＄x＝「a`＋改行＋`b」` | `Expr::String("a\nb")`。パースは成功する | 1.1〜1.3 |
| `＠w：「abc`（閉じ無し・1 行） | 候補 `「abc`（引用なしの値として読む） | 範囲外（現行維持） |
| `＠w：あ、い   `（行末の空白） | 候補 `い   `（空白は残る） | 4.6（現行維持） |

U24 の失敗は、パースもトランスパイルも通ったあと、`crates/pasta_lua/src/runtime/factory.rs:222` の `require_startup_module(lua, "pasta.scene_dic")` で起きる。`scene_dic.lua` は各モジュールを `require` で並べるだけ（`crates/pasta_lua/src/loader/cache.rs:336-348`）なので、1 モジュールの構文エラーでロード全体が止まる。SHIORI は `500` と `X-ERROR-REASON` を返す（`crates/pasta_shiori/src/error.rs:72-119`）。

パースエラーは `ParseError::SyntaxError{file,line,column,message}` として行・桁を持つ（`crates/pasta_dsl/src/error.rs:14-35`、`crates/pasta_dsl/src/parser/mod.rs:97-114`）。ローダはファイルごとに `error!` でログに出し、`LoaderError::PartialTranspileError` で返す（`crates/pasta_lua/src/loader/process.rs:151-173,254-267`）。要件 1.2 の「行番号付きのパースエラー」は、この既存の経路で満たせる。

## Requirement-to-Asset Map

| 要件 | 関係する資産 | ギャップ | 種別 |
| ---- | ------------ | -------- | ---- |
| 1.1〜1.3 引用は 1 行で閉じる | `grammar.pest:123` `string_contents = @{ (!PEEK ~ ANY)+ }` | 改行（`\r`・`\n`）を除外していない | Missing |
| 1.4 全文脈に適用 | `string_literal` の利用箇所: `term`（式）・`key_literal`・`word`・`attr_value`・`cue_arg`。`choice_label`（`:220`）は `string_contents` を直接使う | `string_contents` 1 箇所の修正で全文脈に効く | Constraint（好都合） |
| 1.2 行番号付きエラー | `ParseError::SyntaxError`・ローダのログ | 既存で足りる。エラーの行が「開始行」か「次の行」かは文脈で変わる（下記 Research Needed） | Unknown |
| 2.1・2.2 別々の行の `""` | `grammar.pest:118,138` `string_literal = string_fenced \| string_blank`、`slfence_en = PUSH("\""+)` | 1.x の修正の結果として直る（`""` の後ろが行内で閉じないと囲みの分岐が失敗し、`string_blank` に落ちる） | Missing（1.x に従属） |
| 2.3 同じ行の `""…""` | 同上 | 可変長の囲みと本質的に曖昧。現行の解釈を仕様として注記する（Q2 で確定） | Constraint |
| 3.1 単語値の `「」`・`""` | `parse_elements.rs:64-66`（`Rule::string_blank` の `as_str()`） | 空文字列にしていない | Missing |
| 3.2 属性値の `「」`・`""` | `parse_elements.rs:21-23`（`parse_attr`） | 同上 | Missing |
| 3.3 文脈で同じ | 式 `parse_action.rs:359`（`Expr::BlankString`）、キュー引数 `parse_scene.rs:415-417`（`String::new()`） | 単語値・属性値だけが外れている | Missing |
| 3.4・3.5 空の候補の実行時 | `word.lua:29-36`（`entry` は値を素通し）、`act.lua:290-293`・`actor.lua:143-144`（`result ~= nil` で判定。`""` は見つかった扱い）、`pasta_core` の `word_table` | 追加の変更は不要と見込む。空の候補の実行時テストは無い | Unknown（テストで確認） |
| 4.1〜4.3 行末コメント | `grammar.pest:154` `word_nofenced = @{ (!(comma_sep \| "\r" \| "\n") ~ ANY)+ }` | `comment_marker` を除外していない。前例は `attr_string`（`:165`） | Missing |
| 4.2 コメント前の空白 | 同上・`or_comment_eol`（`:203`） | `word_nofenced` が貪欲に空白を取り込む。コメント前の空白を除くには、否定先読みを `s ~ comment_marker` にする必要がある | Missing（Q5 で確定） |
| 4.5 さくらスクリプトだけの値 | `word = string_literal \| sakura_script \| word_nofenced`、`sakura_body`（`:191`） | 現行どおり（`\` で始まる値は `sakura_script` が先に取る）。変更不要 | — |
| 4.6 行末の空白（コメント無し） | `word_nofenced` | 現行どおり。変更不要 | — |
| 4.7 3 種の単語定義 | `file_word_line`・`global_scene_word_line`（アクター配下も同じ規則）が同じ `key_words`/`words` を使う | `word_nofenced` 1 箇所の修正で全種に効く | Constraint（好都合） |
| 5.1〜5.3 生成 Lua | `string_literalizer.rs:38-60` `needs_long_string` は `\`・`"` だけを見る | 改行入りの値が `"…"` のまま出る。呼び出し元は `scope_gen.rs:59,370-371`・`element_gen.rs` の各所（`element_gen.rs` は触らない） | Missing |
| 6.x マニュアル | `book/src/grammar/literals.md:34-67`、`words.md:44-53`、`block-structure.md:275-283` | 改行禁止・空文字列・同じ行の `""` の注記・単語値の行末コメントが未記載。`block-structure.md:280` は行末コメントを「値を囲んだ単語定義」に限っている | Missing |
| 6.5 スキル references | `book/tools/gen-skill-refs.mjs:38,40,44`（3 章とも生成対象） | 再生成と `--check` | Missing（手順のみ） |
| 7.1・7.2 ハイライト | `editors/vscode/syntaxes/pasta.tmLanguage.json`（マニュアルは `book/tools/highlight/highlight-html.mjs:39` で再利用） | 引用文字列の色分けはキュー引数（`:256-259`、1 行の正規表現）だけ。全規則が 1 行の `match`。変更不要 | — |
| 7.3 マニュアルの例 | `book/tools/highlight/tokenizer-test.mjs`・`scope-map-test.mjs` | 足す例がハイライトのテストを壊さないことの確認だけ | Constraint |
| 8.x テスト | `crates/pasta_dsl/tests/`（`ast_test.rs`・`choice_line_test.rs` など）、`crates/pasta_lua/src/string_literalizer.rs` の単体テスト、`crates/pasta_lua/tests/transpiler/`・`tests/runtime/` | 空の単語値・行末コメント付きの単語行・改行入りの文字列・複数の `""` のテストが無い | Missing |

### 付随して見つかったこと（範囲外・記録のみ）
- LSP の単語トークン（`crates/pasta_lsp/src/analysis/visit_scope.rs:103-107`）は `KeyWords.span` から作られる。U06 の修正で、取り込まれていた `＃…` の分だけトークンが短くなる（望ましい変化。テストは無い）。
- `sakura_body`（`grammar.pest:191`）も改行をまたぐ（`\s[`＋改行＋`0]`）。値は `\` で始まるため長い文字列 `[=[…]=]` で出力され、Lua の構文エラーにはならない。R4（保留）の領域なので触らない。
- 閉じていない 1 行の引用（`＠w：「abc`）は引用なしの値 `「abc` になる。属性値でも同じ（`attr_string`）。現行挙動として残す（要件の範囲外に明記済み）。

## 実装アプローチの選択肢

### U24（文法）

| 案 | 内容 | 長所 | 短所 |
| -- | ---- | ---- | ---- |
| A1（推奨） | `string_contents = @{ (!(PEEK \| "\r" \| "\n") ~ ANY)+ }` | 1 行の修正。`choice_label` も含め全文脈に効く。別々の行の `""` も同時に直る | 同じ行の `""…""` の曖昧さは残る（Q2 で注記に確定） |
| A2 | `string_literal = string_blank \| string_fenced` に並べ替え | — | PEG の順序付き選択は戻らないため `""abc""`（2 重の囲み）が壊れる。不可 |
| A3 | `string_blank` を「`""` の直後が区切り（読点・`）`・空白・行末・コメント）」のときだけ優先する先読み | 同じ行の `""、""` も 2 つの空文字列になる | 「2 重の囲みの中身が読点で始まる」書き方（`""、x""`）と衝突する。規則が増えてマニュアルの説明が難しくなる（仮定 Q2 の別案） |
| A4 | 改行を許可し、値に改行を含める | 複数行の台詞データを単語値に書ける | 閉じ忘れが遠くの引用まで黙って取り込む問題が残る。別々の行の `""` の問題は別途 A3 相当が要る（仮定 Q1 の別案） |

### U24（出力の多層防御）

| 案 | 内容 | 長所 | 短所 |
| -- | ---- | ---- | ---- |
| B1（推奨） | 値が `\r`・`\n` を含むときは `"…"` 形式で `\\`・`"`・`\r`・`\n` をエスケープして出す（含まないときは現行の規則のまま） | 値を変えずに読み戻せる（要件 5.1）。改行を含まない値の出力は不変（5.3）。既存スナップショットに影響しない | 規則が 1 つ増える |
| B2 | 値が改行を含むときは長い文字列 `[[…]]` にし、先頭に改行を 1 つ足す | 既存の長い文字列の仕組みを使える | Lua が長い文字列の中の CR・CRLF を LF に正規化するため、CR を含む値が変わる（5.1 に反する） |
| B3 | 何もしない（A1 で改行が来なくなる） | 変更が最小 | brief の多層防御を満たさない。将来の別経路（さくらスクリプトの引数など）で同じ事故が起き得る |

### U25（パーサ）
- C1（推奨）: `parse_elements.rs` の単語値（64-66 行）と `parse_attr`（21-23 行）で、`Rule::string_blank` を `String::new()` にする。キュー引数（`parse_scene.rs:415-417`）と同じ形。A1 が前提（直さないと 2 つ目の `""` が囲みとして消費される）。
- C2: 空の候補を許さず、パースエラーまたは警告にする（仮定 Q3 の別案）。式・キュー引数では空文字列を許しているため、文脈で規則が割れる。

### U06（文法）

| 案 | 内容 | 長所 | 短所 |
| -- | ---- | ---- | ---- |
| D1 | `word_nofenced` の否定先読みに `comment_marker` を足す | `attr_string` と同じ形。1 行 | `い ＃c` の値が `い `（空白付き）になる（要件 4.2 を満たさない） |
| D2（推奨） | 否定先読みに `s ~ comment_marker` を足す | コメント前の空白も値から外れる（4.2）。コメントが無いときの行末の空白は現行どおり（4.6） | D1 より少し読みにくい |
| D3 | `#` は直前が空白のときだけコメントにする | `a#b`・`赤い\f[color,#ff0000]` を引用なしで書ける | 属性値の規則（`#` を含められない）と食い違う。規範「`#` から行末まではコメント」と合わない（仮定 Q4 の別案） |

### 全体の組み立て
- 推奨は **A1 + B1 + C1 + D2**（すべて既存部品の拡張。Option A「既存の拡張」に当たる）。新しいファイル・新しい部品は要らない。
- 編集するソースは brief の並走条件の範囲（`crates/pasta_dsl/src/parser/grammar.pest`・`parse_elements.rs`、`crates/pasta_lua/src/string_literalizer.rs`）に収まる。`element_gen.rs`・`act.lua` は触らない。
- マニュアル 3 章の更新と `node book/tools/gen-skill-refs.mjs` での再生成が同じ変更に入る。ハイライト文法の変更は不要（Q6 で範囲外に確定）。

## Effort / Risk
- **Effort: S（1〜3 日）** — 文法 2 規則・パーサ 2 箇所・出力 1 関数の局所修正と、テスト・マニュアル 3 章の更新。既存のパターン（`attr_string` の除外、キュー引数の空文字列）をなぞるだけ。
- **Risk: Low** — 影響は文字列リテラルと単語値の解析に閉じる。リポジトリ内の `.pasta` とスナップショットに期待値が変わるものは無い。非互換（引用なしの単語値の `#`、改行入りの引用）は利用者の辞書に影響し得るが、現行でも改行入りの引用はランタイムを壊しており、実質的な利用は無いと見込む。

## Research Needed（設計フェーズへ）
1. 改行をまたぐ引用のパースエラーが、文脈ごとにどの行を指すか（開始行か次の行か）。単語値では、1 行目が引用なしの値 `「a` として通り、2 行目でエラーになる見込み。要件 1.2 の受け入れ基準の書き方（「開始行または次の行」）を設計で確定する。
2. 空の候補が選ばれたときの実行時の挙動（要件 3.4・3.5）を、`pasta_core` の単語テーブルと `act:word`・プロキシの両経路でテストして確かめる。コード上は `result ~= nil` 判定で「見つかった」扱いになる見込み。
3. B1 のエスケープ形式で、さくらスクリプトを含む値（`\` を含む）と改行が同時に来た場合の出力（`\\` の二重化）が正しく読み戻せるか。
4. マニュアルの追加例が `book/tools/highlight` のハイライトテストと `link-check`・`verify-content` を壊さないか。

## 未決事項（要件ディスカッションへ）
- **Q1**（要件 1）改行を含む引用文字列: 禁止（パースエラー）か許可か。仮定は禁止。
- **Q2**（要件 2.3・6.3）同じ行の `""…""`: 現行の可変長の囲みとして注記するか、先読みで空文字列を優先するか。**確定（要件ディスカッション A）: 注記として残す**（brief の Approach に明記済み）。
- **Q3**（要件 3）単語値・属性値の空文字列: 空の候補を許すか、エラー・警告にするか。仮定は許す。
- **Q4**（要件 4.3・6.4）引用なしの単語値の `#`・`＃`: 位置にかかわらずコメントにするか、直前が空白のときだけにするか。仮定は位置にかかわらず（`attr_string` と同じ）。
- **Q5**（要件 4.2）値と行末コメントの間の空白: 値から外すか、行末の空白と同じく残すか。**確定（要件ディスカッション A）: 外す**（属性値の `attr_string` は空白を含まない。コメント前の空白を値とする意図は無い）。
- **Q6**（要件 7・範囲外）単語定義の行・属性行の行末コメントのハイライト: 足すか、範囲外とするか。**確定（要件ディスカッション A）: 範囲外**（単語定義の行は `meta.word.pasta` で行全体 1 色。今回の変更で新たな食い違いは生じない）。
