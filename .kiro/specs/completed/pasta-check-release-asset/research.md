# ギャップ分析: pasta-check-release-asset

調べた日: 2026-10-22（main dc46f446 の上のワークツリー）。対象は `requirements.md` の R1〜R7。

## 1. 分析のまとめ

- **作りの大半は今ある形の写しで済む**。`release.ps1` の段 4（`pasta.dll.zip` を作り、中身を `ZipFile.OpenRead` で検査する）と、`release.yml` の「Verify release assets」の `pasta.dll.zip` の検査が、そのまま `pasta_check.zip` の手本になる。新しい部品は要らない。
- **一番の論点はライセンス表示**。今の `about.hbs` は LuaJIT の表示を固定で書き、`about.toml` の `targets` に `wasm32-unknown-unknown` を含む。`cargo about generate -m crates/pasta_check/Cargo.toml` を手元で試すと、LuaJIT・`mlua` は出ないが、`wasm-bindgen` 系（wasm 向けの `getrandom` の依存）が混ざる。`--target x86_64-pc-windows-msvc` で絞る必要がある（R2.1・R2.2）。
- **添付の数は 3 か所で決め打ち**（`github-release.ps1` の `$names`・skip の文言、`release-state.ps1` の配列と「/3」、`release` スキルの `procedure.md` の表の読み方）。どれも 1 行〜数行の書き換えで済む。添付の数のテストは今は無いが、`github-release.ps1` は gh を呼ぶ前に添付の有無を検査するので、ネットワーク無しでテストを足せる（R5.6）。
- **文書の書き換えが量としては最も多い**（入手の手順 3 か所、配布物の一覧 7〜8 か所、`release.bat` の説明）。マニュアルの `debug/validate.md` はスキル `references/` の生成の対象に入っていないので、`gen-skill-refs.mjs --check` は確かめだけで、作り直す物は無い見込み。
- **推奨は Option A（今の部品を伸ばす）**。ライセンス表示のひな形だけは、分ける（Option B 寄り）か条件で出し分けるかを設計で決める。規模は S（1〜3 日）、リスクは低い。

## 2. 今の状態

### 2.1 配布物を作る所

| 所 | 今の中身 | 本仕様との関係 |
|---|---|---|
| `crates/pasta_sample_ghost/release.ps1`（263 行） | 段 1〜6。段 1 で `cargo build --release --target i686-pc-windows-msvc -p pasta_shiori`、段 2 で DLL を写し `cargo about generate --output-file … about.hbs`、段 3 で `cargo run -p pasta_check -- release …`、段 4 で `pasta.dll.zip` を作り中身を検査、段 5 で版の確認、段 6 で `.nar` の大きさの検査とまとめ | `pasta_check.zip` の段を足す。段 4 の zip と検査（L183〜204）がそのまま手本。`cargo-about` の有無の検査（L108〜112）は段 2（setup）の中にあるので、`-SkipSetup` のときは走らない。新しい段は自分で有無を検査する必要がある（R2.6・R3.6） |
| `release.bat` | `powershell.exe -ExecutionPolicy Bypass -File …release.ps1`。冒頭の説明に段の並びと作る物 | 説明を直す（R3.5）。**`powershell.exe`（5.1）で走る**ので、足すコードは 5.1 でも動く書き方にする（今の段 4 の `Compress-Archive`・`System.IO.Compression` は 5.1 で動く） |
| `.cargo/config.toml` | `x86_64-pc-windows-msvc` にも `+crt-static` | そのまま使える（R1.3）。手元の rustup に `x86_64-pc-windows-msvc` は入っている |
| `crates/pasta_check/Cargo.toml` | `[[bin]]` は `required-features = ["cli"]`、`default = ["cli"]` | 既定の機能でビルドすれば CLI の依存（`lexopt`・`md5`・`zip`）が入る（R1.4） |

### 2.2 ライセンス表示

