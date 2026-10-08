# ギャップ分析: release-ci

- 分析日: 2026-10-08
- 対象: `requirements.md`（Requirement 1〜12、未決事項 1〜11）
- base: `2cbaf510`
- 方法: コードベースの調査（ワークフロー・スクリプト・追跡中の成果物・`release-workflow` spec・文書）と、外部サービスの既知の仕様の照合。外部仕様のうち確証の無いものは「要調査」とした。

## 1. 分析の要約

- **新設が中心**: タグを契機に動くワークフローはまだ無い。`build.yml` は main への push と PR だけで動き、`permissions:` も secrets も使っていない。リリース CI の骨格（起動・関門・ビルド・公開・Release・再実行・結果の報告）はすべて新しく作る。
- **流用できる部品はそろっている**: `release.ps1`（DLL・画像・ライセンス表示・nar）、`pasta_check release`（nar と `updates.txt`）、`build-wasm.ps1` と `npm run package`（VSIX）、`release-workflow` design.md の公開順・冪等判定・リリースノートの分類。ただし、どれも「開発機で人やエージェントが動かす」前提で書かれており、CI 向けの調整が要る（ツールの導入・`powershell` 5.1 の呼び出し・ソースツリーへの書き込み・dll.zip がスクリプトの外にあること）。
- **成果物の追跡の解除は小さいが波及がある**: テストは追跡中の `pasta.dll`・`THIRD_PARTY_LICENSES.txt`・`release/**` に依存していない（調査済み）。一方で `release.ps1` はソースツリー（`ghosts/hello-pasta/`）へ生成物を書き込む作りで、追跡を解除しても書き込み先は同じなので `.gitignore` での無視が要る。シェルの画像と `scripts/README.md` も追跡中の生成物で、扱いは未決（未決事項 7）。
- **外部サービスの一回限りの設定が最大のリスク**: crates.io の Trusted Publisher（5 クレート）、Azure のマネージド ID とフェデレーション資格情報、Marketplace の publisher へのメンバー追加。設定の名前の一致（ワークフローのファイル名・environment 名）を誤ると公開が認証で失敗する。期限（2026-12-01）があるので、Marketplace の経路を早く実地で確かめる必要がある。
- **推奨**: Option C（ハイブリッド）。ワークフローは新設し、成果物の生成は既存スクリプトを最小限に調整して呼ぶ。公開済みの判定とリリースノートの生成は、ワークフロー内の小さなスクリプトに切り出す。

## 2. 現状の調査

### 2.1 ワークフロー

| ファイル | 起動 | 内容 | リリース CI との関係 |
| --- | --- | --- | --- |
| `.github/workflows/build.yml` | main への push・PR・手動 | windows-latest で x86・x64 の matrix。日本語ロケール（`shell: powershell` で `Set-WinSystemLocale`・`Set-Culture`）→ `cargo build -p pasta_shiori` → `cargo test --all` → clippy（x64 のみ・`-D warnings`）→ DLL を artifact へ（7 日）。別 job で cargo-deny（ubuntu）・luacheck（ubuntu）・WASM ビルド（ubuntu・`cargo build --target wasm32`・10MB 上限） | 関門（R2）のツールチェーン構成の手本。変更しない（範囲外） |
| `.github/workflows/manual.yml` | `book/**` 等の変更 | mdBook のビルドと Pages 公開。`permissions: id-token: write` の前例あり。`verify-content.mjs` がマニュアルの版行と `Cargo.toml` を照合 | 範囲外。`id-token: write` の書き方の前例 |

- タグを契機に動くワークフローは無い。`permissions:` の指定は `manual.yml` だけ。
- `rust-toolchain.toml` は無く、`dtolnay/rust-toolchain@stable` を使っている。リリースのたびに stable の版が変わりうる。
- `NoDefaultCurrentDirectoryInExePath` はワークフローに現れない（開発機だけの制約）。

### 2.2 成果物を作るスクリプト

