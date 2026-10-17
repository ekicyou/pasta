# ギャップ分析: pasta-check-validate-edges

2026-10-17。基準は main 838fbde8（brief の再測定の b4164100 から、この spec の触るファイルは変わっていない）。

## まとめ

- 3 つの端は、どれも今の部品を少し広げれば直る。新しいファイルは要らない見込み。依存も足さない（`Cargo.lock` は変わらない）。
- いちばん大きな発見は、論点 b の前提の誤り。brief は「pasta.dll はフォルダを読まない」としたが、コードを読むと pasta.dll はフォルダを辞書として読もうとして失敗し、ゴーストの読み込み全体が失敗する。上流の決まり（読み込みに失敗するものは誤り）から、分類は**誤り**になる（要件の OQ-3）。
- 論点 e: VSCode の見張りのパターンは、**1.106 までは大文字・小文字を区別**し、1.107 から大文字・小文字を区別しないファイルシステムの上で区別しなくなった。拡張は `engines.vscode: ^1.85.0` なので、古い版では見張りを直さないと `Ghost/Master` の作成・削除を拾わない。
- 論点 c・d: 前方一致の重なりは、上流の要件 5.3 で「報告しない」と確定していて、入門ガイドが機能として教えている。足さないのが筋。`manual.yml` の段は `validate` に置き換えられる（hello-pasta は今、誤り 0・注意 0）。`scene_names_do_not_prefix_collide` は hello-pasta の段階辞書の決まりの検査なので、組み込みの名前の部分だけを外せる。
- 規模は 5〜7 タスク（brief の見積もりどおり）。入力の形を変えるので、WASM の作り直しと `wasmBridge.ts`・`transport.rs` の形のテストに効く。

## 今の状態

### 検査の核（`crates/pasta_check/src/validate/`）

- `mod.rs` の `validate_ghost`（55〜91 行）が、設定 → 集め方 → モジュール名・構文 → トランスパイル・シーン → 飛び先、の順に段を呼ぶ。飛び先を省く条件は 74 行の 1 か所だけ: `findings.iter().any(|f| f.code == FindingKind::Syntax)`。
- 読めないファイルは `syntax.rs` の `check`（8〜34 行）が `FileNotUtf8`・`FileUnreadable` の誤りにして、構文木を作らない。そのファイルのシーンは登録されないので、そこへの Call・選択肢が `call-target-missing`・`choice-target-missing` の注意になる（上流の tasks.md の 2.4 の注記どおり）。
- 殻が渡し忘れたファイルも `FileUnreadable`（「読み込まれていません」）になる（`mod.rs` 105〜109 行）。殻の側の「一覧にありません」も同じ種類。
- `pasta.toml` を読めないときは `config.rs` 42〜44 行で `FileUnreadable`（パスは `pasta.toml`）になる。TOML として読めない・型の誤り・別名表の誤りは `ConfigSyntax`・`ConfigLoaderType`・`ConfigSceneAlias`、パターンとして読めないのは `ConfigPatternInvalid`（`discovery.rs` 47〜53 行）。どれも、誤りのまま既定値で続ける。
- 案内の行とまとめの行は `report.rs` の `Report::lines`（213〜230 行）。案内の文言は「構文エラーがあるため、飛び先の検査をしていません。」で決め打ち。`Stats.references_skipped` は真偽値（183 行）。
- 核の入力 `GhostInput`（`input.rs`）は `pastaToml` と `entries`（ファイルの相対パスの一覧）だけ。フォルダは入らない。
- 辞書の選び出し `discovery.rs` は、一覧の各要素をパターンの成分と照らす（`collect` 134〜153 行・`matches` 156〜179 行）。最後の `**` はフォルダにしか一致しないので、ファイルには一致させない（162〜163 行）。

### 一覧を集める 2 か所

