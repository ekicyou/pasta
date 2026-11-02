# ギャップ分析: ponytail-editor-tooling

## Summary

- **Feature**: `ponytail-editor-tooling`
- **Discovery Scope**: Extension（既存の `pasta_lsp`・`partial.rs`・VSCode 拡張の直しと撤去。新しい部品は無い）
- **測った所**: ワークツリーの HEAD `ae04a28b`（main。brief の再測定の `491cc23c` の後は roadmap の 2 コミットだけで、持ち場のソースは同じ）。2026-11-02。
- **Key Findings**:
  - 監査の 1〜13 番は、**すべて残っている**。行番号は brief の 2026-11-02 の再測定のとおり（下の表）。1・2・4・8 は、`AnalysisEngine::analyze` と `parse_str` を一時のテストで走らせて、再現を確かめた（テストのファイルは消した）。
  - 監査のやり直しで、同じ領域に **新しい指摘が 12** 増えた（N1〜N12）。大きいのは 3 つ。
    - brief の箱の正規表現は、`＠＠＠メール`（エスケープの後ろの参照）を取り落とす。行の種類（単語定義・コメント・Lua・キューのスコープ）も見ない（N4）。
    - TextMate のエスケープの規則が文法と違う。`\＠` を色にし、本当のエスケープ `＠＠`・`＄＄` を色にしない（N6）。
    - 行末のコメントは、どの行でも、意味の色にも TextMate の色にもコメントの色が付かない（N2。4 番の広がり）。
  - brief のフェンスの開きの正規表現（`\s*$`）は、文法が受け付けない「言語名の後ろの空白」を受け付ける（N5）。
  - brief の Scope の `editors/vscode/wasm/README.md` は、wasm-pack が `crates/pasta_lsp/README.md` を写す生成物で、git の外・VSIX の外にある。直す物は無い（N12）。
  - 規模は **17〜19 タスク**（下の「タスクの見積もり」）。20 を超えないので、**分けない**のを推奨する。要件ディスカッションで `host`（10 番）を外したので、**16〜18 タスク**になる。

## 監査のやり直し（2026-10-19 の控えとの突き合わせ）

`/ponytail:ponytail-audit` の観点（最小の直し・死んだコード・使い道の無い柔軟性・正しさ）で、`crates/pasta_lsp`・`editors/vscode`（`src`・`syntaxes`・`scripts`・`package.json`）・`crates/pasta_dsl/src/partial.rs`・`crates/pasta_check/src/validate/{syntax,report}.rs` を読み直した。プラグインのスキルは呼ばず、コードを読んで確かめた。「使われない」の判断は、リポジトリ全体の grep による。

再現は、`crates/pasta_lsp/tests/` に一時のテストを置き、`AnalysisEngine::analyze`・`parse_str`・`parse_str_partial` の結果を出して確かめた（終わったら消した。`git status` は clean）。単語参照の正規表現は、スクラッチの node で比べた。cargo test の全体と clippy は走らせていない。

### 控えの指摘の今

