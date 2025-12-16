# Research & Design Decisions: pasta-engine-independence

## Summary

- **Feature**: `pasta-engine-independence`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. Rune VMのAPI制約により`Arc<Unit>`/`Arc<RuntimeContext>`はVm::new()に必須
  2. グローバルキャッシュ(`static PARSE_CACHE`)のみが共有状態、インスタンス化で解決可能
  3. パーサー・トランスパイラーは既に純粋関数として実装済み

---

## Research Log

### Topic 1: Rune API制約の調査

- **Context**: `Arc`の使用が要件違反かどうかの判断が必要
- **Sources Consulted**:
  - [Rune Multithreading Documentation](https://rune-rs.github.io/book/multithreading.html)
  - [Rune API - Vm::new()](https://docs.rs/rune/latest/rune/runtime/struct.Vm.html#method.new)
- **Findings**:
  - `Vm::new()`のシグネチャは`pub const fn new(context: Arc<RuntimeContext>, unit: Arc<Unit>) -> Self`
  - API設計の意図: Unitとコンテキストのコンパイルは高コスト、VM作成は低コスト
  - 複数VMで同じUnit/Contextを共有する設計前提
  - **Arc排除は技術的に不可能**（API制約）
- **Implications**:
  - ユーザー確認済み: 外部API制約によるArc使用は許容
  - 問題はエンジン**間**での共有であり、Arc自体ではない

### Topic 2: 型特性の分析

- **Context**: PastaEngineのSend/Sync実装可能性の確認
- **Sources Consulted**:
  - Rune 0.14 rustdoc
- **Findings**:
  | 型 | TryClone | Send | Sync | 備考 |
  |----|----------|------|------|------|
  | `Unit<S>` | ✅ | 条件付き | 条件付き | S依存 |
  | `RuntimeContext` | ✅ | ✅ | ✅ | 無条件 |
  | `Vm` | ✅ | ❌ | ❌ | スレッド移動不可 |
  | `Arc<Unit>` | - | ✅ | ✅ | Arc保証 |
  | `Arc<RuntimeContext>` | - | ✅ | ✅ | Arc保証 |
- **Implications**:
  - PastaEngineは`Arc<Unit>`と`Arc<RuntimeContext>`を保持する限りSend可能
  - VM実行はスレッドローカル（メソッド内でVm::new()）

### Topic 3: グローバルキャッシュの分析

- **Context**: 現在の共有状態の特定と影響範囲
- **Sources Consulted**:
  - `crates/pasta/src/engine.rs` L17-27
  - `crates/pasta/src/cache.rs`
- **Findings**:
  ```rust
  // 現状: グローバル共有
  static PARSE_CACHE: OnceLock<ParseCache> = OnceLock::new();
  ```
  - ParseCacheは内部で`Arc<RwLock<HashMap>>`を使用
  - 全エンジンインスタンスが同一キャッシュを参照
  - キャッシュミス時のみパース・トランスパイル実行
- **Implications**:
  - **解決策**: `cache: ParseCache`をPastaEngineのフィールドに移動
  - パフォーマンス: 各インスタンスが独立キャッシュを持つ（同一スクリプト再利用時に有効）
  - メモリ: インスタンスごとにキャッシュ領域が必要（許容範囲）

### Topic 4: 既存パターンの確認

- **Context**: 純粋関数的実装の確認
- **Sources Consulted**:
  - `crates/pasta/src/parser/mod.rs`
  - `crates/pasta/src/transpiler/mod.rs`
- **Findings**:
  - `parse_str()`: 純粋関数（入力文字列→AST）
  - `Transpiler::transpile()`: 純粋関数（AST→Rune Source）
  - 両関数ともグローバル状態への依存なし
- **Implications**:
  - 要件2.4は**既に適合**
  - 変更不要

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: グローバルキャッシュ維持 | 現状維持 | 実装変更なし | 要件違反（共有状態） | 却下 |
| B: キャッシュ削除 | ParseCache完全削除 | シンプル | 毎回パース・コンパイル（性能低下） | 却下 |
| **C: インスタンスキャッシュ** | PastaEngineのフィールド化 | 独立性保証、性能維持 | インスタンス間キャッシュ共有不可 | **採用** |
| D: 外部キャッシュ注入 | DIパターン | 柔軟性高い | API複雑化、過剰設計 | 却下 |

---

## Design Decisions

### Decision 1: ParseCacheのインスタンス化

- **Context**: 要件1.5「static変数ゼロ」と要件2「キャッシュ独立性」の実現
- **Alternatives Considered**:
  1. グローバルキャッシュ削除（性能低下）
  2. 外部キャッシュDI注入（API複雑化）
- **Selected Approach**: PastaEngine構造体のフィールドとしてParseCache保持
  ```rust
  pub struct PastaEngine {
      unit: Arc<rune::Unit>,
      runtime: Arc<rune::runtime::RuntimeContext>,
      label_table: LabelTable,
      cache: ParseCache,  // NEW: インスタンス所有
  }
  ```
- **Rationale**: 
  - 構造変更のみでロジック変更不要
  - キャッシュ機能維持（性能影響なし）
  - 所有権による自動解放
- **Trade-offs**: 
  - 同一スクリプトの複数エンジン間でキャッシュ共有不可
  - メモリ使用量は微増（各インスタンスがキャッシュ保持）
- **Follow-up**: テストでインスタンス間独立性を検証

### Decision 2: Arc使用の許容（API制約）

- **Context**: Rune Vm::new()がArc<Unit>/Arc<RuntimeContext>を要求
- **Alternatives Considered**:
  1. Runeフォーク（非現実的）
  2. 別VMライブラリ（移行コスト大）
- **Selected Approach**: Rune API要求のArc使用を許容
- **Rationale**: 
  - ユーザー確認済み: 外部API制約は許容
  - 問題はエンジン間共有であり、Arc自体ではない
  - 各PastaEngineは独自のArc<Unit>とArc<RuntimeContext>を所有
- **Trade-offs**: 
  - 厳密には「共有ポインタなし」に反するが、実質的に各エンジンが独立所有
- **Follow-up**: ドキュメントでAPI制約を明記

### Decision 3: ParseCache内部実装の簡素化

- **Context**: グローバル使用前提の`Arc<RwLock<HashMap>>`が不要に
- **Alternatives Considered**:
  1. 現状維持（Arc/RwLock）
  2. 簡素化（HashMap直接所有）
- **Selected Approach**: 単純な`HashMap<u64, CacheEntry>`に変更
  ```rust
  pub struct ParseCache {
      entries: HashMap<u64, CacheEntry>,  // Arc/RwLock不要
  }
  ```
- **Rationale**: 
  - シングルスレッドアクセス（PastaEngine内）
  - RwLockのオーバーヘッド排除
  - コード簡素化
- **Trade-offs**: 
  - スレッド間共有不可（要件通り）
- **Follow-up**: ベンチマークで性能改善確認（任意）

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| キャッシュ非共有による性能低下 | Low | Low | 同一エンジンでの再実行時にキャッシュ有効 |
| メモリ使用量増加 | Low | Medium | スクリプトサイズに依存、通常は許容範囲 |
| 後方互換性破壊 | Medium | Low | 公開API変更なし、内部構造のみ変更 |
| テスト不足によるリグレッション | High | Medium | 包括的テストスイート整備（Req 4-7） |

---

## References

- [Rune Multithreading Guide](https://rune-rs.github.io/book/multithreading.html) — Arc使用の設計根拠
- [Rune 0.14 API Documentation](https://docs.rs/rune/latest/rune/) — Vm, Unit, RuntimeContext型情報
- [Rust std::sync::OnceLock](https://doc.rust-lang.org/std/sync/struct.OnceLock.html) — 現在使用中のグローバル初期化パターン
