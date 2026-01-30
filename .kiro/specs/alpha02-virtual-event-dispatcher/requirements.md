# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **OnTalk/OnHour 仮想イベントの条件判定・発行機構** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: alpha01-shiori-alpha-events（OnSecondChange イベントをトリガーとして使用）
- **目的**: 定期的なトークと時報機能を実現する仮想イベント発行機構の確立

### 対象仮想イベント

1. **OnTalk** - ランダムトーク発動（一定時間経過後）
2. **OnHour** - 時報発動（正時）

---

## Requirements

### Requirement 1: OnTalk 仮想イベント発行

**Objective:** As a ゴースト開発者, I want 一定時間経過後に自動でトークが発動してほしい, so that ゴーストが自発的に話しかけてくる

#### Acceptance Criteria

1. The alpha02-virtual-event-dispatcher shall OnSecondChange をトリガーとして OnTalk 発行判定を行う
2. The OnTalk 発行条件 shall 以下を全て満たす場合:
   - 非トーク中（`ctx.save.virtual_event.is_talking == false`）
   - 前回トークから設定秒数経過
   - 時報までの余裕時間が設定秒数以上
3. The alpha02-virtual-event-dispatcher shall トーク間隔設定を `pasta.toml` の `[ghost]` セクションから読み込む

---

### Requirement 2: OnHour 仮想イベント発行

**Objective:** As a ゴースト開発者, I want 正時に時報イベントを発動したい, so that 時間帯に応じた挨拶ができる

#### Acceptance Criteria

1. The alpha02-virtual-event-dispatcher shall OnSecondChange をトリガーとして OnHour 発行判定を行う
2. The OnHour 発行条件 shall 以下を全て満たす場合:
   - 正時を超過（分=0, 秒=0〜数秒以内）
   - 非トーク中
   - 前回時報から60分以上経過
3. The OnHour shall OnTalk よりも優先して発行される

---

### Requirement 3: 状態管理

**Objective:** As a ゴースト開発者, I want 仮想イベントの状態がセッションを跨いで保持されることを期待する, so that 再起動後も適切なタイミングでトークが発動する

#### Acceptance Criteria

1. The alpha02-virtual-event-dispatcher shall 以下の状態を `ctx.save` テーブルで管理する:
   - `ctx.save.virtual_event.last_talk_time` - 前回トーク発行時刻（unix timestamp）
   - `ctx.save.virtual_event.last_hour_time` - 前回時報発行時刻（unix timestamp）
   - `ctx.save.virtual_event.is_talking` - トーク中フラグ（boolean）
2. The 状態 shall `@pasta_persistence` モジュールにより自動永続化される（Drop時保存）

---

### Requirement 4: 時刻判定

**Objective:** As a 開発者, I want 正確な時刻情報を取得したい, so that 時報判定が正しく動作する

#### Acceptance Criteria

1. The alpha02-virtual-event-dispatcher shall Rust提供の `req.date` テーブルを使用して時刻判定を行う
2. The `req.date` shall 以下のフィールドを提供する:
   - `year`, `month`, `day` - 日付
   - `hour`, `min`, `sec` - 時刻
   - `weekday` - 曜日
3. The alpha02-virtual-event-dispatcher shall 時刻判定ロジックを OnSecondChange ハンドラ内で実行する

---

### Requirement 5: 設定読み込み

**Objective:** As a ゴースト開発者, I want トーク間隔等を設定ファイルで調整したい, so that ゴーストの個性を設定できる

#### Acceptance Criteria

1. The alpha02-virtual-event-dispatcher shall `pasta.toml` の `[ghost]` セクションから以下を読み込む:
   - `talk_interval_min` - トーク最小間隔（秒）
   - `talk_interval_max` - トーク最大間隔（秒）
   - `hour_margin` - 時報前マージン（秒）
2. If 設定が存在しない場合, then デフォルト値を使用する

---

### Requirement 6: テスト要件

**Objective:** As a 開発者, I want 仮想イベント発行のテストを実行したい, so that 実装の品質を保証できる

#### Acceptance Criteria

1. The alpha02-virtual-event-dispatcher shall OnTalk/OnHour 発行条件のユニットテストを提供する
2. The テスト shall 状態管理（ctx.save）の永続化を検証する
3. The テスト shall 設定読み込みのデフォルト値フォールバックを検証する

---

## Out of Scope

- 実際のトーク内容生成（alpha04 で実装）
- さくらスクリプト組み立て（alpha03 で実装）
- 複雑なトーク条件（天気、記念日等）

---

## Glossary

| 用語 | 説明 |
|------|------|
| OnTalk | ランダムトーク発動の仮想イベント |
| OnHour | 時報発動の仮想イベント |
| ctx.save | セッション永続化テーブル（`@pasta_persistence` モジュール経由） |
| pasta.toml | ゴースト設定ファイル |
| req.date | Rust側から提供される時刻情報テーブル |
