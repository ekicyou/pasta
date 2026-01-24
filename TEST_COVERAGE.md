# Test Coverage Map

このドキュメントは、[SOUL.md](SOUL.md) で定義されたコア機能と、それを検証するテストの対応関係を示します。

**最終更新**: 2026-01-24  
**総テスト数**: 313テスト（pasta_shiori 5テスト失敗中）

---

## 1. コアバリューのテストカバレッジ

| コアバリュー | 対応テスト | 状態 | テスト数 |
|-------------|-----------|------|---------|
| 日本語フレンドリー（全角キーワード） | `transpiler_integration_test.rs` | ✅ 完了 | 24 |
| UNICODE識別子（日本語シーン名・変数名） | `japanese_identifier_test.rs`<br>`ucid_test.rs` | ✅ 完了 | 2<br>3 |
| yield型エンジン（継続出力） | `transpiler_integration_test.rs` | ✅ 完了 | - |
| 宣言的フロー（Call制御） | `transpiler_integration_test.rs` | ✅ 完了 | - |

---

## 2. DSL文法機能のテストカバレッジ

### 2.1 Parser層テスト（文法解析）

| 機能 | テストファイル | 状態 | 説明 |
|------|---------------|------|------|
| グローバルシーン（＊） | `transpiler_integration_test.rs` | ✅ 完了 | シーン定義パース |
| ローカルシーン（・） | `transpiler_integration_test.rs` | ✅ 完了 | サブシーン定義 |
| アクター定義（％） | `actor_code_block_test.rs` | ✅ 完了 | アクターコードブロック |
| 属性定義（＆） | フィクスチャあり | 🔶 部分 | `transpiler2/attribute_inheritance.pasta` |
| 単語定義（＠） | `actor_word_dictionary_test.rs` | ✅ 完了 | 単語定義・参照 |
| 変数定義（＄） | フィクスチャあり | 🔶 部分 | `transpiler2/variable_scope.pasta` |
| Call文（＞） | `transpiler_integration_test.rs` | ✅ 完了 | 制御フロー |
| コメント行（＃） | 暗黙的テスト | 🔶 部分 | 明示的テストなし |
| アクション行（発言） | `transpiler_integration_test.rs` | ✅ 完了 | キャラクター発言 |
| Luaコードブロック | `actor_code_block_test.rs` | ✅ 完了 | 関数定義 |
| バイトオフセット | `span_byte_offset_test.rs` | ✅ 完了 | エラー位置特定 |

### 2.2 Registry層テスト（シーン/単語テーブル）

| 機能 | テストファイル | 状態 | 説明 |
|------|---------------|------|------|
| シーン前方一致検索 | `search_module_test.rs`<br>`fallback_search_integration_test.rs` | ✅ 完了 | 前方一致+ランダム選択 |
| 単語前方一致検索 | `search_module_test.rs` | ✅ 完了 | 単語辞書検索 |
| 単語ランダム選択 | `search_module_test.rs` | 🔶 部分 | ランダム性検証 |
| アクター単語辞書 | `actor_word_dictionary_test.rs` | ✅ 完了 | アクタースコープ単語 |
| finalize_scene処理 | `finalize_scene_test.rs` | ✅ 完了 | シーン初期化 |

### 2.3 Transpiler層テスト（Lua変換）

| 機能 | テストファイル | 状態 | 説明 |
|------|---------------|------|------|
| 包括的制御フロー | `transpiler_integration_test.rs` | ✅ 完了 | 24テストケース |
| 変数スコープ | フィクスチャあり | 🔶 部分 | Local/Global変数 |
| Call/末尾Call最適化 | `transpiler_integration_test.rs` | ✅ 完了 | 自動判定 |
| エンコーディング | `pasta_lua_encoding_test.rs` | ✅ 完了 | 文字エンコード |

### 2.4 Runtime層テスト（実行エンジン）

| 機能 | テストファイル | 状態 | 説明 |
|------|---------------|------|------|
| Luaスクリプトローダー | `loader_integration_test.rs` | ✅ 完了 | スクリプト読み込み |
| 標準ライブラリモジュール | `stdlib_modules_test.rs` | ✅ 完了 | stdlib機能 |
| 正規表現モジュール | `stdlib_regex_test.rs` | ✅ 完了 | 14テスト |
| Lua単体テスト実行 | `lua_unittest_runner.rs` | ✅ 完了 | Luaテストランナー |

### 2.5 統合テスト（E2E）

