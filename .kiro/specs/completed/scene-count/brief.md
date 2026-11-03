# Brief: scene-count

> **ステータス**: 未着手（2026-11-01 の `/kiro-discovery` で起票した。里々の「文「○○」の数」から、開発者が「pasta にもシーン名ごとのトークの数を数える方法が欲しい」と依頼した）。版は 0.5.0「属性関係の整備」の 4 番（2026-11-01 開発者の決定「版は属性関係のリリースに混ぜましょう」）。属性は使わないので、先に要るものは無い。要件定義は Opus。

## Problem

作者は、あるシーン名にいくつのトーク（同名のシーン）がぶら下がっているかを、辞書の中から知る方法を持っていない。

- 里々では、情報取得変数の `（文「一般トーク」の数）` でトーク数が分かる。メニューに「話題 120 個」と出したり、話題が残っているかで分岐したりできる。
- pasta では、同名のシーンを何本も書いて `＞名前` でランダムに選ばせるのが基本の書き方である。それなのに、その数を台詞にも分岐にも使えない。
- Lua から数えようとしても、`STORE.scenes` は関数の実体（`雑談_1`・`雑談_2`…）の表で、`＞` の前方一致・別名表の規則と同じ数え方を作者が再現することになる。

## Current State

2026-11-01（main `6aabf087`）に確かめた。

- **候補を集める部品はもうある**:
  - 前方一致: `SceneTable::collect_scene_candidates(module_name, prefix)`（`crates/pasta_core/src/registry/scene_table.rs` 458 行）。`resolve_scene_id_unified` が `＞名前` の候補を集めるのに使い、`pasta_check` の `validate/references.rs` も使っている。`module_name` が空ならグローバルシーンだけ（`:` で始まるローカルのキーを外す）。
  - 完全一致: `SceneTable::resolve_scene_id_exact`（255 行）が `prefix_index.get(名前)` で同じ名前の候補だけを集める。集めた後に 1 つ選ぶので、数だけを返す口は無い。
  - Lua への口: `SearchContext::search_scene`／`search_scene_exact`（`crates/pasta_lua/src/search/context.rs` 105・119 行。共通の本体は `search_scene_by`）。グローバル検索のときだけ、別名表を当ててから（`会話` → `OnTalk`）`SceneRegistry::sanitize_name` で正規化する（166〜168 行）。メソッドの登録は同じファイルの 357 行から（`search_scene`・`search_scene_exact`・`search_local_scene_exact`・`search_word`・`scene_aliases` など）。
- **DSL から呼べる組み込み関数の置き方**: `GLOBAL` に英語名と日本語名の両方を置く（`crates/pasta_lua/pasta_scripts/pasta/shiori/entry.lua` 35 行の `GLOBAL["自動トーク停止"] = GLOBAL.pause_auto_talk` など）。`＠名前（引数）` の関数呼び出しで、アクション行・変数代入の右辺・式の中から呼べ、戻り値をそのまま使える（`book/src/grammar/words.md` の「動的関数呼び出し」・`grammar/call-jump.md` 265 行の 4 段目）。
- **辞書の検査**: `pasta_check validate` が知っている GLOBAL 関数の名前は `crates/pasta_check/src/validate/builtins.rs` にある。ここに無い名前を `＞`・`＠…（…）` で呼ぶと、無い関数として注意が出る。マニュアルの一覧は `book/src/debug/validate.md` 170 行。

## Desired Outcome

作者が、シーン名にぶら下がるシーンの数を、`＞` と同じ規則の前方一致と、完全一致の 2 通りで読める。

```pasta
＊雑談          ＃ 3 本
＊雑談          
＊雑談          
＊雑談_朝       ＃ 2 本
＊雑談_朝       

＊OnMouseDoubleClick
　＄n＝＠シーン数（「雑談」）               ＃ 5（＞雑談 が選ぶ候補の数）
　＄m＝＠シーン数完全一致（「雑談」）       ＃ 3（名前がちょうど「雑談」のもの）
　女の子：＠通常　話題は＄n　個あるよ。
```

## Approach

2026-11-01 に開発者が承認した（推奨案を全部採り、前方一致と完全一致は関数名で分けると決めた）。

