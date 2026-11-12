# ギャップ分析: vscode-scene-outline-play

## Summary
- **Feature**: `vscode-scene-outline-play`
- **Discovery Scope**: Extension（既存の解析器・拡張・コマンドを広げる。新しい依存は無い）
- **調べた時点**: 2026-11-12・ブランチ `claude/kiro-start-vscode-scene-outline-play-d21000`（main `165d2c46` の上）
- **Key Findings**:
  - 解析器（`crates/pasta_lsp`）は、構文木（全体の解析か部分解析）を 1 回だけ作り、トークンと診断を返している。シーンの並びは、同じ構文木の `GlobalSceneScope`・`LocalSceneScope` の `span.start_line` から作れる。範囲（次の同じ階層か上の階層の見出しの直前まで・最後はファイルの終わり）は、pasta.dll の `scene_join.rs` の決め方と同じ式で作れる。足りないのは取り出す処理と結果の欄だけ。
  - 拡張には DocumentSymbol・CodeLens のプロバイダが無く、試験の `mock/vscode.ts` にも `registerCodeLensProvider`・`registerDocumentSymbolProvider`・`CodeLens`・`DocumentSymbol`・`SymbolKind`・`workspace.openTextDocument` が無い。試験の土台の足し込みが要る。
  - `pasta.runSceneAtCursor` の登録（`devCommands.ts` 190 行）は引数を捨て、`execute()` は `activeTextEditor` だけを見ている。`Command.run(ctx, editor)` もエディタ前提。引数付きの呼び出しを足すには、`execute` の「対象の文書と行」の決め方を、エディタから「文書と行」へ広げる必要がある。`runScene(ctx, fsPath, line1)` はそのまま使える。
  - **注意**: VSCode は `editor/context` のメニューから呼ぶコマンドに、文書の `Uri` を第 1 引数として渡す（要確認。下の Research Needed 1）。右クリックからの呼び出しは「引数が無い」のでなく「uri だけがあって行が無い」形になりうる。引数の見分けは「行があるか」で行う必要がある。
  - `scene-play-locals` とは、触るファイルが重ならない（下の「並行する spec との重なり」）。

## 現状の調査

### 解析器（`crates/pasta_lsp`）
- `analysis/mod.rs`（192 行）: `AnalysisResult { tokens, diagnostics }`（`Debug, Clone, Default, serde::Serialize`）。`AnalysisEngine::analyze(source)` は `pub`。`pasta_dsl::parse_str` が通れば全体の構文木、だめなら `parse_str_partial` の構文木を `visit_file_items` に渡す。部分解析の `failed_lines` はコメントの走査だけに使う。
- `Position { line, character }`・`Range { start, end }` は既に `pub` で、0 始まりの行・UTF-16 の列。シーンの位置にそのまま使える。
- `text_utils.rs`: `get_line_text`・`line_byte_offset`（`pub`）、`char_column_to_utf16`（`pub(crate)`）、`utf8_len_to_utf16`（`token_types` 側）。範囲の終わり（最後の行の UTF-16 の長さ）を出すのに使える。CRLF の `\r` を数えない処理もある。
- `visit_scope.rs`（285 行）: `visit_global_scene` は見出しの行だけを NAMESPACE、名前付きのローカルシーンだけを SCENE にしている。名前の無い `LocalSceneScope`（`local_start_scene_scope`）は見出しの行を持たないので飛ばしている。この判定はシーンの並びでも同じ（要件 1.5）。
- `transport.rs`: `wasm_analyze` は `AnalysisEngine::analyze` の結果をそのまま `serde_wasm_bindgen::to_value` で返す。`test_analysis_result_json_shape_stable` が JSON の形（欄の名前・順）を文字列で固定している。欄を足すとこの試験の期待を直す必要がある。
- `AnalysisResult` を構造体の書き方で作っているのは `mod.rs` 100 行と `transport.rs` 64 行の 2 か所だけ。
- 試験は `crates/pasta_lsp/tests/*_test.rs`（16 本）。`partial_token_test.rs`・`fullwidth_halfwidth_test.rs`・`code_block_token_test.rs` が、部分解析・全角半角・コードブロックの試験の書き方の手本になる。

