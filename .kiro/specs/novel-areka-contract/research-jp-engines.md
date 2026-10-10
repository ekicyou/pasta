# 国産ノベルゲームエンジンのスクリプト言語調査ノート

調査日: 2026-10-10
目的: ノベルゲーム用スクリプト層に必要な表現力を、既存エンジンのスクリプト仕様から割り出す。エンジン内部や宣伝文句ではなく「スクリプトから見える機能と意味論」を対象にする。

## 0. 読み方と信頼度

各記述には出どころの印を付けた。

- 【確認済】 今回の調査で取得したページ・ソースコードに書かれていた内容。
- 【記憶】 取得ページでは裏が取れず、学習時点の知識で書いた内容。設計判断の根拠にする前に原典で確かめること。
- 【未確認】 調べたが分からなかった点。

エンジン別の信頼度:

| エンジン | 取得できた一次資料 | 信頼度 |
|---|---|---|
| 吉里吉里2/KAG3 | タグリファレンス全体、栞・既読・通過記録・変数の各ページ | 高 |
| NScripter/ONScripter | ONScripter API リファレンス（英語版、全 40 万字のうち序文・索引と該当項目） | 高（一部の命令は項目未読） |
| TyranoScript | タグリファレンス（前半 10 万字＋後半の一部）、GitHub 上のソース（kag.menu.js / kag.tag.js） | 高 |
| Suika2 | フォーク上に残る公式コマンドリファレンス全文、save.c / seen.c | 高（ただし旧版リファレンス。@anime @layer @pencil などは未収載） |
| Artemis Engine | 公式サイトの紹介ページのみ。タグリファレンスは SDK 同梱で非公開 | 中〜低（タグの属性は未確認） |
| 宴 (Utage) | コマンド一覧、テキスト表示、テキストタグ、Param シート、シナリオ書式、システムセーブ | 中〜高（セーブの再開位置は未確認） |
| Light.vn | wiki のコマンド目録と初心者向け逆引き | 中 |
| YU-RIS (ERIS) | ERIS マニュアルの目次（文字化けを推定で復号） | 低〜中（命令名のみ） |
| CatSystem2 | 解析 wiki のファイル形式ページのみ | 低 |
| LiveMaker | 公式サイトはプレイヤー向け操作説明しか取れず | 低 |
| ティラノビルダー | 公式サイトとチュートリアル目次 | 中 |

公式サイトが落ちていたもの: suika2.com（"See you soon!" のみ）、github.com/suika2engine/suika2（404）、artemis-engine.net（DNS 解決不可。公式は www.ies-net.com）。

---

## 1. 吉里吉里2 / 吉里吉里Z + KAG3

タグ式（`[tag attr=value]` または行頭 `@tag`）。地の文はそのまま表示テキストになる。KAG3 は TJS2 で書かれたフレームワークで、タグは TJS の関数に対応する。

### A. テキスト表示

- 【確認済】クリック待ちは 2 段階。`[l]` が行末クリック待ち、`[p]` が改ページクリック待ち。`[r]` は改行、`[cm]` は全メッセージレイヤ消去、`[er]` は現在のレイヤだけ消去、`[ct]` は消去＋操作対象を message0 に戻す。
- 【確認済】表示速度は `[delay speed=]`、瞬間表示は `[nowait]`〜`[endnowait]`。特定文字の後に自動で間を入れる `[autowc]`、文字数分待つ `[wc]`。
- 【確認済】装飾は `[font size face color bold italic shadow edge]`、既定値は `[deffont]`、戻すのは `[resetfont]`。行揃え・行間・字間は `[style]`。字下げは `[indent]`〜`[endindent]`。
- 【確認済】`[ruby text=]` は「次の 1 文字」にルビを振る。縦中横は `[hch]`。インライン画像（外字）は `[graph]`。
- 【確認済】メッセージレイヤは複数持て、`[current layer= page=]` で書き込み先を切り替える。`[position]` で位置・大きさ・枠画像・不透明度・縦書き（`vertical`）を決める。つまり ADV 窓と全画面 NVL の違いは `[position]` の設定差でしかない。
- 【確認済】クリック待ち記号は `[glyph]`。窓を一時的に隠すのは `[hidemessage]`。
- 【確認済】履歴は `[history output= enabled=]` で記録可否と表示可否を切り替え、`[hr]` で履歴上の改行、`[hact exp=]`〜`[endhact]` で履歴の行をクリックしたときの式を仕込む（ボイス再生の聞き直しに使う）。
- 【記憶】話者名の欄は標準機能に無い。名前用のメッセージレイヤを別に持ち、マクロで書き分けるのが定番。ボイス同期も標準には無く、マクロと SE バッファで組む。

### B. フロー制御

- 【確認済】ラベルは `*name`。`*name|見出し` と書くと「セーブ可能なラベル」になる。同名ラベルは 2 個目以降に `:2` などの通し番号が付く。
- 【確認済】`[jump storage= target=]`、`[call]`／`[return]`、`[s]`（停止。選択肢待ちに置く）。
- 【確認済】条件は `[if exp=]`／`[elsif]`／`[else]`／`[endif]`、逆条件で読み飛ばす `[ignore]`〜`[endignore]`。式は TJS2。
- 【確認済】変数は 3 種。`f.` はゲーム変数（栞と一緒に保存）、`sf.` はシステム変数（栞と無関係に永続）、`tf.` は一時変数（保存しない）。`[eval exp=]` で代入、`[emb exp=]` で本文へ埋め込み、属性値には `&式` のエンティティ記法で式を書ける。`[clearvar]`／`[clearsysvar]`。
- 【確認済】`[iscript]`〜`[endscript]` で TJS を直書きできる。
- 【確認済】マクロは `[macro name=]`〜`[endmacro]`、消すのは `[erasemacro]`。
- 【記憶】マクロ内では `%属性名` と `*`（全属性の受け渡し）、`mp.属性名` が使える。どのタグにも `cond=` 属性で実行条件を付けられる。
- 【確認済】選択肢は 3 系統。文中リンク `[link target= exp=]`〜`[endlink]`、画像ボタン `[button graphic= target= exp=]`、クリッカブルマップ `[mapimage]`／`[mapaction]`。`[locklink]`／`[unlocklink]`。
- 【確認済】時間切れジャンプ `[timeout time= target=]`、クリックでジャンプ `[click]`、ホイールでジャンプ `[wheel]`。これで時限選択肢を作れる。
- 【確認済】入力は `[input]`（ダイアログ）、`[edit]`／`[checkbox]`＋`[commit]`（フォーム）。
- 【確認済】右クリックの動作は `[rclick call= jump= target= storage=]` でサブルーチンに割り当てる。メニュー画面をスクリプトで書くための入口。
- 【記憶】乱数は TJS の `intrandom()` を式で呼ぶ。専用タグは無い。ループは `[jump]` で組む。

### C. システム機能との協調

- 【確認済】セーブは「栞」。`|` 付きラベルを通過するたびに内部で状態を控え、利用者が栞をはさむと「最後に通過したセーブ可能ラベルの時点」のデータが書かれる。ロードはそのラベルから再開する。離れた場所でセーブしても戻る位置はラベル。
- 【確認済】メッセージレイヤの文字は栞に入らない。だからセーブ可能ラベルの直後に `[cm]` を書くのが作法。`[trans]` と `[wt]` の間などにはセーブ可能ラベルを置かない。
- 【確認済】`[disablestore]` で栞を一時禁止（タイトル画面など）。`[store enabled=]` で恒久設定。`[save]`／`[load]`／`[copybookmark]`／`[erasebookmark]`、メモリ上の `[tempsave]`／`[tempload]`、サムネイル固定の `[locksnapshot]`。
- 【確認済】栞の互換性はラベルに依存する。セーブ可能ラベルを改名・削除したり、サブルーチンの戻り先の構造を変えたりすると古い栞が壊れる。ラベル名を省略した書き方はシナリオ編集でずれやすい。
- 【確認済】「前に戻る」は通過記録。`[record]` か Config の自動記録で、次のセーブ可能ラベルを通過したときに 1 つ前のラベル時点の状態が履歴に積まれる。`[goback]` で戻る。戻り先は `record` の行ではなく直前のセーブ可能ラベル。段数は `maxHistoryOfStore`。`l`／`p`／`s` が 1 つも無い区間の `record` は無視される。
- 【確認済】`[startanchor]` で「最初に戻る」の戻り先を決め、`[gotostart]` で戻る（変数は保持）。
- 【確認済】既読は「ラベルから次のラベルまたは `[s]` まで」が 1 単位。`sf.trail_<ファイル名>_<ラベル>` に通過回数が入る。途中でやめた区間は未読のまま。`jump`／`link`／`button` は既定で現在区間を既読にし、`call` は既定でしない。`countpage` 属性で変えられる。
- 【確認済】スキップ関連は `[clickskip]`、`[nextskip]`（次の選択肢まで進む）、`[cancelskip]`、`[cancelautomode]`。各待ちタグに `canskip` がある。
- 【記憶】タイトル、コンフィグ画面、CG 鑑賞、シーン回想、音楽室は標準に無い。`sf.` 変数と `[button]`／`[rclick]` で作者が組む。