1. **API の形**: GLOBAL 関数を 2 つ置く。英語名と日本語名の両方を持たせる。
   - `＠シーン数（名前）`／`scene_count`: 前方一致
   - `＠シーン数完全一致（名前）`／`scene_count_exact`: 完全一致

   引数で切り替える案（`＠シーン数（「雑談」、「完全」）`）は採らない。エンジンの側の `search_scene`／`search_scene_exact` の分け方と揃え、書いた行を見ただけで分かるようにする。エンジンが値を入れる変数（`＄トーク数`）の案は、どのシーンの数かを指定できないので採らない。里々のような専用の構文も、文法・LSP・VSCode まで手が入るので採らない。
2. **数え方は Call と同じ**:
   - `シーン数` は `＞名前` が選ぶ候補の数、`シーン数完全一致` は完全一致の検索（`search_scene_exact`。後の `＞＞名前`）が選ぶ候補の数と同じにする。
   - グローバルシーンだけを数える。実行中のシーンのローカルシーンは数えない。
   - 別名表を当ててから正規化する（`search_scene_by` と同じ順）。`＠シーン数（「会話」）` は `OnTalk` で前方一致に数えるので、`＊会話` と `＊OnTalk〜` が数に入り、`＊会話_朝` は入らない（`＞会話` と同じ）。
3. **条件の属性**（0.5.0 の `scene-condition-attributes`）: 今は「定義されている数」を返す。条件で今は選ばれないシーンも数に入れる。「今出せる数」を足すかは、`scene-condition-attributes` の側で決める（その brief に申し送りを書いた）。
4. **見つからないとき・書き間違い**:
   - 一致するシーンが無ければ `0` を返す。失敗にしない（「話題がもう無い」を分岐に使える）。
   - 引数が文字列でない・空のときは、ほかの組み込み関数と同じく失敗として報告する（バルーンに失敗表記、ログに警告）。数値の引数を文字列として受けるかは要件で決める。
   - `pasta_check validate` の `builtins.rs` に 4 つの名前を足す。
5. **作り**: Rust の `SearchContext` に数だけを返すメソッドを足し（例 `count_scene(name, exact)`。別名と正規化は `search_scene_by` と同じ道を通す）、Lua の `GLOBAL` 関数から呼ぶ。完全一致の数のために、`scene_table.rs` に候補を集めるだけの関数を足すか、`prefix_index.get` を直に使うかは設計で決める。候補の控え（巡回の状態）には触らない。

## Scope

- **In**:
  - `@pasta_search` の数を返すメソッドと、その裏のシーン表の関数（要るなら）
  - GLOBAL 関数 4 つ（`scene_count`・`シーン数`・`scene_count_exact`・`シーン数完全一致`）
  - `pasta_check validate` の組み込み関数の名前
  - テスト（Rust の数え方・別名・ローカルを数えない・0 件、Lua の呼び出しと失敗の報告、検査）
  - マニュアル（`grammar/call-jump.md` の組み込み関数の表、`debug/validate.md` の一覧、Lua から使う口を書くなら `lua/` の該当章）、スキルの `references/` の再生成
- **Out**:
  - 単語の数（里々の「単語群「」の数」に当たる `＠単語数`）。欲しいと言われるまで作らない。
  - ローカルシーンの数
  - 条件の属性で絞った「今出せる数」（`scene-condition-attributes` が決める）
  - `＞＞` の文法と完全一致のモード（`scene-exact-match`）

## Boundary Candidates

- 数え方（`pasta_core` のシーン表・`pasta_lua` の `search/context.rs`）
- DSL への口（`GLOBAL` 関数・`builtins.rs`）
- マニュアルとスキルの参照

## Out of Boundary

- 同名シーンの選び方（巡回・重み。`scene-pick-modes`）
- 検索の規則そのもの（前方一致・別名・正規化）。数え方は今の規則に従うだけで、規則は変えない。

## Upstream / Downstream

- **Upstream**: なし。今の main で作れる。
- **Downstream**:
  - `scene-exact-match`: `[scene] match = "exact"` のモードを入れるとき、`シーン数` も `＞` に合わせて完全一致になるか（推奨: なる。「`＞` と同じ数」の約束を保つ）を、あちらで決める（申し送りを書いた）。
  - `scene-condition-attributes`: 「今出せる数」を足すか（申し送りを書いた）。

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `scene-condition-attributes`・`scene-exact-match`: `search/context.rs`・`scene_table.rs` を触るので、同じウェーブに置かない。
  - `ponytail-core-registry`（`scene_table.rs`）・`ponytail-lua-scripts`（`entry.lua` を触るなら）: ファイルの重なりを、入る前に測り直す。
  - 組み込み関数の置き場は、`entry.lua` に寄せるか、`recommend.lua` のように別のファイルに分けるかを設計で決める。

