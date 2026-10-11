# Research & Design Decisions

## Summary
- **Feature**: `choice-line-layout`
- **Discovery Scope**: Extension（さくらスクリプトの組み立ての既存の状態機械に、選択肢の前後の行の区切りを足す。原因は出力と決まった。見本の辞書と章は変えない）
- **Key Findings**:
  - 選択肢の出力は `\![*]\q[表示,ジャンプ先,グローバルシーン名]` だけで、前にも後ろにも、選択肢と選択肢の間にも改行を出さない（`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` の `choice_to_script`・`inner_token_to_string`）。brief の候補 1（選択肢の前に改行が出ていない）がコードの上では成り立つ。
  - UKADOC は `\q[タイトル,ID]` について「テキストの自動改行を求める場合は `\__q[]` タグを使うこと」と書いている。`\q` の中の文字は折り返されないので、台詞の後ろに続いた選択肢は行の右端で切れる。実機の観察（「をする」が切れる）と合う。
  - 2 つの選択肢の間にも改行が無い。2 つ目が次の行に出たのは、1 つ目が右端を越えた結果とみられ、短い選択肢どうしなら同じ行に並ぶ（実機で確かめた。下の「実機での確認」）。「1 つずつ別の行に」を満たすには、1 つ目の前だけでなく、選択肢ごとに行の区切りが要る。
  - 組み立ての状態機械は「その吹き出しに字を出したか」（`spot_has_text`）を既に持っている。選択肢の前に行の区切りを入れるかの判定は、この状態の上に足せる。
  - 実機のさくらスクリプトを採った（2026-10-10。下の「実機での確認」）。コードから導いた形と合い、原因は候補 1 に決まった。SSP の MCP の `raise_event` が、返ったスクリプトをそのまま返すので、ログを読むより早い。
  - emo2（`ghost_dev`）は選択肢行を使わず、台詞の中に `\q[…]`・`\n`・`\_l[…]` を手で書いている。選択肢のトークンだけを直す限り、emo2 の出力は変わらない。
  - （設計フェーズ）`[ghost]` の既定値は Rust の `GhostConfig` が 1 か所で持ち、Lua の既定値のリテラルとマニュアルのリファレンスが食い違わないことをテストが検査している。`choice_newlines` を `spot_newlines` と同じ形にするには、`pasta/shiori/act.lua` だけでなく Rust の loader にも足す（下の「設計フェーズの調査」）。
  - （設計フェーズ）台詞の行に書いた `\c` は `sakura_script` のトークンになり、組み立ては吹き出しの消去として数えない。数えるのは `act:clear()` が積む `clear` のトークンだけである。今の段落の区切りも同じである。

## Research Log

### 設計フェーズの調査（2026-10-10）
- **Context**: 「Design Decisions」の 5 つを決めるための、既存のコードの読み直し（Extension 向けの軽い調査）。
- **Sources Consulted**: `sakura_builder.lua`、`pasta/act.lua`（`group_by_actor`）、`pasta/shiori/act.lua`、`pasta/config.lua`、`crates/pasta_lua/src/loader/config/sections.rs`・`config/mod.rs`・`mod.rs`、`crates/pasta_lua/tests/loader/`（`config_defaults_test.rs`・`config_sections_test.rs`・`startup_test.rs`）、`crates/pasta_lua/.luacheckrc`、同梱の luacheck の複雑度の数え方（`stages/detect_cyclomatic_complexity.lua`）、`crates/pasta_lua/src/code_gen/element_gen.rs`・`scope_gen.rs`、`book/src/internals/talk-output.md`、`book/tools/gen-skill-refs.mjs`、`.kiro/steering/roadmap.md`
- **Findings**:
  1. **設定の既定値の持ち方**。`spot_newlines` の既定値は `GhostConfig`（`config/sections.rs`）が持ち、`apply_shiori_defaults`（`config/mod.rs`）が `[ghost]` に書かれていないキーだけを補う。`config_defaults_test.rs` が、Rust の既定値、`pasta/shiori/act.lua` の既定値のリテラル（`"spot_newlines", 1.5` の文字列を探す）、`book/src/reference/pasta-toml.md` の表とテンプレートの 3 つを照合している。補うキーを 4 つと数えている文書は、`book/src/internals/loader.md`・`book/src/lua/modules/pasta-config.md`・`book/AUTHORING.md` にある。
  2. **数値でない値の扱い**。`spot_newlines` は、Rust でも Lua でも検査・丸め・範囲の制限をしていない。`CONFIG.get` は `nil` のときだけ既定値を返し、`BUILDER.build` は `config.spot_newlines or 1.5` を 100 倍して切り捨てる。数値として計算できない値は、区切りを出す時点で Lua のエラーになるとみられる（コードの読み。走らせてはいない）。
  3. **話し手なしのグループ**。`group_by_actor` が話し手なしのグループを開くのは、出力の先頭と `clear_spot` の後だけである。どちらでも、組み立ての `last_spot` は `nil` である。選択肢だけのシーン（選択肢行だけを書いたシーン）は、このグループになる。
  4. **吹き出しの消去**。`clear` のトークンを積むのは `ACT_IMPL.clear`（`act:clear()`）だけである。台詞の行の `\c` は `Action::SakuraScript` として `sakura_script(…)` に変わる（`element_gen.rs`）。組み立ては `inner.type == "clear"` だけで状態を戻す。
  5. **`clear_spot`**。話し手を宣言したシーンの先頭で `act:clear_spot()` が出る（`scope_gen.rs`）。組み立ては、ここで状態を全部戻す。同じ出力の途中で通ると、吹き出しに既に出ているものを組み立ては知らない。
  6. **複雑度の例外**。`.luacheckrc` は `max_cyclomatic_complexity = 15` だけを持ち、ファイルや関数ごとの例外は書いていない。`BUILDER.build` は、関数の行の `-- luacheck: ignore 561` で許している。luacheck は CI（`build.yml` の `luacheck` のジョブ）で `pasta_scripts` と `tests` に走る。複雑度は、`if`・`elseif`・ループ・`and`・`or` を 1 つずつ数える。
  7. **期待値の数**。完全一致で選択肢を照合しているのは、`call_execution_correctness_e2e_test.rs` の 8 件（317・335・346・360・373・384・397・408 行）と `shiori_act_test.lua` の 751 行の 1 件で、合わせて 9 件である（grep で数え直した）。`sakura_builder_test.lua` の選択肢のテストは部分一致と、2 種類のバッファの出力の一致で、区切りが増えても落ちない。
  8. **生成元のページ**。`gen-skill-refs.mjs` の `GENERATION_MAP` にあるのは、本 spec が触るページのうち `grammar/block-structure.md`・`reference/pasta-toml.md`・`lua/script-api.md`・`lua/modules/pasta-config.md`・`internals/internal-modules.md` の 5 つである。`internals/talk-output.md`・`internals/loader.md` は生成元ではない。
  9. **ロードマップの「`act.lua`」**。同じウェーブの約束（108 行）は「`act.lua`・`element_gen.rs`…を触らない」と書き、「共有接点」（146 行）は `crates/pasta_lua/pasta_scripts/pasta/act.lua` を名指しして「1 ウェーブに 1 spec だけが持つ」と書いている。約束の `act.lua` は、選択肢を積む側の `pasta/act.lua` と読める。`pasta/shiori/act.lua` を触る spec は、同じウェーブに無い。
