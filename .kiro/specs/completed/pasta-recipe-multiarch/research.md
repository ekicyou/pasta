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

---

# 設計の調査（2026-11-14・`/kiro-spec-design`）

- **Discovery の種別**: Extension（light discovery）。今の作る道（`package.ps1`）・検査（`release.yml` の `Verify release assets`）・関門（`build.yml`）を arch で 2 本に伸ばす。新しいライブラリは足さない。
- **参照したスキル・規則**: `kiro-spec-design` の `design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`。メモリの「x86 のテストは省かず期待を分ける」（出荷する arch で Rust のテストを走らせる）・「作成ツールは問題があれば止める」（見つからない道具は飛ばさずに失敗）・「pasta.dll は静的 CRT」・「ponytail」。

## Research Log

### R1: 手元でのクロスのビルド（`aarch64-pc-windows-msvc`）

- **Context**: ギャップ分析の Research Needed 1。`embed-resource`・`vswhom-sys`（`cc`）と Tauri 2.12 がクロスで通るか。
- **やったこと**: 手元（Windows 11 x64・VS 18 Community・rustc 1.100.0）で、`aarch64-pc-windows-msvc` の target は入っていた。VS の `Microsoft.VisualStudio.Component.VC.Tools.ARM64` も入っていた（`vswhere -requires` で確認）。`.cargo/config.toml` は書き換えず、同じ意味の環境変数 `CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_RUSTFLAGS=-Ctarget-feature=+crt-static` を付けて、`editors/recipe` で `npm ci` → `npx tauri build --no-bundle --target aarch64-pc-windows-msvc` を走らせた（`NoDefaultCurrentDirectoryInExePath` は外した）。
- **結果**: **通った**（`Finished release profile in 6m 22s`。`embed-resource`・`vswhom-sys` の C のビルドも、LTO のリンクも通った）。できた `src-tauri/target/aarch64-pc-windows-msvc/release/pasta-recipe.exe` は 9,005,056 バイト。
  - PE の Machine は **`0xAA64`**（pwsh でバイトを読んで確かめた。`e_lfanew` の先は `PE`）。
  - `FileVersionInfo` の版は `0.5.2`（`editors/vscode/package.json` の版。今の版の見方が arm64 の exe でもそのまま使える）。
  - `dumpbin /nologo /dependents` の DLL に **`vcruntime*`・`msvcp*` は無い**（静的 CRT が効いている）。ただし **`api-ms-win-crt-*-l1-1-0.dll`（UCRT の 7 つ）は在る**。同じ形を、ルートの `.cargo/config.toml`（`+crt-static`）で作った手元の x64 の `pasta-recipe.exe`（別の worktree の `target/release/`）でも確かめた（同じ 7 つが在る。`vcruntime*` は無い）。どの部品が UCRT を DLL で引くかは調べていない。UCRT は Windows 10 から OS に入っているので、VC++ の再頒布可能パッケージを入れていない Windows でも起動できる（1.6 は満たす）。
  - ビルドの後も `git status` は変わらなかった（`editors/recipe/src-tauri/Cargo.lock` は `--target aarch64-pc-windows-msvc` で書き換わらない。2.9 の検査はそのまま効く）。
  - 同じ手元で `cargo about generate --target <triple>` を 2 つの triple で走らせた: arm64 の表示は 238,614 バイト（MIT が 225 件）、x64 は 239,868 バイト（226 件）、`--target` なし（`about.toml` の `targets` = x64）は x64 と同じ 239,868 バイト。どちらも `tauri` の名前を含む。テンプレートが crate の名前を全部は書かないので、`windows_aarch64_msvc` の名前そのものは表示に出ない。
- **Implications**: クロスのビルドは手元で通る（CI での確認は実装の最初の `workflow_dispatch`）。VC++ ランタイムの検査（2.11）の禁止の名前に UCRT（`ucrtbase*`・`api-ms-win-crt-*`）を入れると、今の x64 の exe も落ちるので入れない。`--target` は `about.toml` の `targets` を置き換える（足すのなら arm64 の表示が x64 より小さくはならない）。

### R2: CI のランナーの ARM64 の MSVC

