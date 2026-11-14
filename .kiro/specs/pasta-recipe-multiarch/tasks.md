# Implementation Plan

- [x] 1. 土台: arm64 の設定と道具の取得
- [x] 1.1 aarch64 の静的 CRT と、依存の表示・検査の targets を足す
  - ルートの `.cargo/config.toml` に `[target.aarch64-pc-windows-msvc]` の `+crt-static` の段を足し、頭の説明を出荷の target に pasta-recipe の arm64 を含む形へ直す。`pasta-dll-multiarch` が先に入れていれば、在るのを確かめるだけにして足し直さない
  - `editors/recipe/src-tauri/about.toml` の `targets` を x64 と aarch64 の 2 つにし、`package.ps1` が arch ごとに `--target` で置き換えることを説明の行に書く
  - `editors/recipe/src-tauri/deny.toml` の `[graph] targets` に `aarch64-pc-windows-msvc` を足す
  - 完了の状態: 3 つのファイルに aarch64 の行が在り、手元の `cargo deny check`（pasta-recipe）が通る。手元に cargo-deny が無ければ、無かったことを記録して CI の job `deny` に委ねる
  - _Requirements: 1.6, 1.3, 6.6_

- [x] 1.2 版と SHA-256 を固定した道具の取得をスクリプトに出す
  - `release.yml` の build の job の `Install wasm-pack and cargo-about` の loop と、`env` の 4 つの固定値（wasm-pack・cargo-about の版と SHA-256）を、そのまま新しい `install-tools.ps1` に移す。`-Name` は `wasm-pack`・`cargo-about` の `ValidateSet`
  - 取得・SHA-256 の照合・`--version` の照合・`GITHUB_PATH` への追加を今と同じに保ち、どれかの失敗で終了コード 1
  - この task では `release.yml` を書き換えない（呼び出しへの置き換えは 5.1）
  - 完了の状態: 手元で `-Name cargo-about` を走らせると cargo-about 0.9.2 が取れ、版の照合を通って終了コード 0。`-Name x` は終了コード 1。このとき cargo-about 0.9.2 で `--target` が `about.toml` の `targets` を置き換えるかも確かめて記録する。置き換えないと分かったら、2.1 に進まず開発者に戻す（設計が手元で確かめたのは 0.9.3）
  - _Requirements: 7.3_

- [x] 2. arch ごとに 2 つの zip を作る
- [x] 2.1 配布物の作成スクリプトに arch の切り替えを入れる
  - `-Arch` を `ValidateSet('x64','arm64')`・既定は x64・arm64 の順で両方にし、arch と triple の表を 2 行持つ
  - arch ごとに、その arch の zip を消す → `--target <triple>` 付きの Tauri のビルド（x64 にも付ける）→ `target/<triple>/release/` の exe の在る確かめ → `cargo about --target <triple>` → 表示をつなぐ → `release/pasta-recipe.<arch>.zip` → 中身の並びの自己検査
  - `npm ci` は最初の 1 回。どの段の失敗でもその arch の zip を消して `ERROR:` の行と終了コード 1 で止まり、残りの arch は作らず、先に作り終えた arch の zip は残す
  - 完了の状態: 手元で引数なしの実行が `release/pasta-recipe.x64.zip`・`release/pasta-recipe.arm64.zip` を作り、`-Arch x86` が受け付ける値を示して終了コード 1、`-Arch arm64` だけの実行が x64 の zip を消さない。x64 の表示に `cpufeatures` の節が在り、arm64 の表示に無い（`windows_aarch64_msvc` はどちらの arch の通常の依存にも届かないため目印にしない。1.2 の確認）
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8_

- [x] 3. 2 つの zip の検査
- [x] 3.1 2 つの zip を同じ項目で検査するスクリプトを作る
  - 新しい `verify-recipe-zips.ps1` が `-Version`（必須・`X.Y.Z`）・`-Dir`（既定はリポジトリの根の `release`）・`-DumpbinCommand`（既定は `vswhere` で見つけた `dumpbin`）を受ける
  - zip ごとに、在る → 中身 2 つ → exe の版 → PE の Machine（x64 = `0x8664`・arm64 = `0xAA64`）→ 表示の `tauri`・`monaco-editor` → 31,457,280 バイト以下 → VC++ の再頒布可能パッケージの DLL（`vcruntime*`・`msvcp*`・`vccorlib*`・`concrt*`・`msvcr<数字>*`。UCRT は数えない）を見る。無い・中身違いはその zip の残りの項目を見ない
  - 問題を 1 つのリストに集めて `::error::` の行で全部出してから終了コード 1。`dumpbin` を動かせないのも、`vswhere` が無い（Windows でない）のも、例外で落とさず問題に数える。展開は一時のフォルダで行い終わりに消す
  - 完了の状態: 2.1 で手元に作った 2 つの zip に対して、リポジトリの版を渡すと「成功（版 V）」と終了コード 0、違う版を渡すと両方の zip の版の問題を出して終了コード 1
  - _Boundary: Verify_
  - _Depends: 2.1_
  - _Requirements: 1.2, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.11, 7.1_

