# ギャップ分析: vscode-mcp-client

2026-11-03（worktree の base `96b75cbf`）に確かめた。要件は `requirements.md`（要件 1〜11）。

## Summary

- **Feature**: `vscode-mcp-client`
- **Discovery Scope**: Extension（既存の拡張の DAP の道を MCP の道に替える。撤去が半分）
- **Key Findings**:
  - 拡張の DAP の部品は brief の再測定のとおり手つかずで残る。`extension.ts` 464 行のうち約 300 行が DAP の糊で、撤去すると 150 行ほどに縮む。
  - 使える部品は 3 つある。`findGhostMaster()`・`ghostKey()`（`ghostValidator.ts`）、出力のパネル（`outputChannel`）、「vscode に触らない純粋なモジュール＋`extension.ts` の薄い糊」という分け方と、それを試す esbuild＋`mock/vscode.ts` の試験の型。
  - 持ち場の外に、直さないと CI が落ちる所が 2 つある。`book/tools/verify-content.mjs` の G 節（デバッグの頁 5 つと DAP の言葉 `9276`・`attach`・`sourcePresentation` を決め打ち）と、消す頁へのリンク（`debug-mcp-runtime` の持ち場の頁から）。
  - スキルへの手順の流し先は、もう在る。`lua/shiori-events.md` の「MCP からの呼び方」節は `gen-skill-refs.mjs` の表に入っていて（`pasta-lua-coding`）、「`reload` は終わるのを待たない。`get_status` が答えるようになってから呼ぶ」とも書いてある。
  - 実機で確かめないと決められないことが 4 つ残る（SSP の HTTP の道・読み込みの終わりの見分け方・「No response」と 204 の見分け方・ルートのパスでの動いているかの確かめ方）。brief のとおり設計の最初に確かめる。

## 1. 現状の調査

### 拡張（`editors/vscode/`）

| ファイル | 行 | この spec での扱い |
| --- | --- | --- |
| `package.json` | 324 | `onDebug:pasta`（32 行）・`pasta.debug.toggleSourcePresentation`・`commandPalette` と `debug/toolBar`（72 行）の `debugType == 'pasta'`・`breakpoints`（235 行）・`debuggers`（243 行。`host` 250・281・293 行）を消す。`test:unit` の esbuild の列から DAP の試験 4 本（`debugAdapterFactory`・`sourcePresentationToggle`・`runSceneAtCursor`・`reloadShiori`）と Command の 2 本を外し、新しい試験を足す。コマンドを 1 つ足す |
| `src/extension.ts` | 464 | 13〜30 行の import、54〜86 行の登録、135〜447 行の DAP の糊（`isPastaSession`・`registerSourcePresentationToggle`・`resolveStartDebuggingConfig`・`registerRunSceneAtCursorCommand`・`setReattachDelayForTests`・`registerReloadShioriCommand`）を MCP の糊に替える |
| `src/debugAdapterFactory.ts` | 74 | 消す |
| `src/debugAttachTarget.ts` | 99 | 消す（`DEFAULT_DEBUG_HOST`・`DEFAULT_DEBUG_PORT` を `runSceneAtCursor.ts` が import している） |
| `src/sourcePresentationToggle.ts` | 69 | 消す |
| `src/runSceneAtCursor.ts` | 106 | 書き直す。`pasta/playSceneAt`・`setPayload`・`resolvePastaDebugConfig`（launch.json を読む）・`host` 61・72 行は全部要らなくなる |
| `src/reloadShiori.ts` | 106 | 書き直す。18〜22 行の再輸出・再アタッチの待ちの表（3 秒・1 秒・15 秒）は要らなくなる。「一定の時間で打ち切る」純粋な関数の形は、読み込みの待ちに流用できる |
| `src/ghostValidator.ts` | 373 | `findGhostMaster()`（21 行。祖先の `ghost/master` を大文字・小文字を区別せずに探す。深さの上限なし。`descript.txt` は見ない）と `ghostKey()`（36 行。小文字にした URI。ゴーストごとの覚えの鍵に使える）を使う |
| `README.md` | — | 14 行「デバッガ接続」 |
| `CHANGELOG.md` | — | 版の記録。要件には無いが、DAP の撤去は利用者に見える変更なので 1 行足すかを設計で決める |

試験（`src/test/`）:

- DAP 専用の 6 本（`debugAdapterFactory` 137・`sourcePresentationToggle` 152・`runSceneAtCursor` 114・`runSceneAtCursorCommand` 239・`reloadShiori` 140・`reloadShioriCommand` 234）は消す。
- `contributions.test.ts` 184: メニューの `when`（`debugType`）と `debug/toolBar` を確かめている。新しい宣言（DAP の部品が無いこと・3 つのコマンドがいつでも出ること）に書き直す。要件 8.3 の置き場になる。
- `vscodeModules.test.ts` 1027: `PastaDebugAdapterFactory` などの試験（765 行〜）を消す。
- `mock/vscode.ts` 477: debug の API の真似（29 行ほど）を消す。`window.showQuickPick`・`window.withProgress` の真似は**無い**ので足す（要件 3.3・7.4）。`createStatusBarItem` は在る。
- `e2e.test.ts` 207: VSCode を起こさない素の node の試験（宣言・文法・ビルドの形）。MCP を真似るサーバーの e2e は、この型（esbuild で束ねて node で走らせる）に合わせて新しいファイルにできる。`node:http` は Node の組み込みなので依存は増えない。
- CI: `build.yml` に拡張の `npm test` の job は見当たらない（拡張の試験は手元の関門）。設計で確かめる。

### マニュアル（`book/src/debug/` ほか）

- 頁: `index.md` 106・`vscode-setup.md` 107・`source-level.md` 158・`dev-actions.md` 165・`constraints.md` 44・`troubleshooting.md` 102・`validate.md` 257（残す）。動画 `media/dev-actions-demo.mp4`。
- `SUMMARY.md` 56〜62 行がデバッグの章。消すのは 58 行（source-level）と 60 行（constraints）。`ponytail-manual-ci` が章の数を目次から数えるようにしたので、目次を直せば章の数の検査は合う（`talk-test.mjs` 456 行も目次から数える）。
- 各頁の頭はクローディアとアンソニーの台詞で始まる。`verify-content.mjs` の G 節が導入と締めの口調（`hasVoice`）を確かめる。
- ルートの `README.md`: 38 行（デバッグの章「VSCode から `.pasta` の行で止めて調べる」）・47 行（拡張「デバッガ接続」）。

### 隣の spec の決まり（`debug-mcp-runtime` の brief）

