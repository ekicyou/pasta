# Implementation Plan

## タスク概要

本実装では、SHIORI/3.0 プロトコルの最小実装として、main.lua を介した SHIORI.load/request 処理を追加する。8つの要件すべてに対応し、段階的な実装とテストを行う。

---

## タスク一覧

- [ ] 1. エラー型拡張と mlua::Error 変換実装
- [ ] 1.1 (P) MyError に mlua::Error からの From trait 実装
  - MyError::Script variant が既存であることを確認
  - mlua::Error から MyError への変換を実装
  - エラーメッセージを文字列化して MyError::Script に格納
  - _Requirements: 5_

- [ ] 2. PastaShiori 構造体拡張
- [ ] 2.1 (P) has_shiori_load と has_shiori_request フィールド追加
  - PastaShiori 構造体に bool フィールド2つを追加
  - Default derive 動作を確認（bool は自動で false）
  - _Requirements: 6_

- [ ] 3. main.lua 自動ロード機能実装
- [ ] 3.1 PastaLuaRuntime::from_loader に main.lua ロードロジック追加
  - transpiled コードロード後に main.lua パス構築
  - ファイル存在確認、読み込み、Lua VM へのロード
  - エラーハンドリング: warn ログで継続、エラー伝播なし
  - _Requirements: 1_

- [ ] 4. main.lua スクリプト作成
- [ ] 4.1 (P) scripts/pasta/shiori/main.lua 新規作成
  - SHIORI グローバルテーブル定義
  - SHIORI.load(hinst, load_dir) 関数実装（常に true 返却）
  - SHIORI.request(request_text) 関数実装（204 No Content 返却）
  - SHIORI/3.0 レスポンス形式準拠（CRLF, Charset, Sender ヘッダー）
  - _Requirements: 2, 3, 7_

- [ ] 5. PastaShiori::load メソッド拡張
- [ ] 5.1 SHIORI 関数存在確認ロジック実装
  - PastaLoader::load 成功後に SHIORI テーブル取得
  - SHIORI.load 関数存在確認、has_shiori_load フラグ設定
  - SHIORI.request 関数存在確認、has_shiori_request フラグ設定
  - 各段階のエラーハンドリング（warn ログ、フラグ false）
  - _Requirements: 6_

- [ ] 5.2 SHIORI.load 呼び出し実装
  - has_shiori_load が true の場合のみ実行
  - SHIORI テーブルから load 関数を都度取得
  - hinst と load_dir を引数として関数呼び出し
  - 戻り値が false の場合は Ok(false) 返却
  - Lua 実行エラー時は error ログ後 Ok(false) 返却
  - _Requirements: 4, 6_

- [ ] 6. PastaShiori::request メソッド実装
- [ ] 6.1 SHIORI.request 呼び出しロジック実装
  - runtime 未初期化チェック（MyError::NotInitialized）
  - has_shiori_request フラグ確認
  - false の場合はデフォルト 204 レスポンス返却
  - true の場合は SHIORI テーブルから request 関数を都度取得
  - request_text を引数として関数呼び出し
  - 戻り値を String として取得、呼び出し元に返却
  - Lua 実行エラー時は MyError::Script 返却
  - _Requirements: 5, 6, 7_

- [ ] 7. 単体テスト実装
- [ ] 7.1 (P) PastaShiori::load テスト追加
  - main.lua 存在時に has_shiori_* フラグが true になることを検証
  - main.lua 不在時に has_shiori_* フラグが false になることを検証
  - SHIORI.load が false を返す場合の動作検証
  - _Requirements: 8_

- [ ] 7.2 (P) PastaShiori::request テスト追加
  - SHIORI.request 呼び出しでレスポンス取得を検証
  - 関数不在時にデフォルト 204 返却を検証
  - Lua エラー時に MyError::Script 返却を検証
  - _Requirements: 8_

- [ ] 8. 統合テスト実装
- [ ] 8.1 テスト用 Fixture 作成
  - tests/fixtures/shiori/minimal-main/ ディレクトリ作成
  - pasta.toml, dic/test.pasta, scripts/pasta/shiori/main.lua を配置
  - main.lua に SHIORI.load/request 実装（テスト用）
  - _Requirements: 8_

- [ ] 8.2 E2E テスト追加
  - load → request → 204 の一連フロー検証
  - Lua 側で hinst/load_dir が正しく受信できることを検証
  - SHIORI/3.0 レスポンス形式準拠を検証
  - _Requirements: 8_

---

## 要件カバレッジ検証

| 要件 ID | 対応タスク |
|--------|-----------|
| Req 1 | 3.1 |
| Req 2 | 4.1 |
| Req 3 | 4.1 |
| Req 4 | 5.2 |
| Req 5 | 6.1 |
| Req 6 | 2.1, 5.1, 5.2, 6.1 |
| Req 7 | 4.1, 6.1 |
| Req 8 | 7.1, 7.2, 8.1, 8.2 |

**カバレッジ**: 8/8 要件すべてマッピング済み ✅