- **Implications**: 設計の決定（下の Design Decisions）と、設計ディスカッションへ送る未決の事項（`design.md` の「未決の事項」）。
- **参照した手引き**: `kiro-spec-design` の規則（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）。変更は最小にし、既にある型（`spot_newlines` の読み方・換算・テスト・文書）を写す方針で決めた。

### 実機での確認（2026-10-10・要件ディスカッションの最初）
- **Context**: brief の議題 1（原因はどれか）。Requirement 1 の記録。
- **やり方**: `getting-started-story-guide` の実機確認で使ったゴースト（v0.3.8 の `pasta.dll`。`dic/09-choice.pasta` は今の main と同じ）を複製し、SSP を `--option readonly --ghost` で起動した。吹き出しは SSP の既定のバルーン。SSP の MCP で `OnMouseDoubleClick` を起こして応答を採り、その後に手書きのさくらスクリプトを送って、吹き出しの絵（`dump_balloon`）を見比べた。
- **Findings**:
  1. 9 段目の選択肢のシーンが返したさくらスクリプト（そのまま）:

     ```text
     \p[0]\s[0]ねえ、\_w[450]ユーザーさん。\_w[950]なんの話をしよっか？\_w[450]\![*]\q[おやつの話をする,おやつの話,OnMouseDoubleClick_4]\![*]\q[おでかけの話,おでかけの話,OnMouseDoubleClick_4]\![set,choicetimeout,10000]\e
     ```

     コードから導いた形（下の「選択肢の出力の経路」）と合う。選択肢の前にも、選択肢どうしの間にも、改行が無い。
  2. 吹き出しの 1 行目は「ねえ、ユーザーさん。なんの話をしよっか？●おやつの話」で右端に届き、残りは切れる。2 行目に「おでかけの話」が出る。2 行目にマーカーは見えない（1 行目の右端の外に出たとみられる）。brief の観察と同じである。
  3. `どっち？\![*]\q[はい,x1]\![*]\q[いいえ,x2]つづきの台詞。` を送ると、「どっち？●はい●いいえつづきの台詞。」が 1 行に並ぶ。短い選択肢どうしは同じ行に並び、選択肢の後ろの台詞は選択肢のすぐ後ろに続く。9 段目で 2 つ目が次の行に出たのは、1 つ目が右端を越えた結果である。
  4. 問いかけの後ろと、選択肢と選択肢の間に `\n` を 1 つずつ入れて送ると、問いかけ・「●おやつの話をする」・「●おでかけの話」が 3 行に分かれ、どれも切れない。マーカーは各行の先頭に出る。行間は台詞の行と同じである。
  5. 問いかけと 1 つ目の選択肢の間だけを `\n[150]` にし、最後の選択肢の後ろに `\n` と台詞を足して送ると、問いかけと選択肢の間が半行ぶん空き、台詞は選択肢の次の行に出る。
  6. 行の先頭から出した長い選択肢（31 文字）は、`\q[…]` では右端で切れ、`\![*]\__q[ID]…\__q` では 2 行に折り返される。
- **Implications**: 原因は brief の候補 1（選択肢の前に改行が出ていない）である。候補 2 は、改行が出ていないので成り立たない。候補 3（バルーンの幅）は、`\n` を入れれば既定のバルーンで収まるので、直す場所ではない。直す場所は出力で、選択肢ごとに行の区切りが要る（1 つ目の前だけでは、短い選択肢が同じ行に並ぶ）。選択肢の後ろの台詞の前にも要る。

### 直した後の実機での確認（2026-10-11・タスク 4.2）
- **Context**: Requirement 6 の 1〜5、Requirement 4 の 1・2・6。直した `pasta.dll` を、配布している形の hello-pasta に入れて確かめる。Requirement 1 の 3 つが記録済みであることの読み合わせも、ここで行う。
- **やり方**:
  - ビルド: コミット `2ef7de6d`（タスク 1.1〜3.3 と `origin/main` のマージが入った状態）で、`crates/pasta_sample_ghost/release.ps1` を引数なしで走らせた。`cargo build --release --target i686-pc-windows-msvc -p pasta_shiori` が `pasta.dll` を作り（3,950,080 バイト、SHA-256 `398504642a471d130a4d0df088058f19b39dab2af15ccd58f0f8e0fedf69c6d5`）、`pasta_check release` が配布の形（`release/hello-pasta/`）と `hello-pasta.nar` を作る。配布の形の `dic/09-choice.pasta` は、リポジトリの辞書と同じである（`diff` で差が無い）。`pasta.toml` に `choice_newlines` は書いていない（既定の `1.5`）。
  - 起動: `release/hello-pasta/` を作業用のフォルダへ複製し、`ghost/master/descript.txt` の `name` だけを `hello-pasta-wt` に変えた（開発機の SSP に、同じ名前の hello-pasta が登録されているため）。SSP 2.9.11 を `--option readonly --ghost <複製の絶対パス>` で起動した。バルーンは SSP の既定（プロパティ `currentghost.balloon.name` は「SSPデフォルト+」）である。
  - 操作: SSP の MCP の `raise_event` で `OnMouseDoubleClick`（Reference0〜5 は `100`・`100`・`0`・`0`・`Head`・`0`）を、選択肢のシーンが返るまで繰り返した。返ったスクリプトは `raise_event` の結果と、`get_log`（`script`）、ゴーストの `profile/pasta/logs/pasta.log` の 3 か所で読んだ。吹き出しの絵は `dump_balloon`（スコープ 0）、状態は `get_status` で見た。最後に `\-\e` を送って SSP を終了した（プロセスが残っていないことを確かめた）。選択肢のクリックは、同じ複製を 13:32 に起動し直して、マウスで行った（下の 4）。
