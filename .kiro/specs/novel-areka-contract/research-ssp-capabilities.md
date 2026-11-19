# 調査ノート: SSP で今できること・足せば出来ること・新しい仕組みが要ること（2026-11-19）

`/kiro-discovery`（ノベルゲームの深掘り）で、開発者の指示「areka でないとできないこと、SSP でも現状で可能なこと、SSP が対応すれば比較的簡単に可能なことを ukadoc も読みながら深掘り検証して仕訳よ」を受けて、調査のサブエージェント（Opus）が書いた。A〜C は ukadoc を読んだだけの調査である。実機で確かめた分は、末尾の D に書いた（2026-11-19。クリックの要らない項目だけ）。【推測】の印の記述は、根拠にする前に実機か原典で確かめる。

## 調べ方
- ukadoc MCP（search_docs / get_doc）を使った。あわせて、その元になっている Markdown のリポジトリ（github.com/ukatech/ukadoc-spec と ukadoc-ssp）をスクラッチパッドに浅く clone して grep した。置き場所はこのセッションのスクラッチパッド。リポジトリには何も書いていない。SSP・areka には何も送っていない。
- 「根拠」の列は ukadoc で確かめた事実だけを書いた。推測には【推測】と付けた。出典は、ukadoc-spec のタグ名・イベント名・ページ名と、SSP ヘルプ（ssp_help）のページ名で示す。

---

## A. 仕訳の表

