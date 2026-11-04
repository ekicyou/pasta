# ギャップ分析: scene-pick-modes

- 対象: `requirements.md`（2026-11-04 生成・未承認）
- 基準: ブランチ `claude/kiro-start-scene-pick-modes-a7f10f`（main `92f6f677` = `scene-condition-attributes` PR#237 の後）
- 行番号はこの時点のもの

## 1. 分析のまとめ

- **シーンの重みは小さい**: 候補の並びは `select_from_cache` が作る（`crates/pasta_core/src/registry/scene_table.rs` 272〜342 行）。並びを作る `new_round`（310〜316 行）で ID を重みの数だけ並べれば、シャッフル・優先の安定な並べ替え・条件の「見て漏れた札も使った」・巡のやり直しは今のまま効く（`ok` は ID の集合なので重複に強い。モックの選択子も位置で並べるので重複に強い）。
- **順番は新しい道が要る**（2026-11-04 追記: 位置は保存しないと決まったので、下の保存の表の話は不要になった）: 今の並びと位置はメモリの中だけ（`CachedSelection`。再読み込みで消える）。順番は「覚えた番号」を保存の表に読み書きする必要があり、`pasta_core` は保存の表を知らない。保存の表は `SCENE.search` が 4 つ目の引数で Rust の検索へ渡している（`pasta_scripts/pasta/scene.lua` 192 行 → `search/context.rs` 420 行）ので、Call・イベント・選択肢の道には届く。名指しの道（`SCENE.get_exact`）には届かない。
- **単語の重みは文法の変更が中心**: 値は `Vec<String>` で、同じ値を重ねれば重みになる（`words.md` 150 行の「2 回書くと 2 回」）。パーサ（`grammar.pest` 155 行の `word_nofenced`・`parse_elements.rs` 122 行）で値を重みの数だけ並べれば、コード生成・Lua・`word_table.rs` は触らずに済む。ただし囲まない値は今 `＠３` を文字として飲み込むので、動きが変わる。
- **読み込みエラーは 2 種類の置き場**: 値の誤り（重み 0・小数・文字列、重みと順番の両方）は `＆優先` の先例（`code_gen/scope_gen.rs` 413〜450 行の `append_set_scene_attrs`・`TranspileError::InvalidPriority`）に並べられる。同じ名前の中の番号の重複は、ファイルをまたぐので 1 ファイルのトランスパイルでは見えず、全シーンが揃う所（`runtime/finalize.rs` 313 行の `finalize_scene_impl`）か `pasta_core` のシーン表の構築で判定する必要がある。
- **規模と危険**: 規模 M（9〜11 タスクの見込みと合う）、危険 中（順番の保存の置き場と、囲まない値の文法の変更による互換）。

## 2. 今あるもの（要件と資産の対応）