### D. 映像

- 【確認済】レイヤは背景 `base`、前景 `0..N-1`、メッセージ `message0..`。数は `[laycount layers= messages=]`。各レイヤに表（fore）と裏（back）のページがある。
- 【確認済】基本の流れは「裏ページに組む → `[trans]` → `[wt]`」。`[backlay]` で表を裏へ写す。`[trans]` の属性は `method`（crossfade／universal／scroll）、`rule`（ルール画像）、`vague`、`from`、`stay`、`time`、`children`。
- 【確認済】`[image storage= layer= page= visible= left= top= opacity= mode= pos= index=]`。`pos` は left／center／right などの位置指定。`[layopt]` で後から属性変更、`index` が重ね順。`[pimage]` は部分追加読み込み（表情差分の重ね合わせ）。`[freeimage]`、`[copylay]`。
- 【確認済】`[move layer= path= time= accel= spline=]` で経路移動、`[wm]` で待つ。`[quake time= hmax= vmax=]`＋`[wq]`。
- 【確認済】`[animstart]`／`[animstop]`／`[wa]` でセルアニメ。
- 【確認済】動画は `[video]`／`[openvideo]`／`[playvideo]`／`[wv]`。レイヤへ描く `[videolayer]`、区間ループ `[videosegloop]`、フレーム到達イベント `[videoevent]`＋`[wp]`。
- 【記憶】拡大・回転・ぼかし・パーティクルは素の KAG3 に無い（プラグインか KAGEX）。`[image]` には色補正（ガンマ、グレースケール）や反転の属性がある。

### E. 音

- 【確認済】BGM は `[playbgm storage= loop= start=]`、`[fadeinbgm]`、`[fadeoutbgm]`、`[fadebgm volume= time=]`、`[xchgbgm overlap=]`（クロスフェード）、`[pausebgm]`／`[resumebgm]`／`[fadepausebgm]`、`[bgmopt volume= gvolume=]`。ループ点通過時の処理 `[setbgmlabel]`、停止時の処理 `[setbgmstop]`。
- 【確認済】SE は `buf` 番号で多重再生。`[playse storage= buf= loop=]`、`[fadeinse]`／`[fadeoutse]`／`[fadese]`、`[seopt buf= volume= gvolume= pan=]`。
- 【記憶】ボイス専用の仕組みは無く、SE バッファの 1 本を充てる。ループ点は `.sli` ファイル。ダッキングは無い。

### F. 同期

- 【確認済】`[wait time= mode= canskip=]`。`mode=until` と `[resetwait]` を組むと「基準時刻から何 ms 後まで待つ」になり、簡易タイムラインが書ける。
- 【確認済】完了待ち: `[wt]`（トランジション）、`[wm]`（移動）、`[wa]`（アニメ）、`[wq]`（揺れ）、`[wb]`（BGM フェード）、`[wl]`（BGM 終了）、`[wf]`（SE フェード）、`[ws]`（SE 終了）、`[wv]`／`[wp]`（動画）。大半に `canskip`。`[waitclick]`、`[waittrig]`（TJS からの合図待ち）。
- 構造: 効果を出すタグは待たずに次へ進み、待つかどうかは作者が待ちタグで決める。並行演出はこの「出しっぱなし＋後で待つ」で書く。

### G. サンプル（自作）

```
*scene1|放課後の教室
[cm]
[image storage="bg_classroom" layer=base page=back]
[image storage="aya_smile" layer=0 page=back visible=true pos=center]
[trans method=crossfade time=800][wt]
[playbgm storage="evening"]
【あや】「一緒に帰らない？」[p]
[link target=*yes]はい[endlink][r]
[link target=*no]いいえ[endlink][s]

*yes|一緒に帰る
[cm][eval exp="f.aya_like += 1"]
【あや】「やった！」[p]
[jump target=*common]
```

---

## 2. NScripter / ONScripter

命令式（1 行 1 命令、`:` で連結）。命令でない行がそのまま本文になる。BASIC 風で、変数は番号。

### A. テキスト表示

- 【確認済】本文中の特殊文字: `@` クリック待ち、`\` 改ページ待ち、`/` 直後の改行を無視、`_` 直後の待ちを抑止、`!w<ms>` 飛ばせない待ち、`!d<ms>` 飛ばせる待ち、`!s<速度>` 文字速度、`!sd` 既定速度、`#rrggbb` 色、`$n`／`%n` 変数埋め込み。
- 【確認済】窓は `setwindow`（文字位置・桁数・行数・字サイズ・字間・速度・太字・影・窓色または窓画像・窓座標）。`texton`／`textoff`、`textclear`、`locate`、`br`、`textspeed`。
- 【記憶】既定は全画面 NVL。`setwindow` で下段の小窓にすれば ADV 風になる。話者名の欄は標準に無く、本文に書くか下記のタグで自作する。
- 【確認済】ルビは `rubyon` のうえで `(本文/ルビ)` 形式。禁則は `kinsoku` 系。折り返しの字下げ `indent`。
- 【確認済】本文の前に `[名前/ボイスファイル]` のようなタグを置け、`pretextgosub` で登録したサブルーチンが `gettag` で読んで処理する。話者名表示やボイス再生をスクリプト側で作る口。
- 【確認済】`textgosub` でクリック待ちの処理そのものをスクリプトに差し替えられる。`clickstr` で指定文字の後に自動クリック待ち、`linepage` で行ごとに改ページ、`autoclick` で一定時間後に自動送り。
- 【確認済】ログは `lookbackbutton`／`lookbackcolor`／`lookbackon`／`lookbackoff`／`lookbackflush`／`maxkaisoupage`。`getlog` でページを読める。

### B. フロー制御

- 【確認済】スクリプトは `*define`（定義部）と `*start`（実行部）に分かれ、`game` で定義部を終える。ラベルは `*name`（英数字と下線、大小区別なし）。
- 【確認済】`goto`、`gosub`／`return`、`if`／`notif`、`for`／`next`／`break`、`jumpf`／`jumpb`（`~` 印への前方・後方ジャンプ）、`skip`（行数指定）、`tablegoto`。
- 【確認済】変数は数値 `%0`〜`%4095`、文字列 `$0`〜`$4095`、配列 `?`（`dim`）。0〜199 が通常変数（セーブデータに入る）、200 以降がグローバル変数（`globalon` が必要、全セーブ共通）。境界は `value` で動かせる。`numalias`／`stralias` で名前を付ける。
- 【確認済】演算は `mov`／`add`／`sub`／`inc`／`dec`／`mul`／`div`／`mod`、`rnd`／`rnd2`、`itoa`／`atoi`、`len`／`mid`／`split`。
- 【確認済】ユーザー定義命令は `defsub` と `getparam`。
- 【確認済】選択肢は `select "文",*ラベル,...`、`selgosub`、`selnum`（番号を変数へ）。`csel` 系はシステムカスタマイズ用。画像ボタンは `btndef`／`btn`／`btnwait`（右クリックや時間切れは負の値で返る）、`spbtn`。`trap` はクリックでラベルへ飛ぶ割り込み。
- 【記憶】ファイル分割は `0.txt`〜`99.txt` の連結。

### C. システム機能との協調

- 【確認済】セーブ位置は既定で「表示する文の先頭ごと」に自動更新される。`saveoff` にすると、ロード時は最後に `saveon` だった位置から再開する（高速なアニメ中に毎回記録されるのを避けるため）。定義部の `autosaveoff` で文頭以外の自動セーブ点を止め、必要な所に `savepoint` を置く。
- 【確認済】`savegame n`／`loadgame n`（確認なし）、`savegame2`（文字列付き）、`savefileexist`、`savenumber`。ロード直後に呼ぶ `loadgosub`。
- 【確認済】右クリックメニューは `rmenu` で項目を並べ、`systemcall` でスクリプトから同じ機能を呼ぶ（skip、save、load、lookback、automode など）。
- 【確認済】既読は `kidokuskip`（kidoku.dat に記録、未読で止まる）と `kidokumode`。`labellog` はラベル通過の記録、`filelog` はファイル使用の記録。
- 【記憶】`filelog`＋`fchk` で「その画像を見たか」を判定して CG 鑑賞を作り、`labellog`＋`lchk` でシーン回想を作る。
- 【確認済】オートは `automode`／`automode_time`、`mode_ext`。`skipoff`。
- 【確認済】`voicevol` などの音量変更はセーブされない。

