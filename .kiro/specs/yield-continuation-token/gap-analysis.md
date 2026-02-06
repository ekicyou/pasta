# Gap Analysis: yield-continuation-token

## 1. 現状調査（Current State Investigation）

### 1.1 関連アセット

| ファイル | 役割 | 本仕様との関連 |
|---------|------|---------------|
| `scripts/pasta/global.lua` | GLOBAL関数テーブル（空テーブル） | **直接変更対象** — デフォルト関数の登録先 |
| `scripts/pasta/act.lua` | ACT_IMPL（call/yield/build 等） | **既存充足** — `ACT_IMPL.call` L3でGLOBAL検索、`ACT_IMPL.yield` で `act:yield()` 実装済み |
| `scripts/pasta/scene.lua` | SCENE.co_exec（コルーチン生成） | **既存充足** — コルーチン生成・wrap済み |
| `scripts/pasta/shiori/event/init.lua` | EVENT.fire（コルーチンresume管理） | **既存充足** — resume_until_valid、set_co_scene 実装済み |
| `scripts/pasta/store.lua` | STORE.co_scene（コルーチン状態管理） | **既存充足** |
| `tests/lua_specs/act_impl_call_test.lua` | ACT_IMPL.call L3テスト | **テストパターン参考** — GLOBAL検索の既存テスト |
| `tests/lua_specs/integration_coroutine_test.lua` | E2E コルーチン統合テスト | **テストパターン参考** — EVENT.fire + yield の既存テスト |
| `tests/common/e2e_helpers.rs` | Rust E2Eテストヘルパー | **テストインフラ参考** — トランスパイル→実行パイプライン |

### 1.2 既存の設計パターン・規約

- **モジュール構造**: UPPER_CASE テーブル + `return MOD` パターン（lua-coding.md 準拠）
- **テスト配置**: Lua BDDテスト → `tests/lua_specs/*_test.lua`、Rust E2E → `tests/*_test.rs`
- **GLOBAL テーブル**: 現在は空テーブルのみ。ユーザーが `main.lua` で追加する設計
- **Luaモジュール読み込み順**: `main.lua`（ユーザー初期化）→ `scene_dic`（トランスパイル済みシーン）→ `finalize_scene`

### 1.3 統合サーフェス

- `ACT_IMPL.call` (act.lua:L313-L347): 4段階検索、Level 3 で `GLOBAL[key]` を参照
- `ACT_IMPL.yield` (act.lua:L289-L293): `self:build()` → `coroutine.yield(result)` → `return self`
- `EVENT.fire` (event/init.lua:L152-L181): handler結果がthreadならresume_until_valid + set_co_scene

---

## 2. 要件実現可能性分析

### 要件対アセットマップ

| 要件 | 既存アセット | ギャップ |
|------|-------------|---------|
| Req 1: GLOBAL関数登録・エイリアス・オーバーライド | `global.lua` 空テーブル、`ACT_IMPL.call` L3、`main.lua` 優先ロード | **Missing** — デフォルト関数（チェイントーク/yield）未定義。Call解決・コルーチン動作・オーバーライドは既存充足 |
| Req 2: ランタイム動作試験 | `act_impl_call_test.lua` パターン | **Missing** — Pasta DSL→トランスパイル→実行の試験なし |
| Req 3: EVENT.fire統合テスト | `integration_coroutine_test.lua` パターン | **Missing** — `＞チェイントーク` 固有の統合テストなし |

### 技術的制約

- **Luaモジュール読み込み順序**: `global.lua` は `act.lua` から `require("pasta.global")` で読み込まれる。デフォルト関数を `global.lua` 自体に定義すれば、最初のrequire時点で登録済みとなる
- **ユーザー上書き**: `main.lua` は `global.lua` より後に実行される。`GLOBAL.チェイントーク = custom_func` で上書き可能
- **コルーチンスタック**: `ACT_IMPL.call` は通常の関数呼び出しであり、新しいコルーチンを生成しない。GLOBAL関数内での `act:yield()` は呼び出し元コルーチンの `coroutine.yield()` として正常に動作する

### 複雑度シグナル

- **シンプルCRUD**: ✅ global.lua への関数追加のみ（実装2行）
- **テスト設計**: Lua BDDテスト + Rust E2E の2層テストが必要

