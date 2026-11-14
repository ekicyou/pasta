# ギャップ分析: pasta-recipe-multiarch

- **日付**: 2026-11-14（worktree の base `61cf9a33`）
- **対象**: `requirements.md`（Requirement 1〜8）と今のコードの差
- **調べた範囲**: `editors/recipe/scripts/package.ps1`・`editors/recipe/src-tauri/`（`Cargo.toml`・`Cargo.lock`・`tauri.conf.json`・`about.toml`・`deny.toml`）・ルートの `.cargo/config.toml`・`.github/workflows/release.yml`・`build.yml`・`release-setup-check.yml`・`.github/scripts/release/github-release.ps1`・`release-scripts-test.ps1`・`.claude/skills/release/scripts/release-state.ps1`・`procedure.md`・`.claude/skills/pasta-check/SKILL.md`・`book/src/debug/recipe.md`・`introduction.md`・`README.md`・`crates/pasta_sample_ghost/RELEASE.md`・steering（`tech.md`・`structure.md`・`roadmap.md`）・`pasta-dll-multiarch/brief.md`

## Summary

- **Discovery Scope**: Extension（今の配布の道を arch で 2 本にする。新しい部品はほぼ要らない）。
- 作る道（`package.ps1`）は x64 に 3 か所で決め打ちしている（exe の置き場・`cargo about --target`・zip の名前）。引数で切り替えれば足りる。アプリの `Cargo.lock` は arch に依らない（`windows_aarch64_msvc` は既に入っている）ので、`--target` を足しても lock の不変検査はそのまま効く。
- **最大のギャップは Requirement 7（タグを打たない確かめ）**。`release.yml` は `push: tags` しか持たず、手動で起動できない。`build.yml` は `workflow_dispatch` を持つが、zip を作らない。どこで確かめるかを決めないと、x64 も含めた「CI の作成と検査がそろった」を本当のリリースの前に示せない。
- **ライセンスの表示と検査の道具も x64 に決め打ち**: `about.toml` の `targets`・`deny.toml` の `[graph] targets` が `x86_64-pc-windows-msvc` だけ。arm64 にしか出ない依存（`windows_aarch64_msvc`）が、表示にも cargo deny の検査にも載らないおそれがある。
- **確かめていない危険**: `windows-latest` で `tauri build --target aarch64-pc-windows-msvc` がクロスで通るか（ARM64 の MSVC・`cc`・`embed-resource` のリソースの埋め込み）。設計の最初に `workflow_dispatch` で試すべき項目。

## Requirement-to-Asset Map

