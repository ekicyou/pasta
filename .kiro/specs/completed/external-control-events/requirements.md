# Requirements Document

## Project Description (Input)

SSP の MCP（`raise_event`）などの外からゴーストを試験・操作する人は、辞書の `＞XXX` と同じ呼び出しを外から 1 回で実行する口を持たない。いちばん欲しいのは、試験の間だけ `＞自動トーク停止`／`＞自動トーク再開`（`ontalk-suppression`）を呼んで、ランダムトークと時報の割り込みを止めることである。グローバル変数（`＄＊名前`、Lua の `save.名前`）を外から書く口も無く、たとえば試験の間だけ `＄＊pasta_talk_interval_min` を縮めることができない。

今の手段はどちらも合わない。未登録のイベントは同じ名前のシーンへ回るだけで、呼びたい名前ごとに `＊OnFoo` を書く必要がある。VSCode のシーンキックはデバッグ専用で、シーン表を完全一致で引くだけであり、`GLOBAL` の関数・前方一致・別名表を通らない。

そこで、外から SHIORI のイベントを 1 つ送るだけで、辞書の `＞XXX` の呼び出し（`OnPastaCallFunction`）と、グローバル変数の代入（`OnPastaSetSaveVariable`）ができるようにする。詳しい経緯と決定は `brief.md` を参照。

## Introduction

この spec は、外から pasta を操作する 2 つの組み込みのイベントを定める。

- `OnPastaCallFunction`: Reference0 の名前を、辞書のシーンの外に書いた `＞名前` と同じ探し方で呼ぶ。Reference1 以降は、文字列のまま引数として渡す。
- `OnPastaSetSaveVariable`: Reference0 の名前のグローバル変数（`＄＊名前`）に、Reference1 を JSON の値として読んだものを代入する。`null` なら変数を消す。

```
OnPastaCallFunction     Reference0=自動トーク停止
  → 辞書に「＞自動トーク停止」と書いたのと同じ効き目（応答は 204）
OnPastaCallFunction     Reference0=初回起動
  → ＊初回起動 を再生（応答は 200 でさくらスクリプト）
OnPastaCallFunction     Reference0=好感度を足す  Reference1=10
  → GLOBAL["好感度を足す"](act, "10") と同じ（引数は文字列のまま）

OnPastaSetSaveVariable  Reference0=pasta_talk_interval_min  Reference1=60
  → ＄＊pasta_talk_interval_min ＝ 60（数値）
OnPastaSetSaveVariable  Reference0=会員番号  Reference1="01"
  → ＄＊会員番号 ＝ "01"（文字列。数値の 1 に化けない）
OnPastaSetSaveVariable  Reference0=初回済み  Reference1=null
  → ＄＊初回済み を消す（値なしに戻す）
```

2 つのイベントはいつでも働き、有効・無効を切り替える設定は足さない。SSTP の SEND で任意のさくらスクリプトを流せるのと同じ信頼の範囲である。止めたい作者は、同じイベントのハンドラを自分で登録すれば止められる。

主な利用者は 2 人である。

- **外から試験・操作する人**: SSP の MCP サーバーの `raise_event` などでイベントを送り、試験の間だけゴーストの振る舞いを変える人（AI エージェントを含む）。
- **ゴーストの作者**: 2 つのイベントを止めたり差し替えたりしたい人。

## Boundary Context

- **In scope**
  - `OnPastaCallFunction`・`OnPastaSetSaveVariable` の 2 つの既定のハンドラと、その引数の確かめ
  - 呼び出しの名前の探し方を、辞書のシーンの外に書いた `＞名前` と揃えること（新しい探し方を作らない）
  - Reference1 を JSON の値として読み、グローバル変数の一段目のキーに代入する・消すこと
  - 見つからないときと引数の誤りを、失敗を出す仕組みに載せること
  - 自動テスト
  - マニュアル（`lua/shiori-events.md`・`grammar/variables.md`）とスキルの references
