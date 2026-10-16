# Requirements Document

## Project Description (Input)
辞書を書く人が、ゴーストが撫でられた・つつかれたときの反応を、決まった名前のシーン（`＊さわり反応女の子Head`・`＊つっつき反応その他` など）を書くだけで書けるようにする。今は、つつきは `＊OnMouseDoubleClick` で `＞transfer_req_to_var` を使って `＄ｒ４`（部位の名前）を読み、部位やキャラで台詞を分けるには Lua で振り分けを書くしかない。撫では pasta に判定が無く、大量に届く `OnMouseMove` から「撫でた」を決める仕組みを作者が Lua で自作している（emo2 は 72 行の `touch_detect.lua` を持ち、pasta-in-windows に複製している）。

pasta が `OnMouseDoubleClick`・`OnMouseMove` の既定の受け方を持ち、撫での判定（トーク中は反応しない）も pasta が持つ。候補の名前は時報と同じく段々に探す（アクター＋部位 → アクターその他 → 部位 → その他。各段の次に英字の名前 `OnPoke…`・`OnTouch…` を挟む）。最後は今までどおり `＊OnMouseDoubleClick`・`＊OnMouseMove` を呼ぶので、ショートカットを書いていないゴーストの挙動は変わらない。入門ガイドの 8・9 段目と hello-pasta の辞書を新しい書き方へ書き直す。詳細は同じフォルダの `brief.md`、emo2 の実装の控えは `research-emo2.md`。

## Introduction

辞書を書く人（ゴーストの作者）が、ゴーストが撫でられた・つつかれたときの反応を、決まった名前のシーンを書くだけで書けるようにする。作者が書きたいのは「女の子の頭を撫でられたらこの台詞」であり、イベントの名前・Reference の番号・Lua の振り分けは、作者が知りたいことではない。

pasta は、つつき（`OnMouseDoubleClick`）と撫で（`OnMouseMove` の流れ）に既定の受け方を持つ。候補の名前は、時報（`時報HH → OnHourHH → 時報その他 → OnHourOther`）と同じ形の段々で探す。撫でについては、「撫でた」と決める判定とトーク中の抑止も pasta が持つ。最後の段は今までどおり `＊OnMouseDoubleClick`・`＊OnMouseMove` なので、ショートカットのシーンを 1 つも書いていないゴーストの挙動は変わらない。ゴーストが自前の Lua で受け方を登録しているときは、それが必ず優先する。

この spec は、見本のゴースト hello-pasta の 8・9 段目の辞書と、入門ガイドの該当の章・リファレンスを、新しい書き方へ書き直すところまでを含む。

### 用語

- **つつき**: シェルの上でのダブルクリック（`OnMouseDoubleClick`）。
- **撫で**: シェルの同じ当たり判定の上で、マウスを動かし続けること（`OnMouseMove` の流れ）。
- **部位**: 当たり判定の識別子（Reference4）。シェルが決める名前で、英字（`Head`）も日本語（`頭`）も来る。
- **スコープ番号**: 触られたキャラの番号（Reference3。本体 0・相方 1・SSP では 2 以降もある）。
- **アクター名**: `pasta.toml` の `[actor."名前"]` の名前。
- **ショートカットのシーン**: 本書が定める名前（`さわり反応…`・`OnTouch…`・`つっつき反応…`・`OnPoke…`）を持つグローバルシーン。
- **自前のハンドラ**: ゴーストが自分の Lua で `REG.OnMouseDoubleClick`・`REG.OnMouseMove` に登録した関数。
- **抑止の状態**: リクエストの Status ヘッダーが、ゴーストがしゃべっている途中などを示している状態（要件 5 で定める）。

## Boundary Context

- **In scope**:
  - つつき・撫での既定の受け方と、候補の名前を段々に探すこと。
  - ショートカットの段の名前を完全一致で探すこと（そのためにシーンの検索へ手を入れることを含む。作り方は設計で決める）。
  - 撫での判定（いつ「撫でた」と決めるか・いつ数え直すか）と、トーク中の抑止。
  - スコープ番号からアクター名を引くこと。
  - hello-pasta の 8・9 段目の辞書と、それを検査するテスト・段階表・辞書の一覧。
  - マニュアル（入門の 8・9 段目、SHIORI イベントのリファレンス、`OnMouseDoubleClick` を例にしている文法の章からの案内）と、スキル references の再生成。
