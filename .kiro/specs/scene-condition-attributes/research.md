# ギャップ分析: scene-condition-attributes

- 日付: 2026-11-03（main `b03a1ca5` を元にしたワークツリーで測った）
- 対象: `requirements.md`（Requirement 1〜10）
- 方針: 情報と選択肢を示す。最終の決定は設計フェーズで行う。

## 1. 分析の要約

- 条件の形はもう倉に入っている（Rust の `SceneInfo.attributes: Vec<SceneAttr>`、Lua の `STORE.scene_attrs`）。足りないのは**評価**と、評価の結果で**袋から引く**ところだけである。文法・パーサ・LSP・VSCode の文法定義には触らなくてよい。
- いちばん大きな穴は、**変数（Lua）と袋（Rust）が別の側にある**ことである。今の検索の口（`SEARCH:search_scene(name, gsn)`）は文字列しか受けず、act も変数も渡らない。どこで評価するか（Rust に値を渡す・Rust が Lua の表を読む・Lua で絞る）が設計の第一の決定になる。
- 袋の今の作り（`select_from_cache`）は「絞った後の id の並び」を袋にしている。要件 4 の「外れた札を消費せずに残す」「満たした札をその巡に混ぜる」は、**袋を名前ごとの全候補で作り、引くときに評価して飛ばす**形への作り直しが要る。条件の無い名前で乱数の順を変えない（要件 4.1）ことが制約になる。
- 「名前が無い」と「全部落ちた」は、Rust でも Lua でも同じ `nil` に潰れている（`SceneNotFound`・`NoMatchingScene` → `Ok(None)`）。素通り（要件 3）には、この 2 つを分けて返す口が要る。
- 自動転記は `EVENT.fire` の `create_act(req)`（`shiori/event/init.lua` 300〜315 行）が 1 か所の入口になり、小さく入る。検査（要件 8）は、`pasta_check` に `＄＊` の代入を集める仕組みが無いので新しく足す。

## 2. 今の状態

### 2.1 シーンの選ばれ方（Lua）

| 経路 | 場所 | 今の振る舞い |
| ---- | ---- | ------------ |
| Call `＞名前` | `act.lua` `call`（683〜711 行）→ `find_handler`（`find_act_handler` 358〜399 行）| 5 段で探す。L1 `current_scene[key]`、L2 ローカルの前方一致（`search_dictionary` → `SCENE.search(key, gsn)`）、L3 act のメソッド、L4 `GLOBAL`、L5 グローバルの前方一致。見つからなければ `FAILURE.report(SCENE_NOT_FOUND)`（705〜710 行）。`attrs` は生成コードから常に `{}` が来て捨てられる |
| イベント | `EVENT.fire`（`shiori/event/init.lua` 285 行）→ `EVENT.no_entry`（217〜221 行）→ `SCENE.co_exec(act, req.id)`（`scene.lua` 237〜243 行）→ `act:find_scene`（`act.lua` 668 行。= `find_handler`）| イベントごとに `create_act(req)` で新しい act（`var = {}`）を作る。OnBoot（`boot.lua` 19 行）、マウス（`mouse.lua` 128 行）、おすすめ（`recommend.lua` 281 行）も `SCENE.co_exec` |
| OnTalk・時報 | `virtual_dispatcher.lua`（315 行）| 時報は `check_hour`（156〜194 行）で**選ぶ前に** `act:transfer_date_to_var()`（182〜184 行）を呼び、`時報HH` → `OnHourHH` → `時報その他` → `OnHourOther` を順に試す。OnTalk は `check_talk`（199〜234 行）。チェイントークの続き（`STORE.co_scene`）があれば選び直さずそれを返す |
| 選択肢の飛び先 | `shiori/event/choice_select.lua` 53〜54 行 | `SCENE.search(choice_id, scope)` を直に呼ぶ。act は手元にあるが渡していない |
| 外からの `＞名前` | `external_control.lua` 45 行（`OnPastaCallFunction`）| `act:find_handler("scene", name)` を直に呼ぶ |
| 名指しの再生（効かせない所） | `SCENE.co_exec_exact`（283〜289 行）・`co_exec_registered`（296〜302 行）・`get_exact`（254〜275 行）| `mouse.lua` 120 行（近道）・`init.lua` 185 行（`チェイントーク中断`）・`recommend.lua` 283 行・`scene_play.lua` 45 行（OnPastaScene）。act を受けない |
| `＞＞`（`scene-exact-match`）| — | **まだ無い**（文法にも生成にも無い。brief だけ） |

- 呼ばれたシーンは呼び出し元と同じ `var` を使う（`init_scene` 205〜217 行が `self.save, self.var` を返す）。要件 2.4 は、評価で `act.var` を見れば今の作りのまま満たせる。
- L1 `current_scene[key]` は検索を通らない直の引きである。生成されたローカルシーンの関数名には連番が付くので、`＞締め` が L1 に当たることは普通は無い見込みだが、条件を飛ばす抜け道にならないかは **Research Needed**。