- **Findings**:
  1. 9 段目の選択肢のシーンが返したさくらスクリプト（そのまま）:

     ```text
     \p[0]\s[0]ねえ、\_w[450]ユーザーさん。\_w[950]なんの話をしよっか？\_w[450]\n[150]\![*]\q[おやつの話をする,おやつの話,OnMouseDoubleClick_4]\n\![*]\q[おでかけの話,おでかけの話,OnMouseDoubleClick_4]\![set,choicetimeout,10000]\e
     ```

     選択肢のシーンは 5 回返り、5 回とも同じ文字列だった。`design.md` の Overview の「直す:」の行と、1 バイトも違わない（`pasta.log` の `Value:` の行と `cmp` で比べた）。直す前の記録（上の「実機での確認」の 1）から増えたのは、`\n[150]` と `\n` の 2 つだけである。
  2. 吹き出しの絵（`dump_balloon`。335 × 169 ピクセル）。問いかけ「ねえ、ユーザーさん。なんの話をしよっか？」、「●おやつの話をする」、「●おでかけの話」が、3 行に分かれて出る。
     - 字のある行の縦の位置は、上から 14〜24・35〜45・49〜59 ピクセルである。行の送りは、1 行目から 2 行目が 21 ピクセル、2 行目から 3 行目が 14 ピクセルで、問いかけと 1 つ目の選択肢の間が半行ぶん広い（`\n[150]` と `\n` のとおり）。
     - 各行の字の右端は、258・126・103 ピクセルである。吹き出しの幅（335）の手前で終わっており、どの行も切れていない。
     - マーカーは、2 行目と 3 行目の先頭（左から 25〜33 ピクセル。問いかけの字の左端は 23 ピクセル）に出る。
     - 直す前は、1 行目が右端に届いて「をする」が切れ、2 行目のマーカーが見えなかった（上の「実機での確認」の 2）。どちらも無くなった。
  3. 章（`book/src/getting-started/09-choice.md`）の「成功の目印」と読み合わせた。手順 3 の「女の子の吹き出しに『ねえ、ユーザーさん。なんの話をしよっか？』と出て、続けて選択肢『おやつの話をする』と『おでかけの話』が出る」は、2 の見え方と食い違わない。手順 4・5・6 の後の目印の台詞と動きは、下の 4・5 の結果と同じである。
  4. 選択肢を選んだ後。吹き出しの選択肢を、画面の上で実際にマウスでクリックした（同じ日の 13:40〜13:41。開発者の許可を得て、Claude が computer-use で SSP を操作した。ビルドと複製したゴーストは上と同じ。選択肢のシーンを出すところは `raise_event` の `OnMouseDoubleClick` で、クリックだけがマウスである）。
     - 2 行目の「おやつの話をする」をクリック → `pasta.log` に、13:40:50.9 の要求 `OnChoiceSelectEx`（`Sender: SSP`・`SenderType: internal`・`Status: choosing,balloon(0=0)`。Reference0〜2 は `おやつの話をする`・`おやつの話`・`OnMouseDoubleClick_4`）と、応答 `\p[0]\s[6]今日のおやつは、\_w[450]プリンがいいな！\_w[450]\p[1]\s[15]きのうもプリンだったよね。\_w[950]\e` が残った。画面でも、女の子と男の子の吹き出しにこの掛け合いが出た（章の手順 4 の目印）。
     - 3 行目の「おでかけの話」をクリック → 13:41:20.0 の要求 `OnChoiceSelectEx`（同じく `Sender: SSP`・`SenderType: internal`。Reference0〜2 は `おでかけの話`・`おでかけの話`・`OnMouseDoubleClick_4`）と、応答 `\p[0]\s[0]晴れたら、\_w[450]お弁当を持って出かけたいな。\_w[950]\p[1]\s[11]ユーザーさんの分も作ろうか。\_w[950]\e`（章の手順 5 の目印）。
     - 3 行に分かれた選択肢は、2 行目も 3 行目もクリックできる。マウスが載っている選択肢の行は、反転して表示される（2 行目で見た）。SSP が送る Reference は、返ったスクリプトの `\q[…]` の 3 つの引数（表示・ジャンプ先・グローバルシーン名）と同じで、pasta はジャンプ先のシーン（`＊おやつの話`・`＊おでかけの話`）へ進む。
     - クリックの前（12 時台）に、デスクトップを操作する許可が出ていなかった間の代わりの確認として、選択肢が出ている間に同じ Reference の `OnChoiceSelectEx` を `raise_event` で起こしていた（`Sender: MCP`）。応答は、上の 2 つとそれぞれ同じ文字列だった。
  5. 時間切れ。選択肢を出して何も選ばずに待ち、`get_status` を読んだ（時刻は、`pasta.log` の応答の時刻と、待ちの前後に採った時計）。

     | 回 | 選択肢の応答 | 途中 | 待った後 | その間に再生されたスクリプト |
     | -- | ------------ | ---- | -------- | ---------------------------- |
     | 1 | 12:25:18.9 | 直後に `choosing,balloon(0=0)` | 12:25:39 に空（選択中でない。吹き出しも閉じている） | 無い |
     | 2 | 12:31:46.3 | 12:31:55 に `choosing,balloon(0=0)` | 12:32:08 に `balloon(0=0/1=0)`（選択中でない） | 12:31 台に `OnSecondChange` のランダムトークが 1 つ |
     | 3 | 12:32:30.7 | 12:32:40 に `choosing,balloon(0=0)` | 12:32:49 に空 | 無い |

     - 3 回とも、選んでいないのに選択中の状態が終わり、「おやつの話」「おでかけの話」のどちらの掛け合いも出なかった。章の手順 6 の目印（「選択肢が時間切れになり、どちらの掛け合いも出ない」）と合う。3 回目では、応答から 9.5 秒の時点でまだ選択中で、18.3 秒の時点では終わっていた（応答の時刻は、台詞を表示し終える前である）。
     - `OnChoiceTimeout` のイベントそのものは見ていない。`pasta.log` は 204 でない応答だけを書き、選択肢の応答の後に `OnChoiceTimeout` への応答は 1 件も無い（9 段目の辞書に受けるシーンが無いので、204 を返したとみられる）。SSP の MCP には、イベントの記録を読む道具が無い。時間切れは、状態の変化と、掛け合いが出ないことから読んだ。
     - 2 回目のランダムトークは、`pasta.log` では 12:31:59.4 の応答である（選択肢の応答の 13.1 秒後。12:31:55 の時点では、まだ選択中だった）。この `OnSecondChange` は Reference3 が `1`（トークできる状態）で来ており、選択中の状態が終わった後のトークである。選択肢の制限時間と、トークの間隔（この見本は 45〜75 秒）が重なった回で、選択肢の配置とは関係が無い。
  6. エラーは出ていない。SSP のエラーログ（`get_log` の `error`）は空で、`pasta.log` に `ERROR` は無い（`WARN` は、初回起動の「保存ファイルが無い」の 1 件だけ）。
  7. 洗い出し（Requirement 6 の 5）。リポジトリの中を、選択肢行（`＠？`・`@?`）と `act:choice`（`:choice(`）で検索し直した。
     - 選択肢行を使う辞書（`*.pasta`）は 3 枚で、下の「既存のゴーストへの影響」の一覧と変わらない: `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/09-choice.pasta`、`crates/pasta_shiori/tests/fixtures/call_execution_correctness/dic/choice_scope.pasta`、同 `characterization.pasta`。どの選択肢行も、直前の行は台詞の行（末尾に `\n` が無い）か、別の選択肢行である。
     - マニュアルの作例: `book/src/grammar/block-structure.md`（2 つ）、`book/src/lua/shiori-events.md`、`book/src/reference/pasta-toml.md`（本 spec が足した `choice_newlines` の例）、`book/src/getting-started/09-choice.md`（辞書の写し）。スキルの `authoring-patterns.md`・`SKILL.md` と、生成物の参照資料も同じ形である。どれも「台詞の行 → 選択肢行」で、選択肢の直前に改行を書いていない。
     - `act:choice` を使う作例: `book/src/lua/script-api.md`・`book/src/lua/patterns.md`（どちらも `act:talk(…)` の直後に `act:choice` を並べる。改行は積んでいない）。
     - テスト: `pasta_dsl`・`pasta_lua` のテストの中の DSL と Lua も、選択肢の直前に改行を書いていない。例外は、作者の改行と区切りが並んで出ることを固定するテスト 1 件（`sakura_builder_test.lua` の「2.7: 台詞の末尾の `\n` と newline のトークンは、区切りと並んで両方出る」）で、意図して書いたものである。`shiori_act_test.lua` の 748 行は `act:newline()` → `act:clear()` → `act:choice` の順で、間に消去が入る。
     - 直した後に見え方が壊れる辞書と作例は、見つからなかった。
  8. Requirement 1 の読み合わせ。3 つとも、上の「実機での確認（2026-10-10）」と「ベースウェアの仕様」に記録済みである（書き足していない）。1.1 は Findings の 1（v0.3.8 が返したさくらスクリプト）、1.2 は Implications と「ベースウェアの仕様」の `\q` の記述、1.3 は Findings の 2・3（1 つ目が右端を越えたので 2 つ目が次の行に出たこと、短い選択肢どうしは同じ行に並ぶこと）である。
