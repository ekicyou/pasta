# Requirements Document

## Project Description (Input)

1 つのイベントに、いくつもの処理を足したいゴーストがある。窓際のぱすたさん（ghost_dev）は、`REG.OnSecondChange` を 3 か所（`dic/pose.pasta`・`dic/tategaki.pasta`・`dic/boot.pasta`）で包み合っている。包み合いには決まり文句（前のハンドラを控える・呼び忘れない・`nil` のときの扱い）が要り、どれが外側になるかはファイルを読む順で決まる。作者自身が「読み込みの順に頼っていて壊れやすい」と書いた。呼び忘れると、組み込みの既定の OnSecondChange（コールバックの期限切れの掃除・時報・ランダムトーク）が黙って止まる。

今の `REG` はただの表で、1 つのイベントに 1 つのハンドラしか置けない。重ねて登録する口は、どこにも無い。

そこで、`REG:register(id, fn)` で 1 つのイベントにハンドラを重ねて登録できるようにする。新しく登録したものから呼び、`nil`・`false` を返せば次（先に登録したもの）へ回り、`true` ならそこで止めて 204、文字列・コルーチンならそこで止めて今の決まりどおりの応答にする。登録したものが全部見送ったら、今どおり `REG[id]`（組み込みの既定か作者の代入）を呼び、それも無ければイベントと同じ名前のシーンを探す。`register` は外すための関数を返す。すべてのイベントと、`On` で始まらないリソースの問い合わせに効く。詳しい経緯と決定（2026-10-22 開発者の承認）は `brief.md` を参照。

## Introduction

この spec は、SHIORI のイベントハンドラの登録表 `REG` に、1 つのイベントのハンドラを重ねて登録する口を足し、イベントの振り分けがその重なり（以下「鎖」）を新しい順に呼ぶようにする。

```lua
-- 処理を足す（何も返さなければ、次へ回る）
REG:register("OnSecondChange", function(act)
    if 静かな秒が続いた then
        return [=[\0\s[0]\e]=]   -- 文字列を返せば、ここで止まって 200
    end
end)

-- 一時的な門（true を返せば、ここで止まって 204）
local off = REG:register("OnSecondChange", function(act)
    if SAVE["初回待ち"] then return true end
end)
-- 演出が終わったら外す
off()
```

1 つのイベントが届いたときの呼び順は次のとおりである。

```text
イベント到着
  ↓（待っているコールバック・時計の返事は今どおり先に処理する）
鎖を新しく登録した順に呼ぶ
  ├─ 真の値を返した → そこで止めて応答にする
  └─ 全部が見送った（nil・false）
        ↓
     REG[id]（組み込みの既定か作者の代入）があれば呼ぶ
        ↓ 無ければ
     イベントと同じ名前のシーンを探す → 無ければ 204
```

主な利用者は、Lua（`scripts/` の `.lua` や辞書の Lua ブロック）でイベントハンドラを書くゴーストの作者である。

## Boundary Context

- **In scope**
  - `REG:register(id, fn)` による重ねた登録と、それが返す外す関数
  - イベントの振り分けの引き方（鎖 → `REG[id]` → イベントと同じ名前のシーン）と、鎖のハンドラの返り値の読み方（偽なら次へ・`true` は 204・文字列とコルーチンは今の決まり）
  - `On` で始まらないリソースの ID を含む、すべての ID への適用
  - 鎖の中のハンドラがエラーで止まったときの扱い（今の止まるエラーの表記に載せる）
  - テストが鎖をまっさらに戻せること、`REG` の表を空にしている既存の 3 つのテストの合わせ
  - Lua の自動テスト（順番・止まり方・`true` の 204・外す・底の既定と代入・`register` という ID・リソースの ID・コルーチンを返すハンドラ・エラー）
  - マニュアル（`lua/shiori-events.md` の REG の節と処理の順の説明・フォールバックの図、内部の説明 `internals/execution-model.md`・`internals/shiori.md`）と、生成のスキルの references の作り直し
- **Out of scope**
  - 優先度の数での並べ替え（順番は登録の新しい順だけ）
  - 応答が出た回にも全部のハンドラに見せるだけの「見張り役」の口（新しいハンドラが応答した回は、古いハンドラは呼ばれない。包み合いの今と同じ）
  - 窓際のぱすたさん（ghost_dev）の辞書の書き換え（ghost_dev の側の作業。マニュアルの書き換えの例を手がかりにする）
  - DSL の文法への追加（これは Lua の API で、DSL には足さない）
  - 組み込みの既定のハンドラの置き方（`REG.X = REG.X or …`）の変更
  - 待っているコールバックへの振り分けと、時計の合図と返事を見る処理の順（鎖より前）の変更
  - 自動トークを止める口（`ontalk-suppression` が持つ）
