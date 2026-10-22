# Implementation Plan

- [x] 1. 下地: 変更前の記録とライセンス表示のひな形
- [x] 1.1 変更前のリリースビルドを通し、比べるための記録を残す
  - cargo を走らせる前に、環境変数 `NoDefaultCurrentDirectoryInExePath` を外す（LuaJIT のビルドが止まるため）
  - `x86_64-pc-windows-msvc` のターゲットと cargo-about が手元に入っていることを確かめる（無ければ入れる）
  - 変更前の `release.ps1` を pwsh から直に呼んで通す（`-ExecutionPolicy Bypass` は使わない）
  - `pasta.dll.zip` と `hello-pasta.nar` の中身の名前の一覧、`ghosts\hello-pasta\ghost\master\THIRD_PARTY_LICENSES.txt` のハッシュを、スクラッチのファイルに書き出す
  - 完了の姿: 変更前の 3 つの記録がスクラッチにあり、2.3 の比較に使える
  - _Requirements: 2.4, 3.7_

- [x] 1.2 pasta_check.exe 用のライセンス表示のひな形を作る
  - ルートに `about-pasta_check.hbs` を置く。冒頭に「pasta_check.exe のための表示」「pasta 自体は MIT License」を日本語と英語で書き、本体は今の `about.hbs` と同じ形の一覧と節にする。LuaJIT の固定の節は持たない
  - `about.toml` は冒頭のコメントに、このひな形も同じ設定を使うと一言足すだけで、値は変えない。`about.hbs` は触らない
  - pasta_check だけを根にし、x64 のターゲットに絞った cargo-about の生成をスクラッチへ回し、`lexopt`・`md5`・`zip` があり、`LuaJIT`・`mlua`・`wasm-bindgen` が無く、冒頭の文があることを確かめる
  - `Zlib` を除いた `about.toml` の写しで同じ生成を回し、終了コード 1 で出力のファイルができないことを確かめる
  - 完了の姿: 生成した表示が上の条件を満たし、許可に無いライセンスでは生成が失敗する
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 2. リリースビルドに pasta_check.zip の段を足す
- [x] 2.1 pasta_check.zip を作って検査する段を足す
  - 準備の段（1〜2）の後、`pasta_check release` の段の前に、新しい段を置く。`-SkipSetup`・`-SkipDllBuild` でも必ず走る
  - 古い zip を消す → cargo-about の有無を検査（無ければ入れ方を示して失敗）→ x64 の exe をビルド（失敗なら `rustup target add x86_64-pc-windows-msvc` を示す）→ 1.2 のひな形で表示を生成（失敗なら zip を作らない）→ zip にまとめる → 中身がちょうど 2 つかを検査（違えば期待と実際を示して失敗）、の順にする
  - exe と表示は `target\` の下に置き、`release\` には zip だけを出す。`release\` が無ければ作る
  - 今の `pasta.dll.zip` の段の zip と検査の行の形を写し、関数に括り出さない。Windows PowerShell 5.1 で動く書き方に限る
  - 完了の姿: `release.ps1` を通すと `release\pasta_check.zip` ができ、中身は最上位に `pasta_check.exe` と `THIRD_PARTY_LICENSES.txt` の 2 つだけ
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.5, 2.6, 3.2, 3.3, 3.4, 3.6_

- [x] 2.2 `pasta_check release` の段を x64 の exe で走らせ、段の並びと表示を合わせる
  - `pasta_check release` の段を、`cargo run -p pasta_check` から 2.1 で作った exe の起動に替える。引数と失敗の扱いは変えない
  - 段の見出しを `[1/7]`〜`[7/7]` に、冒頭の説明・`-SkipSetup` の説明・リリースの段の範囲の注釈を合わせる
  - 頭の表示とまとめに `Check Zip:` の行を足す
  - `release.bat` の冒頭の説明（作る物と段の並び）を合わせる
  - 完了の姿: 通しの実行で段の番号が 1〜7 で並び、まとめに `pasta_check.zip` の場所が出る
  - _Requirements: 1.4, 3.1, 3.5, 3.7, 3.8_

- [x] 2.3 手元のリリースビルドで通しの確かめをする
  - pwsh から `release.ps1` を直に通し、`release\` に 3 つの配布物ができることを確かめる
  - zip を展開し、`--version` がワークスペースの版を示し、`validate --help` が動くことを確かめる。表示の中身を 1.2 と同じ条件で確かめる
  - `pasta.dll.zip`・`.nar` の中身の名前の一覧と、ゴーストの `THIRD_PARTY_LICENSES.txt` のハッシュが、1.1 の記録と同じことを確かめる
  - `-SkipSetup` でも段 3 が走って zip が作り直されること、`Cargo.lock` が変わらないことを確かめる
  - 展開した exe に `dumpbin /dependents` を当て、`vcruntime*.dll`・`msvcp*.dll` に依存していないことを確かめる
  - cargo-about が無いときの失敗の道は、段 2 と同じ `Get-Command` の検査と文言であることをコードの読み合わせで確かめる（cargo-about は cargo と同じフォルダにあり、外すと cargo も消えるため実行では試さない）
  - `powershell.exe`（5.1）の構文解析を `release.ps1` に当ててエラーが無いこと、7 だけの書き方を使っていないことを確かめる
  - 完了の姿: 上の確かめがすべて通り、`release\pasta_check.zip` が 3.1 の確かめに使える形で残っている
  - _Requirements: 1.1, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.6, 3.1, 3.6, 3.7, 4.6_

- [x] 3. リリース CI の配線
- [x] 3.1 (P) リリース CI の build の job で pasta_check.zip を作り、検査して渡す
  - toolchain の targets に x64 を足し、rust-cache の key を `i686-x86_64-pc-windows-msvc` にする。リリースビルドの段の名前を合わせ、実行の行は変えない
  - 配布物の検査に、`pasta_check.zip` の中身がちょうど 2 つかの項目と、中身が合うときに exe の `--version` が `pasta_check <タグの版>` と完全に一致するかの項目を足す
  - 受け渡しの path に `release/pasta_check.zip` を足し、冒頭の説明を 4 つの配布物にする。verify・gate・公開系の job と `Cargo.lock` の検査の行は変えない
  - 検査の `run` をスクラッチの `.ps1` に写し、2.3 の `release\` で版が合う場合と違う場合の 2 通りを走らせる。手元に VSIX が無いので、VSIX の項目の NG の行は除いて読む
  - `actionlint` を当てる（手元に無ければ入れる）
  - 完了の姿: 版が合えば `pasta_check.zip` の項目は通り、違えば版の違いの行が出る。`actionlint` がエラーを出さない
  - _Boundary: ReleaseBuildJob_
  - _Depends: 2.3_
  - _Requirements: 1.5, 3.8, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_

- [x] 3.2 (P) GitHub Release の添付を 5 つにし、欠けのテストを足す
  - `github-release.ps1` の添付の一覧に `pasta_check.zip` を足し、説明と文言を「5 つ（配布物 4 つと詳しいリリースノート）」に合わせる。gh の前の有無の検査・下書き → 添付 → 公開・再実行の規則の流れは変えない
  - `release-scripts-test.ps1` に項目 10 を足す。`pasta_check.zip` だけが無いフォルダで呼び、終了コード 1 で出力にその名前が出ることを確かめる。冒頭の説明に `github-release.ps1` を足す
  - 完了の姿: `release-scripts-test.ps1` を走らせると項目 1〜10 がすべて ok になる
  - _Boundary: GitHubReleaseAssets, ReleaseScriptsTest_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

- [x] 4. (P) release スキルの状態の読み取りと手順書を 4 つの配布物に合わせる
  - `release-state.ps1` の配布物の一覧に `pasta_check.zip` を足し、数の表示を `n/4` に、公開済みの判定を 4 つそろった公開に変える
  - `procedure.md` の状態の表と読み方を `4/4`・`n/4` にし、0.4.1 までの版は `3/4` の添付不足と出ることを一言足す。`SKILL.md` の「添付 3/3」を「添付 4/4」に。公開先の「7/7」は変えない
  - `.claude/skills/**` の編集が分類器に拒否されたら、入った分と残りを示して開発者の許可を求める
  - 完了の姿: `release-state.ps1` を 0.4.1 に向けると、GitHub Release の行が `添付不足（公開・添付 3/4）` と出る
  - _Boundary: ReleaseStateReader, ReleaseSkillDocs_
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 5. 入手の手順と配布物の一覧の文書
- [x] 5.1 (P) 入手の手順を Release の zip が第一の道になるように書き直す
  - マニュアル `book/src/debug/validate.md` の入れ方の節を、固定のリンクから落として展開する道（64 ビット版・SmartScreen の一言・パスで呼ぶ例 1 つ）を第一に、`cargo install pasta_check` を Rust の利用者向けの別の道にする。ほかの節のコマンド例と PATH の話は変えない
  - `crates/pasta_check/README.md`（絶対 URL）と `pasta-check` スキルの入れ方を同じ順にそろえる。同じスキルの段の並び・出力の木・公開の説明にも `pasta_check.zip` を足す
  - `pasta-check/SKILL.md` は、入れ方も配布物の一覧の行もこのタスクが持つ（5.2 は触らない）
  - `.claude/skills/**` の編集が分類器に拒否されたら、入った分と残りを示して開発者の許可を求める
  - `gen-skill-refs.mjs --check` と `link-check.mjs` を走らせる。`--check` が落ちても `references/` の作り直しはせず、6 に任せる（5.2 と同じ木を同時に書かないため）
  - 完了の姿: 3 か所の入れ方が同じ順で、`link-check.mjs` が通る
  - _Boundary: InstallDocs, AssetListDocs（`pasta-check/SKILL.md` だけ）_
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 7.9_

- [x] 5.2 (P) 配布物の一覧と段の並びの記述を 4 つ（添付 5 つ）に合わせる
  - ルートの `README.md`・マニュアルの `introduction.md` の直リンクに `pasta_check.zip` を足す
  - `crates/pasta_sample_ghost/RELEASE.md`・`crates/pasta_sample_ghost/README.md` の配布物・build の役目・`release.ps1` の段の並びを合わせる
  - steering の `tech.md`・`structure.md` の配布物の数と添付の数を合わせる
  - `pasta-check/SKILL.md` は 5.1 が持つので触らない
  - `gen-skill-refs.mjs --check` と `link-check.mjs` を走らせる。`--check` が落ちても `references/` の作り直しはせず、6 に任せる
  - 完了の姿: 挙げた文書に「3 つの配布物」「添付 4 つ」「6 段」の古い記述が残らず、`link-check.mjs` が通る
  - _Boundary: AssetListDocs（`pasta-check/SKILL.md` を除く）_
  - _Requirements: 7.7, 7.8, 7.9_

- [x] 6. 全体の確かめと開発者への引き継ぎ
  - `gen-skill-refs.mjs --check` が落ちていれば `gen-skill-refs.mjs` でスキルの `references/` を作り直す
  - `release-scripts-test.ps1` の全項目、`gen-skill-refs.mjs --check`、`link-check.mjs` を通しで走らせる
  - 古い数（「3 つの配布物」「添付 4 つ」「/3」「[n/6]」）の取り残しを、触ったファイルと File Structure Plan の文書で検索する
  - 開発者に、`release.bat`（Windows PowerShell 5.1）での通しの実行と、0.4.2 のリリースでの本番の確かめ（Release に 5 つ・固定のリンク）を頼む旨を報告に書く
  - 完了の姿: 3 つの検査が通り、取り残しが無く、開発者に頼む 2 つの確かめが報告に書かれている
  - _Depends: 3.1, 3.2, 4, 5.1, 5.2_
  - _Requirements: 3.5, 5.6, 7.7, 7.8, 7.9_

## Implementation Notes

- 完了時の棚卸（2026-10-22）: その場で解決 0 件・申し送り 2 件（ponytail-manual-ci・ponytail-check-release の brief.md。release.yml／github-release.ps1 の変更と、release.ps1 の段 4 が x64 の exe で走ること）・起票 0 件。