- CLI: `crates/pasta_check/src/validate_cmd.rs` の `walk`（114〜134 行）。フォルダの中に入るが、フォルダそのものは一覧に入れない。最上位の `profile` に入らず、シンボリックリンクを含めない。`ghost/master` は `target.join("ghost").join("master")`（41 行）で探すので、Windows では大文字・小文字を区別しない。
- 拡張: `editors/vscode/src/ghostValidator.ts` の `walk`（263〜277 行）。CLI と同じ集め方で、フォルダは入れない。

### VSCode 拡張の範囲

- `findGhostMaster`（18〜27 行）: `segments[i] === 'master' && segments[i - 1] === 'ghost'`。大文字・小文字を区別する。
- 見張り（70 行）: `createFileSystemWatcher('**/ghost/master/**/*.{pasta,lua}')`。
- 保存（81〜89 行）: `.pasta` で終わる（区別する）か、`${master.path}/pasta.toml` と同じ（区別する）ときに起動する。
- ゴーストの鍵は `master.toString()`（100・131 行）。違う綴りで届くと、同じゴーストが 2 つの鍵に分かれる。
- 報告の `stats` は表示に使っていない（案内の行はエディタには出ない）。

### 代用品

- `.github/workflows/manual.yml` 142〜143 行: `cargo test -p pasta_sample_ghost`。ubuntu で走り、`pasta_lua`（LuaJIT）をビルドする。137〜141 行のコメントは「置き換えは別に決める」。
- `crates/pasta_sample_ghost/tests/tutorial_stages_test.rs` 450〜483 行の `scene_names_do_not_prefix_collide`: (1) グローバルシーン名どうしの前方一致、(2) `Kick`・`Gate` で始まらない（`pasta_shiori` の e2e と衝突しない）、(3) 手書きの `GLOBAL_KEYS`（`yield`・`チェイントーク`・`ゴースト終了`・`close_ghost`。401〜402 行）と同じでない、の 3 つを確かめる。根拠は `hello-pasta-tutorial-stages` の要件 2.5（新しい名前を既存の名前で始めない）・5.3（e2e が足すシーンと衝突しない）。
- `build.yml` は main への PR とプッシュのすべてで、Windows で `cargo test --all` を走らせる（49 行。パスの絞り込みは無い）。hello-pasta の段階のテストも、`pasta_check` の hello-pasta のテスト（`validate_cli_test.rs` 176〜192 行。今は「誤り 0 件」だけを確かめる）も、ここで走る。

## 論点ごとの根拠

### 論点 a: 読めないファイルがあるときに省く範囲（要件 1・OQ-1・OQ-2）

- 全体を省く案は、74 行の条件に種類を足すだけで済む。ファイルごとに省く案は、読めないファイルにどのシーンがあったかが分からないので、どの Call が引きずられたかを決める手がかりが無い（brief の推奨の理由と同じ）。
- 同じ根の問題が `pasta.toml` にもある。`pasta.toml` が壊れると既定値（`dic/**/*.pasta`・既定の別名表）で集めるので、`pasta_patterns` で足した場所の辞書が集まらず、そこのシーンへの Call が「見つからない」と出る。別名表が壊れたときも、`＊雑談` を `OnTalk` の別名にしていたゴーストで `＞OnTalk` が「見つからない」と出る。
- 省く条件に入れる候補の種類: `Syntax`（今）、`FileNotUtf8`、`FileUnreadable`（辞書ファイルと `pasta.toml`）、`ConfigSyntax`、`ConfigLoaderType`、`ConfigPatternInvalid`、`ConfigSceneAlias`。
- 入れない種類: `ConfigMissing`（既定値で集めるのは pasta.dll と同じ）、`PatternIgnored`・`PatternNotPasta`（注意。pasta.dll も同じパターンを無視する）、`NoDictionaryFiles`、`InitFileName`、トランスパイルの 3 つ、モジュール名の 3 つ（どれも構文木はあり、シーンは登録される）、論点 b の新しいフォルダの誤り（フォルダにシーンは無い）。
- 案内の行の文言を変えると、Revalidation Triggers の「1 行の形・まとめの行の形を変えるとき」に当たる。決め打ちの文言を確かめているのは `validate_report_test.rs`（158 行〜）・`validate_scenes_test.rs`（422・436 行）・`validate_cli_test.rs`・マニュアル `validate.md`（54・62・168 行）。