- **Adjacent expectations**
  - **自動トークの停止**（完了 spec `ontalk-suppression`）: `＞自動トーク停止` は既定の OnSecondChange の中の自動トークの発行だけを止め、鎖に重ねた作者のハンドラは止めない。鎖の底の既定のハンドラは、止めている間も期限切れの掃除と仮想時刻の読み取りを続ける。この spec はその関係を変えない。
  - **チェイントークの続き**（`chain-talk-interrupt`、同じウェーブ）: 鎖のハンドラが返したコルーチンは、今の `REG` のハンドラが返したコルーチンと同じ道で再開する。続きの置き換えの決まりは、あちらが決める。
  - **外から呼ぶイベント**（`external-control-events`、同じウェーブ）: あちらが足す既定のハンドラは、ほかの既定のハンドラと同じく鎖の底に置かれる。
  - **おすすめメニュー**（`recommend-menu-scenes`、版は未定）: あちらの「Lua の `REG` が勝つ」は、鎖でも保たれる。鎖のハンドラが見送れば、鎖の底の組み込みの受け手へ回る。この順を書くのはあちらの要件である。
  - **止まるエラーの表記**（完了 spec `runtime-error-balloon` ほか）: 表記の形と、出す・出さないの設定に従う。

## Requirements

### Requirement 1: ハンドラを重ねて登録する

**Objective:** As a ゴーストの作者, I want 前のハンドラを控えずに、1 つのイベントへ処理を足すこと, so that 包み合いの決まり文句と、読み込みの順への依存をなくせる。

#### Acceptance Criteria

1. When 作者が `REG:register(id, fn)` を呼んだとき, the pasta shall `fn` をイベント `id` の鎖に加え、その登録を外すための関数を返す。
2. The pasta shall 1 つのイベントに、いくつでもハンドラを重ねて登録できるようにする。
3. When 作者が同じ関数を同じイベントに 2 回登録したとき, the pasta shall それぞれを別の登録として扱い、どちらも呼ぶ（それぞれの外す関数は、自分の登録だけを外す）。
4. The pasta shall `scripts/` の Lua・辞書の Lua ブロック・シーンの中の Lua のどこからでも、また読み込みの後のいつでも、登録を受け付ける。
5. The pasta shall 鎖の登録を `REG` の表のキーとして見せない（`pairs(REG)` で回るのは、今どおりイベントのハンドラだけにする）。
6. The pasta shall `REG.X = 関数` の代入と `REG.X` の読み出しを、今と同じに働かせる（鎖の登録は `REG.X` の値を変えない。ただし `REG.register` を読むと登録の口が返る）。
7. If `id` が文字列でないか、`fn` が関数でないとき（`REG.register(id, fn)` のように `:` を付けずに呼んだときを含む）, the pasta shall 登録せず、何が違うかを示すエラーにする。

### Requirement 2: 鎖を呼ぶ順と止まり方

**Objective:** As a ゴーストの作者, I want 後から登録した処理が先に効き、返り値で次へ回すか止めるかを決められること, so that シーンの中で後から登録した門が、ほかの処理より先に効く。

#### Acceptance Criteria

