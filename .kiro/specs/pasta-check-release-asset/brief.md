# Brief: pasta-check-release-asset

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票）。0.4.2 に入れる（2026-10-19 開発者の決定）。要件定義は Opus。

## Problem

Rust を入れていないゴーストの作者が困る。

- `pasta_check` を手に入れる道は、`cargo install pasta_check` しかない。
- そのため、辞書の検査（`pasta_check validate`）とゴーストのリリースビルド（`pasta_check release`）を使うには、Rust のツールチェーンを入れて、ソースからビルドしなければならない。
- `pasta.dll` は GitHub Release から落とせるのに、同じ作者向けの道具の `pasta_check` だけが落とせない。

## Current State

2026-10-19 に確かめた。

- **配布物**: リリース CI（`.github/workflows/release.yml`）の build job が作るのは、`pasta.dll.zip`・`hello-pasta.nar`・`pasta-vscode-X.Y.Z.vsix` の 3 つである。`pasta_check` は crates.io にしか出ていない。
- **ビルドの場所**: build job（windows-latest）が入れるターゲットは `i686-pc-windows-msvc` だけ。配布物は `crates/pasta_sample_ghost/release.ps1` が作る（手元の `release.bat` と同じスクリプト）。
- **x64 の下地はある**
  - ルートの `.cargo/config.toml` は、`x86_64-pc-windows-msvc` にも `crt-static` を付けている。VC++ ランタイムなしで動く exe が作れる。
  - `pasta_check` は `pasta_core`・`pasta_dsl` だけに依存し、LuaJIT を持たない。純粋な Rust で、C のツールチェーンを要さない。
  - `build.yml` の x64 の matrix は、すでに `cargo test --all --target x86_64-pc-windows-msvc` で `pasta_check` のバイナリを作っている。
- **ライセンス表示**: `about.toml`・`about.hbs` は pasta.dll と VSIX の WASM 向けで、LuaJIT の表示を固定で書いている。pasta_check には当てはまらない。
- **配布物の数を決め打ちしている所**
  - `release.yml` の「Verify release assets」と「Upload release-assets」
  - `.github/scripts/release/github-release.ps1` L18・L72・L79・L89（添付 4 つ＝配布物 3 つと詳しいリリースノート）と、そのテスト `release-scripts-test.ps1`
  - `.claude/skills/release/scripts/release-state.ps1` L84
- **入手の手順を書いている所**: `book/src/debug/validate.md` L23、`crates/pasta_check/README.md` L12、`.claude/skills/pasta-check/SKILL.md` L14-17。配布物の一覧は `README.md` L44、`book/src/introduction.md` L38、`crates/pasta_sample_ghost/RELEASE.md`、`tech.md` L210、`structure.md` L279 にある。

## Desired Outcome

- GitHub Release に、`pasta_check.zip` が 4 つ目の配布物として載る。`https://github.com/ekicyou/pasta/releases/latest/download/pasta_check.zip` で、最新版を単体で落とせる。
- 中身は次の 2 つだけ。

  ```
  pasta_check.zip
  ├─ pasta_check.exe          （x86_64-pc-windows-msvc・crt-static）
  └─ THIRD_PARTY_LICENSES.txt （pasta_check の依存だけのライセンス表示）
  ```

- 手元の `release.bat` とリリース CI が、同じ `release/` の一式（4 つ）を作る。
- リリース CI は、`pasta_check.zip` の中身が上の 2 つであることを検査し、足りなければ公開しない。
- マニュアル・README・スキルの入手の手順は、「Release から `pasta_check.zip` を落として展開する」を第一の道にする。`cargo install pasta_check` は、Rust の利用者向けの別の道として残す。

## Approach

既存の作りに 1 段足す（2026-10-19 開発者の承認）。