- `OnPastaScene`・`OnPastaSceneAt` の引数と応答は変えない（`scene_play.lua` 58〜86 行。Reference0 は `ghost/master/` からの相対か絶対のパス、Reference1 は `^%d+$` の 1 以上）。
- 門 `allowed()`: `@pasta_debug.enabled()` が偽か、SecurityLevel が `external` なら warn を出して `nil`（204）。warn の文言は `OnPastaSceneAt: debug is disabled; scene not played`。
- `OnPastaDumpVariables`（新設）: 保存変数を `profile/pasta/debug/variables.toml` に TOML で書き出し、喋らずに 204。書けない値は飛ばしてログに残す。
- 門は待ち受けを持たなくなる。`[debug] enabled` は load のたびに `pasta.toml` から読み直される。

### MCP の窓口（brief と、このセッションの SSP・areka の道具の説明から）

- `reload(target, ghost_name?)`: 「読み込みの誤りは error の記録に残る。`get_log` で確かめる」。応答に成否は無い。
- `get_log(log_type?, ghost_name?, since_id?, max_count?)`: 1 行 1 件 `#id time [type] name : text`。**`since_id` で増えた分だけを取れる**。`log_type` は `error`・`script`・`network`・`update`・`status`。要件 4.5・4.8・5.4 の「増えた分」はこれで取れる見込み。
- `raise_event(event, references?, ghost_name?, strict?)`: 返った台本を再生し、結果に台本を載せる。
- `get_status(ghost_name?)`: 空なら暇。`talking`・`changing` などの旗。
- `get_property(property_name, ghost_name?)`・`get_active_ghost_list()`。
- `ghost_name` は、名前のほか、ルートフォルダのフルパスを受ける。

## 2. 要件と資産の対応

| 要件 | 使える資産 | ギャップ |
| --- | --- | --- |
| 1 DAP の撤去 | 撤去の一覧は brief の再測定が正確 | **Constraint**: `verify-content.mjs` の G 節（後述 3.1）。**Constraint**: 消す頁へのリンク（3.2） |
| 2 ゴーストの特定 | `findGhostMaster()`・`ghostKey()` | **Missing**: `descript.txt` の有無と `name` の読み取り。**Unknown**: `descript.txt` の文字コード（hello-pasta は `charset,UTF-8`。古いゴーストは Shift_JIS のことがある。拡張の Node で Shift_JIS を読めるか）。**Unknown**: 深さ 3 の上限と、上限の無い `findGhostMaster()` の折り合い（未決 1） |
| 3 ベースウェアの見つけ方 | — | **Missing**: MCP のクライアント（`initialize` → `tools/call`）、20 個の番号の列、打ち切りの時間、相手の選択、ゴーストごとの覚え。**Unknown**: SSP の HTTP の道（`/api/mcp/v1` か）、MCP の窓口を SSP で有効にする設定の有無、`initialize` の後にセッションの印（`Mcp-Session-Id` など）が要るか。**Unknown**: ルートのパスで「動いているか」をどう確かめるか（`get_active_ghost_list` は名前を返す見込み。`ghost_name` にパスを付けた `get_status` の成否で見る案がある） |
| 4 ▶ シーンを実行 | 保存（`document.save()`）とカーソルの行の読み方は今の糊に在る | **Missing**: 読み込みの待ち、誤りの増加の判定、`OnPastaSceneAt` の送信、台本の表示、「No response」の読み直し、204 の案内。**Unknown**: 読み込みの終わりの見分け方（`reload` の直後は古い SHIORI がまだ答えるかもしれない）、「No response」と 204 の結果の形の違い |
| 5 SHIORI をリロード | 同上 | **Missing**: `get_log(error, since_id)` と `pasta.log` の増えた分の要約。**Unknown**: `pasta.log` の場所（既定は `ghost/master/profile/pasta/logs/pasta.log`。`[logging] file_path` で変えられる。未決 5） |
| 6 保存変数を表示 | — | **Missing**: コマンド・`OnPastaDumpVariables` の送信・ファイルの読み取り・新しく書かれたかの判定（送る前の更新時刻との比べ）。**Constraint**: `debug-mcp-runtime` の実装が先に入らないと実機では試せない（e2e は真似るサーバーで試せる） |
| 7 結果の出し方 | `outputChannel`（「Pasta Language」） | **Missing**: 進み具合の表示、同じゴーストへの重なりの止め。**Unknown**: 出力のパネルを今の「Pasta Language」に混ぜるか、別に作るか |
| 8 試験 | esbuild＋`mock/vscode.ts` の型、素の node の e2e | **Missing**: 真似るサーバー（`node:http`）、`showQuickPick`・`withProgress` の真似 |
| 9 マニュアル | 頁の骨組みと台詞の型 | **Constraint**: 他の頁からのアンカー（3.2）。動画の撮り直しは実機が要る |
| 10 AI エージェントの手順 | `lua/shiori-events.md`「MCP からの呼び方」（755 行〜。スキルの表に在る） | **Constraint**: `book/src/debug/` はスキルの表（`gen-skill-refs.mjs` 40〜63 行）に無い。`SKILL.md` を手で直すのは分類器が拒む（3.3） |
| 11 README | — | 書き換えるだけ |

## 3. 持ち場をまたぐ所（統合の難所）

### 3.1 `book/tools/verify-content.mjs` の G 節（持ち場の外）

- 442 行 `DEBUG_CHAPTERS = ['index', 'vscode-setup', 'source-level', 'constraints', 'troubleshooting']`。頁が無いと `G-exist` が落ちる。
- 490 行 `FACTS = ['9276', 'PASTA_DEBUG', 'attach', '.pasta', 'sourcePresentation']`。デバッグの章全体にこの言葉が無いと `G-fact` が落ちる。要件 9.2 で `9276`・`attach`・`sourcePresentation` を消すと、必ず落ちる。
- ウェーブの約束で、この spec の持ち場は `editors/vscode/`・`book/src/debug/`・README である。`book/tools/` は入っていない。けれど、頁を消すのも言葉を消すのもこの spec なので、直せるのもこの spec だけである。
- 案: 一覧を `['index', 'vscode-setup', 'dev-actions', 'troubleshooting']`（`dev-actions` は今も G 節に入っていない）にし、FACTS を `PASTA_DEBUG`・`.pasta`・`OnPastaSceneAt`・`[debug]` などの新しい言葉にする。`debug-mcp-runtime` は `book/src/debug/` を触らないので重ならない（未決 9）。

### 3.2 消す頁・変える見出しへのリンク

リンクの検証（`link-check.mjs`）は、章のファイルと見出しのアンカーの実在を見る。`book/src/debug/` の外から次が引かれている。