## Constraints

- 既存の検索の動き（巡回の状態・候補の控え）を変えない。数えても `＞` の次の選びに影響しない。
- 規模は 3〜5 タスクの見込み（数え方 1、GLOBAL 関数と検査 1、テスト 1、マニュアル 1）。

## 2026-11-03 棚卸の再測定（main c575febd）

- **前提の変化**: `ponytail-core-registry` が入り、シーン表の索引が `BTreeMap` になった。数えるのに使う公開の関数は残った。前方一致の候補は `SceneTable::collect_scene_candidates(module_name, prefix)`（`scene_table.rs` 372 行。0 件は `SceneNotFound`、空の名前は `InvalidScene`）、候補の情報は `get_scene(id)`（349 行）。完全一致の `resolve_scene_id_exact`（195 行）は、非公開の索引を `get` で引く。`ponytail-lua-scripts` は `entry.lua` の `SHIORI.load`・`unload` の中身を消したが、GLOBAL 関数の置き方は変わらない。Current State の行番号は動いた: `collect_scene_candidates` 458→372 行、`resolve_scene_id_exact` 255→195 行、`search_scene`／`search_scene_exact` 105・119→100・114 行、別名と正規化 166〜168→161・162 行、`add_methods` 358→350 行。
- **完全一致の数は `scene_table.rs` を触らずに数えられる**（設計の最初に確かめることの答え）: `collect_scene_candidates("", 名前)` の候補のうち、`SceneRegistry::split_registered_name(&get_scene(id).name).0 == 名前` のものを数える。実行時のシーン表は `finalize` が `register_global_raw` で作るので、`name` は通し番号つきの登録名（`雑談_1`）である。通し番号を外して比べれば、索引のキー（`fn_name_to_search_key`）と同じになる。どの関数も `pub` なので、`search/context.rs` の中だけで済む。`scene-attribute-store` と同じウェーブに置ける。
- **触るファイル**: `crates/pasta_lua/src/search/context.rs` 1,009（435 行から中のテスト。1,000 行を超えているので、足す試験は別のファイルに置く）、GLOBAL 関数の置き場、`crates/pasta_check/src/validate/builtins.rs` 68（`GLOBAL_NAMES` の数と上の説明）、マニュアル `grammar/call-jump.md` 703・`debug/validate.md` 257・口を書くなら `lua/modules/pasta-search.md` 266、スキルの `references/` の再生成。`scene_table.rs` は触らない。
- **置き場の推奨**: `crates/pasta_lua/pasta_scripts/pasta/global.lua`（29 行。`yield`・`チェイントーク` が既にある。数えるのに SHIORI は要らない）。`shiori/entry.lua`（130 行）に置かなければ、`debug-mcp-runtime`（`entry.lua` の `KICK`・`SHIORI.kick` を消す）と重ならない。
- **規模**: 3〜5 タスクのまま（数え方 1、GLOBAL 関数と検査 1、テスト 1、マニュアルと references 1）。
- **先に要るもの**: 機能の上では無い。ファイルの重なり（未完了）: `scene-condition-attributes`・`scene-pick-modes`・`scene-exact-match`（`search/context.rs`）とは同じウェーブに置かない。`scene-attribute-store` とは、あちらが読み出しの口を `search/context.rs` に置かなければ重ならない。`debug-mcp-runtime` とは、置き場を `entry.lua` にしなければ重ならない。`vscode-mcp-client` とは重ならない。`builtins.rs` は、版の違う `shiori4-v4-lua-api` も触る。
- **種別**: 機能（シーン名にぶら下がるシーンの数を辞書から読む口が無い）。開発者の依頼から（「pasta にもシーン名ごとのトークの数を数える方法が欲しい」）。
- **要件定義のモデル**: Opus。公開の GLOBAL 関数を足すが、形（関数 2 つ・英日の名前）と数え方（`＞` と同じ・グローバルだけ・別名を当ててから正規化・0 件は 0）は 2026-11-01 に開発者が承認済みで、残る問い（数値の引数を文字列として受けるか・置き場のファイル）は浅く、推奨を付けられる。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**: 前の測り直しの「候補の `get_scene(id).name` を比べる」は、そのままでは通し番号つきの `雑談_1` と比べてしまう。上のとおり通し番号を外して比べる。
