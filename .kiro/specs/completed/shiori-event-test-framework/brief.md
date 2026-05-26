# Brief: shiori-event-test-framework

## Problem
SHIORIイベントが絡むテスト（プロパティ読み書き、非同期トーク等）の実装が煩雑。モック注入のボイラープレート、時刻制御の欠如、レスポンス検証の手動文字列マッチが各テストで繰り返されており、`shiori-async-talk` のような複雑な非同期往復テストを書く基盤がない。

## Current State
- `pasta_lua` のテストは `runtime.exec()` で Lua コードを直接実行し、`package.loaded["@pasta_*"]` を各テストで手動モック
- `pasta_shiori` のテストは RAW SHIORI リクエスト文字列を `PastaShiori::request()` に投入し、レスポンスを文字列マッチで検証
- 時刻制御不可: `parse_request()` が常に `OffsetDateTime::now_local()` を使用（`lua_date_from()` は固定時刻対応だが呼び出しパスがない）
- Lua テストは 28 スイートが `lua_specs/` に存在し `lua_test` BDD フレームワークを使用しているが、モック設定は各テスト内で個別実装

## Desired Outcome
- SHIORI プロトコルレベルのテスト: RAW リクエスト文字列 + 固定時刻 → Lua 経由レスポンス → 構造化検証が簡潔に書ける
- pasta_lua レベルのテスト: SHIORI 非依存で Lua モック一括注入、イベントディスパッチ、コルーチン制御が可能
- `shiori-async-talk` 仕様のテスト（マルチステップ SHIORI 往復）を自然に書ける前提条件が整う

## Approach
2層テスト環境アーキテクチャ:

**Layer 1 — pasta_lua（SHIORI 非依存）**:
- Lua モックライブラリ (`pasta.test.mocks`) で `@pasta_persistence`, `@pasta_search`, `@pasta_sakura_script`, `@pasta_config`, `@pasta_log` を一括スタブ化
- 既存 `lua_test` BDD フレームワーク上に構築
- 時刻は Lua テーブルのフィールドとして直接設定（`time` クレート非依存）

**Layer 2 — pasta_shiori（SHIORI プロトコルレベル）**:
- `parse_request()` に `X-Pasta-Time` カスタムヘッダー対応を追加（RFC 3339 形式、`time` クレートの `Rfc3339` パーサー使用）
- ヘッダーが存在すれば `now_local()` の代わりにその時刻で `req.date` を生成
- `ShioriResponse` パーサー（status, value, headers を構造化）でレスポンス検証を簡潔化
- RAW リクエスト文字列をそのまま投入するスタイル（ビルダーパターン不使用）

## Scope
- **In**:
  - `parse_request()` への `X-Pasta-Time` ヘッダー対応
  - Lua モックライブラリ (`pasta.test.mocks`)
  - `ShioriResponse` パーサー（Rust、`pasta_shiori` 内テストユーティリティ）
  - `ShioriTestEnv` ラッパー（フィクスチャ管理 + load + request の統合）
  - 各層の基本テストケースによる動作検証
- **Out**:
  - 既存テストの全面移行（本仕様では新基盤の提供のみ）
  - コルーチンステップ制御 API（`shiori-async-talk` 側で必要に応じて拡張）
  - `pasta_check` への `test` サブコマンド追加
  - Lua テストのパフォーマンス最適化

## Boundary Candidates
- Layer 1 (pasta_lua) と Layer 2 (pasta_shiori) の責務分離
- モックライブラリと実モジュール登録の境界

## Out of Boundary
- `shiori-async-talk` で必要となるコルーチン中断/再開のステップ制御 API（本仕様のモックライブラリを土台として shiori-async-talk 側で構築）
- 既存 28 テストスイートの書き換え（段階的に移行可能だが本仕様のスコープ外）
- Property SET/GET の具体的な実装テスト（property-write-helpers, shiori-async-talk 各仕様で実施）

## Upstream / Downstream
- **Upstream**: property-write-helpers（テスト対象の一つとして利用可能だが直接依存なし）
- **Downstream**: 
  - shiori-async-talk（マルチステップ SHIORI 往復テストの基盤として必須）
  - property-dsl-extension（将来フェーズのテストにも再利用）

## Existing Spec Touchpoints
- **Extends**: なし（新規テスト基盤）
- **Adjacent**: property-write-helpers（テスト方法の参考例）、shiori-async-talk（主要ユーザー）

## Constraints
- `pasta_lua` は `time` クレートに依存しない（SHIORI 非依存性の維持）
- `X-Pasta-Time` ヘッダーは SHIORI/3.0 の `key_other` として自然にパースされる（PEG 文法変更不要）
- RFC 3339 形式: `2026-05-20T14:00:00+09:00`
- 既存テストとの後方互換性: `X-Pasta-Time` なしのリクエストは従来通り `now_local()` を使用
