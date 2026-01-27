# Gap Analysis: scene-search-integration

## 概要

本ドキュメントは `scene.lua` に `@pasta_search` モジュールを活用したシーン検索関数を追加する要件に対するギャップ分析を行う。

## 1. 現状調査

### 1.1 関連ファイル・モジュール

| ファイル/モジュール | 責務 | 状態 |
|------------------|------|------|
| `scripts/pasta/scene.lua` | Luaシーンレジストリ管理 | ✅ 存在 |
| `scripts/pasta/store.lua` | データストア（STORE.scenes） | ✅ 存在 |
| `src/search/mod.rs` | `@pasta_search` Rustモジュール | ✅ 存在 |
| `src/search/context.rs` | SearchContext UserData | ✅ 存在 |
| `src/runtime/mod.rs` | PastaLuaRuntime | ✅ 存在 |

### 1.2 既存コードパターン

#### scene.lua の現在のAPI

```lua
-- 既存関数
SCENE.register(global_name, local_name, scene_func)
SCENE.get(global_name, local_name) -> function|nil
SCENE.get_global_table(global_name) -> SceneTable|nil
SCENE.get_start(global_name) -> function|nil
SCENE.create_scene(base_name, local_name?, scene_func?) -> SceneTable
```

**特徴**:
- `STORE` モジュールのみ依存（循環参照回避）
- `WORD` モジュールを require
- `@` プレフィックスモジュール（Rust拡張）は未使用

#### @pasta_search の API（Rust側）

```lua
-- SearchContext UserData メソッド
SEARCH:search_scene(name, global_scene_name?) -> global_name, local_name | nil
SEARCH:search_word(name, global_scene_name?) -> string | nil
SEARCH:set_scene_selector(...)
SEARCH:set_word_selector(...)
```

**特徴**:
- `package.loaded["@pasta_search"]` に登録
- `PastaLuaRuntime::with_config()` で自動登録
- 初期化順序: `@pasta_search` 登録 → スクリプトロード

### 1.3 初期化順序の確認

`PastaLuaRuntime::from_loader_with_scene_dic()` の初期化順序:

1. `Self::with_config()` → `@pasta_search` 登録
2. `setup_package_path()` → Lua module resolution
3. `register_config_module()` → `@pasta_config`
4. `register_enc_module()` → `@enc`
5. `register_persistence_module()` → `@pasta_persistence`
6. `entry.lua` ロード
7. `register_finalize_scene()` → Rust binding
8. `scene_dic.lua` ロード → シーン登録

**重要**: `@pasta_search` は最初に登録されるため、**scene.lua から require 可能**

### 1.4 類似パターン（参考）

`scripts/pasta/save.lua` での `@pasta_persistence` 使用:

```lua
local persistence = require("@pasta_persistence")
local save = persistence.load()
return save
```

**パターン**:
- モジュールスコープで `require`
- 即座に使用（遅延ロードなし）

## 2. 要件実現可能性分析

### Requirement 1: シーン検索関数の追加

| 項目 | 評価 |
|------|------|
| 技術的実現性 | ✅ 容易 |
| 既存コード利用 | `SCENE.get()` を活用可能 |
| 新規実装量 | 関数1つ（約15行） |

**ギャップ**: なし - 既存パターンで実装可能

### Requirement 2: 検索結果からのシーン関数解決

| 項目 | 評価 |
|------|------|
| 技術的実現性 | ✅ 既に存在 |
| 既存コード利用 | `SCENE.get()` がそのまま使える |
| 新規実装量 | エイリアス追加のみ（または既存関数で十分） |

**ギャップ**: `SCENE.get()` で既に要件を満たしている

### Requirement 3: act:call() との統合パターン

| 項目 | 評価 |
|------|------|
| 技術的実現性 | ✅ 容易 |
| 統合方式 | `SCENE.search()` が関数を返すので直接呼び出し可能 |

**ギャップ**: なし - 設計通りに動作

### Requirement 3: @pasta_search モジュールのロード

| 項目 | 評価 |
|------|------|
| 技術的実現性 | ✅ 容易 |
| 実装方式 | 直接 require |

**ギャップ**: なし

**設計決定**:
- `@pasta_search` は `PastaLuaRuntime::with_config()` で必ず最初に登録される
- scene.lua ロード時には既に存在するため、直接 require で問題なし
- pcall は末尾再帰最適化を崩す懸念があるため使用しない
- テスト時も PastaLuaRuntime を使用する前提

### Requirement 5: エラーハンドリング

| 項目 | 評価 |
|------|------|
| 技術的実現性 | ✅ 容易 |
| 実装方式 | 引数検証 + nil 返却 |

**ギャップ**: なし

### Requirement 6: 既存 API との互換性

| 項目 | 評価 |
|------|------|
| 技術的実現性 | ✅ 容易 |
| リスク | 低（新規関数追加のみ） |