- **ビルド**: `release.ps1` に段を 1 つ足し、`cargo build --release --target x86_64-pc-windows-msvc -p pasta_check` で exe を作って zip にする。中身の検査は、`pasta.dll.zip` の段と同じ形にする。
- **ライセンス表示**: pasta_check の依存だけで `cargo about generate` を回す。LuaJIT の固定の表示を出さないように、テンプレートを分けるか、条件で出し分ける（どちらにするかは設計で決める）。
- **リリース CI**
  - build job の targets に `x86_64-pc-windows-msvc` を足す（rust-cache の key も合わせる）。
  - 「Verify release assets」に `pasta_check.zip` の検査を、「Upload release-assets」にその path を足す。
  - `github-release.ps1` の添付を 5 つ（配布物 4 つと詳しいリリースノート）にし、`release-scripts-test.ps1` を合わせる。
- **毎回の CI（`build.yml`）**: 新しい job は足さない。x64 の matrix が、すでに `pasta_check` をビルドしてテストしている。

## Scope

- **In**
  - `release.ps1` の `pasta_check.zip` の段（ビルド・ライセンス表示・zip・中身の検査）
  - pasta_check 用のライセンス表示の生成（`about.toml`・`about.hbs` か、その分け方）
  - `release.yml` の build job（targets・cache・検査・upload）
  - `github-release.ps1` と `release-scripts-test.ps1` の添付の数
  - `release` スキルの `release-state.ps1` の配布物の一覧
  - 入手の手順と配布物の一覧の文書（上の「Current State」の場所）と、スキル `references/` の再生成
  - マニュアルの入手の手順に、署名の無い exe なので SmartScreen の警告が出ることがある、と一言書く
- **Out**
  - コード署名（証明書が要る）
  - x86（i686）版の exe（開発者の指示で x64 だけ）
  - インストーラー・自動更新・PATH への登録
  - Windows 以外のビルド
  - `pasta_check` の機能の変更

## Boundary Candidates

- 配布物の作成（`release.ps1` と、ライセンス表示の生成）
- リリース CI の配線（`release.yml` の build job・`github-release.ps1` とそのテスト・`release` スキルの状態の読み取り）
- 入手の手順の文書（マニュアル・README・スキル・steering）

## Out of Boundary

- `pasta_check` の Rust のソース（`release.rs`・`copy.rs`・`nar.rs` などは `ponytail-check-release` が持つ）
- `release.yml` の publish 系の job の action の SHA 固定と Node の版（`ponytail-manual-ci` が持つ）
- `build.yml` の構成

## Upstream / Downstream

- **Upstream**: リリース CI（完了 spec `release-ci`）、`release.ps1`、cargo-about、ルートの `.cargo/config.toml` の `crt-static`
- **Downstream**: GitHub Release の利用者（ゴーストの作者）、`release` スキル（リリースの手順と、状態の読み取り）、0.4.2 のリリース

## Existing Spec Touchpoints

- **Extends**: なし（`release-ci` は完了済み。挙動はこの spec が足す）
- **Adjacent**
  - `ponytail-manual-ci`（ponytail・隙間の spec）: `.github/workflows/**` と `.github/scripts/release/*.ps1` を持つ。この spec が先に入り、`ponytail-manual-ci` が後から合わせる。この spec が触るのは `release.yml` の build job の行で、`ponytail-manual-ci` が触る publish 系の job の行・Node の版の行とは別である。
  - `ponytail-check-release`（ponytail・隙間の spec）: `pasta_check` の `release` まわりのソースと `pasta_sample_ghost/tests/**` を持つ。`release.ps1` は持たない。この spec は Rust のソースを触らない。

## Constraints

