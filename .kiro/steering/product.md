# Product Steering
Memories of pasta twine together—now and then a knot, yet always a delight.

> **📖 プロジェクトビジョン・設計原則**: [SOUL.md](../../../SOUL.md) を参照してください。  
> このドキュメントは開発ロードマップと進捗管理に特化しています。

## プロジェクト概要
**pasta**は、「伺か」のようなデスクトップマスコットなどを実現するための対話スクリプトエンジンです。Pasta DSLをLuaにトランスパイルし、「ゴースト」基盤として機能します。

**ビジョン・コアバリュー・設計原則**: [SOUL.md](../../../SOUL.md) 参照

## 機能の優先順位

### Phase 0: 一次設計の再構築 ✅ 完了
**最終更新**: 2026-01-30

- [x] 「パスタスクリプト」DSL設計の見直し → [doc/spec/](../../../doc/spec/) 完成
- [x] ２パストランスパイル設計の再検討 → `pasta-lua-cache-transpiler` 完了
- [x] シーンジャンプテーブル設計の修正 → `scene-search-integration` 完了
- [x] 宣言的制御フロー（Call/Jump文）の再実装 → `act-impl-call` 完了

**完了仕様**: 110件（`.kiro/specs/completed/` に格納）

**主要成果**:
- ✅ **act-impl-call** - `ACT_IMPL.call` 4段階優先順位検索実装
- ✅ **event-handler-call-equivalence** - `act:find_scene()` 統合名前解決（イベント/call 経路1本化）
- ✅ **scene-search-integration** - `SCENE.search()` 動的シーン検索機能
- ✅ **pasta-transpiler-variable-expansion** - 変数スコープ管理（Local/Global）
- ✅ **remove-root-crate** - Pure Virtual Workspace化
- ✅ **pasta_search_module** - Rust/Lua間の辞書検索バインディング
- ✅ **shiori-entry** - SHIORI APIエントリポイント

**品質指標**: 950+ テスト全パス、リグレッション0件

### Phase 1: 基盤確立 ✅ 完了
**現状**: Phase 0の再構築により基盤確立完了

- [x] パーサー（pasta_core）- Pasta DSL解析
- [x] トランスパイラ（pasta_lua）- Lua コード生成
- [x] ランタイム（pasta_lua）- Lua 5.5 実行環境
- [x] SHIORI インターフェース（pasta_shiori）- DLL エクスポート

### Phase 2: コア機能拡張（進行中）🔄
**最終更新**: 2026-06-03（現行バージョン v0.1.23）

**繰り返し仕様**:
- 🔁 **release-workflow** - リリース作業手順（`/kiro-spec-impl` 実行のたびにタスクリセット、永続的に未完了）

**完了仕様**:
- ✅ **pasta-cue-dsl-extension** - キューコマンド行(`!`)のDSLパース機能追加
- ✅ **release-workflow-dll-zip-fix** - リリースワークフローDLL ZIP修正
- ✅ **yield-continuation-token** - `＞チェイントーク` / `＞yield` 継続トーク機能（GLOBAL テーブル L3 登録）
- ✅ **lua55-manual-consistency** - Lua 5.5 リファレンスマニュアル日本語化整合性
  - マニュアル本体は独立リポジトリに移行: [ekicyou/lua55-manual-ja](https://github.com/ekicyou/lua55-manual-ja)
- ✅ **budoux-line-breaker** - BudouX日本語改行位置推定の統合
- ✅ **pasta-check** - `pasta_check` CLIツール（NAR生成・更新ファイル管理）
- ✅ **luajit-migration** - Lua 5.5からLuaJIT 2.1への移行

**SSPプロパティアクセス機能群**（トーク合成中の同期/非同期 SHIORI 通信基盤）:
- ✅ **property-write-helpers** - `act:set_property(name, value)` プロパティ書き込み
- ✅ **shiori-event-test-framework** - SHIORIイベントフロー試験基盤（Luaモック + 時刻注入 + 応答検証）
- ✅ **shiori-async-talk** - トーク中SHIORI非同期通信基盤 + `act:get_property(name)`
- ✅ **property-dsl-extension** - `＄％prop.path` スコープ修飾子によるプロパティアクセスDSL構文

**DSL/ランタイム拡張**:
- ✅ **choice-definition-dsl** - 選択肢定義DSL
- ✅ **handler-resolution-fallback** - ハンドラ名前解決フォールバック
- ✅ **ontalk-block-condition** - OnTalkブロック条件

**Phase 3（監査）: 脆弱性監査・コード簡素化 ✅ 完了**（全クレート対象、外部振る舞い不変）:
- ✅ **Wave 1**: `audit-pasta-core` / `audit-pasta-dsl` / `audit-pasta-lua` / `audit-pasta-shiori` / `audit-pasta-check` / `audit-pasta-lsp` / `audit-pasta-sample-ghost`
- ✅ **Wave 2**: `audit-dependency-supply-chain`（依存サプライチェーン監査・cargo-deny導入） / `audit-workspace-patterns`（クレート横断パターン統一）

**保留/評価中仕様**:
- ⏸️ **pasta-conversation-inline-multi-stage-resolution** - 動的単語参照（Phase 3相当、削除検討中）
- ⏸️ **ukagaka-desktop-mascot** - メタ仕様（Phase 4相当）

### Phase 3: 高度機能（計画中）
- [ ] シーン継続チェーン（`pasta-label-continuation`）
- [ ] 動的単語参照（`＠＄変数` - doc/spec/11-actor-dictionary.mdで文法予約済み）
- [ ] ランタイム拡充・使い勝手向上
- [ ] イベントハンドリングの拡充

### Phase 4: エコシステム統合（将来）
- [ ] SHIORI.DLLとしてのコンパイル
- [ ] arekaへの投入（`ukagaka-desktop-mascot`メタ仕様）
- [ ] MCPまたはLLMとの連携

**現在地**: Phase 2（コア機能拡張）- **基盤確立済み** ✅