1. When 鎖にハンドラのあるイベントが届いたとき, the pasta shall 鎖のハンドラを、新しく登録したものから順に呼ぶ。
2. When 鎖のハンドラが `nil` を返したか、何も返さなかったか、`false` を返したとき, the pasta shall 次（その 1 つ前に登録したもの）へ回る。
3. When 鎖のハンドラが `true` を返したとき, the pasta shall そこで止め、204 No Content を返す。
4. When 鎖のハンドラが文字列を返したとき, the pasta shall そこで止め、今の `REG` のハンドラの戻り値と同じ決まりで応答にする（`SHIORI/` で始まれば応答全体としてそのまま、それ以外は `Value` の 200、空文字列なら 204）。
5. When 鎖のハンドラがシーンのコルーチンを返したとき, the pasta shall そこで止め、今の `REG` のハンドラが返したコルーチンと同じく再開して、出力があれば `Value` の 200、無ければ 204 を返す。
6. When 鎖のハンドラが、`true`・文字列・コルーチンのどれでもない真の値（数・表など）を返したとき, the pasta shall Lua の真偽のとおりにそこで止め、204 No Content を返す（空文字列 `""` も真なので、2.4 のとおり止めて 204 にし、次へは回らない）。
7. When 鎖のハンドラがそこで止めたとき, the pasta shall それより前に登録した鎖のハンドラ・`REG[id]`・イベントと同じ名前のシーンを、その回には呼ばない。
8. When 待っているコールバックへの応答か、時計の返事（`OnPastaClock`）を受け取った回, the pasta shall 今どおりそちらを先に処理し、鎖を呼ばない。
9. When 鎖のハンドラが、前の鎖のハンドラが見送った後に呼ばれたとき, the pasta shall 見送ったハンドラが積んだ台詞を、後のハンドラ・`REG[id]`・シーンの出力に混ぜない（鎖のハンドラ・`REG[id]`・シーンには、それぞれ新しい act を渡す）。

### Requirement 3: 鎖の底（REG[id] とシーン）

**Objective:** As a ゴーストの作者, I want 鎖を足しても、組み込みの既定のハンドラと今の書き方が今どおり動くこと, so that 既定の OnSecondChange が黙って止まらず、今のゴーストを書き換えずに済む。

#### Acceptance Criteria

1. When 鎖のハンドラが全部見送ったとき, the pasta shall `REG[id]`（組み込みの既定のハンドラか、作者が代入したハンドラ）があればそれを呼び、その戻り値を今と同じ決まりで応答にする。
2. When 鎖のハンドラが全部見送り、`REG[id]` も無いとき, the pasta shall 今どおりイベントと同じ名前のシーンを探して実行し、見つからなければ 204 No Content を返す。
3. While あるイベントの鎖が空のとき, the pasta shall そのイベントに、鎖を足す前と同じ応答を返す。
4. When 作者が `REG.X = 関数` を代入したとき, the pasta shall 鎖の底だけを差し替え、鎖に登録したハンドラはそのまま残す。
5. The pasta shall 今のゴーストの包み合い（`local prev = REG.X` で前のハンドラを控え、`REG.X` に自分の関数を代入して中で `prev(act)` を呼ぶ書き方）を、書き換えずに今どおり動かす。
6. While 作者が鎖に OnSecondChange のハンドラを登録しているとき, the pasta shall 鎖のハンドラが全部見送った秒には、既定の OnSecondChange（コールバックの期限切れの掃除・仮想時刻の読み取り・自動トークの発行）を今どおり働かせる。

### Requirement 4: 登録を外す

**Objective:** As a ゴーストの作者, I want 登録したハンドラを後から外せること, so that 初回の演出の間だけ毎秒のイベントを止める門のような、一時的な処理を置ける。

#### Acceptance Criteria

1. When 作者が `REG:register` の返した関数を呼んだとき, the pasta shall その登録を鎖から外し、次のイベントからそのハンドラを呼ばない。
2. When 登録を外したとき, the pasta shall ほかの登録と、その呼ぶ順を変えない。
3. When 外す関数が 2 回目以降に呼ばれたとき, the pasta shall 何もせず、エラーにもしない。
4. When あるイベントの鎖の登録を全部外したとき, the pasta shall そのイベントに、鎖を足す前と同じ応答を返す。
5. When 鎖を呼んでいる途中のハンドラの中で、登録や外す関数の呼び出しがあったとき, the pasta shall その回の呼ぶ順を変えず（まだ呼ばれていないハンドラを飛ばさず、新しく登録したハンドラはその回には呼ばず）、次のイベントから反映する。

### Requirement 5: すべての ID に効く

**Objective:** As a ゴーストの作者, I want イベントでもリソースの問い合わせでも、同じ書き方で重ねられること, so that `sakura.recommendsites` のような ID にも処理を足せる。

#### Acceptance Criteria

1. The pasta shall `On` で始まるイベントにも、`On` で始まらないリソースの問い合わせの ID（`sakura.recommendsites` など）にも、同じ鎖の決まりを当てる。
2. When `register` という ID の要求が届いたとき, the pasta shall 登録の口そのものを呼ばず、ほかの ID と同じく鎖 → `REG[id]` → イベントと同じ名前のシーンの順で扱う（どれも無ければ 204 No Content）。

### Requirement 6: 鎖の中のエラー

