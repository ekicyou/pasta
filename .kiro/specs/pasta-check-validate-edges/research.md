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

---

# 設計フェーズの調査（2026-10-17）

## Summary

- **Feature**: `pasta-check-validate-edges`
- **Discovery Scope**: Extension（今の検査の核・2 つの殻・CI とテストを広げる。新しい依存・新しいファイルは無い）。軽い調べ（`design-discovery-light.md`）で進めた。
- **Key Findings**:
  - フォルダで pasta.dll の読み込みが失敗することを、**実際に動かして確かめた**（下の「確かめた振る舞い」）。中身の無い `dic/x.pasta/`・`dic/x.lua/`・入れ子の `dic/sub/y.pasta/` のどれも `Partial transpilation failure` で読み込み全体が失敗し、2 回目の読み込みでも同じ（キャッシュで逃れない）。
  - `glob` がフォルダに一致する規則は、ファイルの規則と 2 点だけ違う: パターンの最後の `**` は **1 段以上のフォルダ**に一致する（`dic` そのものには一致しない）。`/` で終わる・最後の成分が `.` のパターンはフォルダだけに一致する。ワイルドカードの無い成分は大文字・小文字を区別せずパターンの綴りで見え、ある成分は区別する（ファイルと同じ）。
  - 配布フォルダへの写し（`copy.rs` の `copy_dir_inner`）はフォルダごとに `create_dir_all` するので、中身の無いフォルダも写す。`release` の段 4 は配布フォルダを同じ核で検査するので、要件 4.4 は核を直すだけで効く。

## Research Log

### 確かめた振る舞い（pasta.dll のフォルダの扱い）

- **Context**: ギャップ分析は「コードを読んだだけ」だった。設計の前提（分類＝誤り・対象の範囲）を確かめるため、一時のテスト（`crates/pasta_check/tests/` に置いて走らせ、すぐ消した）で `pasta_lua::loader::discover_files` と `PastaLoader::load` を呼んだ。Windows・このブランチ（main 838fbde8 の後）。
- **Findings**（`pasta_patterns = ["dic/**/*.pasta"]`・`dic/a.pasta` あり。読み込みは 2 回ずつ）:

| 置いたフォルダ／パターン | `discover_files` の結果（フォルダは末尾 `/`） | 読み込み |
| ------------------------ | --------------------------------------------- | -------- |
| なし | `dic/a.pasta` | 成功 |
| 空の `dic/x.pasta/` | `dic/a.pasta`, `dic/x.pasta/` | 失敗（1 succeeded, 1 failed）・2 回目も失敗 |
| `dic/x.lua/` | `dic/x.lua/` を含む | 失敗・2 回目も失敗 |
| `dic/X.PASTA/` | 含まない（`*.pasta` は区別する） | 成功 |
| `dic/sub/y.pasta/` | `dic/sub/y.pasta/` を含む | 失敗 |
| `profile/z.pasta/` | 含まない | 成功 |
| パターン `dic/**`・`dic/sub/` | `dic/sub/` | 失敗（0 succeeded, 1 failed） |
| パターン `dic/`・`dic/*.pasta` | `dic/`, `dic/a.pasta` | 失敗 |
| パターン `dic/*`・`dic/sub/` | `dic/a.pasta`, `dic/sub/` | 失敗 |

- フォルダの一致の規則（`discover_files` だけを、`dic/sub/deep/deeper/`・`dic/a.pasta/b.pasta/`・`Dic2/x.pasta/`・`dic/.hidden.pasta/` とファイル `dic/f.pasta` で）:
  - `dic/**` と `dic/**/`: `dic` の下のすべてのフォルダ（1 段以上。`dic` そのものは含まない）。`**`・`**/` も同じで、`ghost/master` そのものは含まない。
  - `dic/.`・`dic`（ワイルドカードなし）: `dic/`。
  - `dic/**/*.pasta`: `dic/.hidden.pasta/`・`dic/a.pasta/`・`dic/a.pasta/b.pasta/`（入れ子は両方）と、ファイル `dic/f.pasta`。
  - `dic2/*.pasta`: `dic2/x.pasta/`（実物は `Dic2`。パターンの綴りで見える）。
- **Implications**: 分類は誤り（上流の OQ-1）で確定でよい。キャッシュで読み込みが通ることは無い（新しく作ったフォルダの更新時刻はキャッシュより新しく、`needs_transpile` が真になる。そもそもフォルダのキャッシュは一度も作られない）。フォルダの照合は、今の成分の照合を「最後の `**` は 1 段以上」「フォルダだけのパターンも使う」に広げれば写せる。

