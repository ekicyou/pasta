# 欧米系ノベル／ナラティブ記述系の調査ノート（スクリプト言語とホスト契約）

調査日: 2026-10-10。目的は「既存の対話 DSL（Lua へトランスパイル・LuaJIT のコルーチンで実行・描画と音は別プロジェクトのホスト）の上にノベルゲーム層を載せる」ための要求抽出。

## 0. 凡例と信頼度

- 【確認】= 今回 WebFetch で取得したページの内容から確認した記述。
- 【二次】= 公式ページではなく検索結果（フォーラム・issue・リリースノート）経由で確認した記述。
- 【記憶】= 取得ページでは確認できず、学習済み知識から書いた記述。採用前に要再確認。
- 各システムの F（サンプル）は本ノート用に自作したもので、公式からの転記ではない。構文の細部（特に Naninovel の `pos`、Monogatari のアニメ名）は例示。

取得できなかった／内容が得られなかったもの:
- Naninovel の旧 URL（`/guide/naninovel-scripts`, `/guide/save-load-system`）は 404。ロールバックは「設定リファレンス」の項目説明からのみ確認（仕組みの解説ページは見つからず）。
- Ink `ArchitectureAndDevOverview.md` は状態の中身を説明していない → `StoryState.cs` のソースで確認。
- Ink `WritingWithInk.md` はタグ節が要約に出ず（タグは `RunningYourInk.md` 側で確認）。複数行 alternatives（`{stopping:}` 等）は【記憶】。
- Yarn Spinner の Dialogue Presenter のメソッド詳細は「custom dialogue views」ページで確認。`OnDialogueStartedAsync` 等は【記憶】。
- SugarCube／Harlowe の単一巨大ページは先頭 10 万字のみ読めた。履歴・セーブ・PRNG の詳細は【二次】または【記憶】。
- Monogatari は旧ドキュメント URL がリダイレクトされるため GitHub の Documentation リポジトリ raw で確認。
- Fungus は wiki の save_system・narrative_text_tags の 2 ページのみ確認。

---

## 1. Ren'Py

### A. 物語構成要素
- 【確認】say 文: 裸の文字列＝地の文、`"名前" "台詞"`、`e "台詞"`（Character オブジェクト）。`Character()` の主な引数: `name`, `kind`, `image`（立ち絵タグと紐付け）, `voice_tag`, `what_prefix/suffix`, `who_*`, `dynamic`, `condition`, `interact`, `advance`, `callback`, `ctc*`, `screen`, `retain`。
- 【確認】台詞に画像属性を添えられる: `e happy "..."`（立ち絵が出ていれば属性変更、出ていなければサイド画像用に保持）、`e @ happy "..."` はその行だけの一時変更、`-happy` で属性除去。
- 【確認】特殊キャラクタ: `narrator`, `extend`（直前行への追記）, `centered`, `nvl`（NVL モード）, `adv`。モノローグモード（三重引用符を空行で say 文に分割）。
- 【確認】menu 文: キャプション（say）＋選択肢。選択肢ごとに `if` 条件。`set` 節で「選んだキャプションを集合に入れ、集合にあるものは隠す」＝一度きりの選択肢。全選択肢が落ちたら menu 自体を飛ばして次の文へ。`config.menu_include_disabled` で不成立の選択肢を無効ボタンとして表示。menu／選択肢に引数を付けて画面へ渡せる。
- 【確認】ラベル: グローバル／ローカル（`.name`）、引数付き。`jump`（戻り先なし）、`call`（戻り先をコールスタックへ）、`return [値]`（`_return` に入る）。`jump expression` / `call expression` で動的ターゲット。
- 【確認】変数: `default x = ...`（常に保存対象）、`define`（定数扱い・保存されない）、`$ x = ...`（1 行 Python）。条件は `if/elif/else`、ループは `while`。式は Python そのもの。
- 【確認】乱数: `renpy.random`（`random()`, `randint`, `choice`, `shuffle`, `Random(seed)`）。ロールバックと協調し、何度巻き戻しても同じ数を返す。
- 【確認】既読／訪問: `renpy.seen_label()`, `renpy.mark_label_seen()`, `renpy.is_seen()`（現在行が既読か。全プレイ通算 or 今回のみ）, `renpy.count_seen_dialogue_blocks()` / `renpy.count_dialogue_blocks()`（既読率）。
- 【記憶】訪問回数のカウント機能は言語組込みではなく、変数で数える。

### B. 演出指示
- 【確認】画像名＝「タグ＋属性」（`mary beach night happy` → タグ `mary`）。同じタグを `show` すると置換。レイヤ: `master`（背景・立ち絵）, `transient`, `screens`, `overlay`（＋カスタムレイヤ）。
- 【確認】文: `image`（定義）, `show`（節: `at` 変換, `behind`, `onlayer`, `as`, `zorder`, `expression`）, `scene`（レイヤを消してから表示）, `hide`, `with`（トランジション）, `camera`／`show layer`（レイヤ全体に変換）, `window show/hide/auto`。
- 【確認】`show` はインタラクションを起こさない（＝待たない）。`with` はインタラクションを起こし、ユーザーが早送りで打ち切れる。`show x with t` は「前に `with None`、後に `with t`」へ展開される。`with None` は「ここまでを見えない形で確定」して次のトランジションの起点を揃える。
- 【確認】位置: `left`, `right`, `center`, `truecenter`, `topleft`, `top`, `topright`, `offscreenleft/right`, `reset`。
- 【確認】ATL: プロパティ設定、補間（`linear`/`ease`/`easein`/`easeout` + 秒数、スプライン・円運動）, `pause`, `time`, `repeat`, `block`, `parallel`, `choice`（重み付きランダム）, `on show/hide/replace/replaced/...`（イベント）, `event`, `contains`, `function`。プロパティ: `xpos/ypos/xalign/yalign/anchor/zoom/alpha/rotate/crop` など。ATL はスクリプト進行と並行で走る（完了待ちの文はなく、待つなら `pause` を明示）。
- 【確認】トランジション: `dissolve`, `fade`, `pixellate`, `move*`, `ease*`, `zoomin/out`, `vpunch/hpunch`（画面揺れ）, `blinds`, `squares`, `wipe*`, `slide*`, `slideaway*`, `push*`, `iris*`。クラス: `Dissolve`, `Fade`, `ImageDissolve`（ルール画像）, `AlphaDissolve`, `CropMove`, `MoveTransition`, `Pixellate`, `PushMove`, `Swing`, `ComposeTransition`, `MultipleTransition`, `Pause`。辞書トランジションでレイヤ別指定（待たずに次のインタラクションで開始）。
- 【確認】レイヤードイメージ: `layeredimage` 内に `always` / `attribute` / `group`（排他）/ `if` / `default` / `auto`。`show eileen happy glasses` の属性集合からレイヤを合成。
- 【確認】サイド画像: タグ `side` ＋話者の画像タグ＋属性の最長一致で自動選択。
- 【確認】音: 既定チャンネル `music`（ループ）, `sound`, `audio`（多重再生・キュー不可）, `voice`（台詞の進行に合わせ自動停止）。文: `play <ch> file [fadein][fadeout][loop|noloop][if_changed][volume]`, `stop <ch> [fadeout]`, `queue <ch> file`。ファイル名修飾 `<from to loop>`（部分再生）, `<sync ch>`, `<silence N>`, `<volume N>`。カスタムチャンネル登録 `renpy.music.register_channel`。
- 【確認】時間: `pause`（クリック待ち）, `pause 3.0`（秒 or クリック）, `renpy.pause(delay, hard=)`。テキストタグ `{w}`, `{p}`, `{nw}`。
- 【記憶】音声文はスクリプトを止めない（play は即戻る）。

