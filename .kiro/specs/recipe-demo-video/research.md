# ギャップ分析: recipe-demo-video

- 調べた日: 2026-11-15（worktree `claude/kiro-start-recipe-demo-video-daa207`、main `cd2ea431` の上。v0.6.1 から `editors/`・`crates/`・`Cargo.*` の変更は 0 件）
- 調べ方: 読み取りだけ（Grep・Glob・Read・git）。アプリは動かしていない。Openscreen は道具の定義（スキーマ）だけを読んだ。
- **読むときの注意（2026-11-15 追記）**: 1〜7 節は、要件ディスカッションの前のギャップ分析である。その後、pasta-recipe の並びを変えられる直し（要件 11）がこの spec に入り、撮る範囲と配置は `direction.md` 13 節の実測に替わった。「コードの変更は要らない」「印刷用のページ」「写しの置き場所」などの 1〜7 節の論点は、いまの `requirements.md` が正である。設計フェーズの調査と決定は 8 節に書く。

## 1. 要約

- **変える物はマニュアルの 2 章と動画 1 本だけ**で、コードの変更は要らない。参照の検査（`verify-static.mjs` の `<video>` の `src`・`poster` の実在、`.mp4` の許可）と見出しへのリンクの検査（`link-check.mjs`）は今のまま効く。`dev-actions-demo.mp4` を参照するのは `dev-actions.md` 27〜28 行だけ。
- **要件の前提が 1 つ古い**: 印刷用のページ（`print.html`）は `manual-print-page-removal`（#124）で廃止済み。`book.toml` は `[output.html.print] enable = false`、`verify-static.mjs` は逆に「`print.html` が無いこと」を検査する（4.3）。`manual-print-media-refs` の書き換えの仕組みも残っていない。要件の Introduction・Adjacent・Out of scope・5.4・9.1 の「印刷用のページ」の記述は満たせない／意味を持たない。
- **個人の情報の最大の穴は出力の欄**: 操作ごとの頭の行が `表示名（ゴーストのルートの絶対パス）→ SSP 127.0.0.1:番号` を出し、「保存変数を表示」は `保存変数（<variables.toml の絶対パス>）を表に出しました` を出す。写しをスクラッチパッド（`C:\Users\maz-o\...`）に置くとユーザー名が必ず映る。窓の題名はフォルダの名前だけ（`hello-pasta - pasta-recipe 0.6.1`）で安全。
- **Openscreen の録画は「窓 1 つ」か「画面 1 枚」**。窓で撮ると別の窓の SSP のゴーストが入らない（要件 1.1 に反する）ので、画面で撮る。映す範囲は、編集で切り抜ける（`setClipRange` の `cropRegion`。録った絵の一部を 0〜1 の割合で選ぶ。エディタの「Edit clip」の切り抜きと同じ）。※ 初版は「切り抜きが無い」と書いたが誤り（2026-11-15、要件ディスカッションの前に直した）。
- **hello-pasta には、ローカル変数を台詞に出すシーンも、写した直後の保存変数も無い**。値を入れて実行は `13-calendar.pasta` の `＊会話`（`＄行事` を上書きして `＊行事_バレンタイン` へ）か、写しに台本用のシーンを足すかを選ぶ。保存変数は `10-save.pasta` の `＊会話` を一度鳴らせば `＄＊回数` ができる。

## 2. 今の状態（調べた事実）

### 2.1 動画のファイルと参照

| 項目 | 事実 |
| -- | -- |
| 実体 | `book/src/debug/media/dev-actions-demo.mp4`（1,403,681 バイト）。`media/` にはこの 1 本だけ |
| 前の撮影 | `vscode-mcp-client` の research.md: 1440x900・1 分 51 秒。開発者が VSCode を手で操作して撮った |
| 参照 | `book/src/debug/dev-actions.md` 27〜28 行（`<video ... src="media/dev-actions-demo.mp4">` と、中の直接開くリンク）。ほかの章・ツール・スキル・README・試験の材料には無い（完了した spec の記録を除く） |
| 見出しへのリンク | 「デモ：書いて、鳴らして、直す」へのリンクはどこにも無い |
| 旧版（classic） | `manual.yml` が固定コミット `CLASSIC_REF`（`6d4e868d`）の `book/` から `book/book/classic/` を作る。そのコミットに `dev-actions-demo.mp4` が入っているので、**Pages の `classic/` の下には古い動画が残り続ける**。新版の検査は `classic/` を見ない（`verify-static.mjs` の `listFiles`）。要件 4.4・4.5（リポジトリ・章から消す）とはぶつからない |

### 2.2 マニュアルの検査

| 検査 | この spec への効き方 |
| -- | -- |
| `verify-static.mjs` | 出力の全ページの `href`・`src`・`poster` の相対参照の先が在るかを見る。`.mp4` は許可リストにある。消した動画を指す参照が残れば落ちる（要件 9.2 は今のまま満たす）。印刷用のページは「無いこと」を見る（4.3） |
| `link-check.mjs` | 本の中のリンクと見出し（`#…`）を見る。`dev-actions.md` → `recipe.md#…` の新しいリンクのアンカーの綴りはここで確かめられる |
| `verify-content.mjs` | `dev-actions.md` は `DEBUG_CHAPTERS` にあり、本文 800 字以上・台詞部品の口調などを見る。デモの節を 1 行にしても本文は十分に残る。`recipe.md` は対象外 |
| `gen-skill-refs.mjs` | 2 章とも生成の対象外（要件の記述どおり） |
| CI の順 | `manual.yml`: gen-skill-refs --check → mdbook build → 着色 → 台詞部品 → bigram 索引 → link-check → tutorial-check → verify-static --no-build --self-test → verify-search → verify-content → 自己テスト群 → 旧版の生成 |

### 2.3 2 つの章の構造

