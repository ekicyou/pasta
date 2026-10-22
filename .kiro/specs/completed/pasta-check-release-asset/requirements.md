# Requirements Document

## Project Description (Input)
**誰が困っているか**: Rust を入れていないゴーストの作者。

**現状**: `pasta_check` を手に入れる道は `cargo install pasta_check` しかない。辞書の検査（`pasta_check validate`）とゴーストのリリースビルド（`pasta_check release`）を使うには、Rust のツールチェーンを入れて、ソースからビルドしなければならない。`pasta.dll` は GitHub Release から落とせるのに、同じ作者向けの道具の `pasta_check` だけが落とせない。リリース CI が作る配布物は `pasta.dll.zip`・`hello-pasta.nar`・`pasta-vscode-X.Y.Z.vsix` の 3 つである。

**何を変えるか**: GitHub Release に `pasta_check.zip`（x64 の `pasta_check.exe` と、pasta_check の依存だけのライセンス表示 `THIRD_PARTY_LICENSES.txt` の 2 つだけ）を 4 つ目の配布物として載せ、`https://github.com/ekicyou/pasta/releases/latest/download/pasta_check.zip` で最新版を落とせるようにする。手元の `release.bat` とリリース CI が同じ 4 つの一式を作り、リリース CI は中身を検査して、足りなければ公開しない。マニュアル・README・スキルの入手の手順は「Release から `pasta_check.zip` を落として展開する」を第一の道にし、`cargo install pasta_check` は Rust の利用者向けの別の道として残す。詳細は `brief.md` を参照。

## Introduction

本仕様は、`pasta_check` を Rust なしで使えるように、実行ファイルを GitHub Release の配布物に加える。

今、ゴーストの作者が `pasta_check` を使うには、Rust のツールチェーンを入れて `cargo install pasta_check` を実行するしかない。本仕様の後は、Release のページ（または固定のリンク）から `pasta_check.zip` を落として展開すれば、すぐに `pasta_check.exe` を使える。

使い方の例（本仕様の後）:

```text
1. https://github.com/ekicyou/pasta/releases/latest/download/pasta_check.zip を落とす
2. 好きなフォルダに展開する（pasta_check.exe と THIRD_PARTY_LICENSES.txt が出てくる）
3. pasta_check.exe validate --target <ゴーストの開発フォルダ>
```

配布物を作るのは、今の作りに 1 段足した手順である。手元の `release.bat`（中で `release.ps1` を呼ぶ）とリリース CI の build は同じ `release.ps1` を使い、`pasta.dll.zip`・`hello-pasta.nar`・`pasta_check.zip` の 3 つを `release/` に作る。VSIX はこれまでどおり VSCode 拡張のパッケージの手順（`npm run package`）が作る。手元で両方の手順を回せば、リリース CI と同じ 4 つの配布物がそろう。

用語:
- **配布物**: GitHub Release に添付する、利用者向けのファイル。本仕様の後は `pasta.dll.zip`・`hello-pasta.nar`・`pasta-vscode-X.Y.Z.vsix`・`pasta_check.zip` の 4 つ。
- **添付**: GitHub Release に付けるファイル。配布物 4 つと、詳しいリリースノート `release-notes-vX.Y.Z.md` の、合わせて 5 つ。
- **リリースビルド**: 配布物を作る手順。手元では `release.bat`（`release.ps1`）で、リリース CI では build の job で動く。同じ手順である。
- **リリース CI**: リリースタグ `vX.Y.Z` の push で動く GitHub Actions のワークフロー（完了 spec `release-ci`）。
- **ライセンス表示**: 配布物に同梱する `THIRD_PARTY_LICENSES.txt`。組み込んだ第三者のソフトウェアの著作権表示と許諾文を並べたもの。
- **リリースの状態の読み取り**: `release` スキルが、ある版が各公開先に出ているかを調べて表にする手順（定型コマンド A）。

## Boundary Context