| 項目 | SSP で今できること（ukadoc の根拠） | 仕訳 | 補足 |
|---|---|---|---|
| 1a 1 行ずつ渡す（`\x` → `\![raise,OnPlayNext]`） | `\x`「バルーンをクリック待ちにする。クリック後、スコープはリセットされ\0となり…\fの効果も解除。\eで解除されるものは継続」。`\x[noclear]` は内容・スコープ・\f を保つ（SSP）。`\![raise,イベント名,r0,…]` は再生がそのタグまで来たときにイベントを出す（記述例「トークの末端にあるraiseが再生された時…通知される」）。`\![notify]` は「raiseと同じで、違いは…値を返してもSSPに反映されない」ので、raise の返り値は反映される（＝スクリプトを返せる）。`\t` の説明には「`\![raise,イベント名]`を用いて一旦スクリプトを終了させ、別のイベントを発生させる」とある。 | SSP で今できる | 【驚き】`\![embed,イベント名,…]` もある（SSP/Ninix）。実行がそのタグまで来るとイベントを呼び、返り値でタグを置き換えて、そのまま続けて実行する。「スクリプトが1つなので…フォント変更などが持続する」。`\x\![embed,OnPlayNext]` なら 1 本のスクリプトのまま次の行をつなげられる【推測：\x の解除後に embed が評価されるかは実機で確かめる】。`\C` を先頭に書くと、直前のバルーンの文に追記できる（SSP/Ninix）。ADV/NVL の切り替えに使える。 |
| 1b `\x` を解く操作 | ukadoc の書き方は「クリック後」だけ。SSP ヘルプ「設定：喋り/バルーン」の「バルーンの操作」で、ボタンごとの割り当て（何もしない／左クリックと同じ／スクリプトブレーク…）を変えられる。既定は左ダブルクリックでブレーク。キー操作で解けるかは記載なし。 | 実機の確認が要る（キー） | クリックの範囲（バルーンだけか、シェルもか）も記載なし。 |
| 1c `\![raise]` の制限 | SHIORI 応答の SecurityLevel が external だと「一部のスクリプトがセキュリティ機能により実行不能」（spec/shiori3 response）。raise が external で止まるかは記載なし。NOTIFY 応答でも ValueNotify ヘッダで `\![raise]` を実行できる（同 response）。 | SSP で今できる | 普通の GET 応答は local【推測】。 |
| 1d 待っている間の Status と OnSecondChange | Status の値は talking / choosing / minimizing / induction / passive / timecritical / nouserbreak / online / opening(…) / balloon(…)（spec/shiori3 request）。`\x` で待っている間の値は記載なし。OnSecondChange：「トーク再生不能な時は、Reference3が0になった上でNOTIFY…返されたスクリプトは無視」。sound オプションの注記にも「スクリプトを再生している最中に起きたOnSecondChangeなどは…スクリプトを返せないNOTIFYになる」とある。 | 実機の確認が要る（\x 中の値） | |
| 1e 再生中に別のスクリプトが届いたとき | OnSoundSpectrum の注記：「返したスクリプトは、頭に`\C`を付けない限り、再生中や再生待ちのスクリプト(ほかのイベントが返したものを含む)を中断する」。SSTP には `Option: nobreak`（中断せずキューに積む）がある。`\t` はマウス系などのイベントを止める。nouserbreakmode はユーザーによる中断を止める。 | SSP で今できる（中断の規則）／実機（\x 中のマウスイベント） | `\t` が止めるのだから、`\t` が無ければ再生中もマウスイベントは届くはず【推測】。 |
| 1f バルーン系のイベント | OnBalloonBreak「SSTP以外でブレイクされた際」（R0 スクリプト、R1 スコープ、R2 中断位置）。OnBalloonClose「バルーンを閉じた際」。OnBalloonTimeout「選択肢以外でバルーンがタイムアウトした際」（R1 残り時間）。 | SSP で今できる | |
| 2 オート | SSP ヘルプ「喋り/バルーン」の「クリック待ちタイムアウト（標準＝しない）…クリックを待たずに読み進める」は利用者の設定。スクリプトから変えるタグは記載なし。`\![set,choicetimeout,時間]` は選択肢用、`\![set,balloontimeout,時間]` はバルーン表示用（どちらもそのスクリプト中だけ有効）。`\_w[ms]`・`\__w[ms]` は `\x` を起点に数えられる。`\![timerraise,ms,1,Ev]` もある。 | SSP で今できる（SHIORI が `\x` を `\_w[n]` に置き換える）／クリックでもタイマーでも進む形は「SSP が対応すれば比較的簡単」 | 足すもの：`\x` にタイムアウトを付ける引数か、`\![set,clickwaittimeout,ms]`。`\x` と timerraise を組み合わせた割り込みは実機で確かめる。 |
| 3 早送り・スキップ | `\_q`・`\![quicksection,true/false]`（瞬間表示。ただしウェイトのタグは実行する）。`\![set,balloonwait,倍率]`（0 で常に即時、スクリプトが終わると 100% に戻る）。SSP ヘルプ「早送りキーを有効にする…押している間はウェイトを無視」。既読スキップは記載なし。 | SSP で今できる（SHIORI が既読を覚え、`\x` を省いて `\_q` を付ける） | 本体の「既読スキップ」機能は無い。早送りキーが `\x` を飛ばすかは記載なし。 |
| 4 知らないタグ（`\![play,…]`） | ukadoc に記載なし。SSP ヘルプのエラーログの節は「ものによります」とだけ書いている。 | 実機の確認が要る | |
| 5 バックログ | 【驚き】`\![open,backlogviewer]`「バックログ(発言履歴)ウィンドウを開く」（SSP 2.8.52）。ショートカットは Ctrl-B。ゴーストごとの履歴で、吹き出しの形に顔の画像が付く（SSP ヘルプ「機能」）。`\__v[disable]` の区間は記録しない。`\__v[alternate,文字列]` はその文字列で記録する。顔の切り出しは surfaces の `icon.rect`、枠の色は `sakura/kero/char*.icon.frame.color.r/g/b`。ウインドウモードでは中に取り込める。中身を読む・消す・飛ぶためのイベントやプロパティは記載なし。 | SSP で今できる（見るだけ）／戻って読み直すのは「SSP が対応すれば比較的簡単」 | 足すもの：履歴の 1 件をクリックするとイベントが届くこと（R に行の識別子）と、消すタグ。ゴーストが自前で作るなら、大きいバルーンに `\_a`・`\__q` で過去の行を並べられる（今できる）。 |
| 6 印・選択肢・アンカー | `\q[タイトル,ID]`（OnChoiceSelect。SSP は先に OnChoiceSelectEx）。`\q[タイトル,OnID,r0,r1,…]`（OnID を直接出し、R0 から引数）。`\q[タイトル,ID,r2,r3…]`（OnChoiceSelectEx の R2 以降）。`\q[タイトル,script:…]`。`\__q[ID,…]…\__q`（範囲が選択肢で、自動改行あり。ID の扱いは \q と同じ）。`\_a[ID]`（OnAnchorSelect）・`\_a[OnID,r0,…]`・`\_a[ID,r2,…]`（OnAnchorSelectEx。R0 テキスト、R1 ID、R2 以降）。アンカーは「選択肢タイムアウトしないが…バルーンタイムアウトする」。`\*` と `\![set,choicetimeout,0]` でタイムアウトしない。 | SSP で今できる | 巻き戻しそのものは SHIORI 側の仕事。 |
| 7 3 人以上 | `\p[ID]`・`\pID`（SSP/CROW/Ninix）。シェルの `char*.name` 他、`char*.seriko.alignmenttodesktop` 等。バルーンは `balloonp*def*.png`（`\p[2]` は balloonp2def0/1）、clickwait・arrow・marker も p2def 以降がある（manual/balloon）。`\_s[ID1,ID2…]`。 | SSP で今できる | 人数の上限は記載なし。sticky-window の注記には「指定できる数に制限はない」とある。 |
| 8 背景・小物 | ・`\![set,wallpaper,ファイル,オプション]`：通常は OS の壁紙を替える。【驚き】ウインドウモード中は「ウインドウモードのウインドウの背景を変更」（2.8.93）。--position=fit/fill/…、--color。`\![save/restore,wallpaper]`。<br>・`\![bind,カテゴリ,パーツ,1/0]`・`\![bind-noevent,…]`（SSP 2.8.23）<br>・`\i[ID]`・`\i[ID,wait]`・`\__w[animation,ID]`・`\![anim,clear/pause/resume/stop,ID]`・`\![anim,offset,ID,x,y]`（SSP）。`\![anim,add,overlay/text…]` は CROW だけ。<br>・`\![move,--X,--Y,--time,--base,--base-offset,--move-offset,--option]`・`\![moveasync,…]`／`\![moveasync,cancel]`<br>・`\![set,zorder,…]`（s,b 記法でバルーンも制御。front/back 記法は 2.9.35）・`\![set,sticky-window,…]`<br>・`\![set,scaling,…,--time,--wait]`・`\![set,alpha,0-100,--time,--wait]`（2.7.52 以降）<br>・`\![lock,repaint(,manual)]`・`\![set,alignmenttodesktop,top/bottom/left/right/free/osuwari/default]`・`\![set,windowstate,stayontop/!stayontop/minimize]`<br>・`\![effect]`・`\![effect2]`・`\![filter]`・`\![set,position]` は **CROW だけ**（SSP では使えない） | 背景を絵で敷くのはウインドウモード中なら SSP で今できる。デスクトップ上では、背景役のキャラ（大きいサーフェス＋zorder back）で代用 | デスクトップ上での「背景」は窓が 1 枚増えるだけで、舞台にはならない【推測】。 |
| 9 場面転換 | 【驚き】キャラごとのフェード：`\![set,alpha,n,--time=ms,--wait]`。拡大縮小：`\![set,scaling,…,--time]`。移動：`\![move]`。サーフェスを切り替えるとき SSP はクロスフェードする（`seriko.force_disable_crossfade` で強制的に切れる＝既定は有効、SSP 2.8.43）。SERIKO の描画メソッドは overlay・base・move・scaling・blend-*（乗算・スクリーン等多数）・reduce・interpolate。pattern の `--alpha,0-255`（2.8.37）。揺れは move メソッド（瞬間的にずらす。利用者が「SERIKO Moveを無視」で切れる）か、`\![move]` を連打する。 | キャラ単位は SSP で今できる。画面全体のワイプやクロスフェードは「新しいベースウェアの仕組みが要る」 | 画面全体の転換は、合成した舞台が無いとできない。ウインドウモードのコンテナにも、転換のタグは記載なし。 |
| 10 テキストの窓 | `\b[ID]`（偶数だけ。`\b[-1]` で非表示。--fallback）。`\![set,balloonoffset,x,y]`（@ で相対）。`\![set,balloonalign,left/center/top/right/bottom/none]`。`\![lock,balloonmove]`／`sakura.balloon.dontmove`。`\![execute,resetballoonpos]`。`\_l[x,y]`（em/lh/%/@/centerx）。`\c`・`\c[char,…]`・`\c[line,…]`。`\C`。`\n[half]`・`\n[%]`・`\_n`。`\f[align/valign/name/height/color/shadowcolor/shadowstyle/outline/bold/italic/strike/underline/sub/sup/default/disable]`。`\_b[画像,x,y,--option=foreground/fixed,--scaling…]`。`\![set,autoscroll,disable/enable]`。`\![lock,balloonrepaint]`。縦書きはバルーンの descript `vertical,1`（SSP 2.8.80）。話し手の名前を出す専用の欄は記載なし。ルビのタグも記載なし。 | SSP で今できる（ルビと名前欄を除く）／ルビと名前欄は「SSP が対応すれば比較的簡単」 | NVL の全画面は、専用のバルーンを同梱して `\b[2]` などで出す（大きさはバルーンの絵で決まる）。名前は `\_b` の画像か、行の先頭の文字で代用できる。 |
| 11 音 | `\_v`・`\_V`・`\8`。`\![sound,play/load,ファイル,--volume,--balance,--rate,--seektime,--window,--sound-only,--eq,--spectrum…]`。`\![sound,loop,…]`（\_V の対象外）。`\![sound,pause/resume/stop(,ファイル)]`（省略すると全部）。`\![sound,wait]`。`\![sound,option,ファイル,…]` で再生中に変えられる。イベント OnSoundStop（end/close）・OnSoundLoop。プロパティ sound.*（position・duration・playing 等）。フェードの専用オプションは記載なし。 | SSP で今できる（BGM のループ・停止・音量・同時再生・終わりを待つ）／フェードは「option を `\_w` で刻む」工夫でできる、専用にするなら「SSP が対応すれば比較的簡単」 | 足すもの：`--fade=ms`。 |
| 12 入力 | OnKeyPress「本体がアクティブな状態でキーが入力された際」（R0 キー、R1 仮想キーコード、R2 押しっぱなしの回数、R3 窓、R4 修飾キー）。OnMouseClick/DoubleClick/Down/Up/Wheel/…Ex（R3 スコープ、R4 当たり判定、R5 ボタン、R6 touch/pen/mouse）。当たり判定は surfaces の collision*/collisionex*。右クリックメニューは SHIORI リソースの `sakura/kero/char*.popupmenu.visible`（0 で非表示）。`\![open,inputbox,ID,…,--placeholder…]` → OnUserInput / OnUserInputCancel。 | SSP で今できる | キーはゴーストがアクティブなときだけ届く。ウインドウモードでは F11・ESC・Ctrl+W をコンテナが使う（SSP ヘルプ「ウインドウモード」）。 |
| 13 1 枚の窓・全画面の舞台 | 【驚き】SSP の「ウインドウモード」（実験的機能、Windows 8 以降、起動オプション `--windowmode` は 2.8.78 から）。ゴーストを 1 枚のコンテナの中に表示する。全ゴーストで 1 枚か、ゴーストごとに 1 枚かを選べる。全画面（F11）、枠を消す、背景（白/灰/黒/デスクトップ/画像）、バックログの取り込み、スクリーンショットがある。ゴースト側には、プロパティ `baseware.windowmode`（normal/shared/perghost、2.8.93）、イベント OnWindowModeChange、OnDisplayChangeEx（コンテナの中を 1 枚の画面として通知）がある。切り替えは右クリックメニュー・本体設定・起動オプションで行う。ゴーストから切り替えるタグは記載なし。 | SSP で今できる（利用者が切り替える前提）／ゴーストから入るのは「SSP が対応すれば比較的簡単」 | 足すもの：`\![set,windowmode,…]` 相当と、コンテナの大きさ・全画面を指示するタグ。コンテナの中でもキャラは別々の窓のままなので、画面全体の合成（転換・ワイプ）は新しい仕組みが要る【推測】。 |
| 14 セーブ・スクリーンショット | `\![execute,dumpsurface,…]`（合成済みのサーフェスを PNG に、SSP 2.5.98）。`\![execute,dumpballoon,…]`（バルーンの今の描画を PNG に、SSP 2.9.02、`--hide=balloonmarker,balloonnum`）。出力先はゴーストのフォルダの下などに限られる。画面全体を撮るのは、ウインドウモードの「スクリーンショット…」（利用者の操作）だけ。 | 部品ごとなら SSP で今できる。画面全体のサムネイルは「SSP が対応すれば比較的簡単」 | 足すもの：コンテナを撮ってイベントで返すタグ。 |
| 15 プロパティ | `\![get,property,イベント名,名前…]` → 指定したイベントの R0 以降に値が入る。`\![set,property,…]`。関係するもの：currentghost.status、currentghost.scope(ID).surface.num/x/y/rect/scaling/name、currentghost.scope.count、currentghost.balloon.scope(ID).rect/validwidth/validheight/lines/char_width/vertical/scaling、currentghost.shelllist.*、baseware.windowmode、sound.*、system.monitor.*、system.cursor.pos。 | SSP で今できる | バックログやクリック待ちの状態を取るプロパティは無い。 |