**Objective:** As a ゴーストの作者, I want 鎖のハンドラがエラーで止まったとき、どのハンドラで止まったかが分かること, so that 重ねた処理のどこを直せばよいかがすぐ分かる。

#### Acceptance Criteria

1. If 鎖のハンドラがエラーで止まったとき, the pasta shall そこで止め、今の `REG` のハンドラがエラーで止まったときと同じ応答（既定では止まった理由の表記の 200、表記を出さない設定では 500）を返す。
2. If 鎖のハンドラがエラーで止まったとき, the pasta shall それより前に登録した鎖のハンドラ・`REG[id]`・イベントと同じ名前のシーンを、その回には呼ばない。
3. If 鎖のハンドラがエラーで止まったとき, the pasta shall 止まった場所の表記を、今の `REG` のハンドラが止まったときと同じ規則で出し、その場所は鎖のほかのハンドラでなく、止まったハンドラ自身を指す（`scripts/` の `.lua` に書いたハンドラなら、そのファイルと行）。
4. If 鎖のハンドラが返したコルーチンがエラーで止まったとき, the pasta shall 今の `REG` のハンドラが返したコルーチンがエラーで止まったときと同じに扱う。
5. When 鎖のハンドラがエラーで止まった回の後, the pasta shall 次のイベントで、鎖のハンドラを今までどおり呼ぶ（エラーで登録を外さない）。

### Requirement 7: テストのための初期化

**Objective:** As a pasta の開発者, I want テストの前に、鎖の登録をまっさらに戻せること, so that テストどうしが登録を持ち越さず、`REG` の表を空にしている既存のテストもそのまま通る。

#### Acceptance Criteria

1. The pasta shall テストが、`REG` の表の中身とは別に、すべてのイベントの鎖を空に戻せるようにする（専用の口は作らず、既存のテストと同じく `pasta.shiori.event.register` を `package.loaded` から外して読み直せば鎖も空になる形にする）。
2. When `REG` の表を空にしている既存の 3 つのテスト（`event_clock_hook_test`・`event_coroutine_test`・`second_change_thread_test`）を走らせたとき, the pasta shall 鎖を足す前と同じ結果にする。
3. The pasta shall Requirement 1〜6 の振る舞い（順番・止まり方・`true` の 204・外す・底の既定と代入・`register` という ID・リソースの ID・コルーチンを返すハンドラ・エラー）を、Lua の自動テストで確かめ、Lua のテストの一覧に載せる。

### Requirement 8: マニュアルとスキルの references

**Objective:** As a ゴーストの作者, I want マニュアルで鎖の書き方と、包み合いからの書き換え方を読めること, so that 今の包み合いを、読み込みの順に頼らない書き方へ移せる。

#### Acceptance Criteria

1. The pasta shall マニュアルの `lua/shiori-events.md` の REG の節に、`REG:register` の書き方・返り値の読み方の表（偽なら次へ・`true` は 204・文字列とコルーチンは今の決まり・空文字列 `""` も止まる）・新しい順に呼ぶこと・外す関数・鎖の底（`REG[id]` → シーン）を、作例付きで書く。
2. The pasta shall 同じ節に、包み合いから鎖への書き換えの例（毎秒の処理を足す例と、`true` を返す一時的な門を外す例）を書く。
3. The pasta shall 「1 つのイベントに登録できるハンドラは 1 つである」という今の記述を、鎖を前提にした記述に改め、ページ冒頭の処理の順とシーン関数フォールバックの図にも鎖を加える。
4. The pasta shall 鎖に「見張り役」は無いこと（新しいハンドラが応答した回は、古いハンドラが呼ばれないこと）を書く。
5. The pasta shall `＞自動トーク停止` と鎖の門の使い分け（ランダムトークと時報を止めるだけなら `＞自動トーク停止`、表情や窓あわせのような毎秒の処理まで止めるなら鎖の門）を書く。
6. The pasta shall 内部の説明（`internals/execution-model.md`・`internals/shiori.md`）の `EVENT.fire` の振り分けの順を、鎖を含む順に改める。
7. The pasta shall 生成のスキルの references を、マニュアルから作り直す。
8. The pasta shall 手書きのスキル `.claude/skills/pasta-lua-coding/SKILL.md` のモジュール表の `pasta.shiori.event.register` の行と §6 の要約に、`REG:register(id, fn)` を足す（分類器が止めるので、直すその場で開発者の許可を得る）。