- **Out of scope**:
  - `OnMouseClick`（シングルクリック）・`OnMouseWheel`・押しっぱなし（里々の「ころころ」「ホールド」に当たるもの）。
  - エレメントのクリック（`shell-element-click`）と、選択肢の振り分け（`OnChoiceSelectEx`）。
  - DSL の文法・トランスパイラの変更。
  - シェルの当たり判定の定義（hello-pasta の部位は `Head`・`Face`・`Body` のまま）。
  - emo2・pasta-in-windows の辞書の載せ替え（ghost_dev の側で行う）。
  - 完全一致の場面選びを設定で切り替える一般の仕組み（`novel-talk-flow`）。
  - 失敗の見せ方の統一（`failure-output-unification`）。飛び先が無いときは今までどおり 204 を返す。
  - 撫でのしきい値などを `pasta.toml` で変える設定。
- **Adjacent expectations**:
  - ベースウェア（SSP）は、`OnMouseMove` と `OnMouseDoubleClick` に、スコープ番号（Reference3）・部位（Reference4）・Status ヘッダーを付けて送る。areka で同じものが届くかは、この spec では確かめない。
  - 後から入る `pasta-check-dic-validate` が、接頭辞だけの `＊さわり反応`・`＊つっつき反応`（呼ばれないシーン）を見つける検査を足す。この spec は検査を足さない。

## Requirements

### Requirement 1: つつきの反応を名前で探す

**Objective:** As a ゴーストの作者, I want つつかれた部位とキャラに応じたシーンを名前だけで書けること, so that Lua も Reference の番号も知らずに、部位ごと・キャラごとのつつきの反応を書ける

#### Acceptance Criteria

1. When つつきのリクエストを受けたとき, the pasta runtime shall 次の順に候補の名前でシーンを探し、最初に見つかったシーンを実行してその出力を返す: (1) `つっつき反応<アクター名><部位>`、(2) `OnPoke<アクター名><部位>`、(3) `つっつき反応<アクター名>その他`、(4) `OnPoke<アクター名>Other`、(5) `つっつき反応<部位>`、(6) `OnPoke<部位>`、(7) `つっつき反応その他`、(8) `OnPokeOther`、(9) `OnMouseDoubleClick`。
2. When 見つかった段に同じ名前のシーンが複数あるとき, the pasta runtime shall その中から今までの同名シーンと同じ決まりで 1 つを選ぶ。
3. If 9 段のどれにもシーンが見つからないとき, then the pasta runtime shall 204 を返す。
4. If 部位が空（当たり判定の外）のつつきを受けたとき, then the pasta runtime shall 部位を含む段（1・2・5・6）を飛ばして探す（アクター名が決まれば、そのアクターのその他の段で本人が反応できる）。
5. If スコープ番号からアクター名が決まらないとき, then the pasta runtime shall アクター名を含む段（1〜4）を飛ばして探す。
6. The pasta runtime shall 接頭辞だけの名前（`つっつき反応`・`OnPoke`）では探さない。
7. The pasta runtime shall つつきの反応を、抑止の状態でも探して返す（つつきは利用者がはっきり操作したものなので、しゃべっている途中のトークを打ち切ってよい。今の `＊OnMouseDoubleClick` と同じ）。
8. The pasta runtime shall 左ボタンのダブルクリックと右ボタンのダブルクリックを区別せず、どちらもつつきとして扱う。タッチパネル・ペンによる操作も区別しない。

### Requirement 2: 撫でを決める

**Objective:** As a ゴーストの作者, I want pasta が「撫でた」を決めてくれること, so that 大量に届くマウスの移動から撫でを判定する仕組みを自作しなくてよい

#### Acceptance Criteria

1. When 同じキャラの同じ部位の上でマウスの移動が、途切れ（2 秒より長い間隔）なく 2 秒続いたとき, the pasta runtime shall それを 1 回の「撫でた」と決める。
2. While 一度「撫でた」と決めた後、同じキャラの同じ部位を撫で続けている間, the pasta runtime shall 数え直すまで次の「撫でた」を決めない。
3. When 部位かキャラが変わったとき, the pasta runtime shall 撫での時間を数え直す。
4. When マウスの移動が 2 秒より長く途切れた後に届いたとき, the pasta runtime shall 撫での時間を数え直す。
5. When 部位が空（当たり判定の外）のマウスの移動を受けたとき, the pasta runtime shall 撫での時間を数え直す（当たり判定の外へ出て 2 秒以内に同じ部位へ戻っても、発火済みのまま残らない）。
6. The pasta runtime shall 撫でた時間を 1 秒より細かい時刻で測り、しきい値を過ぎた後の最初のマウスの移動で「撫でた」と決める（秒単位の時計で 2〜3 秒にぶれる emo2 のはまりどころを持ち込まない）。
7. The pasta runtime shall 撫での判定の状態を 1 つだけ持ち（カーソルは 1 つなので、キャラを行き来すれば数え直す）、ゴーストを読み直したときに失ってよい（保存しない）。
8. The pasta runtime shall しきい値と途切れの秒数を設定で変えられるようにしない（既定の値に固定する）。

