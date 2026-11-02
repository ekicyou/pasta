# Brief: vscode-mcp-client

> **ステータス**: 未着手（2026-11-02 の `/kiro-discovery`「Pasta DSL MCP移行検討」で起票）。0.5.0（開発者「0.5.0にしてもウェーブ数変わらなさそう」）。要件定義は Opus。対になる spec は `debug-mcp-runtime`（pasta.dll の側）。

## Problem

VSCode でゴーストを書く作者が困っている。

- 「▶ シーンを実行」と「SHIORI をリロード」は、DAP のセッションに attach していないと動かない。デバッガーの制御は不安定である。
- どのゴーストに送るかは、固定の `127.0.0.1:9276` で決まっている。拡張は、編集中のファイルがどのゴーストのものかを見ていない。
- 一方で、SSP と areka は MCP の窓口を持ち、エージェントからの操作は安定してきた。ghost_dev「ぱすた開発」は、SSP の MCP だけで開発を回している（2026-11-02 の聞き取り）。

## Current State

2026-11-02（main `1a02383a`）に確かめた。

- **`editors/vscode/package.json`（324 行）**
  - `activationEvents` に `onDebug:pasta`。
  - コマンドは 3 つ。`pasta.debug.toggleSourcePresentation`・`pasta.runSceneAtCursor`・`pasta.reloadShiori`。
  - `breakpoints`（pasta と lua）と `debuggers` の `type: pasta`（attach。host、port 9276、sourcePresentation、initialConfigurations、snippet）。
  - メニューの `when` に `debugType == 'pasta'`。
- **DAP の側のファイル**
  - `src/debugAdapterFactory.ts` 74
  - `src/debugAttachTarget.ts` 99（既定 `127.0.0.1:9276`）
  - `src/runSceneAtCursor.ts` 106（`pasta/playSceneAt`、`{uri, line}`）
  - `src/reloadShiori.ts` 106（`pasta/reloadShiori`。初回は 3 秒待ち、1 秒ごとに再試行、15 秒で打ち切り）
  - `src/sourcePresentationToggle.ts` 69
  - `src/extension.ts` 467 のうち、56-67・138-250・287-362・389-450 行
- **テスト**
  - DAP 専用: `debugAdapterFactory.test.ts` 137・`sourcePresentationToggle.test.ts` 152・`runSceneAtCursor.test.ts` 114・`runSceneAtCursorCommand.test.ts` 239・`reloadShiori.test.ts` 140・`reloadShioriCommand.test.ts` 234。
  - 一部だけ関わるもの: `contributions.test.ts`・`vscodeModules.test.ts`・`mock/vscode.ts`。
- **ゴーストの特定**: していない。`src/ghostValidator.ts:21` の `findGhostMaster()`（`.pasta` の祖先の `ghost/master` を探す）は使える。
- **MCP の窓口**
  - areka: `http://127.0.0.1:{port}/api/mcp/v1`。JSON-RPC 2.0 を HTTP の POST（`application/json`）で受ける（areka `crates/areka-mcp/src/dispatch.rs:32`）。
    - 番号は 9801・9821 の早い者勝ち。両方塞がっていれば隣の番号へ逃げ、9802・9822…9810・9830 の 20 個（`crates/areka-mcp/src/port.rs`）。
    - `AREKA_MCP_PORT` で変えられる。
  - SSP: 9801。案内は `/api/mcp/help`。Claude Desktop 向けには stdio の中継 `mcp.exe` を使う（SSP ヘルプ「その他の機能」）。
    - **HTTP の道（`/api/mcp/v1` かどうか）は、まだ確かめていない**（2026-11-02 は SSP が起動していなかった）。
  - 道具は両方とも同じ: `get_active_ghost_list`・`get_status`・`get_property`・`get_log`・`sakurascript`・`raise_event`・`reload`・`dump_surface`・`dump_balloon`・`get_expression_table`。areka には `check_script` もある。
  - `ghost_name` には、ゴースト名のほか、**ゴーストのルートフォルダ（ghost と shell を含むフォルダ）のフルパス**も渡せる（SSP ヘルプ。areka の道具の説明も同じ）。