### C. スクリプト↔ホスト契約
Ren'Py はスクリプトエンジンと描画エンジンが同一プロセス・同一言語で、境界は「文 → Python 関数（statement equivalents）→ インタラクション」。
- 【確認】文と同等の Python 関数: `renpy.say(who, what, interact=True)`, `renpy.display_menu(items, interact=True, screen='choice')` → 選ばれた値を返す, `renpy.show/hide/scene`, `renpy.with_statement(trans)` → 中断されたら True, `renpy.jump/call`, `renpy.pause(delay)` → クリックなら True。
- 【確認】「待つ」のは say・menu・pause・with だけ。show/hide/scene/jump/call は待たない。say と menu は `interact=False` で待ちを外せる。
- 【確認】UI 側の受け口は「スクリーン」。`say` スクリーンは who/what を、`choice` スクリーンは項目リストを受ける。履歴は `_history_list` の `HistoryEntry`（`kind`, `who`, `what`, `who_args`, `what_args`, `window_args`, `show_args`, `image_tag`, `voice`, `rollback_identifier`）。
- 【確認】自作インタラクションをロールバック対応にする手順: `renpy.roll_forward_info()` を読む → `ui.interact(roll_forward=...)` → `renpy.checkpoint(rv)`。＝「インタラクションの戻り値」をチェックポイントに記録し、ロールフォワード時に同じ値を再供給する。
- 【確認】キャラクタ単位のコールバック（`callback`）が台詞表示中のイベントで呼ばれる（口パク・ビープ音用途）。

### D. 状態モデル
- 【確認】保存されるもの: 現在の文と戻り先の文、表示中の画像・displayable、スクリーンとその変数、再生中の音楽、NVL のテキスト。Python 側は「ゲーム開始後に変更された store 変数」と、そこから到達可能なオブジェクト。`default` 変数は常に保存。
- 【確認】保存されないもの: 「どう辿ってきたか」（制御経路）— 位置だけを持つので、後から足した文は実行されない。画像名→displayable の対応、config・style。
- 【確認】保存点: 「最外インタラクションコンテキストでの文の先頭」。文の途中でロード／ロールバックされたら**文を最初からやり直す**（Python の `while` で対話するとループごと再実行になるので、Ren'Py スクリプトのループを使えという注意）。
- 【確認】pickle できないもの: Render, イテレータ, **ジェネレータ, コルーチンの task/future**, ファイル, ソケット, 内部関数, lambda。→ pasta（コルーチン実行）と同種の制約を Ren'Py 自身が明記している。
- 【確認】ロールバック: 「ユーザーと対話する各文の先頭で保存し、戻るときに復元」するのと同等。巻き戻るのは init 後に変わった変数と revertable なオブジェクト（スクリプト内で作った list/dict/set・スクリプト定義クラスのインスタンス）。Python 側で作ったデータは対象外（`list(...)` で包む）。`NoRollback` 継承で対象外にできる。
- 【確認】制御: `renpy.block_rollback()`（これより前へ戻れない）, `renpy.fix_rollback()`（戻れるが選択は変えられない）, `renpy.suspend_rollback()`, `renpy.retain_after_load()`（次のチェックポイントまでの変更をロード後も保持）, `config.rollback_enabled`, ソフト／ハードチェックポイント（`renpy.checkpoint(hard=)`）。
- 【確認】ロールバックは基底コンテキストでのみ動く（`renpy.call_in_new_context` 内は無効）。
- 【確認】`after_load` ラベル（ロード後のデータ修復）, `before_load`。
- 【確認】スクリプト改訂と旧セーブ: `call ... from <label>` 節が「戻り先」を安定名にする（公開後に呼び出し元ファイルを編集してもコールスタックが壊れないように。ビルド時の自動付与オプションあり）。クラス・関数は元の名前で残す必要。追加した文は実行されない。リリース間のセーブ互換は保証されない。
- 【確認】履歴の `rollback_identifier` → `RollbackToIdentifier()` でバックログの行へ巻き戻し（ログに残っていれば）。
- 【記憶】内部はロールバックログ（エントリごとに「文の位置・コンテキスト・変更オブジェクトの旧状態・その間に引いた乱数」を持つ）。セーブにはログの一部も入るので、ロード直後も巻き戻せる。文の識別名はファイル名＋行由来で、行がずれると旧セーブの位置が見つからず、見つかる位置までロールバックして復帰する。

### E. システム機能
- 【確認】設定: `preferences.skip_unseen`（未読もスキップするか）, `skip_after_choices`, `afm_enable`/`afm_time`（オート）, `afm_after_click`, `wait_voice`（オートがボイス終了を待つ）, `text_cps`, `transitions`, `voice_sustain`, ミキサー音量 main/music/sfx/voice。
- 【確認】`renpy.choice_for_skipping()`（選択肢直前でスキップ停止＋オートセーブ）, `renpy.is_skipping()`。
- 【確認】履歴（バックログ）: `config.history_length`, `_history_list`, `config.history_callbacks`。ボイス情報を各エントリが持つ。
- 【確認】永続データ: `persistent.x`（未定義は None）、`default persistent.x = ...`、終了時または `renpy.save_persistent()` で保存、複数端末のマージは新しい方優先＋`renpy.register_persistent` で独自マージ、`MultiPersistent`（複数作品間）。
- 【確認】ギャラリー `Gallery`（見た画像・条件で解放）、ミュージックルーム `MusicRoom`（聴いた曲で解放）、リプレイ `Replay(label, scope=, locked=)`＋`renpy.end_replay()`（リプレイ中はセーブ不可、`_in_replay`）。
- 【確認】翻訳: 台詞は「翻訳ユニット」単位。識別子＝「直前のラベル＋内容のハッシュ」（重複は連番）。`translate <lang> <id>:` ブロック。原文を直すと翻訳が外れる → say 文の `id` 節で識別子を固定できる。UI 文字列は `_()` と `translate <lang> strings:`（old/new）。画像・音声は `game/tl/<lang>/` に同じパスで置けば差し替え。
- 【確認】ボイス: `voice "file"`（次の台詞に付く）, `voice sustain`。自動ボイス `config.auto_voice = "...{id}..."` で**翻訳識別子からファイルを探す**。キャラ別ミュート（`voice_tag`）。
- 【確認】テキストタグ: 装飾 `{b}{i}{u}{s}{color}{size}{font}{alpha}{k}{outlinecolor}{plain}{=style}{shader}`、ルビ `{rb}{rt}{art}`（【】| の短縮記法あり）、`{image}{space}{vspace}`、`{a}`（リンク: ラベルへ jump/call 等）、`{cps}`（速度）、読み上げ `{alt}{noalt}`。台詞専用: `{w}`（クリック or 秒待ち）, `{p}`（改段落＋待ち）, `{nw}`（自動送り）, `{fast}`（ここまで即時表示）, `{done}`, `{clear}`。補間 `[var]`（`!t` 翻訳, `!q` タグ無効化 ほか）。
- 【確認】NVL: `Character(kind=nvl)`, `nvl clear`, `nvl show/hide`, `menu = nvl_menu`, `config.nvl_paged_rollback`。

### F. サンプル（自作）
```renpy
define e = Character("エミ", image="emi")
default trust = 0
label start:
    scene bg classroom with fade
    play music "audio/theme.ogg" fadein 1.0
    show emi smile at left with dissolve
    e "おはよう。今日は早いね。"
    e @ surprised "……え、宿題?"
    menu:
        "見せてあげる":
            $ trust += 1
            jump lend
        "自分でやりなよ" if trust < 3:
            jump refuse
```

出典: https://www.renpy.org/doc/html/save_load_rollback.html / displaying_images.html / audio.html / dialogue.html / menus.html / text.html / label.html / statement_equivalents.html / transforms.html / transitions.html / persistent.html / translation.html / voice.html / preferences.html / history.html / other.html / quickstart.html / nvl_mode.html / layeredimage.html / side_image.html / rooms.html

---

## 2. Ink（inkle）

### A. 物語構成要素
- 【確認】内容行はそのまま出力（1 行＝1 段落）。話者の概念は言語にない（`名前: 台詞` は作者とゲーム側の取り決め）。
- 【確認】ノット `=== name ===`、スティッチ `= name`（`knot.stitch`）、ダイバート `-> target`（行の途中でも可）、グルー `<>`（改行抑止）、`INCLUDE`。
- 【確認】選択肢 `*`（既定で**一度きり**）、`+`（スティッキー＝使っても残る）、テキスト無しの `* ->`（フォールバック: 他に選べるものが無いとき自動選択）。`[ ]` で「選択肢表示のみ／出力のみ」を切り分け。条件 `* {cond} [text]`（複数は AND）。
- 【確認】ウィーブ: ギャザー `-` で分岐を合流。入れ子は記号を重ねる（`* *`, `- -`）。ラベル `(label)` を選択肢・ギャザーに付けると、ダイバート先にも読了判定にも使える。
- 【確認】訪問回数: ノット／スティッチ／ラベル名をそのまま整数として参照（`{knot}`, `{knot > 3}`）。`TURNS()`, `TURNS_SINCE(-> x)`（未訪問は -1）, `CHOICE_COUNT()`。
- 【確認】変数 `VAR`（グローバル）、`~ temp x`（一時）、`CONST`、`LIST`（列挙＋集合＋状態機械）。式と論理演算（`and/or/not`）。条件テキスト `{cond: a|b}`。
- 【確認】バリエーション: `{a|b|c}`（シーケンス＝最後で止まる）, `{&a|b}`（サイクル）, `{!a|b}`（一度きり）, `{~a|b}`（シャッフル）。【記憶】複数行版 `{stopping:}`, `{cycle:}`, `{shuffle:}`, `{once:}`, `{shuffle once:}`。
- 【確認】関数 `=== function f(x) ===`（`~ return`, `ref` 引数）。トンネル `-> knot ->` と `->->`（コールスタックを使うサブルーチン）。スレッド `<- knot`（複数箇所の選択肢を 1 つの選択リストへ合流。選ぶと他は畳まれる。グローバル変数は分岐しない）。`-> DONE`（その流れの終了）と `-> END`（物語全体の終了）。
- 【確認】`SEED_RANDOM(n)`。【記憶】`RANDOM(min, max)`。