### 未確認の端（実装の一致テストで確かめる）

- パターン `""`・`.`・`./`（成分が 0 個）は、pasta.dll では `ghost/master` そのものに一致すると見込む（`Path::join("")` が末尾の区切りになり、`dic/` が `dic` に一致したのと同じ規則）。設計は「`ghost/master` そのものもフォルダの候補にする」としたが、仮定として記し、一致テストのパターンの組に入れて確かめる。

### VSCode の見張り（論点 e の続き）

- 拡張の見張りは文字列のパターンで張るので、ワークスペースの見張りの知らせを拡張ホストの側でパターンに照らしている（1.85〜main で同じ。ギャップ分析の出典）。パターンを広げても、届く知らせの数は変わらず、拡張の関数が呼ばれる回数が増えるだけ。
- テストのモック（`src/test/mock/vscode.ts` の `createFileSystemWatcher`）はパターンを記録するだけで照らさない。パターンに大文字・小文字の扱いを頼る案（文字の範囲で書く・`engines` を上げる）は、VSCode の版ごとの振る舞いをテストで確かめられない。パターンを `**/*` にして拡張の関数で絞る案は、絞る関数をモックの上でそのまま確かめられる。

### 一覧を集める 2 か所と WASM の形

- `GhostInput` を struct の式で作っている所は、殻 1 か所（`validate_cmd.rs`）とテスト 11 か所（`validate_config_test.rs`・`validate_parity_test.rs`・`validate_rules_test.rs`・`validate_scenes_test.rs`）。フィールドを足すと全部に `dirs` を足すことになる（機械的）。
- JS の形の検査は `transport.rs` の `test_ghost_validate_js_shape`（stats のキーを決め打ちで比べる）と `wasmBridge.ts` の `isGhostReport`（`referencesSkipped` を真偽値で確かめる）・`vscodeModules.test.ts`。`editors/vscode/wasm/` は `.gitignore` にあり、作り直しはビルドの手順（`build-wasm.ps1`）で行う。
- WASM の入口（`wasm_select_ghost_files`・`wasm_validate_ghost`）は serde で読み書きするだけなので、入口の関数は変わらない。

### 文書の当たり

- `book/src/debug/validate.md`: 54 行（出力の見本の案内の行）・62 行（省く条件）・106〜112 行（「ファイル」の表）・168 行（読むだけでは決まらない範囲）・207 行（検査する時）。
- `editors/vscode/README.md` 12 行（検査する時の `ghost/master`・`pasta.toml`）を直す。
- `crates/pasta_check/README.md` と `.claude/skills/pasta-check/SKILL.md` は、省く条件・報告の種類・検査する時を細かく書いていない（マニュアルへ案内しているだけ）ので変えない（要件 7.4 の Where に当たらない）。

### 使ったスキル・決まり

- `kiro-spec-design` の `design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`、テンプレート `design.md`・`research.md`。
- `pasta-check` スキル（報告・関門の書き方の確かめ）。`kiro-validate-design` の `design-review.md`（設計の検査）。
- 開発者の方針: 部分的な出荷をしない（同じ根の別の形も直す）、マニュアルに回避のレシピを載せない、起票に逃がさない、想定外の書き方を推測で救わない。

## Architecture Pattern Evaluation

