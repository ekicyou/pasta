# Research & Design Decisions

## Summary
- **Feature**: `audit-dependency-supply-chain`
- **Discovery Scope**: Extension（既存ワークスペース依存の横断的監査）
- **Key Findings**:
  - 全7クレートで27の外部直接依存（重複排除後約20種）を使用。workspace.dependenciesで大部分を管理済み
  - pasta_checkの`md5`、`zip`、`lexopt`はworkspace管理外。pasta_lspの`tower-lsp`、WASM依存群も同様
  - Wave 1全specで外部依存サプライチェーン監査は本specに委任済み

## Research Log

### 依存クレート全量調査
- **Context**: ワークスペース全体の依存構造把握
- **Sources Consulted**: 全7クレートのCargo.toml、ルートCargo.toml（workspace.dependencies）
- **Findings**:
  - **pasta_core** (4依存): thiserror, fast_radix_trie, rand, tracing
  - **pasta_dsl** (3依存): pest, pest_derive, thiserror
  - **pasta_lua** (15依存): pasta_core, pasta_dsl, tracing, tracing-appender, tracing-subscriber, thiserror, mlua, mlua-stdlib, toml, serde, serde_json, glob, flate2, regex, budoux, unicode-width + windows-sys (cfg(windows))
  - **pasta_shiori** (7依存): pasta_core, pasta_lua, time, tracing, thiserror, pest, pest_derive + windows-sys (cfg(windows))
  - **pasta_check** (4依存): lexopt, md5, zip, thiserror + pasta_lua
  - **pasta_lsp** (5+4依存): pasta_dsl, thiserror, serde, serde_json, tower-lsp + wasm-bindgen, wasm-bindgen-futures, js-sys, serde-wasm-bindgen (wasm32)
  - **pasta_sample_ghost** (3依存): image, imageproc, thiserror
  - **dev-dependencies** (共通): tempfile, insta, tracing-test + tokio (pasta_lspのみ)
- **Implications**: 外部依存は約20種で中規模。workspace.dependenciesで共通管理されていないものの統合が改善候補

### Workspace管理状況の調査
- **Context**: バージョン固定戦略の網羅性確認
- **Findings**:
  - workspace.dependenciesで管理済み: pest, pest_derive, thiserror, fast_radix_trie, rand, tracing, mlua, mlua-stdlib, regex, toml, serde, serde_json, glob, flate2, budoux, unicode-width, time, windows-sys, tracing-subscriber, tracing-appender, tempfile, insta, tracing-test
  - workspace管理外（個別クレートで直接指定）: lexopt 0.3, md5 0.8, zip 8.6, tower-lsp 0.20, image 0.25, imageproc 0.26, wasm-bindgen 0.2, wasm-bindgen-futures 0.4, js-sys 0.3, serde-wasm-bindgen 0.6, tokio 1
- **Implications**: 10個の依存がworkspace管理外。利用箇所が1クレートのみのものはworkspace統合の必要性は低いが、一貫性のため統合検討の価値あり

### Wave 1監査からの依存関連知見
- **Context**: 各クレート監査で発見された依存関連の指摘
- **Findings**:
  - **audit-pasta-check**: MD5は非暗号学的ファイル変更検出用途（SSP仕様準拠）。用途コメント追記済み
  - **audit-pasta-lua**: mlua vendoredビルド。unsafeはmlua API制約に起因。外部依存バージョン変更は本specに委任
  - **audit-pasta-shiori**: 外部依存クレートのサプライチェーン監査は本specに委任
  - **audit-pasta-lsp**: 外部依存クレートのサプライチェーン監査は本specに委任
  - **audit-pasta-core**: 新規外部依存の追加は禁止
  - **audit-pasta-dsl**: 新しい外部依存の追加は不可
  - **audit-pasta-sample-ghost**: image/imageproc依存クレートの更新は本specの範囲外だが監査は対象
- **Implications**: Wave 1全specが外部依存のサプライチェーン監査を本specに委任している。MD5用途適切性は既に文書化済みだが、本specで統合的に再確認する

### 監査ツール調査
- **Context**: Rustエコシステムにおける依存監査ツール
- **Findings**:
  - **cargo-audit**: RustSec Advisory DBに基づく脆弱性検査。`cargo install cargo-audit`で導入
  - **cargo-deny**: ライセンス監査、ban対象クレートの管理、重複依存検出。deny.tomlで設定
  - **cargo-tree**: 依存ツリーの可視化。重複依存やfeature分析に有用。Rust標準ツール
  - **cargo-udeps**: 未使用依存の検出。nightlyが必要な場合あり
  - **cargo-outdated**: 更新可能な依存の一覧表示
- **Implications**: cargo-audit + cargo-deny + cargo-tree で主要な監査はカバー可能。cargo-udepsはnightly制約あり、手動分析で代替可能

### プロジェクトライセンス互換性
- **Context**: MIT OR Apache-2.0ライセンスとの互換性基準
- **Findings**:
  - MIT: 非常に寛容、ほぼ全てのライセンスと互換
  - Apache-2.0: GPLv2との非互換以外は寛容
  - 警戒対象: GPL, LGPL（静的リンク時）, AGPL, SSPL, BUSL
  - Rust依存の大多数はMIT/Apache-2.0/BSD系
  - mlua vendored（LuaJIT）: MIT License — 互換
- **Implications**: GPL汚染の可能性は低いが、体系的に確認が必要

## Design Decisions

### Decision: 監査レポートの形式
- **Context**: 監査結果をどの形式で文書化するか
- **Alternatives Considered**:
  1. Markdown形式のレポートファイル（audit-report.md）
  2. cargo-deny設定ファイル（deny.toml）+ CI統合
  3. 既存のresearch.mdへの追記
- **Selected Approach**: deny.toml設定ファイル + Markdownレポート（research.mdに統合）
- **Rationale**: deny.tomlは再現可能な機械的チェックを提供し、将来のCI統合の基盤となる。レポートはresearch.mdに監査結果として記録
- **Trade-offs**: deny.tomlの初期設定コストはあるが、将来の反復監査で回収できる

### Decision: Workspace依存の統合方針
- **Context**: workspace管理外の依存をどこまで統合するか
- **Alternatives Considered**:
  1. 全依存をworkspace.dependenciesに統合
  2. 複数クレートで使用されるもののみ統合
  3. 現状維持
- **Selected Approach**: 1クレートのみで使用される依存もworkspace.dependenciesに統合（一貫性優先）
- **Rationale**: バージョン管理の一元化により、将来のアップデート作業を効率化。ワークスペースが小規模なため管理負荷は低い
- **Trade-offs**: ルートCargo.tomlの行数増加だが、管理の一貫性が上回る

## Risks & Mitigations
- cargo-auditで既知脆弱性が検出された場合 → 深刻度に応じてパッチバージョンアップまたはワークアラウンド
- ライセンス非互換が発見された場合 → 代替クレートへの移行検討（本spec範囲で対応可能な範囲）
- 依存除去でビルド破壊 → 全テスト回帰確認を必須とする
- cargo-udepsがnightlyを要求 → cargo-treeの手動分析で代替

## References
- [RustSec Advisory Database](https://rustsec.org/) — Rust脆弱性データベース
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit) — 脆弱性スキャナ
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) — ライセンス・ban・重複チェック
- [SPDX License List](https://spdx.org/licenses/) — ライセンス識別子一覧