**`crates/pasta_sample_ghost/release.ps1`**（対話なし・`-SkipSetup`・`-SkipDllBuild`）

1. `cargo build --release --target i686-pc-windows-msvc -p pasta_shiori`
2. `cargo run -p pasta_sample_ghost` で `shell/master/surface*.png`・`surfaces.txt` を `ghosts/hello-pasta/` へ生成
3. `pasta.dll` を `ghosts/hello-pasta/ghost/master/` へ写す。`cargo about generate` で `THIRD_PARTY_LICENSES.txt` を同じ場所へ生成（`cargo-about` が無ければ exit 1）。`crates/pasta_lua/scripts` を `ghost/master/scripts` へ robocopy /MIR
4. `cargo run -p pasta_check -- release --target ghosts/hello-pasta --release release/hello-pasta --nar release/hello-pasta.nar`
5. ルートの `Cargo.toml` の最初の `version = "..."` を読む（表示用）
6. `gh release create` の案内を表示するだけ

- **`pasta.dll.zip` はこのスクリプトの外**で作っている（`release-workflow` design.md Phase 5 の `Compress-Archive`。中身は x86 の `pasta.dll` と `THIRD_PARTY_LICENSES.txt` の 2 つ。出典は `completed/release-workflow-dll-zip-fix`）。
- ルートの `release.bat` が `powershell.exe -ExecutionPolicy Bypass -File ...release.ps1` を呼ぶ（`structure.md` は置き場所を `crates/pasta_sample_ghost/` と誤記）。
- 生成物をソースツリー（`ghosts/hello-pasta/`）へ書き込んでから nar にする作り。

**`editors/vscode`**

- `package.json`: `pasta-vscode`・版 `0.3.7`・publisher `ekicyou`。`prepackage` = `npm run build:wasm && npm run compile`、`package` = `vsce package`、`build:wasm` = `powershell -File scripts/build-wasm.ps1`。
- **`vscode:prepublish` が無い**: `vsce publish` 単体ではビルドが走らない。`vsce package` で作った VSIX を `vsce publish --packagePath` で出すのが安全。
- `build-wasm.ps1`: `wasm-pack` が PATH に必要。`-Release` が渡らないので **`--dev` ビルド**（VSIX に入る WASM はデバッグビルド。未決事項 6）。`cargo-about` で `wasm/THIRD_PARTY_LICENSES.txt` も作る。
- VSIX の既定の名前は `editors/vscode/pasta-vscode-X.Y.Z.vsix`。`*.vsix`・`wasm/`・`out/`・`node_modules/` は `editors/vscode/.gitignore` で無視済み。
- `package-lock.json` の native addon: `keytar`（vsce の optional）・`@vscode/vsce-sign`・`esbuild`。VS 2026 のイメージで node-gyp が VS を検出できない場合に注意（brief の制約）。`@vscode/vsce` 3.7.1 は node 20 以上が要る。
- `package-lock.json` の L3・L9 にも版 `0.3.7` がある（コミット `75d00f16` で同期）。`release-workflow` design の bump 箇所の一覧に無い。

### 2.3 git で追跡している成果物

| パス | 種類 | 生成元 | 要件での扱い |
| --- | --- | --- | --- |
| `release/hello-pasta.nar` | nar | `pasta_check release` | R10.1 で追跡解除 |
| `release/hello-pasta/**`（`updates.txt` 2 つを含む 34 ファイル） | 展開した配布物 | `pasta_check release` | R10.1 で追跡解除 |
| `ghosts/hello-pasta/ghost/master/pasta.dll` | DLL | `release.ps1` step 3 | R10.1 で追跡解除 |
| `ghosts/hello-pasta/ghost/master/THIRD_PARTY_LICENSES.txt` | ライセンス表示 | `cargo about` | R10.1 で追跡解除 |
| `ghosts/hello-pasta/shell/master/surface*.png`・`surfaces.txt` | 画像 | `cargo run -p pasta_sample_ghost` | R10.5 で現状維持（未決事項 7） |
| `ghosts/hello-pasta/ghost/master/scripts/README.md` | 写し | robocopy | 未決事項 7 |