- `recipe.md`: `## ゴーストを動かす`（171 行）→ 頭の説明（173 行の導入・175〜181 行の操作の表・183 行のデバッグの有効化・185 行の「一度に 1 つずつ」）→ `### 送る相手の探し方`（187 行）→ 未保存のファイルの保存 → ▶ シーンを実行 → 値を入れて実行 → SHIORI をリロード → 出力の欄 → 保存変数の表。前提 A8 の位置（185 行と 187 行の間）はそのまま置ける。
- `dev-actions.md`: `## デモ：書いて、鳴らして、直す`（23 行）→ 25 行の段落 → 27〜29 行の `<video>`。この節の後が `## 3 つのコマンドに共通の動き`。
- 見出しのアンカー: mdBook は「：」「、」を落とす。既存の例は `#シーン再生-シーンを実行`・`#送る相手の選び方と覚え方`。`### デモ：書いて、鳴らして、直す` の id は `デモ書いて鳴らして直す` になる見込み（**link-check で確かめる**）。

### 2.4 pasta-recipe が画面に出すもの（個人の情報）

| 場所 | 出るもの | パスが映るか |
| -- | -- | -- |
| 窓の題名 | `{ゴーストのフォルダの名前} - pasta-recipe {版}`（`commands.rs` 210 行。名前は `ghost/master` の 2 つ上のフォルダ名） | 映らない |
| 版 | `tauri.conf.json` の `version` が `../../vscode/package.json` を指す（今は `0.6.1`）。リリースの exe も手元のビルド（`npx tauri dev`・`tauri build`）も同じ番号を出す | — |
| ツリー | `ghost/master` の中の相対の並び | 映らない |
| 出力の欄の頭の行 | `時刻 操作名 — 表示名（ルートの絶対パス）→ SSP 127.0.0.1:番号`（`editors/shared/devSession.ts` 87・109 行）。表示名は `descript.txt` の `name` | **映る** |
| 保存変数を表示の結果 | `保存変数（<variables.toml の絶対パス>）を表に出しました`（`devActions.ts` 205 行） | **映る** |
| 誤りの行 | `descript.txt` が無い・保存に失敗・pasta.log が読めない、などの行に絶対パス | 失敗したときだけ映る |
| 開始の画面・「最近開いたもの」のメニュー | 最近開いたゴーストのフルパス（`recent.toml` の中身。他のゴーストの場所も出る） | **映る**（撮影の前に CDP で開いておけば開始の画面は出ない。メニューは開かない） |
| 送る相手を選ぶ窓 | 候補が 2 つ以上のときだけ「名前（127.0.0.1:番号）」 | 映らない |

- 前の動画（VSCode）は、頭の行に `c:\home\maz\git\ghost_dev\debug\emo2-debug` を映して公開済み（`vscode-mcp-client` の research.md）。Windows のアカウントのフォルダは `maz-o`、リポジトリの置き場所 `C:\home\maz\...` にも `maz` が入る。
- 相手探しは、候補の全部の番号（9801〜9810・9821〜9830）へ `get_property(baseware.name, ghost_name=<ルート>)` を送る（`editors/shared/mcpClient.ts` 97 行）。**普段の SSP（9801・9821）にも、読むだけの問い合わせが届く**。

### 2.5 hello-pasta（`crates/pasta_sample_ghost/ghosts/hello-pasta/`）

| 項目 | 事実 |
| -- | -- |
| 生成物 | `pasta.dll`・`scripts/` は git に無い（`.gitignore`）。本体のチェックアウトにも今は無い。写しを動かすには `release.ps1` で作るか、リリースの配布物から取る |
| デバッグ | `pasta.toml` に `[debug]` が無い。写しに `[debug] enabled = true` を足すか、環境の `PASTA_DEBUG=1`（試験用の SSP が受け継ぐ）で有効にする |
| 自然なおしゃべり | `talk_interval_min = 45`・`max = 75`。60〜90 秒の撮影の間に、頼んでいないおしゃべりが 1〜2 回はさまる見込み |
| 起動 | `--option readonly` の SSP は毎回 `OnFirstBoot` を送る（メモ）。起動の直後に台詞が出る |
| ローカル変数を台詞に出すシーン | **無い**。台詞に出る `＄…` は日時変数 `＄時１２`（06-hour）・リクエスト変数 `＄ｒ０`（07-greeting の `＊OnGhostChanged` など）・プロパティ `＄％baseware.name` だけ。`13-calendar.pasta` の `＊会話` は `＞「行事_」＆＄行事` で行事のシーンへ飛ぶ |
| 値を入れて実行の後勝ち | Reference2 の値は転記された日時変数・リクエスト変数に勝つ（`shiori-events.md` 833 行）。`行事`＝`バレンタイン` を入れて `13-calendar.pasta` の `＊会話` を鳴らせば `＊行事_バレンタイン` が鳴る（値は台詞に文字として出ないが、選ばれる掛け合いが変わる） |
| 保存変数 | 写した直後は無い（`profile/` は持ってこない）。`10-save.pasta` の `＊会話` を鳴らすと `＄＊回数` ができ、台詞「この話をするのは＄＊回数　回目だよ！」に値が出る。書き換え…で `99` にすれば、次に鳴らすと「100 回目」になる |
| 名前の重なり | 普段の SSP の読み込み先に本体の hello-pasta が入っている（メモ `ssp-ghost-test-launch`）。同じ SSP のインストールから立てる試験用の SSP でも重なるので、`descript.txt` の `name` を変える。**変えた名前は出力の欄の頭の行に映る** |

### 2.6 撮影の道具（Openscreen の MCP の定義から）