---

## B. 実機で確かめる項目と送るスクリプト

1. キーで `\x` が解けるか、シェルをクリックして解けるか
   `\0\s[0]キーかシェルのクリックで進むか？\x\0進んだ。\e`
2. `\x` の解除後に後ろのタグが実行され、raise の返り値が再生されるか
   `\0一行目\x\![raise,OnPlayNext,2]\e`（OnPlayNext は `\0二行目\x\![raise,OnPlayNext,3]` を返す）
3. embed で 1 本のスクリプトのまま続けられるか
   `\0一行目\x\![embed,OnPlayNext,2]`
4. `\x` で待っている間の Status と OnSecondChange の届き方
   `\0待機中\x\e`（SHIORI 側で OnSecondChange の Method・Reference3・Status を記録する）。プロパティでも見る：`\![get,property,OnStatusProbe,currentghost.status]\0待機中\x\e`
5. `\x` で待っている間に OnMouseClick が届くか。その返り値が今のスクリプトを中断するか
   `\0クリックしてみて\x\e`（OnMouseClick で `\0割り込み\e` を返す）
6. クリックでもタイマーでも進む形が組めるか
   `\0三秒で自動\![timerraise,3000,1,OnPlayNext]\x\e`
7. 知らないタグの扱い
   `\0前\![play,bg,test]後\e`（「後」が出るか、エラーログに出るか、タグがそのまま表示されるか）