- `about.toml`: `accepted` は `deny.toml` の `[licenses].allow` と同じ 10 種。`targets = ["i686-pc-windows-msvc", "x86_64-pc-windows-msvc", "wasm32-unknown-unknown"]`。`ignore-dev-dependencies`・`ignore-build-dependencies` は true。`seahorse` の clarify（pasta.dll 側の依存）を持つ。
- `about.hbs`: 冒頭の文が「pasta.dll / VSCode 拡張同梱の wasm」向け。LuaJIT の表示（MIT・Lua 5.1/5.2・dlmalloc）を固定で持つ。
- 使っている所は 2 つ: `release.ps1` 段 2（pasta.dll 用）と `editors/vscode/scripts/build-wasm.ps1` L118（VSIX の WASM 用）。どちらもワークスペース全体を対象に回している。
- **手元で試した結果**（cargo-about 0.9.2・`cargo about generate -m crates/pasta_check/Cargo.toml -c about.toml --format json`）
  - 成功した。ライセンスは MIT 50・Apache-2.0 1・Unicode-3.0 1・Zlib 1 で、すべて `accepted` に収まる（R2.5 は今の依存では起きない）。
  - `lexopt`・`md5`・`zip`・`zlib-rs` などが入り、`mlua`・LuaJIT は出ない（R2.2 は `-m` で満たせる）。
  - ただし `wasm-bindgen`・`wasm-bindgen-macro`・`wasm-bindgen-shared` などが入る。`about.toml` の `targets` が wasm32 を含むためで、`pasta_check.exe` には入らない。`--target x86_64-pc-windows-msvc` で絞る必要がある。
  - `[seahorse.clarify]` について「グラフに無いクレートの設定」の警告が出る（失敗ではない）。
  - `pasta_check`・`pasta_core`・`pasta_dsl`（pasta 自身）と、proc-macro（`pest_derive`・`serde_derive` など）も「Used by」に並ぶ。今の pasta.dll 用の表示と同じ扱いなので、本仕様では変えない。

### 2.3 リリース CI（`.github/workflows/release.yml`、574 行）

- build の job（L58〜200）
  - `dtolnay/rust-toolchain@stable` の `targets: i686-pc-windows-msvc`、`Swatinem/rust-cache@v2` の `key: i686-pc-windows-msvc`。x64 を足す（R4.1）。runner の windows-latest は x64 なので、ホストのターゲットとしては x64 の std は入っているが、明示しておくのが brief の Approach。
  - cargo-about 0.9.2 は版を固定して入れている（L89〜110）。追加の道具は要らない。
  - 段の名前「Build pasta.dll.zip and hello-pasta.nar」（L120）は中身に合わせて名前を直す。
  - 「Verify release assets」（L131〜183）は `Get-ZipEntries` と `$errors` に積む形。`pasta_check.zip` の検査は `pasta.dll.zip` の 3 行（L141〜145）と同じ形で足せる。exe の版の検査（R4.3）は、zip から取り出して `--version` を起動する数行が新しく要る（VSIX の `package.json` の版の検査 L168〜176 と同じ考え）。
  - 「Check Cargo.lock is unchanged」（L186〜187）はそのまま効く（R4.6）。
  - 「Upload release-assets」（L189〜200）の `path` に 1 行足す（R4.5）。
- 後続の job: publish-vsce は `release-assets/*.vsix` で拾うので影響なし。github-release は `-AssetDir release-assets` を渡すだけで、添付の名前は `github-release.ps1` が持つ。publish-crates は配布物を使わない。
- `ponytail-manual-ci` が触る行（公開系の job の action・Node の版）とは重ならない。

### 2.4 GitHub Release の添付（`.github/scripts/release/github-release.ps1`、132 行）

- L18 の説明、L71〜72 の `$names`（4 つ）、L88〜89 の skip の条件の文言（「4 つ（配布物 3 つと詳しいリリースノート）」）、L103 の注釈。`$names` に `pasta_check.zip` を足せば、検査・不足分の添付・公開の順序（R5.3〜R5.5）は今の作りのまま 5 つに広がる。
- 添付の有無の検査（L71〜79）は、`gh` を呼ぶ前（L82 の `Get-Release` より前）に行う。欠けていれば `Complete failed publish` で終了コード 1 と「添付がそろっていない（…に無い: 名前）」を出す。
- `.github/scripts/release/release-scripts-test.ps1`（247 行）は `Test-Item` を並べる形で、今は 9 項目（release-notes・verify-tag・定型コマンド D）。`github-release.ps1` の項目は無い。毎回の CI（`build.yml` L149）でも走る。**添付の欠けを試す項目 10 を足すのは、gh・ネットワーク無しでできる**（一時のフォルダに 4 つだけ置いて呼び、終了コード 1 と `pasta_check.zip` の名前を確かめる）。`$GITHUB_OUTPUT` などは既にテストの冒頭で外している。
- `-DryRun` は `gh release view` を呼ぶので、gh の認証が要る。手元の補助の確かめにしかならない。

