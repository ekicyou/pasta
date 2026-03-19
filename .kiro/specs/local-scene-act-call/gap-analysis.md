# Gap Analysis: local-scene-act-call

## 1. 現状調査

### 影響コンポーネント

| コンポーネント | ファイル | 役割 |
|---|---|---|
| act:call 解決チェーン | `crates/pasta_lua/scripts/pasta/act.lua` L336-382 | 5段階ハンドラー検索 |
| SCENE.search | `crates/pasta_lua/scripts/pasta/scene.lua` L150-185 | `@pasta_search` を呼び出すLua側ラッパー |
| SearchContext | `crates/pasta_lua/src/search/context.rs` | Rust側検索ロジック（`search_scene`、`parse_fn_name`） |
| SceneTable | `crates/pasta_core/src/registry/scene_table.rs` | RadixMap前方一致検索（`collect_scene_candidates`、`fn_name_to_search_key`） |
| SceneRegistry | `crates/pasta_core/src/registry/scene_registry.rs` | シーン登録（`register_global_raw` がfinalize経路） |
| finalize_scene | `crates/pasta_lua/src/runtime/finalize.rs` | Lua側レジストリ収集 → Rust側SearchContext構築 |
| code_gen / scope_gen | `crates/pasta_lua/src/code_gen/scope_gen.rs` | ローカルシーン名マングリング（`__Name_N__` 生成） |

### 根本原因の特定

**2つのレジストリ構築経路のフォーマット不整合**が根本原因。

#### 経路A: トランスパイル時レジストリ（テストで使用、正常動作）

```
register_local("Head0", "OnMouseMove", 1, 1, ...)
  → fn_name = "OnMouseMove_1::Head0_1"
  → fn_name_to_search_key → ":OnMouseMove_1:Head0_1"
  → 検索 ":OnMouseMove_1:Head0" は ":OnMouseMove_1:Head0_1" の前方一致 → ✅ マッチ
```

#### 経路B: finalize時レジストリ（ランタイムで使用、バグ発生）

```
collect_scenes(lua) → ("OnMouseMove1", "__Head0_1__")
build_scene_registry → register_global_raw("OnMouseMove1", ["__Head0_1__"], ...)
  → fn_name = "OnMouseMove1::__Head0_1__"
  → fn_name_to_search_key → ":OnMouseMove1:__Head0_1__"
  → 検索 ":OnMouseMove1:Head0" は ":OnMouseMove1:__Head0_1__" の前方一致ではない → ❌ 不一致
```

**問題の本質**: `register_global_raw` は Lua側から収集したマングル済み名（`__Head0_1__`）をそのまま `fn_name` に埋め込む。しかし `fn_name_to_search_key` は `name_counter` 形式（`Head0_1`）を前提としているため、`__` プレフィックスが前方一致検索を阻害する。

### テスト状況

| テスト | 使用経路 | 結果 |
|---|---|---|
| `test_scene_search_local_search` (scene_search_test.rs) | 経路A（register_global + register_local） | ✅ Pass |
| `test_from_scene_registry_key_conversion` (scene_table_tests.rs) | 経路A | ✅ Pass |
| `test_resolve_scene_id_unified_local_scene` (scene_table_tests.rs) | 経路A | ✅ Pass |
| (ランタイム実行) | 経路B（register_global_raw via finalize） | ❌ Fail |

**ギャップ**: 経路Bを使用した統合テストが存在しない。テストはすべて経路Aで構築されるため、finalize経由のマングル名不整合を検出できていない。

### Level 1 (current_scene テーブル引き) の追加分析

Level 2のRadixMap問題とは別に、Level 1も独立に失敗する：

```lua
self.current_scene[key]  -- self.current_scene["Head0"]
-- テーブルには __Head0_1__ というキーしか存在しない → nil
```

Level 1はLua側のテーブル直接参照であり、RadixMapは関与しない。act.luaの Level 1 に元名→マングル名の変換ロジックが存在しない。

## 2. 要件実現可能性分析

| 要件 | 技術ニーズ | ギャップ |
|---|---|---|
| Req 1-1: 元名でローカルシーン解決 | RadixMap検索キーから `__...__` を取り除く / Level 1逆引き | **Missing**: finalize経路のキー変換が未実装 |
| Req 1-2: 同名重複のランダム選択 | 既存の `CachedSelection` ロジック再利用 | なし（RadixMap問題解決後は自動的に動作） |
| Req 1-3: マングル済み名の直接解決 | Level 1の既存ロジック | なし（現在も動作する） |
| Req 1-4: 未発見時エラーログ | 既存のエラーログ出力 | なし |
| Req 2-1〜4: 互換性 | 既存テスト + リグレッションテスト | なし（修正が限定的であれば） |
| Req 3-1: 前方一致検索 | RadixMap検索キー修正 | **Missing**: 経路Bのキー変換 |
| Req 3-2: 完全一致優先 | RadixMapの前方一致は完全一致も包含 | なし（候補数1のとき自動的に完全一致） |

## 3. 実装アプローチ

### Option A: `register_global_raw` でアンマングル（Rust側修正）

**対象**: `crates/pasta_core/src/registry/scene_registry.rs` の `register_global_raw`

`local_name` が `__...__` 形式の場合、`__` プレフィックスとサフィックスを除去してから `fn_name` を構成する。

