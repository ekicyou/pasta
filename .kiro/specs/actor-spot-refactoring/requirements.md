# Requirements Document

## Project Description (Input)
act.tokenの"actor"と、"spot_switch"の役割が重複している。実際にはactorが変われば明示的にスポットが変わる。それとも、actorのスポット位置変更に対応させる予定かな？まず、"spot_switch"⇒"spot"に変更。また、actorとspotの関係を整理してリファクタリングする。

## Introduction
本仕様は、`pasta.act`モジュールのトークン表現を整理し、spotトークンを廃止してactorトークンにspot情報を内包させることで、より明確で拡張性のある設計へリファクタリングすることを目的とする。

### 現状の問題点
1. **概念の分断**: spotはアクターの属性であるのに、`"spot_switch"`という独立トークンで表現されている
2. **表現の冗長性**: `"actor"`トークンと`"spot_switch"`トークンの二重表現により、トークン列が冗長になる
3. **移動表現の曖昧さ**: 同一アクターが別のspotに移動するケースが表現しづらい

### 設計方針
- `"spot_switch"`トークンを廃止し、actorトークンにspot情報を内包する
- アクター切り替えまたはアクター移動を示すために、actorトークンは必要に応じて連続発行される
- spotはアクターの属性であり、独立した切り替えイベントとして扱わない

### アーキテクチャ原則
- **基本単位**: 「アクターが発言する」（talk）
- **アクター**: 役者そのもの。舞台上のどこか（spot）に立っている
- **スポット**: 舞台の位置（番号）。アクターの属性の一つ
- **切り替わるもの**: アクター（その結果、スポット位置も変わりうる）

## Requirements

### Requirement 1: トークン表現の統合
**Objective:** 開発者として、spotをアクターの属性として一体化したトークン表現により、コードの理解性と保守性を向上させたい。

#### Acceptance Criteria
1. When `act.lua`がアクターの切り替えまたは移動を検出したとき, the act module shall `type = "actor"`トークンにspot情報を含めて生成する
2. The system shall `"spot_switch"`タイプへの参照をすべて削除する
3. The system shall `"spot"`タイプのトークンを使用しない

### Requirement 2: actorトークンのspot反映
**Objective:** 開発者として、actorトークン内のspot情報に基づいてスポットタグと段落区切り改行を出力したい。

#### Acceptance Criteria
1. When actorトークンがsakura_builderで処理されるとき, the sakura_builder shall spot情報からスポットタグ（`\p[N]`）を出力する
2. When actorトークンのspotが前回のspotと異なるとき, the sakura_builder shall 設定に基づいた段落区切り改行（`\n[N]`）を出力する
3. The sakura_builder shall `config.spot_newlines`（旧`spot_switch_newlines`）設定を使用して改行量を決定する

### Requirement 3: 設定プロパティ名の一貫性
**Objective:** 開発者として、設定プロパティ名がトークンタイプ名と一致していることで、APIの一貫性を保ちたい。

#### Acceptance Criteria
1. When BuildConfigが定義されるとき, the sakura_builder shall `spot_newlines`プロパティを使用する（旧`spot_switch_newlines`）
2. The sakura_builder shall `spot_newlines`のデフォルト値を1.5として維持する

**Note**: 後方互換性（旧プロパティ名のフォールバック）はOut of Scopeとし、破壊的変更として扱う。

### Requirement 4: テストの更新
**Objective:** 開発者として、リファクタリング後もテストが正しく機能し、変更が意図通りであることを検証したい。

#### Acceptance Criteria
1. When `act_test.lua`が実行されるとき, the test suite shall `"spot"`トークンタイプに対するテストをパスする
2. When `sakura_builder_test.lua`が実行されるとき, the test suite shall `"spot"`トークンタイプの変換テストをパスする
3. The test suite shall リネーム後もすべての既存テストシナリオをカバーする

### Requirement 5: actorとspotの独立性
### Requirement 5: アクター切り替えと移動の表現
**Objective:** 開発者として、アクター切り替えとアクター移動の両方をactorトークンで表現できるようにしたい。

#### Acceptance Criteria
1. The act module shall アクターが切り替わったときにactorトークンを生成する
2. When 同一actorで連続してtalk()が呼ばれたとき and spotが変化していない場合, the act module shall actorトークンを再度生成しない
3. When 同一actorで連続してtalk()が呼ばれたとき and spotが変化した場合, the act module shall actorトークンを再度生成する
4. The system shall spotをアクターの属性として扱い、actorのspot属性からスポットIDを導出する

## Out of Scope
- spot以外の新しいトークンタイプの追加
- actorオブジェクトの構造変更

## Glossary
| 用語             | 定義                                                                           |
| ---------------- | ------------------------------------------------------------------------------ |
| spot             | さくらスクリプトにおけるキャラクター表示位置（0=sakura, 1=kero, 2+=char2以降） |
| actor            | 発話を行うキャラクターエンティティ（名前、spot位置などの属性を持つ）           |
| トークン         | act.buildで返却される中間表現の単位                                            |
| さくらスクリプト | 「伺か」で使用されるマークアップ言語                                           |
