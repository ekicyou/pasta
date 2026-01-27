# Requirements Document

## Introduction

本仕様は `scene.lua` モジュールに `@pasta_search` を活用したシーン検索機能を追加することを目的とする。

### 背景

現在の `act:call()` は、トランスパイラ出力で `{global_name, local_name}` を直接渡す形式になっている（例: `act:call(SCENE.__global_name__, "グローバル単語呼び出し", {})`）。しかし、動的なシーン検索が必要な場面では、`@pasta_search` モジュールの前方一致検索機能を活用してシーン関数を取得する必要がある。

**主要な使用ケース**:
1. **トランスパイル出力での動的呼び出し**: ユーザー入力やランダム選択に基づくシーン遷移
2. **イベント駆動シーン呼び出し**: `REG.OnBoot` などの SHIORI イベントハンドラからのシーン実行

### 技術コンテキスト

| コンポーネント | 責務 |
|---------------|------|
| `@pasta_search` | Rust側シーン辞書への前方一致検索（`search_scene()`） |
| `pasta.scene` (scene.lua) | Luaシーンレジストリ、シーン関数管理 |
| `pasta.act` (act.lua) | `act:call()` でシーン関数を呼び出し |
| `pasta.shiori.event` | イベント振り分け（`EVENT.fire()`） |
| `pasta.shiori.event.register` | イベントハンドラ登録（`REG.OnBoot` など） |
| `STORE.scenes` | `{global_name: SceneTable}` 形式のシーン格納 |

### 関連仕様

- **完了仕様**: `.kiro/specs/completed/pasta_search_module/` - `@pasta_search` モジュール実装
- **参照ファイル**: 
  - `crates/pasta_lua/scripts/pasta/scene.lua` - 現在のシーン管理実装
  - `crates/pasta_lua/scripts/pasta/shiori/event/init.lua` - イベントシステム
  - `crates/pasta_lua/scripts/pasta/shiori/event/register.lua` - ハンドラ登録

## Requirements

### Requirement 1: シーン検索関数の追加
**Objective:** As a Lua開発者, I want `SCENE.search()` で `@pasta_search` を使ってシーン関数を取得できる, so that 動的シーン遷移が可能になる

#### Acceptance Criteria
1. The scene.lua モジュール shall `search(name, global_scene_name)` 関数を公開する
2. When `search(name, global_scene_name)` が呼び出された場合, the scene.lua shall `@pasta_search:search_scene(name, global_scene_name)` を呼び出して検索を実行する
3. When 検索結果が `(global_name, local_name)` として返された場合, the scene.lua shall 対応するシーン関数を `STORE.scenes[global_name][local_name]` から取得する
4. When シーン関数が取得できた場合, the scene.lua shall メタデータ付きテーブルを返す（`{global_name, local_name, func}` + `__call` メタメソッド）
5. When 検索結果が nil の場合, the scene.lua shall nil を返す
6. When シーン関数が見つからない場合（検索結果はあるがLua側に未登録）, the scene.lua shall nil を返す
7. The 返却されたテーブル shall `__call` メタメソッドにより関数として直接呼び出し可能であること

### Requirement 2: act:call() との統合パターン
**Objective:** As a トランスパイラ開発者, I want SCENE.search() を act:call() と組み合わせて使用できる, so that 動的シーン呼び出しが実現できる

**参照**: sample.generated.lua の `act:call(SCENE.__global_name__, "グローバル単語呼び出し", {}, ...)` 形式

**備考**: 
- 検索結果からのシーン関数取得は既存の `SCENE.get(global_name, local_name)` で対応可能
- `__call` メタメソッドにより、検索結果を関数として直接呼び出し可能

#### Acceptance Criteria
1. When `SCENE.search(name, global_scene_name)` が検索結果を返した場合, the 返されたテーブル shall `__call` メタメソッドにより直接呼び出し可能であること（`result(act, ...)` 形式）
2. The 検索結果テーブル shall `global_name`, `local_name`, `func` フィールドを含む
3. The search() shall 内部で `SCENE.get()` を使用してシーン関数を取得する
4. The search() shall 複数候補がある場合に `@pasta_search` のランダム選択を適用する

### Requirement 3: @pasta_search モジュールのロード
**Objective:** As a システム設計者, I want scene.lua が @pasta_search を適切に参照できる, so that シーン検索機能を実現できる

**技術詳細**: `@pasta_search` は PastaLuaRuntime 初期化時に必ず登録されるため、直接 require する

#### Acceptance Criteria
1. The scene.lua shall モジュールスコープで `local SEARCH = require("@pasta_search")` を使用してロードする
2. The @pasta_search モジュール shall scene.lua ロード前に PastaLuaRuntime によって登録済みであること
3. The 初期化順序 shall PastaLuaRuntime → @pasta_search 登録 → scene.lua ロード を保証する
4. If @pasta_search が未登録の場合, the require shall エラーを発生させる（テスト時は PastaLuaRuntime を使用する前提）

