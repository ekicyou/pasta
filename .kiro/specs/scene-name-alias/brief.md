# Brief: scene-name-alias

> **ステータス**: 着手中（2026-10-08 の `/kiro-discovery` で起票、同日に最優先で先行開始）。Phase 11 の残りに先行し、`scene-attribute-store`・`call-attribute-filter` はこの後に rebase する。入門ガイド（Phase 12 の `hello-pasta-tutorial-stages`）がこの spec に依存する。着手するときは `/kiro-start scene-name-alias` で開始する。

## Problem

ランダムトークのシーンは `＊OnTalk` と書くしかない。OnTalk は pasta が自分で決めた仮想イベントの名前なのに、ベースウェアのイベント名と同じ英語の見た目で、辞書の中で最もたくさん書く行が「システムの都合」を感じさせる。同じ仮想イベントの OnHour には既に `時報HH`・`時報その他` の日本語名があり（dispatcher の 4 段フォールバック）、OnTalk だけが英語名のまま取り残されている。

困るのは辞書を書くゴースト作者、とくに初心者。入門ガイド（Phase 12）が最初に見せる辞書の中心が `＊OnTalk` の繰り返しになる。

## Current State

- 仮想ディスパッチャ `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` の `check_talk` は `create_scene_thread("OnTalk", act)` と名前を直書きしている。`check_hour` は `{ "時報HH", "OnHourHH", "時報その他", "OnHourOther" }` の順に探す。
- シーン名の正規化は `SceneRegistry::sanitize_name`（`crates/pasta_core/src/registry/scene_registry.rs`）に 1 か所へまとめられ、登録側（`code_gen/scope_gen.rs`）と検索側（`search/context.rs`）の両方がこれを通る（`scene-search-key-normalization` の成果）。
- グローバルシーンの検索は前方一致。`＊OnTalk朝` も OnTalk の候補になる。
- 単独の `＊` 行は「直前のグローバルシーンと同名の別候補を開始」（先頭ではパースエラー）という既存の意味を持つ。実際の辞書での使用は 0 件だが、文法として維持する（2026-10-08 に、里々流の「`＊` ＝ OnTalk」案と衝突するため検討し、現行維持を選んだ）。
- pasta.toml にシーン名を扱うセクションは無い（`[talk]` はウェイトと禁則の設定で、他のキーを混ぜない）。
- マニュアルは `＊会話` を汎用の例題名として 21 か所で使う（`grammar/action-line.md`・`actor-dictionary.md`・`block-structure.md`・`call-jump.md` の `＊会話・朝` など）。

## Desired Outcome

- `＊会話` と書いたシーンが、何も設定しなくても OnTalk のシーンとして登録され、ランダムトークとして発行される。既存の `＊OnTalk` もそのまま動く。
- 「会話」は OnTalk の別名として、宣言行だけでなく、シーン名を書く全ての位置（`＞`・`＞＞` の Call/Jump、選択肢の飛び先、Lua の `SCENE.co_exec`・`act:call` など、検索の入口を通る全経路）で同じ 1 つの名前として扱われる。
- 別名の表は pasta.toml で作者が定義でき、定義が無いときは「会話 → OnTalk」の 1 件だけが既定で入っている。表を定義した作者は既定を外すことも（空の表）、増やすことも（`時報`→`OnHour` のような自分の別名）できる。
- 置き換えは**完全一致のみ**。`＊会話・朝` は別名の対象外で、`会話・朝` という別のシーンのまま。
- マニュアル（`grammar/block-structure.md`・`grammar/call-jump.md`・`lua/shiori-events.md` の仮想イベント・`reference/pasta-toml.md`）が別名の規則と既定表を書き、`＊会話` を汎用の例題名に使っている箇所は、別名の意味と食い違わないように整えてある。スキル references は再生成してある。

## Approach

2026-10-08 のディスカバリで決めた（案 A・完全一致）。

- **別名はキー正規化規則の一部にする**: `SceneRegistry::sanitize_name` を通る場所、つまり登録と検索の両側で、名前が別名表のキーに完全一致したら値へ置き換える。これで宣言・Call/Jump・選択肢・Lua API のどこで書いても同じ名前になる。置き換えは 1 段だけ（連鎖しない）。置き換えてから既存のサニタイズを行う。
- **既定表は 1 件**: `会話 → OnTalk`。pasta.toml に表が書かれていれば、その表が既定を**丸ごと置き換える**（既定とマージしない。既定を残したい作者は同じ行を書く）。
- **検索は前方一致のまま**: 別名の置き換えは完全一致で行い、その後の検索の前方一致は現行どおり。`＞会話` は `OnTalk` の検索になり、`OnTalk朝` も候補になる（今の `＞OnTalk` と同じ挙動）。
- **dispatcher は触らない**: `check_talk` の `"OnTalk"` 直書きは、登録側で `会話` が `OnTalk` になるので変更不要。

却下した案:

- **OnTalk だけの特別扱い（`check_talk` に `会話` を足す）**: OnHour の 4 段フォールバックと同形で最小だが、「会話」が Call で呼べない非対称が残り、作者が自分の別名を足せない。
- **宣言行 `＊` だけの書き換え**: `＞会話` が OnTalk のシーンに当たらず、「別名」と言いながら呼び出せない。
- **里々流の単独 `＊` ＝ OnTalk（pasta.toml フラグ付き）**: 単独 `＊` の既存の意味（直前と同名の別候補）と同じ 1 行を取り合い、両立できない。現行文法の維持を選んだ。