| 要件 | 今ある物 | ギャップ | 種別 |
| --- | --- | --- | --- |
| 1.1 2 つの zip を作る | `release.yml` 138〜139 行で `package.ps1` を 1 回 | arch の引数が無い。2 回呼ぶ（または 1 回で 2 つ）形が要る | Missing |
| 1.2 中身 2 つ | `package.ps1` の自己検査・`release.yml` 218〜224 行 | 名前を arch ごとに | Missing |
| 1.3 arch ごとのライセンス表示 | `cargo about ... --target x86_64-pc-windows-msvc`・`about.toml` の `targets = ["x86_64-pc-windows-msvc"]` | arm64 の triple で作る。CLI の `--target` と `about.toml` の `targets` のどちらが勝つか未確認 | Missing / Unknown |
| 1.4 exe の版 | `tauri.conf.json` の `version` → `editors/vscode/package.json`。`release.yml` 228〜230 行が VersionInfo を見る | arm64 の exe でも同じ見方が使えるか（リソースは arch に依らないので使える見込み） | Constraint |
| 1.5 機械の種類 | 無い | PE ヘッダの Machine（`e_lfanew` + 4 の 2 バイト）を読む検査を足す | Missing |
| 1.6 静的 CRT | ルートの `.cargo/config.toml` は i686・x86_64 だけ。ファイルの頭の説明も「x86・x64」 | `[target.aarch64-pc-windows-msvc]` の `crt-static` と説明の更新。`pasta-dll-multiarch` と同じ 1 段 | Missing |
| 2.1〜2.7 検査と失敗の示し方 | `release.yml` 141〜243 行の `Verify release assets`（エラーを集めてから `::error::` で出す形） | pasta-recipe の節を arch の繰り返しにする。名前・期待した機械の種類を出す | Missing |
| 2.8 失敗で公開しない | `publish-*`・`github-release` は `needs: build` | そのまま満たす | 既存 |
| 2.9 lock の不変 | `release.yml` 246〜247 行 `git diff --exit-code -- Cargo.lock editors/recipe/src-tauri/Cargo.lock` | 2 つのビルドの後に置けばそのまま | 既存 |
| 2.10 後続へ渡す | `upload-artifact` の一覧 258 行 | 2 つの名前へ | Missing |
| 2.11 VC++ ランタイムの DLL を読まない（仮置き） | 無い（pasta.dll・pasta_check.exe も検査していない） | 取り込み表を読む検査（`dumpbin /dependents` か、pwsh で PE の import を読むか） | Missing（OQ 2） |
| 3.1〜3.8 手元の作成 | `package.ps1`（失敗で zip を残さない・自己検査あり） | `-Arch` 相当の引数・不正値の拒否・もう一方の zip を消さない・引数なしの振る舞い | Missing |
| 4.1〜4.4 添付 8 つ | `github-release.ps1` 72 行の 7 つの一覧・89 行の「7 つ（配布物 6 つ…）」の文 | 名前の一覧と文を 8 つへ | Missing |
| 4.5〜4.6 自己テスト | `release-scripts-test.ps1` の Test-Item 10・11・13（7 つ中 1 つ欠け） | 10・11 の置くファイルを 2 つの zip へ。13 を x64 欠け・arm64 欠けの 2 項目へ | Missing |
| 5.1〜5.2 状態の集計 | `release-state.ps1` 84〜87 行（7 つ・`n/7`） | 8 つ・`n/8` | Missing |
| 5.3〜5.4 手順書 | `procedure.md` 110〜116 行の表（`6/7 以下`・`0/7`）と読み替え（`4/7`・`6/7` → 7/7）、484 行の数え方 | `/8` に。読み替えは 0.4.2〜0.5.2 = `6/8`、0.4.1 以下 = `4/8` | Missing |
| 6.1・6.3 main の CI の arm64 | `build.yml` の job `recipe`（`windows-latest`・`x86_64` だけ・最後に `tauri build --no-bundle`）。`release.yml` の gate が `build.yml` を `workflow_call` で呼ぶ | arm64 のビルドの段。gate には自動で入る | Missing |
| 6.4 arm64 のランナーのテスト（仮置き） | 無い | `windows-11-arm` の job（OQ 3） | Missing / Unknown |
| 7.1〜7.5 タグを打たない確かめ | `release.yml` は `push: tags` だけ。`build.yml` は `workflow_dispatch` あり（zip は作らない）。`release-setup-check.yml` が「手動起動・公開しない」の前例 | 確かめの実行の置き場と、リリース CI と同じ手順を共有する形（OQ 5） | Missing |
| 8.1〜8.4 マニュアル・README | `book/src/debug/recipe.md` 17〜26 行・293 行、`introduction.md` 39 行、`README.md` 47 行 | 固定のリンク 2 つ・arch の調べ方・対象の書き直し | Missing |
| 8.5〜8.7 名前の付け替え | 下の「名前の残り」の 16 ファイル | 一斉の書き換え | Missing |

### 名前の残り（`pasta-recipe.zip`、`completed/` を除く）

| ファイル | 件数 | 扱い |
| --- | --- | --- |
| `.github/workflows/release.yml` | 8 | 書き換え（step 名・検査・upload）。頭の説明「6 つの配布物」も 7 つへ |
| `editors/recipe/scripts/package.ps1` | 2 | 書き換え |
| `.github/scripts/release/github-release.ps1` | 2 | 書き換え |
| `.github/scripts/release/release-scripts-test.ps1` | 4 | 書き換え（項目を 1 つ足す） |
| `.claude/skills/release/scripts/release-state.ps1` | 1 | 書き換え（分類器が拒むことがある） |
| `.claude/skills/release/procedure.md` | 2 | 書き換え（同上） |
| `.claude/skills/pasta-check/SKILL.md` | 1 | 書き換え（同上。`pasta-skills.zip` に入るので、作った後の中身の照合に効く） |
| `README.md` | 1 | リンク 2 つへ |
| `book/src/introduction.md` | 1 | リンク 2 つへ |
| `book/src/debug/recipe.md` | 2 | リンク 2 つ・arch の選び方 |
| `crates/pasta_sample_ghost/RELEASE.md` | 2 | 書き換え |
| `.kiro/steering/tech.md` | 1 | 書き換え（配布物 7・添付 8・機械の種類の検査） |
| `.kiro/steering/structure.md` | 2 | 書き換え |
| `.kiro/steering/roadmap.md` | 5 | 38 行の配布物の一覧などを書き換え。この spec の台帳の行は「名前を変えたことの説明」として残してよい（8.6） |
| `.kiro/specs/pasta-dll-multiarch/brief.md` | 1 | 107 行は調査の記録（行番号の控え）。8.6 の除外。ただし `release.yml` の行番号が変わるので、申し送りを足す候補（OQ 6） |
| `.kiro/specs/pasta-recipe-multiarch/brief.md` | 8 | この spec の記録。除外 |