### D. 映像

- 【確認済】背景 `bg ファイル|色,効果`。立ち絵は `ld l|c|r,画像タグ,効果` と `cl`。左中右の 3 か所固定で、位置の x は `humanpos`、足元の線は `underline`、立ち絵層の上に来るスプライト番号は `humanz`、不透明度は `tal`。
- 【確認済】スプライトは `lsp 番号,画像タグ,x,y[,α]`（0〜999、番号が重ね順）、`lsph`（非表示で読む）、`csp`、`vsp`、`msp`／`amsp`、`cell`。画像タグ `:a;` などで透過方式とセルアニメ（枚数・間隔・ループ種別）を指定し、`:s/` で文字列スプライトも作れる。
- 【確認済】画面への反映は `print 効果` でまとめて行う。効果は番号で、組み込みは 1〜18（1 が瞬間、15 と 18 がマスク画像を使う）。`effect` で番号に効果＋時間を定義する。`effectskip` でクリック飛ばしの可否。
- 【確認済】`quake`／`quakex`／`quakey`、`monocro`（単色化）、`nega`、`mosaicin`／`mosaicout`、`flushout`。`bar`／`prnum` はゲージと数値表示。
- 【確認済】動画は `avi`／`mpegplay`／`movie`。
- 【記憶】表ページ・裏ページの概念は無く、「変更を溜めて `print` で一括反映」が代わり。拡大・回転は NScripter 2.x 系の `lsp2` で入った。

### E. 音

- 【確認済】BGM は `bgm`／`bgmonce`／`bgmstop`（`mp3`／`mp3loop` も同系）、フェード時間は `mp3fadeout`／`mp3fadein`、イントロ＋ループは `loopbgm`。セーブ時に再開できる `mp3save`。
- 【確認済】SE は `wave`／`waveloop`、チャンネル付きの `dwave ch,file`／`dwaveloop`／`dwavestop`（0〜49）。慣習としてチャンネル 0 がボイス。
- 【確認済】音量は `bgmvol`／`sevol`／`voicevol`／`chvol`。

### F. 同期

- 【確認済】`wait`（飛ばせない）、`delay`（飛ばせる）、`resettimer`＋`waittimer`（基準時刻からの待ち）、`spwait`（スプライトアニメ終了待ち）、`click`／`lrclick`。
- 構造: 表示命令は効果の完了まで止まる（同期実行）。並行演出は苦手で、タイマーとスプライト操作のループで書く。

### G. サンプル（自作）

効果番号 10 がクロスフェードというのは【記憶】。

```
*define
numalias aya_like,10
game
*start
bg "bg\classroom.jpg",10
bgm "bgm\evening.ogg"
ld c,":a;chr\aya_smile.png",10
あや「一緒に帰らない？」\
select "はい",*yes,"いいえ",*no
*yes
inc %aya_like
あや「やった！」\
goto *common
```

---

## 3. TyranoScript

KAG3 の文法を HTML5/JavaScript 上で作り直したもの。タグの多くは KAG3 と同名で、ノベルゲーム向けの高水準タグ（キャラ管理、セーブ画面、コンフィグ、カメラ）が足されている。

### A. テキスト表示

- 【確認済】`[l]`／`[p]`／`[r]`／`[er]`／`[cm]`／`[ct]`／`[current]` は KAG3 と同じ。
- 【確認済】話者名は行頭 `#名前`。`[chara_ptext]` の省略形で、`#名前:表情` と書くと表情も変わり、`#` だけで名前を消す。名前欄は `[chara_config ptext=]` で指定したテキスト領域。
- 【確認済】`[chara_config talk_focus=brightness|blur|none]` で「いま話しているキャラ」以外を暗く／ぼかす。`talk_anim=up|down|zoom` で話すときに動かす。話者の判定は `#名前` による。
- 【確認済】速度は `[delay]`／`[resetdelay]`／`[configdelay]`、瞬間表示 `[nowait]`。装飾は `[font]`／`[deffont]`／`[resetfont]`、行間・字間・禁則は `[message_config]`。`[ruby]`、マーカー `[mark]`〜`[endmark]`、インライン画像 `[graph]`。
- 【確認済】演出文字 `[mtext]`（文字ごとの入退場エフェクト。バックログに残すには `[pushlog]` を併用）、レイヤ直書き `[ptext]`。
- 【確認済】ふきだし表示 `[fuki_start]`／`[fuki_chara]`。窓の属性は `[position]`、窓裏のフィルタ `[position_filter]`、一時非表示 `[hidemessage]`。
- 【確認済】バックログは `[nolog]`〜`[endnolog]` で記録停止、`[pushlog text= join=]` で任意追加、`[showlog]` で表示。
- 【確認済】ボイスは `[voconfig name= vostorage="x_{number}.ogg" number= sebuf= waittime=]` でキャラ名と連番ファイルを結び、`[vostart]` 以降はそのキャラが話すたびに自動再生して番号を進める。`waittime` はオート時のボイス終了後の間。`[popopo]` は文字送り音。`[speak_on]` は読み上げ。
- 【確認済】クリック待ち記号 `[glyph]`、スキップ中・オート中の記号 `[glyph_skip]`／`[glyph_auto]`。

### B. フロー制御

- 【確認済】`*ラベル`、`[jump]`（スタックに積まない）、`[call]`／`[return]`、`[s]`、`[if]`／`[elsif]`／`[else]`／`[endif]`、`[ignore]`、`[macro]`（`%属性|既定値` と `*`）、`[clearstack stack=call|if|macro]`（溜まったスタックを区切りで捨てる）。
- 【確認済】変数は `f.`（セーブデータごと）、`sf.`（システム、全体共通）、`tf.`（一時）。式は JavaScript。`[eval]`、`[emb]`、`[iscript]`、`[loadjs]`、`[trace]`、`[clearvar]`／`[clearsysvar]`。
- 【確認済】選択肢は `[link]`（文中）、`[glink text= target= exp= x= y=]`（文字ボタン。`[glink_config]` で自動配置）、`[button graphic= target= role= fix=]`（画像ボタン）、`[clickable]`（透明領域）。
- 【確認済】`[button role=]` に save／load／title／menu／window／skip／backlog／fullscreen／quicksave／quickload／auto／sleepgame を書くと組み込み機能のボタンになる。`fix=true` は常駐ボタン（call 扱い）。
- 【確認済】`[dialog]`（確認・入力）、`[edit]`＋`[commit]`。
- 【記憶】時限選択肢の専用タグは無く、`[wait]` と `[wait_cancel]` などで組む。

### C. システム機能との協調

- 【確認済・ソース】セーブは状態スナップショット方式。メニューを開いた時点などに `snapSave` が走り、`kag.stat` 全体の複製（変数、スタック、各種状態）とレイヤ部分の HTML、シナリオファイル名、「いま実行中のタグの 1 つ前の番号」を控える。ロードは HTML と stat を戻し、`make.ks` を call で差し込んでから控えた番号のタグから再開する。
- 【確認済・ソース】`[wait]` 中はメニューを開けない。イベントレイヤが隠れていて `[s]` 停止でもないときも開けない。つまりセーブできるのは実質クリック待ちと `[s]` 停止のとき。
- 【確認済】`[savesnap title=]` で任意の位置のスナップを取り、`[autosave]`／`[autoload]`、`[button savesnap=true]`。`[showsave]`／`[showload]`／`[showmenu]`。サムネイル差し替え `[save_img]`。
- 【確認済】`[sleepgame target=]`／`[awakegame]`／`[breakgame]`: 現在の状態を控えて別の場所（コンフィグ画面など）へ飛び、終わったら元の状態へ戻る。`variable_over`／`bgm_over` で変数と BGM を引き継ぐか選ぶ。
- 【確認済】`[checkpoint name=]`／`[rollback checkpoint= variable_over= bgm_over=]`／`[clear_checkpoint]`: 名前付きの巻き戻し点。多用すると重くなる。
- 【確認済・ソース】既読はラベル単位。ラベル通過時に直前のラベル区間を `sf.record.<シナリオ名>_<ラベル名>` に数え、通過済みなら `stat.already_read` が真になる。`[config_record_label color= skip=]` で既読文字の色と「未読もスキップできるか」を決める。
- 【確認済】`[skipstart]`／`[skipstop]`／`[cancelskip]`、`[autostart]`／`[autostop]`／`[autoconfig speed= clickstop=]`。
- 【記憶】タイトル、コンフィグ、CG 鑑賞、回想はテンプレートのシナリオファイル（title.ks、config.ks、cg.ks、replay.ks）として配られ、`sf.` 変数とマクロで動く。エンジン組み込みではなくスクリプト製。