- **In scope**:
  - `pasta_check.zip` を作る段（exe のビルド・ライセンス表示の生成・zip・中身の検査）を、リリースビルドに足す。
  - pasta_check 用のライセンス表示の生成。今の `pasta.dll` 用・VSIX 用の表示は変えない。
  - リリース CI の build の job で `pasta_check.zip` を作り、検査し、後続の job へ渡す。
  - GitHub Release の添付を 5 つにする。添付の数の確かめ方を整える。
  - `release` スキルのリリースの状態の読み取りと、その手順書の配布物の一覧。
  - 入手の手順と配布物の一覧を書いた文書（マニュアル・README・スキル・steering・手順書）。マニュアルの入手の手順には、署名の無い exe なので SmartScreen の警告が出ることがある、と書く。
- **Out of scope**:
  - コード署名（証明書が要る）。
  - x86（i686）版の exe（開発者の指示で x64 だけ）。
  - インストーラー・自動更新・PATH への登録。
  - Windows 以外のビルド。
  - `pasta_check` の機能の変更（Rust のソースは触らない。`release.rs`・`copy.rs`・`nar.rs` などは `ponytail-check-release` が持つ）。
  - 毎回の CI（`build.yml`）の構成。x64 の組み合わせが、すでに `pasta_check` をビルドしてテストしている。
  - リリース CI の公開系の job（publish-crates・publish-vsce・github-release・report）の action の版の固定と Node の版（`ponytail-manual-ci` が持つ）。
  - 過去の版（0.4.1 まで）の Release へ `pasta_check.zip` を足すこと。
- **Adjacent expectations**:
  - リリース CI の verify・gate の job と、公開系の job の流れは今のまま使う。本仕様が変えるのは build の job と、GitHub Release の添付の数である。
  - `ponytail-manual-ci` は、本仕様の後に `release.yml`・`github-release.ps1` を合わせる。本仕様は、その spec が触る行（公開系の job の action・Node の版）を変えない。
  - ルートの `.cargo/config.toml` の `crt-static` の設定（x64 にも付いている）をそのまま使う。
  - 本番の確かめ（GitHub Release に `pasta_check.zip` が実際に載ること）は 0.4.2 のリリースで行う。本仕様の中では、配布物は手元のリリースビルドで、添付の数はスクリプトのテストと `-DryRun` で確かめる。

## Requirements

### Requirement 1: pasta_check.zip の中身と exe

**Objective:** Rust を入れていないゴーストの作者として、落として展開するだけで `pasta_check` を使いたい。そうすれば、Rust のツールチェーンを入れずに辞書の検査とリリースビルドができる。

#### Acceptance Criteria

1. The リリースビルド shall `pasta_check.zip` に、`pasta_check.exe` と `THIRD_PARTY_LICENSES.txt` の 2 つだけを、フォルダに入れずに zip の最上位に置く。
2. The リリースビルド shall `pasta_check.exe` を 64 ビット版の Windows（x64）向けに作る。
3. The リリースビルド shall `pasta_check.exe` を、VC++ の再頒布可能パッケージ（ランタイムの DLL）を入れていない Windows でも単体で起動できる形（ランタイムを静的に組み込んだ形）で作る。
4. The リリースビルド shall `pasta_check.exe` を、`cargo install pasta_check` で入るものと同じ機能（`validate`・`release` のサブコマンドと `--help`・`--version`）を持つ形で作る。
5. The リリースビルド shall `pasta_check.exe` を、そのリリースのワークスペースの版と同じ版で作る（`pasta_check.exe --version` がその版を示す）。

### Requirement 2: pasta_check 用のライセンス表示

**Objective:** pasta の開発者として、`pasta_check.zip` に、`pasta_check.exe` に組み込んだ第三者のソフトウェアだけの正しいライセンス表示を同梱したい。そうすれば、各ライセンスの定めを守り、関係の無い表示（LuaJIT など）で利用者を惑わせずに済む。

