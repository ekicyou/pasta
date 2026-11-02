# Brief: ponytail-editor-tooling

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。0.4.5（2026-10-24 開発者の決定「ポニーテールは 0.0.1 後ろからの着手に変更」。経緯: 2026-10-19 に 0.4.3、2026-10-22 に版に縛らない隙間、2026-10-23 に 0.4.3）。0.4.3 の mlua 0.12 の後の main で測り直す。同じ監査の 10 本と同じウェーブで並走できる。

## 進め方（2026-10-19 開発者の決定）

- 下の「Current State」は、2026-10-19 の監査の控え（出発点）である。
- **ギャップ分析**で、この spec の領域に `/ponytail:ponytail-audit` をやり直す。
  - 領域：`crates/pasta_lsp`、`editors/vscode`、`crates/pasta_dsl/src/partial.rs`、`crates/pasta_check/src/validate/{syntax,report}.rs`。
- 控えと突き合わせ、違いをギャップ分析の報告に書く。
- 要件は「その領域の監査の指摘が、残らず直っている」ことと、挙動を変える直しの決めごとにする。
- **修正は設計以降**で行う。

## Problem

VSCode で辞書を書くゴーストの作者が困る。

- 書き間違いが 1 つあるだけで、シーンの全行に、ずれた位置で波線が付く。正しい保存時の誤りは、隠れてしまう。
- 単語参照の箱、行末のコメント、Lua のフェンスの色が、本当の文法とずれる。

加えて、`pasta_lsp` は本番では言語サーバとして使われていない。それなのに tower-lsp のサーバ（約 850 行と依存 10 個ほど）を抱えている。

## Current State

2026-10-19 の監査で確かめた。

- `cargo test -p pasta_lsp` と clippy は 0 警告。
- 1・4・8 は、`AnalysisEngine::analyze` と `parse_str_partial` を、外の使い捨てのクレートで走らせて再現した。

1. **1 つの書き間違いで、シーン全体に波線が付く**（Must fix）
   - 場所：`crates/pasta_lsp/src/analysis/mod.rs` L66-99。
   - 実例：次の 5 行は、4 行目だけが誤り。
     ```
     ＊挨拶
       さくら：やあ
       うにゅう：おう
       ＞＞
       さくら：ね
     ```
   - これに対して、2〜5 行目に 4 件の誤りが出る。文言は `Parse error: <partial>:1:3: … --> 1:3` に pest の複数行の枠が付いたもので、位置は全部 0 桁目。
   - 開いている文書では、`ghostValidator.ts:L266` が、`pasta_check` の正しい syntax の報告を隠す。
2. **部分解析が、正しい行を誤りにし、シーンの本体を落とす**（1 と同じ根）
   - 場所：`crates/pasta_dsl/src/partial.rs` L99-108（trim した後の行で境界を判定する）、L180-212（第 3 段で、各行をファイル全体として解く）。
   - A：上の例と同じく、正しい行にも誤りが出る。シーンの AST は、項目を全部失う。
   - B：字下げした `　％さくら、うにゅう`・`　＠挨拶：…`・`　＠？choice` を含むシーンは、境界とみなされて割れる。アクター・単語・項目が 0 になる。
   - 誤りが悪い行だけに出ることを確かめるテストは無い。
3. **単語参照の箱の規則が、パーサの `id` と違う**
   - 場所：`editors/vscode/src/wordRefDecorator.ts` L6、tmLanguage の `inline-word-ref`・`inline-dynamic-ref`。
   - `＠挨拶。こんにちは` で、箱が `＠挨拶。こんにちは` の全体に付く。
   - 次の所にも、箱が付いてしまう。
     - `＠＠メール`（エスケープ）の中の `＠メール`
     - 単語定義の行
     - `＃` のコメント行の中の `＠`
4. **シーン行などの行末のコメントと属性が、シーン名として塗られる**
   - 場所：`crates/pasta_lsp/src/analysis/visit_scope.rs` L179-193、tmLanguage の `(.+)$`。
   - `＊OnTest ＃ メモ` が、長さ 12 の NAMESPACE 1 つになる。`＊挨拶 ＆朝：1` には DECORATOR が付かない。
   - マニュアル（`book/src/grammar/block-structure.md` L316-333）は、行末のコメントを書いてよいとしている。