### 構文木（`crates/pasta_dsl`）
- `GlobalSceneScope { name, is_continuation, attrs, words, actors, code_blocks, local_scenes, span }`。続きのシーンは `is_continuation = true` で、`name` に前の名前が入る。
- `LocalSceneScope { name: Option<String>, attrs, items, code_blocks, span }`。`name = None` が名前の無い最初のローカル部分。
- `Span` は 1 始まりの行・列とバイトの位置。`is_valid()` がある。
- 文法（`grammar.pest`）: グローバルの印は `＊`・`*`、ローカルの印は `・`・`-`（字下げ `pad` が要る）。`global_scene_scope = global_scene_start ~ global_scene_init* ~ code_scope* ~ local_start_scene_scope ~ local_scene_scope*`。コードブロックの中の `＊` は文法上シーンにならない。
- 部分解析（`partial.rs`）: 行頭の 1 文字で塊に分け、塊ごとに解く。解けない塊は構文木に入らない（要件 1.11 の「解けない部分のシーンを項目にしない」はこれで自然に満たす）。名前の無い `＊` は区切りにしない（続きのシーンは前のシーンと一緒に解く）。**shortcut**: Lua ブロックの本文の 0 桁目の `＊` も区切りにする（文書に別の誤りがあるときだけ、コードブロックが 2 つに割れて両方が解けなくなりうる）。

### pasta.dll の行からシーンを引く表（読むだけ・触らない）
- `pasta_lua/src/debug/source_map/scene_join.rs`: 終わりの行＝「次の同じ階層か上の階層の宣言の開始行 − 1」、無ければファイルの終わり。グローバルの見出しの行には `__start__` も同じ行で記録され、同じ行は境にしない。
- `scene_identity_index_test.rs`: グローバルの見出しの行 → `(OnTalk_1, __start__)`（そのグローバルシーンの再生）、ローカルの見出しの行 → `(OnTalk_1, 挨拶_1)`、続きのシーンの見出しの行 → `(OnTalk_2, __start__)` を確かめている。見出しの行を渡せば、押したシーンが鳴ることは既に試験で守られている（要件 3.5）。
- 最初のシーンより前の行は「下の最も近いシーン」になる。並びの範囲は見出しの行から始まるので、この後方の補いは並びには現れない（食い違いではない。CodeLens は見出しの行しか送らない）。

### 拡張（`editors/vscode`）
- `extension.ts`: `registerDevCommands` を WASM の起動より前に呼ぶ（要件 5.2 は今のまま満たす）。WASM が読めたときだけ `SemanticTokensProvider`・`DocumentSync`・`GhostValidator` を作る。新しい 2 つのプロバイダも、この `if (activationState.wasmReady && wasmBridge)` の中で登録すれば要件 5.1 を満たす。
- `wasmBridge.ts`: `WasmAnalysisResult { tokens, diagnostics }` と `isWasmAnalysisResult`（2 つの配列があるかだけを見る）。`scenes` を足すなら、型と検査の両方に足す。検査を厳しくすると、`scenes` を持たない古い WASM で着色まで止まるので、検査の範囲は設計で決める。
- `semanticTokensProvider.ts`: 文書の版ごとにトークンを控え、控えが無ければその場で `analyzeDocument` を呼ぶ。失敗したら `null` を返し、通知を出さない（要件 5.3 の手本）。
- `documentSync.ts`: 200 ms のデバウンスで解析し、トークンを控えに入れ、診断を出す。DocumentSymbol・CodeLens のプロバイダがそれぞれ解析すると、1 回の編集で解析が 2〜3 回に増える。
- `devCommands.ts`: `COMMANDS` の 3 つのコマンドは `run(ctx, editor)`。`execute(cmd)` は `activeTextEditor` の文書を見て、`scheme === 'file'`・`!isUntitled`・`accepts` を確かめ、ゴーストを決め、重なりを止め（`busy`）、相手を選び、保存し、`run` を呼ぶ。登録は `registerCommand(cmd.id, () => execute(cmd))` で引数を捨てる。
- `runSceneAtCursor.ts`: `runScene(ctx, fileFsPath, line1)` は vscode に触らない。そのまま使え、形を変えない（`pasta-recipe-dev-actions` への申し送り）。
- `package.json`: `engines.vscode ^1.85.0`（CodeLens・DocumentSymbol の API は十分に古くからある）。`activationEvents: onLanguage:pasta`。CodeLens・DocumentSymbol はコードで登録するので `contributes` の足し込みは要らない。試験は `test:unit`・`test:e2e` のスクリプトに esbuild の行を 1 本ずつ並べる形で、新しい試験のファイルを足すならこのスクリプトを直す（`vsce-4-upgrade` と同じファイル）。
- `src/test/mock/vscode.ts`（488 行）: `languages` は `createDiagnosticCollection`・`registerDocumentSemanticTokensProvider` だけ。`commands.executeCommand(id, ...args)` は引数をそのまま渡すので、引数付きの呼び出しの試験はすぐ書ける。`workspace.openTextDocument` は無い。
- `src/test/devCommands.e2e.test.ts`（419 行）: MCP を真似るサーバー（`fakeMcpServer.ts`）と `executeCommand('pasta.runSceneAtCursor')` で、保存・読み込み直し・`OnPastaSceneAt` の引数まで確かめている。引数付きの呼び出しの試験は、ここに足すのが近い。
- `src/test/contributions.test.ts`: コマンドの題・右クリックのメニューの置き場所を確かめている（要件 4.6 は今の試験が守る）。