- [x] 3.2 検査のスクリプトの失敗の形を自己テストで確かめる
  - `release-scripts-test.ps1` に、偽の exe（`MZ`・`e_lfanew`・`PE\0\0`・Machine の 2 バイトだけ）と偽の表示で zip を作る項目を足す。ネットワーク・gh・Windows の道具を使わない
  - 項目 15: arm64 の zip の exe の Machine が `0x8664` → 終了コード 1・出力に `pasta-recipe.arm64.zip`・`0xAA64`・`0x8664`
  - 項目 16: x64 の zip が無く、arm64 の表示に `monaco-editor` が無い → 終了コード 1・両方の文が出る
  - 項目 17: 偽の `-DumpbinCommand`（`.ps1`）が `VCRUNTIME140.dll` を返す → 終了コード 1・出力に zip の名前と `VCRUNTIME140.dll`
  - 項目 14 は 4.1 が足す「arm64 の zip だけが無い」のために空けておく（番号は飛んでよい）
  - 版の違い（2.3）と 30 MB（2.6）の失敗の経路は入れない（設計議題 4）。頭の説明に検査のスクリプトを足す
  - 完了の状態: 手元の pwsh で `release-scripts-test.ps1` が全部の項目で成功する
  - _Requirements: 2.2, 2.4, 2.5, 2.7, 2.11_

- [x] 4. 添付とリリースの状態を 8 つにする
- [x] 4.1 GitHub Release の添付を 8 つにし、欠けの自己テストを合わせる
  - `github-release.ps1` の名前の一覧を `pasta.dll.zip`・`hello-pasta.nar`・`pasta-vscode-<V>.vsix`・`pasta_check.zip`・`pasta-skills.zip`・`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip`・`release-notes-<tag>.md` の 8 つにし、説明と飛ばした理由の文を「8 つ（配布物 7 つと詳しいリリースノート）」にする
  - `release-scripts-test.ps1` の項目 10・11 の置くファイルを 2 つの zip にし「添付 8 つのうち」に、項目 13 を「`pasta-recipe.x64.zip` だけが無い」に、項目 14「`pasta-recipe.arm64.zip` だけが無い」を足す（無い方の名前だけが出ることを見る）
  - 3.2 と同じ自己テストのファイルを書き換えるので、3.2 の後に行う
  - 完了の状態: 手元の pwsh で `release-scripts-test.ps1` が項目 10〜17 を含めて全部成功する
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [x] 4.2 (P) release スキルの状態の集計と手順書を 8 つで数える
  - `release-state.ps1` の名前の一覧を 4.1 の 8 つに、添付の `n/7`・`7` を `n/8`・`8` にする（公開先の数の `$found/7` は 7 のまま）
  - `procedure.md` の過去の版の読み替え（0.4.2〜0.5.2 は `6/8`、0.4.1 以下は `4/8` を完了と読む）、`GitHub Release`・`集計` の行、「公開済み」の数え方の文を、8 つ（2 つの zip は 0.6.0 から）に合わせる（状態の表の `7/7`・`6/7 以下`・`0/7` と `集計` の `<n>/7` は公開先の数〔5 クレート＋Marketplace＋GitHub Release〕なので 7 のまま。2026-11-14 実装 4.2 で確認）
  - 分類器が `.claude/skills/` の書き換えを拒んだら、回避策を探さず、入った分と残りを示して開発者の許可を求める
  - 完了の状態: 手元で `release-state.ps1` を v0.5.2 に走らせると `公開・添付 6/8` と出て、`procedure.md` に添付の `n/7` が残らない
  - _Boundary: ReleaseSkill_
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 5. CI に組み込む
- [x] 5.1 リリース CI が 2 つの zip を作り、検査して渡す
  - build の job の rust の targets に `aarch64-pc-windows-msvc` を、cache の key に aarch64 を足す（`pasta-dll-multiarch` が先に入れていれば在るのを確かめる）
  - 道具の取得の loop と `env` の固定値を `install-tools.ps1 -Name wasm-pack,cargo-about` の呼び出しに置き換える
  - `Build pasta-recipe zips` で `package.ps1` を引数なしで呼び、`Verify release assets` の pasta-recipe の節を `verify-recipe-zips.ps1 -Version $env:VERSION -Dir release` の呼び出しにする。直書きの検査のエラーとスクリプトの終了コードの両方が出てから失敗する
  - upload の一覧を 2 つの zip にし（`if-no-files-found: error` は保つ）、`Check Cargo.lock is unchanged` は 2 つのビルドの後のまま、頭の説明を「7 つの配布物」にし、固定の版の値が `env` にあることを述べた 21 行あたりの説明を `install-tools.ps1` に持つ形へ直す
  - 完了の状態: `release.yml` に `pasta-recipe.zip` が無く、YAML として読め（`actionlint` が手元にあれば通り、無ければ pwsh の YAML の読み込みで確かめる）、gate・公開の job の `needs` が変わっていない
  - _Depends: 1.2, 2.1, 3.1_
  - _Requirements: 1.1, 1.7, 2.1, 2.7, 2.8, 2.9, 2.10, 7.3_