#### Acceptance Criteria

1. The リリースビルド shall `pasta_check.zip` の `THIRD_PARTY_LICENSES.txt` に、`pasta_check.exe` に組み込んだ第三者のクレート（CLI の機能で入る依存を含む。例: `lexopt`・`md5`・`zip`）の著作権表示と許諾文を載せる。
2. The リリースビルド shall `pasta_check.zip` の `THIRD_PARTY_LICENSES.txt` に、`pasta_check.exe` に組み込んでいないソフトウェア（LuaJIT と、`pasta.dll` だけが使うクレート。例: `mlua`）の表示を載せない。
3. The リリースビルド shall `pasta_check.zip` の `THIRD_PARTY_LICENSES.txt` の冒頭で、その表示が `pasta_check.exe` のためのものであることと、pasta 自体が MIT License で配布されていることを示す。
4. The リリースビルド shall 今の `pasta.dll.zip`（と `hello-pasta.nar` の中の `ghost/master/THIRD_PARTY_LICENSES.txt`）・VSIX の中の WASM 用のライセンス表示の中身を、本仕様の前と変えない。
5. If `pasta_check.exe` の依存に、許可したライセンス（`deny.toml` の `[licenses].allow` と同じ一覧）に無いライセンスのものがあるとき, the リリースビルド shall ライセンス表示を作らずに失敗し、`pasta_check.zip` を作らない。
6. If ライセンス表示を作る道具（cargo-about）が手元に無いとき, the リリースビルド shall 失敗し、道具の入れ方を示す。

### Requirement 3: 手元のリリースビルドで 4 つの一式を作る

**Objective:** pasta の開発者として、手元の `release.bat` でも、リリース CI と同じ 4 つの配布物を作って確かめたい。そうすれば、タグを push する前に、配布物が作れるかを手元で確かめられる。

#### Acceptance Criteria

1. When リリースビルドが成功したとき, the リリースビルド shall `release/` に `pasta.dll.zip`・`hello-pasta.nar`・`pasta_check.zip` を作り、終わりのまとめに `pasta_check.zip` の場所も示す。VSIX はこれまでどおり VSCode 拡張のパッケージの手順が作り、リリースビルドは VSIX を作らない。
2. When `pasta_check.zip` を作ったとき, the リリースビルド shall zip の中身が `pasta_check.exe` と `THIRD_PARTY_LICENSES.txt` のちょうど 2 つであることを検査する。
3. If `pasta_check.zip` の中身がちょうどその 2 つでないとき, the リリースビルド shall 期待する中身と実際の中身の両方を示して失敗する。
4. If `pasta_check.exe` のビルドが失敗したとき, the リリースビルド shall 失敗し、x64 のターゲットが入っていない場合に足す方法（`rustup target add x86_64-pc-windows-msvc`）を示す。
5. The リリースビルド shall 段の見出しの番号（`[1/6]` など）と、スクリプトの冒頭の説明・`release.bat` の説明を、新しい段を含めた並びに合わせる。
6. While `-SkipSetup` または `-SkipDllBuild` を指定して実行しているとき, the リリースビルド shall `pasta_check.zip` の段を飛ばさずに行い、作り直す。`pasta_check.zip` は `pasta.dll` の準備（setup）と別の物である。`pasta_check.zip` の段だけを飛ばす指定は足さない。
7. The リリースビルド shall 本仕様の前から作っていた配布物（`pasta.dll.zip`・`hello-pasta.nar`）の中身と、その検査を変えない。
8. The リリースビルド shall 手元とリリース CI で同じ手順を使い、リリース CI だけの別の作り方を持たない。

### Requirement 4: リリース CI で作り、検査し、渡す

**Objective:** pasta の開発者として、リリース CI が `pasta_check.zip` をタグのソースから作り、中身を確かめてから公開してほしい。そうすれば、壊れた `pasta_check.zip` や欠けた添付のまま Release が公開されることを防げる。

