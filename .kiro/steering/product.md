# Product Steering
Memories of pasta twine together—now and then a knot, yet always a delight.

> **📖 プロジェクトビジョン・設計原則**: [SOUL.md](../../../SOUL.md) を参照してください。  
> このドキュメントは開発ロードマップと進捗管理に特化しています。

## プロジェクト概要
**pasta**は、「伺か」のようなデスクトップマスコットなどを実現するための対話スクリプトエンジンです。Pasta DSLをLuaにトランスパイルし、「ゴースト」基盤として機能します。

**ビジョン・コアバリュー・設計原則**: [SOUL.md](../../../SOUL.md) 参照

## 機能の優先順位

### Phase 0: 一次設計の再構築（進行中）⚠️
**現状**: 過去11仕様は完了扱いだが、実装品質が要件定義意図を満たしておらず、**大規模な差し戻し・再設計中**

- [ ] 「パスタスクリプト」DSL設計の見直し
- [ ] ２パストランスパイル設計の再検討
- [ ] シーンジャンプテーブル設計の修正
- [ ] 宣言的制御フロー（Call/Jump文）の再実装

**完了仕様**: 
- ✅ **scene-search-integration** (2026-01-27)
  - SCENE.search() 動的シーン検索機能実装
  - 14テスト全合格、リグレッション0件
  - [VALIDATION_REPORT.md](./.kiro/specs/completed/scene-search-integration/VALIDATION_REPORT.md)
- ✅ **pasta-transpiler-variable-expansion** (2025-12-21)
  - 変数スコープ管理（Local/Global）実装完了
  - 20テスト合格、リグレッション0件
  - [IMPLEMENTATION_COMPLETION_REPORT.md](./.kiro/specs/completed/pasta-transpiler-variable-expansion/IMPLEMENTATION_COMPLETION_REPORT.md)
- ✅ **remove-root-crate** (2025-12-31)
  - Pure Virtual Workspace 化（ルートクレート削除）
  - 全タスク完了、182テスト成功
  - [COMPLETION_REPORT.md](./.kiro/specs/completed/remove-root-crate/COMPLETION_REPORT.md)

**課題**:
- DSL文法の曖昧性・不完全性
- トランスパイル結果の品質問題
- シーンテーブル設計の不備
- 要件と実装の乖離

**過去の「完了」仕様**: 31件（完了済み）

### Phase 1: 基盤確立（未達）
Phase 0の再構築が完了するまで、基盤確立とは言えない状態。

### Phase 2: コア機能実装（保留）

**進行中仕様**: 8件（Phase 0完了後に着手）

### Phase 3: 高度機能（計画中）
- [ ] シーン継続チェーン（`pasta-label-continuation`）
- [ ] インライン多段解決（`pasta-conversation-inline-multi-stage-resolution`）
- [ ] rune側ランタイムを拡充し、使い勝手を向上させる
- [ ] イベントハンドリングの拡充

### Phase 4: エコシステム統合（将来）
- [ ] SHIORI.DLLとしてのコンパイル
- [ ] arekaへの投入（`ukagaka-desktop-mascot`メタ仕様 - 32子仕様管理中）
- [ ] MCPまたはLLMとの連携（`areka-P0-mcp-server`）

**現在地**: Phase 0（一次設計再構築中）- **基盤未確立**
