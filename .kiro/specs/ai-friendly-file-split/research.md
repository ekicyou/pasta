# Research & Design Decisions

## Summary
- **Feature**: `ai-friendly-file-split`
- **Discovery Scope**: Extension（既存コードベースの内部リファクタリング）
- **Key Findings**:
  - 10個のソースファイル（合計11,189行）に3,634行のインラインテストが存在
  - テストコードがファイル全体の11〜67%を占め、テスト外部化が最も効果的な第一手
  - 8ファイルは `tests/` へ外部化可能、2ファイルは `#[path]` パターンが必要
  - Rustの分割impl（同一型のimplを複数ファイルに分散）パターンが広く適用可能
  - テスト外部化後、6ファイルがPhase B（ソース分割）対象、3ファイルがガイドライン例外

## Research Log

### Rustにおけるモジュール分割パターン

- **Context**: 大きな `.rs` ファイルを複数ファイルに分割する際のRust言語の制約と慣用パターン
- **Sources Consulted**: Rust Reference, The Rust Programming Language Book
- **Findings**:
  - Rustでは同一型の `impl` ブロックを複数ファイルに分散可能（分割impl）
  - `mod.rs` + サブモジュール or フラットモジュール構成の2パターン
  - `pub use` による re-export で外部APIを維持可能
  - `#[cfg(test)] mod tests` を `#[path = "xxx/tests.rs"] mod tests;` で外部ファイル化可能
  - モジュールレベル関数は `pub(crate)` でサブモジュールに移動し、親モジュールから呼び出し可能
- **Implications**: 全分割対象ファイルが標準的なRustパターンで分割可能。API互換性を維持したまま構造を変更できる

### テスト分離パターンの有効性

- **Context**: 分析した10ファイルのうち全ファイルでテストが11%以上を占めている
- **Findings（実測値）**:
  - `shiori.rs`: 1,187行中テスト793行（67%）→ ソース394行
  - `word_table.rs`: 649行中テスト402行（62%）→ ソース247行
  - `scene_table.rs`: 1,053行中テスト604行（57%）→ ソース449行
  - `config.rs`: 850行中テスト423行（50%）→ ソース427行
  - `cache.rs`: 701行中テスト314行（45%）→ ソース387行
  - `runtime/mod.rs`: 1,174行中テスト343行（29%）→ ソース831行
  - `code_generator.rs`: 1,002行中テスト224行（22%）→ ソース778行
  - `parser/mod.rs`: 1,405行中テスト267行（19%）→ ソース1,138行
  - `ast.rs`: 885行中テスト126行（14%）→ ソース759行
  - `analysis.rs`: 1,283行中テスト138行（11%）→ ソース1,145行
- **Implications**: テスト分離だけで `word_table.rs` が300行以下に。他のファイルも大幅な行数削減が可能

### 実測値とギャップ分析時推定値の差異

- **Context**: ギャップ分析（gap-analysis.md）の推定値と設計時実測値に差異が発生
- **Findings**:
  | ファイル | gap推定(src) | 実測(src) | 差異 |
  |---|---:|---:|---:|
  | parser/mod.rs | ~1,034 | 1,138 | +104 |
  | analysis.rs | ~1,048 | 1,145 | +97 |
  | runtime/mod.rs | ~683 | 831 | +148 |
  | shiori.rs | ~207 | 394 | +187 |
  | scene_table.rs | ~319 | 449 | +130 |
  | code_generator.rs | ~668 | 778 | +110 |
  | ast.rs | ~674 | 759 | +85 |
  | config.rs | ~340 | 427 | +87 |
  | cache.rs | ~268 | 387 | +119 |
  | word_table.rs | ~155 | 247 | +92 |
- **原因**: gap分析時の行数は初回調査時点の値。その後の開発で各ファイルが成長
- **Impact**:
  - requirements.mdの「Phase B不要: 3ファイル」→ 実測では **1ファイル（word_table.rsのみ）** が300行以下
  - cache.rs (~387), shiori.rs (~394), config.rs (~427) は300行超だが、自然な分割境界がなくガイドライン例外
  - scene_table.rs (~449) は当初「微超」だったが実測では150行超過。型分離で軽減可能