### D. 映像

- 【確認済】`[bg storage= time= method= cross=]` で背景切り替え。`[image]`／`[free]`／`[freeimage]`／`[layopt]`／`[backlay]`／`[trans]`／`[wt]` は KAG3 互換。
- 【確認済】キャラ管理: `[chara_new name= storage= jname=]` で定義、`[chara_face]` で表情登録、`[chara_show name= face= left= top= zindex=]`（位置を省くと人数に応じて自動配置）、`[chara_hide]`／`[chara_hide_all]`、`[chara_mod face= cross=]`、`[chara_move]`。
- 【確認済】差分合成: `[chara_layer name= part= id= storage= zindex=]` でパーツを定義し、`[chara_part name= 部位=id ...]` で切り替える。チュートリアルに目パチ・口パクの項がある。
- 【確認済】アニメは `[anim]`（プロパティの補間）、`[keyframe]`＋`[frame]`＋`[kanim]`（キーフレーム）、`[xanim]`。待ちは `[wa]`。
- 【確認済】カメラ `[camera x= y= zoom= rotate= time= ease_type= layer=]`／`[reset_camera]`／`[wait_camera]`。
- 【確認済】画面効果: `[quake]`／`[quake2]`、`[filter]`（grayscale、sepia、blur、brightness など。レイヤ指定可）、`[layermode]`（乗算やスクリーンで色・画像を重ねる）、`[layermode_movie]`、`[mask]`／`[mask_off]`（暗転）。
- 【確認済】動画 `[movie]`、背景動画 `[bgmovie]`／`[wait_bgmovie]`。`[html]` で任意の HTML をレイヤに置ける。3D 系（`[3d_*]`）と AR 系もある。
- 【記憶】Live2D はプラグイン。

### E. 音

- 【確認済】`[playbgm storage= loop= volume= sprite_time= seek= restart=]`、`[fadeinbgm]`／`[fadeoutbgm]`、`[xchgbgm]`、`[pausebgm]`／`[resumebgm]`、`[bgmopt]`、`[changevol]`。
- 【確認済】`[playse storage= buf= loop= clear= volume=]`。`buf` がスロットで、同じスロットに鳴らすと前の音が止まる。`[fadeinse]`／`[fadeoutse]`、`[seopt]`、`[pausese]`／`[resumese]`。
- 【確認済】ボイスは SE のスロットを `[voconfig sebuf=]` で充てる。

### F. 同期

- 【確認済】`[wait time=]`／`[wait_cancel]`、`[wt]`、`[wa]`、`[wbgm]`／`[wse]`、`[wait_camera]`、`[wait_bgmovie]`、`[wait_preload]`。
- 【確認済】多くの演出タグが `wait=true|false` 属性を持つ。タグごとに「終わるまで待つか」を選ぶ方式で、KAG3 の「出して後から待つ」より書きやすい。

### G. サンプル（自作）

```
*start
[chara_new name="aya" storage="chara/aya/normal.png" jname="あや"]
[bg storage="classroom.jpg" time=800]
[playbgm storage="evening.ogg"]
[chara_show name="aya"]
#aya
一緒に帰らない？[p]
[glink text="はい" target=*yes x=300 y=200]
[glink text="いいえ" target=*no x=300 y=300]
[s]
*yes
[eval exp="f.aya_like = (f.aya_like || 0) + 1"]
#aya:smile
やった！[p]
```

---

## 4. Artemis Engine

Windows／iOS／Android／Web（wasm）／PS4／Switch で同じスクリプトが動く商用寄りのエンジン。一般公開はされておらず、作者への問い合わせで入手する。タグリファレンスは SDK 同梱で、公開ページからはタグの属性が分からない。

### A. テキスト表示

- 【確認済】文字の大きさ・色・配置は自由、縦書き・横書き、簡単な文字アニメ、ルビ、禁則、クリック待ち記号。
- 【確認済】フォントをアプリに同梱でき、文字に独自画像（ハート、汗マークなど）を割り当てられる。
- 【確認済】マクロを「改行」「空行」「行頭」に結び付けられる。これで地の文の改行や空行に行末待ち・改ページのタグを自動で差し込む。
- 【確認済】キャラ名を角括弧で書くキャラマクロで、立ち絵の表示と台詞の話者指定をまとめて行う。
- 【未確認】履歴登録やボイス同期のタグ名。

### B. フロー制御

- 【確認済】文法は KAG を参考にしたタグ式。ラベルは `*name`、コメントは `//`、変数参照は `$name`（式は `"$i < 100"` のように書く）。
- 【確認済】公式ページの例に出るタグ: `[lyc]`（レイヤ生成）、`[trans]`、`[wait]`、`[call]`、`[stop]`、`[sxfade]`、`[splay]`、`[var]`、`[loop]`〜`[/loop]`、`[dialog]`、`[print]`。
- 【確認済】サブルーチンとマクロで新しい命令を作れる。フラグ管理、四則演算、簡単な文字列編集、条件分岐、ループがタグでできる。
- 【確認済】複雑な処理は組み込みの Lua で書く。シナリオスクリプトより速く、オブジェクト指向や再利用のあるコードはこちらに寄せる、という役割分担が明記されている。
- 【確認済】選択肢は文字リンクと画像ボタン。タイトルやメニューなどの UI はすべてスクリプトで組む。レイヤごとにロールオーバー、クリック、ドラッグ開始・中・終了のイベントを拾える。
- 【確認済】利用者の文字入力（主人公の名前など）。

### C. システム機能との協調

- 【確認済】変数は 3 スコープ: セーブデータごと、全セーブ共通（グローバル）、保存しない一時変数。
- 【確認済】オートセーブがあり、スマホでホーム画面に戻されたときにも自動で保存する。画面のスクリーンショットをセーブのサムネイルに使える。
- 【確認済】オート、既読スキップ、強制スキップ、窓の非表示、バックログが組み込み。タッチ操作（フリック、マルチタッチ）への割り当ては変更・無効化できる。
- 【未確認】セーブの再開位置の粒度（ラベルか、行か、スナップショットか）。

### D. 映像

- 【確認済】レイヤは多数（端末性能が上限）。各レイヤが位置・拡大・回転・合成モードを持ち、値を時間変化させてアニメにする。
- 【確認済】レイヤセットで複数レイヤをまとめて動かす。
- 【確認済】全画面動画（mp4 など）と、レイヤとして置く動画（Ogg Theora、アルファ付き）。E-mote 対応は要相談。
- 【未確認】表裏ページの有無、ルール画像トランジションのタグ名、カメラ、パーティクル。

### E. 音

- 【確認済】BGM はフェードイン・アウト、クロスフェード、音量。イントロとループを別ファイルにして、イントロ後にループへつなぐ。
- 【確認済】SE とボイスは多重再生、フェード、チャンネルごとの音量。

### F. 同期

- 【確認済】`[wait]`、`[trans]`、`[stop]` が例に出る。
- 【未確認】完了待ちの系統とスキップ可否の指定方法。

### G. サンプル（自作・属性名は推定）

公開資料で確かめられたのはタグ名とラベル・変数の記法だけ。属性名は KAG からの類推で、実物とは違う可能性が高い。

```
*main
[lyc id="bg" file="bg/classroom"]            // 属性名は推定
[lyc id="aya" file="ch/aya_smile" x="400"]
[trans time="800"]
[sxfade file="bgm/evening" time="1000"]
[あや]「一緒に帰らない？」                  // キャラ名マクロ。行末の待ちは改行マクロが補う
[select text1="はい" label1="*yes" text2="いいえ" label2="*no"]  // 選択肢マクロは作者定義
*yes
[var name="aya_like" data="$aya_like + 1"]
[あや]「やった！」
```

---

## 5. Suika2