| 引く所 | 引く先 | 持ち場 |
| --- | --- | --- |
| `reference/pasta-toml.md:807` | `debug/index.md#pastatoml-による有効化`、`debug/source-level.md#提示モードの切替` | `debug-mcp-runtime`（`[debug]` 節を書き直す） |
| `internals/debug.md:11・126・273・392` | `debug/index.md`・`#優先順位`・`source-level.md#初期モードの指定`・`#サイドカー出力任意`・`constraints.md`・`dev-actions.md#シーン再生-シーンを実行` | `debug-mcp-runtime`（頁ごと書き直す） |
| `lua/shiori-events.md:707・729・1140` | `dev-actions.md#シーン再生-シーンを実行`・`index.md#デバッグの有効化` | `debug-mcp-runtime`（別の節）／この spec（要件 10 の節） |
| `grammar/call-jump.md:465・547` | `dev-actions.md#シーン再生-シーンを実行` | `debug-mcp-runtime`（言及の見直し） |
| `reference/startup.md:143` | `troubleshooting.md`（「デバッガが接続できない場合」） | `debug-mcp-runtime`（言及の見直し） |
| `introduction.md:19` | `debug/index.md` | `debug-mcp-runtime` |
| `DEBUGGING.md:12-15` | 頁の一覧（`source-level`・`constraints` を含む） | `debug-mcp-runtime` |

- 要件 9.9 で、`シーン再生-シーンを実行`・`デバッグの有効化`・`pastatoml-による有効化` の 3 つのアンカーを残すと、上の行の多くは触らずに済む。
- 残る切れ目は、`source-level.md`・`constraints.md` と `#優先順位` を引く行（`pasta-toml.md:807`・`internals/debug.md`）。どちらも `debug-mcp-runtime` が書き直す頁である。
- 先に入る側が CI を緑に保つ要がある。案は 2 つ（未決 10）。
  - (a) この spec が先に入るなら、その数行だけを最小に直す（リンクを外すか、新しい頁へ向ける）。後から入る `debug-mcp-runtime` が rebase で書き直す。
  - (b) `debug-mcp-runtime` が先に入るのを待つ。ウェーブで並べる意味が薄れる。
- `#優先順位`（`index.md` 66 行）は、門の優先順位（env > `pasta.toml`）を書く節として残せば切れない（要件 9.9 に足した）。

### 3.3 AI エージェントの手順をスキルへ流す道

- `gen-skill-refs.mjs` の `GENERATION_MAP` に `debug/` の頁は無い。`debug/` に書いても、スキルの references には流れない。
- `lua/shiori-events.md` の「MCP からの呼び方」節（755 行〜）は表に在り（`pasta-lua-coding`）、`OnPastaScene`・`OnPastaSceneAt`・`reload` の待ちを、もう書いている。足りないのは、相手の確かめ（`baseware.name`）・`ghost_name` にルートのパス・`OnPastaDumpVariables`・「No response」のときの `get_log(script)`・誤りの数え方（`get_log(error, since_id)`）。
- 案:
  - (a) その節を広げて、手順の正本にする。`debug/` の頁は短い案内とリンクにする。同じ頁の別の節を `debug-mcp-runtime` も触る（`OnPastaDumpVariables` の節・門の決まり）が、ウェーブの約束で「同じ頁の別の節は rebase で合わせる」と決まっている。`gen-skill-refs.mjs --check` の鮮度を合わせるため、references の作り直しも要る（`.claude/skills/pasta-lua-coding/references/shiori-events.md` は生成物。生成器で作るので `SKILL.md` の手直しには当たらない）。
  - (b) `debug/` に手順の頁（か節）を置き、`GENERATION_MAP` に足す。`book/tools/gen-skill-refs.mjs` と表の試験（`gen-skill-refs-test.mjs`）を触るので、持ち場の外へ出る。`SKILL.md` の references の一覧の行が要るなら、分類器が拒む。
  - (a) が持ち場の約束にも、分類器の制約にも合う（未決 8）。

### 3.4 その他

- 古い `launch.json` に `type: pasta` が残ると、VSCode は「種類が分からない」と出す。拡張では何もできないので、マニュアルで案内する（要件 9.5）。
- `editors/vscode/package.json` は `vsce-4-upgrade` が後で触る。この spec が先（roadmap のとおり）。
- `SUMMARY.md` は `debug-mcp-runtime` も 82 行（`internals/debug.md` の題）を触る。別の行なので rebase で合う。

## 4. 実装の案

### 案 A: 今のファイルを書き直すだけ

- `runSceneAtCursor.ts`・`reloadShiori.ts` に MCP の呼び出しまで全部入れ、`extension.ts` の糊を書き直す。新しいファイルは e2e だけ。
- ✅ ファイルが増えない。
- ❌ 番号の列・相手の選び方・JSON-RPC が 2 つのファイルに写しで入るか、片方がもう片方を import する形になる（今の `reloadShiori.ts` の再輸出と同じ臭い。`ponytail-editor-tooling` が外した要件 12.3 の再来）。保存変数の表示の置き場が無い。

### 案 B: 役目ごとに新しいモジュールを作る

- `mcpClient.ts`（JSON-RPC・番号の列・打ち切り）、`ghostLocator.ts`（`descript.txt`・ルート・表示名）、`devCommands.ts`（3 つのコマンドの流れ）を新しく作り、`runSceneAtCursor.ts`・`reloadShiori.ts` も消す。
- ✅ 役目の境がはっきりし、純粋な部分を試しやすい。
- ❌ 名前の変わるファイルが多く、差分が大きい。`ghostLocator` は `ghostValidator.ts` の `findGhostMaster()` と役目が重なる。

### 案 C: 混ぜる（brief の見立てに近い）

- 新しく作るのは MCP の窓口の部分だけ（`mcpClient.ts`: 番号の列・`initialize` → `tools/call`・打ち切り・相手の確かめ）。brief の境界の候補「MCP の窓口を見つけて呼ぶ部分」と一致する。
- `runSceneAtCursor.ts`・`reloadShiori.ts` は、vscode に触らない純粋な流れ（引数の作り方・誤りの増えた分の判定・台本の取り出し）として書き直す。保存変数の表示は、どちらかに足すか、小さな 1 本にする。
- ゴーストの特定は `ghostValidator.ts` の `findGhostMaster()` を使い、`descript.txt` の確かめと `name` の読み取りだけを足す（新しいファイルにしない）。
- `extension.ts` の糊は、コマンドの登録・保存・選択・出力・進み具合だけにする。
- ✅ 今の分け方（純粋なモジュール＋薄い糊）と試験の型を保つ。新しいファイルは 1〜2 本。
- ❌ `ghostValidator.ts`（373 行）に役目が 1 つ増える。

