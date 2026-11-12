# Brief: vscode-scene-outline-play

> **ステータス**: 未着手（2026-11-09 の `/kiro-discovery` で起票。開発者の依頼「VSCode 拡張で、シーン再生への手数を減らして簡単に再生をキックしたい」）。版は 0.5.3（2026-11-12 の `/kiro-discovery`「pasta-recipe」で開発者が承認。下の「申し送り」）。要件定義は Opus（Fable なし）。依頼にはコードの例が付いていたが、開発者の指示「大事なのは要件なので、コード例にとらわれず実装案を考えて」に従い、下の「Approach」は既存の作りに合わせて組み直した。

## Problem

ゴーストの作者が `.pasta` の 1 つのシーンを鳴らして確かめるには、今は、カーソルをそのシーンの中に置き、右クリックのメニューから「▶ シーンを実行」を選ぶ。シーンを 1 つずつ確かめるときは、この手数が毎回かかる。

また、VSCode のアウトライン・パンくず・`Ctrl+Shift+O`（シンボルへ移動）・スティッキースクロールに、シーンの並びが出ない。長い辞書の中で、今どのシーンにいるか、どこにどのシーンがあるかが見えない。

## Current State

2026-11-09（main `8ed2a677`）に確かめた。

- **再生のコマンド**: `editors/vscode/src/devCommands.ts` 49 行の `pasta.runSceneAtCursor`（題「▶ シーンを実行」。`package.json` の `editor/context` のメニューにある）。`run(ctx, editor)` が `editor.selection.active.line + 1` を `runSceneAtCursor.ts` の `runScene(ctx, fsPath, line1)` へ渡す。`execute()`（147 行）は、アクティブなエディタの文書を見て、`accepts` で `.pasta` かを確かめ、`saves` ならゴーストの文書を保存し、SHIORI を読み込み直してから再生する。コマンドの登録（190 行）は引数を受けない。
- **行からシーンを引くのは pasta.dll**: `runScene` は `OnPastaSceneAt` に Reference0＝ファイルの絶対パス、Reference1＝行（1 から数える）を付けて送るだけ。引く規則は `book/src/lua/shiori-events.md` の「OnPastaSceneAt」にある。その行を含む最も内側のシーンを再生し、グローバルシーンの見出しの行（ローカルシーンより前の行）ならグローバルシーン、ローカルシーンの見出しの行ならそのローカルシーンになる。同じ名前のシーンが複数あっても、指した 1 つを再生する。最後のシーンはファイルの終わりまで続く。
  - したがって、見出しの行の番号だけで「どのシーンか」が決まる。シーンの名前などの識別の情報を別に渡す必要は無く、実行時の側は触らない。
- **解析**: `crates/pasta_lsp` の `AnalysisEngine::analyze`（`analysis/mod.rs` 78 行）が、`pasta_dsl::parse_str_partial` で書きかけの文書も部分的に構文木にし、セマンティックトークンと診断を作る。WASM の `wasm_analyze`（`transport.rs` 22 行）がそれを拡張へ渡す（`wasmBridge.ts` の `analyzeDocument`）。今の結果は `tokens` と `diagnostics` だけで、シーンの並びは渡していない。
- **構文木のシーン**: `pasta_dsl/src/parser/ast/scene.rs` の `GlobalSceneScope`（`name`・`is_continuation`・`local_scenes`・`span`）と `LocalSceneScope`（`name: Option<String>`・`span`）。名前の無い `LocalSceneScope` は、グローバルシーンの見出しの直後から最初のローカルシーンまでの部分（`local_start_scene_scope`）で、見出しの行を持たない。続きのグローバルシーン（`global_scene_continue_line`、`＊` の後に名前を書かない行）は、前のシーンの名前を受け継ぐ。
- **拡張のプロバイダ**: 今あるのは `semanticTokensProvider.ts` だけ。DocumentSymbol と CodeLens のプロバイダは無い。WASM が読めないときは、セマンティックトークンも作らず TextMate の着色だけで動く（`extension.ts`）。右クリックの 3 つのコマンドは WASM より前に登録していて、WASM なしでも動く。
- **マニュアル**: `book/src/debug/dev-actions.md` の「シーン再生（▶ シーンを実行）」が、右クリックからの使い方を書いている。

## Desired Outcome

- `.pasta` の見出しの行（グローバルシーン・ローカルシーンのどちらも）の上に、CodeLens「▶ シーンを実行」が出る。押すと、その見出しのシーンが、右クリックの「▶ シーンを実行」と同じ流れ（保存・SHIORI の読み込み直し・`OnPastaSceneAt`・出力パネルへの台本）で再生される。
- アウトライン・パンくず・`Ctrl+Shift+O`・スティッキースクロールに、グローバルシーン ＞ ローカルシーン の 2 階層が出る。
- 右クリックの「▶ シーンを実行」は、今までどおりカーソルの位置で動く。
- シーンの並びを取り出す処理は 1 か所だけにあり、アウトラインと CodeLens はどちらもそれを使う。