- [x] 5.2 main の CI の job `recipe` が 2 つの zip を作り、検査して残す
  - targets を `x86_64-pc-windows-msvc,aarch64-pc-windows-msvc` にし、型の検査・フロントのテスト・`npm run build`・`cargo test --locked`・clippy は今のまま保つ
  - 最後の `npx tauri build --no-bundle` を、`install-tools.ps1 -Name cargo-about` → `package.ps1`（引数なし）→ `verify-recipe-zips.ps1 -Version <editors/vscode/package.json の version>` → `actions/upload-artifact@v7`（`pasta-recipe-zips`・2 つの zip・`if-no-files-found: error`）に置き換える
  - `build.yml` に公開の job も Release を作る段も足さない
  - 完了の状態: job `recipe` の段がこの順に並び、YAML として読める
  - _Depends: 1.2, 2.1, 3.1_
  - _Requirements: 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4_

- [x] 5.3 main の CI に arm64 のランナーの 2 つの job を足す
  - `recipe-arm64-test`（`windows-11-arm`・`needs` なし）: checkout → rust-toolchain（host の aarch64）→ rust-cache（`editors/recipe/src-tauri`）→ setup-node 22 → `npm ci` → `npm run build` → `cargo test --locked --manifest-path src-tauri/Cargo.toml`
  - `recipe-arm64-smoke`（`windows-11-arm`・`needs: recipe`・checkout なし）: `pasta-recipe-zips` を落として arm64 の zip を展開し、exe を起動して 30 秒まで 1 秒ごとに見る。exe が終わった → 失敗（終了コード）、`ParentProcessId` が exe の `msedgewebview2.exe` が在る → 成功、30 秒で無い → 失敗。最後に exe を止める
  - 5.2 と同じ `build.yml` を書き換えるので 5.2 の後に行う
  - 完了の状態: `build.yml` に 2 つの job が在り YAML として読める。煙試験の判定の数行は、手元の x64 の exe で同じ判定が成功を返すことを確かめる
  - _Requirements: 6.4, 6.5_

- [x] 6. 名前の付け替えと利用者向けの文書
- [x] 6.1 (P) マニュアルと README で arch ごとの zip を選べるようにする
  - `book/src/debug/recipe.md` に 2 つの固定のリンク、arch の調べ方（設定 → システム → バージョン情報 → システムの種類が「x64 ベース プロセッサ」なら x64 版、「ARM ベース プロセッサ」なら arm64 版）、対象は x64 と arm64 の Windows で 32 ビット・mac・Linux 向けは無いこと、を書き、293 行あたりの名前も直す
  - `book/src/introduction.md`・`README.md` の固定のリンクを 2 つの zip にする
  - 完了の状態: 3 つのファイルに `pasta-recipe.zip` が無く、2 つの固定のリンクが在る。手元で mdbook のビルドが通る（`book/node_modules` が無ければ在るかを先に確かめ、無い自己テストの失敗は spec のせいとしない）
  - _Boundary: Docs-User_
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [x] 6.2 (P) 配布物の名前を使う所と steering を 2 つの名前にそろえる
  - `crates/pasta_sample_ghost/RELEASE.md`・`.claude/skills/pasta-check/SKILL.md` の配布物の一覧の名前を 2 つの zip にする（`.claude/skills/` が拒まれたら 4.2 と同じく開発者に戻す）
  - `.kiro/steering/tech.md`・`structure.md` に、配布物 7・添付 8、2 つの zip の中身と検査の項目（機械の種類・VC++ の DLL）、job `recipe` と arm64 の 2 つの job の形、`release/` の中身を書く
  - `.kiro/steering/roadmap.md` の配布物の数と名前を直す（この spec の行は名前を変えた説明として残す）
  - 完了の状態: これらのファイルに、名前を変えた説明の行のほか `pasta-recipe.zip` が無い
  - _Boundary: Docs-Steering_
  - _Requirements: 1.7, 8.5, 8.7_

