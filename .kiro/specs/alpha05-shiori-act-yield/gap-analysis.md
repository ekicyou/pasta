# 実装ギャップ分析レポート

## 分析概要

**機能名**: alpha05-shiori-act-yield  
**分析日**: 2026-01-31  
**分析者**: GitHub Copilot (Claude Opus 4.5)

### サマリー

- **スコープ**: SHIORI_ACT用yield制御、init_script初期化、pasta.toml設定管理、総合テスト
- **主要な発見**: `@pasta_config` は既にRust側で実装済み。Luaラッパー `pasta.config` を新規作成すれば設定アクセス可能
- **リスク**: 低〜中。既存パターンの拡張が主で、新規アーキテクチャ変更なし
- **推奨アプローチ**: Option A（既存コンポーネント拡張）を中心に、`pasta.config`は新規ファイルとして追加（Hybrid）
- **工数見積**: M（3-5日）- 既存パターンに沿った実装、テスト整備が主

---

## 1. 現状調査

### 1.1 関連ファイル・モジュール

| ファイル | 役割 | 状態 |
|----------|------|------|
| `scripts/pasta/shiori/act.lua` | SHIORI_ACT実装 | 既存・拡張対象 |
| `scripts/pasta/act.lua` | ACT_IMPL基底クラス | 既存・参照のみ |
| `scripts/pasta/config.lua` | 設定モジュール | **新規作成** |
| `scripts/pasta/store.lua` | データストア | 既存・参照のみ |
| `tests/lua_specs/shiori_act_test.lua` | 単体テスト | 既存・拡張対象 |
| `tests/lua_specs/shiori_act_integration_test.lua` | 統合テスト | **新規作成** |

### 1.2 既存アーキテクチャパターン

**モジュール構造（MODULE/MODULE_IMPL分離パターン）**:
```lua
local SHIORI_ACT = {}          -- モジュールテーブル
local SHIORI_ACT_IMPL = {}     -- 実装メタテーブル
setmetatable(SHIORI_ACT_IMPL, { __index = ACT.IMPL })  -- 継承
SHIORI_ACT.IMPL = SHIORI_ACT_IMPL  -- 継承用公開
```

**継承チェーン**:
```
SHIORI_ACT_IMPL → ACT.IMPL
```

**yieldパターン（ACT.IMPL）**:
```lua
function ACT_IMPL.yield(self)
    table.insert(self.token, { type = "yield" })
    local token = self.token
    self.token = {}
    self.now_actor = nil
    coroutine.yield({ type = "yield", token = token })
end
```

**設定読み込みパターン（Rust→Lua）**:
- `@pasta_config` モジュールがRust側で登録済み（`runtime/mod.rs:register_config_module`）
- `loader_context.custom_fields` がLuaテーブルとして提供される
- `[loader]` セクションは除外済み（セキュリティ考慮）

### 1.3 統合ポイント

| 統合点 | 現状 | 要件との関係 |
|--------|------|--------------|
| `@pasta_config` | Rust側で登録済み | Requirement 4のバックエンド |
| `coroutine.yield` | ACT_IMPL.yieldで使用 | Requirement 1のオーバーライド対象 |
| `reset()` | SHIORI_ACT_IMPLで実装済み | Requirement 2の差分理解 |
| `build()` | SHIORI_ACT_IMPLで実装済み | Requirement 1で呼び出し |

---

## 2. 要件実現可能性分析

### Requirement 1: SHIORI_ACT用yieldメソッド

**技術的ニーズ**:
- `coroutine.yield(script)` でさくらスクリプト文字列をyield
- `build()` → `reset()` → `yield` → `_resume_value`保存

**ギャップ**:
- **Missing**: `_resume_value` フィールドが未定義
- **Missing**: SHIORI_ACT_IMPL.yield() オーバーライドが未実装

**複雑性**: 低 - 既存パターン（ACT_IMPL.yield）の書き換え

**実装方針**:
```lua
function SHIORI_ACT_IMPL.yield(self)
    local script = self:build()
    self:reset()
    local rc = coroutine.yield(script)
    self._resume_value = rc
    return rc
end
```

