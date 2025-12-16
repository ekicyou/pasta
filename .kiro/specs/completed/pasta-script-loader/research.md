# Research & Design Decisions - pasta-script-loader

---
**Purpose**: ディレクトリベーススクリプトローダー機能の設計判断根拠を記録

**Usage**:
- Gap分析結果とアーキテクチャ調査の要約
- 設計決定のトレードオフと根拠
- 将来のメンテナンス・リファクタリング時の参照資料
---

## Summary
- **Feature**: `pasta-script-loader`
- **Discovery Scope**: Extension（既存システム拡張）
- **Key Findings**:
  1. 既存`PastaEngine`は文字列ベース初期化のみ、ディレクトリ走査ロジックは未実装
  2. areka-P0-script-engine規約（`dic/` + `main.rune`）への準拠が必要
  3. 既存コンポーネント（`register_labels()`, `PARSE_CACHE`, Rune統合）は再利用可能

## Research Log

### 1. 既存PastaEngine初期化フロー調査
- **Context**: ディレクトリローダー追加に向けた既存実装の理解
- **Sources Consulted**: 
  - `crates/pasta/src/engine.rs` (L1-350)
  - `crates/pasta/src/cache.rs`
  - `crates/pasta/src/runtime/labels.rs`
- **Findings**:
  - `PastaEngine::new(script: &str)` は単一文字列のみ対応
  - 内部フロー: `parse_str()` → `Transpiler::transpile()` → `register_labels()` → Runeコンパイル
  - 同名ラベルには自動連番付与（`挨拶_0`, `挨拶_1`）- Req 4に適合
  - グローバル静的キャッシュ `PARSE_CACHE: OnceLock<ParseCache>` - Req 8に適合
- **Implications**: 
  - 新規コンストラクタ`from_directory()`追加が最適（既存`new()`は不変）
  - `register_labels()`はそのまま再利用可能

### 2. ファイル配置規約（areka-P0-script-engine）
- **Context**: Req 2「DSL/Runeファイル配置ルール」の技術的要件確認
- **Sources Consulted**: 
  - `requirements.md` Req 2
  - `gap-analysis.md` Section 1
- **Findings**:
  - 必須ディレクトリ構造: `<root>/dic/*.pasta`, `<root>/main.rune`
  - `.pasta`ファイル: `dic/`配下を再帰探索、`_`プレフィックス・隠しファイル除外
  - `main.rune`: Runeエントリーポイント（必須ファイル）
  - `.rn`モジュール: Runeコンパイラの`mod`文解析に委譲
- **Implications**: 
  - `DirectoryLoader`モジュールでディレクトリ走査を独立実装
  - `dic/`不在時は`DicDirectoryNotFound`エラー
  - `main.rune`不在時は`MainRuneNotFound`エラー

### 3. Rune Sourcesモジュール統合
- **Context**: 複数ファイル（DSLトランスパイル + main.rune）のRune統合方法
- **Sources Consulted**: 
  - https://docs.rs/rune/latest/rune/struct.Sources.html
  - `gap-analysis.md` Section 5 Research Item 3
- **Findings**:
  - `Sources::insert()` 順序は影響なし（Runeは全ソース収集後に依存解析）
  - 推奨追加順序: トランスパイル済みDSL ("entry") → main.rune（可読性）
  - `mod`解決はRuneコンパイラ責務（エンジンは`main.rune`のみ追加）
  - `Source::from_path(&path)` でファイルから直接ソース作成可能
- **Implications**: 
  - 複数.pastaファイルはトランスパイル後に単一Runeソースとして統合
  - `main.rune`は別ソースとして追加、Runeが`mod`依存を自動解決
  - 循環依存検出はRuneコンパイラが担当→`RuneCompileError`でラップ

### 4. エラー収集戦略
- **Context**: Req 3「複数ファイルパースエラー収集」の実装方法
- **Sources Consulted**: 
  - `crates/pasta/src/error.rs`
  - `gap-analysis.md` Section 5 Research Item 2
- **Findings**:
  - 既存エラー型: 7種類（`ParseError`, `IoError`, `RuneCompileError`等）
  - 追加必要: 4エラー型（`DirectoryNotFound`, `DicDirectoryNotFound`, `MainRuneNotFound`, `MultipleParseErrors`）
  - エラー収集パターン: `Vec<Result<T, E>>` → `partition()` で成功/失敗分離
  - IoErrorのみ早期停止（ファイル読み込み失敗は致命的）
- **Implications**: 
  - `MultipleParseErrors(Vec<PastaError>)` でパースエラーを集約
  - `pasta_errors.log`生成後にエラー返却（開発者体験優先）
  - `std::error::Error`トレイト実装継続

### 5. パフォーマンス（キャッシュ戦略）
- **Context**: Req 8「パフォーマンス最適化」の既存実装活用
- **Sources Consulted**: 
  - `crates/pasta/src/cache.rs`
  - `gap-analysis.md` Section 2
- **Findings**:
  - `PARSE_CACHE: OnceLock<ParseCache>` - グローバル静的キャッシュ
  - FNV-1aハッシュでスクリプト内容をキー化
  - 全PastaEngineインスタンス間で共有
  - 注意: `pasta-engine-independence`スペックで変更予定（インスタンス所有へ）
- **Implications**: 
  - 現時点では既存`PARSE_CACHE`を利用（設計はこれを前提）
  - 将来のキャッシュ移行には影響を受ける可能性（設計時に疎結合を意識）
  - ファイルパス→ファイル内容→ハッシュ の順で処理

### 6. テストインフラ
- **Context**: Req 5, 6「テスト用ディレクトリ・統合テスト」の設計
- **Sources Consulted**: 
  - `crates/pasta/tests/` ディレクトリ構造
  - `crates/pasta/Cargo.toml` (`tempfile` 依存)
