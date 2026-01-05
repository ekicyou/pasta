# Requirements Document

## Introduction
本仕様は、pasta_luaトランスパイラーにおけるシーンアクター初期化処理の出力形式変更を定義する。具体的には、「％アクター名」構文によるシーン内アクター設定のLua出力を、より明示的なAPI呼び出し形式に変更する。

## Project Description (Input)
トランスパイラーの「　％さくら、うにゅう」に関する出力を、以下のように変更してほしい。

**現在の形式:**
```lua
function SCENE.__start__(ctx, ...)
    local args = { ... }
    local act, save, var = PASTA.create_session(SCENE, ctx)

    act.さくら:set_spot(0)
    act.うにゅう:set_spot(1)

    act:call("メイン1", "グローバル単語呼び出し", {}, table.unpack(args))
```

**新しい形式:**
```lua
function SCENE.__start__(ctx, ...)
    local args = { ... }
    act:clear_spot()
    act:set_spot("さくら", 0)
    act:set_spot("うにゅう", 1)
    local act, save, var = PASTA.create_session(SCENE, ctx)

    act:call("メイン1", "グローバル単語呼び出し", {}, table.unpack(args))
```

## Requirements

### Requirement 1: アクター初期化の出力位置変更
**Objective:** トランスパイラー開発者として、アクター初期化コードを`PASTA.create_session()`呼び出しの前に出力したい。これにより、セッション作成時にアクター設定が事前に確定していることを保証できる。

#### Acceptance Criteria
1. When シーン関数（`__start__`またはローカルシーン関数）を生成する場合、the Transpiler shall `local args = { ... }` の直後、`PASTA.create_session()` の直前にアクター初期化コードを出力する
2. The Transpiler shall `local act, save, var = PASTA.create_session(SCENE, ctx)` 呼び出しをアクター初期化コードの後に配置する

### Requirement 2: clear_spot()呼び出しの追加
**Objective:** ランタイム開発者として、アクター設定の前に既存設定をクリアしたい。これにより、シーン遷移時のアクター状態を明確にリセットできる。

#### Acceptance Criteria
1. When シーンにアクター設定（％構文）が存在する場合、the Transpiler shall アクター初期化ブロックの先頭に `act:clear_spot()` を出力する
2. If シーン内のアクター登録行（例：「　％さくら、うにゅう」）が一切存在しない場合、then the Transpiler shall `act:clear_spot()` 呼び出しを出力しない

### Requirement 3: set_spot()呼び出し形式の変更
**Objective:** ランタイム開発者として、`act:set_spot("アクター名", 位置番号)` 形式でアクター位置を設定したい。これにより、actテーブルのプロパティアクセスに依存せずアクターを参照できる。

#### Acceptance Criteria
1. When アクター設定（％アクター名）を処理する場合、the Transpiler shall `act:set_spot("アクター名", 位置番号)` 形式でコードを出力する
2. The Transpiler shall アクター名を文字列リテラルとして出力する（従来の `act.アクター名:set_spot(位置番号)` 形式ではない）
3. When 複数のアクターが設定される場合（例: ％さくら、うにゅう）、the Transpiler shall 各アクターに対して順番に `act:set_spot()` を出力し、位置番号は0から順に割り当てる

### Requirement 4: 生成コード構造の整合性
**Objective:** トランスパイラー開発者として、生成されるLuaコードが正しく動作することを保証したい。

#### Acceptance Criteria
1. The Transpiler shall シーン関数内で以下の順序でコードを出力する:
   - `local args = { ... }`
   - `act:clear_spot()` (アクター設定がある場合)
   - `act:set_spot("アクター名", 位置)` (各アクターに対して)
   - `local act, save, var = PASTA.create_session(SCENE, ctx)`
   - 以降のシーン処理コード
2. If アクター設定が存在しない場合、then the Transpiler shall 従来通り `local args` の直後に `PASTA.create_session()` を出力する

### Requirement 5: 既存機能との互換性
**Objective:** ユーザーとして、この変更が他のトランスパイル機能に影響しないことを確認したい。

#### Acceptance Criteria
1. The Transpiler shall アクター辞書定義（トップレベルの％構文）の出力に影響を与えない
2. The Transpiler shall シーン内の会話アクション（`act.アクター名:talk()`、`act.アクター名:word()`）の出力に影響を与えない
3. The Transpiler shall `act:call()` 形式のシーン呼び出し出力に影響を与えない