### Requirement 4: エラーハンドリング
**Objective:** As a Lua開発者, I want 検索エラー時に適切な動作が行われる, so that デバッグが容易

#### Acceptance Criteria
1. If `search()` の引数 `name` が nil または非文字列の場合, the scene.lua shall nil を返す
2. If `@pasta_search:search_scene()` がエラーを発生させた場合, the scene.lua shall そのエラーを上位に伝播させる
3. The scene.lua shall 検索失敗（候補なし）をエラーではなく nil として処理する

### Requirement 5: 既存 API との互換性
**Objective:** As a Lua開発者, I want 既存の SCENE API が引き続き動作する, so that 後方互換性が維持される

#### Acceptance Criteria
1. The 新規追加 `search()` 関数 shall 既存の `get()`, `register()`, `create_scene()` 等の動作に影響を与えない
2. The scene.lua shall 既存のモジュール構造を維持する
3. The STORE.scenes へのアクセスパターン shall 変更しない

### Requirement 6: イベント駆動シーン呼び出しのサポート
**Objective:** As a ゴースト開発者, I want SHIORI イベントハンドラからシーン検索・実行できる, so that イベント駆動のシナリオフローを実現できる

**背景**: `REG.OnBoot` などの SHIORI イベントハンドラ内で、イベント名（例: "OnBoot"）から対応するシーン関数を検索して実行するパターンをサポートする。

**備考**: CTX/ACT の初期化方法とトークン変換については、本仕様のスコープ外（既存機能を利用）

#### Acceptance Criteria
1. When イベントハンドラ内で `SCENE.search(req.id, nil)` を呼び出した場合（例: `SCENE.search("OnBoot", nil)`）, the scene.lua shall グローバルシーン検索を実行してマッチするシーン関数を返す
2. When イベント名にマッチするシーンが存在しない場合, the scene.lua shall nil を返し、イベントハンドラはデフォルト動作を実行可能であること
3. The SCENE.search() shall イベントハンドラと act:call() の両方のコンテキストで動作すること

## Implementation Guidance

### 推奨実装パターン

```lua
--- scene.lua への追加

local SEARCH = require("@pasta_search")

--- シーン検索結果メタテーブル（__callメタメソッド付き）
local scene_result_mt = {
    __call = function(self, ...)
        return self.func(...)
    end
}

--- シーン検索（@pasta_search 活用）
--- @param name string 検索プレフィックス
--- @param global_scene_name string|nil 親シーン名（nil でグローバル検索）
--- @return table|nil メタデータ付き検索結果テーブル、またはnil
function SCENE.search(name, global_scene_name)
    if type(name) ~= "string" then return nil end
    
    local global_name, local_name = SEARCH:search_scene(name, global_scene_name)
    if not global_name then return nil end
    
    local func = SCENE.get(global_name, local_name)
    if not func then return nil end
    
    return setmetatable({
        global_name = global_name,
        local_name = local_name,
        func = func,
    }, scene_result_mt)
end
```

### 使用例

#### パターン1: トランスパイル出力での動的呼び出し

```lua
-- act:call() と組み合わせた動的シーン呼び出し
local result = SCENE.search("選択肢", SCENE.__global_name__)
if result then
    -- メタデータアクセス（デバッグ用）
    print("Selected scene:", result.global_name, result.local_name)
    
    -- __callメタメソッドにより関数として直接呼び出し
    result(act, ...)
end
```

#### パターン2: イベントハンドラからのシーン呼び出し

```lua
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")
local SCENE = require("pasta.scene")

REG.OnBoot = function(req)
    -- OnBoot イベントに対応するシーンを検索
    local result = SCENE.search("OnBoot", nil)  -- グローバル検索
    
    if result then
        -- デバッグログ
        print("OnBoot scene found:", result.global_name)
        
        -- シーン実行（CTX/ACT初期化は既存パターンを使用、本仕様のスコープ外）
        local output = execute_scene(result)  -- 既存ヘルパー想定
        return RES.ok(output)
    else
        -- シーンが見つからない場合はデフォルト動作
        return RES.ok([[\0\s[0]起動しました\e]])
    end
end
```

### テスト観点

1. **正常系**: 存在するシーンの検索・取得
2. **フォールバック**: ローカル→グローバルの段階検索
3. **未登録シーン**: 検索結果はあるがLua側に未登録の場合
4. **nil 入力**: name が nil の場合
5. **イベント駆動**: `REG.OnBoot` からの呼び出し（前方一致検索）