### テスト外部化時の `#[cfg(test)]` メソッド問題

- **Context**: `config.rs` のL125に `#[cfg(test)]` 付きメソッド（テストモジュール外）が存在
- **Findings**:
  - `#[cfg(test)] fn from_str()` はテストモジュール外・impl内で定義
  - `tests/` への外部化時、統合テストはライブラリを `cfg(test)` なしでコンパイルする
  - そのため `#[cfg(test)]` 付きメソッドは統合テストから利用不可
- **Implications**: `from_str()` を `pub(crate)` に変更し `#[cfg(test)]` を除去する必要がある。設定のパーサーとして正当なAPIであり、テスト専用ではない

### 分割困難な構造の特定

- **Context**: すべてのファイルが均等に分割可能とは限らない
- **Findings**:
  - `analysis.rs` の `impl AnalysisEngine` は~900行の巨大implブロック。visitor関数群は相互参照が少なく分割可能
  - `code_generator.rs` の `impl LuaCodeGenerator` は単一ジェネリクス型（`<'a, W: Write>`）の分割implとなる
  - pest文法ファイル（`.pest`）は分割対象外
  - `parser/mod.rs` のパース関数はモジュールレベル関数のため、最も分割が容易
  - `scene_table.rs` は449行で300行を大きく超えるが、SceneTable単一structの凝集した実装。公開型分離で~60行削減可能
- **Implications**: すべての対象ファイルで実行可能な分割戦略が存在する

### 既存テストディレクトリの状態

- **Context**: Phase Aで `tests/` にテストを移動する際の前提確認
- **Findings**:
  | クレート | tests/ディレクトリ | 既存ファイル数 | common/ |
  |---|---|---|---|
  | pasta_dsl | ✅ 存在 | 5ファイル | なし |
  | pasta_core | ❌ **不存在** | — | — |
  | pasta_lua | ✅ 存在 | 27+ファイル | ✅ あり |
  | pasta_lsp | ✅ 存在 | 9ファイル | なし |
  | pasta_shiori | ✅ 存在 | 4ファイル | ✅ あり |
