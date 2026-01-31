# Requirements Document

## Introduction

本仕様は、pasta.shiori.act モジュールに SHIORI リクエスト情報（req）へのアクセス手段を追加し、イベントディスパッチ処理において act オブジェクトを生成・活用する機能を定義する。

現状、シーン関数はリクエスト情報（イベントID、Reference配列、送信者情報など）に直接アクセスする手段を持たない。本仕様により、act.req を通じてリクエスト情報を参照可能とし、コンテキスト依存の応答生成を実現する。

## Background

### 現状の問題
- `EVENT.fire()` / `EVENT.no_entry()` でシーン関数を呼び出す際、`req` テーブルが渡されていない
- シーン関数内で Reference[0]（クリック座標）やイベント ID に応じた分岐ができない
- `pasta.shiori.act` の `new()` は actors のみを受け取り、リクエスト情報を保持しない

### 解決アプローチ
- `SHIORI_ACT.new(actors, req)` で req を受け取り、`act.req` フィールドに格納
- イベントディスパッチ処理で act を生成し、シーン関数に渡す仕組みを構築

## Requirements

### Requirement 1: act.req フィールドの追加
**Objective:** As a シーン開発者, I want act オブジェクトから SHIORI リクエスト情報にアクセスしたい, so that イベント ID や Reference に応じた動的な応答を実装できる

#### Acceptance Criteria
1. When `SHIORI_ACT.new(actors, req)` が呼び出された時, the `pasta.shiori.act` shall act オブジェクトに `req` フィールドを設定する
2. The `pasta.shiori.act` shall `act.req` を通じて元の req テーブルへの参照を提供する
3. The `pasta.shiori.act` shall req パラメータが省略された場合、`act.req` を nil として初期化する
4. The `pasta.shiori.act` shall `act.req` を読み取り専用として扱う（変更は推奨しない）

### Requirement 2: イベントディスパッチでの act 生成
**Objective:** As a イベントハンドラ開発者, I want イベントディスパッチ処理で act オブジェクトを受け取りたい, so that さくらスクリプト生成とリクエスト情報参照を統合できる

#### Acceptance Criteria
1. When イベントハンドラが呼び出される時, the `pasta.shiori.event` shall act オブジェクトを生成して渡す
2. When `EVENT.no_entry()` でシーン関数にフォールバックする時, the `pasta.shiori.event` shall 生成した act オブジェクトをシーン関数に渡す
3. The `pasta.shiori.event` shall act 生成時に `req` を `SHIORI_ACT.new(actors, req)` に渡す
4. If act 生成中にエラーが発生した場合, then the `pasta.shiori.event` shall エラーレスポンス（500）を返す

### Requirement 3: アクター辞書の取得
**Objective:** As a イベントディスパッチャ, I want act 生成時にアクター辞書を取得したい, so that シーン関数で定義されたアクターを利用できる

#### Acceptance Criteria
1. The `pasta.shiori.event` shall `pasta.store.actors` からアクター辞書を取得する
2. The `pasta.shiori.event` shall 取得したアクター辞書を `SHIORI_ACT.new(actors, req)` の第1引数に渡す
3. The `pasta.shiori.event` shall テスタビリティのため、act 生成時に actors を直接注入する設計を維持する（`SHIORI_ACT.new()` 内で STORE を直接参照しない）

## Technical Notes

### 影響範囲
- `crates/pasta_lua/scripts/pasta/shiori/act.lua` - `new()` シグネチャ変更、`req` フィールド追加
- `crates/pasta_lua/scripts/pasta/shiori/event/init.lua` - act 生成・引き渡しロジック追加
- 関連テストファイルの更新

### req テーブル構造（参考）
```lua
req = {
    id = "OnBoot",           -- イベント名
    method = "get",          -- HTTP風メソッド
    version = 30,            -- SHIORI/3.0
    charset = "UTF-8",       -- 文字セット
    sender = "SSP",          -- 送信者
    reference = { ... },     -- Reference配列（0始まり）
    dic = { ... },           -- 全ヘッダー辞書
    date = { unix = ... },   -- 日付情報（OnSecondChange等）
    status = "..."           -- ステータス情報
}
```

### 使用例（将来）
```lua
-- ハンドラでの利用
REG.OnMouseClick = function(req, act)
    local x = req.reference[0]
    act.女の子:talk("クリック座標: " .. x)
    return RES.ok(act:build())
end
実装後）
```lua
-- ハンドラでの利用（actのみを受け取る）
REG.OnMouseClick = function(act)
    local x = act.lk("今は" .. hour .. "時だよ")
    act:yield()
end
```