### 2.2 シーン表と袋（Rust）

- `search/context.rs`（1,060 行。テスト込み）: `search_scene_by`（124〜189 行）が、いつも空の `filters`（130 行）で `resolve_scene_id_unified`・`resolve_scene_id_exact` を呼ぶ。`SceneNotFound` と `NoMatchingScene` は、どちらも `Ok(None)` にする（152〜155・173〜185 行）。`search_local_scene_exact`（235 行）も空の `filters`。`count_scene_by`（207 行）は袋を使わない。
- Lua の口（387 行〜）は `search_scene(String, Option<String>)` などで、**表や関数を受ける引数は無い**。
- `scene_table.rs`（451 行）:
  - `resolve_scene_id_unified`（170 行）・`resolve_scene_id_exact`（195 行）・`resolve_global_owning_local_exact`（237 行）が、`collect_scene_candidates` → `filter_by_attributes` → `select_from_cache` の順に進む。
  - `filter_by_attributes`（283〜316 行）は `SceneAttr::Decl` だけを文字列の表と比べる。`Cond`・`Flag` は見ない。空になると `NoMatchingScene` を返す。本番では `filters` がいつも空なので、事実上使われていない。
  - `select_from_cache`（324〜355 行）: 袋 `CachedSelection { candidates, next_index }` は**絞った後の id の並び**から作る。最初に 1 回シャッフルし、`next_index` が末尾に来たら、そのときの `filtered_ids` で並べ直す。袋の鍵 `SceneCacheKey`（15〜49 行）は `module_name`・`search_key`・`filters`・`exact`・`owning_local` で、変数の値は入っていない。
- `scene_types.rs`（273 行）: `SceneAttr { Decl, Cond { key, op: CmpOp, value: SceneAttrValue }, Flag { key } }`。`CmpOp` は 6 種、`SceneAttrValue` は `Integer(i64)`・`Float(f64)`・`String`。`Cond`・`Flag` の `key` は `＄名前`・`＄＊名前`（印は全角にそろえ、名前は書いたまま）。

### 2.3 失敗・ログ・転記

- `FAILURE.LABEL` は 12 種（`failure.lua` 11〜24 行）。素通りは失敗ではないので足さない（要件 3.6）。
- `transfer_date_to_var`（`shiori/act.lua` 151〜191 行）は `year month day hour min sec wday`（数）、`年 月 日 時 分 秒`・`曜日`・`week`・`時１２`・`季節`・`節句` を入れる。`transfer_req_to_var`（205〜234 行）は `ｒ０〜ｒ９`・`r0〜r9`・`req_id`・`req_base_id`。実行時は時報だけが自動で呼ぶ。
- Lua の注意のログは `@pasta_log`（`act.lua` 644 行・`failure.lua` 139 行）、Rust は `tracing`（`search_scene_by` の別名の warn）。

### 2.4 検査（pasta_check）

- `validate/report.rs`（448 行）の `FindingKind` は 27 種。`SkipReason { Syntax, UnreadableFile, Config }` があると、飛び先とおすすめの検査は走らない（`mod.rs` 121〜128 行）。
- `mod.rs` の `scene_attrs`（61〜90 行）は `#[doc(hidden)]` でテストからしか使われていない。条件の項目はここから読める。
- **辞書の中の `＄＊` の代入を集める仕組みは無い**（`scenes.rs`・`references.rs`・`recommend.rs` は変数を見ない。`transpile_rules.rs` は引数への代入とプロパティの位置だけ見る）。
- 報告の名前は `kebab-case`（`call-target-missing` など）。マニュアル `debug/validate.md` の表（114〜188 行）に載せる。

### 2.5 マニュアルと入門

- 直す所: `block-structure.md` 392 行（「まだ評価されず」）、`script-api.md` 326・337・339 行（`find_scene`・`SCENE.search` の `attrs` を使わない）、`call-jump.md` 291〜293 行（候補の選択）・328 行（Call が失敗したとき）・678〜714 行（シーン数。「属性によって変わらない」と既に書いてある）、`registry-search.md` 196・199・207・297 行、`variables.md` 342〜350・375 行、`debug/validate.md` の表。
- 入門の 7 段目: `hello-pasta/ghost/master/dic/07-greeting.pasta` は OnGhostChanged・OnGhostChanging の 4 本が本文の先頭で `＞transfer_req_to_var` を呼ぶ。`07-greeting.md`（306 行）はその説明を 56〜174・296 行で書く。属性も `transfer_date_to_var` も使っていない。`STAGES.md` 17 行。段階の検査に `tutorial_stages_test.rs` がある。
- `book/tools/gen-skill-refs.mjs` がある（テスト `gen-skill-refs-test.mjs` つき）。

## 3. 要件と資産の対応