| 要件 | 今あるもの | ギャップ |
| ---- | ---------- | -------- |
| 1 重み | `select_from_cache`（scene_table.rs 272）・`new_round`（310）・`scene_priority`/`PRIORITY_KEYS`（scene_types.rs 158〜181）・`SceneAttrValue::Integer`（38） | **Missing**: `WEIGHT_KEYS`（`重み`・`weight`）と重みの読み手。並びに ID を重みの数だけ入れる。`count_scene`（context.rs 248）は候補の集めで数えるので重みの影響を受けない（Req 1.8 は今のままで満たす見込み。テストで確かめる） |
| 2 順番 | 候補の集め（`collect_scene_candidates` 368）・今の段（291〜308） | **Missing**: 今の段の候補が全部 `＆順番` を持つかの判定、覚えた番号より大きい最小の番号で条件を満たすものを選ぶ道、戻り。`MockRandomSelector` の影響を受けない（Req 9.5 の仮定と合う） |
| 3 位置の保存 | 保存の表は 1 つ（`pasta/save.lua`）、`profile/pasta/save/save.toml`（難読化で `.dat`）。終了と再読み込みで丸ごと書く（`runtime/lifecycle.rs`・`persistence.rs` 189/210）。予約の名前は `pasta_talk_interval_min`/`_max` だけ（`variables.md` 436〜448） | **Missing**: 予約の名前（例: `pasta_scene_order` の表に「探した名前 → 番号」）。**Constraint**: 保存の表は Lua の表で、Rust の検索には `search_scene_lua` の引数（mlua の `Table`。書ける）として届く。TOML にできない値は鍵ごとに捨てて警告（persistence.rs 234）なので、番号は整数で入れる。**Unknown**: ローカルシーンの「探した名前」の鍵の作り方（`SceneCacheKey` の `module_name` + `search_key` + `exact` と揃えるか） |
| 4 混ざり | `tracing::warn!(target: "pasta_lua::search", …)`（scene_cond.rs 182 の先例） | **Missing**: 混ざりの注意。重みと順番の両方・同じ名前の番号の重複の読み込みエラー |
| 5 値の誤り | `TranspileError::InvalidPriority`（`pasta_lua/src/error.rs` 46〜48・88）を `append_set_scene_attrs` が出す。`[scene.alias]` の誤りは `SceneAliasError`（`pasta_core/src/error.rs` 35） | **Missing**: 重み・順番の値の誤りの種類。**Constraint**: Lua から直接入れた属性（`finalize.rs` 161 `collect_scene_attrs`・200 `scene_attr_from_item`）の不正な優先は黙って 0（scene_types.rs 178 の shortcut）。重みも同じ扱いにするかは設計で決める |
| 6 単語の重み | `key_words`/`words`/`word`/`word_nofenced`（grammar.pest 150〜155）、`parse_key_words`（parse_elements.rs 122・136〜152 で値を並べる）、`KeyWords.words: Vec<String>`（ast/action.rs 210）。数字の正規化 `normalize_number_str`（parse_elements.rs 265）、`digit` は全角を含む（grammar.pest 142〜143） | **Missing**: 値の後ろの `＠数` の規則（囲む値・さくらスクリプト・囲まない値の 3 形）、`＠０` の誤り。**Constraint**: `word_nofenced` は読点か改行まで全部を飲む（`＃` も）ので、末尾の `＠数` だけを切り出す規則が要る。`＠` は `at`（26 行）で全角・半角 |
| 7 検査 | `validate/report.rs` 26 `FindingKind`（`TranspileInvalidPriority` が Error）・`transpile_rules.rs` 88 `priority_errors`（scope_gen と同じ判定の写し）・`scenes.rs` 33（実際の `SceneTable` を作る）・`references.rs` 13（名前が決まった Call の候補を `collect_scene_candidates` で引ける）・`conditions.rs` 80 `shadowed`（同じ名前の組の作り方の先例）・`mod.rs` 62 `scene_attrs`（同じ倉の写し。テストに `＊＆重み：5` が既にある） | **Missing**: 重み・順番の値の誤り、番号の重複、両方書き、Call の混ざりの 4 種。`validate_report_test.rs` の種類と記号の表（244・282 行）に足す |
| 8 着色 | `editors/vscode/syntaxes/pasta.tmLanguage.json` 160〜174 の `word` は 1 本の正規表現（捕捉 3 が値の全部）。LSP は `visit_scope.rs` 122 `visit_keywords` が `KeyWords.span` 全体に 1 つの `WORD` を出す | **Missing**: 重みの捕捉。LSP で重みだけ塗るには値ごとの位置が構文木に要る（今は無い）。`partial.rs` 150 `shift_keywords` にも響く |
| 9 マニュアル | `words.md` 124〜150、`block-structure.md` 269〜（482 行に「`＆重み` はまだ選び方を変えない」）、`call-jump.md` 292〜303、`pasta-search.md` 36〜50・262〜299、`variables.md` 436〜448、`debug/validate.md`。生成は `book/tools/gen-skill-refs.mjs`（`--check` あり） | 記述の追加と書き換え。動きが変わる点はコミットの題の `!` でリリースノートの「⚠️ 動きが変わる変更」に載る（`.github/scripts/release/release-notes.ps1` 30・108） |
| 10 互換と検証 | `scene_table_candidate_tests.rs`・`scene_table_resolve_filter_tests.rs`・`tests/word_table_test.rs`・`pasta_lua/tests/scene_condition_e2e_test.rs`・`tests/search/scene_cond_search_test.rs`・`pasta_check/tests/validate_*` | 新しいテストは別のファイルに足す（`search/context.rs` は 1,060 行。brief の申し送り） |