5. **TextMate の Lua のフェンスが、文法と違う**
   - ```` ``` ＃おわり ```` で閉じない。4 連のフェンスも壊れる。
6. **トークンの凡例が 2 か所にあり、比べるテストが無い**
   - 場所：`crates/pasta_lsp/src/analysis/token_types.rs` L12-30、`editors/vscode/src/semanticTokensProvider.ts` L12-30。
7. **`pasta_lsp` は、本番では言語サーバでない**
   - 使われるのは、`#[wasm_bindgen]` の 3 つの関数だけ。
   - 次のものは、自分のテストしか使わない。
     - `server.rs`（184 行）・`document.rs`（327 行）・`error.rs`（48 行）
     - `transport.rs` の native の空の stub
     - `lib.rs` L11-17
     - テスト：`lsp_lifecycle_test.rs`・`document_sync_test.rs`・`crash_recovery_test.rs`
   - `release.yml:L329` は、`pasta_lsp` を公開しない。
8. **Lua ブロックの中の `#` 行が、COMMENT になる**
   - 場所：`analysis/mod.rs` L111-130。
9. **WASM では `catch_unwind` が効かない**
   - wasm32 は `panic=abort` なので効かない。本当の守りは、`wasmBridge.ts` の try/catch。
   - 場所：`transport.rs` L127-150・L176-182。
10. **デバッグの `host` 属性は、使い道の無い柔軟性**
    - 待ち受けは 127.0.0.1 に固定されている。
    - 場所：`package.json`（`attach.host`・`initialConfigurations`・`configurationSnippets`）、`debugAttachTarget.ts` L86-89、`runSceneAtCursor.ts` L61・L72。
11. **死んだ、または同語反復のコード**
    - `scripts/build-wasm.bat`（どこからも参照されない）
    - tmLanguage の `inline-fn-call`（決して当たらない）
    - `src/test/e2e.test.ts` L200-288
    - `reloadShiori.ts` L18-22 の再輸出
    - `DiagnosticsManager.clearAll`
    - `WasmAnalysisResult::from_analysis`
12. **セマンティックトークンのキャッシュが、版を持たない**
    - 場所：`semanticTokensProvider.ts` L54-93、`documentSync.ts` L135。
    - 前の文書の結果を返しうる。
13. **`FindingKind::code()` は serde の名前の重複で、テストだけが使う**
    - 場所：`crates/pasta_check/src/validate/report.rs` L93-123。
    - 使うテスト：`validate_report_test.rs:262`、`validate_config_test.rs:283`、`pasta_lsp/src/transport.rs:484`。

## Desired Outcome

- 構文の誤りは、本当の行と桁に 1 件だけ出る。文言は、保存時や `pasta_check validate` と同じ。
- 誤りの後ろのシーンも、色が付いたまま残る。
- 箱・コメント・属性・フェンスの色が、本当の文法と一致する。
- `pasta_lsp` は WASM の解析器だけになる。
  - `tower-lsp` を `lsp-types` に替え、`thiserror` を外し、`publish = false` にする。
- `launch.json` の `host` は廃止する（2026-10-19 開発者の決定）。書いてあれば「許されない」と注意し、無視する。
- 凡例は、2 か所が一致することをテストが確かめる。

## Approach

監査の最小の直しを採る。

- 診断は、`parse_str` の `SyntaxError{line,column,message}` から 1 件だけ出す。column は UTF-16 に直す。文言は、`pasta_check` の `rsplit_once("\n  = ")` を pub にして使う。
- 部分解析は、トークンにだけ使う。
  - 境界は、trim する前の行で判定する。
  - 失敗した塊は、その塊自身の `ParseError` の行に `line_offset` を足した誤り 1 つにする。
  - `chunk_lines`・第 3 段のループ・`infer_rule_from_line` を消す。
