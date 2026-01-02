# 要件ドキュメント

## プロジェクト説明（入力）
pasta_luaに「SceneActorItem」アイテムへの対応を追加して欲しい。

ルール：「　＆さくら、うにゅう＝２」が入力だった時（さくら＝０，うにゅう＝２）
SCENE.__start__()メソッドの「PASTA.create_session」行の後ろに、下記行を追加する。
　　⇒「act.さくら:set_spot(0)」「act.うにゅう:set_spot(2)」

目的：アクターの立ち位置の番号を初期設定するため。

## 導入
本仕様は、pasta_coreの`scene-actors-ast-support`で実装された`SceneActorItem` ASTを受け取り、Luaコード生成時にアクターの立ち位置（spot番号）を初期設定するコードを追加する。

### 背景
- `GlobalSceneScope.actors: Vec<SceneActorItem>`はパース済み
- 各`SceneActorItem`は`name`（アクター名）、`number`（スポット番号）、`span`を保持
- C#のenum採番ルールで複数行をまたいでも連番計算済み

### 前提条件
- **scene-actors-ast-support完了済み**: AST側（pasta_core）の実装は完成
- **ASTの番号計算は完了**: `SceneActorItem.number`は最終的な値を保持

### スコープ
- **対象**: pasta_lua（コード生成）
- **対象外**: pasta_core（変更なし）

## 要件

### 要件1: set_spotコード生成
**目的:** pasta_lua開発者として、グローバルシーンの`__start__`関数にアクター立ち位置初期化コードを生成したい。これにより、シーン開始時にアクターの立ち位置が自動設定される。

#### 受け入れ基準
1. When `GlobalSceneScope.actors`が空でない時, pasta_lua shall `PASTA.create_session`行の直後に各アクターの`set_spot`呼び出しを生成する
2. The pasta_lua shall 各`SceneActorItem`に対して`act.<name>:set_spot(<number>)`形式のLuaコードを生成する
3. When `GlobalSceneScope.actors`が空の時, pasta_lua shall `set_spot`関連のコードを一切生成しない
4. The pasta_lua shall アクターの宣言順序を保持して`set_spot`呼び出しを生成する

### 要件2: 生成コード配置
**目的:** pasta_lua開発者として、`set_spot`コードを正しい位置に配置したい。これにより、セッション初期化後にアクター立ち位置が設定される。

#### 受け入れ基準
1. The pasta_lua shall `set_spot`呼び出しを`__start__`関数内でのみ生成する（他のローカルシーン関数では生成しない）
2. The pasta_lua shall `set_spot`呼び出しを`local act, save, var = PASTA.create_session(SCENE, ctx)`行の直後に配置する（前後に空行を1行ずつ挿入）
3. When 複数の`scene_actors_line`がグローバルシーン内に存在する時, pasta_lua shall すべてのアクターを1つの`__start__`内で集約初期化する

### 要件3: テスト
**目的:** 品質保証担当者として、新機能が正しく動作することを検証したい。これにより、リグレッションを防止できる。

#### 受け入れ基準
1. When `％さくら、うにゅう＝２`を含むPastaファイルをトランスパイルした時, pasta_lua shall `act.さくら:set_spot(0)`と`act.うにゅう:set_spot(2)`を生成する
2. When アクター宣言がない場合, pasta_lua shall `set_spot`呼び出しを含まないLuaコードを生成する
3. When `cargo test --all`を実行した時, すべてのテスト（既存＋新規）が成功する

**テスト設計方針:**
- パーサー側（pasta_core）で採番ルール検証済みのため、Lua側は「正しく受け取ったactorsをLua形式で出力する」ことのみ検証
- テストパターン: 単一アクター、複数アクター、アクターなし（各1～2ケース）
- 複数行継続のC#採番検証はpasta_coreの責務、Lua側では不要

### 要件4: 生成コード例
**目的:** 開発者として、期待される生成コードを理解したい。

#### 期待される出力
入力：
```pasta
＊シーン１
％さくら、うにゅう＝２
％まりか
・
　さくら：こんにちは
```

出力（`__start__`関数部分）：
```lua
function SCENE.__start__(ctx, ...)
    local args = { ... }
    local act, save, var = PASTA.create_session(SCENE, ctx)

    act.さくら:set_spot(0)
    act.うにゅう:set_spot(2)
    act.まりか:set_spot(3)

    act.さくら:talk("こんにちは")
end
```

※ 複数の`scene_actors_line`がある場合も、すべてのアクターは1つの`__start__`で集約初期化される
