# Research & Design Decisions

## Summary
- **Feature**: `cache-shiori-request-function`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - mlua::Function は Clone/Send/Sync 実装済みの通常の struct でありキャッシュ可能
  - 既存の boolean フラグを Option<Function> に置き換えることでコード重複を削減
  - Drop impl での unload 呼び出しは runtime drop 前に実行が必要

## Research Log

### mlua::Function のキャッシュ可能性
- **Context**: Function を struct フィールドに保持できるか確認
- **Sources Consulted**: 
  - https://docs.rs/mlua/latest/src/mlua/function.rs.html
  - mlua crate ソースコード
- **Findings**:
  - `#[derive(Clone, Debug, PartialEq)]` 実装済み
  - `pub struct Function(pub(crate) ValueRef)` - 通常の struct
  - `ValueRef` は内部で参照カウント管理
  - `send` feature 有効時に Send/Sync 実装
- **Implications**: 
  - PastaShiori の struct フィールドとして安全に保持可能
  - 過去の「参照型なのでキャッシュ不可」という意見は誤り

### 既存実装の関数取得パターン
- **Context**: 現在の request() と call_shiori_load() の関数取得処理を分析
- **Sources Consulted**: crates/pasta_shiori/src/shiori.rs
- **Findings**:
  - request(): 毎回 `globals.get("SHIORI")` → `shiori_table.get("request")` を実行
  - call_shiori_load(): 同様に毎回関数取得
  - check_shiori_functions(): 関数存在確認のみで参照は破棄
- **Implications**:
  - 関数キャッシュにより2段階のハッシュテーブルルックアップを削減
  - パフォーマンス向上が期待される

### Drop impl での unload 呼び出し順序
- **Context**: SHIORI.unload を Drop で呼び出す際の制約
- **Sources Consulted**: Rust Drop semantics, SHIORI protocol specification
- **Findings**:
  - Lua 関数呼び出しは runtime が有効な間のみ可能
  - Drop でエラーを伝播させるべきではない（Rust 規約）
  - SHIORI 仕様では unload は「クリーンアップ機会」であり失敗は許容
- **Implications**:
  - unload → runtime drop の順序を厳守
  - エラー発生時はログ記録のみで続行

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: 既存拡張 | PastaShiori struct に Option<Function> フィールド追加 | 最小変更、既存パターン維持 | なし | **推奨** |
| Option B: 新規コンポーネント | ShioriFunctionCache 別構造体 | 責務分離 | 過剰設計 | 却下 |
| Option C: ハイブリッド | 段階的移行 | 互換性維持 | 不要な複雑性 | 却下 |

## Design Decisions

### Decision: boolean フラグを Option<Function> に置換
- **Context**: has_shiori_load / has_shiori_request フラグの冗長性
- **Alternatives Considered**:
  1. フラグ維持 + 別途 Function キャッシュ追加
  2. フラグを Option<Function> に統合
- **Selected Approach**: Option 2 - フラグを完全に置換
- **Rationale**: 
  - DRY 原則に従い冗長データを削減
  - `.is_some()` で関数存在確認可能
  - フィールド数は増加するが型の意味が明確
- **Trade-offs**: 
  - メリット: コード簡素化、単一ソース・オブ・トゥルース
  - デメリット: なし（メモリオーバーヘッドは無視可能）
- **Follow-up**: テスト内の has_shiori_* アサーションを修正

### Decision: check_shiori_functions() のリネームとリファクタリング
- **Context**: メソッド名が責務を正確に反映していない
- **Alternatives Considered**:
  1. 名前維持、内部実装のみ変更
  2. `cache_shiori_functions()` にリネーム
- **Selected Approach**: Option 2 - `cache_shiori_functions()` にリネーム
- **Rationale**: 
  - 「チェック」から「キャッシュ」への責務変更を名前に反映
  - コードの自己文書化を向上
- **Trade-offs**: 
  - メリット: 意図が明確
  - デメリット: 軽微な breaking change（private メソッド）

### Decision: Drop での unload エラーハンドリング
- **Context**: unload 関数実行失敗時の対応
- **Alternatives Considered**:
  1. panic! で異常終了
  2. Result を返す（不可能 - Drop 制約）
  3. ログ記録のみで続行
- **Selected Approach**: Option 3 - ログ記録のみ
- **Rationale**: 
  - Rust の Drop は失敗を伝播できない
  - SHIORI 仕様では unload 失敗は致命的でない
  - クリーンアップの最善努力アプローチ
- **Trade-offs**: 
  - メリット: 堅牢性、パニックなし
  - デメリット: 一部エラーが静かに処理される

## Risks & Mitigations
- **Risk 1: 複数回 load() でのキャッシュクリア忘れ**
  - Mitigation: reload 時に明示的に全キャッシュを None に設定してから新規取得
- **Risk 2: unload 呼び出し後の runtime 破棄順序**
  - Mitigation: Drop impl 内で unload → self.runtime = None の順序を厳守
- **Risk 3: テスト互換性**
  - Mitigation: 既存テストのアサーション修正（2テスト）、新規テスト追加

## References
- [mlua Function source](https://docs.rs/mlua/latest/src/mlua/function.rs.html) - Function 構造体の定義
- [SHIORI Protocol](http://usada.sakura.vg/contents/shiori.html) - SHIORI 仕様
- [Rust Drop semantics](https://doc.rust-lang.org/std/ops/trait.Drop.html) - Drop の制約