`@` で始まる 1 行 1 命令。命令でない行が本文。意図して機能を絞ってあり、UI は別形式の GUI 定義ファイル、複雑なロジックは WMS（別言語）に分ける。公式サイトは配布を終えており、ここではフォークに残る旧リファレンスとソースを読んだ。後継は Suika3（タグ式の NovelML）。

### A. テキスト表示

- 【確認済】地の文はそのまま 1 行書く。`\n` で改行、`$番号` で変数埋め込み、行頭 `\` で前のメッセージに継ぎ足す。
- 【確認済】話者付きは `*名前*本文`。ボイス付きは `*名前*ファイル.ogg*本文`。`*名前*@beep.ogg*本文` はビープ音。名前は名前欄に出る。
- 【確認済】1 メッセージは必ずクリック待ちで終わる（`@click` は窓を隠したクリック待ち）。KAG の `[l]` と `[p]` のような 2 段階の区別は無い。
- 【確認済】Wikipedia によればルビと縦書きに対応。
- 【記憶】後の版で文中の色・サイズ・待ちのエスケープ、`@pencil`（レイヤへの文字描画）が入った。

### B. フロー制御

- 【確認済】ラベル `:NAME`、`@goto ラベル`、`@gosub`／`@return`、`@load ファイル`（別スクリプトへ移る）。
- 【確認済】条件は `@if $1 == 1 ラベル` の「真ならジャンプ」だけ。ブロック構文は無い。演算子は `>` `>=` `==` `<=` `<` `!=`。左辺は変数、右辺は整数か変数。
- 【確認済】変数は整数だけ。`$0`〜`$9999` がローカル（セーブごと）、`$10000`〜`$10999` がグローバル（全セーブ共通）。`@set $1 += 2` の形で、演算子は `=` `+=` `-=` `*=` `/=` `%=`。乱数は `$RAND`。
- 【確認済】選択肢は `@choose ラベル1 "文1" ラベル2 "文2" ...`（最大 8）。
- 【確認済】`@gui ファイル` で GUI 定義を出す（最大 128 ボタン。ボタン種別に「ラベルへジャンプ」「変数が立っていれば表示」がある）。コンフィグ・セーブ・ロード・履歴の各画面も GUI 定義で作る。
- 【確認済】`@wms ファイル` で WMS を実行する。
- 【記憶】後の版でマクロ（サンプルに macro-demo がある）と名前変数（`%名前`。ソースに name_vars の保存がある）が入った。

### C. システム機能との協調

- 【確認済・ソース】セーブは「スクリプトファイル名＋コマンド番号＋`@gosub` の戻り位置（1 段）」に、ステージ（各レイヤのファイル名・位置・α）、アニメ、音、音量、変数、名前変数、ローカル設定を足したもの。複数行にまたがる構造の途中では、その構造の先頭のコマンド番号へ丸める処理がある。
- 【確認済】`@goto $SAVE`／`@goto $LOAD` でセーブ・ロード画面を出す。`@setsave enable|disable` でクリック待ち中の右クリックからのセーブ・ロードを禁止できる。
- 【確認済】`@skip enable|disable` で時間のかかる命令の飛ばしを禁止できる（ロゴ表示など）。この状態はセーブされないので、禁止中は `@setsave disable` も併用する、と注意書きがある。
- 【確認済】`@chapter "章名"` は章名を設定し、窓タイトルとセーブデータの項目に出る。
- 【確認済・ソース】既読フラグはコマンド単位の配列で、スクリプトファイルごとに保存する。
- 【確認済・ソース】グローバルデータ（グローバル変数、グローバル音量、設定）はセーブデータとは別ファイル。
- 【確認済】CG 鑑賞は GUI の「変数が立っていれば表示」ボタンで作る（旧 `@retrospect` の置き換え）。

### D. 映像

- 【確認済】`@bg ファイル 秒 効果`。背景を替えると立ち絵は全部消える。色指定（`#000000`）も可。
- 【確認済】効果は名前で指定: normal（フェード）、`rule:ファイル`（ルール画像）、curtain／slide／shutter の上下左右、clockwise／counterclockwise（20 度・30 度刻みもある）、eye-open／eye-close、slit-open／slit-close。
- 【確認済】立ち絵は `@ch 位置 ファイル 秒 効果 右ずれ 下ずれ α`。位置は center／right／left／back／face（顔アイコン）の固定 5 か所。`none` で消す。
- 【確認済】`@chs 中 右 左 奥 秒 背景 効果` で複数の立ち絵と背景を 1 回のトランジションで替える（`stay` は据え置き）。
- 【確認済】`@cha 位置 秒 加減速 x y α` で立ち絵を動かす（move／accel／brake）。
- 【確認済】`@shake 方向 秒 回数 振幅`、`@video ファイル`。
- 【確認済】Suika3 の資料では、レイヤ名に目（`-eye`）・口（`-lip`）のサブレイヤがあり、目パチと口パクの文書がある。Suika2 の後期版にも `@anime`／`@layer` がある（ソースに cmd_anime.c、cmd_layer.c）。

### E. 音

- 【確認済】`@bgm ファイル`（既定でループ、`once` で 1 回、`stop` で停止）。フェードアウトは `@vol bgm 0 2`→`@wait 2`→`@bgm stop` と書く。
- 【確認済】`@se ファイル`（`loop`、`stop`）。`@se ファイル voice` でボイストラックに鳴らす（音量確認用）。
- 【確認済】`@vol トラック 音量 秒`。トラックは bgm／voice／se の 3 本。小文字はローカル音量（セーブごと。演出用）、大文字はグローバル音量（全セーブ共通。コンフィグ用）。演出用の音量と利用者設定の音量を分けている。
- 【確認済】ボイスはメッセージ行に書く。

### F. 同期

- 【確認済】`@wait 秒`（入力で打ち切られる）、`@click`。
- 【確認済】各命令が自分の秒数を持ち、終わるまで次へ進まない。並行演出は `@chs` のような「まとめ命令」か、後期版の `@anime`（Suika3 では `async` 属性）で行う。

### G. サンプル（自作）

```
@bg classroom.png 1.0
@bgm evening.ogg
@ch center aya_smile.png 0.5
*あや*aya001.ogg*一緒に帰らない？
@choose YES "はい" NO "いいえ"
:YES
@set $1 += 1
*あや*やった！
@goto COMMON
:NO
*あや*そっか……
:COMMON
```

---

## 6. その他のエンジン（特徴だけ）

### 6.1 宴 (Utage)

Unity のアセット。スクリプトは表計算（Excel、v4 から CSV も）。

- 【確認済】1 行が 1 コマンド。列は見出し名で引く（Command、Arg1〜Arg6、Text、PageCtrl、Voice など）ので、列の並べ替えや削除ができる。行頭・列見出しの `//` とシート名の `#` でコメントアウト。
- 【確認済】Command が空で Arg1 にキャラ名、Text に台詞を書くと「立ち絵表示＋台詞」になる。Arg2 が表情パターン、Arg3 がレイヤ、以降 X／Y／フェード時間。Command も Arg1 も空なら地の文。キャラを 1 回書けば表示と話者指定が同時に済む。
- 【確認済】基本は「1 テキスト＝1 ページ」。PageCtrl 列で変える: Input（入力待ちのあと次へ）、InputBr（改行して入力待ち）、Next（待たずに次へ）、Br、BrPage、空（改ページ入力待ち）。
- 【確認済】ラベルは Command 列に `*名前`。`**名前` はシート内ローカル。
- 【確認済】分岐: `Jump ラベル 条件`、`JumpRandom`（確率付き）、`Selection ラベル 条件 選択時の式`、`SelectionClick`（表示中のオブジェクトを選択肢にする）、`If`／`ElseIf`／`Else`／`EndIf`、`JumpSubroutine`／`EndSubroutine`。
- 【確認済】変数は Param シートで事前に宣言する（Label、Type＝Int／Float／Bool／String、初期値、FileType）。FileType が保存範囲で、Default はセーブごと、System は全セーブ共通のシステムセーブ、Const は常にシートの値を読む定数（古いセーブがあっても調整値を直せる）。式は C# 風で `Param` コマンドに書く。Random などの組み込み関数がある。
- 【確認済】本文タグ: `<ruby=>`、`<em=>`（傍点）、`<speed=>`、`<interval=>`（その場で待つ）、`<param=>`／`<format=>`（変数埋め込み）、`<tips=>`、TextMeshPro の色・サイズなどのタグ。
- 【確認済】演出コマンドは WaitType 引数で「待つか」を選ぶ。`Thread`／`WaitThread` で別ラベルを演出スレッドとして並走させる。`WaitConditional`、`WaitCustom`、`SkipEffect`。
- 【確認済】`Bg` と `BgEvent`（イベント CG。初見で CG ギャラリーに登録）を分ける。`EndSceneGallery` でシーン回想の終わりを示す。音は Bgm／Ambience（環境音）／Se／Voice の 4 系統。
- 【確認済】設定シート: Character、Texture、Sound、Layer、Param、Particle、Animation、EyeBlink、LipSynch、SceneGallery、Localize。素材はシートに登録したラベルで呼ぶ。
- 【確認済】システムセーブは「ゲーム全体で 1 つ」。終了時、通常セーブ時、`BgEvent` で CG が解放されたときに自動保存する設定がある。
- 【記憶】通常セーブはページの先頭で取られ、ロードはそのページの頭から再開する。既読はページ単位。
- 【確認済】`SendMessage` 系で Unity 側のコードを呼ぶ。マクロはマクロシートに定義する。

