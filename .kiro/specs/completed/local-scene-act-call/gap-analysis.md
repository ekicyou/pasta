# Gap Analysis: local-scene-act-call

## 1. 現状調査

### 影響コンポーネント

| コンポーネント | ファイル | 役割 |
|---|---|---|
| **scope_gen（トランスパイラ）** | `crates/pasta_lua/src/code_gen/scope_gen.rs` L192-240 | **ローカルシーン Lua 関数名のコード生成（`__Name_N__` 命名の発生源）** |
| act:call 解決チェーン | `crates/pasta_lua/scripts/pasta/act.lua` L336-382 | 5段階ハンドラー検索 |
| SCENE.search | `crates/pasta_lua/scripts/pasta/scene.lua` L150-185 | `@pasta_search` を呼び出すLua側ラッパー |
| SearchContext | `crates/pasta_lua/src/search/context.rs` | Rust側検索ロジック（`search_scene`、`parse_fn_name`） |
| SceneTable | `crates/pasta_core/src/registry/scene_table.rs` | RadixMap前方一致検索（`collect_scene_candidates`、`fn_name_to_search_key`） |
| SceneRegistry | `crates/pasta_core/src/registry/scene_registry.rs` | シーン登録（`register_global_raw` がfinalize経路） |
| finalize_scene | `crates/pasta_lua/src/runtime/finalize.rs` | Lua側レジストリ収集 → Rust側SearchContext構築 |

### 根本原因の特定

#### 真の根本原因: トランスパイラの不要な `__` ラッピング

`scope_gen.rs` のローカルシーン名生成が問題の**唯一の発生源**：

```rust
// scope_gen.rs L198-202
let fn_name = if let Some(ref name) = scene.name {
    let sanitized = SceneRegistry::sanitize_name(name);
    format!("__{}_{}__", sanitized, counter)  // ← ここが全問題の根源
} else {
    "__start__".to_string()  // ← こちらは正しい（特殊エントリーポイント）
};
```

`__start__` は検索にマッチさせたくない特殊エントリーポイントとして `__` で囲む——これは意図的で正しい。しかし一般のローカルシーン（`・Head0` など）に同じ `__` ラッピングを適用する理由がない。`Head0_1` で十分であり、全経路がそのまま整合する。

#### データフロー全体追跡

**現状（壊れている）**:
```
DSL: ・Head0
  → scope_gen.rs: format!("__{}_{}__", "Head0", 1) = "__Head0_1__"
  → 生成Lua: function SCENE.__Head0_1__(act, ...) 
  → Luaテーブル: STORE.scenes["メイン1"]["__Head0_1__"] = func
  → collect_scenes: ("メイン1", "__Head0_1__")
  → register_global_raw: fn_name = "メイン1::__Head0_1__"
  → fn_name_to_search_key: ":メイン1:__Head0_1__"
  → 検索 ":メイン1:Head0" と前方一致しない → ❌
```

**修正後（正常動作）**:
```
DSL: ・Head0
  → scope_gen.rs: format!("{}_{}",  "Head0", 1) = "Head0_1"
  → 生成Lua: function SCENE.Head0_1(act, ...) 
  → Luaテーブル: STORE.scenes["メイン1"]["Head0_1"] = func
  → collect_scenes: ("メイン1", "Head0_1")
  → register_global_raw: fn_name = "メイン1::Head0_1"
  → fn_name_to_search_key: ":メイン1:Head0_1"
  → 検索 ":メイン1:Head0" と前方一致する → ✅
```

#### 2つのレジストリ構築経路（参考）

経路A（トランスパイル時レジストリ、テストで使用）と経路B（finalize時レジストリ、ランタイムで使用）のフォーマット不整合が直接的な不具合発生箇所だが、その原因はトランスパイラが `__` ラッピングされた名前を生成することにある。

- **経路A**: `register_local("Head0", ...)` → `fn_name = "OnMouseMove_1::Head0_1"` → ✅（元名でキー生成）
- **経路B**: `register_global_raw("OnMouseMove1", ["__Head0_1__"])` → `fn_name = "OnMouseMove1::__Head0_1__"` → ❌

