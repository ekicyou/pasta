# Requirements Document

## Project Description (Input)
act.tokenの"actor"と、"spot_switch"の役割が重複している。実際にはactorが変われば明示的にスポットが変わる。それとも、actorのスポット位置変更に対応させる予定かな？まず、"spot_switch"⇒"spot"に変更。また、actorとspotの関係を整理してリファクタリングする。

## Introduction
本仕様は、`pasta.act`モジュールとビルダーの責務を明確に分離し、状態管理をビルダー層に移すことで、より保守性の高い設計へリファクタリングすることを目的とする。

### 現状の問題点
1. **責務の混在**: `act:talk()`が状態管理（`now_actor`, `_current_spot`）を持ち、トークン生成とビルダーの責務が混在
2. **命名の不明瞭**: `"spot_switch"`トークンは実際にはスポット切り替えの結果であり、名前が誤解を招く
3. **拡張性の制限**: 同一アクターのスポット移動（`set_spot`）がトークン化されておらず、設計原則に反する

### 設計方針
- **責務分離の徹底**: トークン生成層（act）は状態を持たず、ビルダー層（sakura_builder）が状態管理を担う
- **トークン設計の刷新**:
  - `set_spot()` → `{type="spot", actor=actor, spot=spot}` トークン生成
  - `talk()` → `{type="talk", actor=actor, text=text}` トークン生成（状態管理削除）
- **ビルダー責務の明確化**: ビルダーがtalkのactor切り替えを検出し、spotトークンでactor位置を追跡

### アーキテクチャ原則
- **基本単位**: 「アクターが発言する」（talk）
- **アクター**: 役者そのもの。舞台上のどこか（spot）に立っている
- **スポット**: 舞台の位置（番号）。アクターの属性の一つ
- **切り替わるもの**: アクター（その結果、スポット位置も変わりうる）
- **責務分離**: トークン生成（act）は状態レス、状態管理はビルダー

## Requirements

### Requirement 1: トークン生成の状態レス化
**Objective:** 開発者として、トークン生成層が状態を持たず、純粋にトークンを生成することで、テスタビリティと保守性を向上させたい。

#### Acceptance Criteria
1. When `act:set_spot(actor, spot)`が呼ばれたとき, the act module shall `{type="spot", actor=actor, spot=spot}`トークンを生成する
2. When `act:talk(actor, text)`が呼ばれたとき, the act module shall `{type="talk", actor=actor, text=text}`トークンを生成する
3. The act module shall `now_actor`および`_current_spot`などの状態管理フィールドを保持しない
4. The system shall トークン生成時にactor切り替え検出を行わない

### Requirement 2: ビルダーでの状態管理
**Objective:** 開発者として、ビルダー層がactor位置を追跡し、適切なスポットタグと段落区切り改行を出力したい。

#### Acceptance Criteria
1. When spotトークンがsakura_builderで処理されるとき, the sakura_builder shall 指定されたactorの現在spot位置を内部状態として記録する
2. When talkトークンがsakura_builderで処理されるとき, the sakura_builder shall talkのactorが前回のactorと異なる場合にスポットタグ（`\p[N]`）を出力する
3. When talkのactorが前回と異なり、かつspotも異なるとき, the sakura_builder shall 設定に基づいた段落区切り改行（`\n[N]`）をスポットタグの後に出力する
4. The sakura_builder shall `config.spot_newlines`（旧`spot_switch_newlines`）設定を使用して改行量を決定する

### Requirement 3: 設定プロパティ名の一貫性
**Objective:** 開発者として、設定プロパティ名がトークンの役割と一致していることで、APIの一貫性を保ちたい。

#### Acceptance Criteria
1. When BuildConfigが定義されるとき, the sakura_builder shall `spot_newlines`プロパティを使用する（旧`spot_switch_newlines`）
2. The sakura_builder shall `spot_newlines`のデフォルト値を1.5として維持する

**Note**: 後方互換性（旧プロパティ名のフォールバック）はOut of Scopeとし、破壊的変更として扱う。

### Requirement 4: テストの更新
**Objective:** 開発者として、リファクタリング後もテストが正しく機能し、変更が意図通りであることを検証したい。

#### Acceptance Criteria
1. When `act_test.lua`が実行されるとき, the test suite shall 新しいトークン構造（spot/talk）に対するテストをパスする
2. When `sakura_builder_test.lua`が実行されるとき, the test suite shall spotトークンの状態管理とtalk処理のテストをパスする
3. The test suite shall リファクタリング後もすべての既存シナリオをカバーする

### Requirement 5: spot設定のトークン化
**Objective:** 開発者として、`set_spot()`をトークン化することで、すべての操作がトークンベースで統一されるようにしたい。

#### Acceptance Criteria
1. When `act:set_spot(actor_name, spot_id)`が呼ばれたとき, the act module shall `{type="spot", actor=actor_name, spot=spot_id}`トークンを生成する
2. The act module shall `set_spot()`内で状態管理を行わず、トークン生成のみを行う
3. The system shall `set_spot()`の効果をビルダー層で反映する

### Requirement 6: clear_spot()のトークン化
**Objective:** 開発者として、`clear_spot()`もトークン化することで、act層の完全な状態レス化を実現したい。

#### Acceptance Criteria
1. When `act:clear_spot()`が呼ばれたとき, the act module shall `{type="clear_spot"}`トークンを生成する
2. The act module shall `clear_spot()`内で状態管理を行わず、トークン生成のみを行う
3. When clear_spotトークンがsakura_builderで処理されるとき, the sakura_builder shall 内部の`actor_spots`状態を空テーブル`{}`にリセットする
4. When clear_spotトークンがsakura_builderで処理されるとき, the sakura_builder shall `last_actor`状態を`nil`にリセットする

## Out of Scope
- spot以外の新しいトークンタイプの追加
- actorオブジェクトの構造変更

## Glossary
| 用語             | 定義                                                                                     |
| ---------------- | ---------------------------------------------------------------------------------------- |
| spot             | さくらスクリプトにおけるキャラクター表示位置（0=sakura, 1=kero, 2+=char2以降）           |
| actor            | 発話を行うキャラクターエンティティ（名前、spot位置などの属性を持つ）                     |
| トークン         | act.buildで返却される中間表現の単位                                                      |
| さくらスクリプト | 「伺か」で使用されるマークアップ言語                                                     |
| 状態レス         | 内部状態を持たず、入力に対して一意の出力を返す性質                                       |
| ビルダー         | トークン配列をさくらスクリプト文字列に変換する層（sakura_builder）。状態管理の責務を持つ |