サンプル（自作）:

| Command | Arg1 | Arg2 | Text | Voice |
|---|---|---|---|---|
| Bg | classroom | | | |
| Bgm | evening | | | |
| | Aya | smile | 一緒に帰らない？ | aya001 |
| Selection | *Yes | | はい | |
| Selection | *No | | いいえ | |
| *Yes | | | | |
| Param | aya_like+=1 | | | |
| | Aya | | やった！ | |
| Jump | *Common | | | |

### 6.2 Light.vn

- 【確認済】行頭 `~` がコマンド、それ以外は本文。コマンド名は日本語（英語名も併記される。絵=cg、背景=bg、背景音=bgm など）。
- 【確認済】コマンド名の前の `.` が同期指定（効果が終わるまで読み進めない）。付けなければ並行。同期・非同期を 1 文字で切り替える。
- 【確認済】コマンド名に `2` を付けると現在値からの相対指定になる。
- 【確認済】本文は `"` で新しいページ、`-"` で同じページに続ける、`\w` でクリック待ち。自動で待たせる `文字自動待機` もある。話者名は `~【なまえ】` というマクロ。
- 【確認済】オブジェクトに名前（c0、bg0 など）を付けて作り、`イン`／`アウト`／`移動`／`回転`／`拡大`／`透明度`／`色調` を名前あてに掛ける。カメラは「対象がカメラ」の移動・回転・拡大。
- 【確認済】変数の種類が多い: 変数、個体変数（オブジェクト付き）、全域変数、臨時全域変数、保存変数、システム変数。`もし`／`違ってももし`／`違ったら`、`反復条件`／`反復区間離脱`。
- 【確認済】`栞`（ラベル）、`ジャンプ ファイル 栞`、`スクリプト ファイル 栞`＋`スクリプト終了`（呼び出し）、`並列スクリプト`、`マクロ`。
- 【確認済】選択肢は `ボタン`＋`ボタン文字`＋`ボタングループ化`＋`待機 ボタン選択` の組み立て式。専用の選択肢コマンドではなく UI 部品で作る。
- 【確認済】`セーブ`／`ロード`／`クイックセーブ`／`クイックロード`／`ロールバック`、バックログ系（`バックログ設定`、`バックログ台詞登録`）、`文字進行スキップ`／`文字進行オート`、既読文字の色。
- 【確認済】Spine、3D モデル、シェーダー効果、ブレンドモード、マスク、粒子、物理、動画クロマキー。
- 【未確認】各変数の保存範囲の違い、セーブの再開位置。

サンプル（自作）:

```
~スクリプト system/macros.txt
~背景0 bg0 bg/classroom.jpg
~.イン bg0 500
~背景音 evening.ogg 反復
~絵0 c0 scg/aya_smile.png 100 0 10
~.イン c0 300
~文字窓下段
~【あや】
"一緒に帰らない？\w
```

### 6.3 YU-RIS（ERIS）

- 【確認済】YU-RIS は汎用のスクリプト処理系、ERIS はその上に YU-RIS スクリプトで書かれた ADV 用の層。マニュアルも 2 冊に分かれる。「汎用言語＋ADV ライブラリ」の 2 層構成の典型。
- 【確認済】ERIS の命令は `\名前` 形式で系統別（文字化けした目次からの推定復号）:
  - スプライト: `\SP.CG`（読み込み）、`\SP.GO`（動作開始）、`\SP.WA`（完了待ち）、`\SP.FINISH`、`\SP.SK`／`\SP.SKDEF`（スキップ時の扱い）、`\SP.DEL`、座標・α・拡大などの個別命令。
  - カメラ: `\CM.GO`、`\CM.WA`、`\CM.FINISH`。
  - 簡易マクロ: `\BG`、`\EV`（イベント絵）、`\FOUT`／`\FIN`（暗転・明転）、`\FLASH`。
  - 分岐: `\GO`、`\GO.IF`、`\GOSUB`、`\GOSUB.IF`、`\RETURN`、`\GO.TITLE`。変数: `\LET`、`\LET.IF`、`\RND`。
  - 選択肢: `\SEL`、`\SEL.GO`、`\SEL.RND`、`\SEL.TIME`（時限）、汎用の `\CSEL`。
  - 文字: `\C`、`\R`、`\P` など。音: `\BGM`、`\SE`、`\VO`。
  - 鑑賞モード登録: `\CG.SET`、`\RP.SET`／`\RP.END`（回想区間）、`\BGM.SET`、`\MV.SET`。CG・回想・音楽・動画の解放をスクリプトの専用命令で行う。
  - ほか `\SNOW`（雪）、`\MOVIE`、`\DATE`（日付表示）、`\DIALOG`、`\INPUT.STR`。
- 特徴: 「動作を設定 → `GO` で開始 → `WA` で待つ」の 3 段構えと、スキップされたときの挙動を命令ごとに決める `SKDEF`。
- 【未確認】本文の書式、変数スコープ、セーブ位置。

### 6.4 CatSystem2

- 【確認済】シーン脚本（.cst）は行の種別を持つバイナリに翻訳される: メッセージ、名前（次のメッセージの話者）、コマンド、入力待ち、改ページ（ノベルページを区切って入力待ち）。話者名と本文と待ちが別レコード。
- 【確認済】画面（タイトル、セーブ・ロード、ポーズなど）は別言語の .fes、アニメは .anm。立ち絵は .hg3（複数フレーム、基点付き）。翻訳用に言語別テーブル（CSTL）を持てる。
- 【記憶】コマンドは `bg`、`cg`（立ち絵。ポーズ・服・表情を番号の組で指定する合成式）、`bgm`、`se`、`pcm`（ボイス）、`wipe`、`frameon`／`frameoff`、`fselect` など。本文中のエスケープに `\n`、`\@`（入力待ち）、ルビがある。
- 【未確認】命令の正確な書式、セーブの方式。公式サイト（cs2.suki.jp）のマニュアルは取得できなかった。

### 6.5 LiveMaker

- 【確認済】Windows 用。開発終了、サポートなし。
- 【確認済】付属のプレイヤー向け説明から分かるシステム機能: オート、既読スキップと全スキップ、セーブ・ロード、文字速度とオート待ち時間、フォント選択、音量、ウィンドウ切り替え、履歴、Space で窓の表示切り替え。
- 【記憶】最大の特徴はフローチャート編集。シーン、選択肢、計算、条件分岐、ジャンプなどのノードを線でつなぎ、各シーンノードの中身を HTML 風タグのシナリオエディタで書く。スクリプトを書かずに分岐構造を組める反面、差分管理しづらい。
- 【未確認】タグ仕様。

### 6.6 ティラノビルダー

- 【確認済】TyranoScript を下敷きにした GUI 制作ツール。無料版と有料の PRO 版。PC・スマホ・タブレット・ブラウザ向けに出力する。
- 【確認済】チュートリアル項目: シナリオ、キャラ登場、背景、分岐、BGM、タイトル画面、フラグ管理（スクリプト）、変数管理、UI カスタマイズ、文字入力、CG とギャラリー、ふきだし、目パチ・口パク、翻訳、Live2D、プラグイン。
- 【記憶】部品を縦に並べて場面を作り、TyranoScript の断片を部品として差し込める。出力は TyranoScript のプロジェクト。
- 意味: 中身のスクリプト言語が表に出ていれば、GUI ツールは後からかぶせられる。

---

## 7. 統合

### 7.1 共通コア（どのエンジンにもある。「ノベルゲームが作れる」の下限）