### 2.5 release スキル

- `.claude/skills/release/scripts/release-state.ps1` L84〜87: 配布物 3 つの配列と「/3」の表示、公開済みの判定 `$n -eq 3`（R6.1・R6.2）。
- `.claude/skills/release/procedure.md` L471・L477: 状態の表の読み方「あり（公開・添付 3/3）」など（R6.3）。L396 の F5 の案内は「足りない配布物」と書いており、数を含まないので直さなくてよい見込み。
- 公開先の数「7/7」（`SKILL.md`・`procedure.md`）は GitHub Release を 1 つと数えるので変わらない（R6.4）。
- `.claude/skills/**` の編集は分類器が拒否することがある（brief の Constraints・メモリ）。拒否されたら入った分と残りを示して開発者に許可を求める。

### 2.6 文書

入手の手順（R7.1〜R7.6）:

| 所 | 今 |
|---|---|
| `book/src/debug/validate.md` L18〜24 | 「`pasta_check` は Rust のツールチェーンで入れる」＋`cargo install pasta_check` |
| `crates/pasta_check/README.md` L10〜13 | `## インストール` に `cargo install pasta_check` だけ（crates.io の README も兼ねる） |
| `.claude/skills/pasta-check/SKILL.md` L12〜18 | `cargo install` と `~/.cargo/bin` の前提 |

配布物の一覧・段の並び（R7.7）:

| 所 | 今 |
|---|---|
| `README.md` L41〜44 | 最新版の直リンクの箇条（nar・dll.zip） |
| `book/src/introduction.md` L38 | リリースページの行に配布物の直リンク |
| `crates/pasta_sample_ghost/RELEASE.md` L6・L42・L45・L107 | 添付する配布物・build の役目・github-release の「3 つ」・`release.ps1` の段の説明 |
| `crates/pasta_sample_ghost/README.md` L42・L74〜75 | `release.ps1` の説明と段の並び |
| `.claude/skills/pasta-check/SKILL.md` L107・L133・L153 | 段の並び・出力の木・公開の説明 |
| `.kiro/steering/tech.md` L211 | 成果物の説明（「3 つの配布物と詳しいリリースノートの 4 つ」） |
| `.kiro/steering/structure.md` L279・L290 | `release/` の出力と `release.ps1` の役目 |
| `release.bat` L3・L9 | 作る物と段の並び |

- `book/src/getting-started/prerequisites.md` は `hello-pasta.nar`・`pasta.dll.zip` の入手を書くが、`pasta_check` は扱わない。準備の章で `pasta_check` を勧めるかは本仕様の範囲外（要件にも入れていない）。
- `book/tools/gen-skill-refs.mjs` の対応表に `debug/validate.md` は無い（`pasta-check` スキルの `references/` は手書きの `nar-spec.md`・`updates-txt-spec.md`）。R7.8 は `--check` が通ることの確かめで足りる見込み。
- マニュアルを変えると `manual.yml` のリンクの検証（`link-check.mjs`）が走る。`link-check.mjs` が照合する絶対 URL は自分のリポジトリの `blob/main`・`tree/main` だけで、ほかの絶対 URL は飛ばす（L127〜130・L204）。固定のリンク（`releases/latest/download/pasta_check.zip`）は 0.4.2 の公開まで 404 だが、検査は落ちない（確かめ済み）。

## 3. 要件ごとの対応と足りない物