- **Out of scope**
  - 値の読み出し（外から今の値を問い合わせること）
  - 呼び出しの結果を値で返すこと（`GLOBAL` の関数の戻り値を応答に載せるなど）
  - ローカルシーンの呼び出し、ローカル変数（`＄名前`）・`STORE` の書き換え、入れ子のパス（`a.b`）の代入
  - 有効・無効を切り替える設定（`pasta.toml`・`PASTA_DEBUG`）
  - SSP の MCP サーバーの側の変更
  - 書き換えるたびにディスクへ保存すること
- **Adjacent expectations**
  - **`＞` の呼び出しの解決**（`act:call`・`[scene.alias]`）: 探し方・候補の選び方・別名の当て方を、そのまま受け継ぐ。作り直さない。
  - **自動トークのスイッチ**（完了 spec `ontalk-suppression`）: 主な呼び先 `自動トーク停止`／`自動トーク再開` はあちらが持つ。この spec はそれを呼ぶだけで、止める・再開する動きそのものは持たない。
  - **シーンを再生するイベント**（完了 spec `scene-play-event`）: 既定のハンドラの置き方、`SecurityLevel` の門、失敗の出し方、マニュアルの既定ハンドラーの表を先例として揃える。あちらのハンドラと「シーンを再生するイベントの共通の決まり」の節は変えない。
  - **失敗を出す仕組み**（完了 spec `failure-output-unification`・`runtime-error-balloon`）: 失敗の表記の形と、出す・出さないの設定（`[logging] balloon_warning`）に従う。
  - **グローバル変数の保存**（`@pasta_persistence`・`save.json`）: 書いた値の保存は、辞書の `＄＊名前＝値` と同じく、終了時の自動保存に任せる。
  - **チェイントークの続き**（`chain-talk-interrupt`、同じウェーブ）: 外からの呼び出しが、待っているチェイントークの続きを残すか捨てるかは、ふつうのイベントと同じ決まりに従い、あちらが決める。
  - **同じウェーブの約束**（`chain-talk-interrupt`・`reg-handler-chain`）: 既定のハンドラの読み込みの並びには末尾に 1 行足すだけにする。マニュアルの既定ハンドラの節は、表の行と数の文だけを触り、その下の包み方の段落とシーン再生の共通の決まりの節は触らない。後から入る側が合わせる。

## Requirements

### Requirement 1: 外から `＞名前` と同じ呼び出しをする（OnPastaCallFunction）

**Objective:** As a 外から試験・操作する人, I want 辞書の `＞名前` と同じ呼び出しを、イベント 1 つで実行させること, so that 呼びたい名前ごとにシーンを書き足さずに、`自動トーク停止` などをすぐに呼べる。

#### Acceptance Criteria

1. When `OnPastaCallFunction` を受け取り、Reference0 に名前があるとき, the pasta shall その名前を、辞書のシーンの外に書いた `＞名前` と同じ探し方（グローバルシーンの前方一致・`GLOBAL` の関数・`[scene.alias]` の別名）で探し、見つかった呼び先を呼ぶ。
2. The pasta shall 呼び先を探す段と順と、同じ名前の候補が複数あるときの選び方を、辞書のシーンの外に書いた `＞名前` と同じにし、この spec のための探し方を足さず、除く呼び先も作らない（`＞名前` が届く `act` のメソッドにも同じく届く。完全一致のシーンキックや `OnPastaScene` とは、わざと揃えない）。
3. The pasta shall ローカルシーン（`・名前`）を呼び先として探さない。
4. When Reference1 以降に値があるとき, the pasta shall Reference1 から、値の無い最初の Reference の手前までを順に、受け取った文字列のまま（数値や真偽値に変えずに）呼び先の引数として渡す（`Reference1=10` なら引数は文字列の `"10"`。空の文字列も値として渡す）。
5. When 呼び先がシーンで、台詞を出したとき, the pasta shall 1 ビート目のさくらスクリプトをこのイベントの応答（200 OK）として返す。
6. When 呼び先が出力を積まないとき（`自動トーク停止` など）, the pasta shall 204 No Content を返す。
7. The pasta shall 呼び先の戻り値を、応答に載せない。

### Requirement 2: 呼び出しの結果と続き

**Objective:** As a 外から試験・操作する人, I want 外から呼んだものが、ふだんのイベントで呼んだのと同じ動きをすること, so that 試験で確かめた結果が、実際のゴーストの動きと食い違わない。