| 要件 | 使える資産 | ギャップ |
| ---- | ---------- | -------- |
| 1 評価 | 倉の `Cond`・`Flag`（Rust・Lua の両方）、`CmpOp`、`SceneAttrValue` | **Missing**: 評価の関数（数か文字列かの判定・真偽値・値の無い変数・大小の失敗）。**Unknown**: 文字列の値を数として読む範囲（未決の議題 2）。**Constraint**: Lua の数は倍精度、Rust の倉は `i64`。値で比べる（表記で比べない） |
| 2 効く範囲 | `find_act_handler` の 5 段、`SCENE.search`・`SCENE.co_exec`、`choice_select.lua` | **Missing**: 検索の口に act（変数）を渡す道。`choice_select.lua` と `external_control.lua` の直の呼び出しも通す。**Constraint**: 名指しの再生（`co_exec_exact` 系）は通さない。`＞＞` はまだ無いので、完全一致の Rust 側（`resolve_scene_id_exact`）に効かせておく形で足りる |
| 3 ゼロ件 | `SceneNotFound`・`NoMatchingScene` の 2 種のエラー | **Missing**: `Ok(None)` に潰さず「全部落ちた」を Lua まで返す口と、Call・イベント・時報の段での扱い（未決の議題 1）。**Constraint**: 「見つからない」の表記は今のまま |
| 4 袋 | `select_from_cache`、`SceneCacheKey`、`random_selector` | **Missing**: 全候補で袋を作り、引くときに評価して飛ばす。「満たす札が未消費に無い」ときの巡の始め直し。**Constraint**: 条件の無い名前では乱数の消費の順を今と同じにする（要件 4.1。既存のテストの期待を守る） |
| 5 `＆優先` | 倉の `Decl`、`merge_attrs` の受け継ぎ | **Missing**: 優先の段で残す処理、別名 `priority`、整数でない値の読み込みエラー（辞書確定のどこで止めるか）。**Constraint**: `scene-pick-modes` の `＆重み`・`＆順番` はこの後に「優先で残った中」に入る |
| 6 自動転記 | `transfer_req_to_var`・`transfer_date_to_var`、`EVENT.fire` の `create_act(req)` | **Missing**: 選ぶ前に呼ぶ 1 か所。**Unknown**: チェイントークの続きがどの act で動くか（新しい act に転記しても続きの `var` に効かないかの確認）。時報の二重の転記は無害 |
| 7 ログ | `tracing`（Rust）、`@pasta_log`（Lua） | **Missing**: 外れた候補と条件と値の trace、素通りの trace、大小の失敗の warn。**Constraint**: 評価を Rust でするなら Rust の `tracing`、Lua でするなら `@pasta_log`。どちらでも trace の行の形をマニュアルに書く |
| 8 検査 | `FindingKind`、`skip_reasons`、`scene_attrs`、`pasta_dsl` の構文木 | **Missing**: `＄＊` の代入を辞書全体から集める処理と、新しい報告の種類。**Constraint**: `pasta.dll` との一致テスト（`validate_parity_test.rs`・`validate_attr_parity_test.rs`）の考え方に合わせる |
| 9 マニュアル | 上の 2.5 の各所、`gen-skill-refs.mjs` | **Missing**: 書き直し。入門の 7 段目は段階辞書と `tutorial_stages_test.rs` を一緒に直す |
| 10 互換・検証 | `finalize_scene_test.rs`（辞書確定）、`search/context.rs` のテスト | **Missing**: 辞書の読み込みから `@pasta_search` の絞り込みまでの端から端のテスト（`scene-attribute-store` の申し送り）。`search/context.rs` に足すテストは別のファイルに置く（ウェーブの約束） |

## 4. 実装の選択肢

設計の第一の決定は「条件をどこで評価するか」である。袋は Rust にあり、変数は Lua にある。

### 選択肢 A: Rust で評価し、Lua の変数の表を Rust へ渡す（既存の部品を伸ばす）

- 検索の口に、ローカル変数とグローバル変数の表（`act.var`・`save`）を足して渡す（`search_scene(name, gsn, var, save)` の形）。Rust は候補の `Cond`・`Flag` の `key` で表を引き、`SceneAttrValue` と比べる。
- 袋の作り直し・`＆優先`・素通りの区別・trace は、どれも `scene_table.rs` と `search/context.rs` の中で閉じる。`filter_by_attributes` を評価の場所に作り直す（今の `Decl` の文字列一致は本番で使われていないので、置き換えても挙動は変わらない）。
- Lua の側は、`SCENE.search`・`search_dictionary`・`choice_select.lua`・`external_control.lua` で act を運ぶだけ。`act.lua`（812 行）にはほとんど足さない。
- ✅ 袋と評価が同じ所にあり、「外れた札を消費しない」を 1 か所で書ける。`scene-pick-modes` が同じ所に重みと順番を足せる。
- ✅ 評価の規則を `pasta_check` と Rust の関数で共有できる（検査で評価を真似るときの差が小さい）。
- ❌ Lua の値（数・文字列・真偽値・nil）の読み方を Rust に書く（倍精度と `i64` の差に注意）。trace は Rust の `tracing` に出るので、Lua のログと並べて読めるかを確かめる。
- ❌ `search/context.rs` は 1,060 行。評価は新しいモジュールに分ける前提にする。