| 要件 | 今ある物 | 足りない物（ギャップ） |
|---|---|---|
| R1 zip と exe | x64 の `crt-static`、`cli` が既定 | x64 の exe をビルドして zip に入れる段（Missing） |
| R2 ライセンス表示 | cargo-about・`about.toml`・`about.hbs` | pasta_check だけを対象にする回し方（`-m`・`--target`）と、LuaJIT を出さない冒頭の文（Missing）。今の 2 つの表示を変えない分け方（Constraint） |
| R3 手元のリリースビルド | 段 4 の zip と検査の形 | 新しい段・段の番号・まとめの表示・cargo-about の有無の検査を新しい段でも（Missing）。5.1 互換（Constraint） |
| R4 リリース CI | 検査・upload・lock の検査の枠 | targets・cache の key・検査 3 行と exe の版の検査・upload の 1 行（Missing） |
| R5 添付 5 つ | 下書き → 添付 → 公開・冪等な再実行 | `$names` と文言（Missing）、添付の欠けのテスト（Missing） |
| R6 状態の読み取り | `release-state.ps1`・`procedure.md` | 配列と「/4」・読み方の表（Missing）。スキルの編集の拒否（Constraint） |
| R7 文書 | 上の表 | 書き換え（Missing）。SmartScreen・64 ビット・起動の仕方の記述（Missing） |

## 4. 実装の選択肢

### Option A: 今の部品を伸ばす（推奨）

- `release.ps1` に段を 1 つ足す（今の段 4 の写し）。`release.yml` の build の job、`github-release.ps1`、`release-state.ps1` は数行ずつ足す。
- ライセンス表示は、`about.hbs` を 1 つのまま使い、冒頭の文と LuaJIT の節を条件で出し分ける形にする案（A-1）。cargo-about のひな形（handlebars）が、渡せる変数で出し分けられるかは要確認（Research Needed）。cargo-about は外から任意の変数を渡す口を持たない見込みで、`overview` にあるクレート名で判定するなどの工夫が要る。
- ✅ ファイルが増えない。❌ ひな形の条件が読みにくくなり、pasta.dll 用の表示を変えてしまう危険がある（R2.4）。

### Option B: pasta_check 用のひな形を分ける

- `about-pasta_check.hbs`（仮の名前）を新しく置き、冒頭の文を pasta_check.exe 向けにして、LuaJIT の節を持たない。`about.toml` は共用し、`--target x86_64-pc-windows-msvc` を渡して wasm を外す。
- ✅ 今の `about.hbs` に触らないので R2.4 を確実に満たせる。条件の無い素直なひな形。❌ 共通の部分（`{{#each licenses}}` の節）が 2 か所になる（十数行）。
- `[seahorse.clarify]` の警告が出る（失敗ではない）。気になるなら pasta_check 用の `about.toml` も分けるが、`accepted` の一覧が 2 か所になり「`deny.toml` と揃える」所が増えるので勧めない。

### Option C: 段を別のスクリプトにする

- `pasta_check.zip` を作る処理を別の `.ps1` にして、`release.ps1` と CI の両方から呼ぶ。
- ✅ 単独で回せる。❌ 「手元と CI が同じ `release.ps1`」の今の形（R3.8）から外れ、呼ぶ所が 2 つになる。本仕様の規模では得が無い。

**見立て**: リリースビルドの段は Option A、ライセンス表示のひな形は Option B が素直（brief は「設計で決める」）。

## 5. 設計で調べること（Research Needed）

1. cargo-about の `--target` を 1 つ渡したとき、`about.toml` の `targets` を置き換えるか（ヘルプには「Overrides the `targets` configuration value」とある）。`-m crates/pasta_check/Cargo.toml` がワークスペースの中でも pasta_check だけを根にすることは、手元で確かめ済み。
2. 段 3 の `pasta_check release` を、新しく作る x64 の exe で走らせるか、今の `cargo run -p pasta_check` のままにするか（requirements の未決事項 6）。exe で走らせれば、配布する exe そのものが `.nar` を作れることを毎回確かめられる。段の順序（exe のビルドを段 3 より前に置く）に響く。
3. exe の版の検査（R4.3）を手元のリリースビルドでもするか。`--version` の出力の形（`pasta_check 0.4.2` など）を確かめる。
4. rust-cache の key を 2 つのターゲットでどう付けるか（`i686-pc-windows-msvc` のままにするか、両方を含む名前にするか）。key を変えると最初の 1 回はキャッシュが効かない。

## 6. 規模とリスク