### Requirement 3: 撫での反応を名前で探す

**Objective:** As a ゴーストの作者, I want 撫でられた部位とキャラに応じたシーンを、つつきと同じ形の名前で書けること, so that 撫でとつつきを同じ覚え方で書ける

#### Acceptance Criteria

1. When 「撫でた」と決めたとき, the pasta runtime shall 次の順に候補の名前でシーンを探し、最初に見つかったシーンを実行してその出力を返す: (1) `さわり反応<アクター名><部位>`、(2) `OnTouch<アクター名><部位>`、(3) `さわり反応<アクター名>その他`、(4) `OnTouch<アクター名>Other`、(5) `さわり反応<部位>`、(6) `OnTouch<部位>`、(7) `さわり反応その他`、(8) `OnTouchOther`、(9) `OnMouseMove`。
2. When 見つかった段に同じ名前のシーンが複数あるとき, the pasta runtime shall その中から今までの同名シーンと同じ決まりで 1 つを選ぶ。
3. When 「撫でた」と決めなかったマウスの移動を受けたとき, the pasta runtime shall 今までどおり `OnMouseMove` の名前でだけシーンを探し、見つからなければ 204 を返す。
4. If スコープ番号からアクター名が決まらないとき, then the pasta runtime shall アクター名を含む段（1〜4）を飛ばして探す。
5. The pasta runtime shall 接頭辞だけの名前（`さわり反応`・`OnTouch`）では探さない。
6. The pasta runtime shall 部位が空のマウスの移動では「撫でた」と決めない（当たり判定の外では撫での反応を出さない）。

### Requirement 4: アクター名の決め方

**Objective:** As a ゴーストの作者, I want 触られたキャラを番号でなく `pasta.toml` のアクター名で書けること, so that 台詞の名前と同じ名前でシーンを書ける

#### Acceptance Criteria

1. When スコープ番号を受けたとき, the pasta runtime shall `pasta.toml` の `[actor."名前"]` の `spot` がその番号に等しいアクターが 1 人だけなら、その名前をアクター名とする（読み込み時の設定の値で引き、シーンの `％` 行でスポットが入れ替わっても変えない）。
2. If `spot` がその番号に等しいアクターが 1 人もいないとき, then the pasta runtime shall アクター名が決まらないものとして扱う。
3. If `spot` がその番号に等しいアクターが 2 人以上いるとき（スポットを共有して交代で話すアクター）, then the pasta runtime shall そのスポットで最後に話したアクターの名前をアクター名とし、まだ誰も話していなければアクター名が決まらないものとして扱う。
4. The pasta runtime shall スコープ番号そのものを名前に入れた段（`つっつき反応1Head` など）を探さない。
5. The pasta runtime shall アクターだけの段の名前を、日本語は `<接頭辞><アクター名>その他`、英字は `<接頭辞><アクター名>Other` とする（`つっつき反応男の子その他`・`OnTouch男の子Other`）。

### Requirement 5: トーク中は撫でに反応しない

**Objective:** As a ゴーストの利用者, I want ゴーストがしゃべっている途中にマウスを動かしても話が打ち切られないこと, so that トークを最後まで聞ける

#### Acceptance Criteria

1. While リクエストの Status ヘッダーが抑止の状態を示している間, the pasta runtime shall 「撫でた」と決めない。
2. While 抑止の状態の間, the pasta runtime shall 届いたマウスの移動を「撫でていない」と数える（抑止が解けてから、改めてしきい値の時間を撫で続けたときに「撫でた」と決める。撫での反応そのものもトークなので、同じ所を撫で続けたときの 2 回目は、反応の再生が終わった後に出る）。
3. The pasta runtime shall 抑止の状態を、Status ヘッダーに仮想イベント（`OnTalk`・`OnHour`）の抑止と同じ語（`talking`・`choosing`・`online`・`opening`・`passive`・`induction`・`timecritical`・`nouserbreak`・`minimizing`）のどれかが含まれていることとする。
4. While トークが終わってバルーンが残っているだけの間（Status ヘッダーが `balloon(…)` だけを示している間）, the pasta runtime shall 撫でを抑止しない。
5. While 抑止の状態の間, the pasta runtime shall `OnMouseMove` の名前のシーンは今までどおり探す（抑止するのは撫での判定と撫でのショートカットだけである）。