### B. 演出指示
- 言語に演出語彙は**無い**。手段は 3 つ（【確認】RunningYourInk が「外部関数の代わりに、変数オブザーバ・タグ・テキスト規約も使える」と明記）:
  1. タグ `# ...`（行の末尾または直前の行に書く。行タグ／ノット先頭のタグ／ファイル先頭のグローバルタグ／選択肢のタグ／動的タグ＝タグ内にインライン ink）
  2. 外部関数 `EXTERNAL f(args)` → `~ f(...)`
  3. テキスト規約（例 `>>> SHOW emi` のような行をゲーム側が解釈）
- タグの中身（`bg: classroom` 等）の文法は完全に作者とゲームの取り決め。タイミング・待ちの概念もホスト任せ。

### C. スクリプト↔ホスト契約（最重要）
ランタイムはホストから**引かれる（pull）**ライブラリ。
- 【確認】ループ:
  1. `new Story(json)`
  2. `while (story.canContinue) { text = story.Continue(); tags = story.currentTags; ... }`（1 行ずつ。`ContinueMaximally()` は次の選択肢／終端まで一括）
  3. 行が尽きたら `story.currentChoices`（各 `Choice` は `text` と `tags`。【記憶】`index` も）
  4. `story.ChooseChoiceIndex(i)` → 2 へ
- 【確認】ホスト→ランタイムの他の入口: `ChoosePathString("knot.stitch")`（任意位置へ飛ぶ。ラベルは不可）、`variablesState["x"]` の読み書き、`ObserveVariable(name, cb)`、ink 関数の呼び出し（ページ上の表記は `EvaluationFunction`。【記憶】実 API 名は `EvaluateFunction`）、`state.VisitCountAtPathString(path)`、`TagsForContentAtPath(knot)`, `globalTags`。
- 【確認】外部関数: `BindExternalFunction(name, lambda, lookaheadSafe)`。引数・戻り値は int/float/bool/string。**同期呼び出しで即戻る**（待ちの概念なし）。
- 【確認】先読み問題: エンジンはグルー判定のために次の行を先読みする。副作用のある外部関数（音を鳴らす等）は `lookaheadSafe=false`（既定）にしておくと、先読みはその手前で止まる（その代わりグルーが切れる）。純関数は `true` にして何度呼ばれてもよい前提にする。→ **「スクリプトの評価位置」と「プレイヤーが見ている位置」がずれる**ことをランタイム自身が認めている。
- 【確認】フォールバック: `EXTERNAL` が未バインドなら同名の ink 関数を実行（エディタ Inky やホスト無しのテスト用）。
- 【確認】エラー: `onError(msg, type)`（Warning と Error）。
- 【確認】並行フロー（beta）: `SwitchFlow(name)`, `SwitchToDefaultFlow()`, `RemoveFlow(name)`, `aliveFlowNames`。1 つの Story で変数を共有しつつ複数の進行位置を持つ。
- ホストに渡る 1 行の中身＝「テキスト（補間・グルー解決済み）＋タグの文字列リスト」だけ。**行 ID は無い**（ローカライズ・ボイスの鍵は作者がタグで付ける）。

### D. 状態モデル
- 【確認】`story.state.ToJson()` / `story.state.LoadJson(json)` で**任意の時点**の状態を丸ごと往復。
- 【確認・ソース】`StoryState` の保存キー: `flows`（フローごとのコールスタック・スレッド・出力ストリーム・現在の選択肢）, `currentFlowName`, `variablesState`, `evalStack`, `currentDivertTarget`, `visitCounts`, `turnIndices`, `turnIdx`, `storySeed`, `previousRandom`, `inkSaveVersion`（現在 10、8 未満は拒否）, `inkFormatVersion`（未使用）。
- 【確認・ソース】訪問回数はコンテナの**パス文字列**がキー。乱数は `storySeed` と `previousRandom` を状態に持つ → ロードしても同じ列が続く（シャッフルも決定的）。
- 【記憶】ポインタ（実行位置）も「コンテナパス＋インデックス」。コルーチンではなく、コンパイル済み JSON（コンテナ木）を歩くインタプリタなので状態が素直に直列化できる。
- ロールバック機能は組込みに無い。ホストが `ToJson()` のスナップショットを積めば実現できる（状態が純データなので安い）。
- 【二次】スクリプト改訂と旧セーブ: 選択肢・ギャザーの位置は連番的に振られるので、足し引きで旧セーブとずれる（inky issue #253）。`ChoosePathString` は存在しないパスに対し上位へ遡って見つかる内容を探し、ランタイム警告を出す。追加された変数は宣言時の初期値で補う（0.8.1 リリースノート）。「完全にはできないので、本番公開後の編集は注意」とされる。
- 【確認・ソース】ロード時に「保存されたダイバート先が現存するか」の検査は無い。

### E. システム機能
- スキップ・オート・バックログ・ギャラリー・ボイス・ローカライズは**すべてホストの責務**。ink が提供するのは訪問回数（既読判定の材料）と完全直列化可能な状態だけ。
- 【記憶】ローカライズは公式には「言語ごとに ink を分ける」か、タグで行キーを振って外部テーブルを引く運用。

### F. サンプル（自作）
```ink
VAR trust = 0
-> classroom
=== classroom ===
# bg: classroom
# bgm: theme fadein=1
# show: emi smile left
エミ: おはよう。今日は早いね。 # voice: emi_001
エミ: ……え、宿題? # expr: emi surprised
* [見せてあげる]
    ~ trust += 1
    -> lend
* {trust < 3} [自分でやりなよ] -> refuse
```

出典: https://raw.githubusercontent.com/inkle/ink/master/Documentation/RunningYourInk.md / WritingWithInk.md / ArchitectureAndDevOverview.md / https://raw.githubusercontent.com/inkle/ink/master/ink-engine-runtime/StoryState.cs / https://github.com/inkle/inky/issues/253 / https://github.com/inkle/ink/releases/tag/0.8.1

---

## 3. Yarn Spinner

### A. 物語構成要素
- 【確認】ノード: ヘッダ（`title:` 必須、ほか `tags:`, `when:`, `tracking:`, エディタ用の `color:` `group:` 等）＋ `---` 本文 `===`。本文は行・コマンド・オプション。
- 【確認】行: 1 行ずつゲームへ送る。行頭の「空白を含まない文字列＋コロン」が話者名（マークアップの `character` 属性 `name` プロパティとして付く）。
- 【確認】オプション `-> text`。連続したオプションが 1 つの選択群。配下のインデント行は選ばれたときだけ実行。入れ子可。
- 【確認】条件付きオプション `-> text <<if expr>>`: 偽なら**「利用不可」の印を付けて全オプションをゲームへ送る**（灰色表示にするか隠すかはゲームが決める）。全部利用不可のときはゲームが「選択なし」を返せて、オプション群を飛ばして続行（Dialogue Runner の Allow Option Fallthrough）。
- 【確認】変数 `$name`（number / string / bool。型は変わらない）、`<<declare $x = 0>>`, `<<set $x to 1>>`。式・演算子（`eq/is/==`, `and/or/xor/not` ほか）。行内補間 `{$x}`。
- 【確認】条件 `<<if>> <<elseif>> <<else>> <<endif>>`。
- 【確認】`<<jump Node>>`、`<<detour Node>>`＋`<<return>>`（呼び出して戻る。入れ子可。**detour 中に jump すると戻りスタックが消える**）、`<<stop>>`。
- 【確認】一度きり: `<<once>>...<<endonce>>`（`<<once if cond>>`, `<<else>>` 可）、行末 `<<once>>`、オプション末 `<<once>>`。既読状態は Variable Storage 内の（スクリプトから見えない）変数に入る。
- 【確認】ライングループ `=> line`（条件・once 付き、候補から 1 つ選ぶ。バーク向き）、ノードグループ（`when:` ヘッダ。`when: once`, `when: always`, `when: <式>`）。選び方＝サリエンシー戦略: First / Best / Best Least Recently Viewed / Random Best Least Recently Viewed（既定）。該当なしなら何も実行しない。
- 【確認】組込み関数: `visited(node)`, `visited_count(node)`, `random()`, `random_range(a,b)`, `dice(n)`, `min/max/round/round_places/floor/ceil/inc/dec/decimal/int`, `format_invariant`, 型変換 `string()/number()/bool()`。カスタム関数は純関数であること（ゲームへの指示はコマンドで）。