- **Findings**:
  - 既存: `examples/scripts/` にサンプルスクリプト（6ファイル）
  - `tempfile = "3"` は`[dev-dependencies]`に存在
  - `tests/fixtures/test-project/` は未作成
- **Implications**: 
  - 静的フィクスチャ（`tests/fixtures/test-project/`）を新規作成
  - `tempfile`でテスト用一時ディレクトリ作成（エラーログ書き込みテスト）
  - 決定性確保: ファイル順序非依存なテスト設計

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: EngineInput enum | 単一`new()`でEnum分岐 | APIシンプル | 型複雑化、ドキュメント混在 | 却下 |
| **B: from_directory (推奨)** | 新規コンストラクタ + DirectoryLoader | 責務明確、独立テスト可能、後方互換性100% | 新規ファイル追加 | 採用 |
| C: Hybridプライベート | `engine.rs`内にヘルパー追加 | 新規ファイル不要 | テスト困難、ファイル肥大化 | 却下 |

## Design Decisions

### Decision: 新規コンストラクタ + 独立Loaderモジュール (Option B)
- **Context**: 既存`new()`を維持しつつディレクトリ対応を追加する方法の選択
- **Alternatives Considered**:
  1. Option A - `EngineInput` enumで分岐 → 型複雑化、ドキュメント混在
  2. Option C - `engine.rs`内プライベートヘルパー → テスト困難、ファイル肥大化
- **Selected Approach**: Option B - `PastaEngine::from_directory()` + `loader.rs`モジュール
- **Rationale**: 
  - 責務分離（DirectoryLoader = ファイル収集、PastaEngine = コンパイル・実行）
  - 既存`new()`は完全不変（後方互換性100%）
  - DirectoryLoaderの単独ユニットテスト可能
  - 将来の入力ソース拡張（アーカイブ、HTTP等）に統一パターン適用可能
- **Trade-offs**: 
  - ✅ 保守性・テスタビリティ向上
  - ⚠️ 新規ファイル追加（`loader.rs`）
- **Follow-up**: `engine.rs`サイズ監視（現在1040行、拡張後も1200行以下が目標）

### Decision: MultipleParseErrors集約エラー型
- **Context**: 複数ファイルのパースエラーを効率的に報告する方法
- **Alternatives Considered**:
  1. 最初のエラーで即座に停止 → 開発者体験悪化（1エラー修正→再実行→次エラー発見）
  2. 全エラー収集後にログ出力 + 集約エラー返却 → 開発者体験向上
- **Selected Approach**: Option 2 - 全エラー収集 + `MultipleParseErrors(Vec<PastaError>)`
- **Rationale**: 
  - 開発者が全エラーを一括確認可能
  - `pasta_errors.log`に詳細出力、返却エラーはサマリー
- **Trade-offs**: 
  - ✅ 開発効率向上
  - ⚠️ IoError（ファイル読み込み失敗）のみ早期停止（致命的エラー）
- **Follow-up**: エラーログフォーマットの詳細仕様をdesign.mdで確定

### Decision: main.runeのみSources追加
- **Context**: Runeモジュール（`.rn`ファイル）の統合方法
- **Alternatives Considered**:
  1. エンジンが全`.rn`ファイルを探索して追加 → Runeの`mod`システムと競合
  2. `main.rune`のみ追加、`mod`解決はRuneに委譲 → Runeの設計意図に沿う
- **Selected Approach**: Option 2 - `main.rune`のみ`Sources::insert()`
- **Rationale**: 
  - Runeコンパイラが`mod`文を解析し依存ファイルを自動探索
  - 循環依存検出もRuneが担当
  - エンジンの責務を明確化（複雑度削減）
- **Trade-offs**: 
  - ✅ 実装シンプル化
  - ⚠️ Runeのエラーメッセージに依存（`RuneCompileError`でラップ）
- **Follow-up**: Runeコンパイルエラー時のユーザー向けメッセージ改善検討

### Decision: 絶対パスのみ受け入れ
- **Context**: Req 1「絶対パスのみ受け付け」の設計根拠
- **Alternatives Considered**:
  1. 相対パスも受け入れ、内部で`canonicalize()` → カレントディレクトリ依存のバグリスク
  2. 絶対パスのみ受け入れ → 明示的で予測可能
- **Selected Approach**: Option 2 - 絶対パス強制
- **Rationale**: 
  - 呼び出し元の責務でパス解決（カレントディレクトリ依存を排除）
  - エラーメッセージが明確（「相対パスは拒否されました」）
  - デバッグ時のパス特定が容易
- **Trade-offs**: 
  - ✅ 予測可能な動作
  - ⚠️ 呼び出し元で`canonicalize()`が必要
- **Follow-up**: ドキュメントに絶対パス要件を明記

## Risks & Mitigations
- **Risk 1**: ファイル探索順序がOS依存でテスト非決定的 → **Mitigation**: 順序非依存なテスト設計、アサートは`contains()`やセット比較を使用
- **Risk 2**: `pasta_errors.log`書き込み権限エラー → **Mitigation**: IoError即座返却、権限エラー時はログ出力をスキップ（オプション）
- **Risk 3**: `pasta-engine-independence`スペックでキャッシュ構造変更 → **Mitigation**: 現設計は既存キャッシュAPIを使用、将来移行時はインターフェース維持

## References
- [gap-analysis.md](./gap-analysis.md) - 詳細なギャップ分析
- [requirements.md](./requirements.md) - 要件定義（9要件、EARS形式）
- [Rune Sources API](https://docs.rs/rune/latest/rune/struct.Sources.html) - Runeソースコレクション
- [steering/structure.md](../../steering/structure.md) - プロジェクト構造規約