### 選択肢 B: Lua で評価し、Rust は候補の並びと消費を受け持つ（新しい口を足す）

- Rust に「名前に合う候補を、今の袋の順に全部返す」口と「この候補を消費した」と伝える口を足す。Lua が `SCENE.attrs` で条件を読み、`act.var`・`save` で評価して、最初に満たした候補を選ぶ。
- 評価は新しい Lua のファイル（`pasta/scene_cond.lua` など）に置き、`act.lua` には足さない。
- ✅ 変数を Lua の型のまま評価でき、真偽値・nil の扱いが素直。ログは `@pasta_log` に揃う。
- ❌ 袋の規則（外れた札を残す・巡の始め直し・優先の段）が Rust と Lua に割れる。`scene-pick-modes` の重み（札の枚数）と順番も、両方の側を触ることになる。
- ❌ 1 回の選択で Rust と Lua を 2 往復する。

### 選択肢 C: 混ぜる（Rust が Lua の判定の関数を呼ぶ）

- 検索の口に、候補を受けて真偽を返す Lua の関数を渡す。Rust の袋の中で、引くたびにその関数を呼ぶ。評価そのものは Lua の新しいファイルに置く。
- ✅ 袋の規則は Rust の 1 か所、変数の読み方は Lua の 1 か所に分かれる。
- ❌ Rust の袋の処理の途中で Lua を呼ぶので、借用（`&mut SceneTable` を持ったまま Lua を呼ぶ）とエラーの伝え方の設計が要る。ログが 2 つの側に割れる。

### 比べた結果（参考。決定は設計で）

- 袋の規則を 1 か所に置けて、`scene-pick-modes` が後から同じ所に足せる **A** が、要件 4・5 との相性がいちばん良い。B は評価が書きやすいが、袋の規則が割れる。C は A と B の中間だが、借用とエラーの扱いが重い。

## 5. 規模とリスク

- **規模: L（1〜2 週間）**。評価・袋の作り直し・優先・素通りの区別が Rust の検索の芯に入り、Lua の 4 経路・自動転記・検査・マニュアル（入門の段階辞書を含む）に広がる。brief の見込み 11〜15 タスクと合う。
- **リスク: Medium**。新しい技術は無いが、袋の作り直しで「条件の無い名前の乱数の順を変えない」を守る必要があり、既存のテスト（固定の乱数での順）が壊れやすい。評価の置き場を誤ると `scene-pick-modes` が作り直しになる。

## 6. 設計フェーズへの申し送り

### 決めること

1. 評価の置き場（選択肢 A・B・C）と、検索の口に何を運ぶか（act・変数の表・判定の関数）。
2. 並びと巡の作り（2026-11-03 の要件ディスカッションで振る舞いは確定。要件 4）: 条件で絞らずに名前の全候補で並びを作り、取り出すときに評価する。見て漏れた候補もその巡では使ったものとする（brief の「外れた札を残す」は改めた）。巡が尽きたときに、候補を集め直すか、使った印を戻してシャッフルし直すか、次の位置を巻き戻すかは設計で決める（シーンの集まりは辞書確定の後に変わらないので、どれでも結果は同じ。性能の良いものを選ぶ。開発者の示唆は「使った印を戻してシャッフルだけやり直す」）。`＆優先` の段との順。条件の無い名前で乱数の消費を今と同じにする方法。
2b. `＆優先` の段と並び（2026-11-03 の要件ディスカッションで振る舞いは確定。要件 5）: 優先は独占。1 回の選択で、全候補の条件から「今の段」（満たす候補の最大の優先）を決め、並びの今の巡から今の段の満たす札を出す。開発者の示唆は「シャッフルの後に優先で並べ替える（同じ優先の中はシャッフルの順を崩さない）。今の段より小さい札に来たら巡をやり直す」。この作りでは上書きの前後で下の段の巡もやり直しになり、同じ話が続けて出にくい性質が少し崩れる。段ごとに別の並びを持てば下の段の巡を保てる。どちらでも出るものの決まりは同じなので、設計で選ぶ。やり直しは 1 回の選択で 1 回まで（今の段の札が必ずあるため）。
2a. 条件漏れの失敗表記（要件 3.1・3.7）: 13 種目のラベルの名前と表記。`failure_test.lua` の数と hello-pasta の `tutorial_stages_test.rs` を一緒に直す。
3. 「名前が無い」と「全部落ちた」を Lua まで分けて返す形と、Call の 5 段・時報の名前の段での扱い（未決の議題 1 の結論に従う）。
4. 自動転記の置き場（`EVENT.fire` の `create_act(req)` か `SHIORI_ACT.new` か）と、チェイントークの続きへの影響。
5. `＆優先` の不正な値を止める所（辞書確定のとき・`pasta_check` でも同じ誤りにするか）。
6. 検査で `＄＊` の代入を集める範囲（シーンの本文・ファイルの先頭・単語の中の代入など）。