## 3. 実現の案

### 案 A: 今の部品を延ばす

- 重み: `scene_types.rs` に `WEIGHT_KEYS`・`ORDER_KEYS`・読み手（`scene_weight`・`scene_order`）を `scene_priority` と並べて足し、`new_round` で ID を重みの数だけ並べる。
- 順番: `select_from_cache` に「今の段の候補が全部順番を持つ」分かれを足す。覚えた番号を渡す口（読み・書きの関数、または `Option<&mut i64>` のような引数）を `resolve_scene_id_*` に足し、`search_scene_lua` が保存の表から読み・書きする。
- 値の誤り: `append_set_scene_attrs` に重み・順番・両方書きの判定を足す。番号の重複は `finalize_scene_impl`（または `SceneTable::from_scene_registry`）で判定してエラーにする。
- 単語: `grammar.pest` の 3 形に後ろの `＠数` を足し、`parse_key_words` で値を並べる。
- 利点: ファイルが増えない。`＆優先` と同じ所に同じ形で並ぶ。
- 欠点: `select_from_cache` が重み・優先・条件・順番の 4 つを持ち、読みにくくなる。`scene_table.rs` は 438 行で、600 行の目安に近づく。

### 案 B: 順番の選び方を別の部品にする

- 重みは案 A と同じ（並びを作る所の 1 行の変更で済むので分けない）。
- 順番の「覚えた番号から次を選ぶ」を、`pasta_core` の独立した関数（候補の `(SceneId, 番号, 条件の結果)` の並びと覚えた番号を受け、選んだ ID と新しい番号を返す純粋な関数）として新しいファイルに置く。保存の表の読み書きは `pasta_lua` の検索の文脈（`search/` の新しいファイル）が持つ。
- 利点: 順番の規則（飛ばし・戻り・消した番号）を乱数も Lua も無しに単体で試せる。`pasta_core` が保存を知らないまま保てる。
- 欠点: 口が 1 つ増える。今の段の求め方を `select_from_cache` と分け合う必要がある。

### 案 C: 順番を Lua 側で行う

- `SCENE.search` の手前で、Lua が候補の属性（`SCENE.attrs`）と `save` を見て順番を処理する。
- 利点: 保存の表の扱いが Lua だけで閉じる。
- 欠点: 候補の集め・別名・条件・今の段を Lua 側でやり直すことになり、Rust の検索と二重になる。`pasta_check validate` の判定と食い違いやすい。勧めない。

## 4. 統合の難所