8. 早送りキーが `\x` を飛ばすか
   `\0一\x二\x三\e`（早送りキーを押しながら再生する）
9. `\![set,balloontimeout]` が `\x` に効くか
   `\![set,balloontimeout,2000]\0一\x二\e`
10. ウインドウモードで wallpaper が背景を替えるか
    `\![set,wallpaper,--file=bg.png,--position=fill]\e`

---

## C. 驚いたこと（pasta 開発者が思っているより SSP ができること）

- **バックログが本体にある**。`\![open,backlogviewer]`（2.8.52）。顔のサムネイルが付き、`\__v[disable/alternate]` で記録を外す・置き換えることもできる。
- **ウインドウモード**。1 枚の窓・全画面・背景画像・スクリーンショットを、SSP が実験的機能として持っている。その中では `\![set,wallpaper]` が舞台の背景を替えるタグになる。
- **`\![embed]`**。イベントの返り値をその場に差し込んで実行を続けるので、raise でスクリプトを切らずに 1 行ずつ渡せる見込みがある。
- **フェードと拡大縮小に時間を付けられる**。`\![set,alpha,n,--time,--wait]`・`\![set,scaling,…,--time,--wait]`。サーフェスの切り替えは既定でクロスフェードする（force_disable_crossfade があることから）。
- **音がかなり揃っている**。ループ・一時停止・シーク・音量・EQ・スペクトラムのイベント・終了のイベントがあり、BGM はほぼ足りる。フェードだけは専用のオプションが無い。
- **`\_l` が em/lh/%/相対で書け、縦書きのバルーンがある**（2.8.80）。`\_b` は前面・固定・拡大縮小ができる。
- 逆に、`\![effect]`・`\![effect2]`・`\![filter]`・`\![set,position]`・`\![anim,add,…]` は CROW だけのタグで、SSP では使えない。