### Requirement 6: 部位の名前の一致の仕方

**Objective:** As a ゴーストの作者, I want 部位の名前が似ていても、別の部位のシーンが誤って選ばれないこと, so that `胸` と `胸右` のような頭が同じ名前の部位を持つシェルでも、書いたとおりに反応する

#### Acceptance Criteria

1. The pasta runtime shall 部位の名前を、シェルが送ってきたとおりに（英字も日本語も変えずに）候補の名前へ入れる。
2. When ショートカットの段（1〜8）で候補の名前を探すとき, the pasta runtime shall その名前と完全に同じ名前のシーンだけを候補にする（`つっつき反応胸` で探して `＊つっつき反応胸右` を選ばない。部位 `女` で探して `＊つっつき反応女の子Head` を選ばない）。
3. The pasta runtime shall 9 段目（`OnMouseDoubleClick`・`OnMouseMove`）の探し方を、今までのイベントと同じ名前のシーンの探し方のまま変えない。
4. Where 部位の名前が `その他`・`Other` であるシェルでは, the pasta runtime shall 部位を含む段とその他の段を、名前の重なりのまま上から順に探す（特別扱いしない）。この決まりはマニュアルに書く。

### Requirement 7: 反応の中身と、反応しない場所

**Objective:** As a ゴーストの作者, I want 特定の部位では何も反応させないことや、触られた部位とキャラを台詞に入れることができること, so that 「閉じる」ボタンのような身体でない当たり判定でその他の反応が鳴らないようにでき、台詞も書ける

#### Acceptance Criteria

1. When 見つかったショートカットのシーンが何も出力しなかったとき, the pasta runtime shall 204 を返し、それより下の段を探さない（部位つきの空のシーンを書けば、その部位では反応しない）。
2. When ショートカットのシーンを実行するとき, the pasta runtime shall シーンの中で今までどおり `＞transfer_req_to_var` で Reference（スコープ番号・部位）を読めるようにする。
3. The pasta runtime shall この spec では、触られた部位とアクター名を `＞transfer_req_to_var` なしで読める新しい変数を足さない。
4. When ショートカットのシーンの反応を返すとき, the pasta runtime shall 続きを待っているトーク（チェイントーク）を、今のイベントと同じ名前のシーンが反応したときと同じに扱う（扱いを変えない）。

### Requirement 8: 今までのゴーストを変えない

**Objective:** As a 既存のゴーストの作者, I want ショートカットのシーンを書いていないゴーストや、自前の Lua でつつき・撫でを受けているゴーストがそのまま動くこと, so that emo2 のメニューや自前の撫での判定を書き直さずに済む

#### Acceptance Criteria

1. While ゴーストの辞書にショートカットのシーンが 1 つも無い間, the pasta runtime shall つつきでは今までどおり `＊OnMouseDoubleClick` を、マウスの移動では毎回今までどおり `＊OnMouseMove` を探して実行する。
2. Where ゴーストが自前のハンドラを登録しているとき, the pasta runtime shall 登録が pasta の既定の受け方より先でも後でも、そのハンドラを呼ぶ（既定の受け方がゴーストの登録を上書きしない）。
3. Where ゴーストが自前のハンドラを登録しているとき, the pasta runtime shall そのイベントでは既定の受け方を一切動かさない（ショートカットのシーンを探さず、撫での判定もしない）。
4. The pasta runtime shall マウスの移動 1 件ごとに、既定のログの水準でログを出さない。
5. The pasta runtime shall 「撫でた」と決めなかったマウスの移動では、`OnMouseMove` の名前で探す 1 回のほかにシーンを探さない（大量に届くマウスの移動の処理を、今より目立って重くしない）。
6. When pasta のテストを走らせたとき, the pasta tests shall 自前のハンドラを既定の受け方より先に登録した場合と後に登録した場合の両方で、自前のハンドラが呼ばれ、既定の受け方が動かないことを確かめる。
7. The pasta runtime shall 既存の既定ハンドラ（`OnBoot`・`OnChoiceSelectEx`・`OnSecondChange`）も 8.2 と同じく、ゴーストが先に登録したハンドラを上書きしない。

### Requirement 9: 見本のゴーストの 8・9 段目

