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

#### Rust側（pasta_lua クレート）

| ファイル | 責務 | 現状 | 変更 |
|----------|------|------|------|
| `src/code_gen/scope_gen.rs` | `％` 行 → `clear_spot`/`set_spot` コード生成 | `actors.is_empty()` のとき未生成 — **仕様通り** | 不要 |
| `src/loader/config.rs` | pasta.toml パース → `custom_fields` | `[actor]` セクションは自動的に公開済み | 不要 |
| `src/runtime/module_registry.rs` | `@pasta_config` Lua モジュール登録 | `toml_to_lua` でTOML値をそのまま変換 — `[actor]` サブテーブルに `name` フィールドが欠落 | **修正対象** — `register_config_module` で `[actor]` サブテーブルに `name`（= キー名）を注入 |

#### Lua側（scripts/pasta/）

| ファイル | 責務 | 関連機能 | 変更 |
|----------|------|----------|------|
| `store.lua` | グローバルストア | `STORE.actors = CONFIG.actor`（直接参照共有）、`STORE.actor_spots` の初期化 | 不要（Rust 側で `name` 注入されるため） |
| `actor.lua` | アクター管理 | `ACTOR.get_or_create()` | 不要（`name` は Rust 側で設定済み） |
| `act.lua` | アクションオブジェクト | `ACT_IMPL.__index` — プロキシ生成（`actor.name` 依存） | 不要（`name` は Rust 側で設定済み） |
| `shiori/sakura_builder.lua` | さくらスクリプト生成 | `actor.name` でスポット解決 | **修正対象** — インターフェース簡素化（議題1で合意済み） |
| `shiori/act.lua` | SHIORI専用アクション | `build()` で `STORE.actor_spots` を `BUILDER.build()` に渡す | **修正対象** — 書き戻し処理廃止（議題1で合意済み） |

### 1.2 既存パターンと規約

#### CONFIG.actor → STORE.actors パイプライン（config-actors-initialization spec で実装済み）

```
pasta.toml [actor.さくら] spot=0
  ↓ PastaConfig::parse() (Rust)
  ↓ @pasta_config.actor (Lua table)
  ↓ store.lua: STORE.actors = CONFIG.actor  (直接参照共有)
  ↓ store.lua: STORE.actor_spots["さくら"] = 0  (spot値転送)
```

**問題**: `STORE.actors["さくら"]` = `{spot=0}` — `name` フィールドなし。Rust 側 `register_config_module` で `[actor]` サブテーブルに `name` を注入することで解決する（ディスカッションで合意済み）

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

## 3. 実装アプローチ（ディスカッションで確定済み）

### name フィールド注入: Rust 側 `register_config_module` で対応

**対象ファイル**: `crates/pasta_lua/src/runtime/module_registry.rs`

**変更内容**: `register_config_module` で `custom_fields` の `[actor]` セクション配下の各サブテーブルに `name` フィールド（= キー名）を注入してから `toml_to_lua` で Lua テーブルに変換する。

**根拠**:
- データ提供側（Rust）の責務としてデータの整合性を保証する
- `[actor]` セクション固有の対応（他セクションへの影響なし）
- Lua 側（`store.lua` / `actor.lua` / `act.lua`）での正規化が不要になり、変更箇所を最小化

**Lua 側で不要になった検討事項**:
- ~~Option A: `ACTOR.get_or_create` 正規化拡張~~ → 不要
- ~~Option B: `store.lua` 初期化時の正規化~~ → 不要
- ~~Option C: `ACT_IMPL.__index` でのオンデマンド正規化~~ → 不要

---

## 4. 実装複雑度とリスク

| 項目 | 評価 | 理由 |
|------|------|------|
| **Effort** | **S**（1–3日） | Rust 側1ファイル + Lua 側2ファイル（BUILDER.build インターフェース変更）、影響範囲が限定的 |
| **Risk** | **Low** | 既存テストが CONFIG 由来アクターの正規化を前提として設計されている。既存動作（`％` ありシーン）への影響なし |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

2つの変更を組み合わせて解決する:

**1. Rust 側: `[actor]` サブテーブルへの `name` フィールド注入**（ディスカッションで合意済み）

- `register_config_module` で `[actor]` セクション配下の各サブテーブルに `name`（= キー名）を注入
- データ提供側（Rust）の責務としてデータの整合性を保証する
- `[actor]` セクション固有の対応（他セクションへの汎用化は不要）
- Lua 側（`store.lua` / `actor.lua` / `act.lua`）での正規化は不要になる

**2. `BUILDER.build()` のインターフェース簡素化**（ディスカッションで合意済み）

- `actor_spots` テーブルを直接変更（浅いコピーのループを廃止）
- 返却値をスクリプト文字列のみに変更（第2返却値 `actor_spots` を廃止）
- 呼び出し元（`SHIORI_ACT_IMPL.build`）の書き戻し処理を廃止
- テスト再現性は入力テーブルの制御で担保（既存の「入力テーブルが変更されないことを確認」テストは方針に合わせて更新）

### 設計フェーズで決定すべき事項

1. **Req 2 ログ出力の実装場所**: `ACT_IMPL.__index`（アクター参照時）か `BUILDER.build`（スクリプト生成時）か
2. **Research Needed**: Lua 側 `LOGGER` モジュールの API 確認（`tracing` 連携の既存パターン）

### キャリーフォワード Research Items

- [ ] `LOGGER` / `LOG` モジュールの現在の API と `tracing::debug!` / `tracing::warn!` 相当の Lua 関数