## 5. 手間と危うさ

- **手間: M（3〜7 日）**。撤去は機械的。MCP のクライアントと 3 つの流れは新しいが小さい。マニュアルの 4 頁の書き直しと動画の撮り直しが重い。brief の見立て（12〜16 タスク）と合う。
- **危うさ: Medium**。技術は知っている（Node の組み込みの `fetch`・`node:http`）。けれど、実機の挙動に頼る所（SSP の HTTP の道・読み込みの終わり・「No response」と 204 の見分け・パスでの動いているかの確かめ）が 4 つあり、外すと流れの形が変わる。

## 6. 設計への申し送り

### 推す形

- 案 C。MCP の窓口の部分を 1 本にし、コマンドの流れは今のファイル名のまま純粋なモジュールにする。
- AI エージェントの手順は 3.3 の (a)。
- `verify-content.mjs` の G 節は、この spec で直す（3.1）。

### 設計の最初に実機で確かめること（Research Needed）

1. SSP の MCP の HTTP の道。`/api/mcp/v1` か、`/api/mcp/help` の案内で何が書かれているか。SSP で MCP の窓口を有効にする設定が要るか。
2. `reload(target="shiori")` の応答は読み込みの終わりを待つか。待たないなら、終わりをどう見分けるか（`get_status` が答えるか・`get_log(status)` の行・`get_property` の何か）。直後に古い SHIORI が答えて「終わった」と見誤らないか。
3. `raise_event` の結果の形。台本があるとき・204（空）のとき・「No response from server」のときで、`tools/call` の結果（`content`・`isError`）がどう違うか。「No response」の後に `get_log(script, since_id)` で台本が引けるか。
4. `get_active_ghost_list` の返す形（名前だけか、パスも出るか）。ルートの絶対パスを `ghost_name` に付けた `get_status`・`get_property` が、動いていないゴーストに対して誤りを返すか（動いているかの確かめに使えるか）。areka と SSP で同じか。
5. `initialize` の後にセッションの印（`Mcp-Session-Id` など）を送り返す要があるか。SSP と areka で違うか。
6. 拡張の Node（VSCode `^1.85.0` の Electron）で、組み込みの `fetch`・`AbortSignal.timeout` が使えるか、`TextDecoder('shift_jis')` が使えるか（`descript.txt` の文字コード）。使えないなら `engines.vscode` を上げるかを決める（`vsce-4-upgrade` と同じ `package.json`）。
7. `OnPastaDumpVariables` の書き出しは応答より前に終わるか（`debug-mcp-runtime` の設計と合わせる）。「No response」でも書き出しは済むか。

### 要件の議論へ回す未決

`requirements.md` の要件は、下の仮の決めで書いてある。

1. **ゴーストの探し方の深さ**: 「上へ 3 フォルダまで」（brief）と、上限の無い `findGhostMaster()`。仮の決め: `.pasta` のフォルダから上へ 3 つまで（`dic/a/b.pasta` の `master` まで届く）の `ghost/master` で、`descript.txt` を持つもの（要件 2.1）。
2. **リロードのときも黙って保存するか**: brief は「▶ シーンを実行」だけを書く。仮の決め: リロードも確かめずに保存する（要件 5.2）。
3. **保存するのは今の文書だけか、同じゴーストの未保存の文書の全部か**: 仮の決め: 今の文書だけ（要件 4.2・5.2）。
4. **保存変数の見せ方**: 出力のパネルか、`variables.toml` をエディタで開くか。仮の決め: 出力のパネル（要件 6.2）。
5. **`pasta.log` の場所**: 既定の `ghost/master/profile/pasta/logs/pasta.log` だけを読むか、`pasta.toml` の `[logging] file_path` に従うか。仮の決め: 既定の場所だけを読み、無ければ要件 5.6 のとおり添え書きする。
6. **進み具合の見せ方**: ステータスバーか、進み具合の通知か。送り先を常に出すステータスバーの項目を置くか。仮の決め: やり取りの間だけ進み具合を出し、常に出す項目は置かない（要件 7.4）。
7. **同じゴーストへのコマンドの重なり**: 仮の決め: 進んでいる間は次を始めずに伝える（要件 7.5。brief には無い）。
8. **AI エージェントの手順の置き場**: 3.3 の (a) か (b)。仮の決め: (a)（`lua/shiori-events.md` の「MCP からの呼び方」を正本にし、`debug/` からリンクする。要件 10.2）。
9. **`book/tools/verify-content.mjs` を直すか**: 持ち場の外。仮の決め: この spec が直す（要件 9.8 を満たすのに要る）。
10. **他の spec の頁から消す頁へのリンク**: 3.2 の (a) か (b)。仮の決め: (a)。アンカー 3 つを残し（要件 9.9）、それでも切れる数行は、先に入る側が最小に直す。
11. **`AREKA_MCP_PORT` で番号を変えた areka**: 仮の決め: 追わない。候補の 20 個の外は見つからない（要件 3.9。設定を持たない）。
12. **「保存変数を表示」をどこから選べるか、`.pasta` の外（`pasta.toml`・`.lua`）からも 3 つのコマンドを選べるようにするか**: 仮の決め: `.pasta` のエディタの右クリックとコマンドパレット（今のエディタが `.pasta` のときだけ働く。要件 2.5・6.1）。

## 7. 危うさと手当て

- 実機の挙動が見立てと違う → 設計の最初に 7 つを確かめ、流れの形をそこで決める（brief のとおり）。
- `debug-mcp-runtime` が遅れて `OnPastaDumpVariables` が無い → 拡張は 204 と同じ扱い（ファイルが書かれない）になり、案内を出す。要件 6.3 のとおり古い中身は出さない。
- マニュアルの CI が持ち場の外で落ちる → 3.1・3.2 の手当てをタスクに入れる。
- 動画の撮り直しが実機の都合で遅れる → 出す前の実機の 1 回（brief）と同じ機会に撮る。

## 8. 要件の議論で設計へ回した事項（2026-11-03）

- 出力のパネルを今の「Pasta Language」に混ぜるか、別に作るか（要件 7.1）。
- `CHANGELOG.md` に DAP の撤去を書くか。
- 拡張の Node で `fetch`・`AbortSignal.timeout`・`TextDecoder('shift_jis')` が足りないとき、`engines.vscode` を上げるか（6 の 6）。
- 「No response from server」の後に、この再生の台本をどう見分けるか（`get_log(script, since_id)` の使い方。要件 4.8・4.9）。
- `pasta.toml` の `[logging] file_path` を、依存を増やさずにどう読むか（議題で決まった場合）。