- [x] 7. タグを打たない確かめと完了の証拠
- [x] 7.1 手元で作成・検査・自己テストを通しで確かめ、古い名前の残りを調べる
  - 手元で `package.ps1`（引数なし）→ `verify-recipe-zips.ps1 -Version <リポジトリの版>` → `release-scripts-test.ps1` を走らせ、全部成功する
  - `completed/`・各 spec の `brief.md`・この spec の文書・`roadmap.md` のこの spec の行を除いて `pasta-recipe.zip` を grep し、0 件になる
  - 完了の状態: 3 つのコマンドの成功と grep の 0 件を記録に残す
  - _Depends: 4.1, 4.2, 5.1, 5.3, 6.1, 6.2_
  - _Requirements: 3.8, 7.3, 8.6_

- [x] 7.2 作業のブランチで main の CI を手動で起動し、x64 と arm64 の確かめを通す
  - 作業のブランチを remote に置き（main には push しない）、`gh workflow run build.yml --ref <branch>` で起動する
  - job `recipe`（2 つの zip の作成・検査・artifact）、`recipe-arm64-test`、`recipe-arm64-smoke`、`release-scripts`、`deny` が成功することを見る。クロスのビルドや煙試験が CI で落ちたら原因を直して再実行する
  - `windows-11-arm` で exe が起動できない（WebView2 が無い・対話の画面が要る）と分かったら、煙試験を黙って外さず、分かったことを示して開発者に戻す
  - 完了の状態: すべての job が成功した実行の URL と artifact `pasta-recipe-zips` の在ることを、タスクの記録に書く
  - _Depends: 4.1, 5.1, 5.2, 5.3, 7.1_
  - _Requirements: 1.1, 1.5, 6.1, 6.3, 6.4, 6.5, 6.6, 7.1, 7.2, 7.4, 7.5_

## Implementation Notes
- 1.2: cargo-about 0.9.2 の `generate --target <triple>` は `about.toml` の `targets` を置き換える（`--help` の "Overrides the `targets` configuration value"。2 つの triple で表示が違う: x64 は MIT 226 件で `cpufeatures 0.3.1` の節が在り、aarch64 は 225 件で無い）。`windows_aarch64_msvc` は lock に在るが `jni`（Android だけ）経由でしか届かず、どちらの arch の表示にも出ない。arch の目印は `cpufeatures`。
- 1.2: `install-tools.ps1 -Name a,b` は `pwsh -File` で 1 つの文字列になるので、`ValidateSet` でなく `,` で分けて照合する。`RUNNER_TEMP` が無ければ OS の一時フォルダ、`GITHUB_PATH` が無ければ足さずに置き場を出す。
- 2.1: `package.ps1 -Arch` も `,` で分けて照合（`if ($bad.Count)`。`@('')` は偽になるので空の要素を通さない。install-tools.ps1 も同じ）。手元の zip: x64 4,411,365 バイト・arm64 4,298,602 バイト。
- 3.2: 自己テストの偽の dumpbin は `.ps1` を `&` で呼ぶ（shebang・実行ビット不要）。偽の exe の版は 0.0.0 と読まれ版の問題が一緒に出る。Linux の pwsh では未確認（7.2 の CI の job `release-scripts` で確かめる）。
- 4.2: release スキルの `/7` には 2 種ある。添付の数（今は 8）と、`集計`・状態の表の公開先の数（5 クレート＋Marketplace＋GitHub Release＝7）。後者は 7 のまま。
- 7.1（2026-11-14 手元・cargo-about 0.9.2）: 引数なしの `package.ps1` → exit 0（x64 4,411,364 バイト・arm64 4,298,600 バイト）、`verify-recipe-zips.ps1 -Version 0.5.2` → exit 0、`release-scripts-test.ps1` → exit 0、Cargo.lock 不変。`pasta-recipe.zip` の grep は除外の外で roadmap.md の名前を変えた説明の 2 行（343・385 行）だけ。
- 7.2（2026-11-14）: `gh workflow run build.yml --ref claude/pasta-recipe-multiarch-cf4beb` → https://github.com/ekicyou/pasta/actions/runs/43566925714 全 job 成功（`pasta-recipe`・`pasta-recipe (arm64 test)`・`pasta-recipe (arm64 smoke)`・`release-scripts`・`cargo-deny` を含む）。artifact `pasta-recipe-zips`（8,699,595 バイト）在り。windows-11-arm の煙試験は「OK: msedgewebview2.exe started as a child of pasta-recipe.exe after 1 s」。