### Requirement 2: init_script メソッド

**技術的ニーズ**:
- `_buffer`, `_current_scope`, `_resume_value` の完全リセット
- メソッドチェーン対応（`return self`）

**ギャップ**:
- **Missing**: `init_script()` メソッドが未定義
- **Constraint**: `reset()` は `_resume_value` をリセットしない（仕様差分）

**複雑性**: 低 - 単純なフィールドクリア

**実装方針**:
```lua
function SHIORI_ACT_IMPL.init_script(self)
    self._buffer = {}
    self._current_scope = nil
    self._resume_value = nil
    return self
end
```

**コンストラクタ修正**:
```lua
function SHIORI_ACT.new(actors)
    local base = ACT.new(actors)
    base._buffer = {}
    base._current_scope = nil
    base._resume_value = nil  -- 追加
    -- 設定読み込み（Requirement 3）
    return setmetatable(base, SHIORI_ACT_IMPL)
end
```

### Requirement 3: pasta.toml スコープ切り替え改行設定

**技術的ニーズ**:
- `[ghost].scope_switch_newlines` 設定読み込み
- `talk()` メソッドでの改行数制御

**ギャップ**:
- **Missing**: `[ghost]` セクション定義なし（pasta.tomlスキーマ拡張）
- **Missing**: `_scope_switch_newlines` フィールド未定義
- **Constraint**: 既存 `talk()` は固定で `\n` を1つ挿入

**複雑性**: 中 - 設定読み込み+既存ロジック修正

**実装方針**:
```lua
-- SHIORI_ACT.new() 内
local CONFIG = require("pasta.config")
self._scope_switch_newlines = CONFIG.get("ghost", "scope_switch_newlines", 1)

-- talk() 内のスコープ切り替え部分を修正
if self._current_scope ~= nil and self._scope_switch_newlines > 0 then
    for _ = 1, self._scope_switch_newlines do
        table.insert(self._buffer, "\\n")
    end
end
```

### Requirement 4: pasta.config モジュール

**技術的ニーズ**:
- `PASTA_CONFIG.get(section, key, default)` API
- `@pasta_config` のラッパー

**ギャップ**:
- **Missing**: `scripts/pasta/config.lua` ファイル不在
- **Available**: `@pasta_config` Rustモジュールは登録済み

**複雑性**: 低 - 薄いラッパー実装

**実装方針**:
```lua
--- @module pasta.config
local PASTA_CONFIG = {}

-- Rustモジュールをキャッシュ（ロード時1回のみ）
local cached_config = nil

local function get_cached_config()
    if cached_config == nil then
        local ok, config = pcall(require, "@pasta_config")
        cached_config = ok and config or {}
    end
    return cached_config
end

function PASTA_CONFIG.get(section, key, default)
    local config = get_cached_config()
    local section_data = config[section]
    if section_data == nil then
        return default
    end
    local value = section_data[key]
    if value == nil then
        return default
    end
    return value
end

return PASTA_CONFIG
```

### Requirement 5: 総合フィーチャーテスト

**技術的ニーズ**:
- シナリオベースのE2Eテスト
- 複数アクター、yield、設定変更の検証

**ギャップ**:
- **Missing**: `shiori_act_integration_test.lua` ファイル不在
- **Available**: `lua_test` フレームワーク、既存テストパターン

**複雑性**: 中 - 複数シナリオのテスト設計

### Requirement 6: 既存テストの拡充

**技術的ニーズ**:
- yield、init_script、設定関連のテスト追加

**ギャップ**:
- **Constraint**: 既存 `shiori_act_test.lua` は501行あり、追加は末尾が適切

**複雑性**: 低 - 既存パターンに従う

---

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張

**対象ファイル**:
- `scripts/pasta/shiori/act.lua` - yield, init_script, 設定読み込み追加
- `tests/lua_specs/shiori_act_test.lua` - テスト追加

**メリット**:
- ✅ 変更箇所が明確
- ✅ 既存継承構造を活用
- ✅ テストパターン踏襲

**デメリット**:
- ❌ act.luaが大きくなる（現在180行→250行程度）