- **Implications**:
  - **直した場所**（Requirement 6 の 3）。出力である（`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` の `BUILDER.build`。幅は `pasta.toml` の `[ghost]` の `choice_newlines`）。辞書（`09-choice.pasta`）・章・段階表・バルーンは変えていない。今回の確認でも、辞書はリポジトリのまま、バルーンは SSP の既定のままで、3 行に収まった。
  - **辞書で直さなかった理由**。回避の書き方を見本にも入門ガイドにも載せない方針である。辞書に改行を足しても、選択肢と選択肢の間は直らず、作例どおりに書いた他の作者も直らない（下の Architecture Pattern Evaluation の B）。
  - **バルーンで直さなかった理由**。既定のバルーンで収まる。幅を広げても、台詞の後ろに選択肢が続く並びは変わらず、長い問いかけで再発する（同 D。`design.md` の「記録層」の 6.3）。
  - **既存のゴーストで区切りが増える場所**（Requirement 6 の 4。`design.md` の「既存のゴーストでの見え方の変化」）。選択肢行または `act:choice` を使うゴーストで、pasta を更新すると次の 3 か所に区切りが増える。

    | 場所 | 増えるもの | 見え方 |
    | ---- | ---------- | ------ |
    | 字の後の選択肢の前 | `\n[150]`（`choice_newlines` で変えられる） | 選択肢が、台詞から半行ぶん空けた次の行から出る |
    | 選択肢の後の選択肢の前 | `\n` | 選択肢が 1 行に 1 つになる |
    | 選択肢の後の台詞の前 | `\n`（話し手が切り替わって戻った直後なら、今も出ている `\n[spot_newlines × 100]` のまま） | 台詞が選択肢の次の行から出る |

    - 選択肢の直前や直後に自分で改行を書いていたゴーストは、空の行が 1 行増える。
    - 台詞に `\c` を書いているゴーストは、選択肢を使っていなくても、`\c` で消した吹き出しの最初に出ていた `\n[spot_newlines × 100]` が出なくなる（Requirement 7 の 2）。
    - 台詞の中に `\q[…]` を手で書いているゴースト（emo2 など）と、選択肢を使わないトークは、変わらない。
    - 元の見え方へ戻す設定は無い（`choice_newlines` は幅だけを変える）。
  - 今回の実機で見たのは、表の 1 行目と 2 行目（9 段目の並び）だけである。3 行目（選択肢の後の台詞）と `\c` の後の並びは、実機では見ていない（テストの完全一致と、2026-10-10 の手書きのさくらスクリプトでの見え方による）。

### 選択肢の出力の経路
- **Context**: brief の「1 つ目の選択肢の前に改行を入れているかは調べていない」。
- **Sources Consulted**: `crates/pasta_lua/src/code_gen/scope_gen.rs`（`generate_choice`・`generate_choice_timeout`）、`crates/pasta_lua/pasta_scripts/pasta/act.lua`（`ACT_IMPL.choice`・`group_by_actor`・`merge_consecutive_talks`）、`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`
- **Findings**:
  - 選択肢行 `＠？ジャンプ先「表示」` は `act:choice("ジャンプ先", "表示")` に変わる。`!select(10)` は `act:choice_timeout(10)` に変わる。どちらも改行を足さない。
  - `act:choice` は `{ type = "choice", target, display, scope }` を積むだけである。グループ化では、選択肢は直前の話し手のグループに入る（グループが無ければ、話し手なしのグループを開く）。
  - 組み立て（`BUILDER.build`）では、選択肢は「S4」の枝（字を出さないトークン）を通り、`\![*]\q[…]` をそのまま出す。`has-text` も、段落の区切りの保留（`pending_break`）も変えない。
  - 台詞の行の終わりに、改行は出ない（文法の規則。改行は作者が `\n` を書く。`book/src/grammar/action-line.md` の「改行」）。
  - コードから導いた 9 段目の出力（ウェイトのタグと表情の番号は省略。`N` は登録の通し番号）:

    ```text
    \p[0]\s[…]ねえ、ユーザーさん。なんの話をしよっか？\![*]\q[おやつの話をする,おやつの話,OnMouseDoubleClick_N]\![*]\q[おでかけの話,おでかけの話,OnMouseDoubleClick_N]\![set,choicetimeout,10000]\e
    ```

  - 既存のテストの期待値も同じ形である。`crates/pasta_shiori/tests/call_execution_correctness_e2e_test.rs` の `\p[0]どれ？\_w[450]\![*]\q[甲,選択肢甲,OnChoiceMenu_1]\![*]\q[乙,選択肢乙,OnChoiceMenu_1]\e` など。
- **Implications**: 候補 1 がコードの上で成り立つ。候補 2（改行は出ているが折り返しと重なる）は、改行が出ていないので成り立たない。候補 3（バルーンの幅）は、台詞の後ろに選択肢が続くかぎり、幅を広げても長い問いかけで再発する。実機の出力で確かめる（Requirement 1）。

### ベースウェアの仕様（`\q` と折り返し）
- **Context**: 「をする」が切れ、2 つ目が次の行に出る理由。
- **Sources Consulted**: UKADOC「さくらスクリプトリスト」の `\q[タイトル,ID]`・`\q[タイトル,ID,r2,r3...]`・`\__q[ID,...]`・`\![*]`・`\![set,choicetimeout,時間]`
- **Findings**:
  - `\q[タイトル,ID]`:「テキストの自動改行を求める場合は `\__q[]` タグを使うこと」。
  - `\__q[ID,...]` 〜 `\__q`:「`\q[]` タグと異なり、自動改行も行われる」。範囲の中にさくらスクリプトも挟める。
  - 旧仕様の `\q[ID][タイトル]` は「選択肢は改行される」。今の `\q[タイトル,ID]` は、選択肢ごとの改行をしない。
  - `\![*]` は、バルーンが決めたマーカーを表示する。
  - `\![set,choicetimeout,時間]` は、選択肢より後ろに書いても効く。