- **規模: S（1〜3 日）**。今の形の写しと数行の書き換えが中心。文書の書き換えが量としては最も多い。brief の見立て（7〜8 タスク）と合う。
- **リスク: 低**。
  - 本番の確かめ（Release に実際に載る）は 0.4.2 のリリースまでできない。手元のリリースビルド・テスト・`-DryRun` で、届く所まで確かめる。
  - ライセンス表示を誤って今の pasta.dll 用・VSIX 用の表示まで変えると、利用者向けの配布物に影響する（Option B で避けられる）。
  - `.claude/skills/**` の編集が分類器に拒否されると、R6・R7 の一部が止まる（開発者の許可で進める）。
  - `ponytail-manual-ci` が後から同じファイルを合わせるので、触る行を build の job と添付の数に絞る。

## 7. 設計への申し送り

- 推奨: Option A（リリースビルド・CI・添付・状態の読み取り）＋ライセンス表示のひな形は Option B。
- 決めること: ひな形の分け方、`--target` での絞り方、段 3 で使う exe、exe の版の検査を手元でもするか、rust-cache の key、添付の欠けのテスト（項目 10）の形。
- 守ること: `pasta.dll.zip`・`.nar`・VSIX の中身と検査を変えない。`release.bat` の 5.1 で動く。`build.yml` と公開系の job の action の行に触らない。

---

# 設計の調査と決定（2026-10-22・`/kiro-spec-design -y`）

## 8. 設計の調査のまとめ

- **種別**: 既存の仕組みの拡張（Extension）。軽い discovery（`design-discovery-light.md`）で、つなぎ目と今の形を確かめた。
- **参照したスキルと規則**: `kiro-spec-design` の `design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`。ドメインのスキルは `pasta-check`（`release.ps1` の段と `release/` の木の書き方を確かめた）。新しいライブラリは入れないので、外部の調査（WebSearch）はしていない。
- **主な発見**
  - `cargo about generate` の `--target x86_64-pc-windows-msvc` は、`about.toml` の `targets` を置き換える。wasm 向けの `wasm-bindgen` 系が消え、MIT 45・Apache-2.0 1・Unicode-3.0 1・Zlib 1 になる。`lexopt`・`md5`・`zip`・`zlib-rs` は入り、`mlua`・LuaJIT は出ない（R2.1・R2.2）。
  - 許可に無いライセンスがあると、`cargo about generate` は終了コード 1 で、出力のファイルを作らない（R2.5 は終了コードを見るだけで満たせる）。
  - `pasta_check.exe --version` の出力は `pasta_check 0.4.1` の 1 行（`main.rs` L49 の `println!("pasta_check {}", env!("CARGO_PKG_VERSION"))`）。
  - x64 の release ビルドの exe（約 950 KB）は `VCRUNTIME140.dll` を取り込まない（`crt-static` が効く。R1.3）。ビルドで `Cargo.lock` は変わらない（R4.6）。
  - 今のリリース CI の build の job は、rust-cache の復元が「No cache found.」だった（v0.4.1 の実行 39123004799）。key を変えても失う物は無い。
  - `release/` は、今は段 3 の `pasta_check release` が作っている。新しい段を段 3 より前に置くなら、zip を書く前に `release/` を作る必要がある（CI はまっさらのチェックアウト）。

## 9. 調べた記録

### cargo-about で pasta_check だけを数える

- **きっかけ**: R2.1・R2.2（pasta_check.exe に入る物だけ、LuaJIT・wasm 向けを出さない）。
- **確かめたこと**: `cargo about generate -m crates/pasta_check/Cargo.toml -c about.toml --target x86_64-pc-windows-msvc --format json`（cargo-about 0.9.2）。`--help` に「Overrides the `targets` configuration value」とある。
- **結果**: 上の 8 節のとおり。`[seahorse.clarify]` の「グラフに無いクレートの設定」の警告は出るが、終了コードは 0。`-c about.toml` を省くと、設定の既定は `<manifest_root>/about.toml`（`crates/pasta_check/about.toml`、無い）になるので、明示して渡す。
- **設計への影響**: 設定は今の `about.toml` を共用し、`accepted` を 1 か所に保つ。ひな形だけを分ける。

### 許可に無いライセンスのときの振る舞い

- **確かめたこと**: `about.toml` から `"Zlib"` を除いた写しをスクラッチに作り、`-c` で渡して `generate` した。
- **結果**: `error: failed to satisfy license requirements`（`zlib-rs` の `license = "Zlib"`）、終了コード 1、出力のファイルは作られない。
- **設計への影響**: 新しい段は終了コードを見て止まるだけでよい（R2.5）。前の実行で残った `pasta_check.zip` を段の頭で消しておけば、「作らない」が手元の再実行でも厳密に成り立つ。

