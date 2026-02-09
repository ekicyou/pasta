# Research & Design Decisions

---
**Purpose**: DSL分離のための発見調査、アーキテクチャ検討、設計判断の記録

**Usage**:
- 発見フェーズでの調査活動と成果をログ
- design.md には収まらない詳細な設計判断のトレードオフを文書化
- 将来の監査や再利用のための参考資料と証拠を提供
---

## Summary
- **Feature**: `dsl-separation`
- **Discovery Scope**: Extension（既存システムのリファクタリング）
- **Key Findings**:
  - parser と registry は完全に独立しており、相互参照が一切ない（完全疎結合）
  - parser は tracing 依存を持たない（pest, pest_derive, thiserror のみ）
  - 下流クレート（pasta_lua）は pasta_dsl に直接依存する方針（Pattern B採用）
  - テスト移動は機械的な作業（26テスト、4ファイル）

## Research Log

### モジュール独立性の検証

- **Context**: parser と registry が実際に独立しているか、隠れた依存がないかを検証する必要があった
- **Sources Consulted**: 
  - `crates/pasta_core/src/parser/mod.rs` (1406行)
  - `crates/pasta_core/src/registry/mod.rs`
  - `crates/pasta_core/src/error.rs` (130行)
- **Findings**: 
  - parser → registry: 参照なし（完全独立）
  - registry → parser: 参照なし（完全独立）
  - parser → error: `use crate::error::ParseError` のみ（移動時に同一クレート内で完結）
  - registry → error: registry自体はエラー型を使用していない（error.rsにレジストリエラーの定義のみ存在）
- **Implications**: 分離時のリスクが極めて低い。parser と error の ParseError 部分を一緒に移動すれば、依存関係の調整は最小限で済む

### tracing 依存の確認

- **Context**: gap-analysis.md で「tracing依存を含めるか要確認」という設計判断が残されていた
- **Sources Consulted**: 
  - `crates/pasta_core/Cargo.toml`
  - `crates/pasta_core/src/parser/**` の grep 検索
- **Findings**: 
  - Cargo.toml には `tracing.workspace = true` が記載されている
  - parser モジュール内で `tracing` の使用は見つからなかった
  - registry や error.rs でも tracing の使用は確認されず
- **Implications**: pasta_dsl の依存は pest, pest_derive, thiserror のみで十分。tracing は pasta_core に残す

### 下流クレートの移行範囲

- **Context**: pasta_lua が pasta_core::parser をどの程度使用しているか、移行の影響範囲を評価する必要があった
- **Sources Consulted**: 
  - gap-analysis.md の調査結果（10箇所の use 文 + 多数のフルパス参照）
  - pasta_shiori の調査結果（パーサー参照なし）
- **Findings**: 
  - pasta_lua: 広範囲で pasta_core::parser を参照（移行必須）
  - pasta_shiori: pasta_core::parser への参照なし（移行不要）
- **Implications**: 移行対象は pasta_lua のみ。機械的な import パス変更で対応可能

### テスト移動の安全性

- **Context**: 26テストを pasta_dsl に移動する際、意図しない副作用がないか確認する必要があった
- **Sources Consulted**: 
  - `crates/pasta_core/tests/actor_code_block_test.rs` (3テスト)
  - `crates/pasta_core/tests/digit_id_var_test.rs` (4テスト)
  - `crates/pasta_core/tests/sakura_symbol_tag_test.rs` (7テスト)
  - `crates/pasta_core/tests/span_byte_offset_test.rs` (12テスト)
- **Findings**: 
  - すべてのテストが `pasta_core::parser::{...}` のみに依存
  - registry への直接参照は一切ない
  - import パスを `pasta_dsl::parser::{...}` に変更するだけで動作する
- **Implications**: テスト移動は安全かつ機械的な作業。リグレッションリスクは極めて低い

## Architecture Pattern Evaluation

### Pattern A: 完全分離（移行措置なし）

| 要素 | 説明                                                          | 強み                                                      | リスク / 制約                            | 備考                             |
| ---- | ------------------------------------------------------------- | --------------------------------------------------------- | ---------------------------------------- | -------------------------------- |
| 設計 | parser を pasta_dsl に移動し、pasta_core は再エクスポートなし | クリーンな責務分離、依存関係が明確                        | 下流クレートの import パス変更が必須     | **要件レビューで採用**           |
| 移行 | 下流クレートは pasta_dsl に直接依存するよう変更               | pasta_core::parser パスの完全消滅、アーキテクチャの明確化 | pasta_lua の変更範囲が広い（10箇所以上） | gap-analysis.md の推奨アプローチ |

### Pattern B: 互換性ラッパー（段階的移行）

