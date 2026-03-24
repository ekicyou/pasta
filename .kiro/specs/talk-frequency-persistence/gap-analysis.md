# ギャップ分析レポート: talk-frequency-persistence

## 分析概要

- **影響範囲**: Lua ランタイム層のみ（`pasta_scripts/`）。Rust 側の変更は不要
- **主要変更点**: `virtual_dispatcher.lua` の `get_config()` 関数1箇所のみ
- **既存インフラ**: SAVE テーブル永続化機構（`pasta.save` / `@pasta_persistence`）は完全に稼働中
- **リスク**: 低 — 既存パターンの拡張であり、アーキテクチャ変更なし
- **工数**: S（1〜3日）

---

## 1. 現状調査

### 1.1 対象資産マップ

| ファイル | 役割 | 変更要否 |
|---------|------|---------|
| `pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` | OnTalk/OnHour 条件判定・発行 | ✅ 変更必要 |
| `pasta_scripts/pasta/save.lua` | SAVE テーブルロード | 変更不要 |
| `pasta_scripts/pasta/act.lua` | Act オブジェクト（`act.save` で SAVE 参照） | 変更不要 |
| `src/runtime/persistence.rs` | `@pasta_persistence` Rust 実装 | 変更不要 |
| `src/runtime/module_registry.rs` | `@pasta_config` 登録 | 変更不要 |
| `tests/shiori/virtual_event_config_test.rs` | デフォルト値テスト | ✅ テスト追加必要 |
| `tests/shiori/virtual_event_dispatch_test.rs` | ディスパッチテスト | テスト追加推奨 |

### 1.2 既存パターン

**設定読み込み（現行）**:
```lua
-- virtual_dispatcher.lua:get_config()
local ok, config = pcall(require, "@pasta_config")   -- pasta.toml → Lua テーブル
local ghost = config.ghost or {}
cached_config = {
    talk_interval_min = ghost.talk_interval_min or 180,
    talk_interval_max = ghost.talk_interval_max or 300,
    hour_margin = ghost.hour_margin or 30,
}
```

**SAVE テーブルアクセス（既存パターン）**:
```lua
local save = require("pasta.save")  -- persistence.load() 済みテーブル
save.some_key = value               -- 書き込み → Drop時に自動保存
```

**act.save からのアクセス**:
```lua
-- act.lua:ACT.new()
save = require("pasta.save"),  -- Act オブジェクト経由でも参照可能
```

### 1.3 キャッシュ機構

`get_config()` はモジュールローカル変数 `cached_config` に一度キャッシュすると、セッション中は再読込しない。`_reset()` でのみクリアされる。

→ **要件3（実行時変更の反映）に対するギャップ**: キャッシュ無効化メカニズムが必要。

---

## 2. 要件−資産マップとギャップ

| 要件 | 既存資産 | ギャップ |
|------|---------|---------|
| Req 1: SAVE からの読み出し | SAVE テーブル完全稼働（`pasta.save`） | `get_config()` が SAVE を参照していない |
| Req 2: 優先順位 (SAVE > toml > default) | `@pasta_config` でtoml読み込み済み、SAVE ロード済み | `get_config()` 内で SAVE → toml → default の3段フォールバック未実装 |
| Req 3: 実行時変更反映 | `cached_config` がセッション中固定 | キャッシュ無効化 or 毎回読み直しの仕組みが不在 |
| Req 4: バリデーション | なし | 型チェック・min>max 補正が未実装 |

---

## 3. 実装アプローチ

### Option A: `get_config()` の拡張（推奨）

**方針**: `virtual_dispatcher.lua` の `get_config()` のみ変更。キャッシュを廃止し、毎回 SAVE テーブルを参照する。

**変更内容**:
```lua
local function get_config()
    local save = require("pasta.save")
    local ok, config = pcall(require, "@pasta_config")
    if not ok then config = {} end
    local ghost = config.ghost or {}

    -- SAVE > toml > hardcoded default
    -- save_key: SAVE テーブルのキー（pasta_ プレフィックス付き）
    -- toml_key: pasta.toml [ghost] セクションのキー（プレフィックスなし）
    local function resolve(save_key, toml_key, default)
        local sv = save[save_key]
        if type(sv) == "number" then return sv end
        local tv = ghost[toml_key]
        if type(tv) == "number" then return tv end
        return default
    end

    local min = resolve("pasta_talk_interval_min", "talk_interval_min", 180)
    local max = resolve("pasta_talk_interval_max", "talk_interval_max", 300)
    if min > max then max = min end

    return {
        talk_interval_min = min,
        talk_interval_max = max,
        hour_margin = ghost.hour_margin or 30,
    }
end
```

**トレードオフ**:
- ✅ 変更箇所が1関数のみ（最小限の影響範囲）
- ✅ 既存の SAVE・config インフラをそのまま活用
- ✅ `require("pasta.save")` はキャッシュ済みモジュール返却のため高速
- ✅ キャッシュ廃止により Req 3（実行時変更）を自然に満たす
- ❌ 毎 `check_talk()` 呼び出しで `resolve()` が走る（OnSecondChange 毎秒）
  - → `require` はキャッシュ返却＋テーブルキー2回読みのみ → パフォーマンス問題なし

### Option B: キャッシュ維持 + 無効化関数追加

**方針**: `cached_config` を維持しつつ、公開関数 `M.invalidate_config()` を追加。Lua スクリプトが SAVE 変更後に明示的に呼ぶ。

**トレードオフ**:
- ✅ パフォーマンス最適（キャッシュ維持）
- ❌ ゴースト作者が無効化呼び出しを忘れるリスク
- ❌ API 表面積が増える（`invalidate_config()` の追加）
- ❌ 毎秒のテーブルルックアップ程度でキャッシュの恩恵は微小

### Option C: SAVE 変更監視（メタテーブル利用）

**方針**: SAVE テーブルに `__newindex` メタメソッドを設定し、`talk_interval_*` の変更を検知してキャッシュ自動無効化。

**トレードオフ**:
- ✅ 完全自動（ゴースト作者の負担ゼロ）
- ❌ SAVE テーブルのメタテーブル変更は影響範囲が大きい
- ❌ 過度の複雑化（KARPATHYガイドライン違反）
- ❌ 他モジュールへの副作用リスク

---

## 4. 複雑度・リスク評価

| 項目 | 評価 | 根拠 |
|------|------|------|
| **工数** | **S (1〜3日)** | Lua ファイル1箇所の変更 + テスト追加 |
| **リスク** | **Low** | 既存パターンの拡張、アーキテクチャ変更なし、Rust 層は未変更 |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: Option A（`get_config()` 拡張）

**理由**:
1. 変更箇所が最小（1関数）
2. キャッシュ廃止で Req 3 を暗黙的に満たす
3. パフォーマンス懸念なし（`require` キャッシュ + テーブルキー読みのみ）
4. ゴースト作者視点で追加API不要

### 設計フェーズでの決定事項

1. **`hour_margin` の永続化**: 現要件では対象外だが、同じパターンで追加可能。設計時に判断。
2. **テスト戦略**: 既存の `virtual_event_config_test.rs` パターンを踏襲し、SAVE テーブルに値を設定した上で `get_config()` の動作を検証。
