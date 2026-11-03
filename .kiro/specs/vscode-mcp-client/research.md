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
