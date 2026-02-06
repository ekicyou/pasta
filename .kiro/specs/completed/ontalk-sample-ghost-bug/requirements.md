# Requirements Document

## Introduction

サンプルゴーストにおいてOnTalkイベント（定期会話）が発火しなくなる不具合の修正と、サンプルゴースト向け発動間隔の調整を行う。

### 調査結果サマリ

**根本原因**: `SceneTable::resolve_scene_id()` のキャッシュ消費モデルが「使い切り」設計になっており、全候補を一巡すると `NoMoreScenes` エラーを永久に返し続ける。サンプルゴーストには6つの `＊OnTalk` シーンがあるため、6回発火後に永久沈黙する。

**影響範囲**: OnTalkに限らず、重複シーン（同名で複数定義）を持つすべてのシーン検索に同じ問題が潜在する。

**発動間隔**: 現在のデフォルト値は `talk_interval_min=180` / `talk_interval_max=300`（3〜5分）。サンプルゴーストとしてはデモ性を考慮し60〜90秒が適切。

## Requirements

### Requirement 1: シーンキャッシュの循環リセット

**Objective:** As a ゴースト開発者, I want 重複シーンが全候補を一巡した後も再び選択可能になること, so that OnTalkなどの定期イベントが永久に沈黙しない。

#### Acceptance Criteria
1. When SceneTable が全候補を一巡した場合, the SceneTable shall 候補インデックスを0にリセットして再選択可能な状態に復帰する
2. When キャッシュリセットが発生した場合, the SceneTable shall シャッフルが有効ならば候補リストを再シャッフルしてから循環を開始する
3. The SceneTable shall リセット後も既存の候補リスト（シーンID群）を保持し、再登録なしに循環選択を継続する

### Requirement 2: サンプルゴーストのOnTalk発動間隔調整

**Objective:** As a サンプルゴーストのユーザー, I want OnTalkの発動間隔が30〜60秒であること, so that デモ時に素早く確認可能な頻度で会話が発生する。

#### Acceptance Criteria
1. When サンプルゴーストの設定ファイルが読み込まれた場合, the pasta_lua shall `talk_interval_min` を30（秒）として認識する
2. When サンプルゴーストの設定ファイルが読み込まれた場合, the pasta_lua shall `talk_interval_max` を60（秒）として認識する
3. The pasta_lua shall デフォルトの `talk_interval_min` / `talk_interval_max` 値はライブラリ側のデフォルト値（180/300）を維持し、サンプルゴースト固有の設定ファイルでのみ上書きする

### Requirement 3: OnTalk発火の継続性テスト

**Objective:** As a 開発者, I want OnTalkが長時間にわたり継続的に発火することをテストで検証できること, so that リグレッションを防止できる。

#### Acceptance Criteria
1. The テストスイート shall シーン候補数を超える回数の `resolve_scene_id` 呼び出しが成功することを検証するテストケースを含む
2. When 全候補を一巡してリセットが発生した場合, the テスト shall リセット後の選択結果が有効なシーンIDであることを検証する
