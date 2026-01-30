# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **SHIORI EVENT 7種のハンドラ登録・スタブ応答実装** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **現状**: EVENT/REG モジュール実装済み、基本的なイベントディスパッチ機構動作確認済み
- **目的**: ベースウェア（SSP等）からの SHIORI EVENT を受信し、適切にハンドリングする基盤を確立

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

1. The alpha01-shiori-alpha-events shall 既存の `REG` モジュール（`pasta.shiori.event.register`）を活用してイベントハンドラを登録する
2. The イベントハンドラ shall `REG.OnBoot = function(req) ... end` パターンで登録可能とする
3. The alpha01-shiori-alpha-events shall 7種のイベント全てに対してハンドラ登録スロットを提供する

---

### Requirement 2: イベントディスパッチ機構

**Objective:** As a ゴースト開発者, I want 登録したハンドラが自動的に呼び出されることを期待する, so that イベント駆動のロジックが動作する

#### Acceptance Criteria

1. The alpha01-shiori-alpha-events shall 既存の `EVENT` モジュール（`pasta.shiori.event`）を活用してイベントをディスパッチする
2. When ベースウェアから SHIORI EVENT が送信された場合, then 対応するハンドラ関数が呼び出される
3. If ハンドラが登録されていない場合, then 空レスポンス（`204 No Content`相当）を返す

---

### Requirement 3: Reference パラメータ解析

**Objective:** As a ゴースト開発者, I want イベントのReference0〜Reference7を解析したい, so that イベント固有の情報を活用できる

#### Acceptance Criteria

1. The alpha01-shiori-alpha-events shall `req.Reference0` ～ `req.Reference7` でReferenceパラメータにアクセス可能とする
2. The alpha01-shiori-alpha-events shall 各イベントで使用されるReferenceの意味をドキュメント化する:
   - OnFirstBoot: Reference0 = バニッシュからの復帰フラグ
   - OnBoot: Reference0 = シェル名, Reference6 = シェルパス, Reference7 = ゴーストパス
   - OnClose: Reference0 = 終了理由
   - OnGhostChanged: Reference0 = 切り替え先ゴースト名
   - OnMouseDoubleClick: Reference0 = スコープ(0/1), Reference4 = 当たり判定ID

---

### Requirement 4: スタブ応答実装

**Objective:** As a 開発者, I want 各イベントに対する最低限のスタブ応答がほしい, so that 動作確認ができる

#### Acceptance Criteria

1. The alpha01-shiori-alpha-events shall 各イベントに対してデフォルトスタブ応答を提供する:
   - OnFirstBoot: `\\0初めまして。\\e` スタイルの応答例
   - OnBoot: `\\0こんにちは。\\e` スタイルの応答例
   - OnClose: `\\0さようなら。\\e` スタイルの応答例
   - OnMouseDoubleClick: `\\0なに？\\e` スタイルの応答例
   - OnSecondChange/OnMinuteChange: 空応答（他仕様で拡張）
2. The スタブ応答 shall `pasta.shiori.act` モジュール（alpha03）との統合を想定した設計とする

---

### Requirement 5: テスト要件

**Objective:** As a 開発者, I want イベントハンドリングのテストを実行したい, so that 実装の品質を保証できる

#### Acceptance Criteria

1. The alpha01-shiori-alpha-events shall 各イベントのハンドラ呼び出しをテストする統合テストを提供する
2. The テスト shall Reference パラメータの解析が正しく動作することを検証する
3. The テスト shall 未登録イベントへの空応答を検証する

---

## Out of Scope

- OnTalk/OnHour 仮想イベントの発行（alpha02 で実装）
- さくらスクリプト組み立てロジック（alpha03 で実装）
- 複雑な会話ロジック・ランダムトーク

---

## Glossary

| 用語 | 説明 |
|------|------|
| REG | `pasta.shiori.event.register` モジュール - イベントハンドラ登録 |
| EVENT | `pasta.shiori.event` モジュール - イベントディスパッチ |
| Reference | SHIORI EVENT の追加パラメータ（Reference0〜Reference7） |
| スコープ | 0=メインキャラ(sakura), 1=サブキャラ(kero) |