#### Acceptance Criteria

1. When 呼んだシーンに 2 ビート目以降があるとき, the pasta shall 続きを、ふだんのイベントで実行したシーンと同じ時機（次の自然なおしゃべりの機会）に出す。
2. The pasta shall このイベントを受け取ったその場で呼び先を呼び、ランダムトークを抑える Status（選択肢の表示中の `choosing` など）では止めない。
3. If 呼び先の実行がエラーで止まったとき, the pasta shall ふだんのイベントと同じく、止まるエラーの応答とログで知らせる。
4. When 呼び出しによって、チェイントークで待っているシーンの続きがあるとき, the pasta shall その続きを残すか捨てるかを、ふだんのイベントのハンドラが呼び先を呼んだときと同じ決まりにする（決まりそのものは `chain-talk-interrupt` が持つ。あちらより先に入っても、この spec の側で別の決まりを作らない）。

### Requirement 3: グローバル変数に代入する（OnPastaSetSaveVariable）

**Objective:** As a 外から試験・操作する人, I want グローバル変数（`＄＊名前`）に、型を指定して値を書き込ませること, so that 試験の間だけランダムトークの間隔を縮めるなど、辞書を書き換えずに状態を整えられる。

#### Acceptance Criteria

1. When `OnPastaSetSaveVariable` を受け取り、Reference0 に名前が、Reference1 に JSON の数値・文字列・真偽値があるとき, the pasta shall その値を、その名前のグローバル変数（`＄＊名前`）に代入する。
2. The pasta shall Reference1 を JSON の値として読み、数字の形をした JSON の文字列（`"01"`）を文字列のまま代入する（数値の `1` に変えない）。
3. When Reference1 が JSON の `null` のとき, the pasta shall その名前のグローバル変数を消し、値の無い状態に戻す（辞書から `＄＊名前` を読むと、値を代入していない変数と同じになる）。
4. The pasta shall Reference0 を、グローバル変数の一段目の名前としてそのまま扱い、`.` などで区切って入れ子のパスとして読まない（`Reference0=a.b` なら `a.b` という名前の変数に代入する）。
5. The pasta shall `pasta_` で始まる予約グローバル変数にも代入でき、代入した後の効き目を、辞書の `＄＊名前＝値` で代入したときと同じにする（`＄＊pasta_talk_interval_min` に数値でない値を入れたときは、今の決まりどおり無視される）。
6. When 代入したとき, the pasta shall その後に読む `＄＊名前`（辞書と Lua の `save.名前`）に、すぐに新しい値を見せる。
7. The pasta shall 代入した値をディスクへすぐには書き出さず、辞書の `＄＊名前＝値` と同じく、ゴーストの終了時の自動保存に任せる。
8. When 代入に成功したとき, the pasta shall 204 No Content を返す。
9. If Reference1 が JSON の配列かオブジェクトのとき, the pasta shall 値を変えず、失敗として知らせる（辞書の `＄＊名前` には表の値を使う書き方が無いため）。

### Requirement 4: 誤りと見つからないとき

**Objective:** As a 外から試験・操作する人, I want 呼べなかった・書けなかった理由がツールの結果とログに出ること, so that 名前や値の誤りを自分で直せる。

#### Acceptance Criteria

1. If `OnPastaCallFunction` の Reference0 が無いか空のとき, the pasta shall 何も呼ばず、イベント名と誤った引数（Reference0）と値の様子（値なし・空文字列）を、失敗の表記とログの警告に出す。
2. If `OnPastaCallFunction` で、Reference0 の名前の呼び先が見つからないとき, the pasta shall 何も呼ばず、イベント名と探した名前と「見つからない」ことを、失敗の表記とログの警告に出す（`OnPastaScene` の先例に揃える）。
3. If `OnPastaSetSaveVariable` の Reference0 が無いか空のとき, the pasta shall どの変数も変えず、イベント名と誤った引数（Reference0）と値の様子を、失敗の表記とログの警告に出す。
4. If `OnPastaSetSaveVariable` の Reference1 が無いか、JSON の値として読めないとき, the pasta shall 変数の値を変えず（消しもせず）、イベント名と変数の名前と読めなかった値を、失敗の表記とログの警告に出す（変数を消すのは、`null` を明示したときだけ）。
5. If JSON を読む部品（`@json`）を読み込めないために Reference1 を読めないとき, the pasta shall 変数の値を変えず、その旨を、ほかの失敗と同じく失敗の表記とログの警告に出す。
6. The pasta shall 2 つのイベントの失敗の表記を、ほかの実行時の失敗と同じ形（`【種類：対象（説明）】`。対象にイベント名を含める）で、同じ設定（`[logging] balloon_warning`）に従ってこのイベントの応答に載せ、出さない設定では 204 No Content を返す（表記はデバッグの有無によらず、既定で応答に載る）。
7. The pasta shall ログの警告を、表記を出す・出さないの設定にかかわらず出す。
8. If 呼び出しや代入が失敗したとき, the pasta shall チェイントークで待っているシーンの続きを、中断せずにそのまま残す。