## Scope

- **In**:
  - pasta.toml の別名表の読み込み（セクション名・形は要件で決める。候補: `[scene.alias]` の下に `"会話" = "OnTalk"`）。未定義なら既定の 1 件。
  - キー正規化への別名の組み込み（登録・検索の両側。`pasta_core` の `sanitize_name` と、それに表を渡す経路）。
  - ソースマップ・デバッグ（`debug/source_map/scene_join.rs`・`playscene.rs`・`dap/decode.rs` など `sanitize_name` を使う箇所）が `＊会話` のブレークポイントと再生で壊れないこと。
  - ログ・失敗表記で、作者が書いた名前と置き換え後の名前のどちらを出すかの決定と実装。
  - マニュアルの更新と、`＊会話` を汎用の例題名に使っている箇所の整理（別名であることを明示する例へ寄せるか、別の例題名へ付け替えるか）。スキル references の再生成。
  - テスト（`pasta_core` の登録・検索、`pasta_lua` の結合、`pasta_shiori` の E2E で `＊会話` がランダムトークとして出ること）。
- **Out**:
  - hello-pasta の辞書（`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/`）を `＊会話` に書き換えること。Phase 12 の `hello-pasta-tutorial-stages` の持ち場（下の Downstream）。
  - OnHour の 4 段フォールバックの変更。既定表に `時報 → OnHour` は入れない（`時報HH` は完全一致で表現できず、現行の仕組みが既に機能している）。
  - 部分一致・前方一致・正規表現による置き換え。
  - 単独 `＊` 行の意味の変更。
  - アクター名・単語名の別名。

## Boundary Candidates

- 設定: pasta.toml の表の読み込みと既定値（`pasta_lua` の `config.rs`、`reference/pasta-toml.md`）。
- 正規化: 別名の適用を `sanitize_name` の前段に置き、登録・検索・ソースマップの全呼び出し元へ効かせる（`pasta_core`、`pasta_lua` の `scope_gen.rs`・`search/context.rs`・`debug/`）。
- 文書: マニュアルの規則の記述と例題名の整理、スキル references。

## Out of Boundary

- 仮想ディスパッチャのロジック（発行条件・チェイントーク）。
- シーン検索の前方一致・シャッフルの規則。
- LSP・pasta_check への pasta.toml 読み込みの追加（下の Constraints）。

## Upstream / Downstream

- **Upstream**: `scene-search-key-normalization`（正規化の 1 か所化。別名はその前段に乗る）、`scene-identity-format`（ランタイム名の区切り文字。別名の置き換えが区切りを壊さないこと）。いずれも完了済み。
- **Downstream**: `scene-attribute-store`・`call-attribute-filter`（`pasta_core` のシーン登録・`search/` をこの spec の後に触る）、`hello-pasta-tutorial-stages`（hello-pasta の `＊OnTalk` を `＊会話` に切り替えるかを、段階表の中で決める。`first-ghost.md` との逐語照合も同じ spec が持つ）、`getting-started-story-guide`（入門ガイドの説明が「会話」で語れるようになる）、`pasta-ghost-authoring` スキル（references 再生成で追従）。

## Existing Spec Touchpoints

- **Extends**: なし（新規 spec）。
- **Adjacent**:
  - `scene-attribute-store` — `pasta_core` のシーン登録と `grammar/block-structure.md` を共有。この spec が先に入り、向こうが rebase する。
  - `call-attribute-filter` — `search/context.rs`・`scene_table.rs` を共有。この spec が先に入り、向こうが rebase する。
  - `failure-output-unification` — 失敗表記に出す名前（作者が書いた名前か置き換え後か）は、一本化した仕組みの上で決める。
  - `hello-pasta-tutorial-stages` — hello-pasta の `dic/` はこの spec では触らない。

## Constraints

- **完全一致のみ**（2026-10-08 決定）。混乱を防ぐため、部分一致や前方一致での置き換えは入れない。
- **既定表は 1 件・丸ごと置き換え**: 表を書いた作者の意図を優先し、既定とマージしない。
- **LSP と pasta_check は pasta.toml を読まない**: エディタの診断・定義ジャンプは別名を知らない。ランタイムとの食い違い（`＊会話` の定義へ `＞OnTalk` から飛べない等）は許容し、マニュアルに書く。LSP に表を読ませるかは別の動機が生じたときに起票する。
- **マニュアルが権威**: 挙動を変えた同じ変更でマニュアルを直し、`node book/tools/gen-skill-refs.mjs` で references を再生成する。
- **Phase 11 の持ち場の規則**: `act.lua`・`element_gen.rs` は触らない見込み。`search/`・`scene_table.rs`・`pasta_core` のシーン登録は Wave 4〜5 の spec も触るが、この spec を最優先で先行させるので、後続が rebase で合わせる（2026-10-08 決定）。

## 要件フェーズで決めること

- pasta.toml のセクション名と形（`[scene.alias]` テーブル案を推奨。`[talk]` には混ぜない）。値の型が合わないときの扱いは既存の 3 分類表に従う。
- 別名の値がさらに別名のキーだった場合（連鎖）は 1 段だけとするか、設定エラーにするか。
- ログ・失敗表記・デバッグ UI に出す名前（作者が書いた名前／置き換え後の名前／両方）。
- マニュアルの `＊会話` 例題 21 か所の扱い（別名の意味で使う例へ寄せる／別の例題名へ付け替える）。
- ローカルシーン名（`・名前`）は対象外でよいか（表はグローバル名のみを対象とする想定）。
