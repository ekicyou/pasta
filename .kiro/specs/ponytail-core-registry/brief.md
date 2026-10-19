# Brief: ponytail-core-registry

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。0.4.3 に入れる（2026-10-19 開発者の決定）。同じ監査の 10 本と同じウェーブで並走できる。

## 進め方（2026-10-19 開発者の決定）

- 下の「Current State」は、2026-10-19 の監査の控え（出発点）である。
- **ギャップ分析**で、この spec の領域（`crates/pasta_core`）に `/ponytail:ponytail-audit` をやり直す。控えと突き合わせ、違いをギャップ分析の報告に書く。
- 要件は「その領域の監査の指摘が、残らず直っている」ことと、決めごとにする。
- **修正は設計以降**で行う。

## Problem

`pasta_core` のシーン表と単語表に、本番では誰も使わない API・誤りの型・依存がある。読む人と直す人の手間が増える。ゴーストの作者に見える不具合は無い。

## Current State

2026-10-19 の監査で確かめた。`cargo test -p pasta_core` は通り、clippy も 0 件。

1. **使われない SceneTable・SceneRegistry の API**
   - 場所（`crates/pasta_core/src/registry/scene_table.rs`）：
     - L169-209 `resolve_scene_id`
     - L449-466 `find_scene`（Lua の `find_scene` とは別物）
     - L52 `CachedSelection.history`（書くだけで読まない）
     - L373-381 `labels_iter`（テストだけ）
   - `scene_registry.rs` の `fn_path` は、4 か所で書いて読まない。
   - `from_scene_registry` は決して `Err` にならない `Result` を返す。そのため `crates/pasta_check/src/validate/scenes.rs:106` が `.expect` している。
2. **作られない誤りの variant**
   - 場所：`crates/pasta_core/src/error.rs` L10-60。
   - 中身：`DuplicateSceneName`・`DuplicateScenePath`・`NoMoreScenes`・`SceneTableResult`・`WordTableResult`。
   - `NoMoreScenes` を書いているのは、`crates/pasta_lua/src/search/context.rs:146`・`:168` の match の腕だけ。
3. **WordTable の死んだ API**
   - 場所（`crates/pasta_core/src/registry/word_table.rs`）：
     - L54-63 `new`（呼び手 0）
     - L187-192 `_filters`（「将来のため」。唯一の呼び手が `&[]`）
     - L241-249 `entries`・`clear_cache`（テストだけ）
   - `WordCacheKey` は pub で再輸出されているが、中だけで使う。
4. **`fast_radix_trie` は前方一致の列挙にしか使わない**
   - 場所：`scene_table.rs` の L9・L60、`word_table.rs` の L9・L44。
   - 使うのは `iter_prefix`・`get`・`entry` だけ。`BTreeMap<String, Vec<_>>` の `.range(prefix..).take_while(starts_with)` で同じ O(log n + k) になり、`fast_radix_trie` と `memchr` が消える。キーの順（バイト順）も同じ。

## Desired Outcome

領域の監査の指摘が残らず直り、`from_scene_registry` が `Self` を返す。テストは全部通り、シーンと単語の選ばれ方は変わらない。

## Approach

監査が示す最小の直し（削除を優先する）を採る。`fast_radix_trie` は、標準の `BTreeMap` に替える。呼び手（`pasta_lua/src/search/context.rs`・`pasta_check/src/validate/scenes.rs`）は、同じ変更で合わせる。

## Scope

- **In**: `crates/pasta_core` の監査の指摘（上の 1〜4 と、ギャップ分析で増えたもの）。
- **Out**: シーンの属性の絞り込みの仕組みは残す（2026-10-19 開発者の決定）。対象は `filter_by_attributes`・`SceneCacheKey.filters`・`filters` の引数・`NoMatchingScene`・`attributes` の欄。0.5.0 の `call-attribute-filter`・`scene-attribute-store` が使うため。

## Boundary Candidates

- シーン表・シーンの登録の API の削り
- 単語表の API の削り
- 誤りの型の削り
- 前方一致の入れ物を `BTreeMap` に替える

## Out of Boundary

- 属性の絞り込み（上の Out）
- `crates/pasta_lua/src/context.rs`（`ponytail-lua-transpiler` が持つ）

## Upstream / Downstream

- **Upstream**: なし
- **Downstream**:
  - `pasta_lua` の検索（`search/context.rs`）と finalize
  - `pasta_check` の `validate/scenes.rs`

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `scene-attribute-store`・`call-attribute-filter`（0.5.0）
  - `ponytail-editor-tooling`（`Cargo.lock` とルートの `Cargo.toml` を共有する。席は `ponytail-editor-tooling` が持つ。この spec が後から入るなら、rebase で `Cargo.lock` を作り直す）

## Constraints

- 公開クレートの pub API を消してよい（2026-10-19 開発者の決定）。README と `tech.md` の断り書きは、10 本のうち先に入る 1 本が 1 回だけ書く。
- `tech.md` の依存の表から `fast_radix_trie` を外す。
- 振る舞いを固めるテストを先に書き、1 つ直すごとに確かめて 1 コミットにする。
- 品質の関門：`cargo fmt --check`、clippy（警告 0）、`cargo test -p pasta_core -p pasta_lua -p pasta_check`。cargo の前に `NoDefaultCurrentDirectoryInExePath` を外す。