### Research Needed

- L1 `current_scene[key]` の直の引きが、条件を通らずにシーンの関数を返す場合があるか（`current_scene` の表に入る鍵の形）。
- チェイントークの続きが `EVENT.drive` でどの act に resume されるか（新しい act の `var` と続きの `var` の関係）。
- Lua の値を Rust で読むとき（選択肢 A）の型の写し方: 倍精度の数・整数に見える小数・2^53 を超える整数・真偽値・`nil`。
- trace のログを Rust の `tracing` と Lua の `@pasta_log` のどちらに出すと、`OnPastaSetSaveVariable` で試す作者が読みやすいか（`debug/` のマニュアルの今のログの読み方）。
- ~~`recommend.lua` 281 行の `SCENE.co_exec` と 283 行の完全一致の引きの分類~~ → 要件ディスカッションで閉じた。281 行（項目にシーンが無いときの `＊OnRecommendsiteChoice` の前方一致）は 2.5 で絞る、283 行（項目のシーンの完全一致）は 2.9 で絞らない。

### ファイルの重なり

- 同じウェーブには自分 1 本だけ。後の `scene-pick-modes` と `scene_table.rs`（`select_from_cache`・`SceneCacheKey`・`filter_by_attributes`）・`search/context.rs`・`scene.lua`・`validate/report.rs` を共有する。袋の新しい形は、あちらが重みの札と順番を足せるように決める。
- `search/context.rs` に足すテストは別のファイルに置く（ウェーブの約束）。`grammar.pest`・パーサ・`pasta_lsp`・VSCode の文法定義には触らない。

---

# 設計フェーズの調べと決定（2026-11-03・`/kiro-design` の生成。ワークツリーの base は main `72f7596c` ＋ `f6268329` のテストの差分だけ）

## Summary

- **Feature**: `scene-condition-attributes`
- **Discovery Scope**: Extension（既存の検索の芯と Lua の経路の拡張。新しい依存なし。軽い調べ＝`design-discovery-light`）
- **Key Findings**:
  - 名前で選ぶ所は `act:find_act_handler` の L2・L5 と `choice_select.lua` の直の `SCENE.search` の 2 つの口に集まる。イベントのシーンは全部 `SCENE.co_exec` → `find_handler` を通る。名指しは `SCENE.get_exact` だけで、表を渡さなければ条件を見ない形で分けられる。
  - 今の並び（`CachedSelection { candidates, next_index }`）は「次の位置より前＝見た」なので、「見て外れた候補もその巡では使った」（要件 4.4）は印を持たずに次の位置を進めるだけで書ける。並びの形を変えずに済む。
  - Call の今の失敗（`SCENE_NOT_FOUND`）は止まらない。要件 3.1 の「止める（エラー停止）」は、今の「止まるエラー」（`EVENT.drive` → `STOP.hold` → `STOP.response`）に 3 種目の種類を足す形で入る。

## Research Log

### L1 `current_scene[key]` が条件の抜け道になるか（Research Needed 1）

- **Sources**: `act.lua` `find_act_handler`（358〜399 行）、`scene.lua` `create_global_table`・`SCENE.register`、`scene_registry.rs` `registered_name`（230 行、`名前_番号`）。
- **Findings**: `current_scene` は `STORE.scenes[グローバルの登録名]` の表で、鍵は `__global_name__`・`__start__`・ローカルの登録名（`締め_1` の形）・Lua ブロックの `SCENE.名前` の関数、メタ表の `create_word`。`＞締め` は `締め` を引くので L1 に当たらず L2 へ進む。当たるのは登録名を番号ごと書いた `＞締め_1` と Lua ブロックの関数だけ。
- **Implications**: L1 は「名前で候補を選ぶ所」ではなく 1 本を名指しする段とみなし、条件を通さない（設計の Out of Boundary）。マニュアルの段の表の説明（「Lua ブロックで定義した関数など」）のままでよい。

### チェイントークの続きの act（Research Needed 2）

- **Sources**: `event/init.lua` `EVENT.fire`（300〜335 行）・`EVENT.drive`（231〜277 行）、`act.lua` `ACT_IMPL.yield`（641〜655 行）、`virtual_dispatcher.lua` `check_talk`（225〜230 行）。
- **Findings**: 続き（`STORE.co_scene`）は `EVENT.drive(co, act, act)` で再開されるが、コルーチンの中のシーンは `init_scene` が返した自分の `save, var` を使い続ける。再開の値として渡る新しい act は `ACT_IMPL.yield` が `.req` を読むだけ。
- **Implications**: 自動転記を `create_act` で新しい act に入れれば、続きの `var` は上書きされない（要件 6.5 は作りのまま満たす）。