- **Implications**: `pasta_core` は `tests/` ディレクトリを新規作成する必要がある

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| テスト外部化 (tests/) | インラインテストを `tests/` に移動 | 最小リスク、Rust慣例に統一 | pub(crate)昇格が必要な場合あり | 8/10ファイルで適用 |
| テスト分離 (#[path]) | `#[path]` で別ファイルに分離 | privateアクセス維持 | 非標準パターン、src/にテスト混在 | 2/10ファイルで適用 |
| 責務別サブモジュール化 | 型定義・実装・ヘルパーをサブモジュールに分割 | 明確な責務分離、長期保守性向上 | re-exportの管理が必要 | parser, ast, analysis向け |
| 分割impl | 同一型のimplを複数ファイルに分散 | 大きなimplブロックの分割 | 実装の分散による認知負荷 | runtime, code_gen向け |

## Design Decisions

### Decision: テスト外部化を第一優先とする（3フェーズ実行）

- **Context**: 10ファイル中全ファイルでテストが11%以上。テスト外部化は最もリスクが低い
- **Alternatives Considered**:
  1. テスト外部化のみ行い、本体は触らない
  2. テスト外部化と本体分割を同時に行う
  3. `#[path]` パターンで全ファイルのテストを分離
- **Selected Approach**: Phase A（テスト外部化）→ Phase B（ソース分割）→ Phase C（テスト分割）の3段階
- **Rationale**: テスト外部化は機能への影響がゼロ。テスト外部化後の実測値でPhase B対象を判定。段階的にリスクを管理
- **Trade-offs**: 3段階のアプローチとなり工数は増えるが、各フェーズ完了時に安定状態を保証

### Decision: `tests/` 外部化を原則、`#[path]` を例外とする

- **Context**: テスト外部化の方式選択（全ファイル `#[path]` vs 原則 `tests/`）
- **Alternatives Considered**:
  1. 全ファイル `#[path]` パターン（v1設計）
  2. 全ファイル `tests/` 外部化
  3. privateアクセスの有無で分岐（ハイブリッド）
- **Selected Approach**: ハイブリッド — 8ファイルは `tests/`、2ファイルは `#[path]`
- **Rationale**: `tests/` は既存プロジェクト慣例と一致。`#[path]` はprivateフィールドアクセスが構造的に必要な場合のみ使用
- **Trade-offs**: 2パターン混在だが、例外は明確に記録され理由が正当

### Decision: ディレクトリモジュール化（mod.rs方式）を採用

- **Context**: 大きなファイルをサブモジュールとして分割する際、フラットファイル vs ディレクトリの選択
- **Alternatives Considered**:
  1. フラットモジュール（`code_gen_scope.rs`, `code_gen_element.rs` 等）
  2. ディレクトリモジュール（`code_gen/mod.rs`, `code_gen/scope.rs` 等）
- **Selected Approach**: ディレクトリモジュール（mod.rs方式）
- **Rationale**: steering（structure.md）の既存パターンに合致（parser/, registry/, runtime/, loader/ 等が既にディレクトリモジュール）。将来のさらなる分割にも対応可能
- **Trade-offs**: ディレクトリ構造が深くなるが、既存の慣例と一致

### Decision: 分割順序はレイヤー依存関係に従う

- **Context**: 複数クレートにまたがるリファクタリングの実行順序
- **Selected Approach**: pasta_dsl → pasta_core → pasta_lua → pasta_lsp → pasta_shiori の順
- **Rationale**: 下位レイヤーから分割することで、上位レイヤーのリファクタリング中に下位レイヤーの変更が発生しない。re-exportの安定性を保証

### Decision: Phase Bガイドライン例外の分類なし

- **Context**: 7件のガイドライン例外を「恒久例外」「将来レビュー候補」に分類するか否か
- **Selected Approach**: 分類なし。例外は例外のまま一律記録
- **Rationale**: 将来の判断は将来時点で行う。「将来検指」の注記は非拘束のインフォーマルヒントとして残す
- **Impact**: design.mdの例外テーブルは変更なし

### Decision: code_generator → code_gen リネーム（re-exportなし）

- **Context**: `code_generator.rs` をディレクトリモジュール化する際のモジュール名選択
- **Alternatives Considered**:
  1. `code_generator/` 名をそのまま維持（API変更ゼロ）
  2. `code_gen/` にリネーム + `pub use code_gen as code_generator;` で旧パス互換
  3. `code_gen/` にリネーム、re-exportなし
- **Selected Approach**: 案C — `code_gen/` リネーム、re-exportなし
- **Rationale**: 外部クレートからの `pasta_lua::code_generator::*` 直接参照がゼロ（実測済み）。変更箇所はlib.rs, transpiler.rsのクレート内部2ファイルのみ。不要なエイリアスを残すとかえって認知負荷になる
- **Trade-offs**: モジュール名変更は "pure refactoring" から少しはみ出すが、ディレクトリ化の機会を活かした命名改善として許容

## Risks & Mitigations

- **API互換性の破壊** — `pub use` re-exportの徹底検証、各分割後に `cargo test --workspace` を実行
- **テスト外部化時の `#[cfg(test)]` メソッド** — `config.rs` の `from_str()` など、テストモジュール外の `#[cfg(test)]` 付きメソッドは `pub(crate)` に変更し `#[cfg(test)]` を除去する必要がある
- **テスト分離時のヘルパー関数の可視性** — `pub(crate)` スコープを活用、テストモジュール内で必要な型を適切にインポート
- **統合テストからのprivateアクセス不可** — `tests/` ディレクトリのテストはライブラリを通常コンパイルするため、`pub(crate)` 未満の可視性はアクセス不可。必要な項目は `pub(crate)` に昇格
- **分割impl時のジェネリクス制約の不整合** — 型パラメータの完全一致を確認、コンパイルエラーで即座に検出可能
- **IDE支援の低下** — ファイル間の型推論はrust-analyzerが対応しており問題なし
- **pasta_core/tests/ 新規作成** — ディレクトリが存在しないため、最初のテスト移動時に作成が必要

## References

- [The Rust Programming Language - Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [Rust Reference - Items](https://doc.rust-lang.org/reference/items.html)
- steering/structure.md — 既存のモジュール構造パターン
- gap-analysis.md — インラインテスト実態調査（初回推定値）