- `.gitignore` に `release/`・`*.nar`・`pasta.dll` の行は無い。**`Cargo.lock` は無視している**（「ライブラリクレートなので」）。CI のビルドは依存の解決を固定していない（未決事項 5）。
- テストへの影響: `dist_src_validation_test.rs` が要求するのは手書きの 8 ファイルだけ。`pasta_shiori` の e2e テストは `ghost/master` を一時ディレクトリへ写すが、`pasta.dll` は `target/` から探す（`tests/common/mod.rs` の `copy_pasta_shiori_dll`）。追跡中の DLL・ライセンス表示・`release/**` に依存するテストは見つからなかった。→ R10.3 は追跡の解除だけで満たせる見込み（ただしクリーンなチェックアウトでの確認は必須）。
- `updates.txt` は `date=` を含むため、nar はバイト単位では再現しない。R3.9 の「同じ構成」は満たせるが、ハッシュの一致は求められない。
- `descript.txt` の `homeurl` はリポジトリのトップ（`https://github.com/ekicyou/pasta`）を指すだけで、`release/` を git に置くことはネットワーク更新に寄与していない（brief どおり）。

### 2.4 `release-workflow` spec から引き継ぐもの

| 項目 | 内容（出典） | 引き継ぎ先 |
| --- | --- | --- |
| 公開順 | `pasta_core` → `pasta_dsl` → `pasta_lua` → `pasta_shiori` → `pasta_check`（design.md:528）。実際の依存: lua ← core・dsl、shiori ← lua、check は独立 | R4.1 |
| 公開済みの判定 | crates.io: `GET /api/v1/crates/<crate>/<version>`（200=公開済・404=未公開。**User-Agent 必須**。spec には記載が無いが、無いと 403）。Marketplace: `vsce show <publisher>.<ext> --json`。Release: `gh release view`＋添付の差分 | R4.3・R5.3・R6.4〜6.5・R7.2 |
| リリースノート | 前のタグ = `git tag -l "v*" --sort=-version:refname`、`git log <prev>..<tag> --oneline --no-merges`。6 分類の見出し、`spec` スコープ除外、空の見出しは省略、Full Changelog のリンク（design.md:473-483）。`perf`・`ci`・`build`・`style` や Conventional Commits でないコミットの扱いは未定義 | R6.6〜6.9 |
| Release の題名・添付 | `--title "pasta vX.Y.Z"`、dll.zip・nar・VSIX（今は VSIX が任意） | R6.2〜6.3 |
| main の CI 全緑の確認 | `gh run list --branch main --workflow build.yml ...`（design.md:401） | R2.8 で置き換え |
| タグの到達性 | マージコミット方式でタグのコミットを main から到達可能にする | R2.4 で検査（統合方式の見直しは `release-workflow` 側） |

### 2.5 文書（更新候補）

- `crates/pasta_sample_ghost/RELEASE.md`（全体。手で `gh release create` する手順・PAT 不要の記述なし・dll.zip と VSIX の記述なし）
- `.claude/skills/pasta-check/SKILL.md`（L3 の description、L80-92 の 2 段の流れ、L100-112 の木構造、L123-133 の `gh release create`）
- `crates/pasta_sample_ghost/README.md`（L41・L54-65・L77。DLL を `ghost/master` に置く説明）、`release.ps1` L226-253 の案内表示、`src/main.rs` L60-61、`build.rs` L35-42
- `.kiro/steering/structure.md`（L199-200 の `release.bat` の位置、L251）、`workflow.md`（L143 の CI 全緑の関門、L151 のタグ push の扱い）
- `book/src/introduction.md` L18（リリースページの配布物の説明。中身は変わらないので確認だけ）
- `.claude/settings.json`（`cargo publish`・`vsce publish`・`gh release create` 等の許可）— `release-workflow` の更新で扱うのが自然（本仕様の範囲外の候補）