## 9. 要件の議論の結論（2026-11-03・開発者が全議題を推奨案で承認）

- 未決 1（深さ 3）・11（`AREKA_MCP_PORT` は追わない）: brief のとおりで議題にしなかった。
- 未決 2・3 → 保存は、そのゴーストのルートの下の未保存の `.pasta`・`.lua`・`pasta.toml` を全部、確かめずに保存する。▶ シーンを実行と SHIORI をリロードの両方（要件 4.2・5.2）。
- 未決 12 → ▶ シーンを実行は `.pasta` だけから選べる。リロードと保存変数の表示は、`.pasta`・`.lua`・`pasta.toml` とコマンドパレットから選べる（要件 2.5・5.1・6.1）。
- 未決 5 → `pasta.log` は `[logging] file_path` に従う。書かれていなければ既定の場所（要件 5.6）。どう読むかは設計で決める（8 節）。
- 未決 4 → 保存変数は出力のパネルに出す（要件 6.2）。
- 未決 6・7 → 進み具合はやり取りの間だけ出す。常に出す項目は置かない。同じゴーストへのコマンドは重ねない（要件 7.4・7.5）。
- 未決 8 → 3.3 の (a)（要件 10.2）。
- 未決 9・10 → G 節はこの spec が直す（要件 9.8）。アンカー 4 つを残す。残る切れ目は、先に入る側が最小に直す（要件 9.9）。

---

# 設計の段の調べ（2026-11-03・`/kiro-spec-design -y`）

## Summary（設計の段）

- **Discovery Scope**: Extension（light discovery）。拡張の中の撤去と、外のベースウェアとの通信の新設。新しい依存は無い。
- **Key Findings**:
  - この机では SSP も areka も動いていなかった（実機のプローブはつながらなかった）。代わりに、SSP の中継 `mcp.exe` の中の文字列と、areka の受け口のソース・実測の記録（`areka/doc/ssp-mcp/`）から、HTTP の道・セッション・道具の答えの形をほぼ決められた。
  - 「No response from server」は SSP の本体の文ではなく、中継 `mcp.exe` の文である（本文が空か、応答の無いまま接続が閉じたとき）。拡張は HTTP で直に話すので、この文は届かない。代わりに「空の本文」「送った後に切れた」「時間切れ」として見える。
  - `reload` は、areka では終わりを待たない（答えに `It runs asynchronously` と書く）。読み込みの終わりは、`pasta.log` の `SHIORI load entry completed … loaded=true|false` の行で見分けられる（pasta 自身の行。ベースウェアに依らない）。
  - 動いているかの確かめは、`get_property(property_name="baseware.name", ghost_name=<ルートの絶対パス>)` の 1 回で済む。動いていれば `SSP`・`areka` の素の値、動いていなければ `NG:Cannot find active ghost from specified name`（SSP 2.9.07 の実測）。
  - SSP 2.9.05 の `get_log`（`script`）は、台本の逆斜線をエスケープしない不正な JSON を返す（areka の記録 survey §4.2）。読む側に寛容な読み直しが要る。

## Research Log（設計の段）

### 実機のプローブ（2026-11-03 00:3x UTC）

- **Context**: brief は「設計の最初に、SSP と areka の実機で確かめる」と書く。
- **やったこと**
  - `Get-NetTCPConnection -State Listen`（9790〜9840）: 待ち受け 0 件。`Get-Process`: `ssp.exe`・`areka.exe` は無し（`areka-mcp-bridge.exe` 2 つと `C:\wintools\ssp\data\mcp.exe` 2 つだけが動いている。どちらも中継で、本体ではない）。
  - `curl -s -m 2 -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"initialize",…}' http://127.0.0.1:{9801,9821,9802,9822}/api/mcp/v1` → どれも答え無し（終了コード 28）。`GET http://127.0.0.1:9801/api/mcp/help` → 空。
  - Claude Code の MCP（中継経由）: `mcp__ssp__get_active_ghost_list` → `Error: Server not available`。`mcp__areka__get_active_ghost_list` → `Server not available: areka-mcp-server not found on 127.0.0.1 ports 9801, 9821, 9802, 9822, …, 9810, 9830`。
- **結果**: 実機では何も確かめられなかった。`reload`・`raise_event` は一度も送っていない（pasta のゴーストが動いていないので、送ってよい相手が無かった）。下の机の上の調べで置き換え、確かめられなかったことは design.md の Open Questions に残した。

### SSP の HTTP の道（机の上の調べ）

- **Sources**: `C:\wintools\ssp\data\mcp.exe`（94,208 バイト・2026-09-24）と `C:\wintools\ssp\ssp.exe` の ASCII の文字列。areka `doc/ssp-mcp/survey.md` §1・§6（SSP 2.9.05 の実測）。
- **Findings**
  - `mcp.exe` は `POST /api/mcp/v1 HTTP/1.1`・`Host: 127.0.0.1:9801`・`Content-Type: application/json`・`Accept: application/json, text/event-stream`・`Connection: close` を焼き込んでいる。`ssp.exe` にも `/api/mcp/v1`・`/api/mcp/help`・`/api/sstp/v1` がある。
  - どちらにも `Mcp-Session-Id` の文字列は無い（ASCII・UTF-16 の両方で 0 件）。survey §2 も「SSE・`Mcp-Session-Id` を使わない＝無状態」。
  - `No response from server`・`Invalid JSON response from server`・`Server not available` は `mcp.exe` にだけある（`ssp.exe` には無い）。
- **Implications**: SSP の道は areka と同じ `/api/mcp/v1`。セッションの札は送り返さなくてよい。拡張の送り方は中継と同じ札にそろえる。

### areka の受け口（ソース）