- **ぱすた開発の聞き取り**（2026-11-02。数は呼び出し回数）
  - 9801 には areka が答えることがある。毎回 `get_property(baseware.name)` で SSP かを確かめている（18 回。誤って areka へ送ったことがある）。
  - 反映は `reload(target="shiori")` で行う（79 回）。確かめは `get_log(error)`・`pasta.log`・転写キャッシュの 3 つ。
    - 足りないもの: reload の応答に、読み込みの成否と誤りの数が無い。
  - 再生は `raise_event(event=<シーン名>, strict=true)` で行う。前方一致の抽選なので、1 本だけを指名できない。
    - `OnPastaScene` は完全一致（`co_exec_exact`）だが、使われていなかった。
  - `raise_event` が「No response from server」を返すことがとても多い（約 170 回）。実際には再生されているので、毎回 `get_log(script)` で読み直している。
  - 保存変数は `profile/pasta/save/save.toml` を直接読んでいる。動いている間の値は見えない。
  - DAP は一度も使っていない。無くなって困ることはない。

## Desired Outcome

- 拡張から DAP の部品（`debuggers`・`breakpoints`・`onDebug:pasta`・提示モードの切り替え・attach）が無くなる。
- 拡張が、編集中の `.pasta` のゴーストを見つけ、そのゴーストが動いているベースウェア（SSP か areka）の MCP へ直接送る。
- コマンドは 3 つ。
  1. **▶ シーンを実行**（カーソルの位置のシーン）。毎回 SHIORI をリロードしてから再生する（開発者の指示、2026-11-02）。
     1. 未保存なら保存する（勧めるのではなく、黙って保存する）。
     2. `reload(target="shiori", ghost_name=<ルートの絶対パス>)` を送る。
     3. 読み込みが終わるのを待つ。
        - `reload` の応答が完了まで待つかは、設計の最初に SSP と areka の実機で確かめる。
        - 待たないなら `get_status` などを短い間隔で見る。
     4. 読み込みの誤りが増えていれば、再生せずに誤りの要約を出す。
     5. 誤りが無ければ `raise_event(event="OnPastaSceneAt", references=[<.pasta の絶対パス>, <行>], ghost_name=…)` を送る。
     6. 出た台本を出力のパネルに出す。
        - 応答が「No response from server」でも失敗とはしない。`get_log(script)` で台本を引く。
     - `OnPastaSceneAt` が 204 で何もしなかったら（デバッグが無効）、「`pasta.toml` に `[debug] enabled = true` を書いてリロードしてください」と案内する。
  2. **SHIORI をリロード**: `reload(target="shiori")` の後、`get_log(error)` と `pasta.log` の増えた分を要約して出す（ぱすた開発の「リロードの要約がほしい」）。
  3. **保存変数を表示**: `raise_event(event="OnPastaDumpVariables")` の後、`profile/pasta/debug/variables.toml` を読んで表示する（イベントは `debug-mcp-runtime`）。
- 利用者のマニュアルの章（`book/src/debug/`）が、MCP を前提に書き直されている。

例（編集中のファイル `C:\…\ghost_dev\debug\pasta-in-windows-debug\ghost\master\dic\pose.pasta` の 42 行目で「▶ シーンを実行」）:

1. 上へ 3 フォルダまで遡り、`ghost\master\descript.txt` を見つける。
2. ルートは、その 2 つ上の `…\pasta-in-windows-debug`。表示名は descript の `name`（「窓際のぱすたさんDEBUG」）。
3. 9801・9821・9802…の順に `get_active_ghost_list` を聞き、このゴーストが動いている相手を選ぶ。`get_property(baseware.name)` で相手の名前を出す。
4. 保存 → `reload` → 待つ → `OnPastaSceneAt`（`["C:\\…\\dic\\pose.pasta","42"]`）→ 出た台本を表示する。

## Approach

- **ゴーストの特定**
  - `.pasta` から上へ 3 フォルダまで遡り、最初に見つかった `descript.txt` を使う（開発者の案）。`findGhostMaster()` があれば使う。
  - **送るのはルートフォルダの絶対パス**。descript の `name` は表示に使う（開発者の承認、2026-11-02）。
    - 同じ名前のゴースト（本体と DEBUG 版）が並んでも、取り違えない。
    - 編集中のワークツリーの写しは、動いているゴーストと別物として扱われる。
  - descript が見つからなければ、ゴーストのフォルダではないと伝えて止める。
- **ベースウェアの見つけ方**
  - areka と同じ候補の列（9801・9821・9802・9822…9810・9830）を順に聞く。
  - 聞くのは `get_active_ghost_list` で、ゴーストが動いている相手を選ぶ。両方で動いていれば QuickPick で選ばせる。
  - 選んだ相手は、ゴーストごとに覚えておく（この窓の間だけ）。
  - 相手の名前は `get_property(baseware.name)` で出す。
