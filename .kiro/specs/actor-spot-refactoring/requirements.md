# Requirements Document

## Project Description (Input)
act.tokenの"actor"と、"spot_switch"の役割が重複している。実際にはactorが変われば明示的にスポットが変わる。それとも、actorのスポット位置変更に対応させる予定かな？まず、"spot_switch"⇒"spot"に変更。また、actorとspotの関係を整理してリファクタリングする。

## Introduction
本仕様は、`pasta.act`モジュールにおけるトークンタイプ`"spot_switch"`の命名を改善し、より明確で拡張性のある設計へリファクタリングすることを目的とする。

### 現状の問題点
1. **命名の不明瞭**: `"spot_switch"`という名前はアクション（切り替え）を示唆するが、実際には「アクター切り替えに伴うスポット位置変更」を表すべき
2. **概念の混乱**: spotは「スポット切り替え」という独立イベントではなく、アクターの属性（舞台上の位置）である
3. **拡張性の制限**: 現行設計では「actorは変わらないがspotだけ変える」ユースケースに対応しにくい（将来の拡張として検討）

### 設計方針
- `"spot_switch"` → `"spot"` にリネームし、トークンとしての意味を「スポット位置情報」に明確化
- アクター切り替え時、スポット位置が変わった場合にspotトークンを挿入する現行動作を維持
- spotはアクターの属性であり、独立した切り替えイベントではないことを明確化

### アーキテクチャ原則
- **基本単位**: 「アクターが発言する」（talk）
- **アクター**: 役者そのもの。舞台上のどこか（spot）に立っている
- **スポット**: 舞台の位置（番号）。アクターの属性の一つ
- **切り替わるもの**: アクター（その結果、スポット位置も変わりうる）

## Requirements

### Requirement 1: トークンタイプ名変更
**Objective:** 開発者として、トークンタイプ名がその意味を正確に反映していることで、コードの理解性を向上させたい。

#### Acceptance Criteria
1. When `sakura_builder.lua`がトークン配列を処理するとき, the sakura_builder shall `"spot"`タイプのトークンを認識する（旧`"spot_switch"`）
2. When `act.lua`がスポット切り替えを検出したとき, the act module shall `type = "spot"`のトークンを生成する（旧`type = "spot_switch"`）
3. The system shall `"spot_switch"`タイプへの参照をすべて`"spot"`に置き換える

### Requirement 2: spotトークンの責務明確化
**Objective:** 開発者として、spotトークンが「アクター切り替えに伴うスポット位置変更を示すマーカー」であることを明確にし、適切な段落区切り改行を出力したい。

#### Acceptance Criteria
1. When spotトークンがsakura_builderで処理されるとき, the sakura_builder shall 設定に基づいた段落区切り改行（`\n[N]`）を出力する
2. The sakura_builder shall `config.spot_newlines`（旧`spot_switch_newlines`）設定を使用して改行量を決定する
3. While actorトークンが処理されているとき, the sakura_builder shall スポットタグ（`\p[N]`）のみを出力し、改行は出力しない

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
### Requirement 5: actorとspotの関係明確化
**Objective:** 開発者として、spotがアクターの属性であり、アクター切り替え時に適切にspotトークンが生成されることを保証したい。

#### Acceptance Criteria
1. The act module shall アクターが切り替わったときにactorトークンを生成する
2. When 同一actorで連続してtalk()が呼ばれたとき, the act module shall actorトークンを再度生成しない
3. When 異なるspotを持つactorに切り替わったとき, the act module shall actorトークンの直後にspotトークンを生成する
4. The system shall spotをアクターの属性として扱い、actorのspot属性からスポットIDを導出する

## Out of Scope
- actorを変えずにspotだけを変更するAPIの追加（将来の拡張として検討）
- spot以外の新しいトークンタイプの追加
- actorオブジェクトの構造変更

## Glossary
| 用語             | 定義                                                                           |
| ---------------- | ------------------------------------------------------------------------------ |
| spot             | さくらスクリプトにおけるキャラクター表示位置（0=sakura, 1=kero, 2+=char2以降） |
| actor            | 発話を行うキャラクターエンティティ（名前、spot位置などの属性を持つ）           |
| トークン         | act.buildで返却される中間表現の単位                                            |
| さくらスクリプト | 「伺か」で使用されるマークアップ言語                                           |