---

## 3. 実装アプローチ選択肢

### Option A: global.lua 直接拡張（推奨）

**対象**: `scripts/pasta/global.lua` に直接デフォルト関数を定義

```
変更ファイル: global.lua（1ファイルのみ）
```

**変更内容**:
- `GLOBAL.チェイントーク` と `GLOBAL.yield` を定義（`act:yield()` を呼ぶだけ）
- 既存の空テーブル + return パターンを維持

**互換性**:
- ✅ 既存の `require("pasta.global")` は変更なし
- ✅ ユーザーの `main.lua` での上書きは自然に機能
- ✅ `ACT_IMPL.call` の L3 検索は変更不要

**トレードオフ**:
- ✅ 最小変更量（1ファイル、実質3行追加）
- ✅ 既存パターン完全準拠
- ✅ テスト容易（既存テストパターン流用可能）
- ❌ なし（リスク極小）

### Option B: 別モジュール分離

**対象**: 新規 `scripts/pasta/builtins.lua` にデフォルト関数を分離

**トレードオフ**:
- ✅ 責務分離が明確
- ❌ 新ファイル追加（不要な複雑化）
- ❌ 読み込み順制御が必要
- ❌ `global.lua` の「ユーザー定義の場所」という設計思想から逸脱

### Option C: Rust側からの登録

**対象**: `pasta_lua/runtime/` でRust関数としてGLOBALテーブルに登録

**トレードオフ**:
- ✅ Rust型安全性
- ❌ 過剰設計（Luaで2行の処理にRustバインディング不要）
- ❌ `global.lua` との責務境界があいまいに
- ❌ ユーザー上書きの仕組みが複雑化

---

## 4. テスト戦略分析

### Req 2: ランタイム動作試験

**既存パターン**: `runtime_e2e_test.rs` + `e2e_helpers.rs`

**実現方法**:
1. Pasta DSLフィクスチャ作成（`＞チェイントーク` を含むシーン）
2. `transpile()` → `lua.load()` → `finalize_scene()` → `SCENE.co_exec()` パイプライン
3. コルーチンresumeでyield前後の出力を検証

**ギャップ**: 既存の `e2e_helpers` で十分。追加インフラ不要。

### Req 3: EVENT.fire 統合テスト

**既存パターン**: `integration_coroutine_test.lua`（Lua BDDテスト）

**実現方法**: 2つの選択肢

**(A) Lua BDDテスト（推奨）**:
- `tests/lua_specs/` に新テストファイル追加
- `EVENT.fire` + GLOBAL関数 + コルーチン分割の検証
- 既存パターン `integration_coroutine_test.lua` をベースに拡張
- `init.lua` のspecsリストに追加

**(B) Rust E2Eテスト**:
- `tests/` に新Rustテストファイル追加
- Pasta DSL → トランスパイル → Lua VM実行 → EVENT.fire → コルーチン検証
- `e2e_helpers::create_runtime_with_search` を使用

**推奨**: テスト対象が純粋にLua層のロジックのため **(A) Lua BDD** が適切。Rust E2Eは Pasta DSL → トランスパイルのパイプライン検証（Req 2）に使用。

---

## 5. 実装複雑度・リスク評価

| 項目 | 評価 | 根拠 |
|------|------|------|
| **工数** | **S（1日以内）** | global.lua 3行追加 + テスト2ファイル作成 |
| **リスク** | **Low** | 既存パターン完全準拠、アーキテクチャ変更なし、既存テストインフラ流用可能 |

---

## 6. 設計フェーズへの推奨事項

### 推奨アプローチ: Option A（global.lua 直接拡張）

**理由**: 
- 変更量が最小（1ファイル3行）
- 既存の設計思想（GLOBALはユーザー拡張可能なテーブル）と完全に整合
- 全要件が既存インフラで充足可能

### 設計フェーズでの決定事項

1. **テスト配置**: Lua BDDテスト（Req 3）とRust E2Eテスト（Req 2）の具体的ファイル名・構成
2. **Pasta DSLフィクスチャ**: `＞チェイントーク` を含むテスト用 `.pasta` ファイルの設計
3. **ドキュメント更新**: LUA_API.md、GRAMMAR.md への反映範囲

### Research Needed

- なし（全技術要素は既存コードベースで確認済み）

