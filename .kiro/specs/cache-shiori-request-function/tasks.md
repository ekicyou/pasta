# Implementation Plan

## Task Overview

本仕様では、PastaShiori における SHIORI Lua 関数（load/request/unload）のキャッシュ機能を実装します。struct フィールドのリファクタリング、メソッドの簡素化、Drop impl の拡張、テスト修正を段階的に進めます。

---

## Implementation Tasks

### Phase 1: 構造体リファクタリング

- [ ] 1. PastaShiori 構造体のフィールド変更
- [ ] 1.1 (P) 新規フィールドの追加と既存フラグの削除
  - `load_fn: Option<Function>`, `request_fn: Option<Function>`, `unload_fn: Option<Function>` の3フィールドを追加
  - `has_shiori_load: bool` と `has_shiori_request: bool` を削除
  - struct の Send/Sync 制約を維持
  - _Requirements: 1.1, 5.1_

- [ ] 1.2 (P) cache_shiori_functions メソッドの実装
  - Lua グローバルから SHIORI テーブルを取得
  - load/request/unload 関数を取得し、各フィールドにキャッシュ
  - SHIORI テーブル未存在時: warn! ログ + 全キャッシュ None 設定
  - 個別関数取得失敗時: 該当フィールドのみ None 設定
  - _Requirements: 1.2, 1.3_

- [ ] 1.3 (P) clear_cached_functions メソッドの実装
  - load_fn, request_fn, unload_fn の3フィールドを None に設定
  - reload 時に runtime クリア前に呼び出し
  - _Requirements: 1.4_

### Phase 2: 既存メソッドの最適化

- [ ] 2. request() メソッドの簡素化
- [ ] 2.1 キャッシュされた request_fn の利用
  - request_fn が Some の場合、キャッシュされた関数を直接呼び出し
  - request_fn が None の場合、デフォルト 204 応答を返却
  - globals.get("SHIORI") 呼び出しを削除
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 3. call_shiori_load() メソッドの最適化
- [ ] 3.1 キャッシュされた load_fn の利用
  - load_fn が Some の場合、キャッシュされた関数を直接呼び出し
  - load_fn が None の場合、SHIORI.load 呼び出しをスキップして成功を返却
  - globals.get("SHIORI") 呼び出しを削除
  - _Requirements: 3.1, 3.2_

- [ ] 4. load() メソッドの統合
- [ ] 4.1 キャッシュ取得ロジックの組み込み
  - reload 時の clear_cached_functions() 呼び出し追加
  - runtime 取得後に cache_shiori_functions() を呼び出し
  - call_shiori_load() 実行時にキャッシュを活用
  - _Requirements: 1.2, 1.4_

### Phase 3: unload サポート追加

- [ ] 5. Drop impl への unload 機能追加
- [ ] 5.1 unload 関数の安全な呼び出し
  - unload_fn と runtime の両方が Some の場合のみ unload を呼び出し（`if let` パターン使用）
  - unload 呼び出しを runtime drop 前に実行
  - unload 失敗時は warn! ログのみでエラーを伝播させない
  - _Requirements: 4.1, 4.2, 4.3_

### Phase 4: テスト修正と追加

- [ ] 6. 既存テストの修正
- [ ] 6.1 (P) フラグ判定のリファクタリング
  - test_load_sets_shiori_flags_when_main_lua_exists: `has_shiori_load` → `load_fn.is_some()` に変更
  - test_load_flags_false_without_main_lua: `has_shiori_*` → `*_fn.is_some()` に変更
  - その他のフラグ参照箇所を Option::is_some() に統一
  - _Requirements: 5.2, 6.1_

- [ ] 7. 新規テストの追加
- [ ] 7.1 (P) unload 呼び出しの検証テスト
  - test_unload_called_on_drop: Lua 側でグローバルフラグを設定し、drop 後にフラグを確認
  - PastaShiori インスタンスを drop して unload が正しく呼ばれることを検証
  - _Requirements: 4.1, 4.3_

- [ ] 7.2 (P) unload エラー耐性テスト
  - test_unload_error_does_not_panic: エラーを返す unload 関数を定義
  - drop 時にパニックせず warn ログが出力されることを確認
  - _Requirements: 4.2_

- [ ] 7.3 (P) reload 時のキャッシュクリアテスト
  - test_cached_functions_cleared_on_reload: 2回 load() を実行し、2回目で正しいキャッシュが設定されることを確認
  - reload 前のキャッシュが適切にクリアされることを検証
  - _Requirements: 1.4, 6.2_

- [ ]* 7.4 複数インスタンス独立性テスト
  - 複数の PastaShiori インスタンスを並行作成し、キャッシュが独立していることを確認
  - インスタンス間でキャッシュが混ざらないことを検証
  - _Requirements: 6.3_

### Phase 5: 統合テストと検証

- [ ] 8. 全体検証
- [ ] 8.1 ビルド・テスト実行
  - `cargo check -p pasta_shiori` でビルドエラーがないことを確認
  - `cargo test -p pasta_shiori` で全テストが合格することを検証
  - リグレッションがないことを確認
  - _Requirements: 6.1_

- [ ] 8.2 パフォーマンス検証
  - request() と call_shiori_load() で globals.get 呼び出しが削除されていることを確認
  - キャッシュ利用により関数ルックアップが 0 回になることを検証
  - _Requirements: 2.3, 3.1_

---

## Requirements Coverage Summary

| Requirement | Mapped Tasks |
|-------------|--------------|
| 1.1 | 1.1 |
| 1.2 | 1.2, 4.1 |
| 1.3 | 1.2 |
| 1.4 | 1.3, 4.1, 7.3 |
| 2.1 | 2.1 |
| 2.2 | 2.1 |
| 2.3 | 2.1, 8.2 |
| 3.1 | 3.1, 8.2 |
| 3.2 | 3.1 |
| 4.1 | 5.1, 7.1 |
| 4.2 | 5.1, 7.2 |
| 4.3 | 5.1, 7.1 |
| 5.1 | 1.1 |
| 5.2 | 6.1 |
| 6.1 | 6.1, 8.1 |
| 6.2 | 7.3 |
| 6.3 | 7.4 |

**全17要件をカバー（6グループ、17受入基準）**