1. **順番の位置の鍵**: 「探した名前ごと」の鍵を、前方一致・完全一致・ローカル（どのグローバルシーンの中か）で分けるか。`SceneCacheKey` と同じ分け方にすれば、並びの控えと位置の控えが同じ単位になる。鍵の文字列は保存ファイルに出るので、ローカルの鍵にグローバルシーンの登録名（`:会話_1:締め` のような番号付き）を使うと、辞書を書き換えたときに番号がずれて位置が失われる。**Research Needed**: 書いた名前で鍵を作れるか。
2. **名指しの道と保存の表**: `SCENE.get_exact`（scene.lua 255〜276）や `OnPastaScene` は `var`/`save` を渡さない。Req 1.4・2.11 の仮定（名前で選ぶ所だけ）なら影響は無いが、名指しの道は 1 本を起こすだけでテーブルが無いので、範囲を広げないと決まった。
3. **ファイルをまたぐ番号の重複**: トランスパイルは 1 ファイルずつなので、同じ名前のシーンが別ファイルにあると見えない。読み込みエラーを全シーンが揃った時点で出す道（`finalize_scene_impl` の `LuaResult` のエラー）が、止まるエラーのバルーン（`runtime-error-balloon`）と読み込みエラーのログに同じ形で出るか。**Research Needed**。
4. **囲まない値の文法の変更**: `word_nofenced` を「末尾の `＠数` を除く」形にすると、今 `xx＠3` と書いて文字を出している辞書の出力が変わる（Req 9.7）。メールアドレス風の `user@3` も重みになる。PEG で「末尾だけ」を書くには先読みが要り、`partial.rs`・LSP・tmLanguage の 3 か所で同じ規則を写す必要がある。
5. **LSP の値ごとの位置**: 重みだけを塗るには、構文木の `KeyWords` に値ごとの位置（または重みの位置）が要る。今は値の文字列だけで、`element_gen.rs` は値を並べて出すだけ。パーサで値を並べる案（重みの数だけ同じ文字列を入れる）と、位置を持つ案が両立するかは設計で決める。
6. **大きな重み**: 重み N の札を N 枚並べるので、`＆重み：1000000` で並びが大きくなる。上限は設けないと決まった（重みはテーブルに乗せる回数でしかない）。単語も同じ。
7. **`scene-exact-match`・`novel-variable-scope` との重なり**: どちらも未着手。`scene-exact-match` は `search/context.rs` を、`novel-variable-scope` は保存の範囲と `grammar.pest` を触るので、同じウェーブに置かない。

## 5. 規模と危険

- **規模: M（3〜7 日）** — 重みとエラーは今の形に並べるだけ。順番の保存・単語の文法・着色・検査・マニュアルが横に広い（9〜11 タスク）。
- **危険: 中** — 新しい技術は無いが、保存ファイルに新しい予約の名前を足すこと（取り消しにくい）と、囲まない値の読み方を変えること（今の辞書の出力が変わる）がある。

## 6. 設計フェーズへの勧め

- **勧める案**: 重みは案 A（並びを作る所で ID を重みの数だけ並べる）。順番は案 B（純粋な関数として `pasta_core` に置き、保存の表の読み書きは `pasta_lua` の検索の文脈が持つ）。単語の重みはパーサで値を並べ、コード生成・Lua・単語表は触らない。
- **設計で決めること**:
  - 覚えた番号の持ち方（保存しないと決まった。シャッフルの控え `CachedSelection` と同じ所に持てば、保存の表を渡す口は要らない見込み）
  - 番号の重複のエラーを出す所（finalize か、シーン表の構築か）と、エラーの表示の形
  - `select_from_cache` の口（覚えた番号の受け渡し）と、今の段の求め方の分け合い
  - `word_nofenced` の末尾の `＠数` の PEG の書き方と、`partial.rs`・tmLanguage・LSP への写し方
  - Lua から直接入れた属性の不正な重み・順番の扱い（優先の先例どおり黙って既定値にするか）
- **Research Needed**:
  - 全シーンが揃った時点のエラーが、読み込みエラーとして同じ形で見えるか（難所 3）
  - ローカルシーンの鍵を書いた名前で作れるか（難所 1）
  - `word_nofenced` の変更が今のテスト（`parser_test.rs` の単語の値）の出力を変えないか。リポジトリの `.pasta`（サンプルゴースト・入門の段階辞書・テストの辞書）には、`＠数` で終わる囲まない単語の値は無かった（2026-11-04 に grep で確かめた）。外のゴースト（emo2・窓際のぱすたさん）は未確認

---

# 設計フェーズの調査（2026-11-04）

## Summary