### マニュアル
- `book/src/debug/dev-actions.md` の「## シーン再生（▶ シーンを実行）」（89 行〜）。手順 3 が「カーソルの行」と書いている。`lua/shiori-events.md` の `OnPastaSceneAt` の節が `../debug/dev-actions.md#シーン再生-シーンを実行` へ飛ぶので、見出しは変えない（要件 6.5）。
- `book/tools/verify-content.mjs` の G 節: デバッグ章の本文の量・キャラの口調・コードフェンスの中にナレーションが無いこと・主要な語（`OnPastaSceneAt` など）。設定の JSON のコードフェンスに口調の印（`【` など）を入れない。
- `dev-actions.md` はスキルの `references/` を作る対象ではない（`gen-skill-refs.mjs` の表に無い）。スキルの作り直しは要らない。
- `editors/vscode/README.md` 14 行の「開発支援コマンド」も右クリックの説明だけ。要件には入れていないが、1 行足すかは設計で決める。

## 要件と既存の資産の対応

| 要件 | 既存の資産 | ギャップ |
| ---- | ---------- | -------- |
| 1.1〜1.2 並びの項目と木 | `AnalysisEngine::analyze` の構文木、`Position`・`Range` | **Missing**: シーンの項目の型と、構文木から取り出す処理。結果の欄 `scenes` |
| 1.3 全角・半角の印 | 文法（`ast`・`local_marker`） | なし（構文木が吸収する）。試験だけ要る |
| 1.4 続きのシーン | `GlobalSceneScope.is_continuation`・受け継いだ `name` | なし。試験だけ要る |
| 1.5 名前の無い最初の部分 | `LocalSceneScope.name == None` | なし（`visit_local_scene` と同じ判定） |
| 1.6 コードブロックの中の `＊` | 文法 | 全体の解析ではなし。**Constraint**: 部分解析の shortcut（別の誤りがあるときだけ） |
| 1.7〜1.8 範囲 | `scene_join.rs` の式（読むだけ） | **Missing**: 解析器の側の範囲の計算（見出しの行の並びから。span の終わりは使わない）と、最後の行の UTF-16 の長さ |
| 1.9 座標 | `text_utils.rs` | 見出しの行の列の頭・行の終わりの UTF-16 の計算。CRLF の扱い |
| 1.10 空の並び | — | なし（項目が無ければ空） |
| 1.11 書きかけ | `parse_str_partial` | なし（解けない塊は構文木に無い）。試験だけ要る |
| 1.12 既存の欄を変えない | `test_analysis_result_json_shape_stable` | **Constraint**: 試験の期待に `scenes` を足す。トークン・診断の試験 16 本は変わらない見込み |
| 1.13 ほかの呼び手 | `analyze` は `pub` | なし。並びの型も `pub` にし、`serde::Serialize` を付ける |
| 2.x アウトライン等 | — | **Missing**: DocumentSymbol のプロバイダ・登録・試験・mock |
| 3.1〜3.2・3.6〜3.8 CodeLens | — | **Missing**: CodeLens のプロバイダ・登録・試験・mock |
| 3.3〜3.5 見出しの行で再生 | `runScene(ctx, fsPath, line1)`・`scene_identity_index_test.rs` | **Missing**: コマンドへ文書と行を渡す道 |
| 3.9 重なりの止め | `execute` の `busy` | なし（同じ `execute` を通せば満たす） |
| 4.1 引数なし | `execute` | **Constraint**: 右クリックは uri を渡しうる（Research Needed 1）。「行があるか」で見分ける |
| 4.2〜4.3 引数あり | `resolveGhost(fsPath, findGhostMaster(uri))` | **Missing**: 文書の指定から文書を得る処理（開いていない文書なら開く）、`run` への行の渡し方 |
| 4.4 文書の誤り | `BAD_DOCUMENT` と `accepts` | なし（同じ確かめを指定の文書にかける） |
| （行の誤り） | pasta.dll の `OnPastaSceneAt` の引数の確かめ | なし（拡張では確かめない。要件ディスカッションで確定） |
| 4.5〜4.6 コマンドを足さない | `contributions.test.ts` | なし |
| 5.x 解析器が無いとき | `extension.ts` の `wasmReady` の分かれ | なし（登録の置き場所だけ） |
| 6.x マニュアル | `dev-actions.md`・`verify-content.mjs` G 節 | **Missing**: 節の書き足し |