- **Sources**: [Windows2025-VS2026-Readme.md](https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-VS2026-Readme.md)（Image Version 20261103.298.1）・[GitHub Actions: Upcoming image migrations](https://github.blog/changelog/2026-05-14-github-actions-upcoming-image-migrations)。
- **Findings**: `windows-latest` は 2026-06 から Windows Server 2025 ＋ Visual Studio Enterprise 2026（18.11）。部品の一覧に `Microsoft.VisualStudio.Component.VC.Tools.ARM64` がある。VS 2022 の版（Windows2025-Readme.md）にも `VC.Tools.ARM64` があった。
- **Implications**: `windows-latest` でクロスのビルドに要る道具は揃っている見込み。`dumpbin.exe` も `vswhere -find 'VC/Tools/MSVC/**/bin/Hostx64/x64/dumpbin.exe'` で見つかる（手元で確認）。CI での確認は実装の最初の `workflow_dispatch` の実行で行う。

### R3: `cargo about` の `--target` と `about.toml` の `targets`

- **Sources**: 手元の `cargo about generate --help`（cargo-about 0.9.3。CI は 0.9.2 を固定）。
- **Findings**: `--target` の説明は「Overrides the `targets` configuration value」（設定を置き換える。足さない）。複数の target を取れる。
- **Implications**: `package.ps1` が arch ごとに `--target <triple>` を渡せば、その arch の依存だけの表示になる（1.3）。`about.toml` の `targets` は `--target` なしで動かしたときの既定にすぎないので、2 つの triple を並べて実態に合わせる。0.9.2 で同じかは確かめていない（仮定。0.9.2 と 0.9.3 は小さな版の差）。arm64 の表示に `windows_aarch64_msvc` が載ることは、作った表示で確かめる（実装）。

### R4: `windows-11-arm` のランナー（WebView2・画面）

- **Sources**: [arm-windows-11-image.md](https://github.com/actions/partner-runner-images/blob/main/images/arm-windows-11-image.md)・[Windows on Arm runners for all public repos](https://blogs.windows.com/windowsdeveloper/2025/04/14/github-actions-now-supports-windows-on-arm-runners-for-all-public-repos/)・[GitHub-hosted runners の一覧](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)・[windows-11-arm の VS 2026 への切り替え（第三者の記事）](https://ecorpit.com/github-actions-windows-11-arm-vs2026-migration-september-2026/)・UI の試験についての第三者の記事（[sep.com](https://sep.com/blog/executing-windows-desktop-app-system-tests-in-ci/) ほか）。
- **Findings**:
  - 公開リポジトリは `runs-on: windows-11-arm` で無料で使える（4 vCPU・16 GB）。Windows 11 の desktop の image。
  - image の一覧に Rust（rustup 1.28.2）・Node（20・22・24 のキャッシュ）・PowerShell 7.4・Visual Studio（2026 へ切り替え済み。32 ビット ARM の道具などが外れた）・**Microsoft Edge 143** がある。**WebView2 ランタイムは一覧に書かれていない**。Windows 11 は WebView2 ランタイムを OS に含むので在る見込み（確かめていない）。
  - **対話の画面（interactive session）の有無は公式の文書に書かれていない**。x64 の hosted の Windows のランナーは対話のセッションで動くという第三者の記事が多いが、反対の報告もある。arm の image についての記述は見つからなかった。
- **Implications**: 煙試験は「画面に何が出たか」を見ず、「exe が落ちない」と「WebView2 の子のプロセス（`msedgewebview2.exe`）が立ち上がる」の 2 つで判定する（下の Decision）。どちらも対話の画面が無くても判定できる見込みだが、確かめていない。実装の最初の `workflow_dispatch` で走らせ、起動できなければ要件 6.5 のとおり開発者に戻す。

### R5: 起動の煙試験で WebView2 の欠けを見逃さないこと

- **Context**: `editors/recipe/src-tauri/src/main.rs` は、WebView2 ランタイムが無いとき `MessageBoxW` を出してから `exit(1)` する。メッセージ箱は閉じるまで戻らないので、「決めた時間のうちにプロセスが終わらない」だけを見ると、WebView2 が無いのに成功と判定する。メッセージ箱の題名も窓の題名も `pasta-recipe` で、題名でも見分けられない。
- **Findings**: WebView2 が動くと、exe の子として `msedgewebview2.exe` が立つ（`Get-CimInstance Win32_Process` の `ParentProcessId` で引ける）。
- **Implications**: 煙試験の合格を「時間のうちに子の `msedgewebview2.exe` が立ち、その時点で exe が生きている」にする。

### R6: `pwsh -File` での arch の引数の拒否

- **やったこと**: `param([ValidateSet('x64','arm64')][string[]]$Arch = @('x64','arm64'))` の小さなスクリプトを `pwsh -NoProfile -File x.ps1 -Arch x86` で呼んだ。
- **Findings**: 終了コード 1 で、受け付ける値（`"x64,arm64"`）を示すメッセージが出る。引数なしは両方、`-Arch arm64` は 1 つ。
- **Implications**: 3.3・3.6 は `ValidateSet` と既定値だけで満たせる。自前の検査を書かない。

### R7: そのほかの確かめ

- `book/tools/link-check.mjs` は GitHub の `blob`・`tree` の URL だけをリポジトリのパスに写し、外のリンクは飛ばす。`releases/latest/download/` の新しいリンク（0.6.0 を出すまで 404）は検査に掛からない。
- 名前の残りのうち `.kiro/specs/pasta-dll-multiarch/brief.md` の 2 か所は、107 行が調査の記録、127 行がこの spec からの申し送り（名前を変えたことの説明）で、どちらも 8.6 の除外に入る。`roadmap.md` の 125 行（0.6.0 の範囲の `pasta-recipe` の形）・382 行（入った `pasta-recipe` の台帳の行）は除外に入らないので書き換える。
- `release-scripts` の job は `ubuntu-latest` で走る。新しい検査のスクリプトの自己テストは、Windows の道具（`dumpbin`・`FileVersionInfo` の版の読み取り）に頼らない形にする。
- `cargo-about` の版と SHA-256 は `release.yml` の `env` にだけある（`CARGO_ABOUT_VERSION`・`CARGO_ABOUT_SHA256`。ほかに読む所は無い）。`pasta-dll-multiarch/brief.md` 107 行がこの `env` の行番号を控えている。

## Architecture Pattern Evaluation

| 形 | 中身 | 長所 | 短所 |
| --- | --- | --- | --- |
| A. `release.yml` に直に書く | ギャップ分析の Option A | 差が最小 | タグでしか検査が走らない。R7 を満たせない |
| **B. 検査をスクリプトに出し、`build.yml` の `recipe` でも走らせる** | `package.ps1`（両 arch）＋ `verify-recipe-zips.ps1` を `release.yml` と `build.yml` の両方から呼ぶ | R6.1・R6.3・R7 が 1 つの道で済む。検査の書き間違いを毎回の push で見つける | `cargo-about` の取得を `build.yml` にも持ち込む |
| **C. B ＋ `windows-11-arm` の job** | arm64 で Rust のテストと、x64 のランナーで作った arm64 の zip の exe の起動 | 配る exe そのものを arm64 で起動する | ランナーの用意の差・画面の有無が未確認 |

要件の議題 3・4（2026-11-14）で C に決まっている。

## Design Decisions

### Decision: 検査は 1 本のスクリプトにし、リリース CI と main の CI が同じものを呼ぶ

- **Context**: 7.3（片方だけ直して食い違う形にしない）と 2.1〜2.7・2.11。
- **Alternatives**: (1) `release.yml` と `build.yml` に同じ検査を写す。(2) 検査のスクリプトを 1 本にする。
- **Selected**: (2)。`.github/scripts/release/verify-recipe-zips.ps1 -Version X.Y.Z [-Dir release] [-DumpbinCommand <path>]`。置き場はほかのリリースの補助スクリプトと同じ。`release.yml` の `Verify release assets` は、ほかの配布物の検査の後でこのスクリプトを呼び、どちらかが失敗すれば失敗する（2.7 の「全部示してから失敗」を step の中で保つ）。
- **Trade-offs**: `release.yml` の検査が 2 か所（直書きとスクリプト）に分かれる。ほかの配布物の検査は今回は動かさない（範囲の外）。

### Decision: VC++ ランタイムの検査は `dumpbin /dependents`

- **Alternatives**: (1) `dumpbin /dependents`（`vswhere -find` で探す）。(2) pwsh で PE の import の表を読む（section の表で RVA を写す。30 行ほど）。
- **Selected**: (1)。どちらの arch の exe も、x64 の host の `dumpbin` で読める。見つからなければ飛ばさずに失敗する（作成ツールは問題があれば止める）。自己テストは `-DumpbinCommand` に偽のスクリプトを渡す（`publish-vsix.ps1` の `-VsceCommand` と同じ前例）。
- **禁止の名前**: `vcruntime*`・`msvcp*`・`vccorlib*`・`concrt*`・`msvcr<数字>*`（VC++ の再頒布可能パッケージが入れる DLL）。UCRT（`ucrtbase.dll`・`api-ms-win-crt-*`）は入れない。R1 のとおり、`+crt-static` で作った今の x64 の exe も arm64 の exe も `api-ms-win-crt-*` を読む。UCRT は Windows 10 から OS の部品なので、1.6（再頒布可能パッケージなしで起動できる）には効かない。`msvcrt.dll`（OS の部品）は `msvcr<数字>` に合わない。

### Decision: 機械の種類は pwsh でバイトを読む

- PE の `e_lfanew`（0x3C の 4 バイト）→ `PE\0\0` → 次の 2 バイト。`dumpbin /headers` より短く、Linux の自己テストでも動く。x64 = `0x8664`、arm64 = `0xAA64`。

### Decision: `package.ps1` は `-Arch` を `ValidateSet` で受け、既定で両方を作る

- x64 にも `--target x86_64-pc-windows-msvc` を付ける。exe の置き場が `src-tauri/target/<triple>/release/` にそろい、取り違え（前の arch の exe を詰める）を置き場で防ぐ。
- 消すのは作る arch の zip だけ（3.7）。arch ごとに作り、途中で失敗したら、その arch の作りかけの zip を消して終了コード 1（3.5）。先に作り終えた arch の zip は残る（完成品なので）。
- フロントの作り直し（`beforeBuildCommand`）は arch ごとに走るままにする（数秒。省く設定を足さない）。

### Decision: `cargo-about` の取得はスクリプトに出して 2 つの workflow で共有する（設計の OPEN QUESTION 1）

- **Context**: `build.yml` の `recipe` が `package.ps1` を走らせるには `cargo-about` が要る。版と SHA-256 は `release.yml` の `env` にしかない。
- **Alternatives**: (1) `build.yml` に `env` と取得の loop を写す（版の二重持ち）。(2) 取得の loop と版・SHA-256 を `.github/scripts/release/install-tools.ps1` に出し、`release.yml`（wasm-pack と cargo-about）と `build.yml`（cargo-about）が呼ぶ。(3) `cargo install cargo-about --version 0.9.2 --locked`（数分かかり、版はやはり二重）。
- **Selected（仮）**: (2)。`pasta-dll-multiarch` の brief が控えている `release.yml` の `env` の行が消えるので、申し送りが要る。

### Decision: arm64 のランナーの job は 2 つに分ける

- `recipe-arm64-test`（`needs` なし）: arm64 でネイティブに `npm ci` → `npm run build` → `cargo test --locked`（6.4）。`generate_context!` が `dist/` を要るので、フロントを先に作る（今の `recipe` の job と同じ順）。
- `recipe-arm64-smoke`（`needs: recipe`）: `recipe` の job が残した zip の artifact を落とし、`pasta-recipe.arm64.zip` の exe を起動する（6.5）。配る exe（x64 のランナーでクロスで作った物）そのものを arm64 で動かす。ツールチェーンを入れない。
- 1 つの job にすると、テストも `recipe` の 2 回の release ビルドを待つ。2 つにすれば、テストは最初から並んで走る。

### Decision: 煙試験は「落ちない」と「WebView2 の子が立つ」で判定する

- 30 秒まで 1 秒ごとに見る。exe が終わったら失敗（終了コードを示す）。子の `msedgewebview2.exe` が立ち、その時点で exe が生きていれば成功。30 秒たっても立たなければ失敗（WebView2 が無くてメッセージ箱で止まっている場合もここで落ちる。R5）。最後に exe を止める。窓の題名・画面の写しは見ない（対話の画面に頼らない）。

## Synthesis

- **一般化**: 2.1〜2.7・2.11 と 7.1 は「版 V と zip の置き場を受けて、2 つの zip を同じ項目で見る」1 つの検査。版の出どころ（タグか、`editors/vscode/package.json` か）だけが呼ぶ側で違う。検査のスクリプトは版を引数で受け、出どころを知らない。
- **Build vs Adopt**: zip（`System.IO.Compression`）・PE の import（`dumpbin`）・引数の検査（`ValidateSet`）・artifact（`actions/upload-artifact`・`download-artifact`）は既存の道具を使う。新しい依存は無い。
- **簡素化**: 検査のスクリプトに arch の一覧を引数で渡さない（`x64`・`arm64` の 2 つを中に持つ。3 つ目は範囲外）。`package.ps1` は zip の中身の並びだけを自分で確かめ（3.4）、版・機械の種類・DLL は検査のスクリプトに任せる（手元でも同じスクリプトを呼べる）。`build.yml` に `Cargo.lock` の不変の検査は足さない（`cargo test --locked` がある。2.9 はリリース CI の要件）。

## Risks & Mitigations

- `windows-11-arm` で起動できない（画面・WebView2）— 実装の最初の `workflow_dispatch` で確かめ、できなければ 6.5 のとおり開発者に戻す。黙って外さない。
- `windows-11-arm` の Node・Rust・rust-cache の用意の差 — `setup-node`・`dtolnay/rust-toolchain` は arm64 の Windows に対応している。失敗したら実装で直す。
- `main` の CI が 1 回の release ビルドぶん（arm64 の LTO）と `cargo about` 2 回ぶん長くなる — 受け入れる（PR は CI を待たずにマージする運用）。
- `release.yml` の rust の targets・cache の key・`.cargo/config.toml` を `pasta-dll-multiarch` と共有 — 後に入る方が在るのを確かめる。同じウェーブに置かない。
- `.claude/skills/` の書き換えが分類器に拒まれる — 回避策を探さず、入った分と残りを示して許可を求める。
