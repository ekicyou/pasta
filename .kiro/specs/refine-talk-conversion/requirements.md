# Requirements Document

## Project Description (Input)
sakura_builder.lua'内のトーク→スクリプト変換処理を洗練化

「if inner_type == "talk" then」の処理で、escape_sakuraの呼び出しを廃止。代わりに、下記パターンを利用してトークをスクリプトに変換せよ

```lua
-- さくらスクリプト変換 API
local SAKURA_SCRIPT = require "@pasta_sakura_script"
local actor = { talk = { script_wait_default = 50 } }
local result = SAKURA_SCRIPT.talk_to_script(actor, "こんにちは。")
-- 結果: "こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[100]。"
```

## Introduction

本仕様は、`sakura_builder.lua`のトーク変換処理を洗練化し、既存の単純エスケープ処理（`escape_sakura`）を`@pasta_sakura_script`モジュールの`talk_to_script`関数に置き換えることを定義する。これにより、会話テキストに自然なウェイトタグ（`\_w[ms]`）が自動付与され、より自然な会話テンポを実現する。

## Requirements

### Requirement 1: escape_sakura呼び出しの置き換え
**Objective:** As a ゴースト開発者, I want トーク変換処理が自動ウェイト挿入を行う, so that 会話テンポが自然になる

#### Acceptance Criteria
1. When `inner_type == "talk"`のトークンが処理される, the sakura_builder shall `SAKURA_SCRIPT.talk_to_script`を呼び出してテキストをスクリプトに変換する
2. When talk_to_scriptが呼び出される, the sakura_builder shall 現在のactorオブジェクトをパラメーターとして渡す
3. The sakura_builder shall `escape_sakura`関数の呼び出しを廃止する

### Requirement 2: SAKURA_SCRIPTモジュールの読み込み
**Objective:** As a モジュール利用者, I want 必要なモジュールが適切にロードされる, so that talk_to_script関数が利用可能になる

#### Acceptance Criteria
1. The sakura_builder shall ファイル先頭で`@pasta_sakura_script`モジュールをrequireする
2. When モジュールがロードされる, the sakura_builder shall モジュール参照を`SAKURA_SCRIPT`変数に格納する

### Requirement 3: actor情報の受け渡し
**Objective:** As a キャラクター開発者, I want キャラクター固有のウェイト設定が適用される, so that 各キャラクターの個性を表現できる

#### Acceptance Criteria
1. When actorトークンが処理される, the sakura_builder shall そのactorオブジェクトを保持する
2. When talk_to_scriptを呼び出す, the sakura_builder shall 保持したactorオブジェクトを第1引数として渡す
3. If actorがnilの場合, the sakura_builder shall nilを渡し、talk_to_scriptがpasta.tomlのデフォルト設定を使用する

### Requirement 4: escape_sakura関数の削除
**Objective:** As a コード保守者, I want 不要なコードが削除される, so that コードベースがシンプルに保たれる

#### Acceptance Criteria
1. The sakura_builder shall `escape_sakura`ローカル関数の定義を削除する
2. The sakura_builder shall `escape_sakura`への参照を一切含まない

### Requirement 5: 既存動作との互換性
**Objective:** As a ゴースト開発者, I want 変換結果がさくらスクリプトとして有効である, so that 既存のゴーストが正常に動作し続ける

#### Acceptance Criteria
1. When テキストが変換される, the sakura_builder shall 有効なさくらスクリプト文字列を出力する
2. The sakura_builder shall `\e`（スクリプト終端タグ）を維持する
3. When 既存のさくらスクリプトタグが含まれる, the sakura_builder shall それらを破壊しない（talk_to_scriptがタグを保護する）

### Requirement 6: テスト期待値の更新
**Objective:** As a テスト保守者, I want テストが新しいウェイト挿入形式を検証する, so that 実装の正確性が保証される

#### Acceptance Criteria
1. When sakura_builder_test.luaを実行する, the test suite shall 新しいウェイト挿入形式の期待値を使用する
2. The test suite shall escape_sakuraベースの期待値を削除する
3. When 他のインテグレーションテストが失敗する, the test maintainer shall さくらスクリプト出力変更による期待値を修正する
