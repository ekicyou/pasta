# Implementation Gap Analysis

## 分析サマリー

- **スコープ**: `％` アクター宣言なしシーンにおけるスコープ継承・アクター解決の不具合修正（Lua ランタイム側）
- **設計原則**: 全トークは最終的に `BUILDER.build()` を経由してさくらスクリプトに変換される。この関数に適切なスコープ情報（`actor_spots`）を注入し、関数内で直接変更することで外部のスコープ状態を更新する。テスト再現性は入力テーブルの制御で担保する。
- **スコープフロー（合意済み設計変更）**: `STORE.actor_spots` → `SHIORI_ACT_IMPL.build` → `BUILDER.build(tokens, config, actor_spots)` → `actor_spots` を直接変更、スクリプト文字列のみ返却。**コピー＋第2返却値＋書き戻しパターンを廃止し、直接変更方式に移行する**。
- **根本原因**: CONFIG由来アクター（`STORE.actors`）に `name` フィールドが欠落しているため、`BUILDER.build()` 内で `actor_spots[actor.name]`（= `actor_spots[nil]`）のルックアップが失敗し、正しいスコープ情報が注入されているにもかかわらず参照できない。
- **推奨アプローチ**: CONFIG由来アクターの `name` フィールド正規化 — スコープフローを変更せず、データの整合性を修正することで解決

---

## 1. 現状調査

### 1.1 主要コンポーネント

#### Rust側（pasta_lua クレート） — 変更不要

| ファイル | 責務 | 現状 |
|----------|------|------|
| `src/code_gen/scope_gen.rs` | `％` 行 → `clear_spot`/`set_spot` コード生成 | `actors.is_empty()` のとき未生成 — **仕様通り** |
| `src/loader/config.rs` | pasta.toml パース → `custom_fields` | `[actor]` セクションは自動的に公開済み |
| `src/runtime/module_registry.rs` | `@pasta_config` Lua モジュール登録 | TOML → Lua テーブル変換は正常動作 |

#### Lua側（scripts/pasta/） — 修正対象

| ファイル | 責務 | 関連機能 |
|----------|------|----------|
| `store.lua` | グローバルストア | `STORE.actors = CONFIG.actor`（直接参照共有）、`STORE.actor_spots` の初期化 |
| `actor.lua` | アクター管理 | `ACTOR.get_or_create()` — 既存エントリ正規化の欠落箇所 |
| `act.lua` | アクションオブジェクト | `ACT_IMPL.__index` — プロキシ生成（`actor.name` 依存） |
| `shiori/sakura_builder.lua` | さくらスクリプト生成 | `actor.name` でスポット解決（nil なら常にスポット 0） |
| `shiori/act.lua` | SHIORI専用アクション | `build()` で `STORE.actor_spots` を `BUILDER.build()` に渡す |

### 1.2 既存パターンと規約

#### CONFIG.actor → STORE.actors パイプライン（config-actors-initialization spec で実装済み）

```
pasta.toml [actor.さくら] spot=0
  ↓ PastaConfig::parse() (Rust)
  ↓ @pasta_config.actor (Lua table)
  ↓ store.lua: STORE.actors = CONFIG.actor  (直接参照共有)
  ↓ store.lua: STORE.actor_spots["さくら"] = 0  (spot値転送)
```

**問題**: `STORE.actors["さくら"]` = `{spot=0}` — `name` フィールドなし、`ACTOR_IMPL` metatable なし

#### ACTOR.get_or_create の既存動作

```lua
function ACTOR.get_or_create(name)
    if not STORE.actors[name] then
        -- 新規作成時のみ name と metatable を設定
        local actor = { name = name, spot = nil }
        setmetatable(actor, ACTOR_IMPL)
        STORE.actors[name] = actor
    end
    return STORE.actors[name]  -- 既存エントリはそのまま返す
end
```

**ギャップ**: `STORE.actors[name]` が存在する場合（CONFIG由来）、`name` フィールドの追加も `ACTOR_IMPL` metatable の設定も行われない。

#### sakura_builder のスポット解決コード

```lua
-- sakura_builder.lua L81
local actor_name = actor and actor.name    -- CONFIG由来なら nil
local spot = actor_spots[actor_name] or 0  -- actor_spots[nil] → nil → 常に 0
```