**Objective:** As a 入門者, I want 見本のゴーストの 8 段目が新しい書き方で撫で・つつきの反応を見せること, so that 入門ガイドどおりに書けば部位ごと・キャラごとの反応を作れる

#### Acceptance Criteria

1. The hello-pasta shall 8 段目の辞書で、つつきの反応をショートカットのシーン（アクター＋部位・アクターその他・部位だけ・その他の各段を少なくとも 1 つずつ）で書く。
2. The hello-pasta shall 8 段目の辞書で、撫での反応をショートカットのシーンで書く。
3. The hello-pasta shall 8 段目の辞書で、`＊OnMouseDoubleClick`・`＞transfer_req_to_var`・`＄ｒ４` を使わない。
4. The hello-pasta shall 9 段目の選択肢を出すシーンを、8 段目と同じショートカットの名前に入れ、8 段目の反応と一緒に抽選されるようにする（当たり判定の上のつつきで出会える名前にする）。
5. The hello-pasta shall 段階表（`STAGES.md`）の 8・9 段目の行と Reference の表、および辞書の一覧を、新しい辞書に合わせる。
6. When 見本のゴーストのテストを走らせたとき, the hello-pasta tests shall 8 段目の各段（アクター＋部位・アクターその他・部位だけ・その他）のつつき、8 段目の撫で、9 段目の選択肢が、それぞれ期待どおりに選ばれることを確かめる。

### Requirement 10: マニュアル

**Objective:** As a ゴーストの作者, I want マニュアルが新しい書き方と決まりを説明していること, so that 名前の並び・探す順・撫での判定・抑止を、コードを読まずに知れる

#### Acceptance Criteria

1. The pasta manual shall 入門の 8 段目の章で、撫で・つつきの反応をショートカットのシーンで書く作例を、見本のゴーストの辞書と逐語で一致させて載せる。
2. The pasta manual shall 入門の 9 段目の章の作例と説明を、9 段目の辞書の変更に合わせる。
3. The pasta manual shall SHIORI イベントのリファレンスに、つつき・撫での探す順（日本語の名前と英字の名前の 9 段）、アクター名の決め方、撫での判定（しきい値・数え直し・1 回だけ）、トーク中の抑止、部位の名前の完全一致、`その他`・`Other` という名前の部位の扱い、接頭辞だけの名前が呼ばれないこと、自前のハンドラが優先することを書く。
4. The pasta manual shall `OnMouseDoubleClick` を例にしている文法の章から、新しい書き方へ案内する。
5. When マニュアルを変えたとき, the pasta project shall 同じ変更でスキル references を再生成する。

## 決定事項（2026-10-16 の要件ディスカッション）

開発者が全議題を推奨案で承認した。あわせて「Lua で受け方を組んでいるゴースト（emo2 など）を壊さない。イベント関数がすでに登録されている場合に注意する」との指示があり、要件 8.2・8.3・8.6 に入れた。

1. **アクター名**: 読み込み時の `spot` で引く。共有のスポットは最後に話したアクター。番号の段は足さない（要件 4）。
2. **部位の名前**: ショートカットの段は完全一致。同名シーンの抽選は今までと同じ決まり。brief の「Rust の検索は触らない」は外す（作り方は設計で C-1・C-2 を比べる）（要件 6）。
3. **アクターだけの段**: 足す。順は「アクター＋部位 → アクターその他 → 部位 → その他」（要件 1・3・4.5）。
4. **部位が空**: つつきは部位の段を飛ばす。撫では反応せず数え直す（要件 1.4・2.5・3.6）。
5. **撫での判定**: 時間で決める（2 秒・途切れ 2 秒）・1 秒より細かい時刻で測る（設計ディスカッションで文言を直した）・外へ出たら数え直す・状態 1 つ・設定なし（要件 2）。
6. **抑止**: 抑止の間は「撫でていない」と数える・9 語・つつきには効かせない（要件 1.7・5）。
7. **身体でない当たり判定**: 部位つきの空のシーンで受ける（要件 7.1）。
8. **新しい変数**: 足さない（要件 7.3）。
9. **8・9 段目**: 8 段目で `＊OnMouseDoubleClick`・`＞transfer_req_to_var`・`＄ｒ４` を教えない。9 段目は 8 段目と同じショートカットの名前で一緒に抽選（名前は設計で決める）（要件 9）。
10. **右ボタン**: 区別しない（要件 1.8）。
11. **`その他`・`Other` という部位**: 特別扱いせず書くだけ（要件 6.4）。
12. **チェイントーク**: 今までと同じ扱い（要件 7.4）。
