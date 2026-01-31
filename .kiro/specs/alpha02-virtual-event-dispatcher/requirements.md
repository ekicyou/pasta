# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **OnTalk/OnHour 仮想イベントの条件判定・発行機構** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: alpha01-shiori-alpha-events（OnSecondChange イベントをトリガーとして使用）
- **目的**: 定期的なトークと時報機能を実現する仮想イベント発行機構の確立

### 既存実装

本仕様は既存の以下モジュールを活用・拡張する:

| モジュール | パス | 役割 |
|-----------|------|------|
| EVENT | `pasta.shiori.event` | イベント振り分け、`fire(req)` API |
| REG | `pasta.shiori.event.register` | ハンドラ登録テーブル |
| RES | `pasta.shiori.res` | レスポンス組み立て |
| CTX | `pasta.ctx` | 環境コンテキスト、`ctx.save` 永続化テーブル |
| SCENE | `pasta.scene` | シーン検索 |

### 対象仮想イベント

1. **OnTalk** - ランダムトーク発動（一定時間経過後）
2. **OnHour** - 時報発動（正時）

### アーキテクチャ概要

```
OnSecondChange (ベースウェア毎秒通知)
    ↓
pasta.shiori.event.fire(req)
    ↓
REG.OnSecondChange ハンドラ
    ↓
pasta.shiori.event.virtual_dispatcher モジュール
    ├─→ OnHour 判定（優先）
    │     ├─ 発行条件成立 → シーン関数呼び出し
    │     └─ 不成立 → 次へ
    └─→ OnTalk 判定
          ├─ 発行条件成立 → シーン関数呼び出し
          └─ 不成立 → 204 No Content
```

---

## Requirements

### Requirement 1: OnTalk 仮想イベント発行

**Objective:** As a ゴースト開発者, I want 一定時間経過後に自動でトークが発動してほしい, so that ゴーストが自発的に話しかけてくる

#### Acceptance Criteria

1. When OnSecondChange イベントを受信した場合, the virtual_dispatcher shall OnTalk 発行判定を実行する
2. While 以下の条件を全て満たす場合, the virtual_dispatcher shall OnTalk 仮想イベントを発行する:
   - 非トーク中（`ctx.save.virtual_event.is_talking == false`）
   - 前回トークから設定秒数が経過している
   - 次の正時までの余裕時間が `hour_margin` 秒以上ある
3. When OnTalk を発行する場合, the virtual_dispatcher shall `SCENE.search("OnTalk")` でシーン関数を検索し実行する
4. If OnTalk シーン関数が存在しない場合, the virtual_dispatcher shall `204 No Content` を返す
5. When OnTalk を発行した場合, the virtual_dispatcher shall `ctx.save.virtual_event.last_talk_time` を現在時刻で更新する

---

### Requirement 2: OnHour 仮想イベント発行

**Objective:** As a ゴースト開発者, I want 正時に時報イベントを発動したい, so that 時間帯に応じた挨拶ができる

#### Acceptance Criteria

1. When OnSecondChange イベントを受信した場合, the virtual_dispatcher shall OnTalk より先に OnHour 発行判定を実行する
2. While 以下の条件を全て満たす場合, the virtual_dispatcher shall OnHour 仮想イベントを発行する:
   - 正時を超過している（`req.date.min == 0` かつ `req.date.sec` が設定許容範囲内）
   - 非トーク中（`ctx.save.virtual_event.is_talking == false`）
   - 前回時報から60分以上経過している
3. When OnHour を発行する場合, the virtual_dispatcher shall `SCENE.search("OnHour")` でシーン関数を検索し実行する
4. If OnHour シーン関数が存在しない場合, the virtual_dispatcher shall `204 No Content` を返す
5. When OnHour を発行した場合, the virtual_dispatcher shall `ctx.save.virtual_event.last_hour_time` を現在時刻で更新する
6. The virtual_dispatcher shall OnHour を OnTalk より優先して判定する（OnHour 発行時は OnTalk 判定をスキップ）

---

### Requirement 3: 状態管理

**Objective:** As a ゴースト開発者, I want 仮想イベントの状態がセッションを跨いで保持されることを期待する, so that 再起動後も適切なタイミングでトークが発動する

#### Acceptance Criteria

1. The virtual_dispatcher shall 以下の状態を `ctx.save.virtual_event` テーブルで管理する:
   - `last_talk_time` - 前回トーク発行時刻（Unix timestamp、秒）
   - `last_hour_time` - 前回時報発行時刻（Unix timestamp、秒）
   - `is_talking` - トーク中フラグ（boolean）
2. When virtual_dispatcher が初回起動する場合（`ctx.save.virtual_event == nil`）, the virtual_dispatcher shall デフォルト値で初期化する:
   - `last_talk_time = 0`
   - `last_hour_time = 0`
   - `is_talking = false`
3. The `ctx.save` テーブル shall `@pasta_persistence` モジュールにより自動永続化される（Drop時保存）
4. When トーク・時報発行後のシーン実行が開始する場合, the virtual_dispatcher shall `is_talking = true` を設定する
5. When シーン実行が完了した場合, the シーン shall `is_talking = false` を設定する（alpha03 act モジュール統合時に実装）

---

### Requirement 4: 時刻判定

**Objective:** As a 開発者, I want 正確な時刻情報を取得したい, so that 時報判定が正しく動作する

#### Acceptance Criteria