### 1.3 バグ発現フロー（トレース結果）

```
OnSecondChange
  → EVENT.fire(req)
    → create_act(req) → SHIORI_ACT.new(STORE.actors, req) → act.actors = STORE.actors
    → handler(act) → virtual_dispatcher.dispatch(act)
      → check_talk(act) → create_scene_thread("OnTalk", act)
        → SCENE.co_exec("OnTalk", nil, nil) → coroutine.create(wrapped_fn)
    → resume_until_valid(co, act) → coroutine.resume(co, act)
      → wrapped_fn(act) → fn(act)  [％なしシーン]
        → act.さくら  [ACT_IMPL.__index]
          → self.actors["さくら"] → {spot=0}  [CONFIG由来, name=nil]
          → ACTOR.create_proxy({spot=0}, act)
            → proxy:talk("こんにちは")
              → act:talk({spot=0}, "こんにちは")
                → token: {type="talk", actor={spot=0}, text="こんにちは"}
        → act:build()
          → BUILDER.build(tokens, config, STORE.actor_spots)
            → actor_name = actor.name → nil
            → actor_spots[nil] or 0 → 0  [STORE.actor_spots["さくら"]=0 が参照されない]
            → spot_to_tag(0) → "\0"
```

**結果**: 単一アクター（スポット 0）では偶然正常動作するが、複数アクター構成で破綻する。  
例: `[actor.さくら] spot=0`, `[actor.むらさき] spot=1` の構成で `むらさき` が常にスポット 0 になる。

### 1.4 統合ポイント

`BUILDER.build()` は全トークをさくらスクリプトに変換する**唯一の経路**である。スコープ情報（`actor_spots`）はこの関数のパラメータとして注入され、返却値として更新状態が外部に伝搬される。このフロー自体は既に正しく機能しており、変更不要。

| 統合箇所 | 既存インターフェース | 必要な変更 |
|----------|---------------------|-----------|
| `BUILDER.build()` | `actor_spots` を直接変更、スクリプト文字列のみ返却 | **変更あり** — コピーループ廃止、第2返却値廃止 |
| `SHIORI_ACT_IMPL.build()` | `STORE.actor_spots` → `BUILDER.build` | **変更あり** — 書き戻し不要（直接変更のため） |
| `ACTOR.get_or_create` | 新規のみ正規化 | 既存エントリにも `name`/metatable を設定 |
| `store.lua` 初期化 | `CONFIG.actor` 直接参照共有 | 変更不要（get_or_create で正規化すれば解決） |
| `STORE.actor_spots` | CONFIG.actor.spot からの転送 | **既に実装済み** — 問題なし |
| `sakura_builder` | `actor.name` でスポット参照 | 変更不要（`name` が設定されれば自然に解決） |
| `scope_gen.rs` | `％` なし → コード未生成 | **変更不要** — スコープ継承として仕様通り |

---

## 2. 要求仕様の実現可能性分析

### Requirement 1: `％` 省略時のスコープ継承と正しいアクター解決

| AC | 技術ニーズ | 既存実装 | ギャップ |
|---|-----------|---------|---------|
| 1.1 | `STORE.actor_spots` からスポット引き継ぎ | ✅ `BUILDER.build()` がスコープを受け取り返却するフローは正常 | **actor.name 欠落** — `actor_spots[nil]` でルックアップ失敗 |
| 1.2 | 初回実行時の `pasta.toml` spot 適用 | ✅ `store.lua` で `STORE.actor_spots` に転送済み | **actor.name 欠落** — 同上 |
| 1.3 | `％` ありシーンの既存動作維持 | ✅ `scope_gen.rs` の生成コードは正常 | **なし** |
| 1.4 | イベント経路に依存しない一貫性 | ✅ 両経路とも同一 `act` オブジェクトを使用 | **actor.name 欠落** — 同上 |
| 1.5 | `act.アクター名` が有効プロキシを返す | ✅ `ACT_IMPL.__index` → `ACTOR.create_proxy` は動作 | **プロキシの actor.name が nil** |