## 実装の選択肢

### 選択肢 A: 既存のファイルを広げる
- 解析器: `analysis/mod.rs` に項目の型と取り出しを書き、`AnalysisResult` に `scenes` を足す。
- 拡張: `semanticTokensProvider.ts` か `documentSync.ts` に 2 つのプロバイダを同居させる。`devCommands.ts` の `execute` に引数を足す。
- ✅ ファイルが増えない。
- ❌ `mod.rs` の責務（解析の入口・診断・コメントの走査）に、並びの計算が混ざる。`semanticTokensProvider.ts` に関係の薄い 2 つのプロバイダが入る。

### 選択肢 B: 新しい部品を作る
- 解析器: `analysis/scenes.rs`（仮の名前）に、構文木から並びを作る純粋な関数と項目の型を置き、`mod.rs` の `analyze` から 1 行で呼ぶ。`visit_scope.rs` 等の着色の訪問とは分ける。
- 拡張: `sceneOutline.ts`（仮の名前）に、並びから DocumentSymbol と CodeLens へ写す純粋な関数と 2 つのプロバイダを置く。`extension.ts` の `wasmReady` の中で登録する。
- `devCommands.ts` は、`execute` の対象の決め方を「アクティブなエディタとカーソル」か「指定の文書と行」に広げ、`run` に渡すものを `editor` から「文書と行」へ変える（3 つのコマンドの `run` の形が少し変わる）。
- ✅ 並びの計算と写しを vscode なしで試験できる（`runSceneAtCursor.ts`・`ghostFiles.ts` と同じ「vscode に触らない部品＋糊」の作り）。`pasta-recipe` が読むのは解析器の型だけ。
- ❌ ファイルが 2 つ増える。

### 選択肢 C: 組み合わせ
- 解析器は B（`scenes.rs`）、拡張の写しは 1 つの新しいファイル、コマンドは `devCommands.ts` の中で広げる（A）。
- 解析の回数を抑えるため、文書の版ごとの解析の結果の控えを 1 つにし、着色・アウトライン・CodeLens で使い回す（`SemanticTokensProvider` の控えを、トークンだけでなく結果ごと持つように広げる、など）。
- ✅ 既存の作り（vscode に触らない部品・`devCommands.ts` が糊）に揃う。解析が 1 回で済む。
- ❌ 控えの持ち主を決める必要がある（`DocumentSync`・`SemanticTokensProvider` の責務の見直し）。控えを広げないなら、1 回の編集で解析が 3 回になる（文書が小さいので遅さは目立たない見込みだが、測っていない）。

## 規模とリスク
- **規模: M（3〜7 日）**。解析器の取り出しと試験は S、拡張の 2 つのプロバイダと mock の足し込み・コマンドの引数・e2e・マニュアルを合わせて M。
- **リスク: Low〜Medium**。既存の作りを広げるだけで新しい依存は無い。気をつけるのは、右クリックの引数の見分け（取り違えると右クリックの再生が壊れる）と、範囲の計算を pasta.dll と揃えること。

## 並行する spec との重なり
- **`scene-play-locals`**: 触るのは `pasta_scripts/pasta/shiori/event/scene_play.lua`・`scene.lua`・`pasta_lua` の code_gen・`book/src/lua/shiori-events.md`（`OnPastaScene`・`OnPastaSceneAt` の節）・スキル `pasta-lua-coding` の `references/shiori-events.md`。この spec が触るのは `crates/pasta_lsp`・`editors/vscode`・`book/src/debug/dev-actions.md`。**ファイルの重なりは無い**。`shiori-events.md` の `OnPastaSceneAt` の節は、この spec では直さない（行の規則が変わらないため）。同じウェーブに置いてよい（roadmap の表の 2 番の注記とも合う）。
- 参考（brief の Adjacent）: `scene-anchor-link`（`pasta_lsp` の解析）・`vsce-4-upgrade`（`editors/vscode/package.json`）とは同じウェーブに置かない。この spec が `package.json` を直すのは、試験のスクリプトに新しい試験のファイルを足すときだけ。

## 設計への申し送り

### 推奨の向き
- 選択肢 C。解析器は `scenes.rs` に純粋な関数を置き、`AnalysisResult` に `scenes` を足す。拡張は写しの純粋な関数と 2 つのプロバイダを 1 つの新しいファイルに置き、`devCommands.ts` で引数を受ける。