- **Feature**: `scene-pick-modes`
- **Discovery Scope**: Extension（今のシーン表・パーサ・検査の核を延ばす。新しい依存は無い）。`kiro-spec-design` の light discovery で、拡張の口・今の型・互換を確かめた。
- **Key Findings**:
  - 順番の位置は保存しないと決まったので、gap 分析の「保存の表を Rust へ渡す口」は要らない。最後の番号は `SceneTable` の中に、並びの控えと同じ鍵 `SceneCacheKey` で持てば足りる。名指しの道に保存の表が届かない問題（難所 2）も消えた。
  - `pasta_core` は `tracing` を持たず、`scene-condition-attributes` の設計が「`pasta_core` に `tracing`・`mlua` を足さない」と決めている。混ざりの注意は `SceneTable` の控え（`take_pick_warning`）に残し、`pasta_lua` の `SearchContext` が出す。
  - 単語の重みをパーサで並べると、`create_word(k):entry(v1, …, vN)` の N が Lua の 1 回の呼び出しの上限に当たる。PUC Lua 5.4 で試すと 250 個は通り、260 個は「function or expression needs too many registers」で失敗した（LuaJIT の `LJ_MAX_SLOTS` も 250）。`＠３００` で読み込めなくなるので、コード生成で `:entry` を分ける必要がある。

## Research Log

### 抽出した候補と取り出しの境目

- **Context**: 開発者の方針「抽出（名前・条件・優先）は済んでいる。この spec は取り出しだけ」を、コードのどこで切るか。
- **Sources Consulted**: `crates/pasta_core/src/registry/scene_table.rs`（`select_from_cache` 272〜342 行）、`.kiro/specs/completed/scene-condition-attributes/design.md`。
- **Findings**:
  - 述語の評価と今の段（`ok`・`tier`）を求めた後、並びを作る前が境目になる。抽出した候補は「`ok` に入り、優先が `tier` と同じもの」。述語が無いとき（名指し・今の挙動）は段を見ないので、候補の全部。
  - `ok` は ID の集合なので、並びに同じ ID が重みの数だけあっても壊れない。`MockRandomSelector` は位置で並べるので、重なった ID にも効く。
- **Implications**: 重みは `new_round` の 1 か所、順番は `select_from_cache` の中の分かれ 1 つで済む。抽出の段は触らない。

### 番号の重なりの検査の置き場

- **Context**: 同じ名前の番号の重なり（3.3）はファイルをまたぐので、1 ファイルずつのトランスパイルでは見えない。
- **Sources Consulted**: `runtime/finalize.rs`（`finalize_scene_impl`）、`search/mod.rs`（`loader`）、`loader/cache.rs` 368 行（`scene_dic.lua` の最後で `finalize_scene()` を呼ぶ）、`runtime/factory.rs`（`require_startup_module(&runtime.lua, "pasta.scene_dic")?`）。
- **Findings**:
  - `finalize_scene` が `Err` を返すと、起動のモジュール `pasta.scene_dic` の読み込みの失敗として止まる。今の読み込みの失敗と同じ道（gap 分析の Research Needed 1 の答え）。
  - 「同じ名前」は `prefix_index` のキーと同じ単位になる（グローバルは別名を置き換えた後の照合用の名前、ローカルは `:親の登録名:名前`）。キーごとに番号を見れば、別名の畳みもローカルの範囲もそのまま当たる。
  - `pasta_check` も実物の `SceneTable` を作っているので、同じ関数（`duplicate_orders`）を使える。SceneId から位置へは、登録と同じループで（パス, 位置）を積めば引ける。
- **Implications**: 検査は `SceneTable::duplicate_orders` の 1 つ。実行時は `search::loader`、検査の核は `scenes.rs` が呼ぶ。判定の写しを作らない。

### ローカルシーンの鍵（gap 分析の Research Needed 2）

- **Findings**: 位置を保存しないので、鍵の文字列が保存ファイルに出ることは無い。並びの控えと同じ `SceneCacheKey`（登録名を含む）で覚えてよい。辞書を書き換えると再読み込みで消えるので、番号のずれも起きない。
- **Implications**: 問いは消えた。