### 2.6 外部サービスの仕様（2026-10-08 調査）

出典は各項目の末尾。「推測」とあるものは一次資料で確かめていない。

**crates.io Trusted Publishing**
- `rust-lang/crates-io-auth-action@v1`（最新 v1.0.5、2026-06）が GitHub の OIDC トークンを crates.io の一時トークンに交換し、`outputs.token` を `CARGO_REGISTRY_TOKEN` として渡す。job に `id-token: write` が要る。job の終わりにトークンを失効させる。node24 の JS アクションなので Windows ランナーでも動く（推測。公式例は ubuntu）。
- トークンの期限は **30 分**（crates.io のソース `ACCESS_TOKEN_LIFETIME = minutes(30)`）。1 つのトークンで、設定が一致する全クレートを公開できる。
- 設定はクレートごと（owner・repo・ワークフローのファイル名・environment は任意）。owner と repo は大文字小文字を区別しない。ワークフローのファイル名は完全一致。environment を設定したら JWT の environment と一致が必須。owner は数値 ID で照合されるので、リポジトリの改名・移管で作り直しが要る。
- 起動の種類: `push`（タグ）・`release`・`workflow_dispatch` は可。`pull_request_target`・`workflow_run` は拒否。
- 新しいクレートの作成はできない（"Trusted Publishing tokens do not support creating new crates"）→ R4.6 の検出と手順書 R11.8 が要る。
- クレートごとに「Trusted Publishing のみ」（`trustpub_only`）を有効にできる（未決事項 13）。
- 出典: rust-lang/crates.io（`src/controllers/trustpub/tokens/exchange/mod.rs`・PR #12346）、blog.rust-lang.org（2025-07-11・2026-01-21）、rust-lang/crates-io-auth-action

**cargo publish**
- 公開済みの版は、アップロード前に索引で検査され `crate NAME@VER already exists on crates.io index` で失敗する。`--workspace`（Rust 1.90 で安定化）は全メンバーを先に検査するため、**1 つでも公開済みがあると何も公開しない**（brief の判断を裏付け）。`--skip-published` に当たる安定版のフラグは無い。
- 索引の反映待ちの時間切れは stable では **60 秒固定**（`publish.timeout` は nightly の `-Z publish-timeout` が要る）。最後のクレートで時間切れになると警告して成功する。クレートを 1 つずつ `-p` で公開する方式なら、各回が「最後のクレート」になり、時間切れは警告で済む（推測）。次のクレートの検証ビルドが依存先の未反映で失敗したときは再試行が要る（R4.5）。
- 出典: rust-lang/cargo `src/ops/registry/cargo_publish.rs`、Rust 1.90.0 のリリース記事

