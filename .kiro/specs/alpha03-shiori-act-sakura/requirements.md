# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **pasta.shiori.act モジュールによるさくらスクリプト組み立て機能** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: なし（独立実装可能）
- **目的**: Pasta DSL から生成されたトーク内容を、伺かベースウェアで解釈可能なさくらスクリプト形式に変換する

### さくらスクリプトとは

伺かベースウェア（SSP等）で解釈される表示制御スクリプト。キャラクターの表情変更、テキスト表示、待機等を制御する。

主要タグ例:
- `\0` - メインキャラ（sakura）に切り替え
- `\1` - サブキャラ（kero）に切り替え
- `\s[n]` - サーフェス（表情）変更
- `\w[n]` - n ミリ秒待機
- `\e` - スクリプト終端

---

## Requirements

### Requirement 1: pasta.act 継承

**Objective:** As a 開発者, I want pasta.shiori.act が pasta.act を継承してほしい, so that 既存のactインターフェースと互換性がある

#### Acceptance Criteria

1. The alpha03-shiori-act-sakura shall `pasta.shiori.act` モジュールを提供する
2. The `pasta.shiori.act` shall `pasta.act` モジュールを継承（setmetatable）する
3. The 継承 shall `pasta.act` の全メソッド（`new`, `talk`, `jump`, `call` 等）を利用可能とする

---

### Requirement 2: さくらスクリプトタグ生成

**Objective:** As a ゴースト開発者, I want さくらスクリプトタグを生成するメソッドがほしい, so that 表情変更や待機を制御できる

#### Acceptance Criteria

1. The alpha03-shiori-act-sakura shall 以下のメソッドを提供する:
   - `act:sakura()` - `\0` タグ追加（メインキャラ切り替え）
   - `act:kero()` - `\1` タグ追加（サブキャラ切り替え）
   - `act:surface(id)` - `\s[id]` タグ追加（サーフェス変更）
   - `act:wait(ms)` - `\w[ms]` タグ追加（待機）
   - `act:newline()` - `\n` タグ追加（改行）
2. The 各メソッド shall メソッドチェーン可能（`return self`）とする

---

### Requirement 3: talk メソッドのオーバーライド

**Objective:** As a ゴースト開発者, I want talk() でテキストを追加したい, so that 会話内容を組み立てられる

#### Acceptance Criteria

1. The alpha03-shiori-act-sakura shall `act:talk(text)` メソッドをオーバーライドする
2. The `talk` shall テキストを内部バッファに追加する
3. The `talk` shall 特殊文字（バックスラッシュ等）を適切にエスケープする

---

### Requirement 4: build メソッド

**Objective:** As a 開発者, I want 組み立てたさくらスクリプトを文字列として取得したい, so that SHIORIレスポンスに含められる

#### Acceptance Criteria

1. The alpha03-shiori-act-sakura shall `act:build()` メソッドを提供する
2. The `build` shall 内部バッファの内容を連結し、末尾に `\e` を追加した文字列を返す
3. The `build` shall 呼び出し後も内部バッファをリセットしない（再利用可能）

---

### Requirement 5: ShioriAct クラス定義

**Objective:** As a 開発者, I want ShioriAct クラスを new で生成したい, so that 複数のactインスタンスを管理できる

#### Acceptance Criteria

1. The alpha03-shiori-act-sakura shall `ShioriAct:new()` コンストラクタを提供する
2. The `new` shall 空の内部バッファを持つ新規インスタンスを返す
3. The インスタンス shall メタテーブルにより `pasta.act` のメソッドを継承する

---

### Requirement 6: テスト要件

**Objective:** As a 開発者, I want さくらスクリプト生成のテストを実行したい, so that 実装の品質を保証できる

#### Acceptance Criteria

1. The alpha03-shiori-act-sakura shall 各メソッドのユニットテストを提供する
2. The テスト shall メソッドチェーンの動作を検証する
3. The テスト shall エスケープ処理の正確性を検証する
4. The テスト shall `\e` 終端の付与を検証する

---

## Out of Scope

- SHIORI EVENT ハンドラ（alpha01 で実装）
- 仮想イベント発行（alpha02 で実装）
- 高度なさくらスクリプト機能（バルーン制御、アニメーション等）

---

## Glossary

| 用語 | 説明 |
|------|------|
| pasta.act | Pasta DSL 実行時の基本 act クラス |
| pasta.shiori.act | pasta.act を継承した SHIORI 専用 act クラス |
| さくらスクリプト | 伺かベースウェアで解釈される表示制御スクリプト |
| サーフェス | キャラクターの表情画像（surface0, surface1, ...） |
| スコープ | 0=メインキャラ(sakura), 1=サブキャラ(kero) |
