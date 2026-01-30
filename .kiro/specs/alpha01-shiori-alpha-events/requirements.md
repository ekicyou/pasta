# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **SHIORI EVENT 7種のハンドラ登録・スタブ応答実装** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **現状**: EVENT/REG モジュール実装済み、基本的なイベントディスパッチ機構動作確認済み
- **目的**: ベースウェア（SSP等）からの SHIORI EVENT を受信し、適切にハンドリングする基盤を確立

### 既存実装

本仕様は既存の以下モジュールを活用・拡張する:

| モジュール | パス | 役割 |
|-----------|------|------|
| REG | `pasta.shiori.event.register` | ハンドラ登録テーブル（空テーブル） |
| EVENT | `pasta.shiori.event` | イベント振り分け、`fire(req)` API |
| RES | `pasta.shiori.res` | レスポンス組み立て（`ok`, `no_content`, `err`） |
| SHIORI | `pasta.shiori.entry` | エントリポイント（`load`, `request`, `unload`） |
| boot.lua | `pasta.shiori.event.boot` | OnBootデフォルトハンドラ（204 No Content） |

### 対象イベント

1. **OnFirstBoot** - ゴースト初回起動
2. **OnBoot** - ゴースト起動
3. **OnClose** - ゴースト終了
4. **OnGhostChanged** - ゴースト切り替え
5. **OnSecondChange** - 毎秒更新
6. **OnMinuteChange** - 毎分更新
7. **OnMouseDoubleClick** - ダブルクリック

---

## Requirements

### Requirement 1: イベントハンドラ登録機構

**Objective:** As a ゴースト開発者, I want SHIORI EVENT をハンドラ関数で受信したい, so that イベント駆動の対話ロジックを実装できる

#### Acceptance Criteria

1. The pasta.shiori.event.register shall ゴースト開発者が `REG.OnBoot = function(req) ... end` パターンでハンドラを登録できる API を提供する
2. The pasta.shiori.event shall 7種のイベント（OnFirstBoot, OnBoot, OnClose, OnGhostChanged, OnSecondChange, OnMinuteChange, OnMouseDoubleClick）すべてのハンドラ登録を受け付ける
3. When ゴースト開発者が REG テーブルにハンドラ関数を設定した場合, the pasta.shiori.event shall 既存のデフォルトハンドラを上書きしてディスパッチする

---

### Requirement 2: イベントディスパッチ機構

**Objective:** As a ゴースト開発者, I want 登録したハンドラが自動的に呼び出されることを期待する, so that イベント駆動のロジックが動作する

#### Acceptance Criteria

1. When ベースウェアから SHIORI EVENT リクエストが送信された場合, the pasta.shiori.event shall `req.id` に対応するハンドラ関数を呼び出す
2. If ハンドラが登録されていない場合, the pasta.shiori.event shall `204 No Content` レスポンスを返す
3. If ハンドラ実行中にエラーが発生した場合, the pasta.shiori.event shall エラーをキャッチし `500 Internal Server Error` 相当のレスポンスを返す
4. The pasta.shiori.event shall `req` テーブルを read-only 契約としてハンドラに渡す

---

### Requirement 3: Reference パラメータアクセス

**Objective:** As a ゴースト開発者, I want イベントのReference0〜Reference7を利用したい, so that イベント固有の情報を活用できる

#### Acceptance Criteria

1. The pasta.shiori.event shall `req.reference[0]` ～ `req.reference[7]` で Reference パラメータへのアクセスを提供する
2. The SHIORI EVENT ドキュメント shall 各イベントで使用される Reference の意味を明記する:
   - **OnFirstBoot**: Reference0 = バニッシュからの復帰フラグ ("0" or "1")
   - **OnBoot**: Reference0 = シェル名, Reference6 = シェルパス, Reference7 = ゴーストパス
   - **OnClose**: Reference0 = 終了理由 ("user", "shutdown" 等)
   - **OnGhostChanged**: Reference0 = 切り替え先ゴースト名, Reference1 = 切り替え元ゴースト名
   - **OnSecondChange**: Reference0 = 現在秒, Reference1 = 累積秒
   - **OnMinuteChange**: Reference0 = 現在分, Reference1 = 現在時
   - **OnMouseDoubleClick**: Reference0 = スコープ (0: sakura, 1: kero), Reference4 = 当たり判定 ID
3. If Reference パラメータが存在しない場合, the pasta.shiori.event shall `nil` を返す

---

### Requirement 4: デフォルトハンドラ実装

**Objective:** As a 開発者, I want 各イベントに対するデフォルト動作がほしい, so that 最小構成で動作確認ができる

#### Acceptance Criteria

1. The pasta.shiori.event.boot shall OnBoot イベントに対してデフォルトハンドラを提供し `204 No Content` を返す
2. The pasta.shiori.event shall OnFirstBoot, OnClose, OnGhostChanged, OnSecondChange, OnMinuteChange, OnMouseDoubleClick に対して未登録時は `204 No Content` を返す
3. The デフォルトハンドラ shall ゴースト開発者による上書きを許容する設計とする