**VS Marketplace（Entra ID）**
- `vsce publish --azure-credential`（vsce 2.26.1 以上）は Azure DevOps のリソース（`499b84ac-1321-427f-aa17-267ca6975798/.default`）のトークンを、Environment → Azure CLI → ManagedIdentity → Azure PowerShell → azd の順で探す。`azure/login` の後なら Azure CLI のセッションを使う（推測）。
- Microsoft の文書の手順: ユーザー割り当てのマネージド ID（Reader）＋フェデレーション資格情報 → `az rest -u https://app.vssps.visualstudio.com/_apis/profile/profiles/me --resource 499b84ac-...` で ID を得る → publisher に Contributor で追加。GitHub Actions 向けの公式ページは今も `VSCE_PAT` の例だけ。
- **subject の罠**: マネージド ID のフェデレーション資格情報は subject の完全一致だけ（ワイルドカードはアプリ登録だけの preview）。environment なしのタグの push では `sub=repo:O/R:ref:refs/tags/<TAG>` でタグごとに変わる。environment を付けると `repo:O/R:environment:<NAME>` で固定になる → **publish-vsce の job に environment が必須**。2026-07-15 以降に作られたリポジトリは ID ベースの subject（このリポジトリは旧形式のまま。opt-in しない限り）。
- `azure/login`（v3.1.0）: `client-id` にユーザー割り当てマネージド ID を指定する。`subscription-id` は既定で必須（マネージド ID はサブスクリプションに属するので問題ない）。`auth-type: IDENTITY` は Azure VM 上のセルフホスト用で、今回は使わない。
- microsoft/vscode-vsce#1023: サービスプリンシパルで `verify-pat` は通るが `publish` が "corporate credentials" で失敗。原因は示されないまま 2024-11-05 に not_planned で閉じられた（brief の判断を裏付け）。
- **新経路**: vsce に隠しオプション `publish --oidc`（PR #1291、2026-07-23 マージ、2026-09-29 に Marketplace の契約確定のコミット）。GitHub の OIDC トークン（audience `marketplace.visualstudio.com`）を Marketplace のセッショントークン（最大 15 分）に交換する。Marketplace 側に「trusted publishing policy」の設定が要るが、手順の公式文書は見当たらない。npm の `latest` は 4.0.0（2026-09-14）、`next` は 4.0.1-3（2026-10-03）。preview 扱い（未決事項 12）。
- **PAT の廃止**: Azure DevOps の global PAT は 2026-12-01 で使えなくなる。vsce の PAT は「All accessible organizations」＝ global PAT なので、`VSCE_PAT` での公開は 12-01 以降に止まる（推測だが確度は高い）。
- `--skip-duplicate`: 同じ版が既にあるか 409 なら「Version X is already published. Skipping publish.」で exit 0。`vsce show <publisher.ext> --json` で版の一覧が取れる。
- 出典: microsoft/vscode-vsce（`src/auth.ts`・`src/publish.ts`・`src/oidc.ts`・PR #1291・issue #1023）、code.visualstudio.com（publishing-extension・continuous-integration）、devblogs.microsoft.com（global PAT の廃止）、Azure/login、learn.microsoft.com（flexible FIC・immutable subjects）

**GitHub Release・Actions**
- `gh release create` は `contents: write` と `GH_TOKEN` が要る。`--verify-tag` で、タグが無いときに既定ブランチからタグを作ってしまうのを防げる。`gh release view <tag>` は無ければ非 0 で終わる。
- **Immutable Releases**: 公開後はタグが固定され、配布物の変更・削除（追加も）ができない。題名・ノートは編集できる。下書きは保護されない。`gh release create TAG files...` は「下書きを作る → 添付 → 公開」の順で動く。immutable な Release を削除すると、そのタグ名は二度と使えない。→ 作成の途中で失敗した場合は下書きが残る形になり、再実行は下書きから続ける設計が要る（未決事項 8）。
- **失敗した job の再実行**: 初回の実行から 30 日以内・50 回まで。`GITHUB_SHA`・`GITHUB_REF` は同じ。失敗した job とその後続だけを走らせ、成功した job の出力を使い回し、前の試行の artifact を使える。通過済みの保護規則は自動で通る（この最後の点は GHES 3.8 の文書の記述。github.com の現行文書では未確認）。
- **artifact**: 既定の保持は 90 日（1〜90 日で指定）。`build.yml` は 7 日。リリース CI では再実行の期限（30 日）以上にする必要がある。
- **environment の保護規則**: 「選んだブランチとタグ」でタグのパターン（`v*`）に限れる。Free プランでは public リポジトリでのみ使える（このリポジトリは public の前提。要確認）。
- 出典: cli.github.com（gh_release_create）、docs.github.com（immutable-releases・re-run-workflows-and-jobs・deployments-and-environments・oidc）、actions/upload-artifact

## 3. 要件ごとの資産とギャップ

タグ: **Missing**（無い）/ **Adapt**（既存を調整）/ **Reuse**（そのまま使う）/ **Unknown**（要調査）/ **Constraint**（制約）

