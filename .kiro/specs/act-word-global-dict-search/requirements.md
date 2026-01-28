# Requirements Document

## Project Description (Input)
`act:word(name)` のLua実装において、Rust側の `SEARCH:search_word()` API を呼び出すよう修正する。現在 `PROXY_IMPL.word()` にあるLua側の検索・シャッフル・キャッシュロジックを削除し、単語検索の責務をRust側（`@pasta_search` モジュール）に統一する。

**追加スコープ**: Rust側の自動フォールバック仕様（ローカル→グローバル）を廃止し、Lua側で個別にAPI呼び出しを行うことでフォールバックを制御する。

---

## Introduction

本仕様は、PASTAスクリプトエンジンにおける単語・シーン検索アーキテクチャの整理を定義する。

**主要な変更点:**
1. 単語の検索・シャッフル・キャッシュはRust側（`WordTable`/`SceneTable`）の責務
2. Rust側の自動フォールバック（ローカル→グローバル）を廃止
3. Lua側で個別にスコープ指定してAPI呼び出し、フォールバック制御はLua側の責務
4. 既存の `PROXY_IMPL.word()` 内のLua検索ロジックを削除し、`SEARCH:search_word()` への委譲に置き換える

## Requirements

### Requirement 1: SEARCH API仕様変更（フォールバック廃止）

**Objective:** Rust側の自動フォールバック仕様を廃止し、Lua側でスコープを明示的に指定できるようにする。

#### Acceptance Criteria（search_word）

1. When `SEARCH:search_word(key, nil)` が呼び出される, the API shall グローバル単語辞書のみを前方一致検索する
2. When `SEARCH:search_word(key, scene_name)` が呼び出される（scene_name非nil）, the API shall 指定されたシーン/アクターのローカル単語辞書のみを前方一致検索する
3. The API shall ローカル→グローバルの自動フォールバックを行わない
4. The WordTable::collect_word_candidates() shall `module_name` が空の場合グローバルのみ、非空の場合ローカルのみを検索する

#### Acceptance Criteria（search_scene）

5. When `SEARCH:search_scene(key, nil)` が呼び出される, the API shall グローバルシーンのみを前方一致検索する
6. When `SEARCH:search_scene(key, parent_scene)` が呼び出される（parent_scene非nil）, the API shall 指定された親シーンのローカルシーンのみを前方一致検索する
7. The API shall ローカル→グローバルの自動フォールバックを行わない
8. The SceneTable::collect_scene_candidates() shall `module_name` が空の場合グローバルのみ、非空の場合ローカルのみを検索する

### Requirement 2: ACT_IMPL.word による Rust API 呼び出し

**Objective:** PASTAスクリプト開発者として、`act:word(name)` から単語検索を行いたい。検索・シャッフル・キャッシュはRust側が担当し、フォールバックはLua側で制御する。

#### Acceptance Criteria

1. When `act:word(name)` が呼び出される, the ACT_IMPL shall 現在のシーンテーブルから `name` に完全一致するエントリを検索する
2. When シーンテーブルに完全一致で値が見つかる, the ACT_IMPL shall `WORD.resolve_value(value, self)` で値を解決して返す
3. When シーンテーブルに完全一致がない, the ACT_IMPL shall `GLOBAL[name]` から完全一致で検索する
4. When `GLOBAL[name]` に値が見つかる, the ACT_IMPL shall `WORD.resolve_value(value, self)` で値を解決して返す
5. When 完全一致がない, the ACT_IMPL shall `SEARCH:search_word(name, scene_name)` を呼び出してシーンローカル辞書を検索する
6. When シーンローカル辞書に一致がない, the ACT_IMPL shall `SEARCH:search_word(name, nil)` を呼び出してグローバル辞書を検索する
7. If グローバル辞書にも一致がない, then the ACT_IMPL shall `nil` を返す

### Requirement 3: PROXY_IMPL.word のLua検索ロジック削除

**Objective:** 単語検索の責務をRust側に統一し、Lua側の重複実装を削除する。フォールバックはLua側で制御する。