| 案 | 内容 | 良い点 | 気になる点 | 結論 |
| -- | ---- | ------ | ---------- | ---- |
| 入力 A-1 | `entries` に `/` で終わる要素としてフォルダを入れる | JS の型の見た目が変わらない・テストの struct の式を直さない | 文字列の意味が変わる（隠れた約束）。殻が `/` を付け忘れると、フォルダがファイルとして選ばれ、読めないファイル（＝省く条件）に化ける | 採らない（代案） |
| 入力 A-2 | `GhostInput` に `dirs`（フォルダの相対パスの一覧）を足す。必須（既定値なし） | 形がはっきりする・付け忘れは JS の形の誤りとしてすぐ分かる | テストの struct の式 11 か所と JS の型を直す | **採る** |
| 見張り 1′ | `**/*` で張り、拡張の 1 つの関数（`ghost/master` と拡張子を大文字・小文字を区別せず照らす）で絞る | どの版でも同じ。モックで確かめられる。拡張子の大文字・小文字も保存の起動とそろう | 関数の呼ばれる回数が増える（1 回は文字列の分割だけ） | **採る** |
| 見張り 1 | `**/*.{pasta,lua}` に広げ、`findGhostMaster` で絞る | ギャップ分析の推奨に近い | 1.106 までは拡張子の大文字・小文字を区別したまま（`X.PASTA` の作成は版によって拾ったり拾わなかったり） | 代案 |
| 見張り 2・3 | 文字の範囲で書く／`engines` を `^1.107.0` に上げる | — | 2 は読みにくく拡張子は区別のまま。3 は利用者の下限を上げる。どちらもモックで確かめられない | 採らない |
| Stats S-1 | `references_skipped: bool` を `skip_reasons: Vec<SkipReason>` に置き換える（空＝検査した） | 判定と理由が 1 か所。真偽値と理由が食い違わない | JS のキーが変わる（`referencesSkipped` → `skipReasons`） | **採る** |
| Stats S-2 | 真偽値を残し、理由の一覧を足す | 今のキーが残る | 同じことを 2 つで持つ | 代案 |
| Stats S-3 | Stats を変えず、`lines()` が findings から理由を数え直す | JS の形が変わらない | 省く判定（`mod.rs`）と案内の行（`report.rs`）が別々に同じ規則を持つ | 採らない |

## Design Decisions

### Decision: 省く条件は種類の集まりで判定し、理由を Stats に持たせる

- **Context**: 要件 1.1〜1.7・OQ-1・OQ-2・OQ-11。
- **Alternatives Considered**: 上の表の S-1〜S-3。
- **Selected Approach**: `report.rs` に `SkipReason`（`Syntax`・`UnreadableFile`・`Config`）と、findings から理由を決める関数を置く。`mod.rs` はその結果で飛び先の検査を省き、`Stats.skip_reasons` に入れる。案内の行は理由を決まった順（構文エラー → 読めない辞書ファイル → pasta.toml の誤り）に「と」でつなぐ。理由が構文エラーだけのときの文言は今と同じ。
- **Rationale**: 判定の規則を 1 か所にし、案内の行と食い違わない。構文エラーだけのときの文言を変えないので、今のテスト・マニュアルの見本がそのまま通る。
- **Trade-offs**: JS の stats のキーが変わる（Revalidation Trigger に当たる。入力の形も同時に変わるので、WASM の作り直しは 1 回で済む）。

### Decision: フォルダは核が判定する（入力に `dirs` を足す）

- **Context**: 要件 2.1〜2.7・4.1・OQ-3・OQ-4。
- **Alternatives Considered**: 上の表の A-1・A-2。
- **Selected Approach**: 殻（CLI・拡張）は、入ったフォルダの相対パスを `dirs` に積む（最上位の `profile` とリンクは今までどおり除く）。核は `ghost/master` そのもの（空のパス）と `dirs` を候補にし、`.pasta` のパターンと `.lua` のパターンのどれかに pasta.dll と同じ規則で一致するフォルダを、新しい種類 `folder-matches-pattern`（誤り）で報告する。
- **Rationale**: パターンの照合を TypeScript に写さない（上流の OQ-6）。CLI・拡張・`release` の段 4 が同じ結果になる。
- **Trade-offs**: テストの struct の式を直す手間。
- **Follow-up**: 一致テストで、フォルダの選び出しを `discover_files` と照らす。成分が 0 個のパターンの振る舞いもそこで確かめる。

### Decision: 拡張は 1 つの関数で起動の条件を決め、ゴーストの鍵を小文字にそろえる

- **Context**: 要件 3.1〜3.7・OQ-5・OQ-6。
- **Alternatives Considered**: 上の表の見張り 1′・1・2・3。
- **Selected Approach**: `findGhostMaster` は大文字・小文字を区別せずに照らす（返す URI は届いた綴りのまま）。見張りは `**/*` で張り、届いた URI の祖先に `ghost/master` があり、名前が `.pasta`・`.lua` で終わる（区別しない）ときだけ起動する。保存は `.pasta`（区別しない）と、`ghost/master` の直下の `pasta.toml`（区別しない）で起動する。束ねと前回の報告の鍵は、`ghost/master` の URI のパスを小文字にした文字列にする。
- **Rationale**: どの版の VSCode でも同じに動き、モックで確かめられる。同じゴーストが違う綴りで届いても 1 回の検査・1 組の診断になる。
- **Trade-offs**: 大文字・小文字を区別するファイルシステム（Linux）で、綴りだけが違う 2 つのゴーストが同じ親フォルダにあると 1 つにまとまる（OQ-5 の「どの OS でも区別しない」の帰結として受け入れる）。