### B. 演出指示
- 【確認】コマンド `<<name args...>>`。組込みは `wait`（秒）と `stop` だけ。それ以外（`<<fade_out 1.5>>` 等）は**すべてゲーム側が定義**。語彙は言語に無い。
- 【確認】行タグ `#tag`（空白不可・複数可）。組込みタグ `#line:<id>`（行 ID）、`#lastline`（直後がオプションの行にコンパイラが付与）。その他は自由（例 `#tone:sarcastic`）。
- 【確認】マークアップ `[a]..[/a]`, `[a/]`, `[a=1 b=2]`, `[/]`, `[nomarkup]`。テキストからは剥がされ、「属性名・位置・長さ・プロパティ」の列としてゲームに届く。置換マーカー `select` / `plural` / `ordinal`。→ インラインの待ち・速度・強調・イベントはマークアップ属性で表現する設計。

### C. スクリプト↔ホスト契約（最重要）
2 層ある。

**(1) コア VM: `Yarn.Dialogue`**（エンジン非依存）【確認】
- ホストが設定するハンドラ: `LineHandler`, `OptionsHandler`, `CommandHandler`, `NodeStartHandler`, `NodeCompleteHandler`, `DialogueCompleteHandler`, `PrepareForLinesHandler`（これから出る行の予告＝アセット先読み用）。
- ホストが呼ぶ: `SetProgram`, `SetNode(name)`, `Continue()`, `SetSelectedOption(int)`, `Stop()`, `NodeExists`, `GetTagsForNode`, `VariableStorage`（差し替え可能な変数置き場）, `Library`（関数）, `ContentSaliencyStrategy`。
- プロトコル: `Continue()` は命令を回し、(a) 行またはコマンドを渡したら**次の `Continue()` まで待つ**（ハンドラ内から呼んでも、後から呼んでもよい）、(b) オプションを渡したら `SetSelectedOption` → `Continue()` を待つ、(c) 終端なら `SetNode` を待つ。実行中の再入 `Continue()` は無視。
- 【確認】コアが渡す行は **行 ID**（＋【記憶】置換値）で、表示文字列そのものではない。文字列・音声は Line Provider が行 ID から引く。

**(2) Unity 層: Dialogue Runner + Dialogue Presenter（旧 Dialogue View）**【確認】
- Runner は Yarn Project / Variable Storage / Line Provider / Presenter 群を束ね、行・オプション・コマンドを配る。イベント: On Node Start / Complete, On Dialogue Start / Complete, On Unhandled Command。
- Presenter（`DialoguePresenterBase` 派生）:
  - `RunLineAsync(LocalizedLine, LineCancellationToken)` — 「出し終えた」と判断したら戻る。プレイヤーの送りを待ちたければ戻らなければよい。**Runner は全 Presenter の完了を待ってから次へ**。
  - `RunOptionsAsync(DialogueOption[], CancellationToken)` — 選ばれた `DialogueOption` を返す。扱わない Presenter は null。最初の非 null を採用し、他はキャンセル。誰も返さなければ停止したまま。
  - `LineCancellationToken` は 2 段: 「急いで（`IsHurryUpRequested`）」と「次の行へ（`IsNextLineRequested`）」。次行要求は急ぎ要求も立てる（逆は立たない）。入力側は Runner の `RequestHurryUpLine()` / `RequestNextLine()` を呼ぶ。
  - `LocalizedLine` はロケール解決済みテキスト（マークアップ解析結果）と `Metadata`（行タグ）。【記憶】`TextID`, `CharacterName`, `TextWithoutCharacterName`, `Asset`（音声等）, `DialogueOption.IsAvailable`。
- コマンド: `[YarnCommand("name")]`（インスタンスメソッドなら第 1 引数が GameObject 名）、static メソッド、`AddCommandHandler(name, delegate)`。引数は string/int/float/bool/GameObject/Component へ自動変換、省略可能引数あり。
  - **待つかどうかはハンドラの戻り値型で決まる**: `IEnumerator`（コルーチン）／`Coroutine`／`Task`・`YarnTask`・Awaitable・UniTask を返せば完了まで対話が止まる。【記憶・ページは暗示のみ】`void` なら即続行。
  - 未処理コマンドは On Unhandled Command へ。
- 関数: `[YarnFunction]` / `AddFunction`。static・値を返す・純関数。

### D. 状態モデル
- 【確認】保存の公式手段は **Variable Storage の中身だけ**（`SaveStateToPersistentStorage` が変数を JSON で書く）。once・visited の状態も変数置き場に入るので一緒に残る。
- 【二次・未確証】VM の実行位置（ノード内の命令位置・detour スタック）を保存する公式 API は確認できず。会話の途中再開は「ノード名を別途保存してノード先頭からやり直す」のが通例。
- ロールバック機能は無い。乱数（`random`, `dice`）と保存・巻き戻しの整合は規定なし【記憶】。
- スクリプト改訂: 位置を保存しない設計なので、壊れるのは変数名・ノード名の変更だけ。行 ID が安定キー。

### E. システム機能
- 【確認】行 ID: `#line:` で明示、無ければコンパイラが内部 ID を付ける。エディタで未付与行へ自動付与（ランダム 16 進 or 「ノード名＋連番＋話者」の記述的 ID、Unity ではカスタム生成器 `ILineTagGenerator`）。重複はエラー。**翻訳・ボイス・翻訳音声・アニメなど 1 行に紐づく全アセットを行 ID で束ねる**。
- 【確認】Line Provider（組込みローカライズ／Unity Localization）、Voice Over 用 Presenter【記憶】。
- 【確認】`#lastline` で「選択肢の上に直前の台詞を出したまま」にできる。メタデータは CSV 書き出しで翻訳者向け情報にもなる。
- スキップ／オート／バックログは Presenter 側の実装事項（言語は関与しない）。

### F. サンプル（自作）
```yarn
title: Classroom
---
<<declare $trust = 0>>
<<scene classroom fade>>
<<music theme 1.0>>
<<show emi smile left>>
Emi: おはよう。今日は早いね。 #line:cls_001
Emi: ……え、宿題? #line:cls_002 #expr:surprised
-> 見せてあげる
    <<set $trust to $trust + 1>>
    <<jump Lend>>
-> 自分でやりなよ <<if $trust < 3>>
    <<jump Refuse>>
===
```
（`scene` / `music` / `show` はホストが登録するコマンド。Yarn 自体は意味を知らない。）

出典: https://docs.yarnspinner.dev/write-yarn-scripts/scripting-fundamentals/ の commands / options / lines-nodes-and-options / flow-control / logic-and-variables / jumps / detour / once / line-groups / functions、https://docs.yarnspinner.dev/write-yarn-scripts/advanced-scripting/markup / tags-metadata、https://docs.yarnspinner.dev/3.1/write-yarn-scripts/advanced-scripting/saliency、https://docs.yarnspinner.dev/yarn-spinner-for-unity/components/dialogue-runner、https://docs.yarnspinner.dev/components/dialogue-view、https://docs.yarnspinner.dev/components/dialogue-view/custom-dialogue-views、https://docs.yarnspinner.dev/yarn-spinner-for-unity/creating-commands-functions、https://docs.yarnspinner.dev/yarn-spinner-for-unity/assets-and-localization/line-tagging、https://docs.yarnspinner.dev/api/csharp/yarn/yarn.dialogue、https://docs.yarnspinner.dev/api/csharp/yarn/yarn.dialogue/yarn.dialogue.continue、https://docs.yarnspinner.dev/2.5/api/csharp/yarn.unity/yarn.unity.dialoguerunner/yarn.unity.dialoguerunner.savestatetopersistentstorage

