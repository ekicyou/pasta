# Research & Design Decisions

---
**Purpose**: 宣言的コントロールフロー機能の発見調査結果を記録し、技術設計の根拠を提供する。
---

## Summary
- **Feature**: `pasta-declarative-control-flow`
- **Discovery Scope**: Extension（既存システムの修正・拡張）
- **Key Findings**:
  - 現在のトランスパイラーは要件5と4つの主要な乖離点を持つ（モジュール化なし、`__start__`なし、ローカルラベルのフラット化、直接関数呼び出し）
  - Runeジェネレーター内でMut参照をyield跨ぎで保持できない制約があり、責任分離による瞬間ロック方式（案C）が最適解
  - 2パストランスパイラー方式でWord/Call/Jumpの統一的解決が可能

## Research Log

### Runeモジュール構文の検証
- **Context**: 要件5でグローバルラベルをRuneモジュールに変換する必要がある
- **Sources Consulted**: 
  - rune-rs/runeリポジトリ（`src/tests/vm_test_mod.rs`, `vm_test_imports.rs`）
  - https://rune-rs.github.io/ ドキュメント
- **Findings**:
  - `pub mod モジュール名 { pub fn 関数名(引数) { ... } }` 形式でネスト可能
  - 可視性制御: `pub`, `pub(super)`, `pub(crate)`, `pub(in path)`をサポート
  - モジュール内関数もジェネレーター関数として定義可能
- **Implications**: 要件で規定したモジュール構造の生成に問題なし

### Rune Object型操作と致命的制約
- **Context**: `ctx.pasta`オブジェクトの構築とランタイムメソッド実装
- **Sources Consulted**: 
  - rune-rs/rune `runtime/vm.rs`, `generator.rs`
  - 既存コード `crates/pasta/src/stdlib/`
- **Findings**:
  - `#[derive(Any)]` + `module.ty::<T>()?`で外部構造体公開可能
  - **🚨 致命的制約**: Runeジェネレーター内でMut参照をyield跨ぎで保持できない
  - ジェネレーターはyield時に状態をサスペンドし、Mut参照のライフタイムは跨げない
- **Implications**: 
  - 案C「責任分離による瞬間ロック方式」が最適解
  - PastaEngineはラベル名→Rune関数パス解決のみを担当
  - Rune側で直接関数呼び出し（lockなし）

### Rune VMスタック特性
- **Context**: ネストされたCall/Jumpでのスタックオーバーフローリスク評価
- **Sources Consulted**: rune-rs/rune `runtime/memory.rs`, `runtime/vm.rs`
- **Findings**:
  - Rune VMはCスタックではなくヒープ上のVMスタックを使用
  - `Stack { stack: Vec<Value>, top: usize }` でヒープ確保
  - 深いネストでもスタックオーバーフローなし
- **Implications**: Runeレベルでのpastaオブジェクト実装が最適解

### while-let-yieldパターンの検証
- **Context**: ネストされたジェネレーターからのイベント伝播
- **Sources Consulted**: 
  - 実験コード `crates/pasta/examples/test_while_let_yield.rs`
  - Runeドキュメント "Generators"セクション
- **Findings**:
  - `for value in generator() { yield value; }` パターンで完全動作
  - 3層ネスト（outer → middle → inner）で全イベント正しく伝播
  - **必須**: `context.runtime()`を使って`Vm::new(runtime, unit)`で初期化
- **Implications**: 案Cの実装パターンは完全に動作可能

### ワード解決の2パス方式
- **Context**: `＠単語`がRune関数か文字列辞書か区別不可能問題
- **Sources Consulted**: 
  - `unit.debug_info()` API
  - 検証コード `crates/pasta/tests/test_rune_metadata.rs`
- **Findings**:
  - Rune Unitのメタデータから完全修飾関数名を取得可能
  - `unit.debug_info().functions.iter()` で関数一覧取得
  - 2パス解決: 予約関数生成 → メタデータ取得 → 本体生成
- **Implications**: Word/Call/Jumpすべてを予約関数パターンに統一可能

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 既存拡張 | トランスパイラー本体を直接修正 | ファイル構造変更少 | ほぼ全面書き換え必要、責務増大 | 不採用 |
| B: 新規作成 | 新規モジュール追加で責務分離 | クリーンな設計、テスト分離可能 | ファイル数増加 | **採用** |
| C: ハイブリッド | 段階的移行 | リスク分散 | 計画複雑、Phase 1でBreaking | 不採用 |

## Design Decisions

### Decision: 責任分離による瞬間ロック方式（案C）
- **Context**: Runeジェネレーター内でMut参照をyield跨ぎで保持できない制約
- **Alternatives Considered**:
  1. Arc<Mutex<PastaEngine>>で全体ロック → ネストCall時にデッドロック
  2. トランポリン方式 → 実装複雑度が高い
- **Selected Approach**: PastaEngineはラベル解決のみ担当、Rune側で直接関数呼び出し
- **Rationale**: ロック時間極小、ネスト呼び出し対応、Rune VMスタック活用
- **Trade-offs**: 瞬間ロックのオーバーヘッド（極小）vs 完全なネスト対応
- **Follow-up**: 実装時にロック競合テストを実施

### Decision: 2パストランスパイラー
- **Context**: `＠単語`がRune関数か文字列辞書か静的に区別不可能
- **Alternatives Considered**:
  1. 動的ディスパッチ → Rune VMはevalなし
  2. DSL構文拡張 → 既存コード破壊的変更
- **Selected Approach**: 予約関数 + unit.debug_info()による2パス解決
- **Rationale**: 静的解決で型安全、既存DSL互換維持
- **Trade-offs**: 2回のコンパイルオーバーヘッド vs 完全な型安全
- **Follow-up**: パフォーマンス計測（初期化時のみなので許容範囲の見込み）

### Decision: 予約関数パターンの統一
- **Context**: Word/Call/Jumpすべてが同じスコープ解決問題を持つ
- **Alternatives Considered**:
  1. 各構文で別ロジック → コード重複
- **Selected Approach**: `__word_*__`, `__call_*__`, `__jump_*__` 予約関数に統一
- **Rationale**: 一貫したアーキテクチャ、デバッグ性向上
- **Trade-offs**: 予約パターン禁止のユーザー制約 vs 統一設計
- **Follow-up**: Pest文法に予約パターンチェック追加

## Risks & Mitigations
- **Rune VM API理解不足** → Runeドキュメント、既存stdlib実装参照、段階的テスト
- **yield伝播メカニズムの検証** → 単純テストケースで動作確認済み
- **既存テスト全面書き直し** → 新形式に合わせて段階的修正

## References
- [Rune公式ドキュメント](https://rune-rs.github.io/) - モジュール、ジェネレーター、型システム
- [rune-rs/runeリポジトリ](https://github.com/rune-rs/rune) - 実装詳細
- `crates/pasta/src/transpiler/mod.rs` - 現在のトランスパイラー実装
- `crates/pasta/src/runtime/labels.rs` - LabelTable実装
- `crates/pasta/tests/` - 既存テストコード
