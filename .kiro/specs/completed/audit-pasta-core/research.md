# 調査・設計判断ログ

## 概要
- **機能**: `audit-pasta-core`
- **調査スコープ**: Extension（既存クレートの内部改善）
- **主要な発見**:
  - `scene_table.rs` の `resolve_scene_id` / `resolve_scene_id_unified` に重複するキャッシュ処理ロジックが存在
  - キャッシュリセット後の `cache.get_mut(&cache_key).unwrap()` がパニックリスク（理論上は安全だが防御的コーディングに反する）
  - エラーメッセージの言語が混在（`WordTableError::WordNotFound` は日本語、`SceneTableError` は英語）

## 調査ログ

### パニック安全性の調査
- **コンテキスト**: `scene_table.rs` 内の `unwrap()` 使用箇所の安全性確認
- **情報源**: ソースコード直接調査
- **発見**:
  - `fn_name.split("::").next().unwrap_or(fn_name)` (L146): `split()` は常に少なくとも1要素を返すため安全。ただし `unwrap_or` で既に防御済み
  - `cache.get_mut(&cache_key).unwrap()` (L225, L232, L301, L308): キャッシュ挿入直後のため理論上安全だが、借用スプリットパターンの都合で直接呼び出し。防御的コーディングの観点から改善推奨
  - `candidates[next_index]` (L237, L313): `next_index < candidates.len()` のチェック後だが、インデックスアクセスにパニックリスクあり
- **影響**: パニック安全性を完全に保証するには、これらのパターンを `.get()` + エラーハンドリングに置換する必要がある

### 重複ロジックの調査
- **コンテキスト**: `resolve_scene_id` と `resolve_scene_id_unified` の比較
- **情報源**: ソースコード直接調査
- **発見**:
  - Phase 3（キャッシュ取得/作成）、Phase 4（リセット）、Phase 5（選択）の3フェーズが完全に重複
  - 差異は Phase 1 のみ: `resolve_scene_id` はプレフィックス検索直接実行、`resolve_scene_id_unified` は `collect_scene_candidates` 委譲
  - `SceneCacheKey::new` の `module_name` 引数が `resolve_scene_id` では常に空文字
- **影響**: 共通キャッシュ処理メソッドの抽出で約40行の重複除去が可能

### デッドコード調査
- **コンテキスト**: `#[allow(dead_code)]` の存在確認
- **情報源**: `grep_search` 結果
- **発見**: pasta_core内に `#[allow(dead_code)]` は存在しない。未使用コードの有無はコンパイラ警告で確認要

### エラーメッセージ言語混在
- **コンテキスト**: エラー型のフォーマット文字列の言語
- **情報源**: `error.rs` 直接調査
- **発見**:
  - `SceneTableError` の全バリアント: 英語 ("Scene not found: {scene}" 等)
  - `WordTableError::WordNotFound`: 日本語 ("単語定義 @{key} が見つかりません")
  - エンドユーザー向けには日本語が適切だが、開発者向け内部エラーとしては英語が一般的
- **影響**: 統一方針を決定する必要がある。英語で統一が最も自然（Rustエコシステムの慣例）

## 設計判断

### 判断: キャッシュ処理の共通化アプローチ

- **コンテキスト**: `resolve_scene_id` と `resolve_scene_id_unified` の重複排除
- **検討した代替案**:
  1. 内部ヘルパーメソッド抽出 — キャッシュ取得・リセット・選択を1メソッドに
  2. `resolve_scene_id` を `resolve_scene_id_unified` のラッパーに変換
- **選択**: 案1（内部ヘルパーメソッド抽出）
- **理由**: 案2は `resolve_scene_id` の `module_name=""` 固定呼び出しになるが、`collect_scene_candidates` のモジュール検索ロジックが走るため、元のプレフィックス検索直接実行とはコードパスが異なる。ヘルパー抽出なら両メソッドの Phase 1 を変えずに Phase 3-5 を共通化できる
- **トレードオフ**: ヘルパーメソッドが `&mut self` を要するため借用分割が複雑になる可能性があるが、候補IDリストをヘルパーに渡す設計で回避可能
- **追跡**: タスク実装時に借用チェッカーとの整合性を検証

### 判断: エラーメッセージ言語の統一

- **コンテキスト**: `WordTableError` の日本語メッセージと `SceneTableError` の英語メッセージの混在
- **選択**: 英語で統一
- **理由**: Rustエコシステムの慣例に従い、エラーメッセージは英語とする。エンドユーザー向けのメッセージ変換は呼び出し側の責任
- **トレードオフ**: `WordTableError::WordNotFound` のメッセージを変更するが、`Display` トレイト実装は `thiserror` が自動生成するため、既存テストで `format!` や `.to_string()` でメッセージ文字列を検証している箇所があれば影響を受ける

## リスクと緩和策
- **リスク1**: 借用チェッカーとの衝突でヘルパー抽出が困難 → 候補IDリストとフィルタ結果を引数で渡す設計で回避
- **リスク2**: エラーメッセージ変更でテスト失敗 → テストケースも同時に更新
- **リスク3**: 内部リファクタリングで意図しない振る舞い変更 → 既存テスト全パス + 差分レビュー

## 参照
- [Rust APIガイドライン: 命名](https://rust-lang.github.io/api-guidelines/naming.html)
- [thiserror ドキュメント](https://docs.rs/thiserror/latest/thiserror/)
