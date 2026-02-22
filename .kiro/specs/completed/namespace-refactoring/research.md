# Research & Design Decisions

## Summary
- **Feature**: `namespace-refactoring`
- **Discovery Scope**: Extension（既存クレート構造のリファクタリング、新機能追加なし）
- **Key Findings**:
  - pasta_lua の 36 テストファイルは 14 機能ドメインに分類可能。16/36 ファイル (44%) が `mod common;` を使用
  - Rust 統合テストのサブディレクトリ化には `tests/<category>/main.rs` パターンが必要。`#[path = "../common/mod.rs"]` で共有ヘルパーを参照可能
  - insta スナップショットはテストファイル移動時にパス・ファイル名が変わるため、再生成が必要
  - `pasta_lua/src/` 直下の `string_literalizer.rs`, `normalize.rs` は `code_gen/` への移動候補だが、現時点では移動不要と判断

## Research Log

### Rust 統合テストのサブディレクトリメカニクス
- **Context**: Req1・Req2 で `tests/` 直下のファイルをサブディレクトリに整理する必要がある。Rust の統合テスト検出ルールの確認が必要
- **Sources Consulted**: Rust Reference (Module System), Cargo Book (Integration Tests), プロジェクト内既存パターン
- **Findings**:
  - `tests/` 直下の各 `.rs` ファイルは独立クレートとしてコンパイルされる
  - サブディレクトリ内のファイルは自動検出されない。`tests/<dir>/main.rs` を配置することで、そのディレクトリが 1 つの独立テストクレートとなる
  - `tests/<dir>/main.rs` から `mod common;` と書くと `tests/<dir>/common/` を探すため、ルートの `tests/common/` は直接参照不可
  - `#[path = "../common/mod.rs"] mod common;` で親ディレクトリの common を参照可能。`common/mod.rs` 内の `pub mod e2e_helpers;` は `common/` 基準で解決されるため正常動作
- **Implications**: 各サブディレクトリの `main.rs` に `#[path]` を記述する統一テンプレートが必要

### pasta_lua テストファイル 36 件の分類
- **Context**: Req2 の機能ドメイン別サブディレクトリ設計のための全件調査
- **Sources Consulted**: 全 36 テストファイルの `use` 宣言、`mod common;` 使用有無、テスト内容を精査
- **Findings**:

| ドメイン | ファイル数 | `mod common;` 使用 | 主要ファイル |
|---------|-----------|-------------------|------------|
| Transpiler | 6 | 3 | transpiler_basic_test, transpiler_comparison_test, transpiler_scene_test, transpiler_snapshot_test, actor_word_dictionary_test, fallback_search_integration_test |
| Loader | 6 | 2 | loader_cache_test, loader_config_test, loader_lifecycle_test, loader_startup_test, config_actors_initialization_test, lua_passthrough_test |
| SHIORI | 4 | 4 | shiori_event_dispatch_test, shiori_event_handler_test, shiori_res_test, virtual_event_config_test, virtual_event_dispatch_test |
| Runtime E2E | 3 | 3 | finalize_scene_test, runtime_scene_test, runtime_syntax_test |
| Logging | 3 | 0 | log_integration_test, log_module_test, log_stack_level_test |
| SakuraScript | 2 | 2 | sakura_script_basic_test, sakura_script_output_test |
| Search | 2 | 1 | scene_search_test, search_module_test |
| Stdlib | 2 | 0 | stdlib_modules_test, stdlib_regex_test |
| Lua 基盤 | 2 | 0 | japanese_identifier_test, ucid_test |
| CodeGen | 1 | 0 | code_generator_test |
| Runtime | 1 | 0 | runtime_test |
| Persistence | 1 | 1 | persistence_integration_test |
| Encoding | 1 | 0 | pasta_lua_encoding_test |
| Lua 単体テスト | 1 | 0 | lua_unittest_runner |

- **Implications**: 14 ドメインをそのままサブディレクトリにすると過剰分割。類似ドメインを統合して 7〜9 サブディレクトリに集約すべき

### common/mod.rs のヘルパー共有分析
- **Context**: サブディレクトリ化後の common 共有設計
- **Sources Consulted**: common/mod.rs (209行), common/e2e_helpers.rs (210行), 全 16 参照ファイル
- **Findings**:
  - 全 11 関数 + 1 サブモジュール（e2e_helpers: 4 関数）が純関数・ファクトリ関数で構成。状態保持なし
  - ヘルパー重複が 3 ファイルで発見: `config_actors_initialization_test.rs`, `lua_passthrough_test.rs`, `shiori_res_test.rs` が `copy_dir_recursive` や `create_empty_context` を自前定義
  - `stdlib_modules_test.rs`, `stdlib_regex_test.rs` も `create_empty_context` を重複定義
- **Implications**: サブディレクトリ化と同時に重複ヘルパーを `common/` に統合し、`#[path]` 参照に切り替えることで重複排除も実現可能

### insta スナップショットのパス依存性
- **Context**: `transpiler_snapshot_test.rs` を `transpiler/` サブディレクトリに移動した場合の影響
- **Sources Consulted**: insta ドキュメント、既存スナップショットファイル
- **Findings**:
  - insta はテストファイルのディレクトリ + モジュール名でスナップショットパスを決定
  - 移動により: `tests/snapshots/transpiler_snapshot_test__*.snap` → `tests/transpiler/snapshots/snapshot_test__*.snap` に変更
  - `.snap` ファイル内の `source:` メタデータも更新が必要
  - 最もシンプルな対応: 移動後に `cargo insta test --review` で全スナップショットを再生成
- **Implications**: スナップショット再生成を実装タスクに含める必要あり