- **静的 CRT**: exe も `crt-static` で作る（ルートの `.cargo/config.toml`。動的に戻さない）。
- **LuaJIT のビルドの環境変数**: 手元で `cargo build` する前に `NoDefaultCurrentDirectoryInExePath` を外す（`pasta_check` 単体は LuaJIT を持たないが、`release.ps1` の他の段は持つ）。
- **リリースのスクリプトの起動**: `release.ps1` を `-ExecutionPolicy Bypass` で呼ぶと分類器が拒否する。今の pwsh から直に呼ぶ。
- **スキルの編集**: `.claude/skills/**`（`release-state.ps1`・`pasta-check/SKILL.md`）の編集は分類器が拒否することがある。拒否されたら、入った分と残りを示して開発者に許可を求める。
- **CI の確かめ方**: draft の PR は分類器が止める。途中の CI は `gh workflow run build.yml --ref <branch>` で回す。`release.yml` はタグの push でしか動かないので、配布物の作成は手元の `release.ps1` で、添付の数は `github-release.ps1 -DryRun` と `release-scripts-test.ps1` で確かめる。本番の確かめは 0.4.2 のリリースで行う。
- **ライセンス**: 許可するライセンスは `deny.toml` の `[licenses].allow` と `about.toml` の `accepted` で揃える。pasta_check の依存（`lexopt`・`md5`・`zip` など）が、そこに収まることを確かめる。
- **マニュアル**: 挙動の説明を変えたら、同じ変更で `node book/tools/gen-skill-refs.mjs` を回してスキル `references/` を作り直す。

## 2026-10-22 棚卸の再測定（main 6a8dc479）

- **前提の変化**: 2026-10-19 の後に入った FFI の読み込みの作業（`lua-cmodule-robustness`）は、毎回の CI（`.github/workflows/build.yml`）に FFI のテストの段を足しただけで、リリースの道具には触れていない。x64 の組み合わせが `pasta_check` をビルドしてテストすることも変わらない。Current State の行番号（`github-release.ps1` L18・L72・L79・L89、`release-state.ps1` L84）は今も合っている。
- **触るファイル**
  - `crates/pasta_sample_ghost/release.ps1`（263 行。段の見出しの `[4/6]` などの数も直す）
  - `.github/workflows/release.yml`（574 行。build の段だけ。検査は L131〜、添付は L189〜）
  - `.github/scripts/release/github-release.ps1`（132 行）、`.claude/skills/release/scripts/release-state.ps1`（L84〜87 の「/3」の表示も）
  - ライセンス表示のひな形（`about.toml`・`about.hbs`。分けるなら新しいひな形が 1 つ増える）と、Current State に挙げた文書
- **規模**: 7〜8 タスク。
- **先に要るもの**: なし。ほかの 0.4.2 の 4 本とソースのファイルを共有しない。`release.ps1`・`release.yml`・`github-release.ps1`・`release-state.ps1`・`about.*` を触る 0.4.2 の作業は、ほかに無いことを確かめた（後から隙間の `ponytail-manual-ci` が `release.yml`・`github-release.ps1` を合わせる）。マニュアルの `book/src/debug/validate.md` は `ontalk-suppression` と共有するが、こちらは入れ方の節（L20〜23）、あちらは組み込みの名前の一覧（L155）で、節が別である。`tech.md` も別の行。
- **種別**: 機能（新しい配布物を足す。2026-10-19 開発者の決定）。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - `release-scripts-test.ps1` が試すのは、リリースノートの作成・タグの確かめ・見どころの検査だけで、`github-release.ps1` の添付の数を試すテストは今は無い。「そのテストを合わせる」ではなく、添付の数の確かめを足すか、`-DryRun` だけで確かめるかを要件で決める。
  - 配布物の一覧や `release.ps1` の段の説明は、Current State のほかにもある。`crates/pasta_sample_ghost/README.md` L42・L75（段の並び）、`.claude/skills/pasta-check/SKILL.md` L107・L133・L153、`release.yml` L120 の段の名前「Build pasta.dll.zip and hello-pasta.nar」。
  - `pasta-check/SKILL.md` の入れ方は、今は L12〜18 にある（brief の L14-17 から少しずれた）。
  - `pasta_check` の CLI の依存（`lexopt`・`md5`・`zip`）は、任意の機能（feature）で入る。ライセンス表示を pasta_check だけで作るとき、CLI の機能を入れた形で数えることを設計で確かめる。