- `startRecording`: **窓 1 つ（題名の一部）か画面 1 枚**。録るときの範囲の指定は無いが、編集で切り抜ける（次の `setClipRange`）。窓は「ほかの物が映らない」ので推奨とされる。録画は Openscreen の録画のバーを通り、カウントダウンがある。マイク・システムの音は既定で取らない。
- `setClipRange`: クリップの `cropRegion`（`{x, y, width, height}`、録った絵に対する 0〜1 の割合）で、映す範囲を切り抜く。`null` で全体に戻す。`duplicateClip` も切り抜きを写す。
- `listSources`: 最小化・隠れた窓は出ない。2026-11-15 の一覧では画面が 2 枚（「画面 1」「画面 2」）。
- `setPointerTrack`: CDP で操作した撮影用。`x`・`y` は**撮った窓（画面）に対する 0〜1 の割合**、`at` は `Date.now()`。窓の中の点を割合にする式は、窓を撮ったときのもの。画面を撮るなら、窓の画面上の位置と表示の倍率から自分で割合にする。
- `exportVideo`: MP4（H.264）。高さ 720・1080（既定）・元のまま、fps 24・30・60（既定 60）。形は `setAspectRatio`。
- `setEditorSettings`: 書き出しには背景（wallpaper）・余白（padding）・枠（frame）・角丸・影が付く。**既定のままだと、Openscreen の飾りの背景と余白が動画に入りうる**。
- 撮った録画と編集はプロジェクトとして残る（前提 A10）。

### 2.7 既にある撮影の決まり（`book/SCREENSHOTS.md`）

入門ガイドの 16 枚の絵の撮り直しの記録。この spec に効くもの:

- 写しは**リポジトリの外の作業フォルダ**に置き、`name` を変える（名前は画面に写らない前提だった。今回は出力の欄に映る）。
- 背景は壁紙でなく、**ゴーストの後ろに無地の白の全画面の窓**を置く（ブラウザのキオスク表示）。
- Windows の表示 200%・SSP のシェル倍率とバルーン倍率は 100%（写しの `profile/ghost.dat` に入る）。
- 写す前の確かめ: 送り主の表示・開発用パレット・エラー・ほかのゴースト・デスクトップ・タスクバー・カーソルが写っていない。
- 撮る範囲は ffmpeg の gdigrab で切り抜いた（静止画の撮影だったため。今回は Openscreen の `cropRegion` で切り抜ける）。

## 3. 要件ごとの対応（Requirement-to-Asset Map）

| 要件 | 今ある物 | ギャップ |
| -- | -- | -- |
| 1.1 窓とゴーストを同じコマに | Openscreen の画面の録画 | **Constraint**: 窓の録画では SSP が入らない。画面の録画で撮る範囲を整える必要 |
| 1.2〜1.6 流れ | pasta-recipe 0.6.1 の開発の操作（`pasta-recipe-dev-actions`） | **Missing**: 台詞にローカル変数を出すシーン（1.4）・最初の保存変数（1.6）。写しで用意するか既存シーンで代える |
| 1.7・1.8 本文と食い違わない | `recipe.md` の操作の名前 | 無し（撮った後に照らす） |
| 2.1〜2.3 ポインタ・拡大 | `setPointerTrack`・クリックに寄るズーム | **Unknown**: 画面の録画のときの割合の求め方（窓の位置・DPI 200%）。2.3「始めと終わりは全体」は、全体に何が映るか（3.1）と組で決まる |
| 2.4・2.5 長さ・止まって見える長さ | `addTrim`・`setSpeed` | **Constraint**: 自然なおしゃべり（45〜75 秒ごと）・OnFirstBoot が挟まる |
| 2.6 文字が読める | ズーム・書き出しの高さ | **Unknown**: 本文の幅（約 750 px）で読める倍率 |
| 2.7 音声なし | 既定でマイク・システムの音を取らない | **Unknown**: 書き出しの MP4 に無音の音声のトラックが付くか（付くなら ffmpeg `-an`） |
| 3.1 映す範囲 | Openscreen の `cropRegion` | 無し。撮る画面（ノートPCの主画面）は、開発者がタスクバーを自動で隠し・アイコンをよけ・通知を切った（要件の前提 A11） |
| 3.2 ユーザー名のパス | — | **Missing**: 写しの置き場所。出力の欄の頭の行・保存変数の行に絶対パスが必ず出る |
| 3.3・3.4 コマの確かめ | `getFrames` | 無し |
| 4.1〜4.5 ファイル | `media/`・`verify-static` | **Unknown**: 60 fps・1080 の H.264 で 5 MB に収まるか（前は 1440x900・1 分 51 秒で 1.4 MB）。収まらなければ ffmpeg で詰め直す |
| 5.1〜5.3 recipe.md | `dev-actions.md` の段落と `<video>` の書き方 | 写して名前を変えるだけ |
| 5.4 印刷用のページ | — | **Constraint（要件の誤り）**: `print.html` は廃止済み。満たせない |
| 5.5 公開とオフライン | `verify-static`（file:// で解決する相対参照） | 無し |
| 6.1〜6.4 dev-actions.md | `link-check` のアンカー検査 | アンカーの綴りを確かめるだけ |
| 7.1・7.2 Openscreen | MCP の窓口 | 無し |
| 7.3・7.4 普段の SSP | 試験用の SSP（`--option readonly --sstp-listen 9802`） | **Constraint**: pasta-recipe の相手探しが普段の SSP にも読むだけの問い合わせを送る。7.3 の「イベントやスクリプトを送らない」に入るかを決める |
| 7.5 マウスとキーボード | CDP（メモ `pasta-recipe-ui-check-via-cdp`） | 無し。SSP のゴーストの位置は PowerShell の SetWindowPos で動かせる |
| 7.6〜7.9 写しと後片付け | `recent.toml` の写し | **Missing**: 撮る pasta-recipe と `pasta.dll` の入手の仕方（下の 4.1）。写しの置き場所の後片付け |
| 8.x 不具合 | — | 無し（手順の決まり） |
| 9.1 検査 | `manual.yml` の順 | **Constraint**: 「印刷用のページの動画の参照を含む」は今の検査に無い |
| 9.2 | `verify-static` | 今のまま満たす |

## 4. 実装の進め方の案

### 4.1 撮る物の入手（前提 A1）

| 案 | 中身 | 良い点 | 気になる点 |
| -- | -- | -- | -- |
| A1-a リリースの配布物 | GitHub の v0.6.1 から `pasta-recipe.x64.zip` と hello-pasta の nar（または `pasta.dll`）を落とす | 読者と同じ物 | ファイルを落とすので開発者の許しが要る。`recent.toml` は同じ場所に書く |
| A1-b 手元でビルド | worktree で `tauri build`（または `npx tauri dev`）と `release.ps1` | v0.6.1 と中身が同じ（`editors/`・`crates/` の差分 0）。題名も `0.6.1` | `tauri dev` は Vite の開発サーバーで動く。ビルドの時間。LuaJIT のビルドには `NoDefaultCurrentDirectoryInExePath` を外す（メモ） |

