# トーク出力とアピアランス

ごきげんよう。ゴーストの台詞が、ウェイトや改行の効いたさくらスクリプトに仕立て上がるまで――その仕立ての工程をご覧に入れますわ。
組立、後処理、そして表情と着せ替え。舞台衣装の裏側まで、わたくしがお見せいたします。さあ、参りましょう。

---

この章では、ACT に積まれたトークからさくらスクリプトを組み立てる出力処理と、サーフェス・着せ替えの観測と復旧（アピアランス）を扱う。ウェイトの長さ・自動改行の幅・外見の復旧規則など、利用者から見た振る舞いは [@pasta_sakura_script](../lua/modules/pasta-sakura-script.md)・[pasta.toml リファレンス](../reference/pasta-toml.md)・[アクター辞書](../grammar/actor-dictionary.md#同一スポット共有時の外見の復旧) が正である。

## 目的と責務

トーク出力は、シーンの実行中に ACT が積んだトークンの列を、ベースウェアへ返す 1 本のさくらスクリプト文字列にする。

この章が責務を持つのは次の事項である。

- ACT の `build` が行う前処理: トークンをアクターの切り替わりで束ねるグループ化と、連続する発言の結合
- `pasta.shiori.sakura_builder` による組立: トークンの種類ごとの出力、スポットの解決と `\p[N]` の出力、段落区切りの改行、終端の `\e`
- `@pasta_sakura_script` の後処理（Rust）: さくらスクリプトの単位（タグ・エスケープ・囲み）と文字を分けるトークナイザ、ウェイトの挿入、budoux による改行
- アピアランス: 出力した文字列からサーフェス変更・着せ替え・スコープ切替を観測して記録し、アクターの切り替え時に外見を復旧するタグを出力する仕組み

次の事項は他の章が権威を持ち、この章では再記述しない。

- `build` がいつ呼ばれ、その戻り値がどう応答になるか（`act:yield()`・シーン終了時の `build`・再開のループ）は [ランタイム実行モデル](execution-model.md#イベントからシーンへ) で扱う。
- `@pasta_sakura_script` を VM に登録する継ぎ目（`RendererInjection`）と presentation マーカーは [SHIORI 層](shiori.md#presentation-マーカーとレンダラ注入) で扱う。
- ACT のメソッドとフィールドの個々の仕様は [Lua ランタイム内部モジュール](internal-modules.md#act-の内部) で扱う。シーン関数がどの ACT のメソッドを呼ぶかは [トランスパイラ](transpiler.md#生成される-lua-コードの形) で扱う。
- 文字の分類・ウェイトの値・budoux の幅の指定方法は [@pasta_sakura_script](../lua/modules/pasta-sakura-script.md) と [pasta.toml リファレンス](../reference/pasta-toml.md#talkトーク表示制御) が正である。段落区切りの改行量は [spot_newlines](../reference/pasta-toml.md#spot_newlines)、スポットの決まり方は [バルーン連携](../grammar/actor-dictionary.md#バルーン連携)、外見の復旧の規則と記録できない指定は [同一スポット共有時の外見の復旧](../grammar/actor-dictionary.md#同一スポット共有時の外見の復旧) が正である。

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
pasta.shiori.sakura_builder  BUILDER.build(グループ化トークン, { spot_newlines }, STORE.actor_spots, STORE.appearance)
    │  アクターの切り替え: \p[spot] → APPEARANCE.restore（復旧タグ）
    │  talk / sakura_script: @pasta_sakura_script.talk_to_script(アクター, テキスト)
    │                          └ Rust: トークナイザ → ウェイト挿入 → budoux 改行
    │  その他のトークン: 種類ごとのタグ
    │  出力した各片を APPEARANCE.observe に観測させる
    ▼
さくらスクリプト文字列（末尾は \e）
```

### Lua 側

| モジュール（ファイル） | 役割 |
| ---------------------- | ---- |
| `pasta.act`（`crates/pasta_lua/pasta_scripts/pasta/act.lua`） | トークンの蓄積（`talk`・`sakura_script`・`raw_script`・`surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout`・`set_spot`・`clear_spot`）と、`build` の前処理（局所関数 `group_by_actor`・`merge_consecutive_talks`） |
| `pasta.actor`（`crates/pasta_lua/pasta_scripts/pasta/actor.lua`） | アクタープロキシ。`act:actor_proxy("アクター")` などで得たプロキシの `:talk(…)`・`:sakura_script(…)` を、アクターを添えた ACT の `talk`・`sakura_script` に委ねる |
| `pasta.shiori.act`（`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`） | `pasta.act` を継承し、`build` を「`ACT_IMPL.build` でグループ化トークンを得て `BUILDER.build` に渡す」処理に差し替える。`spot_newlines` は `[ghost]` から読む |
| `pasta.shiori.sakura_builder`（`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`） | グループ化トークンをさくらスクリプト文字列にする `BUILDER.build` |
| `pasta.shiori.appearance`（`crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua`） | 外見状態の生成（`APPEARANCE.new`）・観測（`APPEARANCE.observe`）・復旧（`APPEARANCE.restore`）と、さくらスクリプトの単位の読み取り（`APPEARANCE.tag_at`。ビルダーが字の判定に使う）。状態を持たず、渡された状態の表をその場で更新する。`STORE` や `@pasta_*` を `require` しない |
| `pasta.buf`（`crates/pasta_lua/pasta_scripts/pasta/buf.lua`） | 組立に使う文字列バッファ。LuaJIT の `string.buffer` があればそれを、無ければ同じ `put`・`tostring` を持つ最小実装を使う |
| `pasta.store`（`crates/pasta_lua/pasta_scripts/pasta/store.lua`） | ビルドをまたいで保持する `STORE.actor_spots`（アクター名 → スポット）と `STORE.appearance`（外見状態）を持つ |

### Rust 側（`crates/pasta_lua/src/sakura_script/`）

| ファイル | 役割 |
| -------- | ---- |
| `crates/pasta_lua/src/sakura_script/mod.rs` | `@pasta_sakura_script` のモジュール表を作る `register`。`talk_to_script` と `break_lines` の実装、アクター表からのウェイト値と budoux の幅の読み取り |
| `crates/pasta_lua/src/sakura_script/tokenizer.rs` | `Tokenizer`（さくらスクリプトの単位（タグ・エスケープ・囲み）の正規表現 `SAKURA_TAG_PATTERN` と文字の集合 `CharSets`）と、文字の種類 `TokenKind` |
| `crates/pasta_lua/src/sakura_script/wait_inserter.rs` | ウェイト値 `WaitValues` と、トークン列に `\_w[ms]` を挿入する `insert_waits` |
| `crates/pasta_lua/src/sakura_script/line_breaker.rs` | budoux の分かち書きで `\n` を挿入する `break_lines_impl` |

文字の集合とウェイトの既定値は `TalkConfig`（`crates/pasta_lua/src/loader/config/sections.rs`）が持つ。`[talk]` セクションを `TalkConfig` にする仕組みは [ローダ自己展開とモジュール解決](loader.md#設定読込) で扱う。budoux の分かち書きは依存クレート `budouy` の既定の日本語モデルを使う。

## 処理とデータの流れ

### トークンの蓄積

ACT のメソッドは、さくらスクリプトに依存しない表を `act.token` に積むだけで、文字列は作らない。トークンの形は次のとおりである。

| `type` | フィールド | 積むメソッド |
| ------ | ---------- | ------------ |
| `talk` | `actor`・`text` | `act:talk(アクター, テキスト)`（アクタープロキシの `talk` 経由を含む）。`text` が `nil` なら積まない。未登録のアクターの目印（`【未登録アクター：名前】`）は `act:actor_proxy` が積む（[生成コード用のメソッド](internal-modules.md#生成コード用のメソッドactor_proxyglobal_fnarithconcat)） |
| `sakura_script` | `actor`・`text` | `act:sakura_script(アクター, テキスト)`（アクタープロキシ経由） |
| `raw_script` | `text` | `act:raw_script(テキスト)`。SHIORI 用の ACT では `set_property`・`get_property` も積む |
| `surface` | `id` | `act:surface(ID)` |
| `wait` | `ms` | `act:wait(ミリ秒)`（0 以上の整数に丸める） |
| `newline` | `n` | `act:newline(回数)`（既定 1） |
| `clear` | — | `act:clear()` |
| `choice` | `target`・`display` | `act:choice(ジャンプ先, 表示)` |
| `choice_timeout` | `seconds` | `act:choice_timeout(秒)` |
| `spot` | `actor`・`spot` | `act:set_spot(名前, 番号)`。`act.actors` に無い名前なら積まない |
| `clear_spot` | — | `act:clear_spot()` |

`talk` と `sakura_script` だけが発言したアクターを持つ。`surface` などアクターを持たないトークンは、次のグループ化で直前の発言者のグループに入る（出力の先頭か `clear_spot` の後で、発言より前に積んだものはアクター `nil` のグループに入る）。

### グループ化トークン

`ACT_IMPL.build` は `act.token` を取り出して空にし、0 件なら `nil` を返す。1 件以上なら次の 2 段で列を作り直す。

1. `group_by_actor` は列を先頭から走査する。
   - `spot` は、そのまま結果に置く。グループは閉じない。
   - `clear_spot` は、そのまま結果に置き、現在のグループを閉じる。後の発言と表示制御などは新しいグループに入る。
   - `talk`・`sakura_script` は、グループがまだ無いか、トークンの `actor` が現在のグループのアクターと別の表であれば、新しいグループ `{ type = "actor", actor = …, tokens = {} }` を結果に追加してから、そのグループの `tokens` に入れる。同じアクターなら現在のグループに入れる。
   - `raw_script` は、グループがあればその `tokens` に、無ければ結果に直接置く。
   - それ以外（`surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout`）は、グループがあればその `tokens` に入れ、無ければアクター `nil` のグループ `{ type = "actor", actor = nil, tokens = {} }` を結果に追加してから、そのグループの `tokens` に入れる。グループが無いのは、出力の先頭か `clear_spot` の後で、まだ発言を積んでいないときである。
2. `merge_consecutive_talks` は、各グループの `tokens` の中で隣り合う `talk` のテキストを連結して 1 つの `talk` にする。`sakura_script` など `talk` 以外のトークンが間にあれば結合は切れる。

結果は、`type = "actor"` のグループと、`spot`・`clear_spot`・グループの外の `raw_script` が並ぶ列になる。アクター `nil` のグループは、発言より前に積んだ表示制御などを積んだ順に持つ。組立はこのグループでスコープ切替タグを出さず、内側の出力はアクター未指定（`nil`）として観測される（[さくらスクリプトの組立](#さくらスクリプトの組立)）。

### トークンの種類と出力

`BUILDER.build` は、グループ化トークンの列の最上位とグループの内側で、種類ごとに次の出力をする。

| 位置 | `type` | 出力 |
| ---- | ------ | ---- |
| 最上位 | `spot` | 出力しない。`actor_spots[アクター名] = spot` を記録する |
| 最上位 | `clear_spot` | 出力しない。`actor_spots` の全エントリを消し、組立の状態（直前の発言者・スポット・段落区切りの判定）をリセットする |
| 最上位 | `actor` | アクターが直前の発言者と別なら切り替えの出力（次節）。続けて内側のトークンを順に出力する |
| 最上位 | `raw_script` | `text` をそのまま |
| 内側 | `talk`・`sakura_script` | `SAKURA_SCRIPT.talk_to_script(グループのアクター, text)` の戻り値（後処理済みの文字列） |
| 内側 | `surface` | `\s[id]` |
| 内側 | `wait` | `\_w[ms]` |
| 内側 | `newline` | `\n` を `n` 回 |
| 内側 | `clear` | `\c` |
| 内側 | `raw_script` | `text` をそのまま |
| 内側 | `choice` | `\![*]\q[display,target]`（`display`・`target` の `\`・`]`・`,` を `\` でエスケープする） |
| 内側 | `choice_timeout` | `\![set,choicetimeout,ミリ秒]`（`seconds` の 1000 倍を切り捨て。`seconds` が無ければ 0） |
| 内側 | 上記以外 | 空文字列 |

`talk_to_script` を通るのは `talk` と `sakura_script` だけである。`surface`・`wait` などのタグ、`raw_script`、`choice` の出力にはウェイトも改行も入らない。

### さくらスクリプトの組立

`BUILDER.build(グループ化トークン, config, actor_spots, appearance)` は、`config.spot_newlines`（既定 1.5）と、`config.buffer_factory`（既定 `pasta.buf` の `new`。テストが差し替える）を使う。SHIORI 用の ACT は `actor_spots` に `STORE.actor_spots` を、`appearance` に `STORE.appearance` を渡し、`BUILDER.build` はどちらもその場で書き換える。`nil` を渡された場合は、そのビルドの中だけの空の表を使う。

組立は、1 回のビルドの中だけで生きる次の状態を持つ。

| 状態 | 意味 |
| ---- | ---- |
| `last_actor` | 直前に切り替えの出力をしたアクター（`nil` から始まる） |
| `last_spot` | 現在のスポット（現在のスコープ） |
| `spot_has_text` | スポットごとに、このビルドで字を出すトークン（字のある `talk`、字を表示する `sakura_script`）を出力したか |
| `pending_break` | 現在のスポットで、次の字を出すトークンの前に段落区切りの改行を出すか |

グループのアクターが `last_actor` と別の表であれば、アクターの切り替えとして次を行う（`emit_actor_switch`）。

1. スポットを `actor_spots[アクター名]` から引く。無ければ 0 とし、アクターに名前があれば警告をログに出す。`act:actor_proxy` が作るその場限りのアクター（未登録の名前）は `actor_spots` に無いため、ここで 0 になる。
2. `\p[スポット]` を出力する。
3. 続けて `APPEARANCE.restore(appearance, アクター, スポット, グループの内側のトークン)` が返す復旧タグを出力する（[アピアランスの観測と復旧](#アピアランスの観測と復旧)）。
4. `pending_break` を `spot_has_text[スポット]` で決め直し、`last_spot`・`last_actor` を更新する。

`last_actor` は各ビルドの初めに `nil` なので、アクターを持つ最初のグループでは必ず `\p[N]` が出る。グループのアクターが `nil` のときは切り替えの出力をせず、現在のスコープのまま内側を出力する。

内側のトークンのうち、字を出すトークン（[字の定義](#字の定義)）の前では、`pending_break` が真なら `\n[spot_newlines × 100 の切り捨て]` を出力して `pending_break` を偽にし、`spot_has_text[last_spot]` を真にしてから本文を出力する。改行は字を出すトークンの出力の先頭（テキストの先頭のタグより前）に入る。`clear` は `\c` を出力したうえで `pending_break` を偽にし、`spot_has_text[last_spot]` を偽に戻す。それ以外のトークン（空の `talk`・字の無い `talk`・字を表示しない `sakura_script` を含む）は出力するだけで、判定の状態を変えない。そのため、字の無い `talk` だけを出したスポットは字を出したスポットにならず、保留中の改行はその後の字を出すトークンの直前まで持ち越される（次の切り替えか列の終わりまでに字を出すトークンが無ければ捨てられる）。復旧タグは内側のトークンの経路を通らないため、段落区切りの判定に影響しない。

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

内側のトークンを出力するたびに、出力した文字列を `APPEARANCE.observe(appearance, アクター, last_spot, 文字列)` に渡す。内側の `raw_script` と最上位の `raw_script` は、アクター未指定（`nil`）として観測させる。

列の終わりでは、出していない段落区切りの改行を捨て、`\e` を付けてバッファを 1 本の文字列にする。

### さくらスクリプトの後処理

`talk_to_script(actor, talk)` は、`talk` が `nil` か空文字列なら空文字列を返す。それ以外は次の順に処理する。

1. **ウェイト値の決定**: `actor` が表なら、その直下の `script_wait_normal`・`script_wait_period`・`script_wait_comma`・`script_wait_strong`・`script_wait_leader` を整数として読み、読めないキーは登録時の既定値（`[talk]` の値、無ければ `TalkConfig` の既定値）を使う。表でなければ既定値だけを使う。
2. **トークナイザ**: 文字列を先頭から走査する。`\` の位置で `SAKURA_TAG_PATTERN` に一致すれば、その一致を 1 つの単位として `TokenKind::SakuraScript` のトークンにし、そうでなければ 1 文字を `CharSets::classify` で分類する。分類の優先順は句点・読点・強調・リーダー・行頭禁則・行末禁則で、どれにも当たらなければ一般の文字である。`SAKURA_TAG_PATTERN` は次の 3 つの選択肢を持ち、左の選択肢を優先する。
   1. 囲み: `\_?` から次の `\_?` まで（最短一致）と、`\_!` から次の `\_!` まで。`(?s)` を付けてあるため、途中に改行文字があっても読みは変わらない。
   2. エスケープ: `\\` と `\%`。
   3. タグ: `\` に続けて、`s`・`p`・`b` と数字 1 桁（0〜9）、`w` と数字 1 桁（1〜9）、または `_` 0〜2 個と名前の 1 文字（`[0-9A-Za-z!+*?&-]`）。最後の形だけが、直後の `[` から始まる引数を取れる。引数は `,` で区切った並びで、引数の中では `\` と続く 1 文字を組として読み、引数の先頭の `"` から次の `"` まで（中の `""` は 1 文字）を引用として読む。エスケープも引用もされていない `]` で閉じ、閉じなければ名前までがタグになる。

   名前の後ろの文字は一致に入らず、通常の文字として分類する（`\nHello` は `\n` と `Hello`、`\n!?` は `\n` と強調の `!?`）。エスケープはタグより先の選択肢のため、`C:\\new` は `\\` と `new` の文字になり、`\n` のタグにはならない。どの選択肢にも一致しない `\`（`\あ` の `\`、末尾の `\`）は 1 文字として分類する。囲みは、1 回の `talk_to_script` に渡したテキストの中でだけ読まれる。単位はすべて `TokenKind::SakuraScript` であるため、ウェイトは付かず、budoux 改行の幅にも数えない。
3. **ウェイト挿入**（`insert_waits`）: 単位（タグ・エスケープ・囲み）と行末禁則の文字はそのまま出力する。一般の文字とリーダーは 1 文字ごとに `\_w[値 - 50]` を後ろに付ける。句点・読点・強調・行頭禁則は連続する並びとしてためておき、並びが途切れたところで、並びの中の最大値から 50 を引いたウェイトを 1 つだけ付ける（行頭禁則の文字は自分の値を持たず、並びを延ばすだけである）。計算したウェイトが 0 以下なら付けない。
4. **budoux 改行**: `actor` が表で、`budoux` が空でない配列なら、ウェイト挿入後の文字列に `break_lines_impl` を適用する。

`break_lines_impl(文字列, 幅の配列, タグの正規表現, パーサ)` は次のように改行を入れる。

1. タグの正規表現（`talk_to_script`・`break_lines` からはトークナイザの `SAKURA_TAG_PATTERN`。一致した単位は、タグ・エスケープ・囲みのどれでもタグとして扱う）で文字列を「先頭のタグ列」と「本文の文字と、その直後に続くタグ列の組」の並びに分ける。本文の文字が無ければ入力をそのまま返す。
2. 本文の文字だけをつないだ文字列を budoux で語に分ける。
3. 語を順に置き、行の幅（`UnicodeWidthStr::width_cjk`。全角は 2）を足していく。行に語がすでにあり、次の語を足すと現在の行の幅の上限を超えるときに、その語の前を改行位置にする。上限は 1 行目が配列の 1 番目、2 行目が 2 番目で、配列の最後の値がそれ以降の行に使われる。1 語だけで上限を超える場合は改行しない。
4. 先頭のタグ列を出力し、各文字の前に必要なら `\n` を、各文字の後ろにその文字に続いていたタグ列を出力して組み立て直す。タグは幅に数えず、途中に `\n` が入ることもなく、元の相対位置に残る。

`break_lines(text, widths)` は同じ `break_lines_impl` をウェイト挿入なしで呼ぶ関数で、組立の経路からは使われない（利用者が `scripts/` から呼ぶための関数である）。

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

単位の読み取りは局所関数 `tag_at` が行い、`APPEARANCE.tag_at` として公開している（ビルダーの字の判定と適合テストが使う）。`tag_at(s, i)` は位置 `i` の `\` から始まる単位を読み、名前（`\` を除く。数字付きの形は `s3` のように数字を含み、囲みは `_?`・`_!`）、引数（角括弧の中身。無ければ `nil`）、単位の直後の位置、囲みの中身（囲みのときだけ）を返す。エスケープと単位にならない `\` は名前を `nil` で返し、直後の位置はそれぞれ 2 文字後と 1 文字後になる。読み方は Rust の `SAKURA_TAG_PATTERN` と同じ規則の写しで（[さくらスクリプトの後処理](#さくらスクリプトの後処理)）、左から 1 回読むだけで後戻りしない。モジュールが `@pasta_*` を `require` しない方針のため、Rust の正規表現を呼ばずに Lua で同じ規則を持つ。

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

## 境界の受け渡し

| 境界 | 経路 | 渡すもの | 所有 |
| ---- | ---- | -------- | ---- |
| シーン関数 → ACT | ACT のメソッド | トークンの表 | `act.token` が所有し、`build` で取り出して空にする |
| `pasta.act` → `pasta.shiori.act` | `ACT_IMPL.build` の戻り値 | グループ化トークンの列（0 件なら `nil`） | 呼び出し側が受け取り、`BUILDER.build` に渡す |
| `pasta.shiori.act` → `sakura_builder` | `BUILDER.build` の引数 | グループ化トークン、`{ spot_newlines }`、`STORE.actor_spots`、`STORE.appearance` | `actor_spots` と `appearance` は `STORE` が所有し、ビルダーがその場で書き換える |
| `sakura_builder` → Rust | `@pasta_sakura_script` の `talk_to_script` | アクターの表（またはそれ以外の値）とテキスト → 後処理済みの文字列 | Rust はアクターの表を呼び出しのたびに読むだけで、保持も変更もしない |
| Rust の内部 | `register` が作るクロージャ | `SakuraScriptState`（トークナイザ・既定のウェイト値・budoux のパーサ） | `Arc` でクロージャが所有する。VM を作るたびに `register` が `TalkConfig` から作り直す |
| `sakura_builder` → `appearance` | `APPEARANCE.observe`・`APPEARANCE.restore` | 外見状態の表、アクター、スポット、出力した文字列またはグループの内側のトークン | 状態の表は呼び出し側の所有で、`appearance` はその場で書き換える。文字列とトークンは読むだけ |
| `sakura_builder` → `appearance` | `APPEARANCE.tag_at` | 変換前のトークンのテキストと位置 → 単位の名前・引数・直後の位置・囲みの中身 | 状態を持たない読み取りで、テキストは読むだけ |
| `pasta.shiori.act` → 呼び出し側 | `build` の戻り値 | さくらスクリプト文字列 | `act:yield()` などが `coroutine.yield` で渡す（[ランタイム実行モデル](execution-model.md#イベントからシーンへ)） |

## 不変条件と制約

- 後処理（ウェイト挿入と budoux 改行）は、`talk`・`sakura_script` のトークンごとに組立の途中で適用される。組み立て終わった文字列に後から適用する処理は無い。
- 後処理は入力の文字とタグを並べ替えず、`\_w[…]` と `\n` を挿入するだけである。タグの文字列はそのまま残る。
- 後処理の設定は 2 か所から来る。文字の集合とウェイトの既定値は `register` の時点の `TalkConfig` で固定され、トークナイザの正規表現も登録時に 1 度だけコンパイルされる。アクター別のウェイト値と `budoux` の幅は `talk_to_script` の呼び出しのたびにアクターの表から読む。
- budoux の行の幅は `talk_to_script` の呼び出しごとに 1 行目から数え直す。`merge_consecutive_talks` で結合された `talk` は 1 回の呼び出しになるが、`sakura_script` などで区切られた `talk` は別の呼び出しになる。組立が出す `\n`・`\n[N]` や前の呼び出しの改行は幅の計算に入らない。テキスト内の `\n` も幅を数えないタグとして扱うため、そこでも行の幅は数え直さない。
- 単位（タグ・エスケープ・囲み）はどれも budoux 改行の幅に数えない。エスケープと中身のある囲みはバルーンに字を表示するが、ほかのタグと同じく幅 0 として扱われる。そのため囲みが長いと、その行は幅の上限を超えることがある。
- `budoux` の配列に整数にできない要素があると `talk_to_script` は Lua のエラーになる。ウェイトのキーは読めなければ既定値になる。
- 出力の先頭と `clear_spot` の後で、発言より前に積んだ `surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout` は、アクター未指定として、積んだ位置に積んだ順で出力される。スコープ切替タグを伴わないため、その時点のスコープに効く。内側の出力にサーフェス変更などの分類に当たるタグがあれば全スポットが不明になり（[アピアランスの観測と復旧](#アピアランスの観測と復旧)）、次の発言者の切り替えで復旧タグが出ることがある。
- `clear_spot` は現在のグループを閉じる。そのため `clear_spot` の後の発言は、前のグループと同じアクターでも新しいグループに入り、`clear_spot` で組立の状態をリセットした後の切り替えの出力（`\p[N]`）の後に出力される。
- `spot` はグループを閉じない。同じアクターの発言が `spot` の前後にあると、後の発言も前のグループに入り、`spot` の処理はそのグループの出力の後になる。それでも出力は変わらない。`spot` をまたいで同じグループに入るのは切り替えを伴わない同じ発言者の発言だけであり、`spot` が記録した立ち位置は次に発言者が切り替わるときに初めて使われるためである。
- 組立の状態のうち、`actor_spots` と外見状態はビルドをまたいで `STORE` に残り、`last_actor`・`spot_has_text`・`pending_break` はビルドごとに作り直す。そのため段落区切りの改行は 1 回の出力の中だけで判定され、外見の復旧はトークをまたいで判定される。
- `clear_spot` は `actor_spots` を表ごと差し替えず、エントリを 1 つずつ消す。`STORE.actor_spots` と同じ表を指し続けるためである。外見状態は `clear_spot` では消さない。
- アピアランスは pasta 自身が出力した文字列だけを観測する。ベースウェア側だけで起きた外見の変化は状態に入らない。
- さくらスクリプトの単位の読み方の定義は 4 か所にある。Rust の `Tokenizer::SAKURA_TAG_PATTERN`（`crates/pasta_lua/src/sakura_script/tokenizer.rs`）が正で、Lua の `tag_at`（`crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua`）、DSL の文法の `sakura_script`・`sakura_escape` の規則（`crates/pasta_dsl/src/parser/grammar.pest`）、VS Code 拡張の文法定義の `inline-sakura-script`・`inline-escape`（`editors/vscode/syntaxes/pasta.tmLanguage.json`）がその写しである。正の正規表現は、後戻りの有無で一致が変わらない形にしてあるため、左から 1 回読むだけの写し（Lua・pest）と結果が一致する。
- Rust・Lua・DSL の 3 つの読み取りは、適合テスト `crates/pasta_lua/tests/sakura_script/conformance_test.rs` の 1 つの事例の表（入力と期待する単位の並び）で、同じ位置で単位を切り出すことを確かめる。DSL だけが異なるのは、単位にならない `\` をパースエラーにすることである。また DSL は引数・引用・囲みを 1 行の中だけで読む。VS Code 拡張の文法定義は色分けだけに使われ、適合テストの対象ではない（`editors/vscode/src/test/tmGrammar.test.ts` で確かめる）。読み方を変えるときは 4 か所を同じ変更でそろえる。
- 囲みはトークンをまたがない。後処理は 1 回の `talk_to_script` に渡したテキスト（結合後の `talk`、または 1 つの `sakura_script`）の中でだけ囲みを読み、字の判定と外見の観測もトークン（観測は出力した文字列）ごとに読むため、トークンをまたぐ状態を持たない。開きと閉じを別々のトークンで積むと、それぞれが閉じの無い `\_?`（`\_!`）として、ただのタグに読まれる。
- 字の判定は変換前のテキストに対して行い、外見の観測は後処理済みの出力文字列に対して行う。後処理が挿入する `\_w[…]`・`\n` は字の判定に入らない。

## ソースの所在

- `crates/pasta_lua/src/sakura_script/`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`

本文で参照したその他のファイルは次のとおりである。トークンの蓄積とグループ化は `crates/pasta_lua/pasta_scripts/pasta/act.lua`、アクタープロキシは `crates/pasta_lua/pasta_scripts/pasta/actor.lua`、`STORE.actor_spots`・`STORE.appearance` は `crates/pasta_lua/pasta_scripts/pasta/store.lua`、文字列バッファは `crates/pasta_lua/pasta_scripts/pasta/buf.lua`、`TalkConfig` は `crates/pasta_lua/src/loader/config/sections.rs` にある。振る舞いのテストは `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua`（組立と字の判定）と `crates/pasta_lua/tests/lua_specs/appearance_test.lua`（外見の観測と復旧）にある。単位の読み方の写しは `crates/pasta_dsl/src/parser/grammar.pest` と `editors/vscode/syntaxes/pasta.tmLanguage.json` にあり、3 つの読み取りの適合テストは `crates/pasta_lua/tests/sakura_script/conformance_test.rs` にある。

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
- [paragraph-break-tag-only-talk](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/paragraph-break-tag-only-talk/) — 字を出すトークンによる段落区切りの判定と、単位（タグ・エスケープ・囲み）の読み方を SSP にそろえる変更、3 つの読み取りの適合テスト

---

台詞ひとつにも、これだけの仕立てが施されておりますのよ。フンッ、見直しまして？
次は、開発者の強い味方、デバッグ基盤へ参りましょう！