#### Acceptance Criteria

1. The PROXY_IMPL.word shall アクター完全一致検索（`actor[name]`）のみLua側で行う
2. When アクター完全一致で値が見つかる, the PROXY_IMPL shall `WORD.resolve_value(value, self.act)` で値を解決して返す
3. When アクター完全一致がない, the PROXY_IMPL shall `SEARCH:search_word(name, "__actor_" .. actor.name .. "__")` を呼び出してアクター辞書を検索する
4. When アクター辞書に一致がある, the PROXY_IMPL shall `WORD.resolve_value(value, self.act)` で値を解決して返す
5. When アクター辞書に一致がない, the PROXY_IMPL shall `act:word(name)` を呼び出して結果を返す（シーン→グローバルのフォールバックは `act:word` 内で実行される）
6. The PROXY_IMPL shall actor.lua の `search_prefix_lua()` 関数を削除する
7. The PROXY_IMPL shall actor.lua の `resolve_value()` 関数を削除する（pasta.word に移動）
8. The PROXY_IMPL shall `math.random` による候補選択ロジックを削除する
9. The PROXY_IMPL shall `WORD.get_actor_words()` / `WORD.get_local_words()` / `WORD.get_global_words()` の呼び出しを削除する

### Requirement 4: アクター単語辞書のRust側収集（finalize.rs修正）

**Objective:** アクター単語辞書もRust側で検索できるようにする。

#### 現状
- ✅ Lua側: `WORD.get_all_words()` が `actor` を返却済み
- ✅ Rust側: `WordDefRegistry::register_actor()` API存在
- ✅ Rust側: `WordTable` がアクター単語検索対応済み（キー形式: `:__actor_xxx__:key`）
- ❌ `finalize.rs::collect_words()` が `all_words.actor` を処理していない

#### Acceptance Criteria

1. The finalize_scene::collect_words() shall `all_words.actor` からアクター単語辞書を収集する
2. The build_word_registry() shall 収集したアクター単語を `register_actor()` で登録する

### Requirement 5: WORD.resolve_value() 実装

**Objective:** 完全一致検索時の値解決ロジックを共通化し、ACT_IMPL.word と PROXY_IMPL.word で再利用する。

#### Acceptance Criteria

1. The pasta.word module shall `resolve_value(value, act)` 関数をエクスポートする
2. When `value` が `nil`, the function shall `nil` を返す
3. When `value` が関数, the function shall `value(act)` を実行してその戻り値を返す
4. When `value` が配列（table with #value > 0）, the function shall 最初の要素 `value[1]` を返す
5. When `value` がその他の型, the function shall `tostring(value)` を返す
6. The function shall ACT_IMPL.word と PROXY_IMPL.word から呼び出し可能である

### Requirement 6: 後方互換性の維持

**Objective:** 既存のPASTAスクリプトの動作を壊さないこと。

#### Acceptance Criteria

1. The ACT_IMPL shall 既存の `act:word()` 呼び出し箇所で従来と同等の動作を維持すること
2. The PROXY_IMPL shall 既存の `act.さくら:word("通常")` 呼び出し箇所で従来と同等の動作を維持すること
3. While 既存テストが存在する, the システム shall 全ての既存テストがパスすること

---

## SEARCH API仕様（フォールバック廃止後）

### search_word(key, scope)

| scope             | 検索対象                   | キー形式                      |
| ----------------- | -------------------------- | ----------------------------- |
| `nil`             | グローバル単語辞書のみ     | `key` 前方一致                |
| `scene_name`      | シーンローカル単語辞書のみ | `:scene_name:key` 前方一致    |
| `"__actor_xxx__"` | アクター単語辞書のみ       | `:__actor_xxx__:key` 前方一致 |

### search_scene(key, scope)

| scope          | 検索対象                     | キー形式                     |
| -------------- | ---------------------------- | ---------------------------- |
| `nil`          | グローバルシーンのみ         | `key` 前方一致               |
| `parent_scene` | 親シーンのローカルシーンのみ | `:parent_scene:key` 前方一致 |

