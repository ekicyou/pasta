# Implementation Gap Analysis

## 分析サマリー

- **スコープ**: pasta.toml の `[actor.*]` セクション定義とSTORE.actors自動初期化、ライブラリモジュール早期公開順序最適化
- **主要課題**: 
  - 既存の `@pasta_config` パイプラインは完全に機能しており、`[actor.*]` セクションは自動的に `CONFIG.actor` として公開される
  - Lua側（pasta.store）でCONFIG.actorからSTORE.actorsへの初期化ロジック追加が必要
  - メタテーブル設定パターンは既存のACTOR.get_or_create()に存在
  - ライブラリ公開順序は既に最適（`@pasta_config`最優先）で、実装不要
- **推奨アプローチ**: **Option A: 既存コンポーネント拡張** - pasta.store.luaとpasta.actor.luaに最小限の変更で実装可能

---

## 1. 現状調査

### 1.1 主要コンポーネント

#### Rust側（pasta_lua クレート）

| ファイル                | 責務                | 関連機能                                                           |
| ----------------------- | ------------------- | ------------------------------------------------------------------ |
| `src/loader/config.rs`  | pasta.toml パース   | `PastaConfig::parse()` で全TOMLセクションを `custom_fields` に格納 |
| `src/loader/context.rs` | Loader コンテキスト | `LoaderContext.custom_fields` を保持・伝搬                         |
| `src/runtime/mod.rs`    | Lua VM 初期化       | `register_config_module()` で `@pasta_config` 登録                 |
| `src/runtime/mod.rs`    | モジュール登録順序  | `from_loader_with_scene_dic()` で各モジュール登録                  |

#### Lua側（scripts/pasta/）

| ファイル    | 責務             | 関連機能                                           |
| ----------- | ---------------- | -------------------------------------------------- |
| `store.lua` | グローバルストア | `STORE.actors = {}` 初期化、`STORE.reset()`        |
| `actor.lua` | アクター管理     | `ACTOR.get_or_create()`, `ACTOR_IMPL` メタテーブル |

### 1.2 既存パターンと規約

#### TOML → Lua パイプライン

```
pasta.toml (全セクション)
  ↓ PastaConfig::parse() (Rust)
  ↓ custom_fields: toml::Table ([loader]以外)
  ↓ LoaderContext::from_config() (Rust)
  ↓ register_config_module() (Rust)
  ↓ @pasta_config (Lua)
```

**重要**: `[loader]` セクション以外の**全てのセクション**が自動的に `custom_fields` に含まれる。`[actor.*]` も例外なく含まれる。

#### 現在のモジュール登録順序（from_loader_with_scene_dic）

```rust
// Line 535-570 (simplified)
1. setup_package_path()              // Lua module resolution
2. register_config_module()          // @pasta_config ← 最初に登録済み
3. register_enc_module()             // @enc
4. register_persistence_module()     // @pasta_persistence
5. Load entry.lua (optional)         // SHIORI integration
6. register_finalize_scene()         // finalize stub overwrite
7. load_scene_dic()                  // @pasta_search構築
```

**確認**: `@pasta_config` は既に **Phase 2** で最初に登録されており、Requirement 3 は既に満たされている。

#### STORE.actors 現状

```lua
-- store.lua: Line 23
STORE.actors = {}  -- 空テーブルで初期化（静的）

-- store.lua: Line 64 (STORE.reset())
STORE.actors = {}  -- リセット時も空テーブルで初期化
```

**ギャップ**: CONFIG.actorからの初期化ロジックが存在しない。

#### ACTOR.get_or_create() パターン

```lua
-- actor.lua: Line 69-79
function ACTOR.get_or_create(name)
    if not STORE.actors[name] then
        local actor = {
            name = name,
            spot = nil,
        }
        setmetatable(actor, ACTOR_IMPL)  -- メタテーブル設定
        STORE.actors[name] = actor
    end
    return STORE.actors[name]
end
```

**既存**: メタテーブル設定パターンは確立済み（`setmetatable(actor, ACTOR_IMPL)`）

