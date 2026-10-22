# Brief: ponytail-editor-tooling

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。版には縛らない。0.4.2 を出した後、0.5.0 の spec が並べられずに空いた所へ入れる（2026-10-22 開発者の決定。2026-10-19 の「0.4.3 に入れる」から改めた）。同じ監査の 10 本と同じウェーブで並走できる。

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