### 単語の値の重みの文法

- **Sources Consulted**: `grammar.pest` 150〜155 行、`parse_elements.rs` 122 行、`element_gen.rs` 498 行、`word.lua` 29 行（`entry` は自身を返す）、`pasta_lsp/src/analysis/visit_scope.rs` 122 行、`editors/vscode/syntaxes/pasta.tmLanguage.json` 160 行。
- **Findings**:
  - 値の後ろに `word_weight` を任意で足し、囲まない値は「`＠数` の後に値の終わり（空白・読点・改行・入力の終わり）が続く位置」の手前で止めれば、末尾だけを重みとして読める。値の途中の `＠３`（後ろに文字が続く）は今までどおり値になる。
  - PEG の選択は一度成功すると戻らないので、値の無い `＠３` を受ける形は「値の終わりが続くとき」に限る（`＠３abc` を今までどおり囲まない値として読むため）。
  - LSP は `KeyWords.span` 全体に `WORD` を 1 つ出している。重みだけ塗るには位置が要るので、構文木に `weight_spans` を足す。`KeyWords { … }` を直に書く所は 23 か所（多くはテスト）で、欄を足すだけ。
  - TextMate の `word` は 1 本の正規表現で、囲み・さくらスクリプトの値の後ろに `＠３` が来る形を今は受けない。各形に重みの後置を足す。
- **Implications**: コード生成・Lua・`WordTable` は、`:entry` の分割を除いて触らない。

### 参照したスキルと指針

- `kiro-spec-design` の `design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md` を当てた。
- ドメインのスキル（`pasta-ghost-authoring`・`pasta-lua-coding`）の `references/` はマニュアルから生成した写しなので、権威のマニュアル（`book/src/grammar/words.md`・`block-structure.md`・`call-jump.md`・`lua/modules/pasta-search.md`）を直接読んだ。
- ステアリングは `product.md`・`tech.md`・`structure.md`（600 行の目安・テストの置き場の規則）と、文法を触るので `grammar.md`。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: `select_from_cache` に分かれを足す | 重みは `new_round`、順番は同じ関数の中の分かれ | 抽出の結果（`ok`・`tier`）をそのまま使える。足すのは 20 行ほど | 関数が少し長くなる | **採用** |
| B: 順番を独立の純粋な関数に | 候補と覚えた番号を受けて選んだ ID を返す | 単体で試しやすい | 抽出した候補の求め方を分け合う口が増える。保存が無くなった今は利点が薄い | gap 分析の勧め。保存しないと決まったので見送り |
| C: Lua 側で順番 | `SCENE.search` の手前で処理 | — | 抽出を Lua でやり直すことになる | 見送り（gap 分析と同じ） |

## Design Decisions

### Decision: 最後の番号はシーン表の中に持つ

- **Context**: 2.4（メモリの中だけ・シャッフルの続きと同じ寿命）。
- **Alternatives Considered**:
  1. `CachedSelection` に `last_order` を足す — 控えを作るときに並びのシャッフルが走り、乱数を使ってしまう（8.4 に反する）。
  2. `SceneTable` に別の表 `last_orders: HashMap<SceneCacheKey, i64>` — 並びと独立。
- **Selected Approach**: 2。`replace_selector` で `cache` と一緒に消す。
- **Trade-offs**: 表が 1 つ増えるが、順番の選択が乱数の列を消費しないことが保たれる。

### Decision: 注意は控えに残し、`pasta_lua` が出す

- **Context**: 3.1 の warn。`pasta_core` に `tracing` を足さない取り決め。
- **Alternatives Considered**:
  1. `pasta_core` に `tracing` を足す — 2 行で済むが、取り決めに反する。
  2. `resolve_*` の戻り値を変える — 公開の 3 つの関数と多くのテストに響く。
  3. `SceneTable` に直前の注意を 1 つ残し、`take_pick_warning` で読む — 公開の関数の形は変わらない。
- **Selected Approach**: 3。`SearchContext` の 2 か所で読んで warn にする。