要件の前提 A1 の理由「開発中の版は題名の版が違うことがある」は、今の作り（版は `editors/vscode/package.json` から）では当たらない。違いが出るのは、v0.6.1 より後に `editors/`・`crates/` が変わったときだけ。

### 4.2 撮る範囲（要件 1.1・2.3・3.1）

| 案 | 中身 | 良い点 | 気になる点 |
| -- | -- | -- | -- |
| R-a 2 台目の画面を撮る | pasta-recipe の窓と試験用の SSP のゴーストを 2 台目の画面（17MT70）へ動かし、その画面を撮る | 開発者の主画面の作業を邪魔しにくい | 2 台目の画面のタスクバーの出方は Windows の設定しだい（変えるのはシステムの設定）。開発者が窓をその画面へ動かすと映る |
| R-b 主画面を撮り、白の全画面の窓を後ろに置く | `SCREENSHOTS.md` と同じく白の窓を背景にし、全体をずっと少し拡大してタスクバーを外す | 既にある決まりに合う | 2.3「始めと終わりは拡大しない」と両立しない。開発者の主画面を撮影の間ふさぐ |
| R-c 書き出しの後に ffmpeg で切り抜く | Openscreen で画面を撮って書き出し、`crop` で範囲を切る | 範囲を自由に決められる | 要件 7.1「録画と編集と書き出しに Openscreen」の外の工程が増える。ズームの位置が切り抜きとずれうる |
| R-e 画面を撮り、Openscreen で切り抜く | 画面を録り、`setClipRange` の `cropRegion` で pasta-recipe とゴーストの入る範囲を切る | Openscreen の中で済む（7.1）。範囲を自由に決められる | ポインタの割合（`setPointerTrack`）は切り抜く前の画面に対するもの |
| R-d 窓だけを撮る | pasta-recipe の窓を撮る | ほかの物が映らない | ゴーストが映らない（1.1 に反する）。採れない |

### 4.3 値を入れて実行と保存変数の見せ方（前提 A2・A3）

| 案 | 中身 | 良い点 | 気になる点 |
| -- | -- | -- | -- |
| V-a 既存のシーンだけ | 1.2・1.3・1.6 は `10-save.pasta` の `＊会話`（「N 回目だよ！」）、1.4 は `13-calendar.pasta` の `＊会話` に `行事`＝`バレンタイン` | 写しを書き換えない。1.6 の書き換え（`99`）が次の「100 回目」で見える | 1.4 の値は台詞に文字として出ない（前提 A3 の言い方に合わない） |
| V-b 写しに台本用のシーンを足す | 例 `＊天気を言う` に `女の子：今日は＄天気　だね。`。1.4 で `天気`＝`雨` | 入れた値が台詞にそのまま出る | 読者の hello-pasta に無いシーンが映る |
| V-c `＄ｒ０` を使う | 07-greeting の `＊OnGhostChanged` に `ｒ０`（または `r0`）を入れる | 値が台詞に出る・既存のシーン | 名前の全角・半角の扱いを確かめる必要（Research）。イベント名のシーンを鳴らすのは例として分かりにくい |

### 4.4 マニュアルの直し（要件 5・6・9）

`dev-actions.md` 25〜29 行の段落と `<video>` を写して `recipe.md` の 185 行と 187 行の間へ置き、文言を pasta-recipe の画面に合わせる。`dev-actions.md` の節は見出しを残して 1 行のリンクにする。新しい道具・検査は要らない（既存の拡張・Option A の 1 択）。

### 4.5 撮り直しの記録

`SCREENSHOTS.md` は入門ガイドの絵の撮り直しの記録で、動画の記録は無い。要件は記録を求めていない。Downstream に「画面が大きく変わったときの撮り直し」があるので、手順（撮る範囲・写しの用意・CDP の台本・Openscreen の設定）を残すかは論点になる。

## 5. 手間と危うさ

- **手間: S（1〜3 日）**。マニュアルの直しは数十行。手間の大半は撮影の環境作り・CDP の台本・撮り直しの回数。
- **危うさ: 中**。コードは触らないが、Openscreen の範囲の制約・個人の情報（出力の欄の絶対パス）・自然なおしゃべりの割り込み・5 MB の大きさは、撮ってみないと決まらない部分がある。開発者の環境（普段の SSP・`recent.toml`）に触れる手順がある。

## 6. 設計の判断の材料（要件ディスカッションへ）