#### Acceptance Criteria

1. When リリースタグが push され、関門の検査が通ったとき, the リリース CI shall build の job で、タグのコミットのソースから `pasta_check.zip` を、ほかの配布物と同じリリースビルドで作る。
2. When build の job が配布物を検査するとき, the リリース CI shall `pasta_check.zip` が在り、中身が `pasta_check.exe` と `THIRD_PARTY_LICENSES.txt` のちょうど 2 つであることを検査する。
3. When build の job が配布物を検査するとき, the リリース CI shall `pasta_check.zip` の中の `pasta_check.exe` を起動し、`--version` が示す版がリリースタグの版と一致することを検査する。VSIX の版の検査と同じく、壊れた exe や古い exe の取り違えを公開前に止める。
4. If `pasta_check.zip` が無いか、中身か版が合わないとき, the リリース CI shall 何が合わないかを示して build の job を失敗させ、どの公開先（crates.io・Marketplace・GitHub Release）へも公開しない。
5. When build の job の検査が通ったとき, the リリース CI shall `pasta_check.zip` を、ほかの配布物と同じ受け渡し（artifact `release-assets`）に入れて後続の job へ渡す。
6. The リリース CI shall build の job が `Cargo.lock` を書き換えていないことの検査を、`pasta_check.exe` のビルドを足した後も保つ。
7. The リリース CI shall 毎回の CI（`build.yml`）の構成と、関門（gate）が呼ぶ検査を変えない。

### Requirement 5: GitHub Release に 5 つを添付する

**Objective:** Rust を入れていないゴーストの作者として、GitHub Release に `pasta_check.zip` が載っていて、固定のリンクで最新版を落とせてほしい。そうすれば、版を気にせずに最新の `pasta_check` を入れられる。

#### Acceptance Criteria

1. When GitHub Release を作るとき, the リリース CI shall 配布物 4 つ（`pasta.dll.zip`・`hello-pasta.nar`・`pasta-vscode-X.Y.Z.vsix`・`pasta_check.zip`）と詳しいリリースノート `release-notes-vX.Y.Z.md` の 5 つを添付する。
2. The リリース CI shall `pasta_check.zip` をこの名前（版を含まない名前）で添付し、`https://github.com/ekicyou/pasta/releases/latest/download/pasta_check.zip` で最新版を落とせるようにする。
3. If 添付する 5 つのうち 1 つでも手元に無いとき, the リリース CI shall Release を作らず・公開せずに失敗し、無いファイルの名前を示す。
4. When 5 つの添付がすべてそろったとき, the リリース CI shall Release を公開する。5 つがそろう前の Release を公開状態にしない。
5. When 失敗した job を再実行したとき, the リリース CI shall 今と同じ規則（公開済みで 5 つ添付済みなら飛ばす・下書きなら足りない添付だけを付け直す・公開済みの添付は消さない）で続ける。
6. The リリース CI shall 添付の数を 5 つとする判定を、手元で、ネットワークと gh を使わずに確かめられるテストを持つ。テストは少なくとも、`pasta_check.zip` が無いときに Release を作らずに失敗し、その名前を示すことを確かめる。テストは `release-scripts-test.ps1` に足し、毎回の CI でも走らせる。`-DryRun`（gh の認証が要る）は手元の補助の確かめに留める。

### Requirement 6: リリースの状態の読み取りと手順書

**Objective:** リリースを進めるエージェントとして、`release` スキルのリリースの状態の読み取りが、4 つの配布物を数えてほしい。そうすれば、`pasta_check.zip` が欠けた Release を「公開済み」と取り違えずに済む。

#### Acceptance Criteria