### 設計で決めること
1. 項目の型の名前と JSON の欄の名前（`name`・`kind`・見出しの行・`range`・`children`）。既存の `SemanticToken` は camelCase に名前を変えているので、それに揃えるか。`kind` を文字列（`"global"`・`"local"`）にするか。
2. 見出しの行を「行の番号だけ」で持つか、「見出しの行の範囲（`Range`）」で持つか。DocumentSymbol の selectionRange は範囲が要るが、`pasta-recipe` は行だけで足りる。
3. 範囲の終わりの位置（最後の行の終わりの列。ファイルの最後が改行で終わるときの最後の行の扱い。CRLF）。
4. `isWasmAnalysisResult` で `scenes` をどこまで確かめるか（厳しくすると、`scenes` の無い WASM で着色も止まる）。
5. 解析の結果の控えを着色・アウトライン・CodeLens で共有するか。
6. コマンドの引数の形（`(uri, line)` の位置引数か、1 つの物か）と、行を 0 始まりで渡すか 1 始まりで渡すか。右クリックの `uri` だけの呼び出しとの見分け方。
7. 指定の文書が開かれていないときに開くか（`openTextDocument`）。
8. （閉じた）行の誤りは pasta.dll に任せ、拡張の文言は作らない。
9. `editors/vscode/README.md` に 1 行足すか。

### Research Needed
1. `editor/context` のメニューから呼んだコマンドが受け取る引数（文書の `Uri` が第 1 引数で来るか）。拡張の試験の mock では確かめられないので、VSCode の挙動を確かめて設計に書く。
2. CodeLens のコマンドの引数に `vscode.Uri` を入れたとき、そのまま `Uri` の物として届くか（VSCode は CodeLens のコマンドの引数を内部で受け渡す）。
3. スティッキースクロールが DocumentSymbol を使う条件（`editor.stickyScroll.defaultModel` の既定 `outlineModel`）。拡張の側で何も足さずに出ることを確かめる。

## Risks & Mitigations
- 右クリックの呼び出しに uri だけが来て、それを「引数あり」と取り違える → 「行があるか」で見分け、右クリックの e2e（今の `devCommands.e2e.test.ts`）に uri だけの呼び出しを足す。
- 範囲の計算が pasta.dll とずれる → `scene_join.rs` と同じ式にし、続きのシーン・ローカルの後のグローバル・最後のシーンの試験を解析器の側に置く。CodeLens が送るのは見出しの行だけなので、範囲がずれても鳴るシーンは変わらない（範囲のずれはアウトラインの見え方だけに出る）。
- 書き換えの直後に古い CodeLens を押して、ずれた行で鳴る → VSCode が書き換えのたびに出し直す。確かめ直しは足さない（要件ディスカッションで確定。要件の範囲外に記載）。
- 部分解析の shortcut で、誤りのある文書のコードブロックの後ろのシーンが並びから消える → 誤りのある文書は再生まで進まないので害は見え方だけ。試験で現状の振る舞いを記録する。
- `test_analysis_result_json_shape_stable` を直し忘れて CI が赤くなる → 欄を足すタスクに試験の直しを含める。
- WASM のビルドが手元と CI の 3 か所で通ること（`pest-2-9-4-upgrade` の注意）→ 新しい依存を足さないので、ビルドの手順は変わらない見込み。

## References
- `.kiro/specs/vscode-scene-outline-play/brief.md` — 問題・決定 1〜6・範囲・申し送り
- `.kiro/steering/roadmap.md`「0.5.3 の範囲」
- `crates/pasta_lua/src/debug/source_map/scene_join.rs`・`crates/pasta_lua/tests/scene_identity_index_test.rs` — 行からシーンを引く表と、その試験
- `book/src/lua/shiori-events.md`「OnPastaSceneAt」・`book/src/debug/dev-actions.md`「シーン再生」

---

# 設計フェーズの調査（2026-11-12・`/kiro-spec-design -y`）

## Summary（設計）
- **Discovery Scope**: Extension（軽い discovery。新しい依存は無い）
- **Key Findings**:
  - VSCode は、メニュー（`editor/context`）から呼ぶコマンドに文書の `Uri` を第 1 引数で渡す。右クリックの呼び出しは「`Uri` だけ・行なし」の形になる。引数あり／なしの見分けは「第 2 引数が数か」で行う。
  - CodeLens のコマンドの引数は、拡張のホストの中の控えに置かれ、押したときに元の JS の物のまま渡る（直列化されない）。`vscode.Uri` は `Uri` のまま届き、`instanceof vscode.Uri` で見分けられる。
  - スティッキースクロールの既定の作り方は `editor.stickyScroll.defaultModel = "outlineModel"`（`enabled` の既定は `true`）。DocumentSymbol のプロバイダを足すだけで、アウトライン・パンくず・`Ctrl+Shift+O` と一緒にスティッキースクロールにも出る。拡張の側の設定・`contributes` は要らない。
  - 範囲の終わりの決め方は pasta.dll の `scene_join.rs` と同じ「次の同じか上の階層の見出しの行 − 1・無ければファイルの終わり」。ローカルシーンは親の中にしか無いので、この式で 1.8（子は親の中）が自然に満たされる。