1. **印刷用のページの記述**: `print.html` は廃止済み（#124）。Introduction の 4 つ目の点・Adjacent の 1 つ目・Out of scope の最後・5.4・9.1 から外すか、「章を印刷したとき」の扱い（動画は刷れない）に言い換えるか。
2. **写しの置き場所とパス（3.2）**: 出力の欄に絶対パスが必ず出る。ユーザー名の入らない場所（例 `C:\pasta-demo\hello-pasta`・`subst` のドライブ）に置くか。`C:\home\maz\...` の `maz` をユーザー名と見るか（前の動画はこれを映して公開済み）。
3. **写しの `name`**: 変えた名前が出力の欄の頭の行に映る。読者に違和感の無い名前にするか、試験用の SSP で重ならない別の手を探すか。
4. **撮る範囲（1.1・2.3・3.1）**: 2 台目の画面・主画面と白の背景・Openscreen の切り抜き（R-e）・書き出しの後の切り抜きのどれにするか。2.3 の「全体」を、画面全体でなく「pasta-recipe とゴーストが入る範囲」と言い直すか。背景を壁紙にするか白の窓にするか（壁紙が私的な絵のこともある）。
5. **Openscreen の飾り**: 背景・余白・枠・角丸・影を消して（padding 0・frame none）画面だけにするか。
6. **値を入れて実行の見せ方（前提 A3・1.4）**: 既存のシーン（`＄行事` で掛け合いが変わる）か、写しに足す台本用のシーン（値が台詞に出る）か。
7. **保存変数の用意（1.6）**: `10-save.pasta` の `＊会話` を 1.2 の ▶ に使って `＄＊回数` を作る流れにするか、撮る前に用意するか。
8. **割り込みのおしゃべり**: 写しの `talk_interval` を延ばす（写しの書き換え）か、切り詰めで落とすか。OnFirstBoot の台詞は撮り始める前に済ませるか。
9. **撮る pasta-recipe と pasta.dll（前提 A1）**: リリースの配布物を落とす（許しが要る）か、手元でビルドする（v0.6.1 と同じ中身）か。前提 A1 の理由の書き方を直すか。
10. **普段の SSP への問い合わせ（7.3）**: 相手探しが普段の SSP にも `get_property` を送る。読むだけの問い合わせを 7.3 の外と書くか。
11. **5 MB と音声（2.7・4.3）**: 書き出しの高さ・fps（例 720・30）と、超えたときの ffmpeg の詰め直し・無音のトラックの除去を、Openscreen の外の工程として認めるか（7.1 との関係）。
12. **撮り直しの記録**: `SCREENSHOTS.md` に動画の節を足すか、残さないか。
13. **旧版の古い動画**: Pages の `classic/` の下に `dev-actions-demo.mp4` が残ることを、そのままでよいと書いておくか。

## 7. 設計で調べること（Research Needed）

- Openscreen の書き出しの MP4 に音声のトラックが付くか。録画のバーとカウントダウンが撮る画面に映るか。
- 画面を撮ったときの `setPointerTrack` の割合の求め方（窓の位置・表示の倍率 200%・2 台目の画面の原点）。
- `### デモ：書いて、鳴らして、直す` の mdBook のアンカーの綴り（link-check で確かめる）。
- 値を入れて実行の名前に全角の `ｒ０` を入れたとき、`＄ｒ０` に届くか（案 V-c を採るときだけ）。
- 試験用の SSP（readonly）が MCP の `raise_event` で鳴らしたとき、送り主の表示が出るか（`SCREENSHOTS.md` の確かめの項目）。
- 60〜90 秒の H.264 を 5 MB 以下にする高さ・fps・画質の組。

## 8. 設計フェーズの調査と決定（2026-11-15）

### 8.1 Summary

- **Feature**: `recipe-demo-video`
- **Discovery Scope**: Extension（軽い調査）。流れ A（要件 11）は今ある pasta-recipe への拡張、流れ B（要件 1〜10）は製品のコードではない手順である。新しい依存・外のサービスは無いので、全面の調査には上げなかった。
- **調べ方**: 読み取りだけ（`editors/recipe/` のソース・`book/` の章と道具・`.github/workflows/`・Openscreen の道具の定義・UKADOC のさくらスクリプト）。アプリは動かしていない。録画もしていない。
- **Key Findings**:
  - 並びを覚える型は、もうアプリの中に在る。`src-tauri/src/recent.rs` が「ファイルが正本・毎回読み・変えたら書く・壊れていれば既定・記録の失敗で止めない」を `app_data_dir` の TOML でやっている。並びの記録は同じ作りの `layout.toml` で足りる（新しい crate は要らない）。
  - 画面は、大きさを決めている所が 3 か所（`#tree` の `width: 260px`・`.panels-face` の `height: 180px`・`.panel` の `flex: 1`）だけで、Monaco は `automaticLayout: true` である。仕切りは CSS の値を変えるだけで効き、エディタの側に手を入れなくてよい。
  - `direction.md` 13 節の実測（切り抜き 1280×800）で文字の大きさを計算し直すと、13 px の文字（出力の欄・保存変数の表）を本文の幅で 10 px にするには 1.32 倍以上の寄りが要る。13 節の見込み（1.25〜1.3 倍）では足りない。1.35 倍の画（948×593）には、押す ▶ から表の行までの高さ（下の欄が 180 px のとき約 580〜620 px）が入りきらないので、撮影の並びで下の欄を上げる（仕切りの使いどころが 2 つになった）。
  - 撮影の並びは、仕切りを CDP で引きずらなくても、`layout.toml` に書いてから起動すれば決まる。流れ B が流れ A に触れる所はここだけになる。
  - 要件 2.8 は「要件 1 の 2〜6 と 9 の各操作」に名前の見出しを求める。`direction.md` 7.1 節の見出しは 5 つ（5 つ目が締めの ▶ を含む）なので、6 つに直した。

### 8.2 Research Log

#### pasta-recipe の画面の作りと、足す所

- **Context**: 仕切り 3 本と入れ替えを、どのファイルに・どの大きさで足すか。
- **Sources Consulted**: `editors/recipe/index.html`・`src/style.css`・`src/panels.ts`・`src/main.ts`・`src/devPanels.ts`・`src/editor.ts`・`src/devActions.ts`
- **Findings**:
  - `#tree` と `#editor-area` は `#workspace`（横の flex）の子、`#panels` は `body`（縦の flex）の子で、`index.html` に静的に書いてある。動かす面の 2 つの `.panel` は `Panels` が作る。だから仕切りは、2 本を `index.html` に、1 本を `panels.ts` に置くのが今の作りに合う。
  - 開始の画面の間は、CSS が `body` の直下を隠す（`body:has(#start:not([hidden])) > :not(...)`）。`body` の直下に足す仕切りも、同じ決まりで隠れる。
  - 出力の欄は `section.scrollTop` を一番下へ送る。DOM を並べ替えるとスクロールの位置が頭に戻るので、入れ替えのときは位置を保つ手当てが要る。
  - `panels.show("run")` は開発の操作のたびに呼ばれる。入れ替えのボタンの出し隠しは `show` に入れればよい。
  - `Panels` の `output`・`variables`（`devPanels.ts` が中身を満たす口）は、順を変えても参照が切れない。
