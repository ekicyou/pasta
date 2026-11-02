# pasta_lsp

Pasta DSL の WebAssembly の解析器クレート。VSCode 拡張に埋め込む（`publish = false`）。

`*.pasta` ファイルに対するセマンティックハイライトと構文の診断、ゴースト全体の検査（`pasta_check` の核）を WASM の関数として提供します。文書の状態は持たず、呼ばれるたびに渡された文字を解きます。

## アーキテクチャ

```
┌─────────────────────────────────────────────────┐
│  Transport (transport.rs)                        │
│  - WASM の入口 3 つ（wasm32 のときだけ）         │
│    - wasm_analyze: 解析（トークン＋診断）        │
│    - wasm_select_ghost_files: 辞書ファイルの選び出し │
│    - wasm_validate_ghost: ゴースト全体の検査     │
│  - 入力の形が違う・直列化に失敗したときは null   │
│  - panic は包まない（trap を拡張の try/catch が受ける）│
├─────────────────────────────────────────────────┤
│  AnalysisEngine (analysis/)                     │
│  - mod.rs: AST → セマンティックトークン変換・診断の型 │
│  - token_types.rs: 17 トークンタイプ / 3 モディファイア │
│  - visit_scope.rs / visit_action.rs / visit_expr.rs: AST ビジター │
│  - text_utils.rs: 行の取り出し・UTF-16 位置・行末のコメント │
│  - 構文の誤りは pasta_check の syntax_finding から診断 1 件 │
│  - 全体が解けないときの色は部分パース (parse_str_partial) │
└─────────────────────────────────────────────────┘
         ↓ 依存
  pasta_dsl (PEG パーサー、AST、部分パース API)
  pasta_check (検査の核、既定 feature 無し)
```

## セマンティックトークンタイプ

| トークンタイプ | 対応する Pasta 構文要素             |
| -------------- | ----------------------------------- |
| comment        | コメント行 (`＃` / `#`)             |
| namespace      | グローバルシーン (`＊` / `*`)       |
| scene          | ローカルシーン (`・` / `-`)         |
| decorator      | 属性定義 (`＆` / `&`)               |
| word           | 単語定義 (`＠` / `@`)               |
| variable       | 変数参照 (`＄` / `$`)               |
| call           | Call文 (`＞` / `>`)                 |
| actor          | アクター定義 (`％` / `%`)           |
| actorName      | アクション行のアクター名            |
| codeBlock      | Lua コードブロック                  |
| talk           | 文字列リテラル / Talk テキスト      |
| sakuraScript   | さくらスクリプトタグ                |
| escape         | エスケープシーケンス                |
| operator       | コロン区切り (`：` / `:`)・代入 (`＝` / `=`)・括弧・二項演算子 |
| number         | 数値リテラル                        |
| cueMarker      | キューコマンドマーカー (`！` / `!`) |
| cueCommand     | キューコマンド名                    |

## 公開 API

```rust
// 解析（純粋な関数・共有する状態なし）
pub struct AnalysisEngine;
impl AnalysisEngine {
    pub fn analyze(source: &str) -> AnalysisResult;
}

// WASM の入口（wasm32 のときだけ）
pub fn wasm_analyze(source: &str) -> JsValue;
pub fn wasm_select_ghost_files(input: JsValue) -> JsValue;
pub fn wasm_validate_ghost(input: JsValue, files: JsValue) -> JsValue;
```

## ビルド

### ネイティブ（テスト）

```sh
cargo build -p pasta_lsp
cargo test -p pasta_lsp
```

### WASM

```sh
rustup target add wasm32-unknown-unknown
cargo build -p pasta_lsp --target wasm32-unknown-unknown --release
```

拡張に入れる WASM は `editors/vscode/scripts/build-wasm.ps1`（wasm-pack）で作ります。

## テスト

139 テスト（インラインテスト 4 + 統合テスト 135）:

| テストファイル                  | テスト数 | 内容                                   |
| ------------------------------- | -------- | -------------------------------------- |
| analysis インラインテスト       | 2        | 位置の無い誤り・行末を越える桁の切り詰め |
| transport インラインテスト      | 2        | JSON 形状・ゴースト検査の入出力の形    |
| semantic_token_test.rs          | 25       | 17 トークンタイプ識別・行末のコメント  |
| fullwidth_halfwidth_test.rs     | 5        | 全角/半角マーカー同等認識              |
| japanese_identifier_test.rs     | 5        | 日本語識別子のトークン化               |
| utf16_conversion_test.rs        | 12       | UTF-8→UTF-16 位置変換                  |
| diagnostics_test.rs             | 12       | 構文の診断（保存時の検査と同じ位置・文） |
| partial_token_test.rs           | 7        | 部分パース→トークン提供                |
| cue_command_token_test.rs       | 10       | キューコマンドトークン生成             |
| var_set_token_test.rs           | 10       | 変数代入行のトークン生成               |
| var_set_multibyte_panic_test.rs | 3        | 変数代入行の多バイト文字で panic しない |
| binary_chain_token_test.rs      | 13       | 二項演算の連鎖トークン                 |
| code_block_token_test.rs        | 7        | Lua コードブロックのフェンス           |
| dynamic_ref_token_test.rs       | 8        | 動的な単語参照・関数呼び出し           |
| legend_sync_test.rs             | 2        | 凡例が拡張の凡例と一致                 |
| analysis_test.rs                | 15       | コメント走査・改行正規化・解析結果     |
| analyze_robustness_test.rs      | 1        | 敵対的入力コーパスの no-panic 境界     |

## 依存関係

- `pasta_dsl` - Pasta DSL パーサー（PEG ベース、部分パース API）
- `pasta_check`（既定 feature 無し）- 構文の診断の文とゴースト全体の検査
- `serde 1` - 解析結果のシリアライズ定義
- `wasm-bindgen 0.2` (WASM ターゲット条件コンパイル)
- `serde-wasm-bindgen 0.6` - Rust→JS型変換（WASMエントリポイント）
- dev: `serde_json 1` - JSON 形状検証（テスト専用）

バージョンは workspace で一元管理。

## ライセンス

MIT（ワークスペース共通）