---

## D. 実機で確かめた結果（2026-11-19。SSP 2.9.34）

discovery のセッションが、B の項目のうち、クリックの要らない物を実機で確かめた。画面の操作（クリック）は、開発者が許可を見送ったので行っていない。

### 確かめ方

- hello-pasta を複製して名前を変えたゴースト（辞書は確認用の 1 枚。台詞は仮の物）を、`ssp.exe --option readonly --sstp-listen 9900 --ghost <複製>` で、普段の SSP とは別に立てた。pasta.dll は、この作業ツリーの x86 のリリースビルド（0.6.2）。
- イベントは SSTP の NOTIFY で送った。Status は SSTP の `EXECUTE GetStatus` で 0.3 秒おきに読んだ。届いた要求は pasta.log で読んだ（pasta.log は、204 を返した要求を残さない）。吹き出しの中身は `\![execute,dumpballoon]` の画像で見た。
- スクリプトがどこまで進んだかは、`\![notify,OnProbe,印]` を挟んで、その要求が届いたかで見た。

### 結果

| B の番号 | 項目 | 結果 |
| -------- | ---- | ---- |
| 1 | キー・シェルのクリックで `\x` が解けるか | **未確認**（クリックが要る） |
| 2 | `\x` の後ろのタグが動き、raise の返り値が再生されるか | **クリックで解く道は未確認**。タイマーの道（6）では、raise の返り値が、待っているスクリプトを置き換えて再生された |
| 3 | embed で 1 本のまま続くか | **未確認**（クリックが要る） |
| 4 | `\x` で待つ間の Status | **`talking,balloon(0=0)` のまま**（95 秒見た）。その間、ランダムトークは 1 度も出なかった（間隔は 45〜75 秒の設定）。待ちの外では、OnSecondChange が GET・Reference3＝1 で届き、ランダムトークが出た。待ちの間の OnSecondChange の形（GET か NOTIFY か）は、見ていない |
| 5 | 待ちの間のクリックと、その返り値 | **クリックは未確認**。SSTP で送ったイベントの返り値は、`\x` の待ちを置き換えて再生された（SSTP は 200 を返した） |
| 6 | クリックでもタイマーでも進む形 | **組める**。`\![timerraise,3000,1,OnPlayNext,three]\x` は、3 秒後に OnPlayNext が GET で届き（SenderType は `internal,raise`。Status は talking）、返したスクリプトが待ちを置き換えて再生された。`\x` の後ろのタグは実行されない（解けたのでなく、置き換わったため）。クリックで先に進んだときにタイマーを取り消す形（時間 0 の timerraise）は、見ていない |
| 7 | 知らないタグ（`\![play,bg,test]`） | **読み飛ばされる**。字としては出ず、後ろの字とタグは続く（吹き出しの画像は「前後」）。エラーログは見ていない |
| 8 | 早送りキーが `\x` を飛ばすか | **未確認**（キーが要る） |
| 9 | `\![set,balloontimeout]` が `\x` に効くか | **効かない**。2 秒を指定して 12 秒見たが、`\x` は解けず、Status は talking のままだった |
| 10 | ウインドウモードで wallpaper が背景を替えるか | **未確認**（利用者の切り替えが要る） |