### 論点 b: 辞書のパターンに一致するフォルダ（要件 2・OQ-3・OQ-4）

- pasta.dll の振る舞い（コードを読んで確かめた。実機での読み込みはしていない）:
  - `crates/pasta_lua/src/loader/discovery.rs` の `discover_files`（59〜129 行）は `glob` の結果をそのまま積む。`glob` はフォルダにも一致し、`is_file` での絞り込みは無い。
  - `process.rs` の `process_incremental`（103 行〜）は、キャッシュが無ければ `needs_transpile` が真（`cache.rs` 137〜144 行）になり、`fs::read_to_string` でフォルダを読もうとして失敗し、`failures` に積む（138〜148 行）。`.lua` の側も同じ（221〜236 行）。
  - 失敗が 1 つでもあると `LoaderError::partial_transpile` で読み込み全体が失敗する（257〜271 行）。
  - 上流の一致テスト `validate_parity_test.rs` 121〜124 行のコメントも「pasta.dll はそのフォルダを辞書として読もうとして失敗する」と書き、`is_file()` で絞って比べている。上流の tasks.md 3.1 の注記も同じ。
- したがって brief の推奨（注意）の前提は当たらない。上流の確定事項 OQ-1（読み込みに失敗するものは誤り）と要件 9.4 から、誤りになる。
- 対象は `.pasta` で終わる名前に限らない。`.lua` のパターン（`dic/**/*.lua`）に一致するフォルダ、`.pasta` で終わらないパターン（`dic/*`・`dic/**`）に一致するフォルダも、同じ理由で読み込みを失敗させる。
- 中身の無いフォルダは、今の一覧には何も現れないので、核に渡す一覧にフォルダそのものを入れないと見つけられない。
- `release` は段 4 で同じ核を使うので、配布フォルダにそのフォルダがあれば関門で止まる。配布フォルダへのコピー（`copy.rs` の `copy_dir_recursive`）が中身の無いフォルダを写すかは、設計で確かめる（写さなければ、配布物には入らないので止まらなくても正しい）。

### 論点 c: 前方一致の重なりの注意（要件 5.4・OQ-9）

- 上流の要件 5.3 は「名前どうしの前方一致の重なり（`＊挨拶` と `＊挨拶朝`）を、重なりとして報告しない」と確定している。
- `hello-pasta-tutorial-stages` の要件 2.2 は、11 段目で前方一致を機能として教えると決めている（`＞挨拶` が `＊挨拶朝`・`＊挨拶夜` の両方を候補にする）。書いたとおりに動く使い方に注意を出すことになる。
- `novel-talk-flow`（brief 11・27・40・54 行）は、ノベル向けの設定で場面の名前を完全一致で選ぶ道を入れる（部品は `touch-poke-shortcuts` で入った。brief 112〜114 行）。`＊第1章` と `＊第10章` の問題は、場面の選び方で解く。ノベル向けの設定に合わせた検査を足すかは、その spec の要件で決めることになっている（brief 127 行）。
- 足さないなら、この spec の範囲は増えない。

### 論点 d: 代用品の置き換え（要件 5.1〜5.3・5.5・5.6・OQ-7・OQ-8・OQ-10）