## Approach

**pasta_lsp の解析結果にシーンの並びを足し、拡張はそれを写すだけにする**（discovery で開発者が承認した推奨案）。

1. **シーンの並びは Rust で作る**: `AnalysisEngine::analyze` が作る構文木（部分解析を含む）から、シーンの並びを取り出して `AnalysisResult` に `scenes` として足す。1 つの項目は、名前・種類（グローバル／ローカル）・見出しの行・シーン全体の範囲・子（ローカルシーン）を持つ。WASM の `wasm_analyze` はそのまま渡す。文法の正典（pest）から作った構文木を使うので、全角・半角の印、続きのシーン、コードブロックの中の `＊` などを、拡張の側でもう一度判定しなくて済む。
2. **拡張は写すだけ**: DocumentSymbol のプロバイダと CodeLens のプロバイダが、どちらも同じ解析結果の `scenes` を使う。DocumentSymbol は、range をシーン全体、selectionRange を見出しの行にする。
3. **コマンドに引数を足す**: `pasta.runSceneAtCursor` が、引数（文書の uri と見出しの行）を受けたときはその文書のその行で再生し、引数が無いときは今までどおりアクティブなエディタのカーソルの行で再生する。CodeLens はこのコマンドを引数付きで呼ぶ。新しいコマンドは作らない。

退けた案: 拡張の TypeScript で見出しの行を正規表現で拾う案。TS だけで済むが、文法を二重に持つことになり、いずれパーサーとずれる。

### discovery で決めたこと（2026-11-09 開発者の承認）

1. **CodeLens の文言**は、既存のコマンドと同じ「▶ シーンを実行」。マニュアルの言い方と揃える。
2. **`pasta.codeLens.enabled` の設定は作らない**。VSCode の言語ごとの設定で同じことができる（`"[pasta]": { "editor.codeLens": false }`）。マニュアルにこの書き方を載せる。
3. **`onDidChangeCodeLenses` は使わない**。CodeLens が文書の中身だけから決まるので、VSCode が編集のたびに問い合わせ直す。自前のデバウンスも置かない（独自の設定を作らないので、発火する理由も無い）。
4. **続きのグローバルシーン**（`＊` の後に名前の無い行）は、受け継いだ名前の別の項目としてアウトラインに出し、CodeLens も付ける。名前の無い最初のローカル部分（グローバルシーンの見出しの直後から最初のローカルシーンまで）は、見出しの行が無いので項目にしない。
   ```
   ＊挨拶        → 挨拶 ▶
     ・朝        → 挨拶 > 朝 ▶
   ＊            → 挨拶 ▶（続き。2 つ目の「挨拶」）
   ```
5. **範囲**は実行時の決まりと揃える。シーンの範囲は見出しの行から次のシーンの見出しの直前まで、最後のシーンはファイルの終わりまで。ローカルシーンの範囲は親のグローバルシーンの範囲の中に収める（収めないとアウトラインの階層が崩れる）。
6. **WASM が読めないとき**は、アウトラインも CodeLens も出さない（セマンティックトークンと同じ扱い）。右クリックの「▶ シーンを実行」は、今までどおり WASM なしで動く。

## Scope

- **In**:
  - `pasta_lsp`: 解析結果にシーンの並び（階層・名前・種類・見出しの行・範囲）を足す。部分解析の結果からも取り出す。`wasm_analyze` が渡す形の安定のテスト（`transport.rs` の `test_analysis_result_json_shape_stable`）の更新。
  - `editors/vscode`: `wasmBridge.ts` の結果の型と検査に `scenes` を足す。DocumentSymbol のプロバイダと CodeLens のプロバイダを足して登録する。`pasta.runSceneAtCursor` を引数付きでも呼べるようにする（引数の文書を見て `accepts` を確かめ、その行を `runScene` へ渡す）。
  - テスト: Rust の側でシーンの並びを取り出すテスト（2 階層・範囲・空のファイル・続きのシーン・名前の無い最初のローカル部分・書きかけの文書・コードブロックの中の `＊`）。TS の側で、シーンの並びから DocumentSymbol と CodeLens への写しのテスト、引数付きのコマンドの呼び出しのテスト。
  - マニュアル: `book/src/debug/dev-actions.md` に、見出しの上の「▶ シーンを実行」とアウトラインを書き、CodeLens を消す設定（`"[pasta]": { "editor.codeLens": false }`）を添える。