- **Implications**: 変えるファイルは `index.html`・`style.css`・`panels.ts`・`main.ts`・`ipc.ts` と、新しい `layout.ts`。`devPanels.ts`・`devActions.ts`・`editors/shared/` は触らない。

#### 記録の型（`recent.toml`）

- **Context**: 並びをどこに覚えるか（管理役の指示: 先に `recent.toml` の書き方を見る）。
- **Sources Consulted**: `src-tauri/src/recent.rs`・`commands.rs`（`open_ghost`・`recent_list`・`recent_remove`）・`capabilities/default.json`・`Cargo.toml`・`book/src/debug/recipe.md` の節「入手と置き方」
- **Findings**:
  - 置き場所は `app.path().app_data_dir()`（`%APPDATA%\io.github.ekicyou.pasta-recipe\`）。ファイルの型（`RecentFile`）と画面への型（`RecentEntry`）を分けている。試験は同じファイルの `#[cfg(test)]` で、`tempfile` を使う。
  - アプリのコマンドは `commands::handler()` の表に足すだけで呼べる（許可のファイルには、アプリのコマンドは書かれていない）。
  - `recipe.md` は、アプリが書く場所を「最近開いたゴーストの記録」と「WebView2 の作業用のデータ」の 2 つと書いている。並びの記録を足すなら、この文も直す。
- **Implications**: `layout.rs` を `recent.rs` と同じ作りで足す。章の文に 1 か所足す。

#### 試験と CI

- **Sources Consulted**: `editors/recipe/package.json`・`src/*.test.ts`・`.kiro/steering/tech.md`（job `recipe`）・`.github/workflows/build.yml`
- **Findings**: `npm test` は `node --test "src/**/*.test.ts"`。DOM を持つ試験は無く、DOM を使うファイル（`devPanels.ts`）は純粋な関数を書き出して試している。CI は `tsc --noEmit`・`npm test`・`npm run build`・`cargo test --locked`・clippy（`-D warnings`）と、zip の検査・arm64 の煙試験を回す。
- **Implications**: 並びの決まりを純粋な関数にして `layout.test.ts` で試す。DOM の糊は、手元のビルドを CDP で確かめる（メモ「pasta-recipe の画面は CDP で確かめる」）。依存を足さないので `--locked` はそのまま通る。

#### Openscreen の道具の定義

- **Sources Consulted**: `startRecording`・`stopRecording`・`setClipRange`・`setPointerTrack`・`addZoom`・`addAnnotation`・`setEditorSettings`・`exportVideo`・`getFrames`・`addSpeed`・`addTrim`・`setAspectRatio` の定義（スキーマ）
- **Findings**:
  - 時刻の基準が 2 つある。クリップの出入りと切り（`addTrim`）は元の時刻、寄り・速さ・見出しは定規の秒。切りは定規に穴を空けるだけで詰めないので、クリップが 1 本で頭から始まるかぎり、定規の秒は元の時刻と同じである。クリップの入りの点を動かすと、定規がずれる。
  - `addZoom` は `customScale`（1〜5）で好きな倍率を取れる。録画がぼける手前より深い倍率は断られる。1.5 秒以内に続く寄りは、引かずに滑る。寄りの出入りには 0.7〜0.8 秒かかり、その最中に切ると画が飛ぶ。
  - `addAnnotation` の文字は、書き出しの枠に付く（寄っても動かない）。箱は文の長さに合わせて作られ、枠の中に収められる。濃い板が既定。
  - `setEditorSettings` で、余白・枠・角丸・影・動きのぼかし・背景の色・ポインタの大きさと弾みを決められる。
  - `exportVideo` は MP4（H.264）で、高さ 720・1080・元のまま、fps 24・30・60。形は `setAspectRatio`（16:10 がある）。
  - `setPointerTrack` の位置は、録った画面に対する割合である。
- **Implications**: 頭と尻は、クリップの出入りの点でなく `addTrim` で切る。台本は、操作ごとに `Date.now()` と場面の目印を記録し、編集の秒をそこから出す。

#### さくらスクリプト（位置の合わせ方）

- **Sources Consulted**: UKADOC の `\![move]`・`\![set,balloonoffset,x,y]`（伺かの技術文書の検索）・`book/SCREENSHOTS.md`
- **Findings**: `\![move]` は `--X`・`--Y`・`--base`・`--base-offset`・`--move-offset` で、画面を基準に動かせる。`\![set,balloonoffset,x,y]` は吹き出しのオフセットを置き換え、利用者のドラッグの分がそこへ足される。吹き出しの左は「立ち絵の左 − 吹き出しの幅 ＋ オフセットの x」、上は「立ち絵の上 ＋ オフセットの y」と読める（`direction.md` 4.2 節の逆算と合う）。
- **Implications**: 13 節の矩形から出したオフセットの見込みは、女の子 244・-102、男の子 97・142。読み込み直しの後も残るかは分からないので、送った後に矩形を読んで合わせ、駄目なら開発者のドラッグに頼む。

#### 13 節の配置と文字の大きさ

- **Sources Consulted**: `direction.md` 13 節・`style.css`・`devPanels.ts`
- **Findings**（切り抜いた画 1280×800 の上の px。見込み）:
  - 窓の内側は x 36〜1276・y 76〜798。エディタの左は 297、行は 147 から 19 px ごと。`＊会話` の上の ▶ は y 約 185、`＊天気` の上は約 298。
  - 立ち絵の体は、男の子が x 952〜1077、女の子が 1076〜1234。吹き出しは、女の子が (898,198)-(1233,367)、男の子が (610,442)-(945,538)。
  - 下の欄が 180 px のとき、保存変数の表の行は y 約 740〜766 に来る。▶（185）から表の行（766）までは約 581 px で、1.35 倍の画（高さ 593 px）には見出しの分の余りが残らない。240 px に上げると、行は約 680〜706 に上がる。
  - 出力と保存変数の境が既定の 0.5 だと、表（幅 約 276 px）の右の端は約 940 で、男の子の腕（約 950）まで 10 px しか無い。0.42 にすると約 841 になる。
  - 本文の幅 750 px での文字の高さは `高さ × 750 ÷ (1280 ÷ 倍率)`。1.35 倍で 13 px → 10.3 px、14 px → 11.1 px、12 px → 9.5 px。