## Research Log（設計）

### Research Needed 1: `editor/context` から呼んだコマンドの引数
- **Sources**: VS Code API「Contribution Points」の `contributes.menus`（https://code.visualstudio.com/api/references/contribution-points ）。要旨: メニューから呼ぶと、VS Code はそのメニューの文脈を `registerCommand` のハンドラへ渡す。`editor/context` は文書の URI を受ける。
- **Findings**: 右クリックの「▶ シーンを実行」は `(uri)` で呼ばれる。コマンドパレットからは引数なし。今の登録 `() => execute(cmd)` はこれを捨てている。
- **Implications**: 「`Uri` があるか」で見分けると右クリックを引数ありと取り違える。「第 1 引数が `vscode.Uri` で、かつ第 2 引数が数」のときだけ引数ありとし、それ以外（引数なし・`Uri` だけ）は今までどおりアクティブなエディタとカーソルの行を使う。e2e に `Uri` だけの呼び出しを足す。

### Research Needed 2: CodeLens のコマンドの引数に `vscode.Uri` を入れたとき
- **Sources**: VS Code のソース `src/vs/workbench/api/common/extHostCommands.ts`（`CommandsConverter.toInternal`・`_executeConvertedCommand`。https://github.com/microsoft/vscode/blob/main/src/vs/workbench/api/common/extHostCommands.ts ）。
- **Findings**: 引数のある拡張のコマンドは、`this._cache.set(id, command)` で拡張のホストに控え、表示の側へは控えの鍵だけを送る。押されると `executeCommand(actualCmd.command, ...actualCmd.arguments)` で、元の物をそのまま渡す。
- **Implications**: `arguments: [document.uri, line]` は `Uri` の物と数のまま届く。引数を文字列にする・`Uri.parse` し直す必要は無い。

### Research Needed 3: スティッキースクロールの既定
- **Sources**: VS Code のソース `src/vs/editor/common/config/editorOptions.ts`（stickyScroll の設定の定義）。
- **Findings**: `defaultModel` の値は `outlineModel`・`foldingProviderModel`・`indentationModel`、既定は `outlineModel`。アウトラインの作り方が使えなければ折りたたみ、さらに字下げへ落ちる。`enabled` の既定は `true`。
- **Implications**: DocumentSymbol を出せば、既定の設定のままスティッキースクロールに見出しが出る（2.2・2.6）。出る行は DocumentSymbol の `range` の始まりの行なので、`range` を見出しの行の頭から始める。

### その他の確かめ（設計）
- `@types/vscode`（node_modules は 1.140.0。`engines.vscode` は `^1.85.0`）: `DocumentSymbol(name, detail, kind, range, selectionRange)`・`selectionRange` は `range` に含まれること・`CodeLens(range, command?)`（range は 1 行に収める）・`registerDocumentSymbolProvider`・`registerCodeLensProvider`。どれも 1.85 より前からある。
- `DocumentSymbol` は名前が空だと例外になる。続きのシーンは前の名前を受け継ぎ、ファイルの頭の名前の無い `＊` は構文の誤り（`parse_scene.rs`）で部分解析でも塊ごと捨てられるので、空の名前は並びに現れない。
- `wasm_analyze` は `serde_wasm_bindgen::to_value`（既定の直列化）で返す。`Option` の欄は `undefined` になるので、シーンの型には `Option` の欄を置かない（`children` は空の配列にする）。構造体は JS の素の物、単位の列挙は文字列になる。
- 拡張の試験: 本物のモジュールを mock で動かす `vscodeModules.test.ts` は `test:unit` に既にある。新しいプロバイダの試験をここに足せば、`package.json`（`vsce-4-upgrade` と共有）の試験のスクリプトを直さなくて済む。`vscodeModules.test.ts` の `bridgeWithWasmResult({ tokens: [], diagnostics: [] })` が「正しい形」として通る試験（565・582 行）は、`scenes` の確かめを足すと期待を直す必要がある。
- pasta.dll の記録（`pasta_lua/src/code_gen/scope_gen.rs`）は、グローバルシーンとローカルシーン（名前の無い開始の部分は `__start__`）の `span.start_line` を記録する。`__start__` はグローバルの見出しと同じ行なので境にならない。アクター辞書・単語定義・ファイル属性は記録しないので、シーンの範囲を切らない（`＊挨拶` の後ろの `％さくら` の塊は、次のシーンの見出しまで `挨拶` の範囲に入る）。解析器も同じにする。