### Requirement 5: 受け付ける相手と作者による差し替え

**Objective:** As a ゴーストの作者, I want 2 つのイベントが、設定なしで動き、必要なら自分で止められること, so that 試験のための設定が増えず、配るゴーストでも外からの操作を断てる。

#### Acceptance Criteria

1. The pasta shall 2 つのイベントを、デバッグが有効かどうか（`pasta.toml` の `[debug]`・環境変数 `PASTA_DEBUG`）にかかわらず働かせ、有効・無効を切り替える設定を足さない。
2. If リクエストの `SecurityLevel` が `external` のとき, the pasta shall 何も呼ばず・どの変数も変えずに 204 No Content を返し、その旨をログの警告に書く（`OnPastaScene`・`OnPastaClock` の先例に揃える。MCP の `raise_event` は `local` で届くので断られない）。
3. Where 作者が Lua で `OnPastaCallFunction` か `OnPastaSetSaveVariable` のハンドラを登録しているとき, the pasta shall 作者のハンドラを優先し、既定のハンドラで上書きしない（登録が既定のハンドラより前でも後でも）。
4. The pasta shall 2 つのイベントで、イベント名で始まる名前のシーン（`＊OnPastaCallFunction` など）への前方一致のフォールバックをしない。

### Requirement 6: マニュアルとスキルの references

**Objective:** As a 外から試験・操作する人, I want 2 つのイベントの使い方がマニュアルとスキルに載っていること, so that 自分も AI エージェントも、呼び方と結果の読み方が分かる。

#### Acceptance Criteria

1. The マニュアル shall SHIORI イベントの章（`book/src/lua/shiori-events.md`）の既定ハンドラーの表と既定ハンドラーの数の文に 2 つのイベントを足し、主要イベントの表にも載せる。
2. The マニュアル shall 2 つのイベントそれぞれの節で、引数、探し方（`OnPastaCallFunction`）・値の読み方（`OnPastaSetSaveVariable`）、応答、失敗の出方、`SecurityLevel` による断り、作者のハンドラでの止め方を説明する。
3. The マニュアル shall `OnPastaCallFunction` と `OnPastaScene` の違い（いつ働くか・探し方・ローカルシーンを指せるか）を、1 か所で読み比べられるように書く。
4. The マニュアル shall MCP の `raise_event` からの呼び方の例（`自動トーク停止` の呼び出しと、`pasta_talk_interval_min` の代入）を載せる。
5. The マニュアル shall 変数の章（`book/src/grammar/variables.md`）のグローバル変数か予約グローバル変数の説明から、`OnPastaSetSaveVariable` の節へ案内する。
6. When マニュアルを直したとき, the スキルの references shall マニュアルから作り直され、マニュアルと食い違わない。

### Requirement 7: 今の動きを壊さない

**Objective:** As a ゴーストの作者, I want この spec が今の動きを変えないこと, so that 配るゴーストと既存のデバッグの手段が、今までどおりに動く。

#### Acceptance Criteria