### Lua の値を Rust で読む型（Research Needed 3）

- **Sources**: `runtime/finalize.rs` `scene_attr_from_item`（`#[allow(clippy::useless_conversion)]`。x86 では `mlua::Integer` が i32）、`persistence.rs`（`save` は素の表。配列だけ `array_metatable`）、`shiori/act.lua` `SHIORI_ACT.new`。
- **Findings**: `var`・`save` は素の表なので `raw_get` で読める（メタメソッドを呼ばない）。数は `Integer` を `i64::from` で受けて f64 にし、`Number` はそのまま。2^53 を超える整数は Lua に入った時点で丸まっているので、右辺の `i64` も f64 にして比べれば Lua で書いたときと同じ結果になる。真偽値は 1・0。
- **Implications**: 評価は f64 と文字列の 2 つの形に揃える `Operand` で書く。x86・x64 で同じ（`cfg` で分けない）。

### trace をどちらのログに出すか（Research Needed 4）

- **Sources**: `runtime/log.rs`（`@pasta_log` は Rust の `tracing` へ橋渡し）、`book/src/reference/pasta-toml.md` 175・761〜773 行（`[logging] filter` は tracing の EnvFilter の書式）。
- **Findings**: Lua の `@pasta_log` も Rust の `tracing` も同じログファイルに出る。Rust の target（`pasta_lua::search`）で出せば `filter = "info,pasta_lua::search=trace"` で条件の trace だけを足せる。
- **Implications**: 評価を Rust に置き、`tracing` で出す。`pasta_core` には `tracing` を足さない（評価と記録は `pasta_lua` の述語の中）。

### 止まるエラーの運び方（設計で足した調べ）

- **Sources**: `failure.lua`、`shiori/stop.lua`（`STOP.response` は種類を「時間切れ」か「スクリプトのエラー」に決める）、`shiori/entry.lua`（`error_handler` は文字列でないエラーを `nil` にする）、`event/callback.lua` の `time_out`、`book/src/reference/runtime-failures.md`「止まるエラーの表記」。
- **Findings**: `EVENT.drive` は投げられた値をそのまま `STOP.hold` に控える（表でも残る）。`EVENT.fire` の `error(yielded_value)` も表なら位置を付けずにそのまま投げる。コルーチンの外（選択肢のハンドラ）で投げた値は `entry.lua` の `error_handler` を通るので、表を通すように直す必要がある。
- **Implications**: `FAILURE.raise` がメタ表付きの記録を投げ、`STOP.response` がそれを見分けてラベルを使う。`entry.lua` は記録を通し、500 の理由は `tostring(記録)`。直すのは 3 ファイルで数行ずつ。

### `＆優先` の不正な値を止める所

- **Sources**: `code_gen/scope_gen.rs` `append_set_scene_attrs`（属性ごとの `span` がある）、`pasta_check/src/validate/transpile_rules.rs`（pasta_lua のトランスパイルが失敗する書き方を写す段）、`pasta_check/tests/validate_parity_test.rs`（読み込み・トランスパイルの成否の一致）。
- **Findings**: 辞書確定（`finalize.rs`）では位置が分からない。トランスパイルでは属性の位置があり、pasta_check は同じ条件を `transpile_rules.rs` に写す決まりがある。
- **Implications**: トランスパイルエラー＋pasta_check の誤り `transpile-invalid-priority` にする。値の判定は `pasta_core` の `priority_value` を両方が使う。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A' 述語を渡す（採用） | pasta_core の抽選が `FnMut(&SceneInfo) -> bool` を受け、pasta_lua が Lua の表で評価する述語を作る | 並びの規則が 1 か所。pasta_core に mlua・tracing を入れない。ログは Rust の target で絞れる | 述語の中の Lua 読み出しのエラーを扱えない（素の表の `raw_get` なので起きない） | gap 分析の A を、pasta_core の言語非依存に合わせて変えたもの |
| A 値を pasta_core へ写す | 変数の値の列挙型を pasta_core に定義し、評価も pasta_core に置く | 評価の規則を pasta_check と共有できる | pasta_check は評価を使わない（検査は代入の有無だけ）。ログに tracing を足すか結果を返す仕組みが要る | 使い手の無い共有になる |
| B Lua で評価 | Rust が並びの順を返し、Lua が選んで消費を伝える | Lua の型のまま評価できる | 並びの規則が 2 か所に割れ、1 回の選択で 2 往復 | 採らない |
| C Lua の判定関数を Rust から呼ぶ | 検索の口に Lua の関数を渡す | 評価を Lua に置ける | 借用とエラーの伝え方が重い。ログが 2 か所 | 採らない |

## Design Decisions

### Decision: 評価の置き場と口