---

### Requirement 5: スタブ応答サンプル

**Objective:** As a ゴースト開発者, I want 各イベントの応答サンプルコードがほしい, so that 実装の参考にできる

#### Acceptance Criteria

1. The ドキュメント shall 各イベントに対するさくらスクリプト応答のサンプルコードを提供する:
   - **OnFirstBoot**: `RES.ok([[\0\s[0]初めまして。\e]])`
   - **OnBoot**: `RES.ok([[\0\s[0]こんにちは。\e]])`
   - **OnClose**: `RES.ok([[\0\s[0]さようなら。\e]])`
   - **OnGhostChanged**: `RES.ok([[\0\s[0]いらっしゃい。\e]])`
   - **OnMouseDoubleClick**: `RES.ok([[\0\s[0]なに？\e]])`
   - **OnSecondChange/OnMinuteChange**: `RES.no_content()` （通常は空応答、alpha02 で仮想イベント発行に拡張）
2. The サンプルコード shall `pasta.shiori.res` モジュールの API を使用する
3. The サンプルコード shall `pasta.shiori.act` モジュール（alpha03）との将来統合を想定したコメントを含む

---

### Requirement 6: テスト要件

**Objective:** As a 開発者, I want イベントハンドリングのテストを実行したい, so that 実装の品質を保証できる

#### Acceptance Criteria

1. The alpha01-shiori-alpha-events shall 7種のイベント各々に対するハンドラ登録・呼び出しテストを提供する
2. The テスト shall Reference パラメータの解析が正しく動作することを検証する
3. The テスト shall 未登録イベントに対して `204 No Content` が返されることを検証する
4. The テスト shall シーン関数フォールバック機構（`＊OnBoot` 等のグローバルシーン検索）を検証する
5. The テスト shall ハンドラ内エラー発生時のエラーレスポンス生成を検証する
6. The テスト shall 既存の `shiori_event_test.rs` テストスイートを拡張して実装する

---

### Requirement 7: シーン関数によるイベントハンドリング

**Objective:** As a ゴースト開発者, I want Pastaスクリプトでイベントハンドラを定義したい, so that Luaコードを書かずにイベント駆動ロジックを実装できる

#### Acceptance Criteria

1. When REG テーブルにハンドラが登録されていない場合, the pasta.shiori.event shall グローバルシーン検索によるフォールバックを試みる
2. The pasta.shiori.event shall `＊{req.id}` パターンでグローバルシーンを検索する（例: `＊OnFirstBoot`, `＊OnBoot`）
3. If 対応するグローバルシーンが見つかった場合, the pasta.shiori.event shall そのシーン関数を実行してレスポンスを生成する
4. If グローバルシーンも見つからない場合, the pasta.shiori.event shall `204 No Content` を返す
5. The シーン関数 shall `act` オブジェクトを通じてさくらスクリプト生成を行える（alpha03 統合を想定）

---

### Requirement 8: ドキュメント要件

**Objective:** As a ゴースト開発者, I want SHIORI EVENT の使い方ドキュメントがほしい, so that イベント駆動開発を始められる

#### Acceptance Criteria

1. The ドキュメント shall 7種の SHIORI EVENT それぞれの目的・発火タイミング・Reference 構造を説明する
2. The ドキュメント shall ハンドラ登録パターン（REG テーブルへの代入）を説明する
3. The ドキュメント shall シーン関数フォールバック機構を説明する（`＊OnBoot` 等のグローバルシーン利用）
4. The ドキュメント shall RES モジュールを使ったレスポンス組み立て方法を説明する
5. The ドキュメント shall `LUA_API.md` の前方セクション（セクション2または3）に「SHIORI EVENT ハンドリング」として追記する

---

## Out of Scope

- **OnTalk/OnHour 仮想イベントの発行** - alpha02-virtual-event-dispatcher で実装
- **さくらスクリプト組み立てロジック** - alpha03-shiori-act-sakura で実装
- **複雑な会話ロジック・ランダムトーク** - サンプルゴースト（alpha04）で実装
- **OnMouseMove, OnSurfaceChange 等の他イベント** - 本仕様では7種に限定

---

## Dependencies

- **既存モジュール**: pasta.shiori.event, pasta.shiori.event.register, pasta.shiori.res（実装済み）
- **後続仕様**: alpha02（仮想イベント）、alpha03（さくらスクリプト）は本仕様のハンドラ機構を活用

---

## Glossary

| 用語 | 説明 |
|------|------|
| REG | `pasta.shiori.event.register` モジュール - イベントハンドラ登録 |
| EVENT | `pasta.shiori.event` モジュール - イベントディスパッチ |
| Reference | SHIORI EVENT の追加パラメータ（Reference0〜Reference7） |
| スコープ | 0=メインキャラ(sakura), 1=サブキャラ(kero) |