1. The pasta shall 2 つのイベント以外のすべてのイベントの挙動を変えない。
2. The pasta shall 辞書の `＞名前` の呼び出し、VSCode からのシーンの再生、`OnPastaScene`・`OnPastaSceneAt` の挙動を変えない。
3. The pasta shall 今ある自動テストを、書き換えずに通す（例外は、失敗の種類の名前の表の数を数えるテストだけ。表の決まり「後続 spec は自分の名前を足す」どおり、足した数に直す。2026-10-23 設計の話し合いで決定）。
4. The pasta shall 呼び出し（シーン・`GLOBAL` の関数・別名・引数・見つからない名前・Reference0 の誤り）と、代入（数値・文字列・真偽値・`null`・読めない JSON・名前の欠け）と、`SecurityLevel` が `external` のときを、自動テストで確かめる。

## 要件の話し合いでの決定（2026-10-23）

下書きの「仮定と未決事項」は、すべて閉じた。brief の Approach と食い違う決定（1〜3）は、こちらを正とする。

1. **見つからないときの出し方**（要件 4.2・4.6）: **決定（推奨案で承認）**。brief の Approach の「警告のログを残して 204」をやめ、先例（`OnPastaScene`）に揃えて、失敗の表記（`【シーンが見つからない：OnPastaCallFunction ＞名前】` のようにイベント名付き）とログの警告を出す。表記を出す設定なら応答に載る（既定は 200）。MCP のツールの結果に理由が出るので、送った側が自分で直せる。brief の「バルーンに出るのはデバッグのときだけ」は今のコードと合わない。`[logging] balloon_warning` の既定は `true` で、デバッグとは関係なく表記が応答に載る（`research.md` 参照）。外から送るのは試験する人だけなので、配るゴーストで出ても害は無い。
2. **代入の失敗も表記に載せるか**（要件 4.3・4.4・4.5・4.6）: **決定（推奨案で承認）**。brief の Approach の「成功でも失敗でも 204」をやめ、呼び出しと揃えて、失敗の表記とログの警告を出す。JSON の部品を外したゴーストのとき（未決事項 10）も同じ。読めない JSON の種類の名前は設計で決める。
3. **`SecurityLevel` が `external` のとき**（要件 5.2）: **決定（推奨案で承認）**。断って 204 とログの警告。変数の書き換えや `ゴースト終了` を、ほかの機械から呼べないようにする。MCP は `local` で届くので、試験には効かない。
4. **`act` のメソッドに届くか**（要件 1.2）: **閉じた（2026-10-23 要件の話し合い）**。brief の「`＞XXX` と同じ・新しい探し方を作らない」から、`＞名前` と完全に同じにし、メソッドにも届く。除くと探し方を別に作ることになる。
5. **JSON の配列・オブジェクト**（要件 3.9）: brief は「JSON の値として読む」とし、例は数値・文字列・真偽値・`null` だけ。**決定（推奨案で承認）**: 配列とオブジェクトは断る（辞書から表の値を使う書き方が無いため）。欲しくなったら、そのときに足す。
6. **入れ子のパスに見える名前**（要件 3.4）: **閉じた**。brief の「一段目のキーだけ」から、`a.b` は名前のまま代入する（区切らない・断らない）。
7. **Reference1 が無いとき**（要件 4.4）: **閉じた**。読めない値と同じ失敗にし、変数は消さない（消すのは `null` を明示したときだけ。値の欠けで消えると、送り間違いで状態を失う）。
8. **チェイントークの続き**（要件 2.4）: **閉じた**。brief の再測定の推奨どおり、ふつうのイベントと同じ決まりに従う。`chain-talk-interrupt` より先に入ると、今の決まりでは、台詞を出さない `自動トーク停止` を外から呼んでも待っている続きが捨てられるが、どちらの順でも壊れはしない。
9. **Reference の並び**（要件 1.4）: **閉じた**。Reference1 から、値の無い最初の Reference の手前までを引数にする。空の文字列は値として渡す（`＞名前（）` に空の引数を書いたのと同じ）。
10. **JSON を読む部品を外したゴースト**（要件 4.5）: JSON の部品（`@json`）を読み込めないと、Reference1 を読めない（注: `pasta.toml` の `[lua] libs` は撤去済みで、今は設定で外す口が無い。実装は `pcall(require, "@json")` の守りとして残す）。**決定（推奨案で承認）**: 変数を変えず、ほかの失敗と同じく失敗の表記とログの警告を出す（決定 2）。
