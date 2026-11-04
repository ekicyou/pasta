# ギャップ分析: scene-pick-modes

- 対象: `requirements.md`（2026-11-04 生成・未承認）
- 基準: ブランチ `claude/kiro-start-scene-pick-modes-a7f10f`（main `92f6f677` = `scene-condition-attributes` PR#237 の後）
- 行番号はこの時点のもの

## 1. 分析のまとめ

- **シーンの重みは小さい**: 候補の並びは `select_from_cache` が作る（`crates/pasta_core/src/registry/scene_table.rs` 272〜342 行）。並びを作る `new_round`（310〜316 行）で ID を重みの数だけ並べれば、シャッフル・優先の安定な並べ替え・条件の「見て漏れた札も使った」・巡のやり直しは今のまま効く（`ok` は ID の集合なので重複に強い。モックの選択子も位置で並べるので重複に強い）。
- **順番は新しい道が要る**: 今の並びと位置はメモリの中だけ（`CachedSelection`。再読み込みで消える）。順番は「覚えた番号」を保存の表に読み書きする必要があり、`pasta_core` は保存の表を知らない。保存の表は `SCENE.search` が 4 つ目の引数で Rust の検索へ渡している（`pasta_scripts/pasta/scene.lua` 192 行 → `search/context.rs` 420 行）ので、Call・イベント・選択肢の道には届く。名指しの道（`SCENE.get_exact`）には届かない。
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
  - 順番の位置の鍵の形と予約の名前（置き場は今の保存ファイルの `＄＊pasta_` の予約の名前と決まった）
  - 番号の重複のエラーを出す所（finalize か、シーン表の構築か）と、エラーの表示の形
  - `select_from_cache` の口（覚えた番号の受け渡し）と、今の段の求め方の分け合い
  - `word_nofenced` の末尾の `＠数` の PEG の書き方と、`partial.rs`・tmLanguage・LSP への写し方
  - Lua から直接入れた属性の不正な重み・順番の扱い（優先の先例どおり黙って既定値にするか）
- **Research Needed**:
  - 全シーンが揃った時点のエラーが、読み込みエラーとして同じ形で見えるか（難所 3）
  - ローカルシーンの鍵を書いた名前で作れるか（難所 1）
  - `word_nofenced` の変更が今のテスト（`parser_test.rs` の単語の値）の出力を変えないか。リポジトリの `.pasta`（サンプルゴースト・入門の段階辞書・テストの辞書）には、`＠数` で終わる囲まない単語の値は無かった（2026-11-04 に grep で確かめた）。外のゴースト（emo2・窓際のぱすたさん）は未確認
