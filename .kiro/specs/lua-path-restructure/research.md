# Research & Design Decisions: lua-path-restructure

## Summary
- **Feature**: lua-path-restructure
- **Discovery Scope**: Extension（既存システムの文字列/パス置換とディレクトリ移動）
- **Key Findings**:
  - 変更対象は単一関数 `default_lua_search_paths()` とそれを参照するテスト・設定・ドキュメント群
  - 新規依存関係・新規アーキテクチャパターンなし。全変更が既存パターンの踏襲
  - `context.rs` 内テストは汎用パス名テストのため変更不要、`lifecycle_test.rs` はデフォルトパス依存のため変更必須

## Research Log

### 拡張ポイントの特定
- **Context**: `default_lua_search_paths()` がデフォルト検索パスの唯一の定義元
- **Sources Consulted**: `crates/pasta_lua/src/loader/config.rs` L165-173
- **Findings**:
  - `LoaderConfig` の `lua_search_paths` フィールドに `#[serde(default)]` が付与されており、TOML未指定時に `default_lua_search_paths()` が呼ばれる
  - pasta.toml で明示指定すればデフォルト値は無視される（Req 1.3 の既存動作）
  - 関数シグネチャ・戻り値型に変更なし（`Vec<String>`）
- **Implications**: コア変更は関数本体の文字列値のみ。インターフェース変更なし

### テストコードの分類
- **Context**: テスト内の `"scripts"` 文字列が「デフォルトパス参照」か「任意のテスト値」かの判別
- **Findings**:
  - `context.rs` テスト（7箇所）: `LoaderContext::new()` に任意パスを渡す汎用テスト → **変更不要**
  - `lifecycle_test.rs`（4箇所）: `PastaLoader::load()` 使用、デフォルトパスに依存 → **変更必須**
  - `config_test.rs`（3箇所）: デフォルト値のアサーション → **変更必須**
  - その他テスト: `scripts/` ディレクトリのコピーヘルパー → `pasta_scripts/` に更新
- **Implications**: テスト変更は機械的な文字列置換だが、各テストの意図を理解した上での修正が必要

### hello.lua の用途調査
- **Context**: `hello.lua` がランタイムで必要か、テスト専用か
- **Findings**:
  - Rustコード・Luaランタイムからの `require` なし
  - `transpiler_test.lua` が `require("hello")` で3箇所参照（テスト用サンプルモジュール）
  - `.vscode/launch.json` がデバッグエントリーポイントとして参照
  - 配布物（updates2.dau, updates.txt）に含まれている
- **Implications**: テストフィクスチャ（`tests/fixtures/`）に移動し、配布物から除外

### .gitignore 確認
- **Context**: `pasta_scripts/` ディレクトリが .gitignore で除外されないことの確認
- **Findings**: .gitignore に `scripts`/`pasta_scripts` 関連パターンなし
- **Implications**: 問題なし

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A. 既存コンポーネント拡張 | 全変更が既存ファイルの修正 | 新規ファイルなし、git mv で履歴保持 | 修正箇所が多い（~30ファイル） | **選択** |

新規アーキテクチャパターンの導入は不要。全変更が文字列置換とディレクトリ移動。

## Design Decisions

### Decision: ディレクトリリネーム方式
- **Context**: `crates/pasta_lua/scripts/` を `pasta_scripts/` に移動する方法
- **Alternatives Considered**:
  1. `git mv` で一括リネーム
  2. 新規ディレクトリ作成＋ファイルコピー＋旧ディレクトリ削除
- **Selected Approach**: `git mv crates/pasta_lua/scripts crates/pasta_lua/pasta_scripts`
- **Rationale**: git の履歴追跡が維持される。単一コマンドで完了
- **Trade-offs**: なし（git mv が最適解）

### Decision: hello.lua の扱い
- **Context**: ランタイム不要の `hello.lua` をどう扱うか
- **Alternatives Considered**:
  1. `tests/fixtures/` に移動（テストからの参照が自然）
  2. 削除して `transpiler_test.lua` も削除
  3. `pasta_scripts/` に残すが配布物から除外
- **Selected Approach**: 削除（+ `transpiler_test.lua`, `init.lua` エントリ, `launch.json` デバッグ設定も削除）
- **Rationale**: `hello.lua` はランタイムとは無関係なサンプルファイル。テスト残存価値もないため、依存するものごと削除するのが最もクリーンな対応
- **Trade-offs**: テストカバレッジが1件減るが、hello.lua 自体に実装すべきロジックはないため問題なし

### Decision: README.md の内容方針
- **Context**: `pasta_scripts/` と `scripts/` の役割をユーザーに明示する方法
- **Selected Approach**: 各ディレクトリに README.md を配置
- **Rationale**: ファイルマネージャーや GitHub で直接確認でき、ドキュメントの分散を防ぐ
- **内容確定済み**:
  - `pasta_scripts/README.md`: 編集禁止の案内、`scripts/` での上書き方法
  - `scripts/README.md`: ユーザーカスタム用、優先順位の説明

## Risks & Mitigations
- **修正漏れリスク** — ギャップ分析で全影響箇所を網羅的に特定済み。`cargo test --all` で検証
- **配布物の不整合** — release.ps1 修正後に配布物を再生成して検証
- **外部ゴーストへの影響** — pasta.toml で `lua_search_paths` を明示指定しているゴーストは影響なし（Req 1.3）

## References
- ギャップ分析: `.kiro/specs/lua-path-restructure/gap-analysis.md`
- コア実装: `crates/pasta_lua/src/loader/config.rs` L165-173
- ステアリング: `.kiro/steering/structure.md`, `.kiro/steering/tech.md`