---

## 4. Naninovel（Unity）

### A. 物語構成要素
- 【確認】行種は先頭記号で決まる: `@`＝コマンド行、`#`＝ラベル、`;`＝コメント、記号なし＝汎用テキスト行（`Author: text`、`Author.Happy: text` は `@char` の省略形を兼ねる）。
- 【確認】テキスト行内のインラインコマンド `[cmd ...]`。`[>]`（行末の入力待ちを飛ばす）、`[-]`（入力待ち）。
- 【確認】コマンド引数: 無名引数 1 つ（先頭）＋ `name:value`。真偽は `flag!` / `!flag`。値型: 文字列・整数・小数・真偽・named（`key.value`）・リスト。
- 【確認】式 `{...}`（実行時評価）。`@set`, `@if/@else/@unless`（インデント 4 スペースの入れ子ブロック）、全コマンド共通の `if:` / `unless:`、インライン `[if ...][else][endif]`、`@while`, `@group`, `@random`（配下から 1 つ。重み可）。
- 【確認】フロー: `@goto Script#Label`, `@gosub` / `@return`, `@stop`, `@title`。
- 【確認】選択肢 `@choice "text" goto:... `、または配下の入れ子行が選択時コールバック。`lock:`（条件で無効表示）, `handler:`（ButtonList / ButtonArea＝自由配置 / ChatReply）, `button:`。`@choice` は「選ばれるまで止まる」、`@addChoice` は止まらない版。
- 【確認】式関数: `random(min,max)` / `random(文字列...)`, `calculateProgress()`（既読率）, `isUnlocked(id)`, `hasPlayed()`（このコマンドが既に再生済みか）/ `hasPlayed(scriptPath)`, `getName(id)`, 数学関数。C# の `ExpressionFunction` 属性で追加。
- 【記憶】カスタム変数はローカル（セーブスロット別）と `g_` 接頭辞のグローバル（全スロット共通）。

### B. 演出指示（＝コマンド語彙が言語に同梱）
- 【確認】アクター: キャラクタ・背景・テキストプリンタ・選択肢ハンドラ。
- 【確認】`@back appearance[.Transition] id pos tint visible time easing wait`、`@char Id.Appearance look pos pose avatar tint visible time wait`、`@hide`, `@hideAll`, `@hideChars`, `@arrange`（横並び自動配置）, `@slide`, `@shake`, `@camera`（offset/zoom/roll/rotation）, `@spawn`/`@despawn`（エフェクト）, `@trans`（配下の変更をトランジションで覆う）, `@movie`, `@showUI`/`@hideUI`, `@toast`, `@input`（文字入力）。
- 【確認】音: `@bgm path intro volume loop fade group time`, `@stopBgm`, `@sfx`（状態に保存）, `@sfxFast`（低遅延・保存なし）, `@stopSfx`, `@voice`, `@stopVoice`。
- 【確認】テキスト: `@print`, `@printer`（プリンタ切替: Dialogue / Wide / Fullscreen＝NVL / Chat / Bubble）, `@resetText`, `@append`, `@format`。リッチテキスト `<b>`, `<color>`, `<ruby="...">`, イベントタグ `<@...>`（その位置が表示されたら発火）, `<:...>`（言語切替で再評価）, select タグ。
- 【確認】時間: `@wait`（秒 / 入力 `i` / スキップ可能タイマ `i` + 秒）。共通の `time:`（所要時間）と `lazy`。
- 【確認】**コマンドは既定で待たない**（すぐ次の行へ）。`wait!` を付けると完了まで待つ。`@await`（配下を並行実行して全完了を待つ／名前付きタスクを待つ）、`@async [name] [loop!]`（並行トラック）、`@stop name`（中止）/ `complete!`、`@sync`。設定 `Complete On Continue`（待ち中に送りを押すと即完了）。旧来の「既定で待つ」は後方互換オプション（廃止予定）。

### C. スクリプト↔ホスト契約
- 同一プロセスの Unity サービス群。【確認】`Engine.GetService<T>()`、`IScriptPlayer`（`MainTrack.LoadAndPlay`, `LoadAndPlayAtLabel`, `ExecuteTransientCommand`）, `IStateManager`, `IInputManager`, `ICameraManager`。
- 【確認】カスタムコマンド: `Command` 派生クラスで `Execute(ExecutionContext)` を `Awaitable` として実装、`[Alias("...")]`、型付き引数（`StringParameter` 等、`Assigned()`）。
- 【確認】ゲーム側との切替: サンプルの `@adventure` / `novel` コマンドで「入力・カメラ・スクリプト再生」を受け渡す。
- つまり契約は「**名前付きコマンド＋型付き名前引数＋ awaitable**」。待つ／待たないはスクリプト側（`wait!`）で決め、ハンドラは常に awaitable を返す。

### D. 状態モデル
- 【確認】3 区分: ゲーム状態（スロット別: 再生中スクリプトとコマンド位置・表示中のキャラと位置・BGM と音量など）／グローバル状態（全スロット共通: 既読コマンドの記録など）／ユーザー設定（人が編集できる形式）。
- 【確認】`IStateManager.SaveGame/LoadGame/QuickSave/QuickLoad`。カスタム状態は `AddOnGameSerializeTask` / `AddOnGameDeserializeTask` と `GameStateMap.SetState<T>/GetState<T>`。「ロールバックはカスタム状態でもそのまま動く」。
- 【確認・設定項目から】ロールバック: `Enable State Rollback`、`State Rollback Steps`（メモリ上に持つ**状態スナップショット**数。既定 1024）、`Saved Rollback Steps`（セーブスロットへ入れる数。既定 128＝ロード後も巻き戻せる）、`@purgeRollback`（これより前へ戻れなくする）。
- 【確認】`Recovery Rollback`: **セーブ後にスクリプトが変更されていたら、ロード時にそのスクリプトの先頭までロールバック**する。
- 【確認】安定参照: スクリプトは GUID ベースの参照、テキストは `|#id|` の安定 ID。
- 【記憶】スナップショットは「プレイヤー入力待ちの地点」で取られる。位置は「スクリプトパス＋行インデックス＋インライン位置」。乱数とロールバックの扱いは未確認。

### E. システム機能
- 【確認】スキップ: 既定は既読のみ（`Default Skip Mode: Read Only`）、`Skip Time Scale`。`@skip`。オート: 文字数と表示速度から待ち時間、`Min Auto Play Delay`。
- 【確認】バックログ: 読み返し・選択肢の確認・**ボイス再生**・任意でそこへロールバック。
- 【確認】アンロック: `@lock` / `@unlock`, `isUnlocked()`（CG ギャラリー等）。
- 【確認】ローカライズ: ツールがスクリプトごとの翻訳文書を生成（`# ID` 行＋原文コメント＋訳）。インラインコマンドで分断された断片を `|` で 1 行に結合可。スプレッドシート連携。リソースは `Localization/<locale>/` に同じパスで置けば差し替え。テキスト ID `|#id|` が翻訳と自動ボイスの紐付けを保つ。
- 【確認】ボイス: `@voice`（単発）、自動ボイス（テキスト ID で紐付け。Voice Map か「スクリプトパス＋テキスト ID」のアドレス）、`Voice Overlap Policy`、キャラ別音量、音声言語を文字言語と別に選択、収録用台本の書き出し。

### F. サンプル（自作・引数の細部は例示）
```nani
@back Classroom.Fade
@bgm Theme fade:1
@char Emi.Smile pos:25 wait!
Emi: おはよう。今日は早いね。
Emi.Surprised: ……え、宿題?
@choice "見せてあげる"
    @set trust=trust+1
    @goto #Lend
@choice "自分でやりなよ" if:trust<3
    @goto #Refuse
```

出典: https://naninovel.com/guide/scenario-scripting / state-management / configuration / localization / voicing / text-printers / integration-options / choices / script-expressions、https://naninovel.com/api/

---

## 5. その他（特徴だけ）