**根本原因**: AC 1.1, 1.2, 1.4, 1.5 のギャップはすべて同一原因 — CONFIG由来アクターの `name` フィールド欠落。スコープフロー（`STORE.actor_spots` → `BUILDER.build` → 書き戻し）は正しく機能しており、`actor.name` が正規化されれば全ACが自動的に充足される。

> **統合済み旧要件の充足状況**:
> - 旧 Req 2（初期スコープの自動設定）: AC 2.1（STORE.actor_spots 初期化）は store.lua L88-92 で✅実装済み。AC 2.2（デフォルト 0）は BUILDER.build の `or 0` で✅対応済み。
> - 旧 Req 3（co_exec/SHIORI一貫性）: AC 3.2（act.actors設定）は✅実装済み。AC 3.1, 3.3 は本 Req の AC 1.4, 1.5 に統合。

### Requirement 2: `％` 行欠落時の診断支援

| AC | 技術ニーズ | 既存実装 | ギャップ |
|---|-----------|---------|---------|
| 2.1 | スコープ継承ログ（debug レベル） | ❌ なし | **Lua 側実装が必要** |
| 2.2 | 未定義アクター参照警告（warn レベル） | ❌ なし | **Lua 側実装が必要** |
| 2.3 | `％` 省略をエラーとしない | ✅ パーサーは `％` なしを正常にパース | **なし** |

**注記**: Req 2 のログ出力は Lua 側で `LOGGER` モジュール（既存の `tracing` 連携）を使用する想定。Research Needed: `LOGGER` モジュールの現在の API を確認。

---

## 3. 実装アプローチ検討

### Option A: ACTOR.get_or_create 正規化拡張 ✅ **推奨**

**対象ファイル**:
1. `crates/pasta_lua/scripts/pasta/actor.lua` — `get_or_create` 拡張（10行程度）
2. `crates/pasta_lua/scripts/pasta/act.lua` — 診断ログ追加（ACT_IMPL.__index 内、5行程度）

**変更内容**:

#### actor.lua: get_or_create の正規化拡張

```lua
function ACTOR.get_or_create(name)
    local actor = STORE.actors[name]
    if not actor then
        -- 新規作成
        actor = { name = name, spot = nil }
        setmetatable(actor, ACTOR_IMPL)
        STORE.actors[name] = actor
    else
        -- 既存エントリの正規化（CONFIG由来の場合 name/metatable 未設定）
        if not actor.name then
            actor.name = name
        end
        if not getmetatable(actor) then
            setmetatable(actor, ACTOR_IMPL)
        end
    end
    return actor
end
```

**根拠**:
- 正規化を「使用時」に行うため、初期化順序の問題が発生しない
- CONFIG由来・動的作成問わず、すべてのアクターが `get_or_create` を通過すれば正規化される
- 既存テスト（`config_actors_initialization_test.rs`）のコメント "CONFIG由来アクターにはnameフィールドがないため、ACTOR.get_or_create経由でnameが設定されたアクターを取得してからcreate_wordをテストする" と整合

**呼び出しパスの確認**:
- `％` 行のトランスパイラ出力: `PASTA.create_actor("さくら")` → `ACTOR.get_or_create("さくら")` ✅ 確実に通過
- `act.さくら` アクセス: `ACT_IMPL.__index` → `self.actors["さくら"]` → `create_proxy` — ❌ `get_or_create` を通らない

**追加変更**: `ACT_IMPL.__index` でプロキシ生成前に `ACTOR.get_or_create(key)` を呼ぶか、直接正規化するか要設計判断。

**Trade-offs**:
- ✅ 変更箇所が最小（actor.lua + act.lua の2ファイル）
- ✅ 既存の `get_or_create` パターンの自然な拡張
- ✅ 既存テストとの整合性が高い
- ❌ `ACT_IMPL.__index` への変更が必要（`get_or_create` だけでは不完全）

### Option B: store.lua 初期化時の正規化

**対象ファイル**:
1. `crates/pasta_lua/scripts/pasta/store.lua` — CONFIG.actor 正規化ループ追加

**変更内容**: `store.lua` の CONFIG.actor 転送処理で `name` とmetatable を同時設定