## 使ったスキル・指針
- `kiro-spec-design` の `rules/design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`。
- プロジェクトの方針（呼び手の指示）: 最小で完全な変更・既存の作りの再利用・先回りの抽象を作らない。

## Synthesis（設計）
- **一般化**: 「アウトライン」と「見出しの上の ▶」は、どちらも同じ木の写し。木を作るのは解析器の 1 か所（`scenes.rs`）、写すのは拡張の 1 クラス（`SceneOutlineProvider` が 2 つのプロバイダの役を持つ）。木の形は DocumentSymbol に寄せず、`pasta-recipe` がそのまま使える素の形にする（インタフェースだけを一般にし、実装は今の要件の分だけ）。
- **作るか使うか**: VSCode の標準の DocumentSymbol・CodeLens を使う（パンくず・`Ctrl+Shift+O`・スティッキースクロールは VSCode が DocumentSymbol から作る）。CodeLens を消す設定は VSCode の言語ごとの `editor.codeLens` を使う。範囲の決め方は pasta.dll の式を写す（pasta.dll の部品は Lua の実行時に依存するので、解析器からは呼べない）。
- **簡素化**:
  - 新しいコマンドを作らず、`pasta.runSceneAtCursor` の登録で引数を受ける。`runScene(ctx, fsPath, line1)` は変えない。
  - 解析の結果の控えを `DocumentSync`・`SemanticTokensProvider` と共有する作り直しはしない。新しいプロバイダの中に「最後に解いた 1 つの文書の版」だけを覚える 1 枠の控えを置き、DocumentSymbol と CodeLens の問い合わせで 2 回解かないようにする。
  - DocumentSymbol と CodeLens を別のファイル・別のクラスに分けない（写す元が同じで、どちらも数行）。
  - `package.json` を触らない（試験は既存の `vscodeModules.test.ts`・`devCommands.e2e.test.ts` に足す）。

## Design Decisions（設計）

### Decision D1: 項目の型と JSON の欄（申し送り 1）
- **Selected**: Rust `SceneNode { name: String, kind: SceneKind, heading: Range, range: Range, children: Vec<SceneNode> }`、`SceneKind { Global, Local }`（`#[serde(rename_all = "lowercase")]` で `"global"`・`"local"`）。JSON は `{"name","kind","heading","range","children"}`。欄の名前はどれも 1 語なので `rename_all = "camelCase"` は要らない（`SemanticToken` は複数語の欄があるので名前を変えている）。`AnalysisResult` の欄 `scenes: Vec<SceneNode>` は `tokens`・`diagnostics` の後ろに置く。
- **Rationale**: 文字列の `kind` は JS・`pasta-recipe` のどちらでも読みやすく、TS の文字の合併型 `'global' | 'local'` にそのまま写る。
- **Alternatives**: `kind` を数にする（読みにくい）／`kind` を持たず深さで決める（要件 1.2 が種類を求める）。

### Decision D2: 見出しを Range で持つ（申し送り 2）
- **Selected**: `heading: Range`。見出しの行の、最初の空白でない文字（印）から、行の終わり（改行・`\r` を除く）まで。
- **Rationale**: DocumentSymbol の `selectionRange` と CodeLens の `range`（1 行）にそのまま使える。行だけが要る呼び手（`pasta-recipe`・CodeLens の引数）は `heading.start.line` を読む。行だけで持つと、拡張の側で行の文字を読み直して列を決めることになる。
- **Trade-off**: 見出しの後ろの属性・行末のコメントも `heading` に入る（選んだときに行の頭へ移れば足りるので害は無い）。

### Decision D3: 範囲の終わり（申し送り 3）
- **Selected**: `range.start` は（見出しの行, 0）。`range.end` は、次の同じか上の階層の見出しがあれば（その行 − 1, その行の UTF-16 の長さ）、無ければ文書の終わり（最後の `\n` の後ろの行, その行の UTF-16 の長さ）。行は `\n` で数え、行の長さは `\r` を数えない（`get_line_text` と同じ。トークン・診断と揃う）。文書が改行で終わるときは、最後の改行の後ろの空の行（VSCode の最後の行）の頭が終わりになる。
- **Rationale**: `scene_join.rs` の「次の同レベル以上宣言の開始行 − 1、無ければファイル末尾」をそのまま写す。ローカルシーンの次の見出しは、同じ親の次のローカルシーンか次のグローバルシーンしかないので、子の範囲は親の範囲を越えない（1.8）。
- **Known limit**: 単独の `\r`（古い Mac の改行）は VSCode では改行だが、解析器では改行にしない。トークン・診断と同じ扱いで、この spec では変えない。