### Twine（Harlowe / SugarCube）
- 単位は「パッセージ」とリンク。1 ターン＝1 パッセージ遷移。ハイパーテキスト寄りで、行単位の送りという概念が薄い。
- SugarCube【確認】: `[[text|passage]]`（setter 付きリンク）、`$x`（履歴に残る物語変数）と `_x`（そのターン限り）、`<<set>> <<if>> <<include>> <<type>>`（タイプライタ表示）。**物語変数には循環参照・関数を入れられない（履歴の持ち方の制約）**。`visited()`。
- SugarCube【二次】: 履歴は「モーメント」の列で `Config.history.maxStates` で深さを制限（1 で戻る不可）。シード付き PRNG `State.prng.init()` を有効にすると、乱数が履歴（セーブ含む）と統合され、戻ってやり直しても同じ値になる（`Math.random()` 直呼びは対象外）。ページ再読込では再シードされるとの報告あり。
- Harlowe【確認】: マクロをフック `[...]` に付ける「チェンジャー」方式、`(set:)`, `(if:)`, `(link:)`, `(go-to:)`, `(display:)`, `(either:)`, `(history:)`, `(visited:)`, `visits`, `(save-game:)/(load-game:)`, `(undo:)`, `(live:)`, `(transition:)`, `(cycling-link:)`, `(seq-link:)`, ストーリーレット `(storylet:)` `(open-storylets:)`, `(seed:)`。
- 示唆: **「ターン＝パッセージ」単位の状態スナップショット履歴**が undo とセーブの両方の土台。位置は「パッセージ名」という安定名だけなので、スクリプト改訂に比較的強い。

### Godot Dialogic 2
- 【確認】タイムライン＝イベント列。ビジュアルエディタとテキスト形式（`.dtl`）が同じデータ。
- 【確認】テキスト構文: `Name (expression): text`、`join Name (portrait) position [animation="..."]` / `leave` / `update`、選択肢 `- text | [if {Var} > 10]`、`if {Var} > 3:` / `elif` / `else:`、`set {Var} += 10`、`label Name` / `jump Timeline/Label` / `return`、`do Autoload.method(arg)`、ショートコードイベント `[background arg="..."]` `[end_timeline]`、行内 `[pause=0.5]` `[portrait=confused]`。
- 【確認】拡張: カスタムイベントを追加できる。CSV 翻訳、用語集（ホバーで説明）。
- 【記憶】`[signal arg="..."]` でゲーム側へ通知（Godot シグナル）、`[wait]`, `[music]`, `[sound]`, `[voice]`、履歴・セーブはサブシステム（`Dialogic.Save` 等）。
- 示唆: **「台詞の頭に join/leave/update という立ち絵専用の動詞」**＋それ以外は汎用ショートコード `[name key="value"]` という二段構え。

### Monogatari（Web）
- 【確認】スクリプトは JS のデータ（ラベル → 文の配列）。文は文字列（`show scene <bg> with <anim>[, ...] [duration t]`、台詞、`jump ...` など）かオブジェクト（Choice, Conditional, Function）。`show scene` は背景を替え、キャラ・画像・テキストボックスを消す。
- 【確認】アクション一覧: Choices, Clear, Conditionals, Dialogs, End, Gallery, Show/Hide Canvas, Show/Hide Character, Character Layer, Show/Hide Image, Show/Hide Particles, Show/Hide Video, Input, Functions, Jump, Next, Pause, Placeholder, Play/Stop Music・Sound・Voice, Preload, Unload, Show Background, Show Message, Show Notification, Show Scene, Vibrate, Wait。
- 【確認】**ロールバック＝逆操作**: 各アクションは Apply サイクル（`willApply` → `apply` → `didApply`、戻り値 `advance` が「自動で次へ／入力待ち」）と Revert サイクル（`willRevert` → `revert` → `didRevert`、`advance` と `step`）を持つ。戻るときは適用済みアクションを 1 つずつ revert。ロード時は `onLoad` で保存状態から見た目・音を再適用。
- 【確認】スクリプト内の素の関数は「戻せない」ので、`{'Function': {Apply, Revert}}` の対で書く（さもないと戻って進むたびに加点が重複する、という公式の注意）。`true` を返すと即次へ、`false`／無返却は入力待ち、Promise を返すと解決まで待つ。
- 示唆: スナップショットを取らずに undo する唯一の例。**作者が逆操作を書く負担**と、書き忘れによる不整合が欠点。

### Fungus（Unity）
- 【確認・wiki 目次のみ】フローチャート／ブロック／コマンド（flow, variable, narrative, audio, sprite, scene, camera, UI）／イベントハンドラ／FungusLua。
- 【確認】セーブ: **Save Point コマンドを通過した時点だけ**が保存点。Save History に「シーン・実行位置・フローチャート変数」を積み、Rewind / Fast Forward で行き来（巻き戻して進むと先の履歴は破棄）。ロード後は Save Point の直後から再開し、`Save Point Loaded` イベントハンドラでカメラや音楽を**作者が再セットアップ**する。保存される変数は Boolean / Integer / Float / String のみ。キーは既定でブロック名。
- 【確認】テキストタグ: `{b}{i}{color=}{size=}`、`{s=}`（速度）、`{w}`/`{w=}`（秒待ち）、`{wi}`（入力待ち）、`{wc}`（入力待ち＋クリア）、`{wvo}`（ボイス終了待ち）、`{wp=}`（句読点で間）、`{c}`（クリア）、`{x}`（入力を待たず次へ）、`{vpunch=}{hpunch=}{punch=}{flash=}`（画面効果）、`{audio=}{audioloop=}{audiopause=}{audiostop=}`、`{m=Message}`（メッセージ送出）、`{$Var}`。
- 示唆: 「チェックポイント＋ロード時フック」方式の典型。テキストタグに画面効果と音まで入れている。

### Narrat（Web）
- 【確認】ラベル `main:` ＋インデント。`talk player idle "text"` / 裸の `"text"`、`choice:` ブロック、`jump label`、`run label args`（関数呼び出し）＋`return value`、`set data.x 1` / `$data.x`、`var`（ラベル内ローカル）、前置記法の式 `(+ 2 3)`、`if (cond):`、補間 `%{...}`。
- 【確認】RPG 機能同梱: スキルとダイス判定、インベントリ、クエスト、実績。プラグインでコマンド・UI・セーブデータを追加。
- 【確認】セーブ: **ラベルへ jump した時点で自動的にセーブデータを生成**。理由は公式が明言 —「行番号で保存するとスクリプト更新で別の台詞を指してしまう。ラベルの途中まで再生し直すのは許容する代償」。`data`（スロット別）と `global`（全スロット共通）。
- 示唆: **「ラベル境界チェックポイント」を、スクリプト改訂耐性のために意図的に選んだ**例。

出典: https://www.motoslave.net/sugarcube/2/docs/ / https://twine2.neocities.org/ / https://intfiction.org/t/question-about-randomness/56535 / https://docs.dialogic.pro/ / https://docs.dialogic.pro/timeline-text-syntax.html / https://raw.githubusercontent.com/Monogatari/Documentation/master/SUMMARY.md（＋ script-actions/javascript.md, building-blocks/actions/life-cycle.md, script-actions/show-scene.md）/ https://github.com/snozbot/fungus/wiki/save_system / https://github.com/snozbot/fungus/wiki/narrative_text_tags / https://docs.narrat.dev/ / https://docs.narrat.dev/scripting/language-syntax.html / https://docs.narrat.dev/features/save-and-load.html

---

## 6. 統合 1 — どのシステムにもある共通の核（最低ライン）

1. **話者付きの行**と地の文。話者は安定 ID（表示名は別に引ける／動的に変えられる）。
2. **選択肢**: 条件付き、一度きり（Ink は既定、Ren'Py は `set`、Yarn は `<<once>>`）、「条件不成立を隠す／無効表示する」の区別（Yarn・Ren'Py・Naninovel は無効表示をホストへ委ねられる）、全滅時の挙動（Ink＝フォールバック選択肢、Ren'Py＝menu を飛ばす、Yarn＝フォールスルー）。
3. **名前付きの飛び先**（ラベル／ノット／ノード）と jump。
4. **呼び出して戻る**（call/return, tunnel, detour, gosub, run）。
5. **変数・式・条件分岐**。型は数値／文字列／真偽が最低ライン。スコープは「セーブスロット別」「一時」「全プレイ共通（persistent/global）」の 3 段が事実上の標準。
6. **既読・訪問の記録**（ラベル単位の訪問回数、行単位の既読）。スキップ（既読のみ）・ギャラリー解放・一度きり・バリエーション選択の共通の土台。
7. **バリエーション**（シーケンス／サイクル／シャッフル／一度きり）。Ink が最も体系的、Yarn はライングループ＋サリエンシー、Ren'Py と Naninovel は乱数関数。
8. **テキスト内の補間とインライン制御**（変数埋め込み、待ち、速度、ルビ、装飾）。
9. **演出の要求手段**が何かしらある（言語組込み／コマンド／タグ）。
10. **保存できる位置の定義**がある（任意の文頭／任意の行／チェックポイントのみ、のどれか）。