### 1.3 統合ポイント

| 統合箇所            | 既存インターフェース     | 必要な変更                                    |
| ------------------- | ------------------------ | --------------------------------------------- |
| TOML→Lua            | `@pasta_config` 自動登録 | **変更不要** - `[actor.*]` は自動的に公開済み |
| STORE初期化         | `STORE.actors = {}`      | CONFIG.actorからのディープコピー追加          |
| STORE.reset()       | 空テーブル再初期化       | CONFIG.actorからの再初期化に変更              |
| ACTOR.get_or_create | 新規アクター作成ロジック | CONFIG由来チェック追加（既存優先）            |

---

## 2. 要求仕様の実現可能性分析

### Requirement 1: pasta.toml での actor セクション定義

| AC  | 技術ニーズ                        | 既存実装                                              | ギャップ                    |
| --- | --------------------------------- | ----------------------------------------------------- | --------------------------- |
| 1.1 | `[actor.*]` → `CONFIG.actor` 公開 | ✅ `PastaConfig::parse()` が全カスタムフィールドを公開 | **なし** - 自動的に公開済み |
| 1.2 | キー・バリューペア保持            | ✅ `toml_to_lua()` が再帰的にテーブル変換              | **なし**                    |
| 1.3 | 任意カスタムフィールド許可        | ✅ TOMLの構造がそのまま保持される                      | **なし**                    |
| 1.4 | セクション不在時の空テーブル      | ✅ TOMLパース仕様（存在しないキーは`nil`）             | **なし**                    |

**結論**: Requirement 1 は**既に完全に実装済み**。追加コード不要。

### Requirement 2: STORE.actors の自動初期化

| AC  | 技術ニーズ                             | 既存実装                     | ギャップ                                       |
| --- | -------------------------------------- | ---------------------------- | ---------------------------------------------- |
| 2.1 | `CONFIG.actor` → `STORE.actors` コピー | ❌ 実装なし                   | **Lua側実装必要** - store.lua 初期化処理追加   |
| 2.2 | CONFIG不在時の空テーブル               | ⚠️ 静的初期化のみ             | **Lua側実装必要** - nilガード付きコピー        |
| 2.3 | メタテーブル設定                       | ✅ `ACTOR_IMPL` パターン確立  | **Lua側実装必要** - CONFIG由来アクターへの適用 |
| 2.4 | 既存アクター優先                       | ⚠️ get_or_create は未チェック | **Lua側実装必要** - CONFIG由来チェック追加     |

**実装ポイント**:
- pasta.store.lua でモジュール読み込み時にCONFIG.actorをコピー
- 各アクターに`setmetatable(actor, require("pasta.actor").ACTOR_IMPL)`を適用
- ACTOR.get_or_createで`STORE.actors[name]`の存在チェック（既存優先ロジックは既にある）

### Requirement 3: ライブラリモジュールの早期公開

| AC  | 技術ニーズ                            | 既存実装                                                              | ギャップ |
| --- | ------------------------------------- | --------------------------------------------------------------------- | -------- |
| 3.1 | `@pasta_config` 最初に登録            | ✅ Line 538 で最初に登録                                               | **なし** |
| 3.2 | 外部依存なしモジュール即座登録        | ✅ Line 541 `@enc` 即座登録、mlua-stdlib は `with_config()` で登録済み | **なし** |
| 3.3 | `@pasta_persistence` シーン読み込み前 | ✅ Line 544 で登録（scene_dic前）                                      | **なし** |
| 3.4 | `@pasta_search` のみ遅延              | ✅ `load_scene_dic()` → `finalize_scene()` で登録                      | **なし** |

**結論**: Requirement 3 は**既に完全に実装済み**。現在の順序で要件を満たしている。

### Requirement 4: 既存コードとの後方互換性

