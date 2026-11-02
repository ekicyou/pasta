# Brief: ponytail-core-registry

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。0.4.5（2026-10-24 開発者の決定「ポニーテールは 0.0.1 後ろからの着手に変更」。経緯: 2026-10-19 に 0.4.3、2026-10-22 に版に縛らない隙間、2026-10-23 に 0.4.3）。0.4.3 の mlua 0.12 の後の main で測り直す。同じ監査の 10 本と同じウェーブで並走できる。

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

## 申し送り（2026-10-29 の discovery「シーン属性による候補の絞り込み」より）

- `call-attribute-filter` は閉じて、新しい spec `scene-condition-attributes` に統合した（開発者の決定）。この brief の `call-attribute-filter` は `scene-condition-attributes` と読み替える。
- 中身が変わった。条件は呼ぶ側（`＞シーン＆k＝v`）でなく、場面の側に書く（`＊雑談_今の部屋　＆＄＊経験値≧500`）。
  - 呼ぶ側の Call の文法（`call_scene`・`parse_action.rs`・`element_gen.rs` の Call）は触らない見込みである。
  - 触るのは、属性の文法、`scene.lua`・`act.lua` の検索の道、`search/context.rs`・`scene_table.rs`、イベントからシーンを起こす道（選ぶ前の自動転記）、`pasta_check validate`、LSP と VSCode の文法定義である。
- 参照: `.kiro/specs/scene-condition-attributes/brief.md`。

## 2026-11-02 棚卸の再測定（main 491cc23c）

- **前提の変化**
  - シーン表・単語表・シーンの登録（`crates/pasta_core/src/registry/`）は、前の測り直し（8fc9bdff）から変わっていない。監査の場所（`resolve_scene_id` 173〜225 行・`find_scene` 505〜522 行・`labels_iter` 429〜432 行・`history` 56 行・`fn_path` の 4 か所・単語表の `new` 55 行・`entries`/`clear_cache` 241〜249 行・`_filters` の呼び手は `&[]`）と、`NoMoreScenes` を書く match の腕（`search/context.rs` 160・182・224 行）もそのまま。
  - おすすめメニューの作業（`recommend-menu-scenes`）が、`error.rs` に別名の表の誤り 2 つ（本番で使う）を、`search/context.rs` に `scene_aliases` を足した。消す対象は変わらない。
  - `ponytail-lua-loader` が読み込みで `TranspileContext::merge_from` を呼ばなくなった。`SceneRegistry::merge_from`（`scene_registry.rs` 275 行）と `WordDefRegistry::merge_from`（`word_registry.rs` 108 行）の本番の呼び手は、呼び手 0 になったその関数（`pasta_lua/src/context.rs` 120 行。`ponytail-lua-transpiler` の持ち場）だけ。あちらが消せば、この 2 つはテストだけになる。後から入る側が消す（こちらが後ならこの spec が消す）。
  - README と `tech.md` の断り書きは `ponytail-lua-loader` が書いた（`README.md` 88 行・`tech.md` 178 行）。この spec は書かない。
- **触るファイル**: `scene_table.rs` 539・`word_table.rs` 258・`scene_registry.rs` 576・`word_registry.rs`（`merge_from` を消すなら）・`error.rs` 196・`lib.rs`・`Cargo.toml`、テスト 3 つ（`scene_table_candidate_tests.rs`・`scene_table_resolve_filter_tests.rs`・`tests/word_table_test.rs`）、`pasta_lua/src/search/context.rs`（1,022 行で 1,000 行を超えたまま。本番の部分は約 440 行）、`pasta_check/src/validate/scenes.rs` 181、ルートの `Cargo.toml`・`Cargo.lock`、`tech.md` 27 行、`crates/pasta_core/README.md` 92 行。新しいファイルは無い見込み。
- **規模**: 10〜14 タスク。
- **先に要るもの**: 機能の上では無い。`Cargo.lock`・ルートの `Cargo.toml` は `ponytail-editor-tooling`（席を持つ）・`pest-2-9-4-upgrade` と共有し、後から入る側が作り直す。ponytail のほかの 7 本とはソースが重ならない。0.5.0 以降の `scene-count`（2026-11-01 起票。`scene_table.rs`・`search/context.rs`）・`scene-attribute-store`（`scene_registry.rs`・`scene_table.rs`・`search/context.rs`）・`scene-condition-attributes`・`scene-pick-modes`・`scene-exact-match`（`scene_table.rs`・`search/context.rs`）、`hello-novel-sample`・`shell-element-click`・`cue-command-runtime`（`search/context.rs`）とは同じウェーブに置かない。
- **種別**: 基盤（作者に見える不具合は無い。読む人と直す人の手間を減らす）。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**: Current State 4 の「`fast_radix_trie` と `memchr` が消える」は誤り（`memchr` は pest なども使うので `Cargo.lock` に残る。消えるのは `fast_radix_trie` だけ）。`fast_radix_trie` を外すときは、`tech.md` と `crates/pasta_core/README.md` の依存の表のほか、`scene_table.rs` 179 行などの説明の「RadixMap」も直す。