- **Sources**: `C:\home\maz\git\areka\crates\areka-mcp\src\dispatch.rs`・`gate.rs`・`tools\*.rs`、`crates\areka\src\mcp\reload.rs`・`raise_event.rs`、`doc/ssp-mcp/transport-diff-areka.md`・`bridge-diff-areka.md`・`areka-tools.md`・`log-convention.md`。
- **Findings**
  - 無状態（`NeverSessionManager`・`legacy_session_mode(false)`）・JSON の単発応答。`initialize` を経ない `ping`・`tools/list` も 200 で、`Mcp-Session-Id` を返さない。
  - `POST` の `Accept` に `application/json` と `text/event-stream` の両方が無いと 406。`Host` はループバックだけ（無しも 403）。`Origin` は無しかループバックだけ（ほかは 403）。本文の上限 4 MiB。
  - `reload` の答え: `OK:reload requested. It runs asynchronously; errors while loading (dictionary errors etc.) will be recorded to the error log. Wait until get_status answers again, then check get_log tool (log_type=error, since_id=<n>)`。終わりを待たない。
  - `raise_event` の答え: 台本があれば 1 行目 `OK:the ghost returned a script and it is being played.`、続けて `Script: <台本>` の行。無ければ `OK:the ghost returned no script.`（SSP 2.9.05 と同じ文。survey §3・§8.2）。
  - ツールの上限 10 秒: `NG:areka did not respond within 10 seconds`（`isError: true`）。
  - `baseware.name` は `areka`（`areka-ghost/src/config.rs`）。SSP は `SSP`（survey §7.1）。
  - `get_log`: 古い順、1 行 `#<id> <yyyy/mm/dd hh:mm> [<表示の語>] <名> : <本文>`、続きの行はタブで始まる。0 件は `(no log entries)`。`max_count` は**新しい方から**の件数。番号は全種類の通し。`ghost_name` は名前かルートのフルパス。
  - `ghost_name` のパスの照合: 大文字・小文字・区切り（`/`・`\`）・末尾の区切りを同じとみなす。ルートでない上位のフォルダは外れ（survey §7.2・§7.4。SSP と areka で同じ）。
- **Implications**: 動いているかの確かめは `get_property(baseware.name, ghost_name=<ルート>)` の 1 回で済む。`get_active_ghost_list` は名前を返すので、ルートのパスでは引けない（brief の案から替えた。Design Decisions）。

### 読み込みの終わりの見分け方

- **Sources**: `crates/pasta_shiori/src/windows.rs:189`（`info!(entry, loaded, "SHIORI load entry completed")`）・`v3.rs:65`、`crates/pasta_lua/src/logging/logger.rs`（`Rotation::NEVER`・追記だけ）・`tracing_init.rs`（`with_ansi(false)`・`with_target(true)`・`with_level(true)`）。手元の `ghost_dev/debug/pasta-in-windows-debug/ghost/master/profile/pasta/logs/pasta.log`（48 MB・39 万行）。
- **Findings**
  - 行の形: `2026-11-02T13:50:23.883713Z  INFO pasta::windows: SHIORI load entry completed entry="loadu" loaded=true`。失敗では `loaded=false`（10-17・10-18 の実例あり）。
  - 誤りと警告は `ERROR`・`WARN` の語で始まる（例 `  WARN pasta_lua::loader::cache: …`）。
  - ログは回転しない追記のファイル。大きくなる（48 MB）ので、読む前の大きさを覚えて、増えた分だけを読む。
- **Implications**: 終わりの印は pasta.dll の行で決める。SSP と areka のどちらでも、`reload` が待っても待たなくても同じに働く。`level` を `warn` 以上にしたゴーストでは印の行が出ないので、時間切れになる（Risks）。

### `pasta.toml` の `[logging] file_path`

- **Sources**: `book/src/reference/pasta-toml.md` 756〜783 行、`crates/pasta_lua/src/loader/config/sections.rs`。
- **Findings**: 既定 `profile/pasta/logs/pasta.log`（`ghost/master/` から）。書けるのは `profile/` の下の相対パスだけ（`..` と絶対パスは不可。`profile` の大文字・小文字は区別する）。外れた値なら既定の場所へ書く。
- **Implications**: 拡張は TOML の読み手を持たない（依存を増やさない）。`[logging]` の表の中の `file_path = "…"`（基本の文字列か、リテラルの文字列）の 1 行だけを読む小さな読み手にし、pasta と同じ条件で外れたら既定へ倒す。WASM（`pasta_check`）に入口を足す案は `crates/pasta_lsp` に手が出る上、WASM が読めないときに働かないので採らない。

### `descript.txt` の文字コード

- **Findings**: `ghost_dev` の DEBUG 版は `charset,UTF-8`。古いゴーストは `charset` の行が無く Shift_JIS のことがある。
- **Implications**: 先頭の `charset,` の行（大文字・小文字を問わない）が `UTF-8` なら UTF-8、それ以外は `TextDecoder('shift_jis')`。読めなければ `name` は無しとし、ルートのフォルダの名前で表示する（要件 2.3）。

### 拡張の Node（VSCode `^1.85.0`）

- **Sources**: `editors/vscode/package.json`（`engines.vscode` `^1.85.0`・`@types/node` `^20`）、手元の VSCode 1.144.0（`ELECTRON_RUN_AS_NODE=1 Code.exe -e …`）、VSCode 1.85 の公開の版の事実（Electron 25.9・Node 18.15）。
- **Findings**
  - 手元の VSCode 1.144（Electron 43.8.3・Node 24.22.1）: `typeof fetch` は `function`、`typeof AbortSignal.timeout` は `function`、`new TextDecoder('shift_jis').decode([0x82,0xa0,0x8a,0xbf])` は `あ漢`。Electron は Chromium の ICU（`icudtl.dat`）を使う。
  - VSCode 1.85 の Node 18.15: 組み込みの `fetch`（undici。既定で有効）と `AbortSignal.timeout`（Node 17.3 から）はある。Shift_JIS は Electron の ICU の data による（1.85 そのものでは試していない）。
  - Node の `fetch` は `Origin` を付けない。`Host` は `127.0.0.1:<port>` になる（areka の検査を通る）。
- **Implications**: `engines.vscode` は上げない。TypeScript の型は `@types/node` 20 の大域の `fetch`・`AbortSignal` で足りる（`tsconfig` の `lib` は `ES2022` のまま）。1.85 での Shift_JIS は Open Question に残し、読めなければ表示名をフォルダ名へ倒す（壊れない）。

### 試験の型と CI

- `editors/vscode` の試験は CI では回らない（`build.yml` に拡張の `npm test` は無い。`release.yml` は `npm run package` だけ）。関門は手元の `npm test`。
- 今の型: 純粋なモジュールは素の node で試し、vscode に触る糊は `--alias:vscode=./src/test/mock/vscode.ts` で束ねる。`e2e.test.ts` は宣言とビルドの形を素の node で見る。
- MCP を真似るサーバーは `node:http`（組み込み）で立てる。番号は 0（空いている番号）で取り、候補の列を試験から差し込む（利用者の設定にはしない）。机の 9801 で動く SSP と取り合わない。

### スキルへの流し先と、消す頁へのリンク

- `.claude/skills/pasta-ghost-authoring/references/pasta-toml.md:802` が `debug/source-level.html#提示モードの切替` を引く（`reference/pasta-toml.md:807` の生成物）。`source-level.md` を消すと、スキルのリンクの検証も落ちる。先に入る側は、元の頁を直して `node book/tools/gen-skill-refs.mjs` で作り直す（生成物なので `SKILL.md` の手直しに当たらない）。
- `call-jump.md`・`shiori-events.md` の references は `debug/dev-actions.html#シーン再生-シーンを実行` を引く。見出し「シーン再生（▶ シーンを実行）」を残せば切れない。
- `editors/vscode/CHANGELOG.md` は 0.1.5 から更新されていない（版の記録は `doc/release-notes/`）。DAP の撤去の 1 行は書かない。