**ギャップ**: なし

### Requirement 7: イベント駆動シーン呼び出し

| 項目 | 評価 |
|------|------|
| 技術的実現性 | ✅ 容易 |
| 統合方式 | `SCENE.search(req.id, nil)` で検索可能 |

**ギャップ**: なし - `SCENE.search()` がグローバル検索をサポート

## 3. 実装アプローチオプション

### Option A: scene.lua への直接追加（推奨）

**概要**: 既存の `scene.lua` に `search()` 関数を追加

**変更内容**:
```lua
-- scene.lua への追加
local SEARCH = require("@pasta_search")

-- シーン検索結果メタテーブル（__callメタメソッド付き）
local scene_result_mt = {
    __call = function(self, ...)
        return self.func(...)
    end
}

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

**トレードオフ**:
- ✅ 最小変更（1ファイル、約20行）
- ✅ メタデータ（global_name, local_name）を保持
- ✅ `__call`メタメソッドで関数として直接呼び出し可能
- ✅ デバッグ情報が豊富
- ✅ シンプルで末尾再帰最適化を崩さない
- ❌ scene.lua が `@pasta_search` への依存を持つ（設計上許容）

### Option B: 新規ヘルパーモジュール作成

**概要**: `scripts/pasta/search_helper.lua` を作成し、scene.lua から委譲

**変更内容**:
- `search_helper.lua`: 検索ロジックを実装
- `scene.lua`: `SCENE.search = require("pasta.search_helper").search`

**トレードオフ**:
- ✅ 責務分離
- ✅ テスト容易性向上
- ❌ ファイル追加
- ❌ 追加の require チェーン

### Option C: act.lua への統合

**概要**: `act:call()` 内部で検索を行う `act:call_dynamic()` を追加

**変更内容**:
```lua
function ACT_IMPL.call_dynamic(self, name, opts, ...)
    local SCENE = require("pasta.scene")
    local scene_func = SCENE.search(name, self.current_scene and self.current_scene.__global_name__)
    if scene_func then
        scene_func(self, ...)
    end
end
```

**トレードオフ**:
- ✅ 使い勝手が良い（ワンステップ呼び出し）
- ❌ 要件スコープ外（act.lua への変更）
- ❌ Option A と並行して実装が必要

## 4. 実装複雑度とリスク

### 工数見積もり: **S (1-3日)**

| 項目 | 見積もり |
|------|---------|
| scene.lua 変更 | 0.5日 |
| テスト作成 | 1日 |
| ドキュメント | 0.5日 |

**根拠**:
- 既存パターンの単純拡張
- 依存モジュール（`@pasta_search`）は完成済み
- テストパターンも確立済み

### リスク評価: **Low**

| リスク項目 | 評価 | 対策 |
|-----------|------|------|
| 循環参照 | 低 | `pcall` で安全に処理 |
| 初期化順序 | 低 | `@pasta_search` は最初に登録される |
| 後方互換性 | 低 | 新規関数追加のみ |
| テスト失敗 | 低 | 既存テストパターンを流用 |

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

**Option A（scene.lua への直接追加）を推奨**

**理由**:
1. 最小変更で要件を満たす
2. 既存の `@pasta_persistence` 参照パターンと一貫性がある
3. テスト作成が容易

### 設計フェーズでの決定事項

1. **`pcall` パターン確定**: モジュールスコープでの安全な require
2. **戻り値の型**: シーン関数直接 vs `{global_name, local_name}` タプル
3. **エラーログ**: 検索失敗時のログ出力有無

### Research Needed（設計フェーズで調査）

| 項目 | 内容 |
|------|------|
| テストパターン | `@pasta_search` 未登録時のテスト方法 |
| イベント統合例 | `REG.OnBoot` での具体的な使用例実装 |

## 6. 要件-アセットマップ

| 要件 | 既存アセット | ギャップ | ステータス |
|------|------------|---------|-----------|
| Req 1: search() 追加 | scene.lua | 関数追加 | Missing |
| Req 2: resolve() | SCENE.get() | なし | Constraint（既存で対応可） |
| Req 3: act:call() 統合 | act.lua | なし | Constraint（設計済み） |
| Req 4: 遅延ロード | - | pcall パターン | Missing |
| Req 5: エラーハンドリング | - | 引数検証 | Missing |
| Req 6: 互換性 | scene.lua | なし | Constraint |
| Req 7: イベント駆動 | EVENT.fire | なし | Constraint（search()で対応） |

## 7. 結論

**実装可能性**: ✅ 高（既存パターンの単純拡張）

**推奨**: Option A（scene.lua への直接追加）で設計フェーズに進む

**キーポイント**:
- `@pasta_search` は既に完成・テスト済み
- `scene.lua` への変更は約15行
- 初期化順序の問題なし（`@pasta_search` は最初に登録）
- `pcall` パターンで安全な遅延ロードを実現