- **Implications**: 選択肢ごとの改行は、出す側（pasta か作者）が入れるものである。UKADOC の記述例も、選択肢を `\n` なしで並べている（1 行に並べる前提の例）。2 つ目の選択肢が次の行に出た理由は、UKADOC からは決まらない（下の Research Needed 2）。

### 既にある改行の規則（話し手の切り替え）
- **Context**: 選択肢の前の行の区切りを、どの状態から判定できるか。
- **Sources Consulted**: `sakura_builder.lua`（`spot_has_text`・`pending_break`・`emits_text`）、`book/src/reference/pasta-toml.md` の `spot_newlines`、`.kiro/specs/completed/sakura-script-newline/`
- **Findings**:
  - `spot_has_text[spot]` は「1 回の出力の中で、その吹き出しに字を出したか」を持つ。`\c` と `clear_spot` で偽へ戻る。
  - 字を出すかの判定（`emits_text`）は、字のある `talk` と、字を表示する `sakura_script` だけを数える。`raw_script`・表情・ウェイト・選択肢は数えない。
  - 話し手が切り替わって、字のある吹き出しへ戻るとき、次の字の直前に `\n[150]`（`spot_newlines` × 100）を出す。保留（`pending_break`）は字が来るまで出さない。
  - `BUILDER.build` は、luacheck の複雑度の上限（15）を越えていて（22）、`-- luacheck: ignore 561` と `ponytail:` のコメントで許している。コメントは「分割はリファクタ時に（特性化テスト先行）」と書いている。
- **Implications**: Requirement 2 の 1・3（字の後なら区切る・最初なら区切らない）は `spot_has_text` で判定できる。2・5（選択肢の後）は、吹き出しごとの「選択肢を出した」状態が新しく要る。保留中の `\n[150]` と、選択肢の後の台詞の区切り（Requirement 2 の 5）が重なる場合の扱いを、設計で決める。枝を足すと `build` の複雑度がさらに上がる。

### 既存のテストと文書への波及
- **Context**: 出力を変えたときに一緒に変わるもの。
- **Sources Consulted**: `crates/pasta_lua/tests/lua_specs/`、`crates/pasta_shiori/tests/`、`crates/pasta_lua/tests/`、`crates/pasta_sample_ghost/tests/`、`book/src/`、`book/tools/gen-skill-refs.mjs`
- **Findings**:
  - `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua` の選択肢のテスト（1612〜2010 行あたり）は、部分一致（`find`）と、2 種類のバッファの出力の一致で見ている。出力に改行が増えても落ちないものが多い。新しい配置のテストは無い。
  - `crates/pasta_lua/tests/lua_specs/shiori_act_test.lua:751` は完全一致で、`\s[5]\_w[500]\n\c\![*]\q[d,t]\![set,choicetimeout,30000]\p[0]A\e` を期待する。話し手なしのグループで `\c` の直後に選択肢を出し、その後で話し手 A が話す形である。最初の選択肢の前は区切らない（Requirement 2 の 3）ので、選択肢の前は変わらない。選択肢の後の `A` は別のグループ（スポット未解決 → 0）なので、選択肢の後ろの台詞の区切り（Requirement 2 の 5）をどう当てるかで期待値が変わる。
  - `crates/pasta_shiori/tests/call_execution_correctness_e2e_test.rs`（317〜408 行）は、選択肢を含む応答を完全一致で 8 件照合している。台詞の後の選択肢、選択肢の後の選択肢、選択肢の後の台詞（`\q[前,…]Ｂ通過Ａ再開`）の形があり、Requirement 2 の 1・2・5 で全部変わる。
  - `crates/pasta_lua/tests/scene_alias_search_test.rs:231`・`symbol_name_search_test.rs:237` は、`\q[…]` の部分一致である。変わらない。
  - `crates/pasta_sample_ghost/tests/tutorial_stages_test.rs` は、9 段目の `OnChoiceSelectEx` が 200 と空でない値を返すことだけを見る。選択肢を出す側の応答の中身は照合しない。出力の修正では落ちない。
  - マニュアルで選択肢の出力の形を書いている場所: `book/src/grammar/block-structure.md` の「選択肢行」（「…として、その行の位置に出力する」）、`book/src/lua/script-api.md` の `act:choice`、`book/src/lua/shiori-events.md` の `OnChoiceSelectEx`。`block-structure.md` と `script-api.md` は、スキルの参照資料の生成元である（`GENERATION_MAP`）。
  - 入門の 9 段目の章（`book/src/getting-started/09-choice.md`）は、辞書を逐語で写している。「成功の目印」は「続けて選択肢…が出る」で、行の配置には触れていない。
- **Implications**: 出力を直すと、`pasta_shiori` の結合テストの期待値 8 件、マニュアル 2〜3 か所、スキルの参照資料の再生成が付いてくる。ロードマップの同じウェーブの約束（「選択肢の出力とそのテスト」）の文面より広いが、同じウェーブの他の spec はこれらを持ち場にしていない（要件の Boundary Context）。

### 既存のゴーストへの影響
- **Context**: brief の Constraints「既にあるゴーストの選択肢の見え方が変わる」。
- **Sources Consulted**: このリポジトリの `.pasta` と `book/src/` の作例、`C:\home\maz\git\ghost_dev`（emo2）の `dic/menu.pasta`
- **Findings**:
  - このリポジトリで選択肢行を使う辞書は、hello-pasta の `09-choice.pasta` と、`pasta_shiori` のテスト用の辞書 2 枚（`choice_scope.pasta`・`characterization.pasta`）だけである。選択肢行の直前に作者が改行を書いたものは無い。
  - マニュアルとスキルの作例（`block-structure.md`・`shiori-events.md`・`authoring-patterns.md`）は、どれも「台詞の行 → 選択肢行を並べる → `!select`」の形で、改行を書いていない。作例どおりに書いた人は、9 段目と同じ見え方になっているはずである。
  - emo2 は選択肢行を使わない。台詞の中に `\q[…]\n\q[…]\_l[5em,2lh]\q[…]` と手で書いている。台詞の中の `\q` は `talk` のテキストとして通るので、選択肢のトークンの修正は届かない。
- **Implications**: 出力の修正で見え方が変わるのは、選択肢行か `act:choice` を使うゴーストだけである。知っている範囲では hello-pasta だけで、変わる向きは「直る」側である。作者が改行を自分で足していた場合だけ、空の行が増える（Requirement 2 の 7）。

## Requirement-to-Asset Map

