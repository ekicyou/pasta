# Requirements Document

## Introduction

本仕様は、OnHour仮想イベント発行時にSHIORIリクエストの日時情報（`req.date.XXX`）をアクションローカル変数（`act.var.XXX`）へ自動転記する機能を定義する。これにより、シーン関数から日時情報に容易にアクセス可能となる。

**スコープ**: 
- Lua ランタイム層（SHIORI_ACT, virtual_dispatcher.lua）
- SHIORI_ACT に日時転記メソッドを追加（将来的な再利用のため）
- 日本語変数名マッピング（`var.時`, `var.分` 等）の実装を含む
- execute_scene への act 引き渡し修正を含む（act-req-parameter 実装ミス修正）

## Requirements

### Requirement 1: SHIORI_ACT への日時転記メソッド追加

**Objective:** As a システム開発者, I want SHIORI_ACT に日時転記メソッドがある, so that 将来的に他のイベントでも再利用できる

#### Acceptance Criteria

1. The SHIORI_ACT shall `transfer_date_to_var()` メソッドを提供する
2. When `transfer_date_to_var()` が呼び出される, the SHIORI_ACT shall 以下の処理を行う:
   - `self.req.date` の全キー・値ペアを `self.var` へそのまま転記する（英語フィールド名）
   - 以下の日本語変数マッピングを追加で設定する:
     - `var.年` ← `req.date.year`
     - `var.月` ← `req.date.month`
     - `var.日` ← `req.date.day`
     - `var.時` ← `req.date.hour`
     - `var.分` ← `req.date.min`
     - `var.秒` ← `req.date.sec`
     - `var.年内通算日` ← `req.date.yday`
     - `var.曜日` ← `req.date.wday`
   - Unix timestamp (`unix`) および ナノ秒 (`ns`) は転記しない
   - エイリアスフィールド (`ordinal`, `num_days_from_sunday`) は転記しない
3. If `self.req` または `self.req.date` が存在しない, then the SHIORI_ACT shall 何もせずに正常終了する
4. The SHIORI_ACT shall 英語フィールド名と日本語変数名の両方で同じ値にアクセス可能にする

### Requirement 2: OnHour イベント発火時の日時変数自動設定

**Objective:** As a ゴースト開発者, I want OnHour発火時に日時情報がact.varに自動設定される, so that シーン関数から日時フィールドにアクセスできる

#### Acceptance Criteria

1. When OnHour仮想イベントが発火する, the virtual_dispatcher shall `act:transfer_date_to_var()` を呼び出す
2. When シーン関数が実行される, the scene function shall `act.var.hour`, `act.var.minute` 等で転記された日時フィールドにアクセスできる

### Requirement 3: execute_scene への act 引き渡し修正

**Objective:** As a システム開発者, I want execute_scene がシーン関数に act を正しく渡す, so that シーン関数が act.var にアクセスできる

#### Acceptance Criteria

1. When execute_scene がシーン関数を呼び出す, the virtual_dispatcher shall `pcall(scene_fn, act)` で act を引数として渡す
2. When テスト用 scene_executor が設定されている, the virtual_dispatcher shall `scene_executor(event_name, act)` で act を引数として渡す
3. The virtual_dispatcher shall check_hour および check_talk から execute_scene を呼び出す際に act を渡す

### Requirement 4: 既存動作との互換性

**Objective:** As a システム管理者, I want 既存のOnHour/OnTalk処理に影響がない, so that 既存ゴーストが正常に動作し続ける

#### Acceptance Criteria

1. When OnHour以外のイベント（OnTalk, OnBoot等）が発生する, the virtual_dispatcher shall `act:transfer_date_to_var()` を呼び出さない
2. While 日時転記処理が失敗した場合, the virtual_dispatcher shall シーン実行は継続し、エラーをログ出力する
3. The virtual_dispatcher shall 既存のOnHour判定ロジック（正時検出、優先度制御）を変更しない
