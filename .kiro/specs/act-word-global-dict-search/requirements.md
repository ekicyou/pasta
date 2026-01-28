# Requirements Document

## Project Description (Input)
`act:word(name)` のLua実装において、Rust側の `SEARCH:search_word()` API を呼び出すよう修正する。現在 `PROXY_IMPL.word()` にあるLua側の検索・シャッフル・キャッシュロジックを削除し、単語検索の責務をRust側（`@pasta_search` モジュール）に統一する。

---

## Introduction

本仕様は、PASTAスクリプトエンジンにおける単語検索アーキテクチャの整理を定義する。単語の検索・シャッフル・キャッシュはRust側（`WordTable`）の責務であり、Lua側はRust APIを呼び出すだけとする。既存の `PROXY_IMPL.word()` 内のLua検索ロジックを削除し、`SEARCH:search_word()` への委譲に置き換える。

## Requirements

### Requirement 1: ACT_IMPL.word による Rust API 呼び出し

**Objective:** PASTAスクリプト開発者として、`act:word(name)` から単語検索を行いたい。検索・シャッフル・キャッシュはRust側が担当する。

#### Acceptance Criteria

1. When `act:word(name)` が呼び出される, the ACT_IMPL shall 現在のシーンテーブルから `name` に完全一致するエントリを検索する
2. When シーンテーブルに完全一致で関数が見つかる, the ACT_IMPL shall 関数を `act` を引数として呼び出し、その戻り値を返す
3. When シーンテーブルに完全一致がない, the ACT_IMPL shall `GLOBAL[name]` から完全一致で検索する
4. When `GLOBAL[name]` に関数が見つかる, the ACT_IMPL shall 関数を `act` を引数として呼び出し、その戻り値を返す
5. When 完全一致がない, the ACT_IMPL shall `SEARCH:search_word(name, scene_name)` を呼び出して結果を返す
6. If `SEARCH:search_word` が `nil` を返す, then the ACT_IMPL shall `nil` を返す

### Requirement 2: PROXY_IMPL.word のLua検索ロジック削除

**Objective:** 単語検索の責務をRust側に統一し、Lua側の重複実装を削除する。

#### Acceptance Criteria

1. The PROXY_IMPL.word shall アクター完全一致検索（`actor[name]`）のみLua側で行う
2. When アクター完全一致で関数が見つかる, the PROXY_IMPL shall 関数を `act` を引数として呼び出し、その戻り値を返す
3. When アクター完全一致がない, the PROXY_IMPL shall `act:word(name)` を呼び出して結果を返す
4. The PROXY_IMPL shall `search_prefix_lua()` 関数を削除する
5. The PROXY_IMPL shall `math.random` による候補選択ロジックを削除する
6. The PROXY_IMPL shall `WORD.get_actor_words()` / `WORD.get_local_words()` / `WORD.get_global_words()` の呼び出しを削除する

### Requirement 3: アクター単語辞書のRust側収集（finalize.rs修正）

**Objective:** アクター単語辞書もRust側で検索できるようにする。

#### 現状
- ✅ Lua側: `WORD.get_all_words()` が `actor` を返却済み
- ✅ Rust側: `WordDefRegistry::register_actor()` API存在
- ✅ Rust側: `WordTable` がアクター単語検索対応済み（キー形式: `:__actor_xxx__:key`）
- ❌ `finalize.rs::collect_words()` が `all_words.actor` を処理していない

#### Acceptance Criteria

1. The finalize_scene::collect_words() shall `all_words.actor` からアクター単語辞書を収集する
2. The build_word_registry() shall 収集したアクター単語を `register_actor()` で登録する
3. The SEARCH:search_word(name, actor_scope) shall アクター名スコープでの検索をサポートする

### Requirement 4: 後方互換性の維持

**Objective:** 既存のPASTAスクリプトの動作を壊さないこと。

#### Acceptance Criteria

1. The ACT_IMPL shall 既存の `act:word()` 呼び出し箇所で従来と同等の動作を維持すること
2. The PROXY_IMPL shall 既存の `act.さくら:word("通常")` 呼び出し箇所で従来と同等の動作を維持すること
3. While 既存テストが存在する, the システム shall 全ての既存テストがパスすること

---

## 責務分担

| 責務                   | 担当             | 備考                  |
| ---------------------- | ---------------- | --------------------- |
| シーンテーブル完全一致 | Lua (ACT_IMPL)   | `scene[name]`         |
| GLOBAL完全一致         | Lua (ACT_IMPL)   | `GLOBAL[name]`        |
| アクター完全一致       | Lua (PROXY_IMPL) | `actor[name]`         |
| 前方一致検索           | Rust (SEARCH)    | `search_word()`       |
| ランダムシャッフル     | Rust (WordTable) | キャッシュ付き        |
| 関数値の解決           | Lua              | `value(act)` 呼び出し |

---

## 検索フロー

### act:word(name)
```
1. scene[name] 完全一致? → 関数なら実行、値なら返す
2. GLOBAL[name] 完全一致? → 関数なら実行、値なら返す
3. SEARCH:search_word(name, scene_name) → 結果を返す
4. nil を返す
```

### proxy:word(name)
```
1. actor[name] 完全一致? → 関数なら実行、値なら返す
2. act:word(name) へ委譲
```

---

## 削除対象コード

| ファイル                                                       | 削除対象                                    |
| -------------------------------------------------------------- | ------------------------------------------- |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua) | `search_prefix_lua()` 関数                  |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua) | `PROXY_IMPL.word()` 内の L2-L6 検索ロジック |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua) | `math.random` による候補選択                |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua) | `WORD.get_actor_words()` 等の呼び出し       |

---

## 関連ファイル

| ファイル                                                         | 役割                                       |
| ---------------------------------------------------------------- | ------------------------------------------ |
| [act.lua](../../../crates/pasta_lua/scripts/pasta/act.lua)       | ACT_IMPL 実装（修正対象）                  |
| [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua)   | PROXY_IMPL 実装（大幅削除・修正対象）      |
| [finalize.rs](../../../crates/pasta_lua/src/runtime/finalize.rs) | ファイナライズ処理（アクター辞書収集追加） |
| [context.rs](../../../crates/pasta_lua/src/search/context.rs)    | SEARCH モジュール実装                      |