### Decision D4: `isWasmAnalysisResult` の確かめ（申し送り 4）
- **Selected**: `Array.isArray(scenes)` を足す（`tokens`・`diagnostics` と同じ浅さ。中身の 1 つずつは確かめない）。
- **Rationale**: WASM と拡張の JS は同じコミットから 1 つの VSIX に入るので、`scenes` の無い WASM は作り間違いの時だけ起きる。そのときは既存の「`WASM analysis failed`」の道で失敗させる（着色も TextMate に落ちる）。確かめを 2 か所に分けると、型の上では `scenes` があるのに実際は無い、という食い違いが残る。
- **Alternatives**: プロバイダの側だけで確かめる（着色は残るが、型と実際がずれる）。

### Decision D5: 解析の結果の控え（申し送り 5）
- **Selected**: 共有しない。`SceneOutlineProvider` が 1 枠（uri・版・並び）の控えを持つ。1 回の編集で解析は `DocumentSync` の 1 回＋並びの 1 回（＋版の食い違いのときだけ `SemanticTokensProvider` の 1 回）。
- **Rationale**: 控えを共有すると `DocumentSync`・`SemanticTokensProvider` の責務を作り直すことになる。辞書の 1 ファイルは小さく、解析は同期で速い。1 枠なら閉じた文書の後始末も要らない。
- **Follow-up**: 大きな辞書で遅さが見えたら、`SemanticTokensProvider` の控えを結果ごとに広げて共有する（コードに shortcut の注記を置く）。

### Decision D6: コマンドの引数の形（申し送り 6）
- **Selected**: 位置引数 `(uri: vscode.Uri, line: number)`。`line` は 0 始まり（解析器の `heading.start.line` をそのまま渡す）。`devCommands.ts` が `line + 1` にして `run` へ渡す（カーソルの道の `selection.active.line + 1` と同じ所で 1 足す）。見分け: `args[0] instanceof vscode.Uri && typeof args[1] === 'number'` なら指定の文書と行、それ以外（引数なし・右クリックの `Uri` だけ）はアクティブなエディタとカーソルの行。
- **Rationale**: 0 始まりは VSCode・解析器の座標と同じで、1 始まりへの直しが 1 か所に集まる。数の確かめ（整数か・0 以上か）は持たない（要件の決定: 行の確かめは pasta.dll）。
- **Alternatives**: 1 つの物 `{ uri, line }`（ほかの拡張の呼び方と揃わない）／1 始まり（直しの所が 2 つに割れる）。

### Decision D7: 指定の文書を開く（申し送り 7）
- **Selected**: 引数ありのときは、いつも `vscode.workspace.openTextDocument(uri)` で文書を得る。開いている文書ならそれが返り（未保存の中身も含む）、開いていなければ読み込む（エディタには出さない）。開けなければ `BAD_DOCUMENT` で止める。
- **Rationale**: 開いているかどうかで道を分けずに済む。CodeLens からは必ず開いている文書なので、実際には読み込みは起きない。

### Decision D8（閉じている）
- 行の確かめは pasta.dll に任せる。拡張は確かめない（要件ディスカッションで確定）。

### Decision D9: `editors/vscode/README.md`（申し送り 9）
- **Selected（仮定・OPEN QUESTION）**: 「機能」の「開発支援コマンド」の行に、見出しの上の「▶ シーンを実行」と、アウトライン・パンくずへの並びを一言足す。要件の範囲（マニュアルだけ）を少し超えるので、開発者の判断に回す。

### Decision D10: SymbolKind（設計で足した論点）
- **Selected（仮定・OPEN QUESTION）**: グローバルシーン＝`SymbolKind.Namespace`（着色のトークン `namespace` と揃う）、ローカルシーン＝`SymbolKind.Function`（シーンは呼べる単位）。`detail` は空。

## Risks & Mitigations（設計）
- 右クリックの `Uri` だけの呼び出しを引数ありと取り違える → 見分けを「第 2 引数が数か」にし、e2e で `Uri` だけの呼び出しがカーソルの行で鳴ることを確かめる。
- `scenes` の確かめを足して既存の試験が赤くなる → `vscodeModules.test.ts` の「正しい形」の試験の期待に `scenes: []` を足す（同じタスクで直す）。
- 範囲が pasta.dll とずれる → `scene_identity_index_test.rs` と同じ並び（グローバル・ローカル・続き・最後のシーン）を解析器の試験に置き、終わりの行を同じ式で確かめる。鳴るシーンは見出しの行だけで決まるので、ずれてもアウトラインの見え方だけに出る。
- `transport.rs` の形の試験の直し忘れ → 欄を足すタスクに含める。