```lua
if ok and type(CONFIG.actor) == "table" then
    -- CONFIG.actorを正規化してSTORE.actorsに設定
    for name, actor in pairs(CONFIG.actor) do
        if type(actor) == "table" then
            actor.name = name  -- name フィールド追加
            -- ACTOR_IMPL は循環参照を避けるため直接設定しない
        end
    end
    STORE.actors = CONFIG.actor
    -- spot値転送
    ...
end
```

**Trade-offs**:
- ✅ 初期化時に一括正規化 — すべてのパスで正規化済みアクターが見える
- ✅ `ACT_IMPL.__index` の変更不要
- ❌ `store.lua` から `actor.lua` への依存追加による循環参照リスク（metatable設定のため）
- ❌ `name` だけ設定して metatable を設定しない場合、`ACTOR_IMPL` のメソッドが使えない
- ❌ `STORE.reset()` でも同じ正規化ロジックの重複が必要

### Option C: ACT_IMPL.__index でのオンデマンド正規化

**対象ファイル**:
1. `crates/pasta_lua/scripts/pasta/act.lua` — `__index` 内で正規化

**変更内容**: `ACT_IMPL.__index` でアクター参照時、`name` がなければ正規化

```lua
function ACT_IMPL.__index(self, key)
    local method = ACT_IMPL[key]
    if method then return method end

    local actor = self.actors[key]
    if actor then
        -- オンデマンド正規化
        if not actor.name then
            actor.name = key
        end
        if not getmetatable(actor) then
            setmetatable(actor, ACTOR.ACTOR_IMPL)  -- 要エクスポート
        end
        return ACTOR.create_proxy(actor, self)
    end
    return nil
end
```

**Trade-offs**:
- ✅ 最も確実 — プロキシ生成パスで必ず正規化される
- ✅ `get_or_create` の変更不要
- ❌ `ACTOR_IMPL` を act.lua にエクスポートする必要がある（現在は非公開）
- ❌ 毎回の `__index` 呼び出しで条件チェックのオーバーヘッド（微小）
- ❌ `get_or_create` 経由以外のアクター参照（将来的な拡張ポイント）では正規化されない

---

## 4. 実装複雑度とリスク

| 項目 | 評価 | 理由 |
|------|------|------|
| **Effort** | **S**（1–3日） | Lua 側のみの変更、既存パターンの拡張、影響範囲が限定的 |
| **Risk** | **Low** | 既存テストが CONFIG 由来アクターの正規化を前提として設計されている。既存動作（`％` ありシーン）への影響なし |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

2つの変更を組み合わせて解決する:

**1. CONFIG由来アクターの `name` フィールド正規化**

`ACTOR.get_or_create` を拡張し、`ACT_IMPL.__index` から呼び出す。

- `get_or_create` は正規化の自然な責務所在
- `__index` からの `get_or_create` 呼び出しで、CONFIG 由来アクターの遅延正規化を統一的に処理
- 既存テスト（`config_actors_initialization_test.rs`）の想定と完全一致
- `store.lua` の循環参照リスクを回避

**2. `BUILDER.build()` のインターフェース簡素化**（ディスカッションで合意済み）

- `actor_spots` テーブルを直接変更（浅いコピーのループを廃止）
- 返却値をスクリプト文字列のみに変更（第2返却値 `actor_spots` を廃止）
- 呼び出し元（`SHIORI_ACT_IMPL.build`）の書き戻し処理を廃止
- テスト再現性は入力テーブルの制御で担保（既存の「入力テーブルが変更されないことを確認」テストは方針に合わせて更新）

### 設計フェーズで決定すべき事項

1. **`ACT_IMPL.__index` の正規化戦略**: `self.actors[key]` の直後に `ACTOR.get_or_create(key)` を呼ぶか、インライン正規化か
2. **Req 2 ログ出力の実装場所**: `ACT_IMPL.__index`（アクター参照時）か `BUILDER.build`（スクリプト生成時）か
3. **Research Needed**: Lua 側 `LOGGER` モジュールの API 確認（`tracing` 連携の既存パターン）

### キャリーフォワード Research Items

- [ ] `LOGGER` / `LOG` モジュールの現在の API と `tracing::debug!` / `tracing::warn!` 相当の Lua 関数
- [ ] `ACTOR_IMPL` metatable の公開方法 — `actor.lua` から `ACTOR.IMPL` としてエクスポート済みか確認