`book/` の固定のリンクは GitHub の外のリンクなので、マニュアルの CI の `link-check` には掛からない見込み（Research Needed: `link-check.mjs` が `releases/latest/download/` を見るか）。`book/src/debug/recipe.md` の写しは配る pasta のスキルに無い（`gen-skill-refs --check` には効かない）。`pasta-check/SKILL.md` は `pasta-skills.zip` に入り、`manual.yml` の起動条件にも入っている。

## 実装の形の候補

### Option A: 今の部品を伸ばす（brief の推奨の形そのまま）

- `package.ps1` に `-Arch x64|arm64` を足し、triple・exe の置き場（`src-tauri/target/<triple>/release/`）・`cargo about --target`・zip の名前を切り替える。
- `release.yml`: `rust-toolchain` の targets に `aarch64-pc-windows-msvc` を足し、`package.ps1` を 2 回呼ぶ。`Verify release assets` の pasta-recipe の節を `foreach ($arch in 'x64','arm64')` にし、機械の種類の検査を足す。upload の一覧を 2 つに。
- `build.yml` の job `recipe` に `cargo build --target aarch64-pc-windows-msvc --locked`（または `tauri build --target ... --no-bundle`）の段を足す。
- `.cargo/config.toml`・`about.toml`・`deny.toml` に aarch64 を足す。
- 確かめ（R7）は別に用意が要る（下の Option B か C の一部を借りる）。

**長所**: 差が最も小さい。今の書き方（検査を `release.yml` の中に直に書く）を崩さない。
**短所**: R7 を満たせない。`release.yml` の検査はタグでしか走らないので、検査の書き間違いは本当のリリースの日まで見つからない（今の `pasta-recipe.zip` の検査も一度も走っていない）。

### Option B: 検査をスクリプトに出し、main の CI でも同じ道を走らせる

- pasta-recipe の 2 つの zip の検査（中身・版・機械の種類・表示の名前・大きさ・〔OQ 2 なら〕VC++ の DLL）を `.github/scripts/release/` の下の 1 本のスクリプト（例 `verify-recipe-zips.ps1 -Version X.Y.Z -Dir release`）に出す。
- `release.yml` はタグの版で、`build.yml` の job `recipe` は `editors/vscode/package.json` の版で、同じスクリプトを呼ぶ。
- `build.yml` の job `recipe` の最後の `tauri build --no-bundle`（今も x64 の release ビルドを 1 回している）を、`package.ps1` で 2 つの arch を作る段に置き換え、検査のスクリプトを走らせ、2 つの zip を artifact に残す。これで R6.1・R6.3（arm64 の壊れを main で見つける）と R7（`gh workflow run build.yml --ref <branch>` で確かめる。公開しない）が同じ段で満たせる。
- 検査のスクリプトの自己テストを `release-scripts-test.ps1` に足せる（偽の PE ヘッダの 2 バイトを書いた小さなファイルで、機械の種類の取り違えを検出するか）。

**長所**: リリースの日の前に、リリースと同じ作成と検査が毎回の push で走る。検査の書き間違いを main で見つけられる。R6 と R7 が 1 つの仕組みで済む。
**短所**: `build.yml` の job `recipe` に `cargo-about` の版固定の取得（今は `release.yml` の `env` と step にある）を持ち込む要がある。job の時間は arm64 の release ビルド 1 回分と `cargo about` 2 回分だけ延びる。`release.yml` と `build.yml` の両方で `cargo-about` の版と SHA-256 を持つと二重になる（共有の置き場を設計で決める）。

### Option C: B に加えて arm64 のランナーで確かめる

- B の上に、`build.yml` へ `windows-11-arm` の job を足す。arm64 でネイティブに `cargo test --locked`（Rust のテスト）を走らせ、〔OQ 3 で選べば〕作った exe を数秒起動して落ちないことを見る。
- リリースの配布物は B のまま x64 のランナーでクロスで作る（brief が退けた「job を 2 つに分けて成果物を受け渡す」形は取らない。arm64 の job は確かめだけで、成果物を渡さない）。

**長所**: arm64 の exe を誰も起動していないまま配る状態を避けられる。「x86 のテストは省かず期待を分ける」の方針（Rust のテストを出荷する arch で走らせる）と揃う。
**短所**: ランナーが 1 つ増える（公開リポジトリなら無料）。GUI の起動の煙試験はランナーの対話の画面と WebView2 の有無に依る（Research Needed）。`windows-11-arm` の Node・Rust・MSVC の用意の差で壊れる箇所が増える。