### Decision: 代用品の置き換え

- **Context**: 要件 5.1〜5.6・OQ-7〜OQ-10。
- **Selected Approach**: `manual.yml` の段を `cargo run -q -p pasta_check -- validate --target crates/pasta_sample_ghost/ghosts/hello-pasta` にし、コメントを直す（Rust の準備・キャッシュの段は残す）。`validate_cli_test.rs` の hello-pasta のテストで「誤り 0 件、注意 0 件」を確かめる。`tutorial_stages_test.rs` から `GLOBAL_KEYS` とその比較を外す（前方一致と `Kick`・`Gate` の検査は残す）。
- **Rationale**: `pasta_check` のバイナリは LuaJIT を持ち込まない（上流の要件 9.1）。組み込みの名前は `builtins.rs` が一致テストで実物と照らしていて、手書きの一覧より強い。

## Synthesis

- **一般化**: 要件 1 の 3 つの条件（構文エラー・読めない辞書・設定の誤り）は「検査した辞書のシーンの集まりが pasta.dll と同じと言えない」という 1 つの条件の別の形。種類→理由の 1 つの対応表で表す。要件 2 の「`.pasta` のフォルダ」「`.lua` のフォルダ」「`.pasta` で終わらないパターンのフォルダ」も「辞書のパターンに一致するフォルダ」1 つにまとめた。要件 3 の保存・作成・削除の起動は、1 つの照合の関数（`findGhostMaster`）と拡張子の判定の組み合わせで表す。
- **作るか使うか**: フォルダの照合は `glob` の振る舞いを一覧の上で写す今の作り（`discovery.rs`）を広げる。ファイルシステムに触れない核の決まり（WASM でも動く）があるので、`glob::glob` を直接使うことはできない。見張りは VSCode の `createFileSystemWatcher` をそのまま使う。
- **簡素化**: 新しいモジュール・新しいファイルは作らない。フォルダの照合は今の `matches` に「最後の `**` を 1 段以上にする」切り替えを足すだけにする。エディタに案内の行を出す仕組みは足さない（今もまとめの行を出していない）。`pasta.toml` の作成・削除での起動は要件に無いので足さない。

## Risks & Mitigations

- フォルダの照合の写し間違い — 一致テストに、上の表のパターンの組（`dic/**/*.pasta`・`dic/**`・`dic/`・`dic/*`・`dic/.`・`**`・`*/*.pasta`・成分 0 個）とフォルダの組を入れて `discover_files` と照らす。
- JS の形の変更の取りこぼし（`dirs`・`skipReasons`） — `transport.rs` の形のテストと `vscodeModules.test.ts` の形の検査を同じタスクで直す。`dirs` を必須にして、付け忘れは `null`（形の誤り）として出る。
- 診断の URI の綴りと、開いている文書の URI の綴りが違うと、エディタに波線が出ないことがある（問題パネルには出る） — 最後に起動した URI の綴りを使う。エクスプローラから開いた文書と見張りの知らせはどちらもディスクの綴りなので、ふつうは一致する。

## References

- 一時のテストの結果（上の表）。テストのファイルは残していない。
- `crates/pasta_lua/src/loader/discovery.rs` 59〜130 行・`process.rs` 105〜275 行・`cache.rs` 137〜165 行。
- `crates/pasta_check/src/copy.rs` 30〜62 行（中身の無いフォルダも写す）。
- VSCode `extHostFileSystemEventService.ts`（1.85.0・1.106.0・1.107.0・main。ギャップ分析の出典と同じ）。

## 設計ディスカッションの決定（2026-10-17）

全 8 議題を推奨案で確定した（開発者の一括承認）。根拠は design.md「確定した設計判断」にある。

- 議題 1〜7: 設計の生成で推奨した形（`dirs` 必須・見張り `**/*` と関数での絞り込み・`skipReasons` への置き換え・`folder-matches-pattern` と末尾 `/` なし・「と」のつなぎ・エディタに案内の行を出さない・`ghost/master` そのものを候補にして一致テストで確かめる）。
- 議題 8（検証の指摘）: 診断の URI の綴りと開いている文書の綴りが違うと波線が出ない端を、`render` で開いている文書の URI に寄せて直す。リスクとして残す案は、端を残すので採らなかった。