| 要素 | 説明                                                | 強み                     | リスク / 制約                                       | 備考                     |
| ---- | --------------------------------------------------- | ------------------------ | --------------------------------------------------- | ------------------------ |
| 設計 | pasta_core に `pub use pasta_dsl::parser::*` を残す | 下流クレートの変更が不要 | pasta_core::parser パスが残り、アーキテクチャが曖昧 | **要件レビューで不採用** |
| 移行 | 段階的移行が可能                                    | 安全性が高い             | 一時的な複雑性の増加                                |                          |

**選択**: **Pattern A（完全分離）** を採用。

**理由**:
1. parser と registry の完全独立性が確認されており、分離自体にリスクがない
2. pasta_core から parser を完全除去することで、クリーンな責務分離を実現
3. 下流クレートが pasta_dsl に直接依存することで、依存関係が明確になる
4. pasta_core は将来的に registry 等のユーティリティに特化

## Design Decisions

### Decision: pasta_dsl の pub mod 構成

- **Context**: parser と error を別モジュールにするか、フラットにするか
- **Alternatives Considered**:
  1. **別モジュール構成** - `pub mod parser; pub mod error;` で明確に分離
  2. **フラット構成** - すべての型を lib.rs で直接公開
- **Selected Approach**: **別モジュール構成**を採用
- **Rationale**: 
  - 既存の pasta_core の構造に合わせて一貫性を保つ
  - parser と error の責務を明確に分離
  - 将来的な拡張性（新しいモジュールの追加）に対応しやすい
- **Trade-offs**: 
  - **利点**: モジュール境界が明確、責務分離が徹底される
  - **欠点**: わずかに import パスが長くなる（`pasta_dsl::parser::parse_str` vs `pasta_dsl::parse_str`）
- **Follow-up**: 実装時に lib.rs で適切な再エクスポートを行い、利便性を確保

### Decision: tracing 依存の除外

- **Context**: gap-analysis.md で「tracing 依存を含めるか要確認」という設計判断が残されていた
- **Alternatives Considered**:
  1. **tracing を含める** - Cargo.toml に記載されているため
  2. **tracing を除外** - parser で実際に使用されていない
- **Selected Approach**: **tracing を除外**
- **Rationale**: 
  - parser モジュール内で tracing の使用が見つからなかった
  - pasta_dsl は最小限の依存に留めるべき（独立利用性の確保）
  - tracing は pasta_core の registry 等で使用される可能性があるため、pasta_core に残す
- **Trade-offs**: 
  - **利点**: pasta_dsl の依存が最小限（pest, pest_derive, thiserror のみ）
  - **欠点**: 将来 parser にログを追加する場合、依存の追加が必要
- **Follow-up**: 実装時に依存一覧を確認し、pest, pest_derive, thiserror のみを含める

### Decision: 下流クレートの移行方針（Pattern B採用）

- **Context**: pasta_core の再エクスポートを残すか、下流クレートを pasta_dsl に直接依存させるか
- **Alternatives Considered**:
  1. **Pattern A（完全分離）** - pasta_core の再エクスポートを廃止、下流クレートは pasta_dsl に直接依存
  2. **Pattern B（互換性ラッパー）** - pasta_core に `pub use pasta_dsl::parser::*` を残す
- **Selected Approach**: **Pattern A（完全分離）**
- **Rationale**: 
  - parser と registry の完全独立性が確認されており、分離自体にリスクがない
  - pasta_core::parser パスを完全に消滅させることで、アーキテクチャが明確になる
  - 将来的な保守性・拡張性が向上する
- **Trade-offs**: 
  - **利点**: クリーンな責務分離、依存関係が明確、pasta_core::parser パスの完全消滅
  - **欠点**: pasta_lua の変更範囲が広い（10箇所以上の import パス変更）
- **Follow-up**: 実装時に pasta_lua のすべての pasta_core::parser 参照を pasta_dsl::parser に変更

## Risks & Mitigations

- **Risk 1: pasta_lua の import パス変更漏れ** — 機械的な grep/置換でカバー、`cargo test --all` で即座に検出可能
- **Risk 2: テスト移動時の import パス変更ミス** — 各テストファイルを個別に移動・変更し、`cargo test -p pasta_dsl` で逐次検証
- **Risk 3: error.rs 分割時の型の取りこぼし** — ParseError, ParseErrorInfo, ParseResult の3型のみを移動、他はすべて pasta_core に残す

## References
- [gap-analysis.md](./../gap-analysis.md) — 実装可能性分析と既存コード調査
- [requirements.md](./../requirements.md) — 6要件とEARS形式受入基準
- [Cargo Workspaceドキュメント](https://doc.rust-lang.org/cargo/reference/workspaces.html) — ワークスペース依存管理
- [Pest パーサードキュメント](https://pest.rs/) — PEG文法定義とビルド設定