| 要件 | 既存の資産 | ギャップ |
| --- | --- | --- |
| R1 起動 | なし | **Missing**: タグのフィルター（数字 3 つの形）、同じタグの同時実行の抑止（concurrency）、書き戻しをしない権限の最小化 |
| R2 関門 | `build.yml` の test・clippy・ロケール設定 | **Adapt**: 版の照合（ワークスペース・`package.json`）、`origin/main` からの到達性（`fetch-depth: 0` 等で履歴が要る）、関門の範囲（未決事項 3）。**Constraint**: x86 の `cargo test` は時間がかかる（関門の所要時間が延びる） |
| R3 ビルド | `release.ps1`・`pasta_check release`・`build-wasm.ps1`・`npm run package` | **Adapt**: `cargo-about`・`wasm-pack` の版固定の導入、`powershell -File`（5.1）の呼び出しの CI での実行ポリシー、dll.zip の作成をスクリプトかワークフローへ取り込む、job 間で配布物を渡す（artifact）。**Unknown**: VS 2026 イメージでの node-gyp（`keytar` 等）の挙動。**Constraint**: `Cargo.lock` 非追跡（未決事項 5）、WASM が dev ビルド（未決事項 6） |
| R4 crates.io | 公開順・判定 API の知見 | **Missing**: クレートごとの判定と公開のループ、索引の反映待ちの時間切れを次のクレートで再試行する処理、未登録クレートの検出（R4.6）。**Constraint**: 索引待ちは stable で 60 秒固定（2.6）。**Unknown**: Trusted Publishing のトークン期限（30 分）内に 5 クレートを公開しきれるか（`pasta_lua`・`pasta_shiori` は LuaJIT のビルドを含み、検証ビルドが長い） |
| R5 Marketplace | `vsce` の利用経験（PAT 経路） | **Missing**: `azure/login`（OIDC）→ `vsce publish --azure-credential` の経路、`--skip-duplicate` または `vsce show` での判定。**Constraint**: environment 必須（subject 固定。2.6）。**Unknown**: Windows ランナーでの実地の動作、`--oidc` 経路の扱い（未決事項 12） |
| R6 Release | `release-workflow` の分類方式・`gh release create` | **Adapt**: リリースノートの生成をスクリプト化（タグ時点で実行。前のタグの求め方は「リリースタグより前の最新のリリースタグ」にする必要がある。HEAD 基準ではない）。**Missing**: 添付漏れの補完（R6.5）。**Unknown**: Immutable Releases 有効時の `gh release create` の挙動（作成と添付の原子性）（未決事項 8） |
| R7 再実行 | 判定の知見（Resume モード） | **Missing**: job の分割と依存（`needs`）の設計、artifact の保持期間。**Unknown**: 「失敗した job の再実行」で前回の試行の artifact を後続 job が取得できるか、再実行の期限（初回の実行から約 30 日とされる）、再実行がタグのコミットのワークフロー定義を使うこと（未決事項 9） |
| R8 結果の報告 | なし | **Missing**: job summary への公開先ごとの結果の書き出し |
| R9 認証 | `manual.yml` の `id-token: write` の前例 | **Missing**: environment（タグ `v*` に限る保護規則）、publish job だけへの `id-token: write`、`rust-lang/crates-io-auth-action`、`azure/login`。**Constraint**: crates.io の Trusted Publisher の設定値（owner・repo・ワークフローのファイル名・environment 名）と実物の一致 |
| R10 追跡解除 | なし | **Missing**: `git rm --cached` と `.gitignore` の追記。**Constraint**: `release.ps1` が `ghosts/hello-pasta/ghost/master/` へ書く（無視の対象に含める）。シェルの画像は `hello-pasta-shell-art` と調整（未決事項 7） |
| R11 手順書 | なし（brief の Constraints に費用調査あり） | **Missing**: 手順書の新設（置き場所は未決事項 10）。公開を伴わない確かめ方（R11.7）の具体は要調査 |
| R12 文書 | 2.5 の各文書 | **Adapt**: 書き換え。`release-workflow` 本体は触らない |

