# トーク出力とアピアランス

> 【考え中】ゴーストの台詞は、どんな工房を通って、ウェイトや改行の効いたさくらスクリプトに仕立て上がるのかしら――組立、後処理、そして表情と着せ替えまで、舞台衣装の裏側をご覧に入れますわ。

> 【アンソニー】採寸から仮縫いまで、お嬢様おひとりで針をお持ちになるのでございますか。

> 【驚き】わたくしひとりで、ですって？ 針を持つのは、Lua と Rust の二人がかりですわよ！

---

この章では、ACT に積まれたトークからさくらスクリプトを組み立てる出力処理と、サーフェス・着せ替えの観測と復旧（アピアランス）を扱う。ウェイトの長さ・自動改行の幅・外見の復旧規則など、利用者から見た振る舞いは [@pasta_sakura_script](../lua/modules/pasta-sakura-script.md)・[pasta.toml リファレンス](../reference/pasta-toml.md)・[アクター辞書](../grammar/actor-dictionary.md#同一スポット共有時の外見の復旧) が正である。

## 目的と責務

トーク出力は、シーンの実行中に ACT が積んだトークンの列を、ベースウェアへ返す 1 本のさくらスクリプト文字列にする。

この章が責務を持つのは次の事項である。

- ACT の `build` が行う前処理: トークンをアクターの切り替わりで束ねるグループ化と、連続する発言の結合
- `pasta.shiori.sakura_builder` による組立: トークンの種類ごとの出力、スポットの解決と `\p[N]` の出力、段落区切りの改行、選択肢の前後の行の区切り、終端の `\e`
- `@pasta_sakura_script` の後処理（Rust）: さくらスクリプトの単位（タグ・エスケープ・囲み）と文字を分けるトークナイザ、ウェイトの挿入、budoux による改行
- アピアランス: 出力した文字列からサーフェス変更・着せ替え・スコープ切替を観測して記録し、アクターの切り替え時に外見を復旧するタグを出力する仕組み
- 装いの出し入れ: `pasta.toml` の `[actor."名前"]` の `font`（装い）のタグ列を、話者の切り替えと `\c`・`\x` の後に出力し、`\f[default]` で既定の表示へ戻す仕組み

次の事項は他の章が権威を持ち、この章では再記述しない。

- `build` がいつ呼ばれ、その戻り値がどう応答になるか（`act:yield()`・シーン終了時の `build`・再開のループ）は [ランタイム実行モデル](execution-model.md#イベントからシーンへ) で扱う。
- `@pasta_sakura_script` を VM に登録する継ぎ目（`RendererInjection`）と presentation マーカーは [SHIORI 層](shiori.md#presentation-マーカーとレンダラ注入) で扱う。
- ACT のメソッドとフィールドの個々の仕様は [Lua ランタイム内部モジュール](internal-modules.md#act-の内部) で扱う。シーン関数がどの ACT のメソッドを呼ぶかは [トランスパイラ](transpiler.md#生成される-lua-コードの形) で扱う。
- 文字の分類・ウェイトの値・budoux の幅の指定方法は [@pasta_sakura_script](../lua/modules/pasta-sakura-script.md) と [pasta.toml リファレンス](../reference/pasta-toml.md#talkトーク表示制御) が正である。段落区切りの改行量は [spot_newlines](../reference/pasta-toml.md#spot_newlines)、台詞の後の選択肢の前の改行量は [choice_newlines](../reference/pasta-toml.md#choice_newlines)、選択肢の後の改行量は [after_choice_newlines](../reference/pasta-toml.md#after_choice_newlines)、budoux の数え方は [budoux](../reference/pasta-toml.md#budoux)、スポットの決まり方は [バルーン連携](../grammar/actor-dictionary.md#バルーン連携)、外見の復旧の規則と記録できない指定は [同一スポット共有時の外見の復旧](../grammar/actor-dictionary.md#同一スポット共有時の外見の復旧)、装いの書き方と利用者から見た出し入れは [装い（font）](../reference/pasta-toml.md#装いfont) と [1 つのスポットを複数のアクターで共有する](../grammar/actor-dictionary.md#1-つのスポットを複数のアクターで共有する) が正である。
- `font` の検査とタグ列への置き換えは [ローダ自己展開とモジュール解決](loader.md#設定読込) で扱う。

## 構成要素

### 全体の経路

```text
シーン関数
  act:actor_proxy("アクター"):talk(…) / :sakura_script(…)、act:surface(…) / :wait(…) / :set_spot(…) …
    │  フラットなトークンの列（act.token）
    ▼
pasta.shiori.act  SHIORI_ACT_IMPL.build
    │  ACT_IMPL.build（pasta.act）
    │    group_by_actor          … アクターの切り替わりで type = "actor" のグループに束ねる
    │    merge_consecutive_talks … グループ内の連続する talk を 1 つに結合する
    │  グループ化トークンの列
    ▼
pasta.shiori.sakura_builder  BUILDER.build(グループ化トークン, { spot_newlines, choice_newlines, after_choice_newlines }, STORE.actor_spots, STORE.appearance)
    │  アクターの切り替え: \p[spot] → APPEARANCE.restore（復旧タグ） → STYLE.switch（装いの出し入れ）
    │  talk / sakura_script: @pasta_sakura_script.talk_to_script(アクター, テキスト, スポットの行の状態)
    │                          └ Rust: トークナイザ → ウェイト挿入 → budoux 改行（行の状態を読み書きする）
    │  その他のトークン: 種類ごとのタグ
    │  内側の各片を STYLE.through に通し（\c・\x の後の装いの出し直し）、APPEARANCE.observe に観測させる
    ▼
さくらスクリプト文字列（末尾は \e）
```

### Lua 側

| モジュール（ファイル） | 役割 |
| ---------------------- | ---- |
| `pasta.act`（`crates/pasta_lua/pasta_scripts/pasta/act.lua`） | トークンの蓄積（`talk`・`sakura_script`・`raw_script`・`surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout`・`set_spot`・`clear_spot`）と、`build` の前処理（局所関数 `group_by_actor`・`merge_consecutive_talks`） |
| `pasta.actor`（`crates/pasta_lua/pasta_scripts/pasta/actor.lua`） | アクタープロキシ。`act:actor_proxy("アクター")` などで得たプロキシの `:talk(…)`・`:sakura_script(…)` を、アクターを添えた ACT の `talk`・`sakura_script` に委ねる |
| `pasta.shiori.act`（`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`） | `pasta.act` を継承し、`build` を「`ACT_IMPL.build` でグループ化トークンを得て `BUILDER.build` に渡す」処理に差し替える。`spot_newlines`・`choice_newlines`・`after_choice_newlines` は `[ghost]` から読む |
| `pasta.shiori.sakura_builder`（`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`） | グループ化トークンをさくらスクリプト文字列にする `BUILDER.build` |
| `pasta.shiori.appearance`（`crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua`） | 外見状態の生成（`APPEARANCE.new`）・観測（`APPEARANCE.observe`）・復旧（`APPEARANCE.restore`）と、さくらスクリプトの単位の読み取り（`APPEARANCE.tag_at`。ビルダーが字の判定と `\c` の読み取りに使う）。状態を持たず、渡された状態の表をその場で更新する。`STORE` や `@pasta_*` を `require` しない |
| `pasta.shiori.actor_style`（`crates/pasta_lua/pasta_scripts/pasta/shiori/actor_style.lua`） | 装いの状態の生成（`STYLE.new`）、話者の切り替えで出す戻すタグと装いのタグ（`STYLE.switch`）、内側のトークンの `\c`・`\x` の後の出し直し（`STYLE.through`）。状態を持たず、渡された状態の表をその場で更新する。タグの読み取りは `APPEARANCE.tag_at` だけを使い、`STORE` や `@pasta_*` を `require` しない |
| `pasta.buf`（`crates/pasta_lua/pasta_scripts/pasta/buf.lua`） | 組立に使う文字列バッファ。LuaJIT の `string.buffer` があればそれを、無ければ同じ `put`・`tostring` を持つ最小実装を使う |
| `pasta.failure`（`crates/pasta_lua/pasta_scripts/pasta/failure.lua`） | 失敗表記の `raw_script` トークンを積む報告の出口。`sakura_builder` は未登録の印を読む（[失敗の報告（pasta.failure）](internal-modules.md#失敗の報告pastafailure)） |
| `pasta.store`（`crates/pasta_lua/pasta_scripts/pasta/store.lua`） | ビルドをまたいで保持する `STORE.actor_spots`（アクター名 → スポット）と `STORE.appearance`（外見状態）を持つ |

### Rust 側（`crates/pasta_lua/src/sakura_script/`）

| ファイル | 役割 |
| -------- | ---- |
| `crates/pasta_lua/src/sakura_script/mod.rs` | `@pasta_sakura_script` のモジュール表を作る `register`。`talk_to_script` と `break_lines` の実装、アクター表からのウェイト値と改行の規則（`budoux`・`budoux_newlines`・`budoux_jump`）の読み取り、`talk_to_script` の 3 つ目の引数（行の状態の表）の読み書き |
| `crates/pasta_lua/src/sakura_script/tokenizer.rs` | `Tokenizer`（さくらスクリプトの単位（タグ・エスケープ・囲み）の正規表現 `SAKURA_TAG_PATTERN` と文字の集合 `CharSets`）と、文字の種類 `TokenKind` |
| `crates/pasta_lua/src/sakura_script/wait_inserter.rs` | ウェイト値 `WaitValues` と、トークン列に `\_w[ms]` を挿入する `insert_waits` |
| `crates/pasta_lua/src/sakura_script/line_breaker.rs` | budoux の分かち書きで改行を挿入する `break_lines_impl`。行の状態 `Column`、改行の規則 `BreakRule`、行を動かす単位の見分け（局所の `column_move` と動き `ColumnMove`） |

文字の集合とウェイトの既定値は `TalkConfig`（`crates/pasta_lua/src/loader/config/sections.rs`）が持つ。`[talk]` セクションを `TalkConfig` にする仕組みは [ローダ自己展開とモジュール解決](loader.md#設定読込) で扱う。budoux の分かち書きは依存クレート `budouy` の既定の日本語モデルを使う。

## 処理とデータの流れ

### トークンの蓄積

ACT のメソッドは、さくらスクリプトに依存しない表を `act.token` に積むだけで、文字列は作らない。トークンの形は次のとおりである。

| `type` | フィールド | 積むメソッド |
| ------ | ---------- | ------------ |
| `talk` | `actor`・`text` | `act:talk(アクター, テキスト)`（アクタープロキシの `talk` 経由を含む）。`text` が `nil` なら積まない（変数参照の展開なら、代わりに値の無い変数の失敗表記を積む） |
| `sakura_script` | `actor`・`text` | `act:sakura_script(アクター, テキスト)`（アクタープロキシ経由） |
| `raw_script` | `text` | `act:raw_script(テキスト)`。SHIORI 用の ACT では `set_property`・`get_property` も積む。失敗表記（[実行時の失敗の出力](../reference/runtime-failures.md)）は `pasta.failure` の `FAILURE.report` が積む（[失敗の報告（pasta.failure）](internal-modules.md#失敗の報告pastafailure)） |
| `raw_script` | `text`・`actor` | 未登録アクターの目印。`act:actor_proxy` が `FAILURE.report` を通して積む（[生成コード用のメソッド](internal-modules.md#生成コード用のメソッドactor_proxyglobal_fn)）。`text` は目印の表記で、失敗表記を出さない設定では空文字列になる |
| `surface` | `id` | `act:surface(ID)` |
| `wait` | `ms` | `act:wait(ミリ秒)`（0 以上の整数に丸める） |
| `newline` | `n` | `act:newline(回数)`（既定 1） |
| `clear` | — | `act:clear()` |
| `choice` | `target`・`display`・`scope` | `act:choice(ジャンプ先, 表示)`。`scope` は積んだ時点の `current_scene.__global_name__`（実行中のシーンが無ければ `nil`） |
| `choice_timeout` | `seconds` | `act:choice_timeout(秒)` |
| `spot` | `actor`・`spot` | `act:set_spot(名前, 番号)`。`act.actors` に無い名前なら積まない |
| `clear_spot` | — | `act:clear_spot()` |

`talk`・`sakura_script` と `actor` 付きの `raw_script`（未登録アクターの目印）だけが発言したアクターを持つ（話者のトークン）。`surface` などアクターを持たないトークンは、次のグループ化で直前の発言者のグループに入る（出力の先頭か `clear_spot` の後で、発言より前に積んだものはアクター `nil` のグループに入る）。

### グループ化トークン

`ACT_IMPL.build` は `act.token` を取り出して空にし、0 件のとき、またはすべてが `text` の空の `raw_script` のとき（失敗表記を出さない設定で残った目印だけの場合など） `nil` を返す。1 件以上なら次の 2 段で列を作り直す。

1. `group_by_actor` は列を先頭から走査する。
   - `spot` は、そのまま結果に置く。グループは閉じない。
   - `clear_spot` は、そのまま結果に置き、現在のグループを閉じる。後の発言と表示制御などは新しいグループに入る。
   - 話者のトークン（`talk`・`sakura_script`・`actor` 付きの `raw_script`）は、グループがまだ無いか、トークンの `actor` が現在のグループのアクターと別の表であれば、新しいグループ `{ type = "actor", actor = …, tokens = {} }` を結果に追加してから、そのグループの `tokens` に入れる。同じアクターなら現在のグループに入れる。
   - `actor` の無い `raw_script` は、グループがあればその `tokens` に、無ければ結果に直接置く。
   - それ以外（`surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout`）は、グループがあればその `tokens` に入れ、無ければアクター `nil` のグループ `{ type = "actor", actor = nil, tokens = {} }` を結果に追加してから、そのグループの `tokens` に入れる。グループが無いのは、出力の先頭か `clear_spot` の後で、まだ発言を積んでいないときである。
2. `merge_consecutive_talks` は、各グループの `tokens` の中で隣り合う `talk` のテキストを連結して 1 つの `talk` にする。`sakura_script` など `talk` 以外のトークンが間にあれば結合は切れる。

結果は、`type = "actor"` のグループと、`spot`・`clear_spot`・グループの外の `raw_script` が並ぶ列になる。アクター `nil` のグループは、発言より前に積んだ表示制御などを積んだ順に持つ。組立はこのグループでスコープ切替タグを出さず、内側の出力はアクター未指定（`nil`）として観測される（[さくらスクリプトの組立](#さくらスクリプトの組立)）。失敗表記（未登録アクターの目印を除く）は `actor` の無い `raw_script` のため、直前に話したアクターのグループの内側か、グループが無ければ最上位に置かれる。どちらもスコープ切替タグを出さず、グループも区切らないため、後に同じアクターが話しても切り替えの出力は増えない。

### トークンの種類と出力

`BUILDER.build` は、グループ化トークンの列の最上位とグループの内側で、種類ごとに次の出力をする。

| 位置 | `type` | 出力 |
| ---- | ------ | ---- |
| 最上位 | `spot` | 出力しない。`actor_spots[アクター名] = spot` を記録する |
| 最上位 | `clear_spot` | 出力しない。`actor_spots` の全エントリを消し、組立の状態（直前の発言者・スポット・段落区切りの判定・最後に表示したもの・行の状態）をリセットする |
| 最上位 | `actor` | アクターが直前の発言者と別なら切り替えの出力（次節）。続けて内側のトークンを順に出力する |
| 最上位 | `raw_script` | `text` をそのまま |
| 内側 | `talk`・`sakura_script` | `SAKURA_SCRIPT.talk_to_script(グループのアクター, text, 行の状態)` の戻り値（後処理済みの文字列） |
| 内側 | `surface` | `\s[id]` |
| 内側 | `wait` | `\_w[ms]` |
| 内側 | `newline` | `\n` を `n` 回（行の状態を `n` 行進める） |
| 内側 | `clear` | `\c`（行の状態を最初の行に戻す） |
| 内側 | `raw_script` | `text` をそのまま |
| 内側 | `choice` | `scope` が文字列で、`target` が `On`・`script:` で始まらなければ `\![*]\q[display,target,scope]`、それ以外は `\![*]\q[display,target]`（`display`・`target`・`scope` の `\`・`]`・`,` を `\` でエスケープする）。`scope` は選択時に OnChoiceSelectEx の Reference2 として戻る（[既定ハンドラ](shiori.md#lua-側の-shiori-エントリとイベント配送)） |
| 内側 | `choice_timeout` | `\![set,choicetimeout,ミリ秒]`（`seconds` の 1000 倍を切り捨て。`seconds` が無ければ 0） |
| 内側 | 上記以外 | 空文字列 |

`talk_to_script` を通るのは `talk` と `sakura_script` だけである。`surface`・`wait` などのタグ、`raw_script`、`choice` の出力は後処理を通らず、ウェイトも budoux の改行も入らない。`choice` と字を出すトークンの直前には、組立が行の区切り（`\n[N]` または `\n`）を出すことがある（[さくらスクリプトの組立](#さくらスクリプトの組立)）。

### さくらスクリプトの組立

`BUILDER.build(グループ化トークン, config, actor_spots, appearance)` は、`config.spot_newlines`（既定 1.5）・`config.choice_newlines`（既定 1.5）・`config.after_choice_newlines`（既定 1.0）と、`config.buffer_factory`（既定 `pasta.buf` の `new`。テストが差し替える）を使う。SHIORI 用の ACT は `actor_spots` に `STORE.actor_spots` を、`appearance` に `STORE.appearance` を渡し、`BUILDER.build` はどちらもその場で書き換える。`nil` を渡された場合は、そのビルドの中だけの空の表を使う。

組立は、1 回のビルドの中だけで生きる次の状態を持つ。

| 状態 | 意味 |
| ---- | ---- |
| `last_actor` | 直前に切り替えの出力をしたアクター（`nil` から始まる） |
| `last_spot` | 現在のスポット（現在のスコープ） |
| `spot_has_text` | スポットごとに、このビルドで字を出すトークン（字のある `talk`、字を表示する `sakura_script`）を出力したか |
| `pending_break` | 現在のスポットで、次の字を出すトークンの前に段落区切りの改行を出すか |
| `spot_tail` | スポットごとに、このビルドでそのスポットのバルーンに最後に表示したもの（`"text"` か `"choice"`。値が無ければ、まだ何も表示していない）。`last_spot` が `nil` の間は、スポット 0 として読み書きする |
| `columns` | スポットごとに、budoux の行の状態の表（`{ line, width, jump }`。[行の状態の表](../lua/modules/pasta-sakura-script.md#行の状態の表)）。局所関数 `column_at` が、無ければ `{ line = 1, width = 0, jump = false }` を作る。添え字は `spot_tail` と同じ（`last_spot` が `nil` の間は 0）。グループの頭で話し手のスポットの表を 1 度取り、グループの内側のトークンに渡す |

グループのアクターが `last_actor` と別の表であれば、アクターの切り替えとして次を行う（`emit_actor_switch`）。

1. スポットを `actor_spots[アクター名]` から引く。無ければ 0 とし、アクターに名前があれば警告をログに出す。`act:actor_proxy` が作るその場限りのアクター（未登録の名前）は `actor_spots` に無いため、ここで 0 になる。このアクターには未登録の印（`FAILURE.is_unregistered_actor`）があるため、警告は出さない（未登録アクターの警告が既に出ている）。
2. `\p[スポット]` を出力する。
3. 続けて `APPEARANCE.restore(appearance, アクター, スポット, グループの内側のトークン)` が返す復旧タグを出力する（[アピアランスの観測と復旧](#アピアランスの観測と復旧)）。
4. 続けて `STYLE.switch(style, アクター, スポット)` が返す戻すタグと装いのタグを出力する（[装いの出し入れ](#装いの出し入れ)）。
5. `pending_break` を `spot_has_text[スポット]` で決め直し、`last_spot`・`last_actor` を更新する。

`last_actor` は各ビルドの初めに `nil` なので、アクターを持つ最初のグループでは必ず `\p[N]` が出る。グループのアクターが `nil` のときは切り替えの出力をせず、現在のスコープのまま内側を出力する。

内側のトークンは、種類と状態から次の表で行の区切りを決める。字を出すトークンと `choice` は、区切り・状態の更新・トークンの出力の順に処理する。区切りはトークンの出力の直前（`choice` のマーカー `\![*]` の前、字を出すトークンのテキストの先頭のタグより前）に入り、1 つのトークンの前に出すのは 1 つだけである。字を出すトークンは [字の定義](#字の定義) で決まる。

| 内側のトークン | 条件 | 直前に出す区切り | 状態の更新 |
| -------------- | ---- | ---------------- | ---------- |
| 字を出すトークン | `pending_break` が真 | `\n[spot_newlines × 100 の切り捨て]` | `pending_break` を偽、`spot_has_text[last_spot]` を真、`spot_tail` を `"text"` にする |
| 字を出すトークン | `pending_break` が偽で、`spot_tail` が `"choice"` | `\n[after_choice_newlines × 100 の切り捨て]`（`N` が 100 なら `\n`） | `spot_has_text[last_spot]` を真、`spot_tail` を `"text"` にする |
| 字を出すトークン | 上の 2 行以外 | 出さない | 同上 |
| `choice` | `spot_tail` の値が無い | 出さない | `spot_tail` を `"choice"` にする。`spot_has_text`・`pending_break` は変えない |
| `choice` | `spot_tail` が `"text"` | `\n[choice_newlines × 100 の切り捨て]` | 同上 |
| `choice` | `spot_tail` が `"choice"` | `\n[after_choice_newlines × 100 の切り捨て]`（`N` が 100 なら `\n`） | 同上 |
| `clear` | — | 出さない | `\c` を出力したうえで `pending_break` を偽、`spot_has_text[last_spot]` を偽にし、`spot_tail` の値を消す |
| 上記以外 | — | 出さない | 変えない |

幅つきの改行のタグは、`spot_newlines` と `choice_newlines` のどちらも局所関数 `newline_tag` が作る。`after_choice_newlines` の区切りは局所関数 `plain_newline_tag` が作り、`newline_tag` の結果が `\n[100]` なら `\n` を返す。`choice` の前の区切りは局所関数 `choice_break` が決める。

表の区切りは、どれも局所関数 `put_break` が出す。`put_break` は、区切りのタグが空文字列でなければ、それを出して話し手のスポットの行の状態を 1 行進める（局所関数 `next_column`。字数は 0 にする）。区切りを出す所はすべて `put_break` を通す。出力バッファは最後に出した片が単位にならない `\` で終わるかを覚えていて（局所関数 `tracking_buffer`）、pasta が出す片を足す `put` は、片が `\` で始まり前の出力が単独の `\` で終わるとき、`\` を 1 つ足してエスケープの `\\` にしてから片を足す（片の頭の `\` が前の `\` とつながって `\\` に読まれるのを防ぐ。足した後の末尾はエスケープなので、続けて足さない）。区切り、`newline` の `\n` と `clear` の `\c`、`talk_to_script` が台詞の頭に入れた改行（変換の結果が `\` で始まり、頭の 2 バイトがテキストの頭と違うとき）、話し手の切り替えの `\p[…]` と外見の復旧のタグ、表情・ウェイト・`choice_timeout`・`choice` のタグ、終わりの `\e` は、どれも `put` を通る。作者が書いた片、つまり変換の結果の頭がテキストの頭と同じ台詞（作者が台詞の頭に書いたタグ）と `raw_script` は、守らない `put_raw` で足す（局所関数 `head_written_by_author`）。`spot_tail` の添え字は `last_spot`（`nil` の間は 0）である。`last_spot` が `nil` の間は `spot_has_text` を書かない。`choice` は `pending_break` を変えないため、保留中の段落区切りの改行は `choice` の後の字を出すトークンの直前に出る。

`talk`・`sakura_script` のテキストに引数の無い `\c` があるトークンは、テキストを切らずにそのまま出力し、状態だけを次のように扱う（局所関数 `text_parts`）。`\c` の位置は、変換前のテキストを `APPEARANCE.tag_at` で読んで探す（局所関数 `clear_span`）。エスケープと囲みの中はタグとして読まない。

1. 表の「字を出すトークン」に当たるかは、最初の `\c` より前の部分だけで判定する（`talk` は `has_text`、`sakura_script` は `script_shows_text`）。字があれば表のとおりに区切りと状態の更新をし、無ければ表の「上記以外」として出力する。
2. 出力の後、`pending_break` を偽にする。
3. 最後の `\c` より後ろの部分に、1 と同じ判定で字があれば `spot_has_text[last_spot]` を真、`spot_tail` を `"text"` にし、無ければ `spot_has_text[last_spot]` を偽にして `spot_tail` の値を消す。後ろの字の前に区切りは出さない。

消去として数えるのは、`tag_at` が名前 `c` を引数なしで返す単位だけである。引数のある `\c[char,…]`・`\c[line,…]`・`\c[]`、`\x`、`\C` は数えない。`raw_script` のテキストと `choice` の表示の文字は読まない。DSL の台詞の行に書いた `\c` は、テキストが `\c` だけの `sakura_script` になるため、`clear` と同じ状態の更新だけが起きる。

表の「上記以外」のトークン（空の `talk`・字の無い `talk`・字を表示しない `sakura_script` を含む。いずれも `\c` の無いもの）は出力するだけで、判定の状態を変えない。そのため、字の無い `talk` だけを出したスポットは字を出したスポットにならず、保留中の改行はその後の字を出すトークンの直前まで持ち越される（次の切り替えか列の終わりまでに字を出すトークンが無ければ捨てられる）。復旧タグは内側のトークンの経路を通らないため、段落区切りの判定に影響しない。Call の失敗表記は `raw_script`（字を出すトークンではない）のため判定の状態を変えず、保留中の段落区切りの改行は失敗表記の前ではなく、その後の字を出すトークンの直前に出る。

#### 字の定義

字を出すかの判定（局所関数 `emits_text`）は、`talk_to_script` に渡す前のトークンのテキスト（ウェイトや budoux の改行を加える前の、トークンに書かれたテキスト）に対して行う。`text` が `nil` のトークンは字を出さない。文字列でない値は `tostring` した文字列で判定する。テキストを単位（タグ・エスケープ・囲み）に分ける読み方は `APPEARANCE.tag_at` を使う（[アピアランスの観測と復旧](#アピアランスの観測と復旧)）。

`talk` は、テキストを左から読み、次のどれかが 1 つでもあれば字を出す（局所関数 `has_text`）。

- `\` 以外の文字。空白（半角空白・全角空白・タブ）も特別扱いせず、字として数える。タグの名前の直後に続く文字（`\nHello` の `Hello`、`\w9OK` の `OK`）もこれに当たる。
- エスケープ `\\`・`\%`。
- 単位にならない `\`（`\あ` の `\`、テキスト末尾の `\` など）。後ろの文字とともに字として数える。
- 文字を表示するタグ `\_u`・`\_m`・`\&`（名前で判定する。いずれも文字を 1 つ表示するタグである）。
- 中身が空でない囲み（`\_?…\_?`・`\_!…\_!`）。中身がそのまま表示されるためである。

これ以外の単位は取り除いて数えない。文字を表示するタグ以外のタグ（`\s[1000]`・`\![bind,腕,組み,1]`・`\_w[500]`・`\n`・`\n[150]`・`\1` など）、引数の文字列をバルーンに表示するタグ（選択肢の `\q[表示,…]` など）、中身が空の囲み（`\_?\_?`）、閉じの無い `\_?`（ただのタグ）がこれに当たる。テキストが空でなくても、これらの単位だけからなる `talk` は字の無い `talk` である。単語参照の値は `talk` のトークンになるため、タグだけを返す単語の行（`＠通常` の値が `\s[1000]` だけ、など）は字を出さない。ただし同じアクターの `talk` が続けば `merge_consecutive_talks` で 1 つの `talk` に結合され、結合後のテキスト全体で判定する。

`sakura_script` は、テキストの中の `\` の位置から読んだ単位のうち、文字を表示するタグか中身が空でない囲みが 1 つでもあれば字を出す（局所関数 `script_shows_text`）。タグ以外の文字・エスケープ・単位にならない `\` は数えない。`\s[5]` だけの `sakura_script` は字を出さない。

`talk`・`sakura_script` 以外の種類のトークン（`surface`・`wait`・`newline`・`choice`・`choice_timeout`・`raw_script`）は字を出さない。

内側のトークンは、種類ごとの出力の文字列を `STYLE.through(style, アクター, last_spot, 文字列)` に通してから出力し（[装いの出し入れ](#装いの出し入れ)）、出力した文字列を `APPEARANCE.observe(appearance, アクター, last_spot, 文字列)` に渡す。内側の `raw_script` と最上位の `raw_script` は、アクター未指定（`nil`）として観測させる。

列の終わりでは、出していない段落区切りの改行を捨て、`\e` を付けてバッファを 1 本の文字列にする。

### さくらスクリプトの後処理

`talk_to_script(actor, talk, column)` は、`talk` が `nil` か空文字列なら空文字列を返す（`column` は書き換えない）。それ以外は次の順に処理する。

1. **ウェイト値の決定**: `actor` が表なら、その直下の `script_wait_normal`・`script_wait_period`・`script_wait_comma`・`script_wait_strong`・`script_wait_leader` を整数として読み、読めないキーは登録時の既定値（`[talk]` の値、無ければ `TalkConfig` の既定値）を使う。表でなければ既定値だけを使う。
2. **トークナイザ**: 文字列を先頭から走査する。`\` の位置で `SAKURA_TAG_PATTERN` に一致すれば、その一致を 1 つの単位として `TokenKind::SakuraScript` のトークンにし、そうでなければ 1 文字を `CharSets::classify` で分類する。分類の優先順は句点・読点・強調・リーダー・行頭禁則・行末禁則で、どれにも当たらなければ一般の文字である。`SAKURA_TAG_PATTERN` は次の 3 つの選択肢を持ち、左の選択肢を優先する。
   1. 囲み: `\_?` から次の `\_?` まで（最短一致）と、`\_!` から次の `\_!` まで。`(?s)` を付けてあるため、途中に改行文字があっても読みは変わらない。
   2. エスケープ: `\\` と `\%`。
   3. タグ: `\` に続けて、`s`・`p`・`b` と数字 1 桁（0〜9）、`w` と数字 1 桁（1〜9）、または `_` 0〜2 個と名前の 1 文字（`[0-9A-Za-z!+*?&-]`）。最後の形だけが、直後の `[` から始まる引数を取れる。引数は `,` で区切った並びで、引数の中では `\` と続く 1 文字を組として読み、引数の先頭の `"` から次の `"` まで（中の `""` は 1 文字）を引用として読む。エスケープも引用もされていない `]` で閉じ、閉じなければ名前までがタグになる。

   名前の後ろの文字は一致に入らず、通常の文字として分類する（`\nHello` は `\n` と `Hello`、`\n!?` は `\n` と強調の `!?`）。エスケープはタグより先の選択肢のため、`C:\\new` は `\\` と `new` の文字になり、`\n` のタグにはならない。どの選択肢にも一致しない `\`（`\あ` の `\`、末尾の `\`）は 1 文字として分類する。ウェイト挿入（`wait_inserter.rs` の `append_with_per_char_wait`）は、その `\` の直後にウェイトを入れない（入れると `\\` のエスケープになるため）。囲みは、1 回の `talk_to_script` に渡したテキストの中でだけ読まれる。単位はすべて `TokenKind::SakuraScript` であるため、ウェイトは付かず、budoux 改行の幅にも数えない。
3. **ウェイト挿入**（`insert_waits`）: 単位（タグ・エスケープ・囲み）と行末禁則の文字はそのまま出力する。一般の文字とリーダーは 1 文字ごとに `\_w[値 - 50]` を後ろに付ける。句点・読点・強調・行頭禁則は連続する並びとしてためておき、並びが途切れたところで、並びの中の最大値から 50 を引いたウェイトを 1 つだけ付ける（行頭禁則の文字は自分の値を持たず、並びを延ばすだけである）。計算したウェイトが 0 以下なら付けない。
4. **改行の規則の読み取り**（`read_actor_break`）: `actor` が表で、`budoux` が表なら整数の配列として読む。それが空でないときだけ、`budoux_newlines`（数値。`nil` なら無し）と `budoux_jump`（表なら整数の配列、表でなければ空）を読む。`budoux_newlines` から改行の文字列を作る（`newline_for_ratio`。100 倍して切り捨てた `N` が 100 か、値が無ければ `\n`、それ以外は `\n[N]`）。
5. **budoux 改行**: `budoux` が空でない配列か、`column` が表なら、ウェイト挿入後の文字列に `break_lines_impl` を適用する。`column` が表ならその `line`・`width`・`jump` を始めの状態として読み（無いフィールドは 1・0・偽）、無ければ新しい状態（`Column::default()`）から数える。適用が終わった後にだけ、終わりの状態を `column` に書き戻す。どちらでもなければ、ウェイト挿入後の文字列をそのまま返す。

`break_lines_impl(文字列, 改行の規則, 行の状態, タグの正規表現, パーサ)` は、改行の規則 `BreakRule`（`widths`・`jump_widths`・`newline`）に従って改行を入れ、行の状態 `Column`（`line`・`width`・`jump`）をその場で書き換える。

1. タグの正規表現（`talk_to_script`・`break_lines` からはトークナイザの `SAKURA_TAG_PATTERN`。一致した単位は、タグ・エスケープ・囲みのどれでもタグとして扱う）で文字列を「先頭のタグ列」と「本文の文字と、その直後に続くタグ列の組」の並びに分ける。このとき、単位ごとに行の動き（`ColumnMove`）を見分けて記録する。`\n` ちょうどと `\n[` で始まる単位は `Next`（次の行）、`\c` ちょうどの単位は `First`（最初の行）、`\_l[` で始まる単位は `Jump`（飛ぶ）で、ほかの単位は行を動かさない。囲みとエスケープは単位の先頭が違うため、中に `\n` などがあっても動かない。
2. 先頭のタグ列の動きを順に当てる。`Next` は `line` に 1 を足し、`First` は `line` を 1・`jump` を偽にし、`Jump` は `jump_widths` が空でなければ `line` を 1・`jump` を真にする（空なら `line`・`jump` を変えない）。どの動きも `width` を 0 にする。
3. `widths` が空か本文の文字が無ければ、本文の文字の幅を `width` に足しながら各文字の後ろの動きを当てるだけで、入力をそのまま返す（`budoux` の無いアクターでも行の状態は進む）。
4. 本文の文字だけをつないだ文字列を budoux で語に分け、語を、後ろに行の動きがある文字の所で「かけら」に割る。語の頭のかけらは、`width` が 0 でなく、かけらの幅（`UnicodeWidthChar::width_cjk`。全角は 2）を足すと今の行の上限を超えるときに、その前を改行位置にして `Next` を当てる。かけらの幅を `width` に足し、かけらの最後の文字の後ろの動きを順に当てる。タグの後ろのかけらは行の頭なので、幅を足すだけである。1 語だけで上限を超える場合は改行しない。
5. 今の行の上限は、`jump` が真で `jump_widths` が空でなければ `jump_widths`、それ以外は `widths` の `line` 番目の値で、配列の最後の値がそれ以降の行に使われる。
6. 改行位置の直前の文字が、後ろに単位の無い単独の `\` なら、改行位置をその `\` の前へ移し（`\` は新しい行に数え直す）、`\` が行の頭にあるなら改行しない（`\` の直後に改行を入れると `\\` に読まれるため）。
7. 先頭のタグ列を出力し、各文字の前に必要なら `newline` を、各文字の後ろにその文字に続いていたタグ列を出力して組み立て直す。タグは幅に数えず、途中に改行が入ることもなく、元の相対位置に残る。

`break_lines(text, widths)` は同じ `break_lines_impl` を、ウェイト挿入なし・新しい行の状態・`newline` が `\n`・`jump_widths` が空で呼ぶ関数で（`break_fresh`）、組立の経路からは使われない（利用者が `scripts/` から呼ぶための関数である）。

### アピアランスの観測と復旧

外見状態は `actors`・`spots`・`owners`・`last_spots` を持つ表である。SHIORI 用の ACT では `crates/pasta_lua/pasta_scripts/pasta/store.lua` が同じ形の表を `STORE.appearance` として作り（初期化時と reset 時）、ビルドをまたいで保持する（永続化の対象ではない）。`APPEARANCE.new` は `BUILDER.build` に `nil` が渡されたときのビルド内の状態を作る。

| フィールド | 中身 |
| ---------- | ---- |
| `actors[アクター名]` | アクターが自分の発話で出力した外見の記録。`surface`（ID の文字列）、`binds`（カテゴリ → パーツ → `0`・`1`。カテゴリが不明なら `false`）、`order`（カテゴリ → パーツを記録した順の配列） |
| `spots[スポット]` | スポットに表示中と推定される外見。`surface` と、`binds`（カテゴリ → パーツ → `0`・`1`・不明の `false`。パーツ名 `""` はカテゴリ全体の値） |
| `owners[スポット]` | そのスポットで直前に切り替えの出力をしたアクター名 |
| `last_spots[アクター名]` | そのアクターが直前に切り替えの出力をしたスポット |
| `detached` | スコープ切替タグを観測してから次の `restore` までの間だけ真 |

`APPEARANCE.observe(state, actor, spot, text)` は、後処理済みの出力文字列を読むだけで書き換えない。`text` を左から走査して単位を読み、タグを拾う。

単位の読み取りは局所関数 `tag_at` が行い、`APPEARANCE.tag_at` として公開している（ビルダーの字の判定・`\c` の読み取りと適合テストが使う）。`tag_at(s, i)` は位置 `i` の `\` から始まる単位を読み、名前（`\` を除く。数字付きの形は `s3` のように数字を含み、囲みは `_?`・`_!`）、引数（角括弧の中身。無ければ `nil`）、単位の直後の位置、囲みの中身（囲みのときだけ）を返す。エスケープと単位にならない `\` は名前を `nil` で返し、直後の位置はそれぞれ 2 文字後と 1 文字後になる。読み方は Rust の `SAKURA_TAG_PATTERN` と同じ規則の写しで（[さくらスクリプトの後処理](#さくらスクリプトの後処理)）、左から 1 回読むだけで後戻りしない。モジュールが `@pasta_*` を `require` しない方針のため、Rust の正規表現を呼ばずに Lua で同じ規則を持つ。

観測は名前を持たない単位を読み飛ばすため、エスケープはタグとして読まれず、エスケープの後ろの文字もタグの一部にならない（`\\s[5]`・`\%s[5]` はサーフェス変更ではない）。囲みは名前 `_?`・`_!` の 1 つの単位として読み飛ばし、中のタグを観測しない。拾ったタグは、読み取った名前との正確な一致で次の 3 種類に分類し、それ以外は無視する。

- サーフェス変更: `\s[ID]` と `\sN`（N は 1 桁の数字）。空の ID は無視する。
- スコープ切替: `\0`・`\1`・`\h`・`\u`、`\p[N]`、`\pN`。
- 着せ替え: `\![bind,…]` と `\![bind-noevent,…]`。引数は `,` の単純な分割で「カテゴリ・パーツ・値」として読み、`"` か `\` を含む引数とカテゴリ名が空の引数は解釈できないものとする。

`actor` が `nil`（アクター未指定の生さくらスクリプト）か `detached` が真のときは、分類に当たるタグが 1 つでもあれば `spots` を空にする（全スポットを不明にする）だけで、アクターの記録は変えない。それ以外では、サーフェス変更をアクターとスポットの両方の `surface` に出現順で上書きし、着せ替えをアクターとスポットの `binds` に記録し、スコープ切替に当たった時点で `spots` を空にして `detached` を真にし、残りを読まない。DSL ではタグごとに別のトークンになるため、`detached` は同じビルドの後続の観測も止める。着せ替えの値のうちトグル・カテゴリ単位の指定・`0`・`1` 以外の値をどう記録するか（不明とするか）は、[記録できない指定](../grammar/actor-dictionary.md#記録できない指定) の規則をこのモジュールの局所関数（`bind_value`・`record_bind`・`reset_category`・`spot_after_wear`）が実装している。

`APPEARANCE.restore(state, actor, spot, tokens)` は、アクターの切り替えのたびに `\p[spot]` の直後へ出力するタグ列を返す。

1. `detached` を解除する（`\p[spot]` の出力でスコープ切替の影響は終わる）。
2. `owners[spot]` が自分で、かつ `last_spots[自分]` が `spot` なら、同じアクターの継続として空文字列を返す。いずれにせよ `owners[spot]` を自分にする。アクターに名前が無いときも空文字列を返す。
3. `last_spots[自分]` を `spot` にし、グループの内側のトークンから先頭のタグ列（最初の `\` 以外の文字・エスケープ・単位にならない `\`・中身が空でない囲み・スコープ切替タグ・`raw_script` まで）にあるサーフェス変更と着せ替えを集める（`leading_tags`）。`talk` のテキストも走査するのは、`＠単語` の参照が `\s[ID]` に展開されて `talk` に結合されるためである。
4. サーフェスの復旧（`restore_surface`）: アクターの記録の `surface`、無ければ pasta.toml の `surface`（数値か空でない文字列のときだけ）を既知のサーフェスとする。既知のサーフェスがあり、スポットの `surface` と異なり、先頭のタグ列にサーフェス変更が無いときだけ `\s[ID]` を返し、スポットの `surface` を更新する。
5. 着せ替えの復旧（`restore_dressup`）: pasta.toml の `dressup` のカテゴリとアクターの記録のカテゴリの和集合をバイト順に並べ、カテゴリごとに既知の着せ替えを求める。全脱衣を記録していれば全脱衣 → 記録したパーツ（記録した順）、そうでなければ既定のパーツ（パーツ名順・記録で上書きされたものを除く）→ 記録したパーツ（記録した順）である。アクター側でカテゴリが不明なら空になり、そのカテゴリは復旧しない。スポットの実効値と 1 つでも食い違う（または全脱衣したアクターでスポット側に他のパーツが残る）カテゴリだけを、`\![bind-noevent,カテゴリ,パーツ,値]` の並びで丸ごと返し、スポットの `binds` を観測と同じ規則で更新する。先頭のタグ列でパーツと値を明示した bind があれば、そのパーツは出力しない。

サーフェスの復旧タグが先、着せ替えの復旧タグが後に並ぶ。`restore` が更新するのはスポット側と `owners`・`last_spots` だけで、アクターの記録は変えない。

### 装いの出し入れ

アクターの装いは、そのアクターの表の `font` が空でない文字列のときだけ有効である。`font` は pasta.dll の読み込みで、`pasta.toml` に書いた表から検査済みのタグ列（`font = { color = "90,110,140", bold = true }` なら `\f[color,90,110,140]\f[bold,1]`）に置き換えてある（[ローダ自己展開とモジュール解決](loader.md#設定読込)）。`font` の無いアクター、空の表を書いたアクター（タグ列が空文字列になる）、`act:actor_proxy` が作るその場限りのアクターは、装いの無いアクターである。`pasta.shiori.actor_style` は 12 個の属性も値の変換も知らず、出来上がったタグ列をいつ出すかだけを決める。

`BUILDER.build` は呼び出しのたびに `STYLE.new()` で装いの状態を作り、ビルドの終わりで捨てる。`STORE` には置かない。SSP 2.9.18 は次のスクリプトで装いを既定の表示に戻すため、装いはビルドをまたがない。

| フィールド | 中身 |
| ---------- | ---- |
| `used[スポット]` | このビルドで切り替えにそのスポットを使い、`\c` で消していないか |
| `styled[スポット]` | このビルドでそのスポットに装いのタグを出し、`\c` で消していないか |
| `any` | このビルドで装いのあるアクターが切り替えに現れたか |

スポットの状態は `used` と `styled` の組で 3 つに分かれる。Fresh（このビルドで未使用、または `\c` の後）、UsedPlain（使ったが装いを付けていない）、Styled（装いを付けた）である。`STYLE.switch(state, actor, spot)` は、切り替え先のスポットの状態とアクターの装いの有無で、次の表のタグを返す。

| 切り替え先のスポットの状態 | 装いのあるアクターへ | 装いの無いアクターへ |
| -------------------------- | -------------------- | -------------------- |
| Fresh | 装いのタグだけ | 空文字列 |
| UsedPlain | `\f[default]` ＋ 装いのタグ | 空文字列 |
| Styled | `\f[default]` ＋ 装いのタグ | `\f[default]` |

返した後、`used[spot]` を真にし、装いのあるアクターなら `styled[spot]` と `any` も真にする。UsedPlain で装いのあるアクターへ替わるときに `\f[default]` を出すのは、前の話者が台詞に書いた `\f[…]` を切り替え先の装いに持ち越さないためである。切り替えは `last_actor` と別のアクターのときだけ起きるので、同じアクターが話し続ける間は何も出さない。状態をスポットごとに持つのは、SSP 2.9.18 がスコープごとに装いを保ち、別のスコープへ移っても装いが移らないためである。そのため別のスポットへ移るときに戻すタグは出さない。

`STYLE.through(state, actor, spot, s)` は、内側のトークンの種類ごとの出力の文字列（`talk`・`sakura_script` なら後処理済みの文字列）を受け取り、次のように返す。最上位の `raw_script` は通さない。

- `any` が偽か、グループのアクターが `nil` なら `s` をそのまま返す。
- `s` を `APPEARANCE.tag_at` で左から読み、引数の無い `\c` と `\x` を探す。エスケープと囲みの中はタグとして読まない。引数のある `\c[…]`・`\x[noclear]` などは数えない。
- 装いのあるアクターなら、見つけた `\c`・`\x` の直後に装いのタグを差し込む。状態は Styled のままにする。`\x` を見つけたときは、その後の字がスコープ 0 に出ることがあるため、`used[0]`・`styled[0]` も真にする。
- 装いの無いアクターなら、文字列は変えず、`\c` を見つけたときにそのスポットの `used`・`styled` を偽にする（Fresh に戻す）。`\x` では状態を変えない。
- 返す文字列は、`s` の文字を消さず並べ替えず、タグを差し込んだだけのものである。

SSP 2.9.18 は、引数の無い `\c` の後の字を既定の表示で出す（装いが外れる）。装いの無いアクターの `\c` で Fresh に戻すのはこのためである。`\x` も全スコープの吹き出しを消してスコープを `\0` に戻し、その後の字を既定の表示で出す。そのため、装いのあるアクターでは `\x` の直後に必ず出し直し、スポット 0 も装い付きとみなす。`clear` トークンも種類ごとの出力が `\c` なので、同じ規則で扱われる。`clear_spot` トークンでは装いの状態を戻さない（吹き出しは消えないため）。

装いのタグと、字の判定・改行・立ち絵の復旧との順序は次のとおりである。

- 切り替えの出力は `\p[スポット]` → 立ち絵の復旧タグ → 戻すタグと装いのタグの順に並ぶ。段落区切りの改行（`\n[N]`）は、その後の字を出すトークンの直前に出るので、必ず装いのタグより後になる。
- 字の判定（`emits_text`・`text_parts`）と `\c` の読み取りは、`STYLE.through` に通す前の変換前のテキストに対して済んでいる。差し込んだ装いのタグは字に数えず、段落区切りと選択肢の前後の行の区切りの判定を変えない。
- 復旧タグは `APPEARANCE.restore` が先に返し、装いのタグはその後に `STYLE.switch` が返す。復旧の判定（先頭のタグ列）はグループの内側のトークンを読むので、装いのタグの影響を受けない。
- 片を `put` と `put_raw` のどちらで足すか（`head_written_by_author`）は、`STYLE.through` に通す前の文字列で決める。話者の切り替えの装いのタグは `put` を通るので、前の出力の単独の `\` に食われない。
- `STYLE.through` で差し込んだタグは、`APPEARANCE.observe` にもそのまま渡る。`\f` は観測の分類に当たらないため、外見の記録は変わらない。

## 境界の受け渡し

| 境界 | 経路 | 渡すもの | 所有 |
| ---- | ---- | -------- | ---- |
| シーン関数 → ACT | ACT のメソッド | トークンの表 | `act.token` が所有し、`build` で取り出して空にする |
| `pasta.act` → `pasta.shiori.act` | `ACT_IMPL.build` の戻り値 | グループ化トークンの列（0 件なら `nil`） | 呼び出し側が受け取り、`BUILDER.build` に渡す |
| `pasta.shiori.act` → `sakura_builder` | `BUILDER.build` の引数 | グループ化トークン、`{ spot_newlines, choice_newlines, after_choice_newlines }`、`STORE.actor_spots`、`STORE.appearance` | `actor_spots` と `appearance` は `STORE` が所有し、ビルダーがその場で書き換える |
| `sakura_builder` → Rust | `@pasta_sakura_script` の `talk_to_script` | アクターの表（またはそれ以外の値）、テキスト、スポットの行の状態の表 → 後処理済みの文字列 | Rust はアクターの表を呼び出しのたびに読むだけで、保持も変更もしない。行の状態の表は組立が所有し、Rust は変換が成功した後にだけ `line`・`width`・`jump` を書き換える |
| Rust の内部 | `register` が作るクロージャ | `SakuraScriptState`（トークナイザ・既定のウェイト値・budoux のパーサ） | `Arc` でクロージャが所有する。VM を作るたびに `register` が `TalkConfig` から作り直す |
| `sakura_builder` → `appearance` | `APPEARANCE.observe`・`APPEARANCE.restore` | 外見状態の表、アクター、スポット、出力した文字列またはグループの内側のトークン | 状態の表は呼び出し側の所有で、`appearance` はその場で書き換える。文字列とトークンは読むだけ |
| ローダ → `sakura_builder` | アクターの表の `font`（`@pasta_config` の `actor."名前".font`） | 検査済みのタグ列（文字列） | 読み込みで作られ、ビルダーと `actor_style` は読むだけ |
| `sakura_builder` → `actor_style` | `STYLE.new`・`STYLE.switch`・`STYLE.through` | 装いの状態の表、アクター、スポット、変換後の文字列 → 戻すタグと装いのタグ、または出し直しを差し込んだ文字列 | 状態の表は `BUILDER.build` が呼び出しごとに作って所有し、`actor_style` はその場で書き換える。文字列は読むだけで、新しい文字列を返す |
| `actor_style` → `appearance` | `APPEARANCE.tag_at` | 変換後の文字列と位置 → 単位の名前・引数・直後の位置 | 状態を持たない読み取りで、文字列は読むだけ |
| `sakura_builder` → `appearance` | `APPEARANCE.tag_at` | 変換前のトークンのテキストと位置 → 単位の名前・引数・直後の位置・囲みの中身 | 状態を持たない読み取りで、テキストは読むだけ |
| `pasta.shiori.act` → 呼び出し側 | `build` の戻り値 | さくらスクリプト文字列 | `act:yield()` などが `coroutine.yield` で渡す（[ランタイム実行モデル](execution-model.md#イベントからシーンへ)） |

## 不変条件と制約

- 後処理（ウェイト挿入と budoux 改行）は、`talk`・`sakura_script` のトークンごとに組立の途中で適用される。組み立て終わった文字列に後から適用する処理は無い。
- 後処理は入力の文字とタグを並べ替えず、`\_w[…]` と改行（`\n` または `\n[N]`）を挿入するだけである。タグの文字列はそのまま残る。
- 後処理の設定は 2 か所から来る。文字の集合とウェイトの既定値は `register` の時点の `TalkConfig` で固定され、トークナイザの正規表現も登録時に 1 度だけコンパイルされる。アクター別のウェイト値と改行の規則（`budoux`・`budoux_newlines`・`budoux_jump`）は `talk_to_script` の呼び出しのたびにアクターの表から読む。
- budoux の行の状態は、組立が 1 回のビルドの中でスポットごとに持ち（`columns`）、`talk_to_script` の 3 つ目の引数で渡す。1 回の呼び出しの中では、テキストの中の行を動かす単位（`\n`・`\n[…]`・引数の無い `\c`・`\_l[…]`）で数え直す。作者が書いたタグも、単語や関数が返して `talk` に結合されたタグも、`sakura_script` のテキストの中のタグも同じに読む。呼び出しの間では、組立が出す区切り（`put_break`）が 1 行、`newline` が `n` 行進め、`clear` が最初の行に戻し、`clear_spot` が全スポットの状態を捨てる。表情・ウェイト・`choice_timeout`・`raw_script`・`choice` は状態を変えない（`raw_script` のテキストの中の `\c` と、選択肢の表示の文字は数えない）。そのため、`sakura_script` や表情で区切られた `talk` も、同じスポットでは前の `talk` の終わりの行と字数を引き継ぐ。状態はビルドをまたがない。
- `budoux` の無いアクターの `talk`・`sakura_script` も、行の状態を進める（改行は入れない）。同じスポットで相手役が `\n`・`\c` を書いたときに、行の番号がずれないためである。
- 組立の 2 つの動き（`next_column`・`first_column`）は、Rust の `ColumnMove::Next`・`ColumnMove::First` の写しである。片方だけを直さない。同じ動きになることは `sakura_builder_test.lua` で確かめる。
- 代わりのレンダラ（[presentation マーカーとレンダラ注入](shiori.md#presentation-マーカーとレンダラ注入)）が 3 つ目の引数を使わなければ、台詞の中の字数は数えられず、行の状態は組立の区切りの分だけ進む。出力は壊れない。
- SSP が自分で折り返した分（1 語で上限を超えた行、`budoux` の無いアクターの台詞）は行の状態に入らないため、実際の行とずれうる。
- 単位（タグ・エスケープ・囲み）はどれも budoux 改行の幅に数えない。エスケープと中身のある囲みはバルーンに字を表示するが、ほかのタグと同じく幅 0 として扱われる。そのため囲みが長いと、その行は幅の上限を超えることがある。
- `budoux` の配列に整数にできない要素があると `talk_to_script` は Lua のエラーになる。`budoux` が空でない配列のときは、数値にできない `budoux_newlines` と、整数にできない要素のある `budoux_jump` も同じくエラーになる。ウェイトのキーは読めなければ既定値になる。
- 出力の先頭と `clear_spot` の後で、発言より前に積んだ `surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout` は、アクター未指定として、積んだ位置に積んだ順で出力される。スコープ切替タグを伴わないため、その時点のスコープに効く。`last_spot` が `nil` の間に出した `choice` は、スポット 0 のバルーンに出したものとして `spot_tail` に記録する。内側の出力にサーフェス変更などの分類に当たるタグがあれば全スポットが不明になり（[アピアランスの観測と復旧](#アピアランスの観測と復旧)）、次の発言者の切り替えで復旧タグが出ることがある。
- `clear_spot` は現在のグループを閉じる。そのため `clear_spot` の後の発言は、前のグループと同じアクターでも新しいグループに入り、`clear_spot` で組立の状態をリセットした後の切り替えの出力（`\p[N]`）の後に出力される。
- `spot` はグループを閉じない。同じアクターの発言が `spot` の前後にあると、後の発言も前のグループに入り、`spot` の処理はそのグループの出力の後になる。それでも出力は変わらない。`spot` をまたいで同じグループに入るのは切り替えを伴わない同じ発言者の発言だけであり、`spot` が記録した立ち位置は次に発言者が切り替わるときに初めて使われるためである。
- 組立の状態のうち、`actor_spots` と外見状態はビルドをまたいで `STORE` に残り、`last_actor`・`spot_has_text`・`pending_break`・`spot_tail`・`columns` はビルドごとに作り直す。そのため段落区切りの改行と選択肢の前後の行の区切りと budoux の行の状態は 1 回の出力の中だけで判定され、外見の復旧はトークをまたいで判定される。
- `clear_spot` は `actor_spots` を表ごと差し替えず、エントリを 1 つずつ消す。`STORE.actor_spots` と同じ表を指し続けるためである。外見状態は `clear_spot` では消さない。
- アピアランスは pasta 自身が出力した文字列だけを観測する。ベースウェア側だけで起きた外見の変化は状態に入らない。
- さくらスクリプトの単位の読み方の定義は 4 か所にある。Rust の `Tokenizer::SAKURA_TAG_PATTERN`（`crates/pasta_lua/src/sakura_script/tokenizer.rs`）が正で、Lua の `tag_at`（`crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua`）、DSL の文法の `sakura_script`・`sakura_escape` の規則（`crates/pasta_dsl/src/parser/grammar.pest`）、VS Code 拡張の文法定義の `inline-sakura-script`・`inline-escape`（`editors/vscode/syntaxes/pasta.tmLanguage.json`）がその写しである。正の正規表現は、後戻りの有無で一致が変わらない形にしてあるため、左から 1 回読むだけの写し（Lua・pest）と結果が一致する。
- Rust・Lua・DSL の 3 つの読み取りは、適合テスト `crates/pasta_lua/tests/sakura_script/conformance_test.rs` の 1 つの事例の表（入力と期待する単位の並び）で、同じ位置で単位を切り出すことを確かめる。DSL だけが異なるのは、単位にならない `\` をパースエラーにすることである。また DSL は引数・引用・囲みを 1 行の中だけで読む。VS Code 拡張の文法定義は色分けだけに使われ、適合テストの対象ではない（`editors/vscode/src/test/tmGrammar.test.ts` で確かめる）。読み方を変えるときは 4 か所を同じ変更でそろえる。
- 囲みはトークンをまたがない。後処理は 1 回の `talk_to_script` に渡したテキスト（結合後の `talk`、または 1 つの `sakura_script`）の中でだけ囲みを読み、字の判定と外見の観測もトークン（観測は出力した文字列）ごとに読むため、囲みの読み取りはトークンをまたぐ状態を持たない（トークンをまたいで持ち越すのは budoux の行の状態だけである）。開きと閉じを別々のトークンで積むと、それぞれが閉じの無い `\_?`（`\_!`）として、ただのタグに読まれる。
- 字の判定と `\c` の読み取りは変換前のテキストに対して行い、外見の観測は後処理済みの出力文字列に対して行う。後処理が挿入する `\_w[…]`・改行は字の判定に入らない。
- 装いの状態はビルドごとに作り直し、ビルドをまたがない。装いのあるアクターが切り替えに現れないビルドでは、`STYLE.switch` は空文字列を、`STYLE.through` は入力をそのまま返すため、出力は装いの仕組みが無いときと 1 文字も変わらない。
- 装いの出し入れが追うのは、組立が出す `\p[N]` と、グループの内側の引数の無い `\c`・`\x` だけである。台詞の中に書いたスコープの切り替え（`\p[1]`・`\1` など）と、最上位の `raw_script` の中の `\f`・`\c` は追わない。最上位の `raw_script` の直後には必ず話者の切り替えが起きるため、装いのあるアクターの表示は切り替えで決まる。

## ソースの所在

- `crates/pasta_lua/src/sakura_script/`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/actor_style.lua`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`

本文で参照したその他のファイルは次のとおりである。トークンの蓄積とグループ化は `crates/pasta_lua/pasta_scripts/pasta/act.lua`、アクタープロキシは `crates/pasta_lua/pasta_scripts/pasta/actor.lua`、`STORE.actor_spots`・`STORE.appearance` は `crates/pasta_lua/pasta_scripts/pasta/store.lua`、文字列バッファは `crates/pasta_lua/pasta_scripts/pasta/buf.lua`、`TalkConfig` は `crates/pasta_lua/src/loader/config/sections.rs` にある。振る舞いのテストは `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua`（組立と字の判定）と `crates/pasta_lua/tests/lua_specs/appearance_test.lua`（外見の観測と復旧）、`crates/pasta_lua/tests/lua_specs/actor_style_test.lua`（装いの状態表と `\c`・`\x` の後の出し直し）にある。単位の読み方の写しは `crates/pasta_dsl/src/parser/grammar.pest` と `editors/vscode/syntaxes/pasta.tmLanguage.json` にあり、3 つの読み取りの適合テストは `crates/pasta_lua/tests/sakura_script/conformance_test.rs` にある。

## 経緯

- [alpha03-shiori-act-sakura](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/alpha03-shiori-act-sakura) — `pasta.shiori.act` によるさくらスクリプトの組立
- [actor-talk-grouping](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/actor-talk-grouping) — アクターの切り替わりでのグループ化と連続する発言の結合
- [actor-spot-refactoring](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/actor-spot-refactoring)・[persist-spot-position](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/persist-spot-position) — `spot`・`clear_spot` トークンとスポットの保持
- [sakura-script-newline](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/sakura-script-newline) — 段落区切りの改行の遅延判定
- [refine-talk-conversion](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/refine-talk-conversion)・[act-sakura-script-method](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/act-sakura-script-method) — `talk`・`sakura_script` を `talk_to_script` で変換する経路
- [sakura-script-wait](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/sakura-script-wait) — トークナイザとウェイト挿入
- [sakura-script-dash-tag-fix](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/sakura-script-dash-tag-fix) — タグ名の文字集合
- [budoux-line-breaker](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/budoux-line-breaker) — budoux 改行
- [actor-surface-restore](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/actor-surface-restore) — 外見の観測と復旧
- [sakura-builder-string-buffer](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/sakura-builder-string-buffer) — 組立の文字列バッファ
- [paragraph-break-tag-only-talk](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/paragraph-break-tag-only-talk) — 字を出すトークンによる段落区切りの判定と、単位（タグ・エスケープ・囲み）の読み方を SSP にそろえる変更、3 つの読み取りの適合テスト
- [choice-line-layout](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/choice-line-layout) — 選択肢を 1 行に 1 つずつ出す改行の配置と `choice_newlines`、台詞に書いた `\c` を吹き出しの消去に数える変更
- [vertical-balloon-lines](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/vertical-balloon-lines) — 列を動かすタグでの budoux の数え直しと、1 回の出力の中の吹き出しごとの列の持ち越し（`put_break`）、`budoux_newlines`・`budoux_jump`・`after_choice_newlines`、単独の `\` の守り
- [actor-display-style](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/actor-display-style) — アクターごとの装い（`font`）の出し入れと、1 つのスポットを複数のアクターで共有する出力

---

> 【照れ】台詞ひとつにも、これだけの仕立てが施されておりますのよ。……少しは、見直しまして？

> 【アンソニー】はい、針子のお二人のことは、すっかり見直しました。

> 【落胆】……わたくしは、数に入っておりませんの？