- **Context**: 変数は Lua、並びは Rust（要件 1・2・4）。
- **Alternatives**: 上の表の A・B・C。
- **Selected Approach**: A'。`@pasta_search:search_scene(name, gsn, var, save)`。表を渡さなければ条件を見ない（名指し）。
- **Rationale**: 並び・段・巡の規則を 1 か所に置き、`scene-pick-modes` が同じ所に重みと順番を足せる。Lua 側は act を運ぶだけで、`act.lua` に足すのは 15 行ほど（1,000 行の約束に触れない）。
- **Trade-offs**: `@pasta_search` の引数が増える（省略すれば今のまま）。
- **Follow-up**: Lua から `SEARCH:search_scene` を直に呼ぶゴーストは、表を渡さないと条件を見ないことをマニュアルに書く。

### Decision: `＆優先` の段の持ち方（1 本の並び）

- **Context**: 優先は独占（要件 5.7〜5.9）。開発者の示唆は「シャッフルの後に優先で安定に並べ替え、段より低い札に来たら巡をやり直す」と、代わりの「段ごとの並び」。
- **Alternatives**: 1) 1 本の並び＋並べ替え 2) 段ごとに別の並びと次の位置。
- **Selected Approach**: 1)。今の段は 1 回の選択ごとに全候補を評価して求める（状態に持たない）。
- **Rationale**: 状態の形が今と同じ「並び＋次の位置」で、足すのは並べ替えと段の判定だけ。始め直しが 1 回で必ず選べることも、並べ替えから直ちに言える。
- **Trade-offs**: 上の段を出すたびに下の段の巡が始め直しになる（巡の境目で直前の話が続けて出うる、今の境目と同じ種類の弱み）。段ごとの並びなら避けられる。
- **Follow-up**: 設計ディスカッションで確かめる（OPEN QUESTION 2）。

### Decision: 満たす候補が 1 つも無いときの並びの扱い

- **Context**: 要件 4.4・4.6 を手順どおりに行うと「今の巡の残りを見る → 始め直す → 全部見る」で、終わった後は「新しい巡を全部見た」状態になる。
- **Selected Approach**: 先に全候補を評価して空なら、並びの次の位置を末尾にするだけ（シャッフルしない）。次の選択で始め直しになるので、選ばれ方は手順どおりと同じ。乱数の消費が 1 回少ないだけ（固定の `MockRandomSelector` は毎回先頭から使うので、テストの結果も同じ）。
- **Rationale**: 空回りしない（要件 4.6）ことが作りで明らか。

### Decision: 条件漏れを止まるエラーにする

- **Context**: 要件 3.1「条件漏れの失敗表記を出して止める（エラー停止）」。今の Call の失敗は止まらない。
- **Alternatives**: 1) 止まるエラー（シーンが止まり続きは捨てる） 2) 今の `SCENE_NOT_FOUND` と同じ「表記を出して次の行へ」。
- **Selected Approach**: 1)。`FAILURE.raise` で投げ、`STOP.response` が 13 種目のラベルで表記にする。
- **Rationale**: 要件の「止める」「止まって見え」を文字どおりに読んだ。
- **Follow-up**: OPEN QUESTION 1 で確かめる。2) なら `FAILURE.raise` と `stop.lua`・`entry.lua` の変更が要らず、`FAILURE.report` に置き換えるだけで済む。

### Decision: 宣言の文字列の絞り込み（`filters`）を消す

- **Context**: `filter_by_attributes` は本番でいつも空で呼ばれ、受け手の `call-attribute-filter` は閉じた。`ponytail-core-registry` が仕組みを残した理由はこの spec が使うためだった。
- **Selected Approach**: 消して、同じ位置の引数を述語に替える。`NoMatchingScene` は「条件漏れ」の意味で使い直す。
- **Rationale**: 使い手の無い 2 つ目の絞り込みを残さない。宣言の意味は変わらない（要件 10.1）。
- **Trade-offs**: `pasta_core` の公開 API の破壊的な変更。`scene_table_resolve_filter_tests.rs` を書き直す。

### Decision: 自動転記は `create_act` に置く

- **Selected Approach**: `EVENT.fire` の `create_act` で `transfer_req_to_var`・`transfer_date_to_var` を呼ぶ。`SHIORI_ACT.new` には置かない（中断のシーン・止まるエラーの表記だけの act にまで入れない）。時報の前の転記は残す（同じ値の上書きで無害）。

## Synthesis

- **Generalization**: 「名前で候補を選ぶ」の全経路を、`SCENE.search(name, gsn, act)` と `find_act_handler` の 2 つ目の戻り値に揃えた。名指しは「表を渡さない」で同じ口の別の使い方になり、新しい関数を足さない。`scene-exact-match` の `＞＞` も表を渡すだけで条件が効く。
- **Build vs Adopt**: 新しいライブラリは使わない。数の判定は正規表現を足さず数行の手書き（`-?[0-9]+(\.[0-9]+)?`）。止まるエラーは既存の `STOP` の仕組みに乗せる。
- **Simplification**: 並びの状態は今の 2 つのまま（印の配列を持たない）。今の段は状態に持たず毎回求める。`filters` を消す。新しい Lua のファイルを作らない（評価は Rust）。