| AC  | 技術ニーズ               | 既存実装                                   | ギャップ                                 |
| --- | ------------------------ | ------------------------------------------ | ---------------------------------------- |
| 4.1 | 動的アクター追加の共存   | ✅ Luaテーブル動作（代入可能）              | **なし** - Luaテーブルは動的追加対応     |
| 4.2 | get_or_create の既存優先 | ✅ `if not STORE.actors[name]` チェック済み | **なし** - 既存ロジックで対応済み        |
| 4.3 | reset時のCONFIG復帰      | ❌ 空テーブル再初期化のみ                   | **Lua側実装必要** - CONFIG.actor再コピー |

**実装ポイント**:
- STORE.reset() で `STORE.actors = {}` を CONFIG.actor コピーに変更（AC 2.1と同じロジック再利用）

---

## 3. 実装アプローチ検討

### Option A: 既存コンポーネント拡張 ✅ **推奨**

**対象ファイル**:
1. `crates/pasta_lua/scripts/pasta/store.lua` (25行追加)
2. `crates/pasta_lua/scripts/pasta/actor.lua` (ACTOR_IMPLエクスポート、5行追加)

**変更内容**:

#### store.lua 変更点

```lua
-- モジュール末尾（return STORE前）に追加
do
    local ok, config = pcall(require, "@pasta_config")
    if ok and config and config.actor then
        local ACTOR_IMPL = require("pasta.actor").ACTOR_IMPL
        for name, props in pairs(config.actor) do
            -- Deep copy + metatable setup
            local actor = { name = name, spot = props.spot }
            for k, v in pairs(props) do
                if k ~= "spot" and k ~= "name" then
                    actor[k] = v
                end
            end
            setmetatable(actor, ACTOR_IMPL)
            STORE.actors[name] = actor
        end
    end
end

-- STORE.reset() 変更
function STORE.reset()
    -- ... existing cleanup ...
    STORE.actors = {}
    -- CONFIG.actor再初期化（上記と同じロジック）
    local ok, config = pcall(require, "@pasta_config")
    if ok and config and config.actor then
        -- ... same initialization logic ...
    end
end
```

#### actor.lua 変更点

```lua
-- ACTOR_IMPLをACTORモジュールに追加（エクスポート）
ACTOR.ACTOR_IMPL = ACTOR_IMPL  -- store.luaからアクセス可能にする
```

**互換性評価**:
- ✅ 既存の `STORE.actors` アクセスコードは無変更
- ✅ `ACTOR.get_or_create()` は既存優先ロジックにより動作不変
- ✅ `@pasta_config` は既に最初に登録済み（初期化コードで利用可能）
- ✅ メタテーブル設定パターンは既存実装と一貫

**Trade-offs**:
- ✅ 最小限のファイル変更（2ファイルのみ）
- ✅ 既存パターンと完全一貫
- ✅ テストケース追加が容易
- ❌ store.luaが一時的にactor.luaに依存（循環参照回避設計は維持）

### Option B: 新規コンポーネント作成 ❌ **非推奨**

**新規ファイル**: `scripts/pasta/config_init.lua`

**Rationale**: 初期化ロジックを独立モジュールに分離

**Trade-offs**:
- ❌ 新規ファイル追加（複雑性増加）
- ❌ require順序管理が必要
- ❌ store.luaの「他モジュールをrequireしない」設計原則に違反
- ✅ 責務分離は明確

**結論**: 初期化ロジックは25行程度でstore.lua内に収まる。新規モジュール不要。

### Option C: Rust側での初期化 ❌ **実装不可**

**Rationale**: Rust側で `STORE.actors` を直接初期化

**制約**:
- ❌ `ACTOR_IMPL` メタテーブルはLua側でのみ定義
- ❌ Rustから Lua メタテーブルを正しく設定するのは複雑
- ❌ Lua側初期化コードとの二重管理リスク

**結論**: Lua側実装に留めるべき。

---

## 4. 実装複雑性とリスク

### 複雑性: **S (Small - 1-3日)**