---

## 責務分担

| 責務                   | 担当             | 備考                  |
| ---------------------- | ---------------- | --------------------- |
| シーンテーブル完全一致 | Lua (ACT_IMPL)   | `scene[name]`         |
| GLOBAL完全一致         | Lua (ACT_IMPL)   | `GLOBAL[name]`        |
| アクター完全一致       | Lua (PROXY_IMPL) | `actor[name]`         |
| 前方一致検索           | Rust (SEARCH)    | `search_word()`       |
| ランダムシャッフル     | Rust (WordTable) | キャッシュ付き        |
| **フォールバック制御** | **Lua**          | 個別API呼び出し       |
| 関数値の解決           | Lua              | `value(act)` 呼び出し |

---

## 検索フロー

### act:word(name)
```
1. scene[name] 完全一致? → 関数なら実行、値なら返す
2. GLOBAL[name] 完全一致? → 関数なら実行、値なら返す
3. SEARCH:search_word(name, scene_name) → シーンローカル辞書検索
4. SEARCH:search_word(name, nil) → グローバル辞書検索
5. nil を返す
```

### proxy:word(name)
```
1. actor[name] 完全一致? → 関数なら実行、値なら返す
2. SEARCH:search_word(name, "__actor_" .. actor.name .. "__") → アクター辞書検索
3. act:word(name) へ委譲（シーン→グローバルのフォールバックは内部で実行）
```

---

## 削除対象コード

| ファイル                                                       | 削除対象                                                   |
| -------------------------------------------------------------- | ---------------------------------------------------------- |
| `crates/pasta_lua/scripts/pasta/actor.lua`                     | `search_prefix_lua()` 関数                                 |
| `crates/pasta_lua/scripts/pasta/actor.lua`                     | `resolve_value()` 関数（pasta.word に移動）                |
| `crates/pasta_lua/scripts/pasta/actor.lua`                     | `PROXY_IMPL.word` L2-L6 検索ロジック（`math.random` 含む） |
| `crates/pasta_core/src/registry/word_table.rs`                 | `collect_word_candidates` Step 2 グローバルフォールバック  |
| `crates/pasta_core/src/registry/scene_table.rs`                | `collect_scene_candidates` Step 2 グローバルフォールバック |
| `crates/pasta_core/tests/*`                                    | フォールバック関連テスト（詳細は design.md 参照）          |
| -------------------------------------------------------------- | -------------------------------------------                |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua) | `search_prefix_lua()` 関数                                 |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua) | `PROXY_IMPL.word()` 内の L2-L6 検索ロジック                |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua) | `math.random` による候補選択                               |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua) | `WORD.get_actor_words()` 等の呼び出し                      |

---

## Rust側修正対象

| ファイル                                                                 | 修正内容                                              |
| ------------------------------------------------------------------------ | ----------------------------------------------------- |
| [word_table.rs](../../../crates/pasta_core/src/registry/word_table.rs)   | `collect_word_candidates()` の自動フォールバック廃止  |
| [scene_table.rs](../../../crates/pasta_core/src/registry/scene_table.rs) | `collect_scene_candidates()` の自動フォールバック廃止 |
| [finalize.rs](../../../crates/pasta_lua/src/runtime/finalize.rs)         | アクター単語辞書収集追加                              |

---

## 関連ファイル

| ファイル                                                         | 役割                                       |
| ---------------------------------------------------------------- | ------------------------------------------ |
| [act.lua](../../../crates/pasta_lua/scripts/pasta/act.lua)       | ACT_IMPL 実装（修正対象）                  |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua)   | PROXY_IMPL 実装（大幅削除・修正対象）      |
| [finalize.rs](../../../crates/pasta_lua/src/runtime/finalize.rs) | ファイナライズ処理（アクター辞書収集追加） |
| [context.rs](../../../crates/pasta_lua/src/search/context.rs)    | SEARCH モジュール実装                      |