```rust
// Before:
let local_fn_name = format!("{}::{}", full_name, local_name);

// After:
let unmangled = unmangle_local_name(local_name);
let local_fn_name = format!("{}::{}", full_name, unmangled);
```

**Trade-offs**:
- ✅ 修正箇所が1つ（`register_global_raw`のみ）
- ✅ `fn_name_to_search_key` と `parse_fn_name` の既存ロジックと整合
- ✅ 経路Aとフォーマットが統一される
- ❌ `name` フィールドも修正する必要あり（現在マングル名が入っている）
- ❌ `parse_fn_name` が `__start__` 以外の `__...__` を受け取る前提のロジックがあるため、アンマングル後に `parse_fn_name` が `__...__` を再付与する箇所との整合性確認が必要

### Option B: `fn_name_to_search_key` でアンマングル（SceneTable側修正）

**対象**: `crates/pasta_core/src/registry/scene_table.rs` の `fn_name_to_search_key`

ローカルシーンの検索キー生成時に `__...__` を除去する。

```rust
fn fn_name_to_search_key(fn_name: &str, is_local: bool) -> String {
    if is_local {
        let parts: Vec<&str> = fn_name.split("::").collect();
        let local_part = unmangle_if_needed(parts[1]); // __Head0_1__ → Head0_1
        format!(":{}:{}", parts[0], local_part)
    } else { ... }
}
```

**Trade-offs**:
- ~~✅ 修正箇所が1つ~~ → ❌ `fn_name_to_search_key` + `parse_fn_name` の計2関数修正が必要（下記参照）
- ✅ 経路A・Bの両方とも正しく機能する（経路Aは元から `__` なし）
- ❌ `fn_name_to_search_key` がマングル規則を知る必要がある
- ❌ `parse_fn_name`（context.rs）が `fn_name` の local_part に `format!("__{}__", local_part)` を適用するため、fn_name が `"OnMouseMove1::__Head0_1__"` のまま保持されると `"____Head0_1____"` というダブルマングリングが発生する。Option B 単独では search 成功後の Lua 名解決が失敗するため、`parse_fn_name` にもアンマングル判定の追加が必要

### Option C: `collect_scenes` でアンマングル（finalize側修正）

**対象**: `crates/pasta_lua/src/runtime/finalize.rs` の `collect_scenes` または `build_scene_registry`

Lua側から収集した `__Head0_1__` を `Head0_1` に変換してから `register_global_raw` に渡す。

**Trade-offs**:
- ✅ 変換責務がfinalize層に集約される
- ✅ `register_global_raw` + SceneTable のコードに変更不要
- ❌ `parse_fn_name`（context.rs）が返すLua向け名前 `__Head0_1__` の再マングルロジックとの整合性確認が必要
- ❌ `register_global_raw` の `name` フィールドも影響を受ける

### Option D: Level 1 にローカルシーン逆引きを追加（Lua側修正）

**対象**: `crates/pasta_lua/scripts/pasta/act.lua` の Level 1

`self.current_scene[key]` が `nil` の場合、`self.current_scene` テーブルを走査して `__key_N__` パターンのマッチを試みる。

**Trade-offs**:
- ✅ Rust側変更不要
- ❌ O(n) テーブル走査が発生（パフォーマンス懸念）
- ❌ Level 2 の RadixMap 問題は未解決のまま
- ❌ 前方一致検索が利用できない
- ❌ Level 2と重複するロジックが生まれる

## 4. 推奨

### 推奨アプローチ: Option B（`fn_name_to_search_key` でアンマングル）

**理由**:

1. ~~**影響範囲最小**~~: `fn_name_to_search_key` の修正だけでは不十分。`parse_fn_name` も修正が必要（計2関数）。詳細は Option B の trade-offs を参照
2. **`parse_fn_name` との自然な対称性**: `parse_fn_name` は `local_part` → `__local_part__` のマングルを行う（出力方向）。`fn_name_to_search_key` はその逆（検索キー方向）を行えば対称的。ただし **fn_name 自体がマングル済みの場合、`parse_fn_name` がダブルマングリングする問題あり**
3. **既存テスト互換**: 経路Aは元から `__` なしなので、テスト結果に変更なし
4. ~~**`register_global_raw` のfn_name保存フォーマット不変**~~: fn_name が `__...__` 付きのまま保持されると `parse_fn_name` がダブルマングリング（`____Head0_1____`）を起こす。Option A/C のように fn_name 自体をアンマングルすればこの問題は発生しない

### 設計フェーズでの確認事項

1. **アンマングル規則の厳密な定義**: `__X__` → `X` の変換が `__start__` と衝突しないことの確認（`__start__` は is_local=false で処理されるため問題ないはず）
2. **Level 1 対応の要否**: Level 1（Luaテーブル直接引き）もアンマングル対応する場合は、Option Dとのハイブリッドが必要。ただしLevel 2で解決可能であればLevel 1修正は不要
3. **finalize経路の統合テスト追加**: `register_global_raw` 経由のローカルシーン検索テストが欠落している

## 5. 複雑性・リスク評価

| 項目 | 評価 | 根拠 |
|---|---|---|
| 工数 | **S**（1-3日） | 既存パターンの修正、影響範囲が限定的 |
| リスク | **Low** | 既知の技術、明確なスコープ、テストで検証可能 |