B に無い物:

- **`＆優先` の勝ち負け**（pasta の側）: 条件の無い `＊OnMouseClick` と、`＆優先：1` と部位の条件が付いた `＊OnMouseClick` を並べた。頭のクリックの要求を 6 回送ると 6 回とも優先つきが、体の要求を 3 回送ると 3 回とも条件なしが返った。
- **`\![notify]`**: スクリプトの途中の notify は NOTIFY で届き、返した値は再生に響かない。
- **SSTP で起こしたイベントの返り値**でも、`\![notify]`・`\![timerraise]`・`\![execute,dumpballoon]` は動いた（応答の SecurityLevel は local）。

### 分かったこと（discovery の見立て。決定ではない）

- `\x` で待つ間は Status が talking なので、今の pasta の門（`virtual_dispatcher.lua` の `is_blocked`）のままで、待っている行に時報・雑談は割り込まない。行と行の間は talking でない瞬間が在りうるので、「演劇の間は時報・雑談を返さない」の決まりは要る。
- オート（時間で次へ）は、`\![set,balloontimeout]` では作れず、`\![timerraise]` なら今の SSP で作れる見込みである。フェーズの表では、オートをフェーズ 2（SSP が対応すれば）に置いている。フェーズ 1 へ移すかは、クリックの道（1・2）を確かめてから、開発者が決める。
- 知らない `\![play,…]` は SSP で害が無い。pasta がベースウェアの出し分けを誤っても、字として画面に出ない。

確かめていないこと：クリック・キー・目で見る項目（1・2・3・5・8・10 と、バックログの `\__v[disable]`、透明度・大きさの `--time`、音）。待ちの間の OnSecondChange の形。ルビのタグは、ukadoc に書いていないだけで、存在しないとは言い切れない。ウインドウモードは SSP 自身が「実験的機能」としており、仕様が変わるおそれがある。