- `cargo run -q -p pasta_check -- validate --target crates/pasta_sample_ghost/ghosts/hello-pasta` を今の main で走らせた結果: 「検査した辞書ファイル 12 個: 誤り 0 件、注意 0 件、飛び先を検査しなかった Call 1 件」。置き換えても今は通る。
- `pasta_check` のバイナリは LuaJIT を持ち込まない（上流の要件 9.1）。ubuntu のマニュアルの CI で、`pasta_lua` のビルドが要らなくなる。
- 置き換えると、マニュアルの CI から外れるのは、段階の辞書を読み込んで応答を確かめるテスト・失敗表記が出ないことのテストなど。これらは実行時の検査で、`build.yml` がどの変更でも走らせる。ただしマニュアルの公開（`deploy` は `needs: build` だけ）は `build.yml` を待たない（PR は CI を待たずにマージする運用）。公開の前の関門として何を残すかは、要件ディスカッションで確かめる価値がある。
- `validate` は注意だけなら 0 で終わる。hello-pasta の注意 0 件を守るなら、`validate_cli_test.rs` の hello-pasta のテストに「注意 0 件」を足すのが最小。マニュアルの CI でも注意で止めるなら、まとめの行を読むか、オプションを足すことになる（上流の要件 7.7 の「分類を入れ替えない」には当たらないが、上流の OQ-5 は出力の形を最小にしている）。
- `scene_names_do_not_prefix_collide` の 3 つのうち、`validate` に重なるのは (3) の組み込みの名前だけ。`validate` の組み込みの名前の一覧（`builtins.rs`）は act と SHIORI 用の act のメソッドも含み、一致テスト（上流の 3.2 (e)）で実物と照らしているので、手書きの `GLOBAL_KEYS` より強い。(1)(2) は hello-pasta の段階辞書の決まりで、一般の検査ではない。

### 論点 e: VSCode の見張りの大文字・小文字（要件 3・OQ-5・OQ-6）

- VSCode の `FileSystemWatcher`（`src/vs/workbench/api/common/extHostFileSystemEventService.ts`）を版ごとに読んだ:
  - 1.85.0〜1.106.0: `const parsedPattern = parse(globPattern);` で、大文字・小文字の扱いの指定が無い。`vs/base/common/glob` の既定は区別する。`**/ghost/master/**` は `Ghost/Master` の下の作成・削除に一致しない。
  - 1.107.0 から: 文字列のパターンは、`file` のファイルシステムが `PathCaseSensitive` を持たなければ `ignoreCase` で照らす（67〜71 行。今の main は、両方を小文字にしてから照らす）。Windows・macOS では区別しない。
  - 出典: https://raw.githubusercontent.com/microsoft/vscode/1.85.0/src/vs/workbench/api/common/extHostFileSystemEventService.ts 、同 1.106.0・1.107.0・main。
- VSCode の API の文書は、大文字・小文字を区別しない OS では、見張りが知らせるパスの綴りがディスクの綴りと違うことがあると書いている。保存の通知の URI とも綴りが違い得るので、ゴーストの鍵を綴りのまま比べると、同じゴーストが 2 つに分かれる（要件 3.6）。
- 拡張は `engines.vscode: ^1.85.0`（`editors/vscode/package.json` 25 行）。
- 見張りの直し方の候補（設計で決める）:
  1. 見張りを `**/*.{pasta,lua}` に広げ、届いた URI を、大文字・小文字を区別しない `findGhostMaster` で絞る。どの版でも同じに動き、コードがいちばん少ない。ワークスペースの全体の `.pasta`・`.lua` の作成・削除が届くが、`ghost/master` の外は捨てるだけ。
  2. 文字の範囲で書く（`**/[gG][hH][oO][sS][tT]/[mM][aA][sS][tT][eE][rR]/**/*.{pasta,lua}`）。版に依らないが読みにくい。拡張子の大文字・小文字は 1.106 までは区別したまま。
  3. `engines.vscode` を `^1.107.0` に上げる。利用者の VSCode の版の下限が上がる。
- CLI の大文字・小文字: Windows では区別しない（OS のファイルシステム）。Linux・macOS（区別するファイルシステム）では区別する。pasta.dll と SSP は Windows だけで動く。

## 要件と今の部品の対応