「核ではないがノベルゲームを名乗るなら要る」もの: ロールバック、バックログ、オート、ボイスと行 ID、ローカライズ、NVL。これらは Ren'Py・Naninovel だけが言語と一体で持ち、Ink・Yarn はホスト任せ。

## 7. 統合 2 — ホスト契約の設計パターン比較

| 観点 | (a) 行に付くタグ／メタデータ（Ink） | (b) 名前付きコマンド＋引数、任意で待つ（Yarn / Naninovel） | (c) 演出文が言語組込み（Ren'Py） |
|---|---|---|---|
| ランタイムが渡すもの | 行テキスト＋文字列タグの配列 | コマンド名＋引数（Yarn は文字列列、Naninovel は型付き名前引数）と、行（行 ID＋メタデータ＋マークアップ属性） | 文が直接エンジン API を呼ぶ |
| 待ちの決定者 | ホスト（タグの解釈次第） | Yarn＝ハンドラの戻り値型、Naninovel＝スクリプトの `wait!` | 文の種類で固定（`with`・say・menu・pause が待つ） |
| 実行順とタイミング | タグは「その行に属する」だけ。行より前か後か同時かは規約 | スクリプト上の順序どおり逐次。並行は明示（`@async`/`@await`） | 逐次＋トランジションの一括確定（`with`） |
| 静的検査 | 不可（文字列） | コマンド表があれば可（Naninovel は IDE 支援あり、Yarn は実行時に未処理イベント） | コンパイラと lint が全部知っている |
| ホスト無しでのテスト | 容易（タグは無害。外部関数は ink 側フォールバック） | 未処理コマンドを無視／ログにすれば可 | エンジンごと必要 |
| スクリプトとホストの独立性 | 最高（語彙はゼロ） | 中（コマンド表が契約。版管理が必要） | 最低（一体） |
| セーブ／ロールバックとの整合 | ホストが自分で画面状態を保存・復元する必要 | 同左（Naninovel は一体型なのでエンジンが保存） | エンジンが「表示中の画像・音」を状態として保存 |
| 弱点 | 構文もスペルも検査されない。副作用の順序が曖昧。先読みで評価位置がずれる（`lookaheadSafe`） | 語彙の版ずれ。待つ／待たないの既定を誤ると全体がもたつく or 取りこぼす | 描画を別プロジェクトにできない。語彙が巨大 |

スクリプトエンジンと描画が**別プロジェクト**の場合の読み:
- (c) はそのままでは採れない。ただし「(c) の見た目の構文を、(b) のコマンド送出にコンパイルする」ことはでき、作者体験は (c)、境界は (b) になる。Naninovel の `Author.Happy: text`（＝`@char` の省略形）と Dialogic の `join/leave/update` がこの折衷の実例。
- (b) を境界に据えるなら、最低限決めること: コマンド名と引数の型／未知コマンドの扱い（無視・警告・エラー）／**待ちの既定**（Naninovel は「待たない」を既定に変えた。Yarn は型で決まる）／完了通知の形／**早送り（hurry-up）と打ち切り（next）の 2 段キャンセル**（Yarn の `LineCancellationToken`、Naninovel の `Complete On Continue`、Ren'Py の「`with` はクリックで打ち切れる」が同じ要求）。
- (a) は「その行に属する補助情報」（表情、声色、ボイスキー、レイアウト指定）に向く。行と独立した時間軸の演出（背景切替・BGM）には向かない。Yarn は両方を持ち、行タグ `#...`＝行の属性、コマンド＝独立した演出、マークアップ＝行内の位置付き属性、と**3 つを使い分けている**。これが最も整理された形。
- どの方式でも、**ホスト側が「現在の画面と音の状態」を保存・復元できること**が別途必要（Ink・Yarn は完全にホスト任せ、Fungus は `Save Point Loaded` で作者が手で復元、Monogatari は各アクションの `onLoad`、Ren'Py・Naninovel はエンジンが状態として持つ）。別プロジェクト構成では「スクリプト側が**宣言的な現在状態**（背景＝X、立ち絵＝{…}、BGM＝Y）を持ち、ロード時に丸ごと再送する」か「ホストに状態の書き出し／読み込みを要求する」かを選ぶことになる。
- 評価位置と表示位置のずれ: Ink の先読みと同じ問題は、スクリプトが 1 回の resume で複数行ぶん先へ進む設計なら必ず出る。副作用のあるホスト呼び出しは「表示キューに順序付きで積む」か「1 ステップ 1 yield」にする。

## 8. 統合 3 — ランタイムが直列化できないとき（コルーチン実行）のセーブ／ロードとロールバック

前提として、調べた主要システムは**どれもホスト言語のコルーチンで物語を走らせていない**。Ink（コンテナ木＋ポインタ）、Yarn（バイトコード VM）、Ren'Py（AST ノード＋Python の store を pickle）、Naninovel（コマンド列＋再生位置）はすべて「データ化された台本をインタプリタが歩く」形で、位置が素直にデータになる。そして Ren'Py は「ジェネレータ／コルーチンは保存できない」「文の途中でロードしたら文の先頭からやり直す」「対話を含む Python の while ループは丸ごとやり直しになる」と明記している（【確認】）。これは pasta の状況そのもの。

| 方式 | 内容 | 採用例 | 既知の落とし穴 |
|---|---|---|---|
| A. チェックポイント（ラベル境界）＋状態スナップショット | 保存点を「シーン／ラベルの先頭」に限り、変数＋現在ラベル（＋呼び出しスタックのラベル列）＋宣言的な画面状態を保存。ロードはラベル先頭から再実行 | Narrat（jump 時に自動保存。改訂耐性のためと明言）、Fungus（Save Point コマンド＋ロード時フック）、Yarn（公式は変数のみ。位置はノード名で自前） | ラベル途中までの台詞が再生し直しになる。再実行区間の副作用（加点・フラグ・once 消費・既読）が二重に効かないよう、**チェックポイント時点の変数へ戻してから**再実行する必要。call の途中（戻り先）は表現できないので、戻り先もラベルにする規律が要る（Ren'Py の `call ... from` と同じ発想）。保存点の粒度＝シーンの長さになる |
| B. チェックポイントからの決定的リプレイ | A に加えて「チェックポイント以降の入力列（選択結果・ホストからの戻り値）と乱数の種／消費数」を記録。ロード時は出力を捨てながら早回しし、保存時のステップ数に達したら通常実行へ | 全面採用の例は見当たらない。部分的には Ren'Py のロールフォワード（`renpy.checkpoint(data)` に入力結果を記録し `roll_forward_info()` で再供給）、Ren'Py の `renpy.random`（巻き戻しても同じ数）、SugarCube のシード付き PRNG（履歴と統合）、Ink の `storySeed`/`previousRandom` が同じ部品 | **決定性の全要件**: 時刻・ホスト問い合わせ・外部関数の戻り値もすべて記録対象。リプレイ中はホストへの副作用を抑止（Ink の `lookaheadSafe` と同じ「副作用あり／純粋」の区別が要る）。スクリプト改訂で発散する → 行 ID 列やコマンド列のハッシュで**発散検知**し、検知したら A（ラベル先頭）へ後退（Naninovel の Recovery Rollback と同じ後退先）。早回しコストはシーン長に比例 |
| C. 文インデックスでの再開 | 台本を「文の列＋プログラムカウンタ」として持ち、位置＝（スクリプト名, 文番号）。ローカル変数もテーブルへ | Naninovel（再生位置＋スナップショット）、Ren'Py（文の先頭で保存）、Ink（コンテナパス＋インデックス）、Yarn VM | コルーチンをやめて状態機械へコンパイルし直すことになる（Lua のローカル・ループ・ネストした呼び出しを全部データ化）。改訂で番号がずれる（Ink の issue、Ren'Py の行名）。対策は安定名（`call from`、行 ID、`|#id|`、GUID）と、位置が見つからないときの後退規則 |
| D. 逆操作による undo | 各操作に Apply/Revert を対で持たせ、戻るときは revert を逆順実行 | Monogatari | スナップショット不要だが、任意コードの逆を作者が書く必要。書き忘れで不整合。コルーチンの「位置」を戻す問題は解決しない |

