# Brief: budouy-migration

## Problem
ゴーストのトーク表示では、日本語テキストを自然な分かち書き位置で改行する必要がある（`@pasta_sakura_script` の `break_lines` / `talk_to_script`）。現在この改行位置推定に `budoux 0.1.1` クレートを利用しているが、開発の主軸は後継と判断される `budouy` クレートへ移っている。古いクレートに依存し続けると、保守・機能追加・将来のバグ修正の恩恵を受けられず、技術的負債となる。

## Current State
- `Cargo.toml`（workspace）で `budoux = "0.1.1"` を定義し、`crates/pasta_lua` が `budoux.workspace = true` で参照。
- 実装は `crates/pasta_lua/src/sakura_script/` に集中:
  - `mod.rs`: `SakuraScriptState.budoux_model: budoux::Model` を保持し、`budoux::models::default_japanese_model().clone()` で初期化。
  - `line_breaker.rs::break_lines_impl(... model: &budoux::Model)` 内で `budoux::parse(model, &plaintext)` を呼び、分かち書き結果を幅閾値で改行挿入。
- テストは `crates/pasta_lua/tests/sakura_script/budoux_test.rs` ほか、`line_breaker.rs` 内のユニットテスト。
- steering / スキルドキュメントに "BudouX" の記載が複数あり（`tech.md`, `product.md`, `pasta-lua-coding`, `pasta-ghost-authoring`）。

## Desired Outcome
- `budoux` への依存が完全に除去され、`budouy` に置き換わっている。
- `break_lines` / `talk_to_script` の改行挙動が従来同等で、既存テストが（API差異の修正のみで）緑のまま通過する。
- ビルド・clippy・テストがクリーンに通る。
- 関連ドキュメント（steering / スキル）のクレート名・バージョン記載が更新されている。

## Approach
`budouy::Parser` ベースのAPIへ移行する。
- 依存定義を `budoux = "0.1.1"` → `budouy = "0.2.2"`（`vendored-models` feature 有効化）へ差し替え。
- `SakuraScriptState` は `budoux::Model` の保持をやめ、`budouy::model::load_default_japanese_parser()` が返す `budouy::Parser` を保持する（パーサーが模型を所有する設計に追従）。
- `break_lines_impl` のシグネチャを `model: &budoux::Model` → `parser: &budouy::Parser` に変更し、`budoux::parse(model, &plaintext)` を `parser.parse(&plaintext)` に置き換える。
- 出力 chunk の型（`Vec<&str>`）に合わせ、幅計算ループを最小限調整。
- テストは API 呼び出し差分（model 取得・parse 呼び出し）のみ修正し、入力・期待値・テストケース構造は流用する。

なぜこの方法か: 変更は依存定義1箇所と `sakura_script` モジュール内の3点に局所化でき、改行アルゴリズム（トークン化・幅閾値ロジック）はそのまま再利用できる。`vendored-models` により外部模型ファイルの配布も不要。

## Scope
- **In**:
  - workspace `Cargo.toml` の依存差し替え（`budoux` → `budouy`、feature 指定）
  - `crates/pasta_lua/src/sakura_script/mod.rs` と `line_breaker.rs` の API 移行
  - 既存テスト（`budoux_test.rs`、`line_breaker.rs` ユニットテスト）の API 差分修正と流用
  - steering / スキルドキュメント内のクレート名・バージョン記載の更新
- **Out**:
  - 改行アルゴリズム（幅閾値ロジック、タグ保持処理）の挙動変更・改善
  - 新言語モデル（中国語・タイ語）対応や HTML 処理・WASM など budouy の追加機能の採用
  - `break_lines` / `talk_to_script` の Lua 公開 API（引数・戻り値）の変更

## Boundary Candidates
- 依存定義の差し替え（Cargo.toml）
- ランタイム実装の API 移行（sakura_script モジュール）
- テストの API 差分修正
- ドキュメント記載の同期更新

## Out of Boundary
- 改行品質・分割精度のチューニングや評価
- budoux/budouy 以外の改行ライブラリ比較・選定
- Lua スクリプト側（`pasta_scripts` 等）の利用コード変更（公開 API 不変のため不要の想定）

## Upstream / Downstream
- **Upstream**: `crates/pasta_lua/src/sakura_script`（改行ロジック本体）、workspace `Cargo.toml`（依存集中管理）
- **Downstream**: `@pasta_sakura_script` モジュールを利用する Lua スクリプト全般（`talk_to_script` 経由の自動改行）。公開 API は不変のため挙動互換が前提。

## Existing Spec Touchpoints
- **Extends**: なし（既存 spec のドメイン拡張ではなく、独立した依存移行）
- **Adjacent**: `ukagaka-desktop-mascot`（`budoux-line-breaker` 機能を内包する母体 spec）。本 spec はその実装に使われるクレートの差し替えであり、機能境界は重複しない。

## Constraints
- ライセンス: budouy は Apache-2.0。workspace は `MIT OR Apache-2.0` で互換、GPL 汚染なし。
- MSRV: budouy は Rust 1.88.0 を要求。本 workspace は edition 2024（Rust 1.85+）を使用済みのため実害は小さいが、CI/開発環境のツールチェーンが 1.88.0 以上であることを確認する。
- 改行挙動は従来同等であること（既存テストの期待値を維持。模型差で分割位置が変わる場合はテスト期待値の妥当性を個別判断）。
- budoux への依存は完全除去すること（`Cargo.lock` を含む）。