## 確かめの要る項目（Research Needed。設計へ持ち越す）

1. **クロスのビルドが通るか**: `windows-latest` で `npx tauri build --no-bundle --target aarch64-pc-windows-msvc`。ARM64 の MSVC（`Microsoft.VisualStudio.Component.VC.Tools.ARM64`）がイメージに在るか、`cc`（`vswhom-sys`）と `embed-resource`（`rc.exe` は host のもの）がクロスで通るか。brief の指示どおり、設計の最初に `workflow_dispatch` で試す。
2. **`cargo about` の `--target` と `about.toml` の `targets`**: CLI の `--target` が設定を置き換えるのか足すのか。arm64 の表示に `windows_aarch64_msvc` が載るかを、作った表示で確かめる。`deny.toml` の `[graph] targets` に aarch64 を足すと cargo deny の結果が変わらないか。
3. **exe の置き場**: `--target` を付けると `src-tauri/target/<triple>/release/pasta-recipe.exe`。x64 も `--target x86_64-pc-windows-msvc` を付けるか（付ければ置き場がそろう。付けなければ今の `target/release/` のまま）。どちらでも `.cargo/config.toml` の `crt-static` は効く（`--target` を付けたときは build script には効かない差がある）。
4. **フロントの作り直し**: `beforeBuildCommand`（`npm run build`）は arch ごとに走る。2 回目を省くか（`--config` で `beforeBuildCommand` を空にする等）は時間の問題だけで、結果は同じ。`js-licenses.txt` も arch に依らない。
5. **WebView2 の読み込み**: `webview2-com-sys` 0.39 が arm64 で loader の DLL を要らないか（今の x64 版は exe 1 つで動いている）。zip の中身を 2 つに保てるかに効く。
6. **機械の種類の読み方**: pwsh で `[System.IO.File]::ReadAllBytes` → `e_lfanew`（0x3C の 4 バイト）→ `PE\0\0` の後の 2 バイト。`dumpbin /headers` は VS の開発の環境が要る。
7. **VC++ の DLL の検査（OQ 2 を採るなら）**: `dumpbin /dependents`（vswhere で探す）か、pwsh で import の表を読むか。x64 の exe でも今は検査していないので、まず今の x64 の exe が本当に `vcruntime140.dll` を読まないかを見る。
8. **`windows-11-arm` のランナー（OQ 3 を採るなら）**: 公開リポジトリで使えること、`dtolnay/rust-toolchain`・`actions/setup-node`・`Swatinem/rust-cache` が arm64 の Windows で動くこと、GUI の exe を起動できる画面があること。
9. **`link-check.mjs`** が `releases/latest/download/` の外のリンクを見ないこと（新しい名前のリンクは、0.6.0 を出すまで 404 になる）。

## 複雑さと危険

- **Effort: M（3〜7 日）** — 書き換える場所は 16 ファイルと多いが、どれも今の形の延長。重いのはクロスのビルドの確かめと、R7 の確かめの道の新設。
- **Risk: Medium** — クロスのビルド（ARM64 の MSVC・リソースの埋め込み）が未確認。arm64 の exe を起動して確かめる手段が x64 のランナーに無い。`.claude/skills/` の書き換えが分類器に止められることがある（止まったら許可を求める）。

## 設計への申し送り

- **推奨の形**: Option B（OQ 3 で arm64 のランナーを採るなら Option C）。R7 を満たすには、リリースと同じ作成と検査を、タグを打たずに走らせる道がどうしても要る。今の `build.yml` の job `recipe` は既に x64 の release ビルドを 1 回しているので、それを `package.ps1` の 2 つの arch に置き換えるのが最も足し算が少ない。
- **決めること**: `package.ps1` の引数の形（OQ 4）、`cargo-about` の版固定の共有の置き場、x64 にも `--target` を付けるか、検査のスクリプトの名前と置き場、`about.toml`・`deny.toml` の targets。
- **隣の spec**: `pasta-dll-multiarch` と `.cargo/config.toml` の aarch64 の段・`release.yml` の build job（rust の targets・cache の key・検査の step）を共有する。同じウェーブに置かない。後に入る方が、在るのを確かめる。
- **版**: 0.6.0（2026-11-14 開発者の決定。v0.5.2 の次のリリース）に入れる前提で、古い名前の互換（固定のリンクの転送など）と、`release-state.ps1` の過去の版の数え方の追加は作らない（OQ 1）。