- 箱の正規表現は `/(?<![＠@])[＠@][\p{ID_Start}_]\p{ID_Continue}*/gu`。
- フェンスは begin `^(`{3,})(\w*)\s*$`、end `^\1(?=\s*(?:[#＃].*)?$)`。

## Scope

- **In**: 上の 1〜13 と、ギャップ分析で増えた同じ領域の指摘。あわせて、次も直す。
  - テスト：`crates/pasta_lsp/tests/*`・`editors/vscode/src/test/*`
  - マニュアル：`book/src/debug/{source-level,troubleshooting,vscode-setup}.md`（`host`）
  - `crates/pasta_lsp/README.md`・`editors/vscode/wasm/README.md`・`TEST_COVERAGE.md`
  - `.kiro/steering/structure.md`（L190-205）・`tech.md`（L15・L65・L110・L123）
- **Out**:
  - デバッガの本体（`ponytail-lua-debug`）
  - 本文の解析（`partial.rs` の外の `pasta_dsl/src/parser`。`ponytail-dsl-parser`）

## Boundary Candidates

- 診断（`analysis/mod.rs`・`partial.rs`・`syntax.rs`）
- 色（`visit_scope.rs`・tmLanguage・`wordRefDecorator.ts`・凡例）
- サーバの撤去（`server.rs`・`document.rs`・`error.rs`・`Cargo.toml`）
- 拡張の整理（`host`・死んだコード・キャッシュ）

## Out of Boundary

- `pasta_dsl/src/parser/**`（`ponytail-dsl-parser`）
- `pasta_lua/src/debug`（`ponytail-lua-debug`）
- マニュアルのハイライトの CI（`ponytail-manual-ci`。文法の変更でマニュアルを再公開する path filter は、あちらで足す）

## Upstream / Downstream

- **Upstream**: `pasta_dsl`・`pasta_check` の validate の中核
- **Downstream**: VSCode 拡張の利用者、マニュアルのハイライト（同じ tmLanguage を読む）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-dsl-parser`（`pasta_dsl` の別のファイル）
  - `ponytail-core-registry`（`Cargo.lock` とルートの `Cargo.toml`。席はこの spec が持つ）
  - `ponytail-check-release`（`pasta_check` の別のファイル）
  - `ponytail-lua-debug`（`book/src/debug/` の別の節）

## Constraints

- 公開クレートの pub API を消してよい（2026-10-19 開発者の決定）。README と `tech.md` の断り書きは、10 本のうち先に入る 1 本が 1 回だけ書く。
- VSIX のビルドには wasm-pack が要る（導入済み）。非対話では、`build:wasm` が AllSigned で落ちる。`build-wasm.ps1` を pwsh で先に走らせ、`vsce package` を直接叩いて避ける。
- 振る舞いを固めるテストを先に書き、1 つ直すごとに確かめて 1 コミットにする。
- 品質の関門：`cargo fmt --check`、clippy（警告 0）、`cargo test -p pasta_lsp -p pasta_dsl -p pasta_check`、拡張の `npm test`。cargo の前に `NoDefaultCurrentDirectoryInExePath` を外す。

## 2026-11-02 棚卸の再測定（main 491cc23c）

- **前提の変化**
  - 持ち場の中身（`crates/pasta_lsp`・`crates/pasta_dsl/src/partial.rs`・`editors/vscode/src`・`syntaxes`・`validate/syntax.rs`）は 10-24 から変わらない。`editors/vscode/package.json` は版（0.4.4）だけ。13 の指摘は残る。
  - `recommend-menu-scenes` が `validate/report.rs` に `FindingKind` を足した（475 → 515 行）。13 番の `code()` は 111 行（105 行から）。使うテストは `validate_report_test.rs` 264 行・`validate_config_test.rs` 283 行・`pasta_lsp/src/transport.rs` 484 行。
  - ルートの `Cargo.toml` は `mlua-0-12-upgrade`（mlua 0.12・`mlua-stdlib` を外す）と `shiori4-v3-portable`（`wintf-winmsg-executor` を外し `shiori4_abi` を足す）が書き換えた。tower-lsp は 59 行、`thiserror` は 22 行のまま。`Cargo.lock` の席はこの spec が持つ（前のまま）。
- **触るファイル**: `crates/pasta_lsp/**`（`server.rs` 184 行・`document.rs` 327 行・`analysis/mod.rs` 131 行・`transport.rs` 487 行・`Cargo.toml`・`tests/`）、`crates/pasta_dsl/src/partial.rs`（477 行）、`crates/pasta_check/src/validate/{syntax,report}.rs`（`report.rs` 515 行）、`editors/vscode/`（`package.json` の `host` は 250・281・293 行のまま、tmLanguage、`src/*.ts`、`scripts/build-wasm.bat`、`src/test/*`）、ルートの `Cargo.toml`・`Cargo.lock`、マニュアルの `debug/` の 3 頁、`tech.md`・`structure.md`。1,000 行に近いものは無い。
- **規模**: 16〜19 タスク（変わらず）。
- **先に要るもの**: 働きの上では無い。
  - ponytail の中: `ponytail-core-registry` とルートの `Cargo.toml`・`Cargo.lock` を共有する。roadmap の境界の戦略どおり、消す行が別（tower-lsp と fast_radix_trie）なので同じウェーブに置いてよく、後から入る側が `Cargo.lock` を作り直す。`ponytail-dsl-parser` とはファイルが別（`partial.rs` はこちら）だが、診断は `parse_str` の `SyntaxError{line,column,message}` を読むので、あちらが誤りの型を削るときにこの形を保つことを両方の要件に書く。`ponytail-lua-debug` とは `book/src/debug/*.md` の別の節、`ponytail-manual-ci` とは `tech.md` の別の行。
  - ほか: `pest-2-9-4-upgrade`（`Cargo.lock` の版を広く動かし、`pasta_lsp` の WASM のビルドも触る）・`vsce-4-upgrade`（`package.json`・`package-lock.json`）と同じウェーブに置かない。0.5.0 以降で文法を足す `scene-anchor-link`（`visit_action.rs`・tmLanguage・`report.rs`）・`scene-exact-match`（`visit_action.rs`）・`novel-variable-scope`（`partial.rs`・tmLanguage・`visit_expr.rs`）とも重なる。
- **種別**: バグ（書き間違いが 1 つでシーンの全行に、ずれた位置で波線が付く）。監査の指摘より。
- **要件定義のモデル**: Opus（`host` の廃止は 2026-10-19 に決まっている）。
- **分割の案**: 今は不要。ギャップ分析で 20 を超えたら、「診断とサーバの撤去（`pasta_lsp`・`partial.rs`・`pasta_check` の 2 ファイル・`Cargo.lock`）」と「VSCode 拡張の色と整理（tmLanguage・`src/*.ts`・`package.json` の `host`）」の 2 本に分ける。
- **見つけた穴・古くなった記述**
  - 7 番の `release.yml:L329` は古い。crates.io へ出す 5 つは `release.yml` 294〜334 行で、`pasta_lsp` が無いのは同じ（10-24 のとおり）。
  - Scope の `tech.md`（L15・L65・L110・L123）は、今は 15 行・66〜69 行（tower-lsp は 69 行）・111 行・124 行。`structure.md`（L190-205）の `pasta_lsp` の木は 186〜205 行。

## 申し送り（ponytail-lua-debug を閉じた、2026-11-02）

- `ponytail-lua-debug` は、要件ディスカッションの後に閉じた（開発者の決定）。0.5.1 で VSCode デバッガー（DAP）の連携を捨て、SSP・areka の MCP 経由に移す方針が出たためである（roadmap の「統合して閉じたもの（2026-11-02）」）。`book/src/debug/*.md` の節を分け合う相手は、もう居ない。
- この spec の DAP の側の直し（`debugAttachTarget.ts`・`runSceneAtCursor.ts`・`attach.host`）も、0.5.1 で書き直すか消すことになる見込みである。絞るかどうかは、セッション「Pasta DSL MCP移行検討」と開発者が決める。
