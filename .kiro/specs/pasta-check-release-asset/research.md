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