- **Implications**: 撮影の並びを `panels_height = 240`・`output_share = 0.42`、寄りを 1.35 倍とした。どれも仮で、配置を見せるコマ（要件 7.13）で決め直す。

#### 参照したスキルと指針

- `kiro-spec-design` の `rules/`（design-principles・design-discovery-light・design-synthesis・design-review-gate）と、`.kiro/settings/templates/specs/` の雛形。
- ponytail（最小の変更の段取り）: 今あるコードの型を先に使う（`recent.rs`）・標準の機能で足りるものに包みを足さない・頼まれていない選べる設定を足さない。
- 仕切りの読み上げとキーは、WAI-ARIA の「Window Splitter」の型（`role="separator"`・焦点を当てられる・`aria-valuenow`・矢印のキー）に合わせた。新しい部品は入れていない。
- プロジェクトのメモ: 書式は TOML を既定にする方針・pasta-recipe の画面は CDP で確かめる・試験用の SSP は readonly で立てて `\-` で閉じる・LuaJIT のビルドの前に `NoDefaultCurrentDirectoryInExePath` を外す・worktree に `book/node_modules` が無いことがある。

### 8.3 Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
| -- | -- | -- | -- | -- |
| 並び: `layout.toml`（Rust 経由） | `recent.rs` と同じ作りで `app_data_dir` に書く | アプリの今の型。操作の終わりに確かに書ける。撮影の前後で写して戻せる。Rust の試験が書ける | コマンド 2 つと Rust のファイル 1 つが増える | **採る** |
| 並び: WebView の `localStorage` | 画面だけで覚える | Rust の変更が無い | WebView2 の書き込みは遅れて行われ、閉じ際の値が残るかを確かめにくい。保存先は `EBWebView` の中の入れ物で、撮影の前後で写して戻すのが重い。アプリのほかの記録（TOML）と型が割れる | 採らない |
| 並び: Tauri の窓の状態のプラグイン | 窓の位置と大きさを覚える包み | 出来合い | 欄の並びは覚えない。新しい依存になる（禁止） | 採らない |
| 仕切り: 自前（Pointer Events） | 境の線に重なる帯を置き、CSS の値を変える | 依存なし。既定の見た目が変わらない | キーと読み上げを自分で付ける | **採る** |
| 仕切り: 分割の包み（Split.js など） | 出来合いの分割の部品 | 手数が少ない | 新しい依存（禁止）。第三者の表示・CSP・着色の面とのすり合わせが増える | 採らない |
| 仕切り: CSS の `resize` | ブラウザのつまみ | 1 行 | 角のつまみだけで、境の線を引けない。キーで動かせない（11.5） | 採らない |
| 入れ替え: DOM の順を変える | 欄の要素を並べ直す | Tab と読み上げの順が見た目と同じ | スクロールの位置を保つ手当てが要る | **採る** |
| 入れ替え: `flex-direction: row-reverse` | 見た目だけ逆にする | 1 行 | Tab の順が見た目と逆になる | 採らない |

### 8.4 Design Decisions

#### Decision: 並びは `layout.toml` に覚える

- **Context**: 要件 11.7（閉じて開き直しても残る）。管理役の指示（`recent.toml` の書き方を見てから決める）。
- **Alternatives Considered**: 上の表の 3 つ。
- **Selected Approach**: `%APPDATA%\io.github.ekicyou.pasta-recipe\layout.toml`。キーは `tree_width`・`panels_height`・`output_share`・`swapped`。操作の終わり（`pointerup`・矢印のキーの `keyup`・入れ替え）に 1 回書く。
- **Rationale**: アプリの今の型と同じ。タイマーもまとめ書きも要らない。撮影の舞台の配置が、ファイルを書くだけで決まる。
- **Trade-offs**: 要件 7.7・7.8 が挙げる記録（`recent.toml`）のほかに、撮影が写して戻すファイルが 1 つ増える。
- **Follow-up**: 要件 7.7・7.8 の文に `layout.toml` を足すかを、設計ディスカッションで決める。

#### Decision: 覚えた値と当てる値を分け、窓の最小を足す

- **Context**: 要件 11.6（最小）・11.8（窓の大きさを変えても、覚えた並びを窓に収まる範囲で保つ）。
- **Selected Approach**: 覚えた値は作者の操作でだけ変わる。窓が狭いときは `fitLayout` が最小を守って収めた値を当てるだけで、覚えた値は書き換えない。窓には `minWidth` 400・`minHeight` 320 を足し、最小が必ず守れるようにする。
- **Rationale**: 窓を一度縮めただけで覚えた並びが潰れるのを防ぐ。「収まる範囲で保つ」の読みに合う。
- **Trade-offs**: 窓の最小の大きさは、今は無い振る舞いである（今はいくらでも縮められる）。
- **Follow-up**: 最小の数は机の上の数。実物で確かめて直す。

#### Decision: 既定の見た目を変えない

- **Selected Approach**: 仕切りは、今の 1 px の境の線に重なる 5 px の透明な帯にし、場所を取らせない。既定の値（260・180・半分ずつ）は CSS に残す。
- **Rationale**: `layout.toml` の無い人の画面が 1 px も動かない。13 節の実測の数字がそのまま使える。

#### Decision: 仕切りは出力｜保存変数だけ（書く面は半分ずつのまま）

- **Context**: 要件 11.3 は出力と保存変数の境だけを挙げる。
- **Selected Approach**: 書く面（診断｜検査）には仕切りを置かない。下の欄の高さは 2 つの面で共通にする（面を切り替えても高さが変わらない）。
- **Trade-offs**: 作者が書く面でも境を動かしたくなるかもしれない。頼まれていないので足さない。

#### Decision: 撮影の並びは `layout.toml` に書いてから起動する

- **Selected Approach**: 記録の写しを取った後、撮影の並びを書いて pasta-recipe を起動する。仕切りは引きずらない。
- **Rationale**: 立て直しのたびに同じ並びになる。撮り直しの記録に 4 つの数を書くだけで再現できる。