**評価**: 推奨（設定モジュール以外）

### Option B: 新規コンポーネント作成

**対象ファイル**:
- `scripts/pasta/config.lua` - 新規作成

**メリット**:
- ✅ 責務分離が明確
- ✅ 他モジュールからも再利用可能
- ✅ テスト容易

**デメリット**:
- ❌ 新規ファイル追加

**評価**: 推奨（設定モジュールのみ）

### Option C: Hybrid（推奨）

- **pasta.config**: 新規作成（Option B）
- **SHIORI_ACT拡張**: 既存拡張（Option A）
- **統合テスト**: 新規作成（Option B）

**理由**:
1. 設定モジュールは汎用的で再利用性が高い
2. SHIORI_ACT拡張は既存継承構造に自然にフィット
3. 統合テストは別ファイルが要件定義通り

---

## 4. 工数・リスク評価

### 工数見積

| タスク | 見積 | 理由 |
|--------|------|------|
| Requirement 1 (yield) | 0.5日 | 既存パターン踏襲 |
| Requirement 2 (init_script) | 0.25日 | シンプル実装 |
| Requirement 3 (設定読み込み) | 0.5日 | talk()修正含む |
| Requirement 4 (pasta.config) | 0.5日 | 薄いラッパー |
| Requirement 5 (統合テスト) | 1日 | 複数シナリオ |
| Requirement 6 (テスト追加) | 0.75日 | 既存ファイル拡張 |
| ドキュメント整合性 | 0.5日 | DoD要件 |
| **合計** | **4日 (M)** | |

### リスク評価

| リスク | レベル | 軽減策 |
|--------|--------|--------|
| coroutine.yield互換性 | 低 | 既存ACT_IMPL.yieldと同パターン |
| 設定読み込みエラー | 低 | デフォルト値フォールバック |
| テスト複雑性 | 中 | 段階的シナリオ追加 |
| 既存テスト破壊 | 低 | 改行設定はデフォルト1で互換 |

**総合リスク**: 低〜中

---

## 5. Research Needed（設計フェーズで調査）

1. **コルーチン外yield時のエラーハンドリング**
   - 現状: Luaエラーが発生する
   - 確認: 明示的エラーメッセージが必要か

2. **pasta.toml `[ghost]` セクション追加**
   - 現状: Rust側のPastaConfigにghostフィールドなし
   - 確認: custom_fields経由でアクセス可能（追加不要の可能性）

3. **テストフィクスチャ設計**
   - 統合テスト用pasta.toml配置場所
   - モック設定のインジェクション方法

---

## 6. 推奨事項

### 設計フェーズへの引き継ぎ

1. **優先実装順序**: Requirement 4 → 1 → 2 → 3 → 6 → 5
   - 設定モジュールを先に作成することで、他要件の実装がスムーズに

2. **テスト戦略**:
   - 単体: 各メソッド個別テスト（shiori_act_test.lua拡張）
   - 統合: シナリオベース（shiori_act_integration_test.lua新規）

3. **互換性確保**:
   - `_scope_switch_newlines` デフォルト=1 で既存動作維持
   - `reset()` は現状維持、`init_script()` は上位互換

### Next Steps

1. `/kiro-spec-design alpha05-shiori-act-yield` で設計ドキュメント生成
2. または `/kiro-spec-design alpha05-shiori-act-yield -y` で要件自動承認＆設計生成

---

## Appendix: 要件-資産マッピング

| 要件 | 既存資産 | ギャップ | アプローチ |
|------|----------|----------|------------|
| R1: yield | ACT_IMPL.yield() | オーバーライド未実装 | 拡張 |
| R2: init_script | reset() | 新規メソッド | 拡張 |
| R3: 設定読み込み | @pasta_config | [ghost]アクセスラッパー | 新規+拡張 |
| R4: pasta.config | @pasta_config | Luaラッパー | 新規 |
| R5: 統合テスト | shiori_act_test.lua | 別ファイル | 新規 |
| R6: テスト拡充 | shiori_act_test.lua | 追加テストケース | 拡張 |