| 要件 | 今の部品 | ギャップ |
| ---- | -------- | -------- |
| 1.1〜1.3・1.6 省く条件 | `mod.rs` 74 行 | Missing: `Syntax` だけ。種類を足す |
| 1.4・1.5 案内の行 | `report.rs` 213〜230 行・`Stats.references_skipped` | Missing: 理由が「構文エラー」に決め打ち。理由を持たせると `Stats` の形（JS の `referencesSkipped`）に効く（Constraint） |
| 1.7 | `syntax.rs`・`config.rs` | 変更なし |
| 2.1〜2.7 フォルダ | `input.rs`・`discovery.rs`・2 か所の `walk` | Missing: 一覧にフォルダが無い。照合（`matches`）はファイルの一覧向けで、最後の `**` をファイルに一致させない作り。フォルダ向けに照らす道が要る。新しい報告の種類（`FindingKind` の 24 個目）と文言 |
| 3.1・3.4・3.5・3.7 ゴーストを探す | `findGhostMaster` | Missing: 大文字・小文字を区別する |
| 3.2 見張り | `register` 70 行 | Constraint: 1.106 までの VSCode は区別する（論点 e） |
| 3.3・3.4 保存の起動 | `register` 81〜89 行 | Missing: `pasta.toml`・`.pasta` を区別する |
| 3.6 二重にしない | `ghosts`・`schedules` の鍵（`master.toString()`） | Missing: 綴りが違うと別の鍵になる |
| 4.1〜4.3 同じ報告 | `transport.rs` の WASM の入口・`wasmBridge.ts` の形の検査・`render` | 入力の形を変えるなら、両側と形のテスト（`transport.rs` 408 行〜・`vscodeModules.test.ts`）を合わせる。診断はフォルダの URI にも付けられる（今の `render` は `f.path` から URI を作るだけ） |
| 4.4 release | `release.rs` 34 行〜（段 4） | 同じ核なので自動で効く。中身の無いフォルダのコピーは Unknown |
| 5.1・5.2 マニュアルの CI | `manual.yml` 130〜143 行 | 置き換え。Rust の準備の段は残す |
| 5.3 注意 0 件 | `validate_cli_test.rs` 176〜192 行 | Missing: 「誤り 0 件」だけ |
| 5.4 前方一致 | 上流の要件 5.3 | 変更なし |
| 5.5・5.6 hello-pasta のテスト | `tutorial_stages_test.rs` 401〜402・450〜483 行 | `GLOBAL_KEYS` の部分を外す |
| 6.3・6.4 一致テスト | `validate_parity_test.rs` 117〜134 行 | Missing: フォルダを `is_file()` で外している。読み込みの失敗を照らすテストを足す（`PASTA_DEBUG` のガード要） |
| 6.5・6.6 拡張のテスト | `ghostValidator.test.ts`・モック（`createFileSystemWatcher` 391 行） | 足す |
| 7.1〜7.3 マニュアル | `validate.md` 54・62・101〜157・168・207 行 | 直す |
| 7.4 README・スキル | `crates/pasta_check/README.md`・`editors/vscode/README.md`・`.claude/skills/pasta-check/SKILL.md` | 今は省く条件・報告の種類を細かく書いていない見込み。拡張の README の「検査する時」は確かめる。スキルの編集は分類器が止めることがある |

## 実装の案

### 案 A: 今の部品を広げる（推奨の見込み）

- 省く条件: `mod.rs` の 74 行を、種類の集まりで判定する形にする。理由は `Stats` に持たせ（真偽値を理由の一覧にするか、真偽値を残して理由を足すか）、`report.rs` の案内の行を作り直す。
- フォルダ: 核の入力にフォルダを入れる。入れ方は 2 通り。
  - A-1: `entries` に `/` で終わる要素としてフォルダを入れる。JS の形は `string[]` のまま。`wasmBridge.ts` の入力の型の見た目は変わらないが、意味が変わる。
  - A-2: `GhostInput` に `dirs`（フォルダの相対パスの一覧）を足す。形がはっきりするが、JS の形が変わる（`transport.rs` の形のテスト・`wasmBridge.ts` の型を直す）。
  - どちらも、2 か所の `walk` がフォルダを積み、`discovery.rs` にフォルダを照らす道（最後の成分までパターンと照らし、最後の `**` もフォルダには一致させる）を足し、新しい種類の誤りにする。