修正後は Lua テーブルキーが `Head0_1` になるため、経路Bも自然に経路Aと同一フォーマットになる。

### `parse_fn_name` の連鎖的問題

`parse_fn_name`（context.rs）は `fn_name` の `local_part` を Lua 関数名に変換する：

```rust
fn parse_fn_name(fn_name: &str) -> (String, String) {
    if local_part == "__start__" {
        "__start__".to_string()
    } else {
        format!("__{}__", local_part)  // "Head0_1" → "__Head0_1__"
    }
}
```

現状の経路Bでは `local_part = "__Head0_1__"` → `format!("__{}__", "__Head0_1__")` = `"____Head0_1____"` とダブルマングリングが発生する。

修正後は `local_part = "Head0_1"` → `format!("__{}__", "Head0_1")` = `"__Head0_1__"` ——**ではなく**、Lua テーブルキーが `"Head0_1"` に変わるため、`parse_fn_name` の `__` 再ラッピングも不要になる。`local_part` をそのまま返せばよい。

### テスト状況

| テスト | 使用経路 | 結果 |
|---|---|---|
| `test_scene_search_local_search` (scene_search_test.rs) | 経路A（register_global + register_local） | ✅ Pass |
| `test_from_scene_registry_key_conversion` (scene_table_tests.rs) | 経路A | ✅ Pass |
| `test_resolve_scene_id_unified_local_scene` (scene_table_tests.rs) | 経路A | ✅ Pass |
| (ランタイム実行) | 経路B（register_global_raw via finalize） | ❌ Fail |

**ギャップ**: 経路Bを使用した統合テストが存在しない。

### 影響範囲の再評価

当初は「Luaコードブロック内からの `act:call` 動的呼び出し」に限定と分析していたが、誤り。DSL `＞` Call文もトランスパイラが `act:call(SCENE.__global_name__, "元名", {})` を生成するため、**すべてのローカルシーン呼び出しが影響を受ける**。

## 2. 要件実現可能性分析

| 要件 | 技術ニーズ | ギャップ |
|---|---|---|
| Req 1-1: Lua関数名から `__` 除去 | scope_gen.rs の format 修正 | **1行修正** |
| Req 1-2: `__start__` 維持 | 既存の条件分岐 | なし（現在も正しい） |
| Req 1-3: `parse_fn_name` 修正 | `__` 再ラッピングの削除 | **条件分岐修正** |
| Req 2: ローカルシーン検索正常化 | 上記修正で自動的に解決 | なし |
| Req 3: 互換性 | 既存テスト + スナップショット更新 | スナップショット更新が必要 |
| Req 4-1: E2Eインテグレーションテスト | DSL→トランスパイル→Lua実行→call解決の一気通貫テスト | **新規テスト作成** |
| Req 4-2: finalize経路テスト | `register_global_raw` 経由の検索テスト | **新規テスト作成** |
| Req 4-3: ラウンドトリップテスト | `collect_scenes` → `build_scene_registry` → `SceneTable` | **新規テスト作成** |

## 3. 実装アプローチ

### Option E: トランスパイラの `__` ラッピング除去（推奨）

**対象**:
1. `crates/pasta_lua/src/code_gen/scope_gen.rs` — `generate_local_scene()`
2. `crates/pasta_lua/src/search/context.rs` — `parse_fn_name()`

**修正内容**:

```rust
// scope_gen.rs: Before
format!("__{}_{}__", sanitized, counter)
// scope_gen.rs: After
format!("{}_{}",  sanitized, counter)

// context.rs parse_fn_name: Before
format!("__{}__", local_part)
// context.rs parse_fn_name: After
local_part.to_string()  // そのまま返す（__start__ は既存の条件分岐で処理済み）
```