**根拠**:
- Requirement 1, 3 は実装済み（0日）
- Requirement 2, 4 は既存パターンの適用のみ（1-2日）
  - store.lua初期化ロジック追加（0.5日）
  - actor.lua エクスポート追加（0.1日）
  - テストケース作成（0.5日）
  - 統合テスト・検証（1日）

### リスク: **Low**

**根拠**:
- ✅ 既存パターン（メタテーブル設定）の再利用
- ✅ `@pasta_config` パイプラインは実証済み
- ✅ Luaテーブル操作は言語仕様で安全
- ✅ 後方互換性は既存ロジックで保証済み
- ⚠️ 唯一のリスク: store.lua → actor.lua 依存によるrequire順序（init処理内pcallでカバー）

**リスク低減策**:
- pcall による安全なモジュールロード
- nil ガードによる CONFIG 不在時の安全動作
- 既存テストスイートでの回帰テスト実施

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

**Option A: 既存コンポーネント拡張**を推奨。

**理由**:
1. Requirement 1, 3 は実装不要（既存機能で満たされている）
2. Requirement 2, 4 は最小限のLua変更で実現可能
3. 既存パターンと完全一貫（学習コスト・保守コスト低）
4. リスクは Low、実装期間は S（1-3日）

### 主要決定事項

1. **初期化タイミング**: store.lua モジュール読み込み時（`return STORE` 前）
   - `@pasta_config` は既にPhase 2で登録済み（利用可能）
   - 初期化コード内で `pcall(require, "@pasta_config")` で安全にアクセス

2. **メタテーブル設定方法**: `ACTOR.ACTOR_IMPL` をエクスポートし、store.luaから参照
   - 既存の `setmetatable(actor, ACTOR_IMPL)` パターンを再利用

3. **nilガード戦略**: `pcall` + `config.actor` 存在チェック
   - CONFIGやactorセクション不在時は空テーブルで初期化（後方互換）

### 調査済み項目（Research Complete）

- ✅ `@pasta_config` 公開パイプライン完全調査済み
- ✅ モジュール登録順序確認済み
- ✅ 既存メタテーブルパターン特定済み
- ✅ Luaテーブル動作（動的追加）検証済み

### 追加調査不要

全ての技術要素は既存実装から特定済み。設計フェーズで詳細実装仕様を策定可能。

---

## 6. 要求仕様とのマッピング

| Requirement                      | 既存アセット                                         | ギャップ | 実装箇所                     |
| -------------------------------- | ---------------------------------------------------- | -------- | ---------------------------- |
| **Req 1**: actor セクション定義  | ✅ `PastaConfig::parse()`, `register_config_module()` | なし     | **実装不要**                 |
| **Req 2.1**: CONFIG→STORE コピー | ⚠️ store.lua 静的初期化                               | Missing  | store.lua 初期化ブロック追加 |
| **Req 2.2**: nil時空テーブル     | ⚠️ 静的初期化                                         | Missing  | nilガード付きコピー          |
| **Req 2.3**: メタテーブル設定    | ✅ `ACTOR_IMPL`                                       | Missing  | setmetatable適用             |
| **Req 2.4**: 既存優先            | ✅ `get_or_create` チェック                           | なし     | **実装不要**                 |
| **Req 3.1**: config最初          | ✅ Line 538                                           | なし     | **実装不要**                 |
| **Req 3.2**: 即座登録            | ✅ Line 541, with_config                              | なし     | **実装不要**                 |
| **Req 3.3**: persistence前       | ✅ Line 544                                           | なし     | **実装不要**                 |
| **Req 3.4**: search遅延          | ✅ finalize_scene                                     | なし     | **実装不要**                 |
| **Req 4.1**: 動的追加共存        | ✅ Luaテーブル仕様                                    | なし     | **実装不要**                 |
| **Req 4.2**: get_or_create優先   | ✅ 既存チェック                                       | なし     | **実装不要**                 |
| **Req 4.3**: reset復帰           | ❌ 空テーブル初期化                                   | Missing  | STORE.reset() 変更           |

**ギャップサマリー**: 3箇所のMissing（全てstore.lua内で解決可能）