1. When リリースの状態を読み取るとき, the リリースの状態の読み取り shall GitHub Release の添付のうち、配布物 4 つ（`pasta_check.zip` を含む）がそろっているかを数え、`n/4` の形で示す。
2. The リリースの状態の読み取り shall 配布物 4 つがすべてそろって公開された Release だけを「あり（公開・添付 4/4）」として公開済みに数え、下書きと添付不足は数えない。
3. The `release` スキルの手順書 shall 状態の表の読み方（`あり（公開・添付 4/4）`・`下書き（添付 n/4）`・`添付不足（公開・添付 n/4）`）と、添付不足のときに `release-assets` から手で添付する案内を、4 つの配布物に合わせる。
4. The `release` スキル shall 公開先の数（crates.io の 5 クレート・Marketplace・GitHub Release の合わせて 7 つ）を変えない。

### Requirement 7: 入手の手順と配布物の一覧の文書

**Objective:** Rust を入れていないゴーストの作者として、マニュアルや README を読めば、`pasta_check` を Release から落として使う手順が分かってほしい。そうすれば、`cargo install` の手順を見て Rust を入れなければならないと思い込まずに済む。

#### Acceptance Criteria

1. The 利用者マニュアル shall `pasta_check` の入れ方として、Release から `pasta_check.zip` を落として展開する道を最初に示し、固定のリンク（`releases/latest/download/pasta_check.zip`）を載せる。
2. The 利用者マニュアル shall `cargo install pasta_check` を、Rust を使っている人向けの別の道として残す。
3. The 利用者マニュアル shall `pasta_check.exe` が 64 ビット版の Windows 向けであることを示す。
4. The 利用者マニュアル shall `pasta_check.exe` には署名が無いため、初めて起動するときに Windows の SmartScreen の警告が出ることがある、と一言書く。
5. The 利用者マニュアル shall 入れ方の節に、展開した `pasta_check.exe` をパスで指定して呼ぶ例を 1 つ示す（例: `C:\tools\pasta_check\pasta_check.exe validate --target ghosts\hello-pasta`）。ほかの節のコマンド例は `pasta_check validate …` の書き方のままにする。PATH への登録の手順は書かない。
6. The 文書 shall `pasta_check` の入れ方を書いた所（`crates/pasta_check/README.md`・`pasta-check` スキル）を、マニュアルと同じ順（Release の zip が第一、`cargo install` が別の道）にそろえる。
7. The 文書 shall 配布物の一覧を書いた所（ルートの `README.md`・マニュアルの `introduction.md`・`crates/pasta_sample_ghost/RELEASE.md`・`crates/pasta_sample_ghost/README.md`・`pasta-check` スキル・steering の `tech.md`・`structure.md`）に `pasta_check.zip` を加え、配布物の数・添付の数・リリースビルドの段の並びの記述を 4 つ（添付は 5 つ）に合わせる。
8. When マニュアルの挙動の説明を変えたとき, the 文書 shall 同じ変更の中でスキルの `references/` を作り直し、鮮度の照合（`gen-skill-refs.mjs --check`）が通る状態にする。
9. The 文書 shall マニュアルの検査（リンクの検証など、`manual.yml` が行うもの）を通る状態に保つ。

## 設計への申し送り

要件ディスカッション（2026-10-22）で、議題 5 件はすべて推奨案で確定した。次の事項は設計で決める（詳しくは `research.md` の 5 節・7 節）。

1. ライセンス表示のひな形を分けるか、1 つのひな形を条件で出し分けるか。要件は R2 の「pasta_check の依存だけ・LuaJIT を載せない・今の 2 つの表示を変えない」とする。CLI の機能（`cli`）を入れた形で依存を数えること、`--target x86_64-pc-windows-msvc` で wasm 向けの依存を外すことを確かめる。
2. リリースビルドの段 3（`pasta_check release` の実行）を、新しく作る x64 の `pasta_check.exe` で走らせるか、今の `cargo run -p pasta_check` のままにするか。要件は R3.7（今の配布物と検査を変えない）だけを課す。
3. exe の版の検査（R4.3）を、手元のリリースビルドでもするか。
4. rust-cache の key の付け方。