- **Out**:
  - 実行時（pasta.dll・`OnPastaSceneAt`）の変更。行からシーンを引く規則は今のまま使う。
  - 独自の設定 `pasta.codeLens.enabled` と、`onDidChangeCodeLenses` による自前の更新。
  - アクター辞書（`％アクター`）・単語定義・コードブロックをアウトラインに出すこと。シーンだけを出す。
  - 名前でシーンを呼ぶ `OnPastaScene` を使った再生、同じ名前のシーンからのランダムな再生。
  - WASM が読めないときに TS で代わりに見出しを拾うこと。

## Boundary Candidates

- シーンの並びを取り出す処理（`pasta_lsp` の解析）と、それを VSCode の形に写す処理（拡張のプロバイダ）。
- 写す処理（DocumentSymbol・CodeLens）と、再生する処理（`devCommands.ts` の `execute` と `runSceneAtCursor.ts` の `runScene`）。CodeLens が渡すのはコマンドの引数だけで、再生の流れは共有する。

## Out of Boundary

- `OnPastaSceneAt` の行の解釈と、シーンマップ（`pasta_lua/src/debug/source_map/scene_index.rs`）。
- 出力パネルの書式と、読み込み直しの待ち方（`reloadShiori.ts`）。

## Upstream / Downstream

- **Upstream**: `pasta_dsl` の構文木（`GlobalSceneScope`・`LocalSceneScope` の span）と部分解析（`parse_str_partial`）。`vscode-mcp-client`（completed）の「▶ シーンを実行」の流れ。`debug-mcp-runtime`（completed）の `OnPastaSceneAt`。
- **Downstream**: 解析結果のシーンの並びは、のちにシーンへの移動（定義へ移動・参照の検索）や折りたたみに使える。

## Existing Spec Touchpoints

- **Extends**: なし（`vscode-mcp-client`・`pasta-vscode-extension` は completed。その spec の範囲を広げるのでなく、新しい spec として足す）。
- **Adjacent**: `scene-anchor-link`（VSCode の文法定義・LSP の着色を触る。`pasta_lsp` の解析の同じファイルに手が入るなら同じウェーブに置かない）。`vsce-4-upgrade`（`editors/vscode/package.json` を共有する。同じウェーブに置かない）。`pest-2-9-4-upgrade`（`pasta_lsp` の WASM のビルド）。

## 申し送り（2026-11-12 の discovery「pasta-recipe」より）

- **版を 0.5.3 にした**: VS Code を使わないエディタ `pasta-recipe` が、ファイルのツリーの下にシーンの並びを出すのに、この spec が足す `AnalysisResult.scenes` をそのまま使う。0.5.3 のウェーブ 1 に置く。
- **`scenes` の利用者が 2 つになる**: VS Code 拡張（WASM の `wasm_analyze` 経由）と、`pasta-recipe`（Tauri の Rust の側が `pasta_lsp` をライブラリとして直接呼ぶ。WASM は通らない）。`scenes` の形は、VS Code の DocumentSymbol に寄せすぎず、木を写すだけで使える素の形（名前・種類・見出しの行・範囲・子）のままにする。位置は UTF-16・0 始まりで、トークン・診断と揃える。
- **`AnalysisResult` は Rust の公開の型として使われる**: `pasta-recipe` の `src-tauri` が、別の Cargo ワークスペースから path 依存で `pasta_lsp` を引き、`AnalysisEngine::analyze` の結果を serde で JSON にする。`analyze` とその結果の型を、`pasta_lsp` の外から呼べる形（`pub`）に保つ。
- **`devCommands.ts`・`runSceneAtCursor.ts`**: この spec が引数付きの呼び出しを足した後で、`pasta-recipe-dev-actions`（0.5.3 のウェーブ 3）が実行の流れを `editors/shared/` へ移す。移しやすいように、引数の受け取りは `devCommands.ts`（VS Code の糊）の側に置き、`runSceneAtCursor.ts` の `runScene(ctx, fsPath, line1)` の形は変えない。

## Constraints

- 範囲（range）の行は、実行時の `OnPastaSceneAt` の決まりと食い違わないこと。見出しの行の CodeLens を押して、別のシーンが鳴ってはならない。
- DocumentSymbol の子の range は、親の range の中に収める（VSCode のアウトラインの決まり）。
- 位置は、既存のトークン・診断と同じく UTF-16 の列で渡す（`text_utils.rs` の `utf8_offset_to_utf16`）。
- 引数の無い呼び出し（右クリックのメニュー・コマンドパレット）の振る舞いは変えない。
- `pasta_lsp` の WASM のビルドは手元と CI の 3 か所で通ること（`pest-2-9-4-upgrade` の注意）。