### Decision: 値の誤りは `＆優先` の先例に並べる

- **Context**: 3.2・4.2・4.4。
- **Selected Approach**: `append_set_scene_attrs`（受け継ぎと畳みの後の並び）に重み・順番の値と両方書きの検査を足し、`pasta_check` の `priority_errors` を同じ判定に広げる。順番の値の判定は優先と同じ（整数なら受け付ける）なので `priority_value` を使う。

### Decision: 単語の重みはパーサで並べ、`:entry` だけ分ける

- **Context**: 5.1。開発者の方針は「パーサで並べてコード生成・Lua・単語表を触らない」。
- **Findings**: 並べるだけでは Lua の引数の上限（約 250）に当たる。
- **Selected Approach（前提。設計ディスカッションで確かめる）**: コード生成で値を 200 個ずつの `:entry(…)` に分け、同じ行に続ける。Lua と単語表は触らない。手で 250 個以上書いた単語（今は読み込めない）も読めるようになる。
- **Alternatives**: 分けずに、値が多すぎる単語を読み込みエラーにする（パーサか検査で上限を決める）。

## Synthesis

- **一般化**: 重み・順番・優先は「宣言の最後の値を鍵の組で引く」同じ形なので、`scene_types.rs` の読み手が非公開の小さな関数を共有する。番号の重なりの検査は実行時と検査の核で同じ関数（`duplicate_orders`）にした。
- **作るか採るか**: 新しい依存は無い。重み付きの抽選の道具（`rand::distributions::WeightedIndex` など）は「毎回の確率のくじ」で、開発者の決定（重み＝並びに乗せる回数）と違うので使わない。
- **簡素化**: gap 分析の案 B（独立した順番の部品）と、保存の表の受け渡し・予約の名前・ローカルの鍵の作り方は、保存しないと決まったので全部落とした。単語の重みは構文木に値を並べるだけで、新しい型を作らない。

## Risks & Mitigations

- 囲まない単語の値の最後の `＠数` の意味が変わる — リポジトリの辞書には無いことを確かめた。squash の題の `!` とマニュアルで知らせる。外のゴースト（emo2・窓際のぱすたさん）は未確認。
- 重みに上限が無い — 極端な値は並び・生成する Lua のメモリが比例して増える。開発者の決定（上限を設けない）に従い、マニュアルに「重みは並びに乗せる回数」とだけ書く。
- TextMate の `word` の正規表現が長く、重みの後置を足すと読みにくい — `tmGrammar.test.ts` で 3 つの値の形と途中の `＠` を確かめる。
- `.claude/skills/*/references/` の再生成は、作業の環境によっては書き込みが拒まれることがある — 生成の道具（`gen-skill-refs.mjs`）で作り、`--check` で一致を確かめる。拒まれたら開発者に許可を求める。

## 統合の記録（kiro-complete 用）

main へ入れるときの squash の題と本文に、そのまま使う（8.6）。

- **squash の題**: `feat(scene-pick-modes)!: 同名シーンを ＆重み・＆順番 で選べるようにし、単語の値に ＠数 の重みを付けられるようにする`
- **動きが変わる点**（本文の `BREAKING CHANGE:` に置き、0.5.0 のリリースノートの「動きが変わる変更」に載せる）:
  - 囲まない単語の値の最後の `＠数`（例 `＠色：赤＠３`）は、文字でなく重みとして読まれるようになった。今までは `赤＠３` という文字を出していた。最後に `＠３` という文字を出したいときは、値を「」で囲む（`＠色：「赤＠３」`）。値の途中の `＠３`（後ろに文字が続く）は今までどおり文字のまま。
  - 最後が `＠０` の囲まない値（例 `＠色：赤＠０`）と、`＠数` だけの囲まない値（例 `＠色：＠３`）は、辞書の読み込みエラー（パースエラー。そのファイルは読み込まれない）になった。今までは読み込めて、その文字を出していた。