**Trade-offs**:
- ✅ 根本原因を直接修正（不要な複雑さの除去）
- ✅ 修正箇所が明確で限定的（2関数、各1行）
- ✅ 経路A/Bのフォーマットが自然に統一される
- ✅ `fn_name_to_search_key`、`register_global_raw`、`build_scene_registry` の変更不要
- ✅ Level 1（Luaテーブル直接引き）、Level 2（RadixMap検索）の両方が同時に修正される
- ✅ `__start__` は既存の条件分岐で別扱いされるため影響なし
- ❌ トランスパイラのスナップショットテスト全更新が必要（`__XXX__` → `XXX` への一括置換）
- ❌ 既存のゴースト辞書から生成済みのLuaコード（キャッシュ等）との互換性確認が必要

### Option A〜D（参考: 議論の過程で検討した対症療法）

<details>
<summary>Option A〜D の詳細（折りたたみ）</summary>

#### Option A: `register_global_raw` でアンマングル（Rust側修正）

`register_global_raw` の `local_name` が `__...__` 形式の場合、`__` を除去してから `fn_name` を構成する。

- ✅ 修正箇所が1つ
- ❌ `raw` という名前のAPI内でマングル知識を持たせるのは設計的に不適切
- ❌ Luaテーブルキーは依然として `__...__` のまま（Level 1 は解決しない）

#### Option B: `fn_name_to_search_key` でアンマングル（SceneTable側修正）

検索キー生成時に `__...__` を除去する。

- ❌ `fn_name_to_search_key` + `parse_fn_name` の計2関数修正が必要
- ❌ `parse_fn_name` のダブルマングリング問題（fn_nameが `__...__` 付きのまま保持 → `____..____`）
- ❌ Luaテーブルキーとの不整合は解消されない

#### Option C: `collect_scenes`/`build_scene_registry` でアンマングル（finalize側修正）

Lua側から収集した `__Head0_1__` を `Head0_1` に変換してから `register_global_raw` に渡す。

- ✅ ドメイン境界での正規化（責務的には最も適切な対症療法）
- ❌ 結局 `parse_fn_name` の `__` 再ラッピングとのミスマッチを解決する必要がある
- ❌ Luaテーブルキーは依然 `__...__` のまま

#### Option D: Level 1 にローカルシーン逆引きを追加（Lua側修正）

`self.current_scene[key]` が `nil` の場合、テーブル走査でマッチを試みる。

- ❌ O(n) テーブル走査（パフォーマンス懸念）
- ❌ Level 2 の RadixMap 問題は未解決
- ❌ 前方一致検索が利用できない

</details>

## 4. 推奨

### 推奨アプローチ: Option E（トランスパイラの `__` ラッピング除去）

**理由**:

1. **根本原因の直接修正**: `__` ラッピングは `__start__` のために存在する命名規約であり、一般ローカルシーンに適用する合理性がない。不要な複雑さを取り除くだけで、全経路が既存ロジックのまま正しく動作する
2. **影響の一貫性**: Option A〜D はいずれも「マングルされたデータを下流で補正する」対症療法であり、複数箇所に散在する `__` 前提のコードとの整合性確認が必要。Option E は発生源で修正するため、下流の変更が不要
3. **Level 1 と Level 2 の同時修正**: Lua テーブルキーが `Head0_1` になるため、Level 1（テーブル直接引き）と Level 2（RadixMap検索）の両方が自然に解決される
4. **`parse_fn_name` の `__` 再ラッピング不要化**: `local_part` をそのまま返すだけでよくなり、ダブルマングリング問題が構造的に消滅する

### 設計フェーズでの確認事項

1. **スナップショットテストの一括更新**: `function SCENE.__XXX_N__` → `function SCENE.XXX_N` への一括置換。テスト数の把握と更新手順の計画
2. **finalize経路の統合テスト追加**: `register_global_raw` 経由のローカルシーン検索テストが欠落している
3. **既存生成済みLuaコードとの互換性**: pasta.dll アップデート時に既存ゴーストの再トランスパイルが必要かどうか

## 5. 複雑性・リスク評価

| 項目 | 評価 | 根拠 |
|---|---|---|
| 工数 | **S**（1-3日） | 2関数の各1行修正 + スナップショット更新 + テスト追加 |
| リスク | **Low** | 不要なコードの除去。`__start__` は条件分岐で保護済み |