| Requirement | 今ある資産 | ギャップ | 種別 |
| ----------- | ---------- | -------- | ---- |
| 1 原因の記録 | Research Log の「実機での確認」（2026-10-10 に採取）。UKADOC の `\q` の記述 | なし（記録済み） | —（追加なし） |
| 2 選択肢を 1 行に 1 つ | `sakura_builder.lua` の `spot_has_text`・`emits_text`・`\c` でのリセット | 選択肢の前後に行の区切りを出す処理が無い。「吹き出しに選択肢を出した」状態が無い | Missing |
| 2 の 6（`act:choice` も同じ） | 選択肢行と `act:choice` は同じトークンを積む | 組み立ての側で直せば、両方に効く | —（追加なし） |
| 2 の 7（作者の改行と重ねる） | — | 重ねるだけなら追加なし。省くなら、直前の出力が改行で終わるかの判定が要る | —（重ねると決めた。追加なし） |
| 3 変えない範囲 | 選択肢のタグの中身・エスケープ・`scope` のテスト（`sakura_builder_test.lua`）。選択肢の振り分けのテスト（`choice_select_test.lua`・`pasta_shiori` の結合テスト） | 完全一致の期待値 8 件（`call_execution_correctness_e2e_test.rs`）と 1 件（`shiori_act_test.lua:751`）は、配置の変更に合わせて直す | Constraint |
| 4 見本と入門の 9 段目 | `dic/09-choice.pasta`、`book/src/getting-started/09-choice.md`、`tutorial-check.mjs`、`tutorial_stages_test.rs` | 出力で直すなら、辞書と作例は変えない。「成功の目印」を書き足すかは未決 | —／Unknown |
| 5 マニュアルの記述 | `block-structure.md` の「選択肢行」、`script-api.md` の `act:choice`、`gen-skill-refs.mjs` | 行の配置の説明が無い。スキルの参照資料の再生成が要る（`.claude/skills/` への書き込みは、開発者の許可が要る） | Missing／Constraint |
| 6 実機での確認と記録 | SSP を `--ghost` で起動する手順（`hello-pasta-tutorial-stages`・`getting-started-story-guide` が使った）。SSP MCP | 9 段目の選択肢は、同名の 4 シーンから 1 つ選ばれるので、出るまでダブルクリックを繰り返す。時間切れの確認に 10 秒かかる | Constraint |

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
| ------ | ----------- | --------- | ------------------- | ----- |
| A: 組み立てで行の区切りを足す | `sakura_builder.lua` の状態機械に、選択肢の前（字または選択肢の後）と、選択肢の後の字の前の区切りを足す | 選択肢行と `act:choice` の両方に効く。吹き出しの状態（字を出したか・`\c`）を既に持っている。作者は何も書き足さない。brief の方針（出力で直す）に合う | `build` の複雑度が上がる。完全一致の期待値 9 件とマニュアルを直す。選択肢行を使う既存のゴーストの見え方が変わる | ロードマップの約束の範囲（`sakura_builder.lua` とそのテスト）に収まるのは本体だけ。期待値とマニュアルは、要件の Boundary Context で本 spec の持ち場にした |
| B: 辞書で直す | `09-choice.pasta` の問いかけの終わりに `\n` を書く。章の作例と段階表をそろえる | エンジンを触らない。差分が小さい | brief の方針に反する（辞書の書き方で避ける手順を載せない）。選択肢と選択肢の間は直らない。作例どおりに書いた他の作者は直らない | 原因が出力でないと分かったときだけの退路 |
| C: A に加えて選択肢の形を `\__q` に変える | 選択肢を `\__q[ID,…]表示\__q` で出し、長い表示を自動で折り返させる | 1 行に収まらない選択肢も切れない | マニュアルに書いてある出力の形（`\![*]\q[…]`）が変わる。選択肢のテストをほぼ全部直す。表示の中の文字がさくらスクリプトとして読まれるので、エスケープの規則が変わる。`scene-anchor-link` と持ち場が重なる | 採らない（2026-10-10 開発者の判断。長い表示は救済しない。別の spec にも起票しない） |
| D: バルーンを広げる | hello-pasta に幅の広いバルーンを付ける、または既定のバルーンを指定する | エンジンを触らない | brief の Out（バルーンの絵）。台詞の後ろに選択肢が続く並びは変わらず、長い問いかけで再発する。他のゴーストは直らない | 採らない見込み |

## Design Decisions

設計フェーズ（2026-10-10）で決めた。開発者の判断が要るものは、仮定を置いて `design.md` の「未決の事項」へ送った。

### Decision: 行の区切りを入れる場所
- **Context**: Requirement 2 の 1〜5。
- **Alternatives Considered**:
  1. 組み立て（`sakura_builder.lua`）— 吹き出しの状態を見て出す。
  2. `act:choice`（`act.lua`）で改行のトークンを積む — 吹き出しの状態を知らないので、最初の選択肢でも改行が出る。ロードマップの約束で `act.lua` は触れない。
  3. 生成（`scope_gen.rs`）で `act:newline()` を足す — `act:choice` を直接呼ぶ Lua には効かない。吹き出しの状態も知らない。
- **Selected Approach**: 1。`BUILDER.build` の内側のトークンのループに、選択肢のための枝を 1 つ足し、字の枝に「最後が選択肢なら `\n`」を足す。区切りの文字列は局所関数が決める。
- **Rationale**: 吹き出しの状態（字を出したか・消したか・話し手が切り替わったか）を知っているのは組み立てだけである。2 と 3 は、吹き出しの最初の選択肢にも改行が出る。
- **Trade-offs**: `build` の複雑度が上がる（下の「`BUILDER.build` の複雑度」）。
- **Follow-up**: 区切りは `\![*]` の前、`\p[N]` と外見の復旧タグの後ろに入る（今の段落の区切りと同じ位置）。実機でマーカーが行の先頭に出ることを確かめる。

### Decision: `BUILDER.build` の複雑度
- **Context**: luacheck の上限は 15、今は 22。枝を足すとさらに上がる。
- **Alternatives Considered**:
  1. 今の例外（関数の行の `-- luacheck: ignore 561` と `ponytail:` のコメント）のまま、判定を局所関数に出して増分を小さくする。
  2. `build` を分割して上限に収める。
  3. `.luacheckrc` の上限を上げる。
- **Selected Approach**: 1。コメントの数字は、luacheck で測った値に直す（26 前後の見込み）。
- **Rationale**: 2 は特性化テストを先に置く別の作業で、この修正に要らない。3 は他のファイルの検査が緩む。
- **Follow-up**: 実装のときに luacheck を手元で走らせ、警告が出ないことと数字を確かめる。

### Decision: 区切りのタグと幅
- **Context**: Requirement 2 の 8。
- **Alternatives Considered**:
  1. `\n`（台詞の中の改行と同じ）。
  2. `\n[half]` や `\n[150]` のような幅付き。問いかけと選択肢の間だけ広くする。