### pasta_lua/src/ ルートレベルファイル評価
- **Context**: Req4 のソースモジュール名前空間レビュー
- **Sources Consulted**: 各ファイルの依存関係・責務を精査
- **Findings**:

| ファイル | 行数 | 判定 | 理由 |
|---------|------|------|------|
| transpiler.rs | 425 | ルート維持 | クレートの中核公開 API |
| context.rs | 341 | ルート維持 | 複数モジュール（transpiler, code_gen, runtime）が参照するクロスカッティング型 |
| config.rs | 109 | ルート維持 | クレート全体で使用される設定型 |
| string_literalizer.rs | 202 | 移動候補 | code_gen の補助だが単独で完結。現時点では移動不要 |
| normalize.rs | 195 | 移動候補 | トランスパイル出力の後処理。現時点では移動不要 |
| error.rs | - | ルート維持 | クレート共通エラー型 |

- **Implications**: `string_literalizer.rs` と `normalize.rs` は将来的に `code_gen/` への移動も検討可能だが、現在の粒度では問題なし。Req4 のレビュー結果として「現状維持」を推奨

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: `main.rs` パターン（採用） | `tests/<category>/main.rs` + サブモジュール | Cargo の公式対応パターン。`cargo test` 完全互換。ドメイン別グルーピングで一覧性向上 | 各 main.rs で common をコンパイルするオーバーヘッド（微小）。`#[path]` の見慣れなさ | プロジェクト内で `#[path]` は既に使用実績あり |
| B: フラット維持 + ドキュメント | ファイル移動せず、命名規則とドキュメントで整理 | 変更リスクゼロ | 36 ファイルの一覧性は改善しない。根本解決にならない | 却下 |
| C: テスト用ヘルパークレート | `common/` を独立クレートに分離 | コンパイル重複解消 | ヘルパー 11 関数程度にはオーバーエンジニアリング | 却下 |

## Design Decisions

### Decision: テストサブディレクトリの粒度設計
- **Context**: 14 の論理ドメインをそのままサブディレクトリにすると、1 ファイルのみのディレクトリが多数発生する
- **Alternatives Considered**:
  1. 14 ドメインそのまま — 論理的に厳密だが過剰分割（1 ファイルディレクトリ×5）
  2. 関連ドメイン統合で 7 ディレクトリ + ルート残留 — 実用的バランス
  3. 大分類 4 ディレクトリ（transpiler, runtime, loader, misc） — 粗すぎて検索性低下
- **Selected Approach**: 関連ドメイン統合で 7〜8 サブディレクトリ + フラット残留ファイル
- **Rationale**:
  - 3 ファイル以上のドメインはサブディレクトリ化する価値がある
  - 単独ファイル（code_generator_test, runtime_test, persistence_integration_test, pasta_lua_encoding_test）は類似ドメインに統合またはフラット残留
  - `lua_unittest_runner.rs` は独立性が高く、フラット残留が適切
- **Trade-offs**: 統合による論理的純粋さの低下 vs 実用的なディレクトリ数の抑制
- **Follow-up**: 具体的なファイル→ディレクトリマッピングは design.md で確定

### Decision: スナップショット移動方式
- **Context**: `transpiler_snapshot_test.rs` を `transpiler/` に移動するとスナップショットが壊れる
- **Alternatives Considered**:
  1. スナップショットファイルを手動でリネーム＆移動
  2. `insta::Settings` でカスタムパスを設定して旧パスを維持
  3. 移動後に `cargo insta test --review` で再生成
- **Selected Approach**: 移動後に `cargo insta test --review` で再生成
- **Rationale**: 最もシンプルで、スナップショット内容自体は変わらないため再生成が安全。手動リネームはエラーの温床
- **Trade-offs**: CI で一時的にスナップショット不一致が発生するが、同一 PR 内で解決

### Decision: src/ ルートファイルの配置
- **Context**: `string_literalizer.rs` と `normalize.rs` が `code_gen/` の責務に近い
- **Selected Approach**: 現状維持（ルートに残す）
- **Rationale**: 両ファイルは単独で完結した機能であり、移動しても依存関係の改善は限定的。リファクタリングスコープを抑制し、テスト名前空間整理に集中する

### Decision: ヘルパー重複の解消方針
- **Context**: 3〜5 ファイルが `common/` と重複するヘルパーを自前定義している
- **Selected Approach**: サブディレクトリ化と同時に、重複ヘルパーを `common/` に統合し `#[path]` 参照に切り替え
- **Rationale**: サブディレクトリ化により `#[path]` 導入が必須になるため、重複排除のコストが事実上ゼロ
- **Trade-offs**: 一部テストファイルの自己完結性が低下するが、DRY 原則に合致

## Risks & Mitigations
- **スナップショット破損**: `cargo insta test --review` で再生成。CI での検証を実装タスクに含める
- **`#[path]` パスの誤り**: 各 `main.rs` のテンプレートを統一し、コピペミスを防止。`cargo test` で即座に検出可能
- **コンパイル時間微増**: 各 `main.rs` が独立クレートとして `common/` を個別コンパイルするが、影響は微小（現状もフラット構成で同様の挙動）
- **git blame 履歴の分断**: `git mv` を使用してファイル移動し、リネーム検出を維持

## References
- [Rust Reference: Modules](https://doc.rust-lang.org/reference/items/modules.html) — `#[path]` 属性の仕様
- [Cargo Book: Integration Tests](https://doc.rust-lang.org/cargo/guide/tests.html) — `tests/` ディレクトリの検出ルール
- [insta: Snapshot Location](https://insta.rs/docs/snapshot-files/) — スナップショットファイルの配置規則
- [pasta_core/src/registry/scene_table_tests.rs](crates/pasta_core/src/registry/scene_table_tests.rs) — プロジェクト内 `#[path]` 使用実績