- 拡張: `findGhostMaster` を大文字・小文字を区別しない比較にし、見張りは論点 e の候補から選ぶ。保存の `pasta.toml`・`.pasta` の比較も区別しない形にし、ゴーストの鍵をそろえる（例 `findGhostMaster` が返す URI を鍵にする前に正規の綴りにそろえる。どうそろえるかは設計）。
- 代用品: `manual.yml` の段を `cargo run -p pasta_check -- validate --target …` にする。`validate_cli_test.rs` に注意 0 件を足す。`tutorial_stages_test.rs` から `GLOBAL_KEYS` を外す。
- 良い点: ファイルが増えない。今の作り（核が規則を持ち、殻は集めるだけ）を守る。
- 気になる点: `report.rs`（379 行）・`ghostValidator.ts`（314 行）が少し増える。600 行の目安には届かない。

### 案 B: 一覧を集める 2 か所でフォルダを別々に報告する

- `walk` が、名前が `.pasta` で終わるフォルダを見つけたら、殻の側で報告を足す。核の入力の形は変わらない。
- 気になる点: パターン（`pasta_patterns`）に従うには照合を 2 か所に写すことになり、TypeScript に照合を写さないという上流の決まり（OQ-6・design）に反する。`.pasta` で終わる名前だけに絞ると、`pasta.toml` でパターンを変えたゴーストと `.lua` のフォルダで、pasta.dll と結果がずれる。
- 推奨しない。

### 案 C: 組み合わせ

- 省く条件・代用品・拡張の大文字・小文字は案 A、フォルダだけを後回しにする、などの分け方。
- 気になる点: 同じ根の問題を別に送ることになる（開発者の方針「起票に安易に逃がさない」）。規模も小さいので分ける理由が無い。

## 規模とリスク

- 規模: S（1〜3 日）。今の部品の延長で、新しい依存も新しいファイルも無い。タスクは 5〜7（省く条件と案内の行 1、フォルダの核と殻 1〜2、拡張 1〜2、代用品 1、文書 1）。
- リスク: 低。規則は pasta.dll の決まった振る舞いに合わせるだけで、一致テストで守れる。入力の形を変える（案 A-2）と、WASM の作り直しと形のテストの直しが要るが、上流の Revalidation Triggers に手順がある。

## 設計に持ち越す調べもの

- 入力の形（案 A-1 と A-2 のどちらにするか）と、`Stats` の形（省いた理由の持ち方）。どちらも Revalidation Triggers に当たる。
- フォルダの照合: `discovery.rs` の `matches` の最後の `**` と、`glob::glob` がフォルダに一致する規則の写し方。`dic/**` のようなパターンで、フォルダがどこまで一致するか。一致テストで pasta.dll と照らす。
- pasta.dll がフォルダで読み込みに失敗することを、一致テストで実際に照らす（今回はコードを読んだだけ）。前回の読み込みでキャッシュがあるときに `needs_transpile` が偽になる場合の振る舞い（フォルダの更新時刻とキャッシュの比較）も確かめる。
- `copy_dir_recursive` が中身の無いフォルダを配布フォルダに写すか。
- VSCode の見張りの書き方（論点 e の 3 案）。モックの `createFileSystemWatcher`（`src/test/mock`）で、どの案もテストできるか。
- ゴーストの鍵のそろえ方（違う綴りで届いた URI を 1 つのゴーストにまとめる方法）。
- 拡張の README と、スキル `pasta-check` の記述のどこが変わるか。