テキスト
1. 地の文をそのまま書ける（命令と本文を行頭の印で見分ける）。
2. クリック待ち。少なくとも「ページ（メッセージ）単位」の待ち。
3. 話者名。エンジン組み込み（TyranoScript、Suika2、宴、Light.vn、CatSystem2）か、マクロ・前置タグでの自作（KAG3、NScripter、Artemis）。
4. 文字送りの速度（利用者設定あり）とクリックでの一括表示。
5. 改行と変数の埋め込み。
6. メッセージ窓の表示・非表示。

フロー
7. ラベルとジャンプ。
8. サブルーチン呼び出しと復帰。
9. 条件分岐（ブロック式か、真ならジャンプ式）。
10. 変数と代入・四則演算・比較。
11. 変数の保存範囲が最低 2 つ: セーブごと（ゲーム変数）と全セーブ共通（システム・グローバル変数）。
12. 選択肢（文を並べて選ばせ、ラベルへ飛ぶか変数に入れる）。
13. 複数ファイルへの分割とファイルまたぎのジャンプ。
14. 乱数。

システム
15. セーブ・ロード（再開位置と画面・音・変数の復元）。
16. バックログ。
17. スキップ（既読だけ／全部）と既読の記録。
18. オートモード。
19. 利用者設定（文字速度、オート速度、音量が BGM・SE・ボイス別）。

映像
20. 背景の表示と切り替え。
21. 立ち絵の表示・消去・差し替え（表情違い）。最低でも左・中・右の位置。
22. 重ね順。
23. トランジション: 瞬間、クロスフェード、ルール画像（マスク）ワイプ。
24. 画面揺れ。
25. イベント CG（背景と同じ仕組みで足りる）。

音
26. BGM: 再生、停止、ループ、フェードイン・アウト。
27. SE: 複数同時、ループ SE。
28. ボイス: 台詞に結び付けて再生。SE とは別の音量区分。

同期
29. 時間待ち（クリックで飛ばせる／飛ばせないの区別）。
30. 演出の完了待ち（トランジションが終わるまで進まない）。

### 7.2 よくあるが必須でない機能

- 行末待ちと改ページ待ちの 2 段階（KAG3、TyranoScript、NScripter、CatSystem2、宴）。Suika2 には無い。
- ADV 窓と全画面 NVL の切り替え、複数メッセージ窓。
- 文中の装飾（色、サイズ、太字）、ルビ、文中の待ち、インライン画像、縦書き。
- マクロ・ユーザー定義命令（KAG3、TyranoScript、Artemis、NScripter の defsub、宴、Light.vn）。
- 一時変数（保存しない第 3 のスコープ）。
- 画像ボタンの選択肢、クリッカブルマップ、時限選択肢。
- 文字入力（主人公の名前）。
- 汎用言語の埋め込み（TJS、JavaScript、Lua、WMS、C#）。
- 立ち絵の移動・拡大・回転・不透明度の補間、キーフレームアニメ。
- 差分パーツ合成（体＋表情＋服）、目パチ・口パク。
- BGM のクロスフェード、イントロ＋ループ、一時停止・再開。
- 演出用の音量と利用者設定の音量の分離（Suika2 のローカル／グローバル音量、KAG3 の volume／gvolume）。
- 色調フィルタ（セピア、モノクロ、ネガ）、暗転マスク、フラッシュ。
- 動画再生。
- CG 鑑賞・シーン回想・音楽室。多くは「全セーブ共通変数＋ボタン UI」で作者が組むが、宴と ERIS は専用の登録命令を持つ。
- タイトル・メニュー・コンフィグ画面をスクリプトで組む仕組み（右クリックサブルーチン、常駐ボタン、GUI 定義ファイル）。
- 巻き戻し（前の選択肢・前のラベルへ戻る）。
- ボイスのバックログ聞き直し、オート時のボイス終了待ち。
- 章タイトル・セーブデータの見出し。
- 既読文字の色変え。
- 多言語切り替え。

### 7.3 まれ・高度な機能

- カメラ（TyranoScript、ERIS、Light.vn、宴の ZoomCamera）。
- 並走するスクリプト・演出スレッド（宴の Thread、Light.vn の並列スクリプト、Suika3 の async）。
- 基準時刻つきの待ちによるタイムライン（KAG3 の `wait mode=until`、NScripter の `waittimer`）。
- 話者以外を暗くする・話者を動かす自動演出（TyranoScript の talk_focus／talk_anim）。
- スキップされたときの演出の挙動を命令ごとに決める（ERIS の SKDEF、宴の WaitType）。
- 文字ごとの入退場アニメ（TyranoScript の mtext）、ふきだし表示。
- パーティクル（雪・雨）、ぼかし、シェーダー、ブレンドモード、物理。
- 動画レイヤ（アルファ付き）、背景動画、動画のフレームイベント。
- BGM のループ点通過・停止のイベントフック（KAG3）。
- Live2D／E-mote／Spine／3D モデル。
- 名前付きチェックポイントへの巻き戻し（TyranoScript）、状態を控えて寄り道して戻る（sleepgame）。
- 環境音を BGM と別系統で持つ（宴）。
- 定数スコープの変数（宴の Const）。
- ダッキング（ボイス中に BGM を下げる）: 今回読んだスクリプト仕様のどれにも命令としては見当たらなかった。エンジンの設定で持つものはある【記憶】。
- ネットワーク、課金、ネイティブ呼び出し（Artemis）。

### 7.4 セーブ・ロードと実行位置

大きく「区切りから再実行する型」（ラベル・文頭・ページ）と「その場の状態を書き出す型」（コマンド番号＋状態・スナップショット）に分かれる。

| 型 | エンジン | 控える時点 | 再開位置 | 位置の表現 |
|---|---|---|---|---|
| ラベル粒度・再実行 | KAG3 | セーブ可能ラベルを通過したとき | そのラベル | ファイル名＋ラベル名 |
| 文頭粒度・再実行 | NScripter | 表示する文の先頭ごと（自動）。`saveoff`／`savepoint` で粗くできる | その文の先頭 | スクリプト上の位置（内部形式は未確認） |
| コマンド番号＋状態 | Suika2 | セーブ操作時（クリック待ち中） | 同じコマンド | ファイル名＋コマンド番号＋gosub 戻り位置 1 段 |
| スナップショット | TyranoScript | セーブメニューを開いたとき、または `[savesnap]`／`[autosave]` | 待っていたタグ（番号を 1 つ戻して再実行） | ファイル名＋タグ番号＋stat 全体＋レイヤ HTML |
| ページ粒度 | 宴 | ページの先頭【記憶】 | ページの頭【記憶】 | ラベル＋ページ番号【記憶】 |
| 未確認 | Artemis、Light.vn、ERIS、CatSystem2 | | | |

各型の性質:

**KAG3（ラベル粒度）【確認済】**
- 利用者はいつでもセーブ操作ができるが、書かれるのは「最後に通過したセーブ可能ラベルの時点の状態」。ラベル以降の変数変更は栞に入らない。だからロードしてラベルから再実行すれば同じ結果になる（乱数を除く）。
- 画面・音の状態はラベル通過時に控える。メッセージの文字は控えないので、ラベル直後で `[cm]` する決まり。
- 作者の負担: セーブ可能ラベルを適切な間隔で置く。演出の途中（trans と wt の間）に置かない。
- 互換性: 位置がラベル名なので、ラベルを変えなければシナリオを直しても古い栞が生きる。サブルーチンの呼び出し構造を変えると壊れる。
- 巻き戻し（通過記録）と既読も同じラベル単位に乗る。1 つの単位が 3 つの機能を支える。

**NScripter（文頭粒度）【確認済】**
- 作者が何もしなくても文ごとにセーブ点ができる。代わりに、ループ中やアニメ中の記録を `saveoff` で止める必要が出る。
- 文の途中の状態は持たない。ロードすると文の頭から表示し直す。

**Suika2（コマンド番号）【確認済・ソース】**
- 命令が少なく状態が小さい（固定 5 か所の立ち絵、BGM 1 本、整数変数）ので、状態を全部書き出せる。位置はコマンド番号 1 個で済む。
- サブルーチンの戻り位置は 1 段しか持たない。言語を絞ることでセーブを単純にしている。
- 弱点: スクリプトを直すとコマンド番号がずれ、古いセーブの位置が狂う。