ロールバック（バックログから戻る）の実現方式も同じ表に載る:
- スナップショット列（Ren'Py のロールバックログ、Naninovel の 1024/128 ステップ、SugarCube のモーメント、Fungus の Save History）— 上限つきリングバッファが標準。セーブに一部を含めると「ロード直後に戻れる」。
- コルーチン実行でスナップショットから「戻る」には、戻り先を A か B の方式で再構築するしかない（コルーチンは巻き戻せない）。つまり**ロールバック＝「過去の時点へのロード」**として同じ機構に乗せるのが筋。
- 戻ったあとの選択: Ren'Py は「変えられる」が既定、`fix_rollback` で「見るだけ」、`block_rollback` で「ここより前は不可」。Naninovel は `@purgeRollback`。作者が戻れない点を置ける語彙は必須。
- 乱数: 「戻って進み直したら結果が変わる」を許すかは方針。Ren'Py と SugarCube（有効時）は**変わらない**を選んでいる。変わらない方式なら、乱数の状態（種＋消費数、または引いた値の列）をスナップショットに含める。
- 全プレイ共通データ（既読・解放・persistent）は**巻き戻さない**（Ren'Py の persistent、Naninovel のグローバル状態）。once や訪問回数をどちら側に置くかは明示的に決める必要がある（Yarn は変数置き場＝巻き戻る側、Ink は state＝巻き戻る側）。

落とし穴の一般則:
1. 保存点より後の副作用の二重適用（Monogatari の「知力が無限に上がる」注意、Ren'Py の「Python 側データは巻き戻らない」注意）。
2. 改訂後の位置ずれ（各システムが安定名か後退規則で対処。完全解は無いと Ink が明言）。
3. 保存できない値（関数・循環参照・ホストオブジェクト）を変数へ入れられると破綻（Ren'Py の pickle 不可リスト、SugarCube の「関数・循環参照は不可」、Fungus の 4 型限定、Yarn の 3 型限定）→ **保存対象の変数は型を絞る**のが共通解。
4. 画面・音の復元はスクリプト位置の復元とは別問題（7 節末尾）。

## 9. 統合 4 — ホストに前提される最小の演出語彙（要求チェックリスト）

### 視覚
- [ ] 背景の設定（単一・置換）。「場面転換＝背景を替えて立ち絵を全消去」の一括操作（Ren'Py `scene`、Monogatari `show scene`）
- [ ] スプライトを**名前（タグ）で**表示／置換／消去。同名は置換（Ren'Py のタグ規則、Naninovel のアクター ID）
- [ ] 属性・表情の差し替え（タグ＋属性集合。部分指定で差分更新、レイヤ合成は任意）
- [ ] 位置指定: 名前付きスロット（left / center / right ほか）＋数値座標（割合）。前後関係（zorder / behind）
- [ ] 台詞に付随する表情変更（その行だけの一時変更を含む）とサイド画像（話者の顔）
- [ ] トランジション: 最低 dissolve（クロスフェード）と fade（色を挟む）。次点で wipe / slide / push / ルール画像、秒数指定
- [ ] 複数の変更を 1 つのトランジションで一括適用（Ren'Py `with`、Naninovel `@trans`）
- [ ] 移動・拡縮・回転・透明度のトゥイーン（所要時間＋イージング）。入退場アニメ
- [ ] 画面効果: 揺れ（縦・横）、フラッシュ、カメラのズーム／パン（レイヤ全体への変換）
- [ ] 単発画像（CG・イベント絵）のオーバーレイ、動画再生（任意）
- [ ] テキスト窓の表示／非表示、ADV（1 行）／NVL（全画面累積、ページクリア）の切替、話者名ラベル
- [ ] 選択肢 UI: 項目列＋各項目の有効／無効＋補助情報（タグ）、直前の台詞を残す指定（`#lastline`）
- [ ] 文字入力（名前入力）は任意

### 音
- [ ] チャンネル: BGM（ループ・1 本）／効果音（単発・多重可）／ボイス（次の台詞で止まる・持続指定可）／環境音（任意のループ）
- [ ] play / stop / queue、フェードイン・フェードアウト・クロスフェード、音量（チャンネル別ミキサー＋個別）
- [ ] 「同じ曲なら鳴らし直さない」（`if_changed`）、イントロ＋ループ区間
- [ ] ボイスを行 ID から自動で引く、キャラ別ミュート／音量、バックログからの再生
- [ ] 再生終了の通知（オート送りが「ボイス終了待ち」をするため）

### 時間・進行
- [ ] クリック待ち／秒待ち／「秒またはクリック」（スキップ可能タイマ）／ハードウェイト（非推奨だが存在）
- [ ] コマンド単位の「完了を待つ／待たない」、複数コマンドの並行実行と合流
- [ ] 完了通知（ホスト→スクリプト）と、**早送り（演出を即完了）**・**打ち切り（次へ）**の 2 段の割り込み
- [ ] 行内制御: 途中待ち、速度変更、ここまで即時表示、自動送り（入力を待たず次へ）、追記（extend）、行内位置でのイベント発火
- [ ] テキスト装飾: 太字・斜体・色・サイズ・ルビ・インライン画像・リンク（最低ルビと色）
- [ ] モード: スキップ（既読のみ／全部、選択肢で止まる）、オート（文字数と速度から待ち時間、ボイス待ち）— ホストまたは中間層が持ち、スクリプトは「既読かどうか」と「ここで止まれ」を供給

### 状態（ホストが協力すべき点）
- [ ] 現在の画面・音の状態を**宣言的に再現**できる入口（ロード・ロールバック時に「背景 X、スプライト {…}、BGM Y」を即時適用。トランジション無し）
- [ ] セーブ用サムネイル（スクリーンショット）
- [ ] バックログ表示用データ（話者・本文・ボイス・戻り先識別子）の蓄積はスクリプト側／中間層

## 10. システム別マトリクス（要約）

| | Ren'Py | Ink | Yarn Spinner | Naninovel |
|---|---|---|---|---|
| 話者 | Character オブジェクト（組込み） | 規約のみ | `Name:` → `character` 属性 | `Author:` / `Author.Appearance:` |
| 選択肢 | `menu`、`if`、`set`（一度きり） | `*` 一度きり既定／`+`／フォールバック | `->`、`<<if>>`（無効として送る）、`<<once>>` | `@choice`、`lock:`、入れ子コールバック |
| サブルーチン | `call`/`return`（`from`） | トンネル `-> x ->`、関数、スレッド | `<<detour>>`/`<<return>>` | `@gosub`/`@return` |
| バリエーション | `renpy.random` | `{a|b}` `{&}` `{!}` `{~}` | ライングループ＋サリエンシー | `random()`、`@random` |
| 訪問・既読 | `seen_label`、`is_seen` | 名前＝訪問回数、`TURNS_SINCE` | `visited()`、`visited_count()` | `hasPlayed()`、`calculateProgress()` |
| 演出の出し方 | 言語組込みの文 | タグ／外部関数／規約 | コマンド `<<>>`＋行タグ＋マークアップ | コマンド `@`＋インライン `[]` |
| 待ち | 文の種類で固定 | 概念なし（ホスト） | ハンドラの戻り値型 | 既定は待たない、`wait!`／`@await` |
| 行 ID | 翻訳 ID（ラベル＋ハッシュ、`id` 節で固定） | 無し | `#line:`（必須・自動付与） | `|#id|` |
| 保存単位 | 任意の文頭（pickle） | 任意の行（state JSON） | 変数のみ（公式） | 任意のコマンド位置＋サービス状態 |
| ロールバック | あり（ログ、fix/block） | 無し（スナップショットで自作可） | 無し | あり（スナップショット 1024/128、`@purgeRollback`） |
| 乱数と巻き戻し | 同じ値を返す | 種と直前値を状態に保存 | 規定なし | 未確認 |
| 改訂と旧セーブ | `call from`、`after_load`、追加文は実行されない | パスずれ（警告・フォールバック） | 位置を持たないので影響小 | Recovery Rollback（スクリプト先頭へ） |
| 全プレイ共通 | `persistent` | 無し（ホスト） | 無し（ホスト） | グローバル状態 |
