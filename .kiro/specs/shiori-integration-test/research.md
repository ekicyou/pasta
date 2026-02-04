# Research & Design Decisions: shiori-integration-test

## Summary
- **Feature**: `shiori-integration-test`
- **Discovery Scope**: Extension（既存テストパターンを拡張）
- **Key Findings**:
  - 既存の `copy_fixture_to_temp()` パターンを拡張して hello-pasta 対応可能
  - TalkConfig は実装済みで pasta.toml の `[talk]` セクション追加のみで動作
  - OnBoot シーンは2つ存在し、決定的テストには1つへの削減が必須

## Research Log

### pasta_shiori テストパターン調査
- **Context**: 既存のインテグレーションテストパターンを確認
- **Sources Consulted**: 
  - `crates/pasta_shiori/tests/common/mod.rs`
  - `crates/pasta_shiori/tests/shiori_lifecycle_test.rs`
- **Findings**:
  - `copy_fixture_to_temp(fixture_name)` は `tests/fixtures/` と `tests/support/` を前提
  - `profile/` ディレクトリはスキップされる設計
  - PastaShiori は `pasta::{PastaShiori, Shiori}` として再エクスポート
  - テストは `mod common;` で共通モジュールを読み込む
- **Implications**: 
  - hello-pasta 用に新しいコピー関数が必要
  - 既存の `copy_dir_recursive()` は再利用可能

### hello-pasta ゴースト構造調査
- **Context**: テスト対象のゴースト構造を確認
- **Sources Consulted**: 
  - `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/`
  - `pasta.toml`, `dic/boot.pasta`
- **Findings**:
  - `scripts/` ディレクトリが含まれる（support 不要）
  - OnBoot シーンが2つ定義されている（ランダム選択される）
  - `[talk]` セクションは未設定
  - `dic/*.pasta` パターンでパスタファイルを読み込む
- **Implications**: 
  - boot.pasta の修正が必要（OnBoot を1つに）
  - pasta.toml に `[talk]` セクション追加

### TalkConfig 実装状況調査
- **Context**: ウェイト設定の実装状況を確認
- **Sources Consulted**: 
  - `crates/pasta_lua/src/loader/config.rs`
- **Findings**:
  - `TalkConfig` 構造体が実装済み
  - デフォルト値: normal=50, period=1000, comma=500, strong=500, leader=200
  - `PastaConfig::talk()` メソッドで取得可能
- **Implications**: 
  - pasta.toml への追加のみで機能する
  - 新規実装不要

### OnBoot リクエスト形式調査
- **Context**: 実際のSSPからのリクエスト形式を確認
- **Sources Consulted**: 
  - ユーザー提供のリクエスト例
- **Findings**:
  - SHIORI/3.0 プロトコル形式
  - ヘッダー: Charset, Sender, SecurityLevel, ID, Reference0
  - 改行は CRLF、終端は CRLFCRLF
- **Implications**: 
  - テストでは完全なリクエスト形式を使用可能
  - 既存テストより実際の運用に近い検証が可能

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: 既存拡張 | common に関数追加 | パターン一貫性 | クレート間参照 | 推奨 |
| Option B: fixture コピー | fixtures/ に配置 | 既存パターン踏襲 | 重複管理コスト | 非推奨 |
| Option C: ハイブリッド | A + ゴースト修正 | 実運用に近い | setup.bat 依存 | 採用 |

## Design Decisions

### Decision: hello-pasta 直接参照方式
- **Context**: テスト環境で hello-pasta をどのように参照するか
- **Alternatives Considered**:
  1. fixtures/ にコピーを配置
  2. common モジュールにコピー関数追加
- **Selected Approach**: common モジュールに `copy_sample_ghost_to_temp()` 関数を追加
- **Rationale**: 
  - ファイル重複を避け保守コスト削減
  - 実際のゴースト定義を直接テストできる
  - CARGO_MANIFEST_DIR からの相対パスで解決可能
- **Trade-offs**: 
  - クレート間依存が発生
  - パス解決がハードコード
- **Follow-up**: CI 環境での動作確認

### Decision: OnBoot シーン削減
- **Context**: ランダム選択によるテスト不安定性の排除
- **Alternatives Considered**:
  1. 単語辞書呼び出しを含む方を残す
  2. 単語辞書呼び出しを含まない方を残す
- **Selected Approach**: 単語辞書呼び出しを含まない方を残す
- **Rationale**: 
  - `＠起動挨拶` はランダム選択を含む
  - 決定的な出力が必要
- **Trade-offs**: 
  - 単語辞書機能のテストは別途必要
- **Follow-up**: 単語辞書テストは別仕様で対応

### Decision: [talk] セクションのデフォルト値使用
- **Context**: テスト期待値の確定
- **Selected Approach**: TalkConfig のデフォルト値をそのまま使用
- **Rationale**: 
  - デフォルト値が既に定義済み
  - カスタム値を設定する必要なし
- **Trade-offs**: 
  - ウェイト値のカスタム検証は行わない

## Risks & Mitigations
- **Risk 1: setup.bat 未実行でのテスト失敗** — CI 設定で事前実行を保証
- **Risk 2: クレート間パス変更** — CARGO_MANIFEST_DIR 使用で相対パス解決
- **Risk 3: さくらスクリプト出力形式変更** — 部分一致検証で柔軟性確保

## References
- [既存テスト: shiori_lifecycle_test.rs](../../crates/pasta_shiori/tests/shiori_lifecycle_test.rs)
- [TalkConfig 実装](../../crates/pasta_lua/src/loader/config.rs)
- [hello-pasta ゴースト](../../crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/)