**TyranoScript（スナップショット）【確認済・ソース】**
- 状態オブジェクト全体と画面の DOM を複製する。作者はセーブ点を意識しなくてよい。
- 弱点 1: JavaScript 側に持った状態（stat の外）は保存されない。ロード時に呼ばれる `make.ks` で作者が復元処理を書く。
- 弱点 2: 位置がタグ番号なので、シナリオ編集で古いセーブがずれる。
- 弱点 3: 待ち中や安定していない状態ではセーブメニュー自体を開かせない。
- 巻き戻しは別の仕組み（名前付き checkpoint）で、多用すると重い。

**宴（ページ粒度）**
- 【確認済】変数の保存範囲をシートの列で宣言する（Default／System／Const）。システムセーブは 1 つで、CG 解放などの節目に自動保存する。
- 【記憶】ページの頭に戻るので、ページ内の演出は再実行される。

設計への含意:
1. 「どこでもセーブ操作ができる」と「どこからでも再開できる」は別。KAG3 と NScripter と宴は前者だけを満たし、再開は直前の区切りから再実行する。区切りの粒度が、ラベル（作者が置く）、文・ページ（自動）で違う。
2. 再実行方式は、区切り時点の状態を控えておけば、区切りから先は同じスクリプトをもう一度走らせるだけでよい。途中状態の直列化（実行中のコルーチン、補間の途中、スタックの中身）が要らない。
3. 区切りを名前（ラベル）で指せばシナリオ修正に強く、番号（コマンド番号、タグ番号）で指せば弱い。
4. 既読・巻き戻し・セーブを同じ区切り単位に乗せると仕組みが 1 つで済む（KAG3）。
5. 再実行方式では、区切りから再開点までの間に「やり直してはいけない副作用」があると困る。全セーブ共通変数への加算（周回数、CG 解放）や乱数が該当する。KAG3 は sf 変数を栞と別管理にしているので、ロードしても sf への書き込みは巻き戻らない。
6. スクリプトに汎用言語を埋め込むと、その言語側の状態は保存の外に漏れる（TyranoScript の make.ks 問題）。Suika2 が WMS を「呼び出すだけの外部言語」に留めているのはこの回避。

### 7.5 スクリプト言語とエンジン（ランタイム）の役割分担

どのエンジンでも共通する線引き:

| 担当 | 内容 |
|---|---|
| スクリプト（シナリオ作者） | 本文と話者、表示する素材の名前と位置、演出の種類と時間、分岐の条件と行き先、フラグの更新、待つか待たないかの指定、セーブ可能な区切り（ラベル粒度の場合） |
| エンジン | 文字送りと禁則・折り返し、クリック入力の解釈（送り・スキップ・オート・窓消し）、トランジションと補間の描画、音のミキシングとフェード、セーブデータの直列化、バックログの蓄積と表示、既読の記録、利用者設定の保持と反映 |

線の引き方が分かれる所:

1. **システム画面（タイトル、メニュー、セーブ・ロード、コンフィグ、鑑賞モード）**
   - スクリプトで組む: KAG3、NScripter（textgosub／rmenu 差し替え）、Artemis、TyranoScript（テンプレートの .ks）、Light.vn（UI 部品）。
   - 別形式の定義ファイル: Suika2（GUI 定義）、CatSystem2（.fes）。
   - エンジン・ホスト側の部品: 宴（Unity の UI プレハブ）。
   - どの場合も、シナリオ本文を書く言語とは別の層として扱われている。

2. **話者名・ボイス・立ち絵の結び付け**
   - 低水準型（KAG3、NScripter、Artemis）: レイヤ・スプライト・SE バッファだけを提供し、「キャラ」という概念はマクロで作者が作る。
   - 高水準型（TyranoScript、Suika2、宴）: キャラ定義、話者記法、ボイス連番、話者強調をエンジンが持つ。
   - 高水準型のほうが 1 行あたりの記述が短い。低水準型は自由度と引き換えに、各作品が似たマクロを書き直す。

3. **待ちの既定**
   - 出しっぱなしで後から待つ: KAG3。
   - 命令ごとに wait 属性: TyranoScript、宴（WaitType）。
   - 既定で待つ・印で切り替え: Suika2（常に待つ）、NScripter（常に待つ）、Light.vn（`.` で同期）。

4. **ロジック**
   - 同じ言語で書く: NScripter、Suika2 の @set／@if。
   - 汎用言語を埋め込む: KAG3（TJS）、TyranoScript（JS）、Artemis（Lua）、宴（C# 呼び出し）、Suika2（WMS）。
   - Artemis の資料は「シナリオ言語は本文を書くためのもの、重い処理は Lua へ」と役割を言い切っている。

5. **素材の指し方**
   - ファイル名を直接書く: KAG3、NScripter、TyranoScript、Suika2、Light.vn。
   - 事前登録したラベルで呼ぶ: 宴（Texture／Sound／Character シート）、TyranoScript の chara_new／chara_face。
   - 登録式は表記ゆれの検査と差し替えがしやすい。

6. **利用者設定と演出の衝突**
   - 音量: 演出用と利用者設定用を分ける（Suika2、KAG3 の gvolume）。
   - 文字速度: 利用者設定を基本に、スクリプトは一時的に上書きして戻す（`[delay]`→`[resetdelay]`、`!s`→`!sd`、`<speed>`）。
   - スキップ: 演出を飛ばせるかを命令ごと（canskip、SKDEF、WaitType）か区間ごと（`@skip disable`、`effectskip`）に指定する。

---

## 8. 出典

吉里吉里2 / KAG3
- タグリファレンス: https://krkrz.github.io/krkr2doc/kag3doc/contents/Tags.html
- セーブ・ロード: https://krkrz.github.io/krkr2doc/kag3doc/contents/SaveLoad.html
- 栞のデータ: https://krkrz.github.io/krkr2doc/kag3doc/contents/SaveData.html
- 未読/既読処理: https://krkrz.github.io/krkr2doc/kag3doc/contents/ReadUnread.html
- 通過記録: https://krkrz.github.io/krkr2doc/kag3doc/contents/HistoryOfStore.html
- 変数: https://krkrz.github.io/krkr2doc/kag3doc/contents/Var.html
- 目次: https://krkrz.github.io/krkr2doc/kag3doc/contents/frame.html

NScripter / ONScripter
- ONScripter API リファレンス（英語）: https://kaisernet.org/onscripter/api/NScrAPI.html

TyranoScript / ティラノビルダー
- タグリファレンス: https://tyrano.jp/tag/
- ソース: https://github.com/ShikemokuMK/tyranoscript （tyrano/plugins/kag/kag.menu.js、kag.tag.js）
- ティラノビルダー: https://b.tyrano.jp/ 、https://b.tyrano.jp/tutorial/

Artemis Engine
- 公式の紹介ページ: https://www.ies-net.com/?page_id=24

Suika2 / Suika3
- 旧コマンドリファレンス（フォーク上）: https://github.com/RaZZlom/suika2/blob/master/doc/old/reference.md
- 同フォークの src/save.c、src/seen.c
- Wikipedia: https://ja.wikipedia.org/wiki/Suika2
- Suika3: https://github.com/awemorris/suika3 、https://suika3.vn/en/docs/novelml-tags/

宴 (Utage)
- コマンド一覧: https://madnesslabo.net/utage/?page_id=252
- テキスト表示: https://madnesslabo.net/utage/?page_id=1732
- テキストタグ: https://madnesslabo.net/utage/?page_id=1921
- Param シート: https://madnesslabo.net/utage/?page_id=1715
- シナリオ書式: https://madnesslabo.net/utage/?page_id=8766
- 設定シート: https://madnesslabo.net/utage/?page_id=249
- システムセーブ: https://madnesslabo.net/utage/?page_id=519
- 会話シーンに使う: https://madnesslabo.net/utage/?page_id=402
- トップ: https://madnesslabo.net/utage/

Light.vn
- コマンド目録: https://wikiwiki.jp/lightvn/command
- 初心者向け逆引き: https://wikiwiki.jp/lightvn/beginnerGP

YU-RIS
- トップ: https://yu-ris.net/
- ERIS マニュアル目次: https://yu-ris.net/manual/eris/html/menu.html

CatSystem2
- 解析 wiki: https://github.com/trigger-segfault/TriggersTools.CatSystem2/wiki
- CST Scene: https://github.com/trigger-segfault/TriggersTools.CatSystem2/wiki/CST-Scene

LiveMaker
- 公式: http://www.livemaker.net/ 、http://www.livemaker.net/files/adv-type1_03.txt

その他
- https://en.wikipedia.org/wiki/List_of_visual_novel_engines