## 4. 実装の選択肢

### Option A: 既存スクリプトを拡張し、ワークフローは呼ぶだけにする

- `release.ps1` に dll.zip の作成・VSIX のビルド・公開済みの判定・公開・Release の作成までを足し、`release.yml` は環境を整えてスクリプトを呼ぶだけにする。
- ✅ 手元と CI で同じスクリプトが動く。ワークフローが薄い。
- ❌ `release.ps1` はゴーストのビルド用（`pasta_sample_ghost` の持ち物）で、crates・Marketplace の公開は責務の外。後続の `hello-pasta-shell-art` も同じファイルを触るので衝突が増える。
- ❌ 公開の各段を別 job に分けにくく、「失敗した job の再実行」の単位が粗くなる。OIDC の権限を公開の処理だけに与えにくい（R9.3）。

### Option B: ワークフローに全部を新しく書く

- `release.yml` の各 job に、ビルド・判定・公開・ノート生成をすべて直接書く。`release.ps1` は手元用に残し、CI は使わない。
- ✅ job の分割と権限の最小化が素直。既存スクリプトに手を入れない。
- ❌ nar・dll.zip の作り方が 2 か所に分かれ、中身が同じであること（R3.2・R3.3）を保つのが難しくなる。
- ❌ YAML の中の長いスクリプトは、手元で試しにくい。

### Option C: ハイブリッド（推奨）

- **新設**: `release.yml`（verify → build → publish-crates / publish-vsce → github-release の job 構成、concurrency、environment、job ごとの permissions、job summary）。
- **調整して流用**: 成果物の生成は `release.ps1`（CI からも呼べるよう最小限の調整。例: ツール導入済みを前提にする・dll.zip の作成を取り込むか別の小さな手順に置く）と `npm run package`（`build-wasm.ps1` の呼び出し方を CI 向けに）。
- **小さなスクリプトへ切り出し**: 公開済みの判定（crates.io API・`vsce show`・`gh release view`）と、リリースノートの生成。手元で試せる形にし、ワークフローから呼ぶ。
- ✅ 成果物の作り方を 1 か所に保ち（R3 の「今と同じ中身」）、公開は job 単位で分けて再実行と権限の最小化を満たす。
- ❌ 新しいスクリプトの置き場所と、`hello-pasta-shell-art` との `release.ps1` の触り分けを設計で決める必要がある。

## 5. 規模とリスク

- **規模: M〜L（1〜2 週間）** — ワークフローと小さなスクリプトの新設、既存スクリプトの調整、追跡の解除、手順書と文書の更新。コード量は大きくないが、外部サービスの設定と実地での確認（タグを打たないと最後まで試せない）に時間がかかる。
- **リスク: High** — (1) 取り消せない公開を、実地でしか確かめられない経路（OIDC・Entra ID）で行う。(2) Marketplace の Entra ID 経路は報告された失敗例（microsoft/vscode-vsce#1023）があり、期限（2026-12-01）がある。(3) Windows ランナーのイメージ（VS 2026）の変化で native addon・LuaJIT のビルドが壊れうる。(4) Trusted Publishing のトークン期限と、LuaJIT を含むクレートの検証ビルドの長さの兼ね合い。

## 6. 設計フェーズへの申し送り

### 推奨と主な判断

