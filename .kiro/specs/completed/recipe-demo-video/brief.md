# Brief: recipe-demo-video

> **ステータス**: 未着手（2026-11-15 の `/kiro-discovery` で起票）。版には縛らない（マニュアルは main から GitHub Pages へ出るので、リリースを待たずに届く）。要件定義は Opus。

## Problem

マニュアルの説明動画は、VSCode 拡張で撮った 1 本だけである。0.6.0 で VS Code を使わないエディタ pasta-recipe を出したが、その動きを見せる動画は無い。

- VS Code を入れない作者は、pasta-recipe でシーンを鳴らす・直す・読み込み直す流れを、文章と表だけで読むしかない。
- 開発者は、説明動画を pasta-recipe で撮り直したい。撮影には Openscreen を使う（開発者の指示）。

## Current State

2026-11-15（main `41cf4790`、v0.6.1）に確かめた。

- **今の動画**: `book/src/debug/media/dev-actions-demo.mp4`（約 1.4 MB・音声なし）。`book/src/debug/dev-actions.md` の節「デモ：書いて、鳴らして、直す」が `<video>` で載せている。VSCode の上に SSP のゴーストが重なった画面で、右クリックのメニューから「▶ シーンを実行」→ 台詞を書き換えてもう一度 ▶ →「SHIORI をリロード」→「保存変数を表示」の流れを撮ったもの（`vscode-mcp-client` で撮り直した）。
- **pasta-recipe の章**: `book/src/debug/recipe.md` の節「ゴーストを動かす」が、▶ シーンを実行・値を入れて実行…・SHIORI をリロード・出力の欄・保存変数の表（書き換え…・消す…・追加…）を表と文章で書いている。絵も動画も無い。
- **マニュアルの検査**: `book/tools/verify-static.mjs` が、章の中の `<video>` の `src` などの参照先が在るかを調べる。動画のファイルを消す・名前を変えるときは、参照も同じ変更で直す。
- **Openscreen**: `%LOCALAPPDATA%\Programs\Openscreen\Openscreen.exe` に入っている。アプリが動いている間だけ、MCP の窓口（`127.0.0.1:47821`）が開く。道具は `listSources`・`startRecording`・`stopRecording`（projectId と checkpointId を返す）・`addTrim`・`setSpeed`・`addZoom`・`setPointerTrack`・`exportVideo`・`getFrames` など。本物のマウスで操作すれば、ポインタを記録してクリックに寄るズームを付ける。スクリプト（CDP など）で操作したときは、画面のポインタが動かないので、操作の記録を `stopRecording` の後で `setPointerTrack` に渡す。
- **pasta-recipe の画面の動かし方**: WebView2 のリモートデバッグのポート（`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333`）を開けて、CDP で本物のキーとマウスを送るのが確実（2026-11-13 の実装では computer-use は開発者の操作で 2 回とも止まった）。開いたゴーストは `%APPDATA%\io.github.ekicyou.pasta-recipe\recent.toml` に残る。

## Desired Outcome

- `recipe.md` の節「ゴーストを動かす」に、pasta-recipe で撮った説明動画が載っている。動画の前に、何をしている流れかを 1 段落で書いてある（今の `dev-actions.md` のデモの段落と同じ書き方）。
- `dev-actions.md` の VSCode の動画は消えていて、デモの節は pasta-recipe の章の動画へのリンクの 1 行になっている。
- マニュアルの検査（`verify-static.mjs` を含む）が通る。
- 撮影の後、開発者の環境は元のまま（`recent.toml`・SSP・hello-pasta の写し）。

## Approach

Openscreen で画面を撮り、Openscreen の編集の道具で切り詰めと速さを整えて mp4 に書き出す。