| 機能 | テストファイル | 状態 | 説明 |
|------|---------------|------|------|
| SHIORI.DLL インターフェース | `shiori_lifecycle_test.rs` | ❌ 失敗中 | 5テスト失敗 |
| SHIORI リクエスト処理 | `lua_request_test.rs` | ✅ 完了 | 18+テスト |

---

## 3. Phase 0完了基準との対応

[SOUL.md Section 5.6](SOUL.md#56-phase-0完了基準definition-of-done) のDoD項目とテストの対応：

| DoD項目 | 対応テスト | 状態 |
|---------|-----------|------|
| SPECIFICATION.md 全マーカー定義完了 | - | ✅ ドキュメント |
| 全角/半角対応表の完全性 | - | ✅ ドキュメント |
| cargo test pasta_core 100%パス | 全pasta_coreテスト | ✅ 91テスト |
| cargo test pasta_lua 100%パス | 全pasta_luaテスト | ✅ 222テスト |
| cargo test pasta_shiori 100%パス | `shiori_lifecycle_test.rs` | ❌ 5テスト失敗 |
| comprehensive_control_flow検証 | `transpiler_integration_test.rs` | 🔶 Lua版Golden Test未整備 |
| スナップショットテスト整備 | - | ❌ 未実装 |
| 最適化レベルの文書化 | - | ❌ 未実装 |
| ドキュメント整合性検証 | - | 🔶 手動検証のみ |
| TEST_COVERAGE.md作成 | - | ✅ 本ドキュメント |
| 未テスト領域の特定 | - | 🔶 本ドキュメント Section 4参照 |
| シーンテーブル設計レビュー | - | ❌ 未実施 |
| Call文の実装 | `transpiler_integration_test.rs` | ✅ 完了 |
| リグレッションテスト整備 | 全313テスト | ✅ 完了 |

---

## 4. 未テスト領域・改善点

### 4.1 明示的テストが不足している領域

| 機能 | 現状 | 推奨アクション |
|------|------|--------------|
| コメント行（＃）パース | 暗黙的テストのみ | 明示的テストケース追加 |
| 属性定義（＆）の継承 | フィクスチャのみ | テストケース実装 |
| 変数スコープ（Local/Global/System） | フィクスチャのみ | テストケース実装 |
| 単語ランダム選択の検証 | 部分的 | ランダム性テスト追加 |
| エラーメッセージの具体性 | 未検証 | エラーケーステスト追加 |

### 4.2 Golden Test（スナップショットテスト）未整備

現在、`comprehensive_control_flow.pasta` に対応する `.rn` ファイルが存在しますが、これはrune時代の遺物です。Lua出力に対するスナップショットテストが必要です。

**推奨**: `insta` crateを使用したスナップショットテスト導入

### 4.3 pasta_shiori失敗テストの修正

以下5テストが失敗中（Phase 0完了のブロッカー）：

- `test_shiori_load_sets_globals`
- `test_shiori_request_calls_pasta_scene`
- `test_shiori_request_increments_counter`
- `test_shiori_unload_creates_marker`
- `test_shiori_lifecycle_lua_execution_verified`

**原因**: `shiori.load()` の呼び出しが失敗（アサーション失敗）

---

## 5. テストカバレッジサマリー

| クレート | テスト数 | パス | 失敗 | カバレッジ評価 |
|---------|---------|------|------|--------------|
| pasta_core | 94 | 94 | 0 | ⭐⭐⭐⭐⭐ 優秀 |
| pasta_lua | 219 | 219 | 0 | ⭐⭐⭐⭐ 良好 |
| pasta_shiori | 23+ | 18+ | 5 | ⭐⭐ 要改善 |
| **合計** | **313+** | **308+** | **5** | **98.4%パス率** |

---

## 6. 次のステップ

### Phase 0完了に向けて

1. **優先度 High**: pasta_shiori 5テスト失敗の修正
2. **優先度 High**: Golden Test（スナップショットテスト）整備
3. **優先度 Medium**: 最適化レベルの文書化
4. **優先度 Medium**: シーンテーブル設計レビュー
5. **優先度 Low**: 未テスト領域への明示的テスト追加

### 保守・拡張

- 新規テスト追加時は本ドキュメントを更新
- 四半期ごとにテストカバレッジレビュー実施
- Phase 1以降の機能追加時は対応するテストを先に作成（Test-First）

---

**参照**:
- [SOUL.md](SOUL.md) - プロジェクトの憲法
- [.kiro/specs/soul-document/gap-analysis.md](.kiro/specs/soul-document/gap-analysis.md) - ギャップ分析レポート