- **Selected Approach**: 要件で決まった（2026-10-10 開発者の判断）。字の後の選択肢の前は `\n[N]`（`pasta.toml` の `[ghost]` の `choice_newlines` × 100。既定 `1.5` → `\n[150]`）、選択肢の後の選択肢の前と、選択肢の後の台詞の前は `\n`。

### Decision: `choice_newlines` の読み込みと受け渡し
- **Context**: Requirement 2 の 8〜10。`spot_newlines` は、`pasta/shiori/act.lua` の `SHIORI_ACT.new` が `CONFIG.get("ghost", "spot_newlines", 1.5)` で読み、`BUILDER.build` の設定の表に入れて渡している。
- **Alternatives Considered**:
  1. 同じ経路に足す（`pasta/shiori/act.lua` に 2 行）。`spot_newlines` と扱いがそろう。
  2. 組み立て（`sakura_builder.lua`）の側で設定を直接読む。`act.lua` を触らないが、組み立てが設定のモジュールに依存する。
- **Selected Approach**: 1。`GhostConfig` の既定値 → `apply_shiori_defaults` の補完 → `SHIORI_ACT.new` が `CONFIG.get("ghost", "choice_newlines", 1.5)` で読む → `BUILDER.build` の設定の表、の順に届ける。Rust の loader にも足す（要件フェーズでは `pasta/shiori/act.lua` の 2 行だけを見込んでいた）。
- **Rationale**: 組み立てが設定のモジュールを読まない今の形を保てる。`[ghost]` の既定値は Rust に 1 つという決まりと、その検査に乗れる。リファレンスの「`[ghost]` は書かなくても必ず `@pasta_config` に現れる」が、新しいキーでも成り立つ。
- **Trade-offs**: 持ち場が Rust の loader・loader のテスト・マニュアルの内部のページ・`book/AUTHORING.md` の 1 行まで広がる。ロードマップの持ち場の文面（「`sakura_builder.lua` とそのテスト」）より広い（設計ディスカッションで確かめて確定した。`design.md` の「決定の記録」）。
- **Follow-up（要件フェーズの記録）**: ロードマップの同じウェーブの約束は「`act.lua` を触らない」と書いている。選択肢を積む `pasta/act.lua` を指すとみられるが、文面は `pasta/shiori/act.lua` と区別していない。同じウェーブの他の spec は Lua を持ち場にしていないので、1 を採っても衝突はしない。数値でない値の扱いは、`spot_newlines` の今の動きを先に確かめてそろえる。`pasta.toml` のリファレンス（`book/src/reference/pasta-toml.md`）は、設定の一覧の表・記述例・キーの説明の 3 か所に `spot_newlines` があり、同じ 3 か所に足す。

### Decision: 選択肢を「字」に数えるか
- **Context**: 選択肢だけを出した吹き出しへ話し手が戻るとき、段落の区切り（`\n[150]`）を出すか。今は選択肢を字に数えない。
- **Alternatives Considered**:
  1. 今のまま数えない。選択肢の後の台詞の区切り（Requirement 2 の 5）だけで分ける。
  2. 選択肢を字に数える。`pasta.toml` の `spot_newlines` の説明（「すでに台詞を出したスポット」）も変わる。
- **Selected Approach**: 1。選択肢は字に数えない。`emits_text`・`spot_has_text` と、保留を立てる条件は変えない。吹き出しごとの「最後に表示したもの」（無し・字・選択肢）を、別の状態として 1 つ足す。
- **重なるときの扱い**: 区切りは 1 つだけ出す。字の前では、保留があれば今も出ている `\n[spot_newlines × 100]` をそのまま出し、`\n` を重ねない。選択肢の前では、選択肢の規則（最後が字なら `\n[choice_newlines × 100]`、選択肢なら `\n`）で出し、保留には触らない（保留は、今までどおり次の字の直前で出る）。
- **Rationale**: 今の出力に区切りを足すだけになり、既に出ている区切りは場所も幅も変わらない。選択肢の無い出力では「最後が選択肢」にならないので、出力が変わらないことが状態の持ち方から言える。`spot_newlines` の説明も変えずに済む。
- **Trade-offs**: 選択肢を出した吹き出しへ、別の話し手の後で戻って台詞を出すと、要件 2.11 の文面（`\n`）ではなく `\n[spot_newlines × 100]` になる（設計ディスカッションで確定し、要件 2.11 に例外を書いた）。選択肢だけを出した吹き出しへ戻るときは、保留が立たないので `\n` になる。
- **採らなかった形**: 「最後に表示したもの」1 つに `spot_has_text` をまとめる形。状態は 1 つ減るが、今 `\n[150]` が出ている場所を保つには選択肢を字に数えることになり、`spot_newlines` の説明と既存の変数の意味が変わる。

### Decision: 話し手なしのグループの選択肢
- **Context**: `shiori_act_test.lua:751` の形（出力の先頭で、話し手なしのまま選択肢を出し、その後で話し手が話す）。話し手なしのグループはスポットが未解決（`nil`）で、後の話し手のスポット 0 と、状態機械の上では別の吹き出しになる。
- **Selected Approach**: 新しい状態（最後に表示したもの）については、未解決のスポットを 0 として数える。`shiori_act_test.lua:751` の期待値は `\s[5]\_w[500]\n\c\![*]\q[d,t]\![set,choicetimeout,30000]\p[0]\nA\e` になる（`\p[0]` と `A` の間に `\n` が入る）。
- **Rationale**: 選択肢行だけを書いたシーンは話し手なしのグループになる。ここで数えないと、選択肢どうしが同じ行に並び、Requirement 2 の 2 を満たせない。さくらスクリプトは、スコープを切り替えるタグが出るまでスコープ 0 の吹き出しに出る。
- **Trade-offs**: 既存の `spot_has_text` は、今までどおり未解決のスポットを数えない（数えると、選択肢の無い出力で `\n[150]` が増える場合があり、Requirement 3 の 5 に反する）。`clear_spot` を出力の途中で通った後の話し手なしの選択肢は、実際のスコープが 0 でないことがある。救済しない。

### Decision: 吹き出しの消去の数え方
- **Context**: Requirement 2 の 3 の「吹き出しを消した直後」。台詞の行の `\c` は `sakura_script` のトークンで、組み立ては消去として数えない（「設計フェーズの調査」の 4）。
- **Alternatives Considered**:
  1. 今の規則と同じく、`clear` のトークン（`act:clear()`）だけを数える。
  2. 字を表示しない `sakura_script` の中の `\c` を、新しい状態についてだけ数える。
  3. 段落の区切りの規則も含めて、台詞の行の `\c` を数える。