### exe の版と静的 CRT

- **確かめたこと**: スクラッチの `CARGO_TARGET_DIR` で `cargo build --release --target x86_64-pc-windows-msvc -p pasta_check` を回し、`--version` を起動し、exe の中の DLL 名を拾った。
- **結果**: `pasta_check 0.4.1`、終了コード 0。DLL 名は `KERNEL32.dll`・`ntdll.dll`・`bcryptprimitives.dll` などで、`vcruntime140` は無い。`git status` はきれいなまま。
- **設計への影響**: CI の版の検査は、出力が `pasta_check <版>` と完全に一致することで判定する（R4.3）。

### rust-cache の key

- **確かめたこと**: v0.4.1 のリリース CI の build の job の「Setup Rust cache」のログ。
- **結果**: key は `v0-rust-i686-pc-windows-msvc-build-Windows_NT-x64-d107b311-…`。環境変数のハッシュ（`CARGO_ABOUT_VERSION` など `CARGO_` で始まる env を含む）が gate の build-x86 と違い、「No cache found.」だった。
- **設計への影響**: key を変えても今より遅くはならない。key の名前を、入れるターゲットの 2 つに合わせる。

### PowerShell 5.1 で動く書き方

- **きっかけ**: `release.bat` は `powershell.exe`（5.1）で `release.ps1` を走らせる。
- **結果**: 足すコードは、今の段 4 と同じ `Compress-Archive`・`[System.IO.Compression.ZipFile]::OpenRead`・`& <exe>`・`$LASTEXITCODE` だけで書ける。`??`・三項演算子・`&&`・`-Parallel` は使わない。
- **設計への影響**: エージェントは `-ExecutionPolicy Bypass` で 5.1 を走らせられない（分類器が拒否）。5.1 の確かめは、`powershell.exe` で `Parser::ParseFile` の構文の検査をするところまでにし、通しの実行は pwsh で行う。

## 10. 設計の合成（design-synthesis）