#### Decision: 寄りは 1.35 倍・画は 4 つ・見出しは 6 つを右上に

- **Context**: 要件 2.6（文字の大きさ）・2.8（操作ごとの見出し）・2.10（結果を同じ画か続いた画に）。`direction.md` 5 節・7 節は、13 節の前の配置（窓 1000×680）で書かれている。
- **Selected Approach**: 13 節の配置で画を計算し直した（8.2 節）。寄りは 1.35 倍。画は、引き・主の画・左下の画・左上の画。見出しは操作ごとに 6 つで、画の右上に置く（主の画の右上は、タブの列の右の空きに当たる）。
- **Rationale**: 1.25〜1.3 倍では 13 px の文字が 10 px に届かない。左上に置くと、主の画でタブの名前とコメントの行に乗る。
- **Trade-offs**: 左下の画では、見出しがエディタの行の終わりに少し乗る。`direction.md` の推奨（左上）と違う。
- **Follow-up**: 試し撮りのコマを見て、開発者と決め直す（要件 7.13）。

#### Decision: 作業ファイルは `C:\pasta-demo\` に集め、台本のスクリプトはリポジトリに置かない

- **Selected Approach**: 写し・記録の写し・台本・書き出しを `C:\pasta-demo\` の下に置く。`book/SCREENSHOTS.md` には台本の表と数字だけを残す。
- **Rationale**: 後片付けが 1 回の削除で済む（要件 7.8）。スクラッチパッドは再起動で消えうるので、戻すための写しを置けない。今の `SCREENSHOTS.md` の決まり（撮影のスクリプトも置かない）に合う。
- **Trade-offs**: 次に撮り直すときは、台本を表から書き起こすことになる。

#### Decision: 2 つの流れを 1 つの spec で進める

- **Context**: 設計の関門は「別々に動かせる責任の境が 2 つ以上あれば分ける」を求める。この spec には、製品のコード（要件 11）と撮影の手順（要件 1〜10）がある。
- **Selected Approach**: 分けない。順を固定する（流れ A → 流れ B）。
- **Rationale**: 開発者が 2026-11-15 に、要件 11 をこの spec に入れると決めた（`requirements.md` の前提 A1・Boundary Context）。撮影は要件 11 を入れたビルドに依るので、別々には進められない。
- **Trade-offs**: `design.md` が長い（約 970 行）。要件が 72 項あり、対応の表だけで 70 行を超える。

#### 統合（Synthesis）の結果

- **まとめたもの**: 3 本の仕切りは「どの値を・どの向きに・どこまで」の違いだけなので、決まりを 1 組の関数（`SplitterId` で分ける）にし、取り付けも 1 つの関数にした。仕切りごとの部品は作らない。
- **作るか・使うか**: 記録は今ある型（`recent.rs`）、仕切りは標準の機能（Pointer Events・`role="separator"`）、撮影は今ある道具（Openscreen・CDP・SSP の MCP）を使う。新しく作るのは、並びの決まりと糊だけである。
- **削ったもの**: 書く面の仕切り・窓の位置と大きさを覚えること・ダブルクリックでの初期化・保存のまとめ書き（タイマー）・リポジトリに置く撮影のスクリプト・白の窓（背景）・ゴーストを手前にする細工。

### 8.5 Risks & Mitigations

- **最小の数と窓の最小が実物に合わない** — 数は `LAYOUT_MIN` と `tauri.conf.json` の 1 か所ずつ。手元のビルドを CDP で確かめて直す。
- **入れ替えで出力の欄のスクロールが頭に戻る** — 並べ替えの前後で位置を保つ。画面の確かめの項目に入れる。
- **CDP の文字の入力が、前面でない窓の Monaco に入らない** — 焦点のまね（CDP）で補う。試し撮りの前の通し稽古で確かめる。入らなければ開発者に示す（窓を前面にするのは、開発者の作業の焦点を奪う）。
- **`\![set,balloonoffset]` が読み込み直しで戻る** — 送った後と ▶ の後に矩形を読む。戻るなら開発者のドラッグに頼む。
- **普段の SSP が動いているときの送り口が未確認** — 送る前の確かめ（`get_active_ghost_list` が `hello-pasta-demo` だけ）を必ず通し、通らなければ送らない。
- **読み取り専用の SSP で、書き換えた保存変数が読み込み直しの後に残らない（「100 回目」にならない）** — 通し稽古で確かめる。残らなければ、流れの締めを開発者に諮る。
- **寄りの中心・見出しの位置が、切り抜いた画に対する割合でない** — 試し撮りで 1 つ置いてコマで見る。違えば式を直す。
- **書き出しが 5 MB を超える・無音のトラックが付く・字がにじむ** — 要件 7.12 の ffmpeg（詰め直し・除去）と、書き出しの高さの切り替え。
- **録画の間にタスクバーや通知が出る** — 開発者への頼みごと（要件 7.15）と、コマの確かめ（要件 3.3）。出たら撮り直す。
- **開発者の pasta-recipe が動いていて、撮影用の起動がそちらへ渡る** — 前の確かめで見つけて止まる。

### 8.6 References

- `editors/recipe/src-tauri/src/recent.rs` — 記録の型の手本
- `editors/recipe/src/panels.ts`・`src/style.css`・`index.html` — 仕切りを足す所
- `.kiro/specs/recipe-demo-video/direction.md` 13 節 — 配置の基準（実測）
- `book/SCREENSHOTS.md` — 撮影の記録の型と決まり
- `.github/workflows/manual.yml` — マニュアルの検査の順
- [UKADOC さくらスクリプトリスト `\![move]`](https://ukatech.shillest.net/ukadoc-spec/list_sakura_script.html#bang-move)・[`\![set,balloonoffset,x,y]`](https://ukatech.shillest.net/ukadoc-spec/list_sakura_script.html#bang-set-balloonoffset-x-y) — 位置の合わせ方
- WAI-ARIA Authoring Practices「Window Splitter」— 仕切りのキーと読み上げの型