1. The virtual_dispatcher shall Rust 側から提供される `req.date` テーブルを使用して時刻判定を行う
2. The Rust 側 shall OnSecondChange リクエストに以下のフィールドを持つ `req.date` テーブルを付与する（既存実装に準拠）:
   - `year` - 年（整数）
   - `month` - 月（1-12）
   - `day` - 日（1-31）
   - `hour` - 時（0-23）
   - `min` - 分（0-59）
   - `sec` - 秒（0-59）
   - `wday` - 曜日（0=日曜 〜 6=土曜）
   - `unix` - Unix timestamp（秒）
3. If `req.date` が存在しない場合, the virtual_dispatcher shall 現在時刻判定をスキップし `204 No Content` を返す

---

### Requirement 5: 設定読み込み

**Objective:** As a ゴースト開発者, I want トーク間隔等を設定ファイルで調整したい, so that ゴーストの個性を設定できる

#### Acceptance Criteria

1. The virtual_dispatcher shall `pasta.toml` の `[ghost]` セクションから以下の設定を読み込む:
   - `talk_interval_min` - トーク最小間隔（秒、デフォルト: 180）
   - `talk_interval_max` - トーク最大間隔（秒、デフォルト: 300）
   - `hour_margin` - 時報前マージン（秒、デフォルト: 30）
   - `hour_tolerance` - 時報許容遅延（秒、デフォルト: 5）
2. If 設定が存在しない場合, the virtual_dispatcher shall デフォルト値を使用する
3. When トーク間隔を決定する場合, the virtual_dispatcher shall `talk_interval_min` と `talk_interval_max` の間でランダムに選択する
4. The 設定読み込み shall 起動時に1回のみ実行し、結果をキャッシュする

---

### Requirement 6: OnSecondChange ハンドラ統合

**Objective:** As a 開発者, I want 仮想イベントディスパッチャが OnSecondChange ハンドラから呼び出されることを期待する, so that 既存のイベント機構と統合できる

#### Acceptance Criteria

1. The virtual_dispatcher shall `pasta.shiori.event.virtual_dispatcher` モジュールとして実装する
2. The virtual_dispatcher shall `dispatch(req)` 関数を公開 API として提供する
3. When REG.OnSecondChange ハンドラを登録する場合, the デフォルトハンドラ shall `virtual_dispatcher.dispatch(req)` を呼び出す
4. The `dispatch(req)` 関数 shall 以下の戻り値を返す:
   - 仮想イベント発行成功: シーン関数の戻り値（さくらスクリプトまたは nil）
   - 仮想イベント発行なし: `nil`（呼び出し元で `204 No Content` に変換）
5. The virtual_dispatcher shall ゴースト開発者による `REG.OnSecondChange` 上書きを許容する

---

### Requirement 7: テスト要件

**Objective:** As a 開発者, I want 仮想イベント発行のテストを実行したい, so that 実装の品質を保証できる

#### Acceptance Criteria

1. The alpha02-virtual-event-dispatcher shall `virtual_event_dispatcher_test.rs` または Lua ユニットテストで以下を検証する:
   - OnTalk 発行条件判定ロジック
   - OnHour 発行条件判定ロジック
   - OnHour の OnTalk に対する優先度
2. The テスト shall 状態管理（`ctx.save.virtual_event`）の初期化と更新を検証する
3. The テスト shall 設定読み込みのデフォルト値フォールバックを検証する
4. The テスト shall `req.date` テーブル不在時のエラーハンドリングを検証する
5. The テスト shall 既存の `shiori_event_test.rs` テストスイートと整合性を保つ

---

### Requirement 8: ドキュメント要件

**Objective:** As a ゴースト開発者, I want 仮想イベントの使い方ドキュメントがほしい, so that ランダムトーク・時報機能を実装できる

#### Acceptance Criteria

1. The ドキュメント shall OnTalk/OnHour 仮想イベントの発行条件・タイミングを説明する
2. The ドキュメント shall `pasta.toml` の `[ghost]` セクション設定項目を説明する
3. The ドキュメント shall シーン関数でのハンドラ実装例を提供する:
   - `＊OnTalk` グローバルシーンの定義例
   - `＊OnHour` グローバルシーンの定義例
4. The ドキュメント shall `ctx.save.virtual_event` 状態テーブルの構造を説明する

---

## Out of Scope

- 実際のトーク内容生成（alpha04 で実装）
- さくらスクリプト組み立て・act オブジェクト統合（alpha03 で実装）
- 複雑なトーク条件（天気、記念日等）
- `is_talking` フラグの自動解除（alpha03 act モジュール統合時に実装）

---

## Glossary

| 用語 | 説明 |
|------|------|
| OnTalk | ランダムトーク発動の仮想イベント |
| OnHour | 時報発動の仮想イベント |
| 仮想イベント | OnSecondChange をトリガーとして条件判定により発行されるイベント |
| virtual_dispatcher | 仮想イベントの条件判定・発行を行うモジュール |
| ctx.save | セッション永続化テーブル（`@pasta_persistence` モジュール経由） |
| pasta.toml | ゴースト設定ファイル |
| req.date | Rust側から提供される時刻情報テーブル（`unix`, `year`, `month`, `day`, `hour`, `min`, `sec`, `wday` 等） |
| Unix timestamp | 1970年1月1日からの経過秒数（`req.date.unix`） |

---

## Dependencies

### 前提仕様

| 仕様 | 状態 | 必要機能 |
|------|------|----------|
| alpha01-shiori-alpha-events | 実装中 | OnSecondChange ハンドラ登録、`req` テーブル構造 |

### 後続仕様

| 仕様 | 依存内容 |
|------|----------|
| alpha03 | act オブジェクト統合、さくらスクリプト生成、`is_talking` 自動管理 |
| alpha04 | 実際のトーク内容生成、シーン選択ロジック |