- **一般化**: 「zip を作って中身がちょうど N 個かを検査する」は段 4 と新しい段で同じ形になる。関数に括り出すと段 4 の行も変わり R3.7（今の配布物の検査を変えない）の確かめが増えるので、括り出さずに同じ形を写す。2 か所なら写しの方が読みやすい。
- **作るか使うか**: ライセンス表示は cargo-about（版固定・CI で導入済み）、zip は PowerShell の標準、添付は今の `github-release.ps1` をそのまま使う。新しい道具・依存は入れない。
- **簡素化**
  - 別のスクリプトに切り出す案（ギャップ分析の Option C）は採らない。手元と CI が同じ `release.ps1` を呼ぶ今の形（R3.8）のままにする。
  - pasta_check 用の `about.toml` は作らない（`accepted` を 2 か所にしない）。ひな形 1 つだけ増やす。
  - ライセンス表示の置き場は新しいフォルダを作らず、exe と同じ `target\x86_64-pc-windows-msvc\release\` に書く。
  - 段 3 の `cargo run -p pasta_check`（ホスト向けの debug ビルド）を、新しく作る x64 の exe の起動に置き換える。ビルドが 1 回減り、配布する exe そのものが `.nar` を作れることを毎回確かめられる。

## 11. 設計の決定

### 決定: ライセンス表示のひな形を分ける

- **きっかけ**: R2.2・R2.3・R2.4。今の `about.hbs` は LuaJIT の表示を固定で持ち、冒頭の文が pasta.dll・wasm 向け。
- **候補**
  1. `about.hbs` を 1 つのまま条件で出し分ける — cargo-about には外から変数を渡す口が無く、`overview` のクレート名で判定するなどの工夫が要る。pasta.dll 用の表示を変える危険がある。
  2. pasta_check 用のひな形 `about-pasta_check.hbs` を新しく置く。
- **採った案**: 2。設定（`about.toml`）は共用し、`--target x86_64-pc-windows-msvc` と `-m crates/pasta_check/Cargo.toml` で数える範囲を絞る。
- **理由**: 今の `about.hbs` と `about.toml` の中身に触らないので、R2.4 を確実に満たせる。条件の無い素直なひな形になる。
- **引き換え**: `{{#each licenses}}` の節（十数行）が 2 か所になる。
- **後で確かめること**: 生成した表示に LuaJIT・`mlua`・`wasm-bindgen` が無く、`lexopt`・`md5`・`zip` があること。

### 決定: 新しい段を `pasta_check release` の前に置き、その exe で `.nar` を作る

- **きっかけ**: 申し送り 2。R3.6（`-SkipSetup` でも飛ばさない）・R3.7。
- **候補**
  1. 新しい段を段 4（`pasta.dll.zip`）の後に足し、段 3 は `cargo run -p pasta_check` のまま。
  2. 新しい段を段 3 に置き、次の段（`pasta_check release`）を、その段が作った x64 の exe で走らせる。
- **採った案**: 2（2026-10-22 の設計ディスカッションで確定）。
- **理由**: 配布する exe そのものが検査と `.nar` の作成をこなせることを、手元と CI のリリースビルドのたびに確かめられる（R1.4 の裏付け）。ホスト向けの debug ビルドが 1 回減る。`pasta_check` のソースは同じなので、`.nar` と `pasta.dll.zip` の中身と検査は変わらない（R3.7）。
- **引き換え**: 段の順序が変わる（どちらの案でも段の数は 7 になり、見出しの番号は全部直す）。x64 の exe を起動できない手元（ARM64 の Windows で x64 のエミュレーションが無い場合など）では段 4 が動かない。
- **後で確かめること**: 手元のリリースビルドで、`.nar` の中身が変わる前と同じであること（CI の「Verify release assets」の nar の項目と同じ物が在る）。

### 決定: exe の版の検査はリリース CI だけで行う

- **きっかけ**: 申し送り 3。R4.3。
- **採った案**: 手元のリリースビルド（`release.ps1`）には版の検査を足さない。リリース CI の「Verify release assets」で、zip から取り出した exe の `--version` がタグの版と一致することを検査する（2026-10-22 の設計ディスカッションで確定）。
- **理由**: 手元では、その場で `cargo build` した exe を使うので、古い exe の取り違えは起きにくい。`release.ps1` は CI でも走るので、足すと CI で同じ検査が 2 回になる。

### 決定: rust-cache の key を 2 つのターゲットの名前にする

- **きっかけ**: 申し送り 4。
- **採った案**: `key: i686-x86_64-pc-windows-msvc`（2026-10-22 の設計ディスカッションで確定）。
- **理由**: 今の build の job は復元できていない（9 節）ので、変えても遅くならない。名前が中身（2 つのターゲット）と合う。

### 決定: 添付の欠けのテストは「無い名前を示して失敗する」だけを試す

- **きっかけ**: R5.6。
- **採った案**: `release-scripts-test.ps1` に項目 10 を足す。一時のフォルダに、5 つのうち `pasta_check.zip` を除く 4 つを置いて `github-release.ps1` を呼び、終了コード 1 と、出力に `pasta_check.zip` があることを確かめる。
- **理由**: `github-release.ps1` は gh を呼ぶ前に添付の有無を検査するので、ネットワークと gh 無しで試せる。5 つそろったときの先（gh の呼び出し）は試さない（`-DryRun` で手元の補助として確かめる）。

## 12. 設計で見つけたリスク

- **5.1 の通しの実行ができない**: エージェントは `release.bat`（`-ExecutionPolicy Bypass`）を走らせられない。構文の検査（5.1 の Parser）と、7 だけの書き方を使わないことの見直しで補う。開発者が `release.bat` をダブルクリックで確かめるのが最後の確かめになる。
- **過去の版の状態の読み取り**: `release-state.ps1` を 0.4.1 までの版に向けると、`pasta_check.zip` が無いので「添付不足（公開・添付 3/4）」と出る。過去の版へ足すのは範囲外（要件の Out of scope）なので、手順書の読み方の節に一言書く。
- **`[seahorse.clarify]` の警告**: pasta_check の表示を作るとき、毎回警告が 1 つ出る。失敗ではない。消すために設定を分けると `accepted` が 2 か所になるので、受け入れる。
- **`ponytail-manual-ci` との重なり**: `release.yml` で触るのは、冒頭の説明 1 行・build の job（toolchain・cache・段の名前・検査・upload）だけ。公開系の job の action の行と Node の版の行には触らない。