- **Selected Approach**: 3（2026-10-10 設計ディスカッション・開発者の判断）。下書きでは 1 を仮定にしていた（3 は選択肢の無い出力を変え、当時の Requirement 3 の 5 に反したため）。開発者の判断は「本 spec で直す。安易に起票に頼らない」である。要件に Requirement 7 を足し、Requirement 3 の 5 に例外を書いた。2 は 2 つの規則がずれるので採らない。
- **数える範囲**: 引数の無い `\c` だけ。`talk` と `sakura_script` のテキストを `APPEARANCE.tag_at` で読む。`\c[char,…]`・`\c[line,…]` は一部だけを消す（UKADOC）。`\x` はクリックの後にスコープも `\0` へ戻す（UKADOC「`\x`」「`\x[noclear]`」）。pasta は作者が書いたタグによるスコープの変化を追っていないので、`\x` は本 spec の外に置く。
- **確かめたこと**: DSL の文法（`grammar.pest` の `sakura_script`）は、タグ 1 つを 1 つの要素にする。台詞の行の `\c` は、テキストが `\c` だけの `sakura_script` のトークンになる。このリポジトリの辞書（`*.pasta`）に `\c` を書いたものは無い。`sakura_builder_test.lua`・`shiori_act_test.lua` の `\c` は、どれも `clear` のトークンである。
- **Trade-offs**: 選択肢を使わないゴーストでも、`\c` の後で `\n[spot_newlines × 100]` が出なくなる場所がある（`design.md` の「既存のゴーストでの見え方の変化」）。マニュアルの記述どおりになる変化である。

## 設計のまとめ（Synthesis・2026-10-10）

- **まとめられたもの**: Requirement 2 の 1・2・3・5 は、吹き出しごとの「最後に表示したもの」（無し・字・選択肢）という 1 つの状態の遷移として書ける。選択肢行と `act:choice` は同じトークンなので、組み立ての 1 か所で両方に効く。幅つきの改行のタグを作る式は、`spot_newlines` と `choice_newlines` で 1 つの局所関数を使う。
- **作らずに使うもの**: 設定の受け渡しは `spot_newlines` の経路をそのまま写す。既定値の検査は `config_defaults_test.rs` の既存の検査に 1 キー足す。新しいモジュール・依存・設定の仕組みは作らない。
- **削ったもの**: 選択肢のタグの形の変更（`\__q`）、作者の改行の検出、`\x` と引数つきの `\c[…]` の検出、`build` の分割、`.luacheckrc` の変更、組み立てからの設定の直接読み込み、9 段目の出力を照合するテストの追加（同じ並びは `pasta_shiori` の結合テストが見ている）。
- **持ち場の広がり**: 要件フェーズの見込み（`sakura_builder.lua`・`pasta/shiori/act.lua`・期待値 9 件・マニュアル 3 ページ）に、Rust の loader 3 ファイル、loader のテスト 4 ファイルとフィクスチャ 1 枚、マニュアルの内部のページ 4 つ、`book/AUTHORING.md` の 1 行が加わった。

## 規模とリスク

- **Effort**: S（1〜3 日）。既にある状態機械への枝の追加と、テスト・マニュアルの追従である。brief の見積もり（2〜4 タスク）と合う。設計で持ち場が広がったので（上の「設計のまとめ」）、タスクは 5〜7 になる見込みである。
- **Risk**: Low〜Medium。直す場所は 1 ファイルで、やり方は既存の改行の規則と同じ型である。一方で、選択肢行を使う既存のゴーストの見え方が変わること、実機でしか確かめられない点（行の配置・マーカーの位置）があること、同じファイルを `scene-anchor-link` も触ることが、Low に留まらない理由である。

## 設計フェーズへの申し送り

- **進め方**: Option A（原因は実機で確かめた）。
- **決めること**: 上の Design Decisions。要件の側は、要件ディスカッション（2026-10-10）ですべて決まった。
- **Research Needed**: 残っていない。実機のさくらスクリプト、2 つ目の選択肢が次の行に出る理由、`\n` を入れたときの見え方、既定のバルーンに収まるかは、2026-10-10 に確かめた（Research Log の「実機での確認」）。既存のゴーストへの影響は、要件の「決めたこと」に書いた。

## 設計ディスカッションへの申し送り

下書きの時点で仮定にしていた 5 つ。1・3・4・5 は、設計ディスカッションの最初に根拠を確かめて、仮定のとおりに確定した（`design.md` の「決定の記録」）。2 は、開発者の判断で「本 spec で直す」に変わった（上の Decision「吹き出しの消去の数え方」）。未決は残っていない。

1. 話し手の切り替えの区切りと重なるとき（`\n[spot_newlines × 100]` を残すか、要件の文面どおり `\n` にするか）。
2. 台詞の行に書いた `\c` を、吹き出しの消去として数えるか（Requirement 2 の 3）。
3. `pasta/shiori/act.lua` を触ること（ロードマップの「`act.lua` を触らない」の読み方）。
4. `choice_newlines` を Rust の既定値の補完に足して、持ち場を広げること。
5. 話し手なしの選択肢をスポット 0 として数えること（`shiori_act_test.lua:751` の期待値が変わる）。

## Risks & Mitigations
- 持ち場がロードマップの文面より広い — 同じウェーブの他の spec が触るファイルと重なるのは `book/AUTHORING.md` だけで、節が違う（`manual-print-media-refs` は「CI と同じ順」の一覧）。後から入る側が rebase する。広げるかどうかは設計ディスカッションで決める。
- 区切りの規則を取り違えて期待値を書く — 新しい期待値は `design.md` の「Supporting References」の表に書いてある。走らせた出力と表を比べる。
- 完全一致の期待値（9 件）の直し漏れ — `pasta_shiori` と `pasta_lua` のテストを両方走らせる。期待値は、新しい規則を手で当てて書く（出力を写すだけにしない）。
- `scene-anchor-link` との衝突（同じ `sakura_builder.lua`） — ロードマップの順番どおり、本 spec を先に入れる。アンカーは台詞の中の `＠？` で、本 spec の選択肢行とは別のトークンである。
- スキルの参照資料の再生成が止まる — `.claude/skills/` への書き込みは開発者の許可が要る。マニュアルを直すタスクで、再生成の前に許可を求める。
- 作者が自分で足した改行と重なって空の行ができる — 変更の記録に書く（Requirement 6 の 4）。知っている範囲で、該当するゴーストは無い。

## References
- [UKADOC さくらスクリプトリスト `\q[タイトル,ID]`](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html#_q_%E3%82%BF%E3%82%A4%E3%83%88%E3%83%AB%2CID_) — 自動改行は `\__q[]` を使うこと、という記述
- [UKADOC さくらスクリプトリスト `\__q[ID,...]`](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html#___q_ID%2C..._) — 自動改行をする選択肢の範囲
- [UKADOC さくらスクリプトリスト `\![*]`](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html#_!_*_) — マーカー
- `.kiro/specs/completed/sakura-script-newline/` — 話し手の切り替えの改行の状態機械（`spot_has_text`・`pending_break`）
- `.kiro/specs/completed/choice-definition-dsl/` — 選択肢行の元の要件（出力は `\![*]\q[…]` を行ごとに出す、までを定めた。行の配置は定めていない）
- `.kiro/steering/roadmap.md` — 同じウェーブの約束（本 spec が触るファイル）と、`scene-anchor-link`・`getting-started-screenshots` との順番