- Option C を基本にする。job の境界は brief の 5 つ（verify・build・publish-crates・publish-vsce・github-release）を出発点にする。
- 配布物は build job で 1 度だけ作り、artifact で後続へ渡す。publish job は再ビルドしない（R3.8）。ただし `cargo publish` はクレートのソースから検証ビルドを行うので、crates の公開には artifact を使わない。
- 公開済みの判定は、各公開の直前に公開先へ問い合わせる（R4.3・R5.3・R7.2）。問い合わせの一時的な失敗は「公開済み」と見なさない（R7.4）。
- `vsce publish` は `--packagePath` で build job の VSIX を出す（`vscode:prepublish` が無いため）。
- Marketplace の公開を、Trusted Publishing のトークンの期限から切り離す（別 job）。
- publish-vsce の job には environment を必ず付ける（マネージド ID のフェデレーション資格情報の subject を固定するため。2.6）。publish-crates も同じ environment に属させ、crates.io の Trusted Publisher の設定に environment 名を入れる。
- GitHub Release は「下書き → 添付 → 公開」の順で作り、Immutable Releases を有効にしても添付漏れの Release が公開されないようにする。
- GitHub Release は配布物が欠けたまま公開状態で残さない（requirements R6.5）。作成の順序と、再実行で残った下書きから続ける方法は設計で決める（未決事項 8）。
- リリースノートの「前のタグ」は、リリースタグより前の最新のリリースタグとして求める（`--sort=-version:refname` でリリースタグ自身を除く）。

### 要調査（Research Needed）

2.6 の調査で多くが解消した。残りは次のとおり。

1. **トークン期限と公開時間**: Trusted Publishing のトークンは 30 分。`pasta_lua`・`pasta_shiori` は LuaJIT を含み、`cargo publish` の検証ビルドが長い。5 クレートが 30 分に収まるかを実測する。収まらなければクレートごとにトークンを取り直す（アクションを複数回呼ぶ）か、検証ビルドの扱いを決める。
2. **Marketplace の経路の実地確認**: マネージド ID を publisher のメンバーに追加する UI での ID の指定方法、`azure/login` の後の `vsce publish --azure-credential` が Windows ランナーで通るか。期限（2026-12-01）があるので、設計の早い段階で試す（例: 公開を伴わない確認の方法の有無）。
3. **`vsce publish --oidc` の成熟度**（未決事項 12）: Marketplace 側の trusted publishing policy の設定手順と、正式な提供の見込み。
4. **再実行と artifact**: github.com の現行の挙動として「失敗した job の再実行で、前の試行の artifact と成功した job の出力を使える」ことの確認。
5. **Immutable Releases と再実行**（未決事項 8）: `gh release create` が下書きの段階で失敗したときに残る下書きを、再実行で見つけて続ける方法（`gh release view` は下書きを返すか）。
6. **windows-latest（VS 2026 イメージ）**: `npm ci` の native addon（`keytar`・`@vscode/vsce-sign`）、`wasm-pack`・`cargo-about` の導入（版固定のバイナリ取得か `cargo install --locked` か）。退避先 `windows-2022` の提供期限。
7. **実行ポリシー**: ランナー上で `powershell -File`（5.1）による `build-wasm.ps1` の呼び出しが通るか（開発機では AllSigned で失敗する既知の問題）。
8. **公開を伴わないセットアップの確認**（R11.7）: crates.io のトークン交換だけを試す、`az rest .../profiles/me` で Marketplace 用の ID が引けることを確かめる、など。
9. ~~**リポジトリの公開範囲**~~ → 解消（2026-10-08 要件ディスカッション）: `gh repo view` で `PUBLIC` を確認。environment の保護規則は使える。
10. **一回限りのセットアップの手順書の置き場所**（requirements 未決事項 10）: `crates/pasta_sample_ghost/RELEASE.md` の一節・リポジトリの開発者向け文書・spec 配下などから選ぶ。

### 他 spec との接点

- `hello-pasta-shell-art`（後続）: `release.ps1` と、シェルの画像の追跡の扱いを共有する。本仕様は画像の追跡を変えない前提（未決事項 7）。
- `release-workflow`（後続の更新）: 版の bump 箇所の一覧に `package-lock.json` を足すこと、タグのコミットを main から到達させる統合方式（マージコミット/squash）の見直し、`.claude/settings.json` の公開系コマンドの許可の整理、`workflow.md` L143 の「main の CI 全緑」の関門の記述の更新は、`release-workflow` の更新で扱うのが自然。