### 参照したスキル

- `kiro-spec-design`（手順・`rules/design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）、`.kiro/settings/templates/specs/design.md`・`research.md`。
- ponytail（最小の完全な変更。新しい部品は役目ごとに 1 本、設定を足さない）。
- `kiro-validate-design`（`rules/design-review.md`）は検証の段で使う。

## Architecture Pattern Evaluation（設計の段）

| 案 | 中身 | 強み | 弱み | 判断 |
| --- | --- | --- | --- | --- |
| C'（採用） | 純粋なモジュール 5 本（`mcpClient`・`ghostFiles`・`reloadShiori`・`runSceneAtCursor`・`dumpVariables`）＋ vscode の糊 1 本（`devCommands`）。`extension.ts` は登録を 1 行呼ぶだけ | 今の分け方と試験の型のまま。ベースウェアの言うこと（`mcpClient`）とゴーストのディスクの上のこと（`ghostFiles`）を分け、3 つの流れが同じ部品を使う | 新しいファイルが 4 本 | ギャップ分析の案 C を、共有の部品が 2 つあると分かった所で 1 段だけ割った |
| C（ギャップ分析の案） | `mcpClient` だけ新しく、ゴーストの特定は `ghostValidator.ts` に足す | ファイルが増えない | `ghostValidator.ts` は vscode を import するので、純粋な流れから使えない。`pasta.log` を読む部品の置き場が無い | 採らない |
| A | 今の 2 ファイルに全部入れる | 差分が小さい | 3 つ目のコマンドの置き場が無い。番号の列と JSON-RPC が写しになる | 採らない |

## Design Decisions（設計の段）

### Decision: 動いているかは `get_property(baseware.name, ghost_name=<ルート>)` で確かめる

- **Context**: 要件 3.1・3.2。brief は `get_active_ghost_list` で聞くと書いた。
- **Alternatives**: (1) `get_active_ghost_list` の名前と descript の `name` を比べる。(2) `get_status(ghost_name=<ルート>)`。(3) `get_property(baseware.name, ghost_name=<ルート>)`。
- **Selected**: (3)。
- **Rationale**: (1) は名前でしか比べられず、本体と DEBUG 版・ワークツリーの写しを取り違える（brief が `name` を送らないと決めた理由と同じ）。(3) は 1 回の呼び出しで「このパスのゴーストが動いているか」と「相手の名前」の両方が分かる。`get_status` は暇なとき空の本文を返すので、答えの形が弱い。
- **Follow-up**: SSP 2.9.07 の実測ではルートのパスで `SSP` が返った。動いていないパスでの文は実測の `NG:Cannot find active ghost from specified name`。

### Decision: 候補の 20 個は並べて聞く

- **Context**: 要件 3.1（順に聞く）・3.8（答えない相手で止まらない）。
- **Selected**: 20 個へ同時に送り、答えを候補の順に並べ直す。1 つの番号の上限は 2 秒。
- **Rationale**: 1 つずつ順に待つと最悪「上限 × 20」になる（areka の中継も同じ理由で並べて開く）。結果の並び（選択肢の順）は候補の順のまま保つ。

### Decision: 探すときだけ `initialize` を送る

- **Context**: brief は `initialize` → `tools/call`。SSP・areka はどちらも無状態で、セッションの札を使わない。
- **Selected**: 探すとき、番号ごとに `initialize`（`protocolVersion` `2025-06-18`）を 1 回送ってから `tools/call`。覚えた相手への以後の呼び出しは `tools/call` だけ。`MCP-Protocol-Version` の札は付けない（旧式の扱いで素通し）。
- **Rationale**: MCP の決まりに沿い、答えの `serverInfo` でも相手が MCP の受け口であることを確かめられる。無状態なので、毎回送り直す要は無い。

### Decision: 読み込みの終わりは `pasta.log` の印の行で待つ

- **Context**: 要件 4.3・4.4・5.3・5.8。areka の `reload` は待たない。SSP は未実測。
- **Alternatives**: (1) `get_status` が答えるまで待つ。(2) `reload` の答えを信じる。(3) `pasta.log` の増えた分に `SHIORI load entry completed` の行が出るまで待つ。
- **Selected**: (3)。印の `loaded=false` も「読み込みの誤り」に数える。
- **Rationale**: (1) は直後に古い SHIORI が答えて見誤る恐れがある。(2) は areka で外れる。(3) は pasta.dll 自身の行なので、ベースウェアに依らず、成否まで分かる。増えた分の誤りと警告も、同じ読み取りで取れる。
- **Trade-offs**: ログの量を `warn` 以上にしたゴーストでは印が出ず、時間切れになる。行の文は pasta の Rust の側にあり、文が変わると黙って壊れる → 拡張の試験が Rust のソースの文字列を読んで確かめる（Revalidation Triggers）。

### Decision: 「応答が無い」は 4 つの形をまとめて扱う

- **Context**: 要件 4.8・4.9。HTTP で直に話すと「No response from server」の文は届かない。
- **Selected**: `raise_event` で、⑴ 送った後に接続が切れた、⑵ 200 で本文が空、⑶ 拡張の上限（15 秒）の時間切れ、⑷ 答えの文が `No response from server` か `did not respond within` を含む、のどれでも「応答が無い」とし、`get_log(log_type="script", ghost_name=<ルート>, since_id=<送る前の最後の番号>)` を 5 秒まで引き直して台本を探す。
- **Rationale**: ぱすた開発の「No response」は中継が見た ⑴⑵ にあたる。areka の上限は ⑷。送り直しはしない（台本が 2 回流れる）。

### Decision: SSP の不正な JSON は読み直す

- **Context**: survey §4.2。`get_log`（`script`）の答えの JSON は、台本の `\` がエスケープされず読めない。
- **Selected**: `JSON.parse` が失敗したときだけ、JSON の決まりに無い `\x` を `\\x` に直してもう一度読む。直しても読めなければ「答えの形が違う」（要件 7.3）。
- **Trade-offs**: 台本の `\n`・`\t` などは JSON のエスケープとして読まれ、改行やタブになる（表示だけの崩れ）。

### Decision: 結果は専用の出力のパネル「Pasta 開発」に積む

- **Context**: 要件 7.1。研究の 8 節で設計へ回した。
- **Selected**: 新しい出力のパネル「Pasta 開発」を 1 つ作り、3 つのコマンドの結果だけを、頭に `[HH:MM:SS]` を付けて積む。今の「Pasta Language」（WASM の起動・ゴースト全体の検査の記録）とは混ぜない。
- **Rationale**: 作者が台本と要約を見返す場所に、拡張の内部の記録を混ぜない。足す行は 1 つ。

### Decision: 撤去は 1 本で、DAP の部品を残さない

- `package.json` の DAP の部品・3 ファイル・6 本の試験・mock の debug の部分を消す。`host` の廃止（`ponytail-editor-tooling` から受け継いだ監査 10 番・要件 12.3 の再輸出）は、ファイルごと消えるので片付く。
- 撤去で `activationEvents` は `onLanguage:pasta` だけになる。コマンドパレットからの起動は、VSCode 1.74 からの「宣言したコマンドで起動する」働きで足りる。

## Synthesis（設計の段）

- **Generalization**: 3 つのコマンドは「ゴーストを決める → 相手を決める → 送る → ディスクで確かめる → 出す」の同じ形である。共通の部品を 2 つ（`mcpClient`・`ghostFiles`）にし、「▶ シーンを実行」は「SHIORI をリロード」の流れ（`reloadAndWait`）をそのまま呼ぶ。
- **Build vs Adopt**: MCP の公式の TypeScript SDK（`@modelcontextprotocol/sdk`）は依存が増えるので採らない（brief の制約）。使うのは `initialize` と `tools/call` の 2 つだけで、どちらも JSON の単発応答である。TOML の読み手も採らず、`[logging] file_path` の 1 行だけを読む。HTTP は Node の組み込みの `fetch`、真似るサーバーは `node:http`。
- **Simplification**: 進み具合は VSCode の `withProgress`（窓の下の帯。やり取りの間だけ）。ステータスバーの項目は作らない。ゴーストごとの覚えは窓の間だけの `Map`（保存しない）。時間の値は定数（設定にしない）。`CHANGELOG.md` は触らない。

## Risks & Mitigations（設計の段）

- `pasta.log` の印が出ないゴースト（`[logging] level = "warn"` など・`pasta.toml` が壊れてロガーを登録する前に止まる） → 時間切れの知らせに「`pasta.log` に読み込みの終わりの行が出なかった」ことと、ログの量の設定を確かめる案内を添える。ベースウェアの誤りの記録はそれでも要約する（要件 5.8）。
- `pasta.log` の印の文が pasta の側で変わる → 拡張の試験が `crates/pasta_shiori/src/windows.rs`・`v3.rs` の文字列を読んで確かめる。
- SSP の `reload` の答えの文と、`get_log` の `script` の表示の語（`Ghost:OnPastaSceneAt` か）が未実測 → 拡張は答えの文に頼らない（印で待つ）。台本の引き直しは、表示の語にイベント名が入る記録を先に選び、無ければ送った後の最初の記録を使う。出す前の実機の 1 回で確かめる。
- SSP の MCP の窓口が既定で開いているかが未確認 → 見つからないときの知らせとマニュアルに、SSP の設定（SSTP サーバー・MCP）を確かめる案内を書く。
- VSCode のプロキシの設定が組み込みの `fetch` に効く版（1.94 前後の `http.fetchAdditionalSupport`）で、127.0.0.1 がプロキシへ回る恐れ → ふつうはループバックを除く。出す前の実機の 1 回で確かめ、回るなら `node:http` の直の呼び出しに替える（Open Question）。

## 設計の関門（design-review-gate）

- 1 回目: 機械の照合は通った（要件の番号 67 個がすべて design.md と追跡の表にある・境の 4 節が埋まっている・ファイルの計画にパスがある・部品はすべてファイルを持つ）。判断で 4 つを直した: 依存の向きの書き方が逆だった／`dumpVariables` が `reloadShiori` の型に頼ることを図と文に足した／`clientInfo.version` に拡張の版を読み込む仕組み（要らない）を消して固定の値にした／流れの試験を `test:e2e` へ入れることを書いた。条件付きで触る持ち場の外のファイル（要件 9.9）と生成物を Modified Files に足した。
- 2 回目: 直しの後の照合も通った。要件と設計の食い違い（spec の穴）は無かった。

## References（設計の段）

- areka `doc/ssp-mcp/survey.md`（SSP 2.9.05・2.9.07 の実測の正本）・`transport-diff-areka.md`・`bridge-diff-areka.md`・`areka-tools.md`・`log-convention.md`
- areka `crates/areka-mcp/src/dispatch.rs`・`gate.rs`・`tools/{get_log,reload,raise_event,get_active_ghost_list}.rs`、`crates/areka/src/mcp/reload.rs`
- ukadoc「その他の機能」<https://ssp.shillest.net/ukadoc/ssphelp/other.html>（MCP の節。survey の出典）
- `debug-mcp-runtime` の要件の決まり（side-session-notes `debug-mcp-runtime-requirements-decisions.md`・ブランチ `claude/kiro-start-debug-mcp-runtime-c5413c` の `a5b4f0ff`）

## 設計の議論の結論（2026-11-03・開発者が全 4 議題を推奨案で承認）

1. **読み込みの終わりの印**: `SHIORI load entry completed` は INFO なので、`[logging] level = "warn"` や `filter` で消える。直すなら出す側で 1 か所直すのが筋なので、`debug-mcp-runtime` に「この行は `level`・`filter` に関係なく必ず出す」を申し送る。拡張は、時間切れの知らせに `[logging]` の確かめを添える。
2. **実機の確かめ**: タスクを作る前に行う。普段の SSP で えも2DEBUG を（あれば areka も）起こし、Open Questions 1〜4 を確かめて、この research.md に記録する。結果で流れの形が変わり得るので、タスクの作り直しを避けるため。
3. **送り方**: 大域の `fetch` ではなく `node:http`。「届かない」と「送った後に切れた（応答が無い）」をソケットの出来事で見分けられる。VSCode の `fetch` のプロキシの仕組みにも乗らない。依存は増えない。
4. **`[logging] file_path`**: 表の書き方・点で区切ったキー・1 行のインラインテーブルの 3 つを読む。どれでも読めないときは、黙って倒さずに出力に添え書きを出す。TOML の読み手を足すのは「依存を増やさない」に反するので採らない。
