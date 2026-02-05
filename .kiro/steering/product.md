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

- [x] 「パスタスクリプト」DSL設計の見直し → [SPECIFICATION.md](../../../SPECIFICATION.md) 完成
- [x] ２パストランスパイル設計の再検討 → `pasta-lua-cache-transpiler` 完了
- [x] シーンジャンプテーブル設計の修正 → `scene-search-integration` 完了
- [x] 宣言的制御フロー（Call/Jump文）の再実装 → `act-impl-call` 完了

**完了仕様**: 48件（`.kiro/specs/completed/` に格納）

**主要成果**:
- ✅ **act-impl-call** - `ACT_IMPL.call` 4段階優先順位検索実装
- ✅ **scene-search-integration** - `SCENE.search()` 動的シーン検索機能
- ✅ **pasta-transpiler-variable-expansion** - 変数スコープ管理（Local/Global）
- ✅ **remove-root-crate** - Pure Virtual Workspace化
- ✅ **pasta_search_module** - Rust/Lua間の辞書検索バインディング
- ✅ **shiori-entry** - SHIORI APIエントリポイント

**品質指標**: 340+ テスト全パス、リグレッション0件

### Phase 1: 基盤確立 ✅ 完了
**現状**: Phase 0の再構築により基盤確立完了

- [x] パーサー（pasta_core）- Pasta DSL解析
- [x] トランスパイラ（pasta_lua）- Lua コード生成
- [x] ランタイム（pasta_lua）- Lua 5.5 実行環境
- [x] SHIORI インターフェース（pasta_shiori）- DLL エクスポート

### Phase 2: コア機能拡張（進行中）🔄

**進行中仕様**: 1件
- 🔄 **lua55-manual-consistency** - Lua 5.5 リファレンスマニュアル日本語化整合性
  - マニュアル本体は独立リポジトリに移行: [ekicyou/lua55-manual-ja](https://github.com/ekicyou/lua55-manual-ja)

**保留/評価中仕様**:
- ⏸️ **pasta-conversation-inline-multi-stage-resolution** - 動的単語参照（Phase 3相当、削除検討中）
- ⏸️ **ukagaka-desktop-mascot** - メタ仕様（Phase 4相当）

### Phase 3: 高度機能（計画中）
- [ ] シーン継続チェーン（`pasta-label-continuation`）
- [ ] 動的単語参照（`＠＄変数` - SPECIFICATION.md 11.7で文法予約済み）
- [ ] ランタイム拡充・使い勝手向上
- [ ] イベントハンドリングの拡充

### Phase 4: エコシステム統合（将来）
- [ ] SHIORI.DLLとしてのコンパイル
- [ ] arekaへの投入（`ukagaka-desktop-mascot`メタ仕様）
- [ ] MCPまたはLLMとの連携

**現在地**: Phase 2（コア機能拡張）- **基盤確立済み** ✅