- **通信**
  - Node の組み込みの `fetch` で JSON-RPC（`initialize` → `tools/call`）を送る。依存も `package-lock.json` も増やさない（`ponytail-editor-tooling` の見立て）。
  - SSP の HTTP の道が違えば、設計で合わせる。stdio の `mcp.exe` を起こす案は、HTTP で届かないときの控えにする。
- **撤去**
  - DAP の側のファイルを消す: `debugAdapterFactory.ts`・`debugAttachTarget.ts`・`sourcePresentationToggle.ts`。`runSceneAtCursor.ts`・`reloadShiori.ts` は MCP で書き直す。
  - `package.json` の `debuggers`・`breakpoints`・`onDebug:pasta`・`debug/toolBar`・`debugType == 'pasta'` の `when`。
  - DAP 専用のテスト 6 本。`contributions.test.ts`・`vscodeModules.test.ts`・`mock/vscode.ts` の debug の部分。
- **マニュアル**（担当は利用者の章。internals とリファレンスは `debug-mcp-runtime`）
  - `book/src/debug/`
    - `source-level.md`（158 行。ブレークポイント・ステップ・変数・提示モード）と `constraints.md`（44 行）は消す。
    - `index.md`・`vscode-setup.md`・`dev-actions.md`・`troubleshooting.md` を MCP を前提に書き直す。`dev-actions.md` の動画 `media/dev-actions-demo.mp4` は撮り直す。
    - `validate.md` は `pasta_check` の話なので残す。
  - 「AI エージェントから MCP で使う」節を足す。相手の確かめ（`baseware.name`）、`reload(shiori)`、`OnPastaScene`（完全一致の名前で 1 本を指名）と `OnPastaSceneAt`、`OnPastaSetSaveVariable`・`OnPastaDumpVariables`、「No response」のときの `get_log(script)`。スキル（`pasta-ghost-authoring` か `pasta-lua-coding`）にも同じ手順を載せる。
  - `SUMMARY.md:56-62`・`:82`、`editors/vscode/README.md:14`、ルートの `README.md`。

採らなかった案:

- **descript の `name` を `ghost_name` に送る案**: 本体と DEBUG 版のように同じ名前が並ぶと外れるので、表示の役に回した。
- **「リロードしない」設定**: 要望が無いので作らない。

## Scope

- **In**
  - DAP の部品の撤去
  - MCP のクライアント
  - ゴーストとベースウェアの見つけ方
  - コマンド 3 つ
  - ステータスバーか出力のパネル
  - 単体テストと e2e
  - `book/src/debug/` と、上の README・`SUMMARY.md`・スキルの手順
- **Out**
  - pasta.dll の側（`debug-mcp-runtime`）
  - ブレークポイント・ステップ・変数のツリー（DAP と一緒に捨てる）
  - 式の評価
  - areka・SSP の MCP そのものの直し

## Boundary Candidates

- MCP の窓口を見つけて呼ぶ部分（ポートの列・`ghost_name`・JSON-RPC）と、コマンド（保存・リロード・待ち・再生・表示）の 2 つ。
- 拡張のコードと、利用者のマニュアルの章。

## Out of Boundary

- LSP・診断・着色（`ponytail-editor-tooling`）
- `vsce` の版上げ（`vsce-4-upgrade`）

## Upstream / Downstream

- **Upstream**
  - `ponytail-editor-tooling`（0.5.0 のウェーブ 1。`editors/vscode` の `extension.ts`・`package.json`・e2e を共有する。あちらが先に入る）
  - `debug-mcp-runtime`（`OnPastaDumpVariables` と、門が残ること。決まりはこの brief に書いたので、並べて設計してよい）
- **Downstream**: `vsce-4-upgrade`（`editors/vscode` の後）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**: `ponytail-editor-tooling`、`debug-mcp-runtime`、`ponytail-manual-ci`（マニュアルの検査）、`vsce-4-upgrade`

## 受け継ぐもの（`ponytail-editor-tooling` から。2026-11-02）

開発者が範囲から外した（`ponytail-editor-tooling` の `f3c747cd`。要件 10 は欠番）。この spec で、DAP の側のファイルと一緒に片付ける。

- 監査の 10 番「デバッグの `host` 属性は、使い道の無い柔軟性」
  - `package.json` の `attach.host`・`initialConfigurations`・`configurationSnippets`
  - `debugAttachTarget.ts` L86-89
  - `runSceneAtCursor.ts` L61・L72
- 要件 12.3 のうち、`reloadShiori.ts` L18-22 の再輸出
- 要件 13.1: `book/src/debug/{source-level,troubleshooting,vscode-setup}.md` の `host` の記述

