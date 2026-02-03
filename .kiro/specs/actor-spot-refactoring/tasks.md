# Implementation Plan

## Overview
act層の状態レス化とビルダー層への責務移譲を実現する。トークン生成を純粋化し、状態管理をsakura_builderに集約することで、テスタビリティと保守性を向上させる。

## Tasks

### 1. トークン生成層のリファクタリング
- [ ] 1.1 (P) set_spot()のトークン化
  - `self.actors[name]`でActorオブジェクトを取得
  - `{type="spot", actor=Actor, spot=spot}`トークンを生成
  - actorオブジェクトの`spot`属性を変更しない（状態レス化）
  - _Requirements: 1.1, 5.1, 5.2_

- [ ] 1.2 (P) clear_spot()のトークン化
  - `{type="clear_spot"}`トークンを生成
  - actorオブジェクトの状態を一切変更しない
  - _Requirements: 6.1, 6.2_

- [ ] 1.3 talk()からのactor情報追加とトークン化
  - `{type="talk", actor=actor, text=text}`トークン生成に変更
  - `now_actor`比較とactor切り替え検出ロジックを削除
  - `{type="actor"}`および`{type="spot_switch"}`トークン生成を削除
  - _Requirements: 1.2, 1.4_

- [ ] 1.4 ACT_IMPLからの状態フィールド削除
  - `now_actor`フィールドを削除
  - `_current_spot`フィールドを削除
  - `build()`での状態リセット処理を削除
  - _Requirements: 1.3_

### 2. ビルダー層の状態管理実装
- [ ] 2.1 spotトークン処理の実装
  - build()内にローカル状態`actor_spots = {}`を追加
  - spotトークン処理で`actor_spots[token.actor.name] = token.spot`を実行
  - _Requirements: 2.1, 5.3_

- [ ] 2.2 clear_spotトークン処理の実装
  - clear_spotトークン検出時に`actor_spots = {}`でリセット
  - `last_actor = nil`でリセット
  - _Requirements: 6.3, 6.4_

- [ ] 2.3 talkトークンのactor切り替え検出と出力
  - build()内にローカル状態`last_actor = nil`を追加
  - spot解決ロジック: `local spot = actor_spots[token.actor.name] or 0`
  - `last_actor ~= token.actor`時に`\p[spot]`を出力
  - spot変更検出時に`\n[N]`を出力（Nは`config.spot_newlines * 100`）
  - テキスト出力後に`last_actor = token.actor`を更新
  - 既存の`{type="actor"}`および`{type="spot_switch"}`トークン処理を削除
  - _Requirements: 2.2, 2.3, 2.4_

### 3. 設定プロパティの更新
- [ ] 3.1 BuildConfigプロパティ名の変更
  - `spot_switch_newlines`を`spot_newlines`に変更
  - デフォルト値1.5を維持
  - _Requirements: 3.1, 3.2_

### 4. テストの更新と強化
- [ ] 4.1 act_test.lua: トークン生成の検証
  - set_spot()が正しいトークン構造`{type="spot", actor=Actor, spot=N}`を生成することを確認
  - set_spot()呼び出し後も`actor.spot`が変更されていないことを検証
  - clear_spot()が正しいトークン構造`{type="clear_spot"}`を生成することを確認
  - clear_spot()呼び出し後もactorオブジェクトが変更されていないことを検証
  - talk()が`{type="talk", actor=Actor, text=text}`トークンを生成することを確認
  - talk()が状態管理（now_actor, _current_spot）を行わないことを確認
  - _Requirements: 4.1_

- [ ] 4.2 act_test.lua: アクタープロキシの独立性検証
  - set_spot()呼び出しなしで`act.さくら:talk()`が動作することを確認
  - `ACT_IMPL.__index`によるプロキシ生成が正常に機能することを確認
  - プロキシ経由のtalk()が正しいトークン（actor付き）を生成することを確認
  - _Requirements: 4.1_

- [ ] 4.3 sakura_builder_test.lua: 状態管理の検証
  - `actor_spots`未設定時にデフォルト値0を使用することを確認
  - spotトークン処理で`actor_spots[actor.name]`が正しく更新されることを確認
  - talkトークン処理でspot解決（`actor_spots[actor.name] or 0`）を確認
  - clear_spotトークン処理で`actor_spots={}`と`last_actor=nil`にリセットされることを確認
  - _Requirements: 4.2_

- [ ] 4.4 sakura_builder_test.lua: 統合シナリオの検証
  - set_spot()なしでのtalk() → デフォルトspot(0)使用を確認
  - set_spot() → talk() → spot切り替えとスポットタグ出力を確認
  - clear_spot() → talk() → デフォルトspot(0)への復帰を確認
  - 複数actorのspot独立管理を確認
  - 既存のactor/spot_switchトークンテストを新トークン構造に更新
  - _Requirements: 4.2, 4.3_

### 5. 統合と検証
- [ ] 5.1 既存テストスイート全体の実行
  - act_test.luaの全テストが成功することを確認
  - sakura_builder_test.luaの全テストが成功することを確認
  - 既存の統合テストが引き続き成功することを確認
  - _Requirements: 4.3_

## Task Summary
- **合計**: 5メジャータスク、11サブタスク
- **並列実行可能**: タスク1.1, 1.2（トークン生成は独立）
- **平均作業時間**: 1-3時間/サブタスク
- **全要件カバー**: 6要件、24受け入れ基準すべてをカバー