## 使ったスキル・ガイド

- `kiro-spec-design` の `design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`、`.kiro/settings/templates/specs/design.md`・`research.md`。
- `kiro-validate-design` の `design-review.md`（設計の検証の報告に使った）。
- ponytail（最小の変更）: 新しい部品を評価と検査の 2 つに絞った。

## Risks & Mitigations

- 並びの作り直しで、条件の無い名前の乱数の順が変わる — 述語なし・優先が同じなら今と同じ手順・同じ回数の `shuffle_usize` になることを単体テストで固定する（固定の乱数の既存のテストもそのまま通す）。
- 止まるエラーの記録が表なので、文字列を前提にした所（`RES.err`・`error_handler`）で崩れる — `tostring` を `FAILURE.body` にし、`entry.lua` で理由を文字列にする。`callback.lua` の時間切れの経路も `STOP.response` を通るのでテストに 1 本入れる。
- 自動転記で、自分で `＄季節` などに入れているゴーストの挙動が変わる — 本文の代入が後から勝つので、本文で入れているものは今のまま。選ぶ前の条件で読む値だけが変わる。マニュアルに注意を書く（要件 6.6）。
- `gen-skill-refs.mjs` の書き先 `.claude/skills/` の書き換えが分類器に止められることがある — 止まったら入った分を示して許可を求める。

## 設計ディスカッションへの申し送り（OPEN QUESTIONS）

1. **「エラー停止」の意味**（要件 3.1・3.2）。仮定: 止まるエラー（シーンが止まり、チェイントークの続きは捨てる。表記 `【条件に合うシーンが無い：＞告白（＊放課後）】`）。代わりの案: 今の `【シーンが見つからない】` と同じく表記を出して Call の次の行へ進む（止まらない）。
   ```pasta
   ＊放課後
   　＄気分＝「普通」
   　＞告白
   　さくら：（ここは出るか）
   ＊告白　＆＄気分＝「高」
   　さくら：……わたしも、好き。
   ```
   仮定では「【条件に合うシーンが無い：＞告白（＊放課後）】」で止まり、最後の行は出ない。代わりの案では表記の後に最後の行も出る。
2. **`＆優先` の段の持ち方**（要件 5.7〜5.9）。仮定: 1 本の並びを優先の降順に安定に並べ替える。代わりの案: 段ごとに別の並び。違いは下の段の巡の続き方だけ（出るものの決まりは同じ）。
   ```
   ＊会話（優先 0）A・B・C、＊会話 ＆優先：1 ＆＄＊記念日（今日だけ立つ）K
   巡: A → B →（記念日が立つ）K → K → …（翌日）
     仮定: 新しい巡で A・B・C をシャッフルし直す（C を待たずに B がまた先に出うる）
     代わりの案: 下の段の巡の続きの C から出る
   ```
3. **13 種目のラベルの名前**（要件 3.7）。仮定: `SCENE_UNMET = "条件に合うシーンが無い"`。代わりの案: `"条件漏れ"`（要件の言い方）、`"選べるシーンが無い"`。
4. **`OnPastaCallFunction` の条件漏れ**（要件 2.8）。仮定: 止めずに、今の「見つからない」と同じ形の表記の応答（`【条件に合うシーンが無い：OnPastaCallFunction ＞告白】`）。代わりの案: Call と同じく止まるエラー。外からの呼び出しの今の決まり「失敗では表記とログの警告を出し、場面の続きを変えない」に合わせた。
5. **expr モード（`＠名前（）` がシーンに当たる道）**。仮定: 同じ `SCENE.search` を通るので条件で絞り、条件漏れは今の「関数が見つからない」のまま（新しい扱いを足さない）。代わりの案: expr モードでは条件を見ない（`search_dictionary` で表を渡さない）。
6. **名指しの再生と `＆優先`**（要件 2.9）。仮定: 名指し（表を渡さない）は条件も段も見ないが、並びは優先の降順に並ぶ（同じ名前の全部を順に出す。高い優先が各巡の先に出る）。代わりの案: 名指しでも段の独占を効かせる（`OnPastaScene` で優先の低い場面が出せなくなる）。
   ```
   ＊OnUpdateComplete ＆優先：1 ＆＄ｒ０＝「changed」 と ＊OnUpdateComplete
   OnPastaScene OnUpdateComplete を 2 回 → 仮定: 1 回目は優先 1、2 回目は優先 0（両方を確かめられる）
   ```
7. **数として読む文字列の形**（要件 1.10）。仮定: `-?[0-9]+(\.[0-9]+)?`（`+5`・`.5`・`1.`・`1e3`・前後の空白は文字列）。代わりの案: 符号 `+` や指数も数にする。