この spec が流れたら、`host` の廃止は宙に浮く。そのときは棚卸で拾い直す。

## Constraints

- **依存を増やさない**: Node の組み込みの `fetch` を使う。
- **送る先は 127.0.0.1 だけ**: 待ち受けが 127.0.0.1 固定である（SSP・areka とも）。
- **送る前に相手を確かめる**: SSP と areka は同じ番号を奪い合うので、番号だけで相手を決めない。
- **e2e**: ベースウェアの代わりに、手元で MCP を真似る小さな HTTP サーバーを立てて試す。SSP・areka の実機の確かめは、設計の最初の 2 点（HTTP の道・`reload` の応答の待ち）と、出す前の 1 回。
- **マニュアルの章の数**: 頁を消すと、章の数を決め打ちする検査がずれる（`ponytail-manual-ci` が決め打ちをやめていれば要らない）。

## 2026-11-03 棚卸の再測定（main c575febd）

- **前提の変化**: 先に要った `ponytail-editor-tooling` は入った。`extension.ts` は 464 行になった（`fallbackMode` が消えて 3 行減った。DAP の部分は 54〜65・135〜247・252〜360・366〜447 行あたり）。`e2e.test.ts` の起動の状態の試験は消え、`package.json` は `test:grammar` の 1 行だけ変わった。DAP の部品（`onDebug:pasta` 32 行・`debug/toolBar` 72 行・`breakpoints` 235 行・`debuggers` 243 行・`debugType == 'pasta'` の `when`）と DAP の側のファイルは手つかずで、Current State は今も成り立つ。
- **受け継ぐもの（`host`）は今も残る**: `package.json` 250・281・293 行、`debugAttachTarget.ts` 87〜88 行、`runSceneAtCursor.ts` 61・72 行、`reloadShiori.ts` 18〜22 行の再輸出、マニュアル `vscode-setup.md` 65・80 行・`source-level.md` 79 行・`troubleshooting.md` 88 行。
- **章の数**: `ponytail-manual-ci` が章の数を目次から数えるようにした。Constraints の「章の数」は、`source-level.md`・`constraints.md` を消すときに `SUMMARY.md`（58・60 行）を直すだけで済む。
- **触るファイル**: `editors/vscode/package.json` 324・`src/extension.ts` 464、消す `src/debugAdapterFactory.ts` 74・`debugAttachTarget.ts` 99・`sourcePresentationToggle.ts` 69、書き直す `runSceneAtCursor.ts` 106・`reloadShiori.ts` 106、MCP のクライアント（新しいファイル）、`ghostValidator.ts`（`findGhostMaster` を使う）、テスト（DAP 専用 6 本を消す。`contributions.test.ts`・`vscodeModules.test.ts`・`mock/vscode.ts` 477。MCP を真似るサーバーの e2e は新しいファイル）、`book/src/debug/` の 6 頁（`validate.md` は残す）、`SUMMARY.md` 56〜62 行、`editors/vscode/README.md` 14 行、ルートの `README.md`。`package-lock.json` と `syntaxes/pasta.tmLanguage.json` は触らない。
- **規模**: 12〜16 タスクのまま。
- **先に要るもの**: 機能の上では `debug-mcp-runtime` の `OnPastaDumpVariables` と門（決まりは brief にあるので、並べて設計してよい）。ファイルの重なり（未完了）: `vsce-4-upgrade`（`package.json`・`package-lock.json`。この spec の後）。属性の 3 本は `syntaxes/pasta.tmLanguage.json` と LSP だけを触るので重ならない。`scene-count` とも重ならない。`debug-mcp-runtime` とは `SUMMARY.md` の別の行だけ（許す）。ウェーブ 2 に置ける。
- **種別**: 機能（拡張が、編集中のゴーストが動いているベースウェアへ送れない。シーンの実行とリロードが DAP に頼っている）。開発者が承認した移行。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**:
  - 「スキルにも同じ手順を載せる」: `book/src/debug/` は、スキルの references を作る表（`book/tools/gen-skill-refs.mjs` 40〜63 行）に入っていない。マニュアルに書いてもスキルには流れない。表に頁を足すか、表にある頁（`lua/shiori-events.md` など。`debug-mcp-runtime` も触る頁の別の節）に書くかを、設計で決める。`SKILL.md` を手で直すのは分類器が拒む。
  - SSP の HTTP の道（`/api/mcp/v1` か）は、まだ確かめていない（設計の最初に確かめる）。