| # | 指摘 | 今 | 今の場所・確かめたこと |
| - | ---- | -- | ---------------------- |
| 1 | 1 つの誤りでシーン全体に波線 | 残る | `analysis/mod.rs:66-99`（`Err(_)` の枝。`e.line - 1`・`character: 0`）。`ghostValidator.ts:266`（開いている文書の syntax を隠す）。再現：brief の 5 行で、0 始まりの 1〜4 行目に 4 件、すべて 0 桁目、文は `Parse error: <partial>:1:3: Parse error in <partial> at 1:3:  --> 1:3` ＋枠。CRLF では 2 件、2 つ目のシーンの誤りでは正しい行（`さくら：ね`）にも出る。`parse_str` は正しく `4:4`・`expected id or call_target_expr` を返している |
| 2 | 部分解析が正しい行を誤りにし、本体を落とす | 残る | `partial.rs:99-108`（`trim_start` した行で境界を判定）・`:180-212`（第 3 段で各行をファイルとして解く）。再現 A：誤りのシーンは `local_scenes[0].items` が 0。再現 B：`　％さくら、うにゅう`・`　＠挨拶：…` を含むシーンに誤りがあると、`actors=0 words=0`（誤りが無ければ `actors=2 words=1`）。悪い行だけに誤りが出ることを確かめるテストは無い |
| 3 | 単語参照の箱が `id` と違う | 残る | `wordRefDecorator.ts:6`（`/[＠@][^\s＠@＄$\\！!）)（(\r\n]+/g`）・tmLanguage の `inline-dynamic-ref`（184-191）・`inline-word-ref`（192-195）。node で：`＠挨拶。こんにちは` は全体、`＠＠メール` は `＠メール`、単語定義の行・`＃ ＠メモ`・Lua の `---@param` にも箱。意味の色は正しい（`＠挨拶` の長さ 3、`＠＠＠メール` はエスケープ＋`＠メール`） |
| 4 | 行末のコメントと属性がシーン名の色 | 残る | `visit_scope.rs:179-193`（`marker_line_token` が行末まで）・tmLanguage の `global-scene`（46 行の `(.+)?$`）・`local-scene`（58）・`actor`（118）。再現：`＊OnTest ＃ メモ` は長さ 12 の NAMESPACE 1 つ。`＊挨拶 ＆朝：1` は DECORATOR 無し。**ローカルシーン（`・ローカル ＃ メモ`・`・ローカル ＆朝：1`）とアクター辞書（`％さくら ＃ メモ`）も同じ**。マニュアルは `block-structure.md:316-333` で行末のコメントを書いてよいとする（brief の行のまま） |
| 5 | TextMate の Lua のフェンスが文法と違う | 残る | tmLanguage の `lua-code-block`（129-147）。begin `^(\s*)(```)(lua)?\s*$`、end `^(\s*)(```)\s*$`。文法は `code_open = PUSH("`"{3,}) ~ id? ~ eol`・`code_close = POP ~ or_comment_eol`（`grammar.pest:226-229`）。```` ``` ＃おわり ```` で閉じない・4 連で開かない・字下げしたフェンスを開く（文法は行頭だけ） |
| 6 | 凡例が 2 か所・比べるテスト無し | 残る | `token_types.rs:12-30`・`semanticTokensProvider.ts:12-30`。拡張のテストは TS の並びを決め打ちで写すだけ（`vscodeModules.test.ts:115`・`e2e.test.ts:87`）。Rust と比べるテストは無い。修飾子（`TOKEN_MODIFIERS`・`PASTA_TOKEN_MODIFIERS`）も同じ |
| 7 | `pasta_lsp` は本番で言語サーバでない | 残る | `server.rs`（184 行）・`document.rs`（327）・`error.rs`（48）・`transport.rs:192-196`（native の空の stub）・`lib.rs:11-17`。テスト `lsp_lifecycle_test.rs`・`document_sync_test.rs`・`crash_recovery_test.rs` だけが `pasta_lsp::document` を使う。`Cargo.toml:8` は `publish = true`。`release.yml` の crates.io は 5 つ（294・304・314・324・334 行の `publish-crate.ps1`）で、`pasta_lsp` は無い（brief の再測定のとおり） |
| 8 | Lua ブロックの中の `#` 行が COMMENT | 残る | `analysis/mod.rs:111-130`（`scan_comment_lines` が全行を見る）。再現：正しい Lua ブロックの `# not comment` の行に、長さ 13 の COMMENT |
| 9 | WASM で `catch_unwind` が効かない | 残る | `transport.rs:127-150`・`:176-182`。ルートの `[profile.release]` は `panic = "abort"`（`Cargo.toml:85`）で、wasm32 も abort。本当の守りは `wasmBridge.ts:187-195` の try/catch と `documentSync.ts:137-141` |
| 10 | デバッグの `host` | 残る | `package.json:250`（属性）・`:281`（`initialConfigurations`）・`:293`（`configurationSnippets`）・`debugAttachTarget.ts:86-89`・`runSceneAtCursor.ts:61`・`:72`。マニュアル：`vscode-setup.md:65`・`:80`、`source-level.md:79`、`troubleshooting.md:88` |
| 11 | 死んだ・同語反復のコード | 残る | `scripts/build-wasm.bat`（36 行。参照は完了済みの spec の文書だけ。`package.json` の `build:wasm` と `update` スキルは `.ps1` を使う）・tmLanguage の `inline-fn-call`（200-203。`inline-word-ref` が同じ所で先に当たる）・`e2e.test.ts:200-288`・`reloadShiori.ts:18-22`（再輸出。読むのは `reloadShiori.test.ts` だけ。`extension.ts` は `runSceneAtCursor` から直に読む）・`DiagnosticsManager.clearAll`（`diagnosticsManager.ts:37`。読むのはテスト `vscodeModules.test.ts:347` だけ）・`WasmAnalysisResult::from_analysis`（`transport.rs:103-108`。読むのは同じファイルのテストだけ） |
| 12 | トークンのキャッシュが版を持たない | 残る | `semanticTokensProvider.ts:54-93`（キーは URI だけ）・`documentSync.ts:135`。加えて、`documentSync.ts:137-141` は解析の失敗で診断だけを消し、古いトークンを残す |
| 13 | `FindingKind::code()` は serde と重複 | 残る | `report.rs:111-141`。使うのは `validate_report_test.rs:264`・`validate_config_test.rs:283`・`pasta_lsp/src/transport.rs:484`（brief の再測定のとおり） |

### 新しく見つけた指摘（同じ領域）

- **N1. 失敗したとき、ファイル全体を 2 回解く**（小物・直しで消える）
  - `analyze` が `parse_str` で失敗すると、`parse_str_partial` の第 1 段がもう一度 `parse_str` を走らせる（`partial.rs:146`）。キーを打つたびに、大きな辞書を 2 回解く。
  - 診断を `parse_str` の誤りから出す形（brief の Approach）にすると、第 1 段は要らなくなる。
- **N2. 行末のコメントに、どこでもコメントの色が付かない**（バグ・4 番の広がり）
  - `or_comment_eol` で終わる行（シーン・アクター・属性・変数代入・Call・キューコマンド・選択肢・閉じのフェンス）の `＃` 以降に、COMMENT の意味の色が無い。`scan_comment_lines` は行全体のコメントだけを見る。
  - TextMate の `comment` も行全体だけで、`call`・`variable`・`attribute` などの規則は `(.+)$` で行末まで自分の色にする。
  - 要件 4.3・4.6 にした。範囲はマニュアル（`block-structure.md:316-333`）の一覧に合わせた。
- **N3. 部分解析の使われない部品**（撤去）
  - `PartialParseError.span` は、どこでも `None` しか入らない。
  - `infer_rule_from_line` は `pub` で、本番では `partial.rs` の中だけ。外で使うのはテスト（`partial_parse_test.rs`・`cue_cmd_parse_test.rs:350-379`）だけ。
  - brief の Approach（診断は `parse_str` から、部分解析はトークンだけ）を採ると、`PartialParseResult.errors` も読み手が無くなる。消すか残すかは設計で決める。
- **N4. brief の箱の正規表現は、まだ文法と違う**（バグ・3 番の直しの穴）
  - `/(?<![＠@])[＠@][\p{ID_Start}_]\p{ID_Continue}*/gu` は、`＠＠＠メール` で何も取らない（文法は `＠＠` のエスケープ＋`＠メール`）。node で確かめた。数を数える形（TextMate の `inline-dynamic-ref` と同じ `(?<![＠@])(?:[＠@]{2})*` の後ろ）にすれば取れる。
  - 正規表現は行を見ないので、単語定義の行・コメント・Lua ブロック・キューコマンドのスコープ（`！コマンド名＠スコープ`）の `＠` にも箱が付く。行の種類を見る手段が別に要る（下の Option）。
- **N5. brief のフェンスの開きの正規表現が、文法より緩い**（3 番の直しの穴）
  - 文法の開きは `` PUSH("`"{3,}) ~ id? ~ eol `` で、言語名の後ろに空白を許さない。`` ```lua ``＋空白の行は、`parse_str` の誤りになる（再現した）。brief の begin `^(`{3,})(\w*)\s*$` は、これを開きとして受け付ける。
  - 言語名は文法では `id`（XID）で、`\w` との違いは設計で確かめる（**Research Needed**）。
- **N6. TextMate のエスケープが文法と違う**（バグ）
  - `inline-escape`（208-211）は `\\[\\%＠@＄$＞>]` で、`\＠`・`\＄`・`\＞` をエスケープの色にする。文法のエスケープは `at_escape`（`＠＠`）・`dollar_escape`（`＄＄`）・`sakura_escape`（`\\`・`\%`）だけで、`a\＠b` は `parse_str` の誤りになる（再現した）。
  - 本当のエスケープ `＄＄x` は、`inline-var-ref` が `＄x` を変数参照の色にする。意味の色は ESCAPE（長さ 2）＋talk で正しい。
  - `tmGrammar.test.ts` の「既存: エスケープ ＠＠単語・＄＄x の色分けは不変」は、誤った今の色を固めている。
  - `book/src/internals/talk-output.md:350` は、`inline-escape` を `sakura_escape` の写しと書く。規則を直すなら、この行も直す。
- **N7. 選択肢の行が、TextMate で単語定義の色になる**（色のずれ）
  - `　＠？はい「はい」` は、TextMate の `word`（`^(\s*)([＠@])(.+)$`）に当たる。意味の色は「将来実装」で無い（`visit_action.rs:35-37`）ので、TextMate の色がそのまま見える。
- **N8. 何も確かめないテスト**（同語反復）
  - `diagnostics_test.rs` の `test_diagnostics_line_numbers_are_0_based`（行が 100 より前）・`test_partial_parse_mixed_errors_and_tokens`（トークンか診断のどちらか）は、1 番の誤りを見逃した。
  - `transport.rs` の `test_wasm_analysis_result_from_error` は、何も assert しない。
- **N9. 重さの 4 通りの対応と、型の写し**（使い道の無い柔軟性）
  - 解析器が出す診断は ERROR だけである。それなのに `transport.rs:63-88` は 4 通りの重さを数に直し、`diagnosticsManager.ts:56-70` はそれを 4 通りに戻す。テスト（`transport.rs:286-349`）は使われない枝を固める。
  - `AnalysisResult` は `tower_lsp::lsp_types` の型で持ち、`transport.rs` が同じ形の `Wasm*` の型へ写す。JS へ渡すのは `Wasm*` だけである。tower-lsp を外すとき、`lsp-types` を足さずに `Wasm*` の形で直に持てば、写しと依存が両方消える（OQ-4）。
- **N10. 起動の状態の `fallbackMode`**（小物）
  - `extension.ts:33-41` の `fallbackMode` は、起動の後は常に `!wasmReady` である。読むのはテスト（`vscodeModules.test.ts:925`）だけ。`getActivationState` も「for testing」。
- **N11. `FindingKind::ALL` もテストだけが使う**（13 番の広がり）
  - `report.rs:58-89`（27 個の配列）。読むのは `validate_report_test.rs:260-268`・`pasta_lsp/src/transport.rs:483` だけ。`code()` と一緒に、`ALL` で回すテストは serde の値で回す形に直せる。
- **N12. `editors/vscode/wasm/README.md` は生成物**（brief の古い記述）
  - `editors/vscode/.gitignore` が `wasm/` を外し、`.vscodeignore` が `wasm/README.md` を VSIX から外す。中身は wasm-pack が写す `crates/pasta_lsp/README.md` である。直すのは `crates/pasta_lsp/README.md` だけでよい。

### 指摘にしなかったもの（記録）

- **閉じのフェンスが行の途中でも閉じる**。文法の `code_contents = (!PEEK ~ ANY)+` は、行の途中の `` ``` `` で本文を終える。`local x = 1```` は正しい辞書として読まれ、意味の色はその行をフェンスにする（再現した）。TextMate の end を行頭に固めると、この書き方だけ色がずれる。文法の側の性質で、`pasta_dsl/src/parser` は範囲外なので、この spec では追わない。本文に `` ``` `` を書くときは 4 連で囲む、というマニュアルの作法で避けられる。
- **修飾子 `declaration`（bit 0）を使う所が無い**。消すと `definition`・`global` の bit がずれる。害が無く、凡例の一致（要件 7）で守れるので、指摘にしない。
- **`semanticTokenScopes` の `word` が `string.key.word.pasta`** で、TextMate の単語参照（`markup.inline.raw.string.pasta`）と別の名前である。テーマの色の当て方の話で、文法とのずれではない。
- **`deactivate` が、`context.subscriptions` にも入れた物をもう一度 dispose する**。害が無い。
- **`pasta_lsp` の名前**は、言語サーバでなくなっても変えない。WASM のファイル名（`pasta_lsp_wasm`）・`release.yml:198` の検査・`build-wasm.ps1` が名前に頼る。
- **ローカルシーンの意味の色が字下げから始まる**（`・ローカル` の SCENE が 0 桁目から）。見た目に出ない（空白）。4 番の直しで名前だけにすれば、ついでに直る。

## brief の読み方（食い違いの解き方）

- 7 番の `release.yml:L329` は古い（brief の再測定のとおり、294〜334 行）。`pasta_lsp` が crates.io に無いのは同じ。
- Scope の `tech.md` の行は、15・66〜69（tower-lsp は 69）・111・124 行。`structure.md` は 186〜205 行（brief の再測定のとおり）。`tech.md:205`（WASM のビルド）は残す。
- Constraints の「README と `tech.md` の断り書きは、先に入る 1 本が書く」は、`ponytail-lua-loader` が書き終えている（`README.md:88`・`tech.md:178`）。この spec は足さない。
- Constraints の「非対話では `build:wasm` が AllSigned で落ちる」は、今の `package.json:311` が `pwsh -NoProfile -File` で `build-wasm.ps1` を呼ぶ形になっている。実装のときに、まず `npm run build:wasm` を試し、だめなら brief の回避（pwsh で先に走らせ `vsce package` を直接）に戻す。
- Scope の `editors/vscode/wasm/README.md` は N12 のとおり生成物で、直す物は無い。

## Requirement-to-Asset Map

| 要件 | 今ある資産 | ギャップ |
| ---- | ---------- | -------- |
| 1 本当の位置に 1 件 | `parse_str` の `SyntaxError{line,column,message}`（行・桁は 1 から、桁は文字）。`syntax.rs:37-56` の `syntax_finding`（接頭辞を外し、`rsplit_once("\n  = ")` で短い説明を取る）。`msg::syntax`。`token_types.rs` の `utf8_offset_to_utf16` | **Missing**: `analysis/mod.rs` で `parse_str` の誤りから 1 件を作る道。`syntax_finding` の文言の作り方を `pasta_lsp` から呼べる形（今は `pub(crate)` の中の private 関数）。**Constraint**: 桁の UTF-16 への直しは行の中身が要る（`ghostValidator.ts` の `utf16Column` と同じ考え） |
| 2 重ねない | `ghostValidator.ts:266`（開いている文書の syntax を隠す） | 規則は今のままでよい。1 が直れば、隠した後に残る 1 件が正しくなる。**Missing**: 二重にならないこと・閉じたら出ることのテスト（`ghostValidator.test.ts` に前例） |
| 3 ほかの色を残す | `split_by_scope_markers`・`try_parse_chunk`・`shift_*`（span の補正は正しい） | **Missing**: 境界を trim する前の行で判定。失敗した塊を飛ばす。**Constraint**: 誤りのシーンの色をどうするか（OQ-1） |
| 4 コメント・属性・名前 | `marker_line_token`（行末まで）・`scan_comment_lines`（行全体だけ）・AST の `scene.attrs`（シーン行の属性が入るかは設計で確かめる） | **Missing**: 名前の span で止める・行末のコメントの位置を知る手段（AST はコメントを持たない）・Lua ブロックの行を飛ばす。**Research Needed**: シーン行の属性の span が AST に入っているか（再現では DECORATOR が出ない） |
| 5 箱とエスケープ | `wordRefDecorator.ts`（正規表現だけ）。意味の色の WORD トークン（単語参照・単語定義・キューのスコープに出る） | **Missing**: 数を数える正規表現（N4）と、行の種類を見る手段。**Constraint**: tmLanguage はマニュアルのハイライトも読む |
| 6 フェンス | `lua-code-block`・`pasta-lua-injection.tmLanguage.json`（`meta.embedded.block.lua.content` に Lua を注入） | **Missing**: 後方参照の end（`\1`）。注入の名前（`contentName`）は保つ |
| 7 凡例 | TS の並びのテスト（`vscodeModules.test.ts:115`）・`package.json` の覆いのテスト（`:135`） | **Missing**: Rust と TS を比べる手段（下の Option） |
| 8 版 | `SemanticTokensProvider`・`DocumentSync`（200ms の束ね） | **Missing**: 版（`document.version`）を鍵に持つか、要るたびに解く |
| 9 WASM だけ | `transport.rs` の 3 つの入口・`Wasm*` の型・JSON の形のテスト（`transport.rs:367-404`・`:408-486`） | **Missing**: 撤去。**Constraint**: JSON の形を変えない（テストが固めている） |
| 10 `host` | `resolveAttachTarget`・`PastaDebugConfigurationProvider.resolveDebugConfiguration`（構成を直す前例：`sourcePresentation` の正規化） | **Missing**: `host` を見て注意し、捨てる所。**Research Needed**: 属性から外すと、VSCode が `launch.json` に「Property host is not allowed」を出すか |
| 11 約束 | `parse_str` の誤りの形 | `ponytail-dsl-parser` の brief は、`SyntaxError` と `From<io::Error>` だけを残すと書く。形（行・桁・文）を保つことを、あちらの要件にも書く必要がある（brief の再測定のとおり、両方の要件に書く） |
| 12 死んだコード | 表の 11 番・N3・N8・N10 | 撤去とテストの書き直し |
| 13 マニュアル | `debug/*.md` の 4 か所・`validate.md:246`・`talk-output.md:350`・README・`TEST_COVERAGE.md:75`・`:185`・`:273`・steering | 直し |
| 14 関門 | `cargo test -p pasta_lsp`（今 118 本）・`npm test`（`test:unit` に 9 本の束）・マニュアルのハイライトの自己テスト（`book/tools/highlight/`。`pasta.tmLanguage.json` を直に読む） | **Constraint**: ワークツリーに `book/node_modules` が無いと、ハイライトの自己テストは `ERR_MODULE_NOT_FOUND` で落ちる（spec のせいではない） |

## Implementation Approach Options

### 診断（要件 1・2・11）

- **Option A（brief の Approach）**：`parse_str` の `SyntaxError` から 1 件を作る。文言は `pasta_check` の作り方を `pub` にして呼ぶ。部分解析はトークンだけにする。
  - ✅ 保存時と同じ関数で文を作るので、食い違いようがない。
  - ✅ N1（2 回解く）が消える。
  - ❌ `pasta_check` の validate の中の関数を 1 つ外へ出す。
- 現実的な代案は無い（部分解析の誤りを直して使う形は、塊ごとの誤りが「本当の 1 件」と合わない）。

### 部分解析（要件 3）

- **Option A（brief の Approach）**：境界は trim する前の行で判定する。失敗した塊は飛ばす（誤りは作らないか、作っても診断には使わない）。第 3 段・`chunk_lines`・`infer_rule_from_line` を消す。
  - ✅ 差分が最小。span の補正（`shift_*`）はそのまま使える。
  - ❌ 誤りのある塊の色は、TextMate の色だけになる（OQ-1）。
- **Option B**：失敗した塊を、`SyntaxError` の行の手前で切り、前半だけ解き直す。
  - ✅ 誤りのシーンの前半にも意味の色が残る。
  - ❌ 切った所が文法の途中になる場合の扱いが増える。最小の直しから外れる。

### 型と依存（要件 9）

- **Option A（brief）**：`tower-lsp` を `lsp-types` に替える。`AnalysisResult` は今の型のまま。
  - ✅ 差分が小さい（`use` の書き換え）。
  - ❌ `lsp-types` の依存と、`Wasm*` への写し・4 通りの重さ（N9）が残る。
- **Option B**：`lsp-types` も足さず、`AnalysisResult` を JS へ渡す形（今の `Wasm*`）で直に持つ。凡例は文字列の並びにする。
  - ✅ 依存が 1 つ減り、写しと `From` の実装と、使われない重さの枝が消える。
  - ✅ JSON の形のテスト（`transport.rs:367-404`）がそのまま守りになる。
  - ❌ brief の Desired Outcome の文言（`lsp-types` に替える）から外れる。テストの書き換えが少し増える。
- 推奨は Option B（OQ-4）。

### 単語参照の箱（要件 5.1〜5.3）

- **Option A**：正規表現を数える形にし、行の種類（単語定義・コメント・Lua ブロック・キューコマンド）を TS で見て除く。
  - ✅ WASM が読めないときも箱が出る。
  - ❌ 行の種類の判定（特に Lua ブロックの中）を TS に写す。文法の写しが 1 つ増える。
- **Option B**：意味の色の WORD トークンのうち、台詞の中の単語参照だけに箱を付ける（解析器が単語参照の種類を区別して返す）。
  - ✅ 文法の写しが増えない。箱が意味の色と必ず一致する。
  - ❌ WASM が読めないときは箱が出ない。解析器の出力に区別を足す（修飾子 1 つ、または別の種類）。
- どちらでも要件は満たせる。設計で比べる。

### 凡例の一致（要件 7）

- **Option A（brief）**：Rust のテストが `semanticTokensProvider.ts` を読み、並びを比べる（または TS のテストが `token_types.rs` を読む）。
  - ✅ 実行時の形を変えない。
- **Option B**：WASM が凡例を返し、拡張はそれで登録する（TS の並びを消す）。
  - ✅ 写しが無くなる。
  - ❌ 起動の順（WASM を読んでから登録）に頼る。今もその順だが、テストのモックが増える。
- 推奨は Option A（brief のまま。小さい）。

### `host` の注意（要件 10.2）

- **Option A**：`package.json` の属性から `host` を外すだけにする。VSCode が `launch.json` の検査で「許されない」と出す（**Research Needed**）。接続は常に `127.0.0.1`。
  - ✅ 足すコードが無い。brief の「許されない」という言い方とも合う。
- **Option B**：A に加えて、`resolveDebugConfiguration` で `host` を見たら、通知か出力チャンネルで注意し、構成から消す。
  - ✅ VSCode の検査を見ない作者にも届く。
  - ❌ SHIORI リロードの後の再アタッチのたびに出るので、うるさい。
- 推奨は Option A（OQ-3）。

## タスクの見積もり

設計で変わりうるが、今の見込みは 17〜19 タスク。

| 束 | タスク |
| -- | ------ |
| 診断とサーバの撤去 | 1. 振る舞いを固めるテスト（1 件・位置・文・CRLF・最後の行） 2. 診断を `parse_str` から（`pasta_check` の文言を pub に） 3. 部分解析の境界と撤去（N1・N3） 4. 開いている文書の診断の重ならないことのテスト 5. コメントの意味の色（8・N2） 6. 名前・属性の意味の色（4） 7. サーバ・文書管理・エラー型の撤去と依存（`Cargo.toml`・`Cargo.lock`） 8. `transport.rs` の整理（9・N9・`from_analysis`・N8） 9. `FindingKind::code()`・`ALL`（13・N11） 10. 凡例の一致のテスト（6） |
| VSCode 拡張の色と整理 | 11. TextMate の単語参照・エスケープ・`inline-fn-call`（3・N6・11） 12. TextMate の名前・属性・行末のコメント・選択肢（4・N2・N7） 13. TextMate のフェンス（5・N5） 14. 箱（3・N4） 15. キャッシュの版（12） ~~16. `host`（10）とマニュアル 3 頁~~（外した） 17. 死んだコード（11・N10） |
| 仕上げ | 18. README・`TEST_COVERAGE.md`・steering・`validate.md`・`talk-output.md` 19. VSIX とマニュアルのハイライトの自己テストを含む最終の関門 |

20 を超えないので、brief の分割の案（「診断とサーバの撤去」と「VSCode 拡張の色と整理」）は採らずに 1 本で進めるのを推奨する。設計で 20 を超えたら、上の表の束で分ける（境目のファイルは `Cargo.lock`・tmLanguage で重ならない）。

## Effort and Risk

- **Effort: M（3〜7 日）**。どの直しも小さい（各 10〜80 行）が、色の直しは tmLanguage・意味の色・箱の 3 か所に同じ規則を入れ、テスト（`tmGrammar.test.ts`・`semantic_token_test.rs` など）の書き直しが広い。
- **Risk: Medium**。
  - tmLanguage はマニュアルのハイライトも読む。色の名前（scope）を変えると、マニュアルの見た目と自己テスト（`tokenizer-test.mjs` の `markup.inline.raw.string.pasta` など）が変わる。
  - `Cargo.lock` は `ponytail-core-registry` と共有し、後から入る側が作り直す。
  - `parse_str` の誤りの形は `ponytail-dsl-parser` が持つ。あちらが文（pest の `  = ` の行）の形を変えると、文言の取り出しが壊れる。

## Research Needed（設計へ持ち越し）

- ~~属性から外した `host` に、VSCode が `launch.json` で「Property host is not allowed」の注意を出すか~~（`host` を外したので要らない）。
- VSCode の TextMate（Oniguruma）が、`\p{ID_Start}`・`\p{XID_Start}` などの Unicode の性質を使えるか。フェンスの言語名（`id`）と単語参照の名前を、文法の XID に合わせるため。
- シーンの宣言の行の属性（`＊挨拶 ＆朝：1`）の span が、AST の `scene.attrs` に入っているか。入っていなければ、意味の色は行の文字から属性の位置を探す。
- wasm32 で panic（trap）した後、同じ WASM のインスタンスを呼び続けてよいか。だめなら、`wasmBridge.ts` が読み直すかを決める。
- `npm run build:wasm` が今の環境で非対話のまま通るか（brief の回避が要るか）。

## 要件ディスカッションの決定（2026-11-02）

開発者が全議題を推奨案で承認した（「承認」）。

- **議題 1（新しい議題）DAP まわりを外す**：監査の 10 番（`host`）・`reloadShiori.ts` の再輸出・`debug/` の 3 頁の `host` の記述を、この spec から外した。要件 10 は欠番。
  - 経緯：セッション「Pasta DSL MCP移行検討」から、VSCode 拡張の DAP をやめ、SSP・areka の MCP（`OnPastaScene` などのイベントを起こす）に移す方針が来た。新しい spec `vscode-mcp-client`（0.5.0。開発者の判断）が受け持ち、外した 3 点はあちらの brief が引き継ぐ。DAP の側のファイル（`debugAdapterFactory.ts`・`debugAttachTarget.ts`・`reloadShiori.ts`・`runSceneAtCursor.ts`・`sourcePresentationToggle.ts`・`package.json` の `debuggers`）は、そちらで消える。
  - 待ち受けは 127.0.0.1 に固定なので、`vscode-mcp-client` が入るまで `host` が残っても害は無い。
  - これで、この spec は `package.json` の `debuggers` に触らない。`extension.ts` は `fallbackMode`（N10）の行だけ触る。`vscode-mcp-client` とは `extension.ts`・`package.json`・`src/test/*` を共有しうるので、ウェーブを組むときに確かめる。
  - `ponytail-lua-debug` は PR#220 で閉じられ、spec のフォルダは main から消えた。申し送りは `vscode-mcp-client` の brief が受け取る。
- **OQ-1** 誤りのあるシーンは、意味の色を付けず TextMate の色に任せる（要件 3.3）。
- **OQ-2** 入力のたびの診断は、幅の無い点にする。位置の無い誤りは 1 行目の頭（要件 1.8・11.2）。
- **OQ-3** 議題 1 で `host` を外したので、要らなくなった。
- **OQ-4** `lsp-types` も足さない。解析の結果は `Wasm*` の形で直に持つ（要件 9.2）。
- **OQ-5** 箱は台詞の中の単語参照と関数呼び出しの名前だけ（要件 5.3）。
- **OQ-6** 選択肢の行は TextMate の規則だけを直す。意味の色は足さない（要件 5.7）。
- **OQ-7** TextMate のエスケープは、この spec で直す（要件 5.5・13.2）。
- **OQ-8** 分けない。

以下は、ディスカッションの前の持ち越しの控え。

## 要件ディスカッションへの持ち越し（控え）

1. **OQ-1 誤りのあるシーンの色**（要件 3.3）
   - 推奨：そのシーン（区切りから次の区切りの手前まで）は意味の色を付けず、TextMate の色に任せる。
   - 理由：brief の最小の直し（失敗した塊は飛ばす）のままで済む。TextMate の色は残るので、真っ白にはならない。前半だけ解き直す形は、切り口の扱いが増える。
2. **OQ-2 診断の範囲**（要件 1.8）
   - 推奨：保存時の診断と同じく、幅の無い点にする。
   - 理由：2 つの診断が同じ見た目になる。トークンの長さを推し量る規則を足さずに済む。
3. **OQ-3 `host` の注意の出し方**（要件 10.2）
   - 推奨：`package.json` の属性から外すだけにし、VSCode の `launch.json` の検査の「許されない」に任せる。値（`127.0.0.1` を含む）によらず出る。接続は常に `127.0.0.1`。
   - 理由：足すコードが無い。実行時の通知は、再アタッチのたびに出てうるさい。VSCode が注意を出さないと分かったら（Research Needed）、出力チャンネルに 1 行書く形に替える。
4. **OQ-4 `lsp-types` に替えるか**（要件 9.2）
   - 推奨：`lsp-types` も足さない（Option B）。解析結果を JS へ渡す形で直に持つ。
   - 理由：本番で使うのは JS へ渡す形だけで、`lsp-types` の型は写しのためにしかない。依存と写しと使われない重さの枝（N9）が一緒に消える。brief の文言（`lsp-types` に替える）から外れるので、確かめたい。
5. **OQ-5 箱を付けない所**（要件 5.3）
   - 推奨：箱は台詞の中の単語参照（関数呼び出し `＠関数（…）` の名前を含む。今と同じ）にだけ付ける。単語定義の行・コメント・Lua ブロック・キューコマンドのスコープ（`！コマンド名＠スコープ`）には付けない。
   - 理由：キューのスコープは単語の引き当てでない（`block-structure.md:231-233`）。関数呼び出しの名前は今も箱が付き、変えると別の挙動の変更になる。
6. **OQ-6 選択肢の行の色**（要件 5.7・N7）
   - 推奨：TextMate の規則で、選択肢の行を単語定義の色にしない（`＠？` を印、`「」` を文字列の色に）。意味の色は足さない。
   - 理由：意味の色は「将来実装」で、足すのは機能の追加になる。TextMate のずれは同じ根（色が文法と違う）なので、ここで直す。
7. **OQ-7 TextMate のエスケープ**（要件 5.5・N6）
   - 推奨：この spec で直す。`＠＠`・`＄＄`・`\\`・`\%` だけをエスケープの色にし、`\＠` などを外す。`talk-output.md:350` の写しの記述も直す。
   - 理由：3 番と同じ根（色が文法と違う）で、別の spec へ送らない。
8. **OQ-8 分けるか**
   - 推奨：分けない（17〜19 タスク）。
   - 理由：brief の決めごと（20 を超えたら分ける）に当たらない。設計で 20 を超えたら、上の表の束で分ける。

---

# 設計フェーズの調査（2026-11-02）

`/kiro-spec-design ponytail-editor-tooling -y` の発見と統合の記録である。種別は Extension なので、軽い発見（`design-discovery-light.md`）で進めた。新しい依存は足さない（pasta_check の dev-dependencies の `serde_json` を除く。下の決定 D7）。

## 使ったスキルと規則

- `kiro-spec-design`（`rules/design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）と、テンプレート `.kiro/settings/templates/specs/{design,research}.md`。
- `ponytail`（full）：最小の直し。使い道の無い型の写し・重さの 4 通り・`catch_unwind` を足し直さない、の判断に使った。
- steering：`product.md`・`tech.md`・`structure.md`・`roadmap.md` の「ウェーブ（2026-11-02）」と同じウェーブの約束。
- プロジェクトの記憶：Bash ツールのヒアドキュメントで二重のバックスラッシュが潰れる（スクラッチの JS は Write で書いた）。cargo の前に `NoDefaultCurrentDirectoryInExePath` を外す。

## 調べたこと（Research Log）

### R1. 宣言の行の属性と名前の span は、AST にあるか（持ち越しの Research Needed）

- **きっかけ**: 要件 4.1・4.2（名前だけを名前の色に、属性を属性の色に）。
- **調べた所**: `pasta_dsl/src/parser/parse_scene.rs` の `parse_global_scene_start`（73 行）・`parse_local_scene_scope`（194 行）、`grammar.pest` の `scene = _{ id ~ s ~ attrs? }`。一時のテストで `＊挨拶 ＆朝：1 ＃メモ` を解いた（テストは消した）。
- **分かったこと**:
  - 宣言の行の属性は、文法では受け付けるが、パーサが名前の `id` だけを拾って捨てる。`GlobalSceneScope.attrs`・`LocalSceneScope.attrs` には入らない。
  - 名前の span も無い。`scene.span` はスコープ全体（宣言の行から次の区切りの手前まで）で、始まりは印の位置である。
  - この捨てる動きは `scene-attribute-store`（0.5.0 のウェーブ 2）の brief が既に書いている（189 行）。
- **設計への影響**: 宣言の行（`＊`・`・`・`％`）の名前・属性・行末のコメントは、解けた塊の行の文字を読んで決める。`scene-attribute-store` が AST に span を足したら、AST に替えてよい（再検証のきっかけ）。

### R2. VSCode の Oniguruma は `\p{XID_Start}` を使えるか（持ち越しの Research Needed）

- **調べた所**: main の作業コピーにある `editors/vscode/node_modules/vscode-oniguruma`（2.0.1。拡張の `tmGrammar.test.ts` が使う）と `book/node_modules/vscode-oniguruma`（マニュアルのハイライトが使う）。スクラッチの node で走らせた。
- **分かったこと**:
  - `\p{XID_Start}`・`\p{XID_Continue}`・`\p{ID_Start}`・`\p{ID_Continue}` を、どちらも使える。`𠮷`（サロゲートペア）も 1 文字として当たる。
  - `^(`{3,})(?:[\p{XID_Start}_]\p{XID_Continue}*)?$` は ```` ````lua ```` と ```` ```日本 ```` に当たり、```` ```lua ````＋空白には当たらない（N5 の直し）。
  - `(?:[＠@]{2})|([＠@][\p{XID_Start}_]\p{XID_Continue}*)` を左から当てると、`＠＠＠メール` は `＠＠` と `＠メール` に分かれる。`＠挨拶。こんにちは` は `＠挨拶` だけになる（N4 の直し）。
  - JS の正規表現（`u` 旗）も同じ書き方で同じ結果になる。tmLanguage の `inline-sakura-script` の `match` は、JS の `u` 旗でそのまま compile できる（`v` 旗は不可）。
- **設計への影響**: 名前の文字の集まりは、文法の `id`（`XID_START`・`XID_CONTINUE`、頭は `_` も可）に合わせて `[\p{XID_Start}_]\p{XID_Continue}*` と書く。箱は tmLanguage の台詞の規則を JSON から読んで JS の正規表現にできる（D6）。

### R3. 改行で終わらないファイル（新しい指摘 N13）

- **きっかけ**: 要件 1.5（改行で終わらない最後の行）と 2.3（保存した中身と同じなら、保存時と同じ診断）。
- **調べた所**: 一時のテストで `parse_str` を走らせた。`pasta_lua/src/loader/process.rs:182`（pasta.dll の読み込み）と `pasta_check/src/validate/syntax.rs:13`（保存時の検査）は、読んだ文字をそのまま `parse_str` に渡す。`book/src/grammar/markers.md:39`。
- **分かったこと**:
  - `＊挨拶\n　さくら：やあ`（最後に改行が無い）は、`parse_str` が 2 行目の行末で誤りにする。最後の行が `＃c` でも誤りになる。pasta.dll も保存時の検査も、このファイルを読めない。
  - 今の `AnalysisEngine::analyze` は、解く前に改行を補う。だから入力のたびの診断は何も出さない。開いている文書では保存時の syntax も隠す（`ghostValidator.ts:266`）。開いている間、作者には誤りが 1 件も見えない。
  - マニュアル（`markers.md:39`）は「エディタ拡張の診断は末尾の改行を補って解析するため、この誤りを報告しない」と書いている。
  - VSCode は既定で最後の改行を足さない（`files.insertFinalNewline` の既定は false）。拡張も `configurationDefaults` を持たない。
- **設計への影響**: 要件 2.3 に合わせ、診断のための `parse_str` には改行を補わない。改行の無いファイルは、保存時と同じ位置・同じ文で 1 件出る。色の部分解析は、最後の塊に改行を補う（今の `try_parse_chunk` のまま）ので、色は残る。`markers.md:39` の文は直す必要がある（要件 13 の一覧の外。OPEN QUESTION）。文法が最後の改行を要らなくするかは、パーサの持ち場（`pasta_dsl/src/parser/**`）で、この spec では決めない。

### R4. wasm32 で panic（trap）した後、同じインスタンスを呼んでよいか（持ち越しの Research Needed）

- **調べた所**: ルートの `[profile.release]` の `panic = "abort"`（`Cargo.toml:85`）、`wasmBridge.ts:182-195`・`documentSync.ts:137-141`・`ghostValidator.ts:212` の try/catch。wasm-bindgen の生成物の作り。手元で WASM を作って trap させる実験はしていない。
- **分かったこと**:
  - wasm32-unknown-unknown の panic は、`unreachable` の trap になる。JS には `RuntimeError` として投げられ、拡張の try/catch が受ける。`catch_unwind` は効かない（9 番のとおり）。
  - trap の後もインスタンスは呼べる。ただし、関数の後始末を飛ばすので、wasm-bindgen の影のスタックの位置や、確保した引数の文字列が戻らないことがある。続けて呼んだ結果は保証されない。
  - 壊れたとしても、次の呼び出しも例外か形の誤りになり、同じ try/catch に落ちる。拡張は落ちず、TextMate の色が残る（要件 9.5）。
- **設計への影響**: 読み直しの仕組みは足さない。今の try/catch を守りとし、`catch_unwind` の包みだけを消す。解析器の panic は、今の頑丈さのテスト（`analyze_robustness_test.rs`）で起きないことを確かめている。

### R5. `npm run build:wasm` が非対話で通るか（持ち越しの Research Needed）

- 設計では確かめない。実装の最後の関門（要件 14.3）で、まず `npm run build:wasm` を試し、だめなら brief の回避（`build-wasm.ps1` を pwsh で先に走らせ、`vsce package` を直に叩く）に戻す。

### R6. 部分解析の塊の区切りと、続きの `＊`

- **調べた所**: `partial.rs:99-108`・`grammar.pest` の `global_scene_continue_line = { global_marker ~ or_comment_eol }`・`parse_global_scene_start`（`last_name` を見る）。
- **分かったこと**:
  - 区切りになる行頭の文字は、字下げしない `＊`・`％`・`＆`・`＠` である（`＆`・`＠` は 0 桁目ならファイルの属性と単語定義、字下げすればシーンの中の行）。ローカルシーンの `・` は字下げが要るので区切りにならない。
  - 名前の無い `＊`（続きのシーン）は、前の名前付きのシーンの名前を受け継ぐ。塊ごとに解くと前の名前が無いので、`Unnamed global scene at start of file` の誤りになる（一時のテストで再現）。
  - Lua ブロックの本文の 0 桁目に `＊`・`％`・`＆`・`＠` が来ると、塊がブロックの途中で切れる。Lua の文の頭には来ない文字なので、まれである。
- **設計への影響**: 区切りは「字下げする前の行頭の文字」で決め、名前の無い `＊` の行は区切りにしない（前の塊と一緒に解く）。Lua の本文の 0 桁目の印は追わない（`shortcut:` のコメントを残す）。

### R7. 箱を意味の色から作るか、文字から作るか（設計に任された Option A・B）

- **比べたこと**:
  - B（意味の色の WORD から作る）は、文法の写しが増えない。けれども、WASM が読めないときと、誤りのあるシーン（要件 3.3 で意味の色を付けない）で箱が消える。書きかけの行があるたびに、そのシーンの箱が消えたり出たりする。台詞の単語参照を見分ける修飾子を凡例に足すことになり、関数呼び出しの WORD は引数まで覆うので、名前の範囲を別に決める必要もある。
  - A（文字から作る）は、書きかけでも WASM が無くても箱が出る（今と同じ）。行の種類（Lua ブロック・アクション行・継続行）の判定を TS に持つ。
  - A の台詞の中の規則（さくらスクリプト・エスケープ・単語参照・動的な単語参照）は、tmLanguage の `match` を JSON から読み込めば、写しを増やさずに TextMate と同じにできる（R2）。
- **結論**: A を採る（D6）。

## 統合（Synthesis）

- **一般化**
  - 要件 1・2・11 の診断は、どれも「保存時と同じ 1 件」の特別な場合である。保存時の `syntax_finding` を 1 つの出どころにし、入力のたびの診断はそれを位置だけ UTF-16 に直して使う。
  - 要件 4.1〜4.3 の行末のコメントは、どの行でも「行の本体の終わりの後で、空白を飛ばした最初の `＃`・`#`」で決まる。行の種類ごとに違うのは本体の終わりの見つけ方だけなので、1 つの助け手（`push_trailing_comment`）にまとめる。
  - 要件 5.1〜5.4 の箱と TextMate の色は、同じ規則の 2 つの使い道である。規則は tmLanguage に 1 つだけ置き、箱はそれを読む。
- **作るか採るか**
  - LSP の型（`tower-lsp`・`lsp-types`）は採らない。JS へ渡すのは `Wasm*` の形だけで、LSP の型は写しのためにしかない（OQ-4 の決定）。
  - 凡例の一致は、実行時に WASM から凡例をもらう形（Option B）を採らない。起動の順とモックが増える。Rust のテストが TS のファイルを読んで比べる。
  - 箱のために vscode-textmate を本番の依存に足すことはしない。JSON の 4 つの `match` を JS の正規表現にするだけで足りる。
- **単純化**
  - `PartialParseError` は型ごと消す。読み手は「どの行を色にしないか」だけを要るので、失敗した塊の行の範囲（`failed_lines`）だけを返す。文は `parse_str` の誤りから作るので要らない。
  - 部分解析の第 1 段（全体を解く）も消す。呼び手（`analyze`）は全体で失敗したときにだけ呼ぶ（N1）。
  - `AnalysisResult` と `WasmAnalysisResult` を 1 つにする。JS の名前は serde の rename で付け、Rust の欄の名前は今の LSP の名前のまま残す（テストの書き換えを減らすため）。
  - 重さは常に Error。Rust の 4 通りの対応と、拡張の `convertSeverity` の 4 通りを消す（N9）。JS の形の `severity` の欄は残す（要件 9.4）。
  - wasm の trap の後の読み直しは足さない（R4）。

## 方式の比較（Architecture Pattern Evaluation）

| 論点 | 案 | 長所 | 短所 | 採否 |
| ---- | -- | ---- | ---- | ---- |
| 診断 | `parse_str` の誤り 1 件＋`syntax_finding` | 保存時と同じ関数。2 回解かない | `pasta_check` の関数を 1 つ pub にする | 採る |
| 部分解析 | 失敗した塊を飛ばす（範囲だけ返す） | 差分が最小。span の補正はそのまま | 誤りのシーンは TextMate の色だけ（OQ-1 で承認済み） | 採る |
| 型 | `Wasm*` の形で直に持つ | 依存・写し・使われない重さが消える | テストの `use` の書き換え | 採る（OQ-4） |
| 箱 | A：文字から（tm JSON を読む） | 書きかけ・WASM 無しでも出る。TextMate と同じ規則 | 行の種類の判定を TS に持つ | 採る |
| 箱 | B：意味の色から | 文法の写しが無い | 書きかけで点滅・WASM 無しで消える・凡例に修飾子 | 採らない |
| 凡例 | Rust のテストが TS を読む | 実行時の形を変えない | ファイルの場所に頼るテスト | 採る |
| 凡例 | WASM が凡例を返す | 写しが無い | 起動の順・モックが増える | 採らない |

## 設計の決定（Design Decisions）

### D1. 入力のたびの診断は、`parse_str` の誤り 1 件から作る

- **背景**: 要件 1・2.3・11.1・11.2。
- **選んだ形**: `analyze` は、受け取った文字をそのまま `parse_str(source, "<lsp>")` に渡す（改行を補わない。R3）。失敗したら、`pasta_check::validate::syntax_finding` で `Finding` を作り、その行・桁を 0 始まりの行と UTF-16 の桁に直した点（幅 0）の診断を 1 件だけ返す。行が無い誤りは 1 行目の頭にする。
- **理由**: 保存時と同じ関数で文を作るので、食い違いようがない。
- **トレードオフ**: `pasta_check` の private の関数を pub にする（pub API が 1 つ増える）。改行で終わらないファイルに、今は出ない誤りが出る（R3。正しい挙動）。

### D2. 部分解析は色のためだけにし、失敗した塊の行の範囲を返す

- **選んだ形**: `parse_str_partial(source) -> PartialParseResult { items, failed_lines: Vec<Range<usize>> }`。区切りは字下げする前の行頭の文字（`＊`・`％`・`＆`・`＠`）で、名前の無い `＊` の行は区切りにしない。塊は `try_parse_chunk` で解き、解けた塊は span を補正して積み、解けない塊は行の範囲だけを積む。第 1 段・第 3 段・`chunk_lines`・`infer_rule_from_line`・`PartialParseError` を消す。
- **理由**: 色を残すのに要るのは、解けた塊の AST と、色を付けない行の範囲だけである。
- **フォローアップ**: 続きの `＊` の塊は、前の塊が解けないと一緒に色を失う（OPEN QUESTION）。

### D3. 解析の結果は JS へ渡す形で直に持つ

- **選んだ形**: `analysis` に `AnalysisResult { tokens: Vec<SemanticToken>, diagnostics: Vec<Diagnostic> }` と、`SemanticToken`・`Diagnostic`・`Range`・`Position` を serde の `Serialize` つきで置く。JS の名前（`deltaStartCharacter`・`tokenModifiers` など）は rename で付ける。`transport.rs` は WASM の 3 つの入口だけになる。
- **トレードオフ**: brief の文言（`lsp-types` に替える）から外れる（OQ-4 で承認済み）。

### D4. 宣言の行は、行の文字を読む

- **選んだ形**: 解けた塊の宣言の行（`＊名前`・`＊`・`　・名前`・`％名前`）を、文法の `印 ~ id? ~ s ~ attrs? ~ or_comment_eol` に沿って読む小さな読み取り（`scan_decl_line`）を `visit_scope.rs` に置く。結果は「印と名前の終わり」「属性ごとの範囲」「行末のコメントの始まり」である。行はもう解けているので、引用（`「…」`・`"…"`）の中の `＃` を飛ばすだけで足りる。
- **理由**: AST に名前と宣言の行の属性の span が無い（R1）。パーサは範囲外。

### D5. 行末のコメントは、行の本体の終わりの後で探す

- **選んだ形**: 助け手 `push_trailing_comment(source, line, from_byte)` が、その行の `from_byte` から空白を飛ばし、次が `＃`・`#` なら行末までを COMMENT にする。本体の終わりは、行の種類ごとに AST の span（属性・単語定義・変数代入・Call・アクター指定）か、行の読み取り（宣言・キュー・選択肢・閉じのフェンス）で決める。
- **理由**: 値を囲まない単語定義では、`＃` 以降が値の span に入るので、何も出ない（要件 4.4 が自然に守られる）。アクション行・継続行には呼ばない。

### D6. 箱は文字から作り、台詞の規則は tmLanguage から読む（Option A）

- **選んだ形**: `wordRefDecorator.ts` に純関数 `findWordRefRanges(text)` を置く。規則はすべて tmLanguage の JSON から読む。行ごとに、Lua ブロックの中か（`lua-code-block` の `begin`・`end`。`end` の `\1` は開きの印で置き換える）、アクション行・継続行か（`action-line`・新しい `continue-action-line` の `match`。名前を `id` に合わせるので、ほかの行の規則と当たりが重ならない）を見る。台詞の部分には、`inline-sakura-script`・`inline-escape`・`inline-dynamic-ref`・`inline-word-ref` の `match` を 1 つにまとめた JS の正規表現を左から当て、単語参照と動的な単語参照の当たりだけを箱にする。
- **理由**: R7。TextMate と箱が同じ規則になり、写しが増えない。
- **フォローアップ**: JSON の `match` に JS で読めない Oniguruma の書き方を入れると、拡張のテストで compile に失敗する（気づける）。

### D7. `FindingKind::code()`・`ALL` を消し、テストは serde の値と網羅の `match` で確かめる

- **選んだ形**: `validate_report_test.rs` の重さの表のテストは、テストの中の `match`（ワイルドカード無し）で種類ごとの重さと code を返し、`serde_json::to_value(kind)` と比べる。種類を足すと `match` が compile できないので、`ALL` の長さの検査より強い。`validate_config_test.rs:283` は `f.code == FindingKind::ConfigActorFont` にする。`pasta_lsp/src/transport.rs` の `ALL` の回しは消す（`validate_report_test.rs` が持つ）。
- **トレードオフ**: `pasta_check` の dev-dependencies に `serde_json`（workspace）を足す（OPEN QUESTION。代案は `toml` で直列化）。

### D8. 意味の色の控えは、文書の版を鍵に持つ

- **選んだ形**: `SemanticTokensProvider` は URI ごとに `{ version, tokens }` を持つ。求められた文書の版と控えの版が違えば、その場で解き直す。解析に失敗したら控えを消して `null` を返す。`DocumentSync` も失敗したら控えを消す。
- **理由**: 要件 8。束ねの 200ms の間に VSCode が色を求めても、前の版を返さない。

## 危険と手当て（Risks & Mitigations）

- tmLanguage はマニュアルのハイライトも読む。scope の名前は変えない（`markup.inline.raw.string.pasta`・`meta.embedded.block.lua.content` など）。マニュアルの自己テスト（`tokenizer-test.mjs`・`highlight-html-test.mjs`）を関門にする。ワークツリーに `book/node_modules` が無いときは、main の作業コピーのものを借りて走らせる。
- `Cargo.lock` は `ponytail-core-registry` と共有する。この spec は tower-lsp の行（ルートの `Cargo.toml:59`）だけを消し、後から入る側が作り直す。
- `parse_str` の誤りの文の形（`Parse error in {file} at {line}:{column}: ` と、末尾の `\n  = 説明`）は `ponytail-dsl-parser` が保つ。形が変わると、`syntax_finding` の取り出しが保存時と入力時の両方で同じように壊れる。両方を同じ関数にしたので、食い違いは起きない。
- 改行で終わらない辞書に、今まで出なかった誤りが出る（R3）。正しい挙動だが、作者には新しく見える。マニュアルの `markers.md:39` を直す（OPEN QUESTION）。
- 箱の正規表現を tmLanguage から組み立てるので、tmLanguage の `match` を JS で読めない形にすると、拡張が箱を出せなくなる。拡張の単体テストで compile を確かめる。

## 参考（References）

- `crates/pasta_dsl/src/parser/grammar.pest`（`id`・`scene`・`or_comment_eol`・`code_open`・`code_close`・`at_escape`・`dollar_escape`・`sakura_escape`・`word_ref`・`word_ref_dynamic`）
- `crates/pasta_check/src/validate/syntax.rs`（`syntax_finding`）
- `book/src/grammar/block-structure.md:316-333`（行末のコメントを書ける行）・`book/src/grammar/markers.md:39`（最後の改行）
- vscode-oniguruma 2.0.1・vscode-textmate 9（`tmGrammar.test.ts` とマニュアルのハイライトが使う）
- ECMAScript の Unicode の性質のエスケープ（`\p{XID_Start}`・`\p{XID_Continue}` は `u` 旗で使える）