1. hello-pasta の写しを、読み取り専用で別のポートの SSP に出す（デバッグを有効にする。手順はメモ `ssp-ghost-test-launch` のとおり）。
2. pasta-recipe をリモートデバッグのポート付きで起動し、その写しを開く。
3. Openscreen の `listSources` で撮る範囲（pasta-recipe の窓と SSP のゴーストが入る画面）を選び、`startRecording` で撮り始める。
4. CDP で次の流れを操作し、各操作の時刻と位置を記録する。
   - ツリーかエディタの見出しの上から「▶ シーンを実行」→ ゴーストが喋る。出力の欄に結果が出る。
   - 台詞を 1 行書き換えて、もう一度 ▶ → 書き換えた台詞で喋る。
   - 「値を入れて実行…」でローカル変数を 1 つ渡して実行する。
   - ツールバーの「SHIORI をリロード」。
   - 保存変数の表を出し、値を 1 つ「書き換え…」で変える。
5. `stopRecording` の後、操作の記録を `setPointerTrack` に渡してポインタとクリックのズームを付け、待ちの間を `addTrim`・`setSpeed` で詰める。60〜90 秒・音声なしを目安にする。
6. `exportVideo` で mp4 に書き出し、`getFrames` で各場面を見て確かめる。大きさは今の動画と同じくらい（数 MB）に収める。大きすぎれば ShareX 同梱の ffmpeg（`C:\Program Files\ShareX\ffmpeg.exe`）で圧縮する。
7. `book/src/debug/media/recipe-dev-actions-demo.mp4` に置き、`recipe.md`・`dev-actions.md` を直し、`dev-actions-demo.mp4` を消す。

**選んだ理由**: 開発者が Openscreen を指名した。CDP で操作すれば開発者のデスクトップを奪わない。動画は、映している画面と同じ章に置くのが分かりやすい。

**採らなかった案**: VSCode の動画を残して 2 本立てにする（同じ流れの動画が 2 本になり、VSCode の側は撮り直しの手間が続く）。pasta-recipe の動画を `dev-actions.md` に置く（VSCode の章の説明と画面が食い違う）。

## Scope

- **In**: pasta-recipe での撮影と編集と書き出し、`book/src/debug/media/` の動画の差し替え、`recipe.md` の動画と説明の段落、`dev-actions.md` のデモの節の書き換え、マニュアルの検査を通すこと、撮影の後の後片付け。
- **Out**: pasta-recipe・VSCode 拡張・pasta.dll の振る舞いの変更。ほかの章の絵の撮り直し。音声や字幕を付けること。

## Boundary Candidates

- 撮影（環境の準備・CDP の操作の台本・Openscreen での録画）
- 編集と書き出し（切り詰め・速さ・ポインタ・mp4 の大きさ）
- マニュアルの文章（2 つの章と動画の参照）

## Out of Boundary

- 撮影中に pasta-recipe の不具合を見つけたときの直し。根の同じものはこの spec で直す案を推奨にする（起票に逃がさない）が、アプリの振る舞いを変える大きなものは開発者に諮る。
- 入門ガイドの絵（`getting-started-screenshots` の範囲）。

## Upstream / Downstream

- **Upstream**: `pasta-recipe`・`pasta-recipe-dev-actions`（入った。撮る画面）、`vscode-mcp-client`（今の動画を撮った）、`manual-print-media-refs`（章の動画の参照の扱い）。
- **Downstream**: pasta-recipe の画面が大きく変わったときの撮り直し。

## Existing Spec Touchpoints

- **Extends**: なし（完了した spec の成果物の差し替え）。
- **Adjacent**: `pasta-recipe`（`recipe.md` の章を書いた）、`vscode-mcp-client`（`dev-actions.md` の章を書いた）。

## Constraints

- 撮影は Openscreen を使う（開発者の指示）。Openscreen の MCP の窓口は、アプリが動いている間だけ開く。つながらないときは、開発者に `/mcp` での再接続を頼む。
- pasta-recipe の操作は CDP で行い、computer-use は使わない。ポインタは `setPointerTrack` で付ける。
- 起動中の SSP にはゴーストを足さない。別の readonly の SSP を別のポートで使い、終わりは `\-` で閉じる（WM_CLOSE は使わない）。
- `recent.toml` は撮影の前に写しを取り、撮影の後で戻す。
- 動画に開発者の個人の情報（ユーザー名の入ったパス・ほかの窓）を映さない。撮る範囲と、映るパスを確かめる。
- 動画は git で追跡する（今の動画と同じ）。大きさは数 MB に収める。
