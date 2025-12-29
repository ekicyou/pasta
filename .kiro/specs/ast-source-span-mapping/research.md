# 調査と設計決定

## 概要
- **機能**: `ast-source-span-mapping`
- **調査範囲**: Extension（既存システム拡張）
- **主要な発見**:
  - Pest 2.8 は `Span::start()`, `Span::end()` でバイトオフセット取得可能（検証済み）
  - 既存 Span 構造体は 4 フィールド（line/col のみ）、バイト情報なし
  - 41 個の既存 Span 呼び出し箇所（実装 29 + テスト 12）

## 調査ログ

### Pest 2.8 バイトオフセット API
- **背景**: 要件実現のため、Pest が提供するバイト位置情報の存在確認が必要
- **参照ソース**: Pest 公式ドキュメント、コンパイル検証
- **発見内容**:
  - `pair.as_span().start()` → 入力先頭からの開始バイトオフセット（`usize`）
  - `pair.as_span().end()` → 入力先頭からの終了バイトオフセット（`usize`）
  - UTF-8 安全（Rust の `&str` 入力を前提）
- **影響**: `From<&Pair<Rule>> for Span` トレイト実装で直接統合、追加計算不要

### 既存 Span 実装調査
- **背景**: 拡張対象の現行実装を把握
- **参照ソース**: `src/parser/ast.rs:55-90`
- **発見内容**:
  ```rust
  pub struct Span {
      pub start_line: usize,   // 1-based
      pub start_col: usize,    // 1-based
      pub end_line: usize,     // 1-based
      pub end_col: usize,      // 1-based
  }
  ```
  - コンストラクタ: `Span::new()`, `Span::from_pest()`, `Default`
  - サイズ: 32 bytes（4 × 8 bytes）
- **影響**: 2 フィールド追加で 48 bytes（許容範囲）

### 既存呼び出し箇所調査
- **背景**: 破壊的変更の影響範囲把握
- **参照ソース**: grep 検索結果
- **発見内容**:
  | 呼び出しパターン | 実装コード | テストコード | 合計 |
  |-----------------|-----------|-------------|------|
  | `Span::new()` | 6 | 6 | 12 |
  | `Span::default()` | 23 | 6 | 29 |
  | 合計 | 29 | 12 | 41 |
- **影響**: `Span::new()` シグネチャ変更で 12 箇所修正必須

### Action 型への Span 追加検討
- **背景**: 要件 3.2 でアクション単位の Span 追跡が必要
- **参照ソース**: `src/parser/ast.rs` Action enum 定義
- **発見内容**:
  - `Action` は enum（Say, Eval, Raw 等）
  - 現在 Span フィールドなし
  - `ActionWithSpan` ラッパー or 各バリアントに追加の選択
- **影響**: `ActionWithSpan` ラッパーパターン採用（シンプル）

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク/制限 | 備考 |
|-----------|------|------|-------------|------|
| A: 既存 Span 拡張 | Span に `start_byte`, `end_byte` 追加 | 単純明快、統一的 | 破壊的変更、テスト修正必要 | **採用** |
| B: 新規 SourceSpan 型 | 別型で byte 情報管理 | 後方互換 | 2 型共存の混乱 | 非採用 |
| C: 外部マッピング | HashMap で位置変換 | 完全互換 | 複雑、パフォーマンス | 非採用 |

## 設計決定

### 決定: From トレイトを使った Pest 統合

- **背景**: Span の生成方法をクリーンに設計
- **検討した代替案**:
  1. `Span::from_pest(start, end, start_byte, end_byte)` メソッド（6 引数版）
  2. `From<&Pair<Rule>> for Span` トレイト実装
- **選択したアプローチ**: From トレイト
- **根拠**:
  - Rust のイディオマティック設計（標準ライブラリ慣例）
  - パーサー層での `Span::from(&pair)` 呼び出しが簡潔・読みやすい
  - `from_pest()` メソッドの二重シグネチャ問題を回避
  - 既存 `span_from_pair()` 関数を From トレイトで置換可能
- **トレードオフ**: なし（API がシンプル化）
- **フォローアップ**: 既存の `span_from_pair()` 関数は削除し、`Span::from(&pair)` で統一

### 決定: ActionWithSpan ラッパーパターン
- **背景**: Action enum への Span 追加方法
- **検討した代替案**:
  1. 各 Action バリアントに Span フィールド追加
  2. ActionWithSpan ラッパー構造体
- **選択したアプローチ**: ActionWithSpan ラッパー
- **根拠**:
  - Action enum の変更を最小化
  - トランスパイラでの扱いが統一的
  - 将来の Action 拡張に影響しない
- **トレードオフ**: 間接参照が 1 段増える（パフォーマンス影響なし）

### 決定: Span::new() 破壊的拡張
- **背景**: コンストラクタ変更方針
- **検討した代替案**:
  1. `Span::new()` を 6 引数に変更
  2. `Span::with_bytes()` 新規追加、旧 API 温存
- **選択したアプローチ**: 6 引数への変更
- **根拠**:
  - API 一貫性（単一のコンストラクタ）
  - 中途半端な互換レイヤー回避
  - テスト箇所は `0, 0` で補完可能
- **トレードオフ**: 12 箇所の `Span::new()` 呼び出し修正

## リスクと緩和策
- **リスク 1**: 既存テスト 41 箇所の修正漏れ → コンパイルエラーで検出可能
- **リスク 2**: UTF-8 マルチバイト計算誤り → Pest が自動処理、追加テストで検証
- **リスク 3**: struct サイズ増加によるパフォーマンス影響 → 16 bytes 増加のみ、許容範囲

## 参考資料
- [Pest 2.8 Span API](https://docs.rs/pest/2.8/pest/struct.Span.html) — `start()`, `end()` メソッド仕様
- gap-analysis.md — 詳細ギャップ分析
- tech.md steering — Rust 2024 edition、Pest 2.8 技術スタック
