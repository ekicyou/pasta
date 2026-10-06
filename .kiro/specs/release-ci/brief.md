# Brief: release-ci

## Problem

リリースは、エージェントが `release-workflow` spec の手順（Stage A〜D）を手元の Windows で毎回たどって行っている。手順が長く、セッションを開いたまま再試行を粘る必要がある（ScheduleWakeup による Resume）。公開に使う認証情報（`CARGO_REGISTRY_TOKEN`・`VSCE_PAT`）も開発機の環境変数に置いている。さらに、Marketplace の公開に必要な Azure DevOps の global PAT は **2026-12-01 に廃止される**。このままでは、手元の手順でも Marketplace へ公開できなくなる。

## Current State

- `.github/workflows/build.yml` は main への push と PR で動く。windows-latest で i686・x86_64 の `pasta.dll` をビルドし、test と clippy を回す（日本語ロケールを設定してから）。タグを契機に動く処理は無い。secrets も使っていない。
- `crates/pasta_sample_ghost/release.ps1` が、DLL のビルド、サンプルゴーストの生成、`cargo about` による `THIRD_PARTY_LICENSES.txt` の生成、`pasta_check release` による `release/hello-pasta.nar` の作成までを行う。`pasta.dll.zip` は別の手順で `Compress-Archive` して作る。
- VSIX は `editors/vscode` の `npm run package` で作る（wasm-pack、cargo-about、esbuild、vsce を使う）。
- ビルドした成果物を git に**コミットしている**。対象は `release/hello-pasta/**`、`release/hello-pasta.nar`、`ghosts/hello-pasta/ghost/master/pasta.dll`、同じ場所の `THIRD_PARTY_LICENSES.txt`。`descript.txt` の `homeurl` はリポジトリのトップページを指すだけで、ネットワーク更新のために git へ置く必要は無い。
- 公開先と公開順の設計は `release-workflow` の design.md にある。
  - crates.io: 5 クレートを依存順（core → dsl → lua → shiori → check）に、公開済みのものは飛ばしながら公開する。
  - Marketplace: `vsce show` で公開済みかを確かめてから公開する。
  - GitHub Release: dll.zip、nar、VSIX を添付する。リリースノートは git log を Conventional Commits の種類ごとに分類して作る。

## Desired Outcome

- 開発者（またはエージェント）がバージョンを上げたコミットを main に入れ、`vX.Y.Z` タグを push する。その後は人手なしで次の 3 つが終わる。
  - crates.io: 5 クレートの公開
  - VSCode Marketplace: 拡張の公開
  - GitHub Release: 作成と成果物（`pasta.dll.zip`・`hello-pasta.nar`・VSIX）の添付
- 失敗した場合は、Actions の「失敗した job の再実行」だけで、公開済みのものを飛ばして最後まで進む。公開済みかどうかは各公開先の実際の状態で判定する（冪等）。
- 長期の認証情報をリポジトリに置かない。crates.io は Trusted Publishing（OIDC）、Marketplace は Microsoft Entra ID のワークロード ID 連携（`azure/login` で OIDC 認証し、`vsce publish --azure-credential` で公開）で認証する。
- ビルドした成果物を git で追跡しない。成果物はタグ時点のソースから CI が作る。タグが指すソースから成果物を再現できる。

## Approach

**タグを契機に動く単一のワークフロー `.github/workflows/release.yml`**（案 A）。2 段の承認（下書きの Release を人が Publish する）と、release-please などの bot は却下した。理由は、「タグを打ったらリリースされる」という要望から外れること、版を決める裁量を失うこと、更新箇所が特殊（workspace の 6 か所・package.json・マニュアルの版行）であること。

job の構成（詳細は設計で決める）:

1. **verify**（windows）
   - タグと `Cargo.toml` の版が一致し、タグのコミットが `origin/main` から到達できることを確かめる。
   - タグのコミットそのもので `cargo test --all` と clippy を回す（日本語ロケールの設定を引き継ぐ）。
   - これを取り消せない公開の前の関門とし、今の「main の CI が全部緑か」の確認を置き換える。
2. **build**（windows）
   - `release.ps1` を流用するか分解して、`pasta.dll.zip` と `hello-pasta.nar` を作る。
   - VSIX は wasm-pack（`wasm-bindgen/wasm-pack`）を版固定で入れて作る。`shell: pwsh` を明示する。
3. **publish-crates**（environment を指定する）
   - 公開の直前に `rust-lang/crates-io-auth-action` でトークンを得る（有効期限は約 30 分）。
   - 1 クレートずつ公開済みかを確かめながら依存順に公開する。`cargo publish --workspace` は公開済みのクレートを飛ばせず、再実行に向かないので使わない。`sleep` は要らないが、index への反映待ちがタイムアウトしたときは次のクレートで再試行する。
4. **publish-vsce**
   - `azure/login`（OIDC）の後に `vsce publish --azure-credential --skip-duplicate` を実行する。
5. **github-release**
   - `gh release view` で作成済みかを確かめてから `gh release create` を実行する。
   - リリースノートは今の分類方式を引き継ぐ。

## Scope

- **In**:
  - `release.yml` の新規作成。
  - 成果物の git 追跡の解除と `.gitignore` への追加。
  - `release.ps1`・`build-wasm.ps1` の CI 向けの調整（必要な範囲）。
  - 公開済みかの判定（crates.io API・Marketplace・GitHub Release の実際の状態）。
  - リリースノートの生成。
  - 一回限りの手動セットアップの手順書。1 回だけ手で行うので、ワークフローの外に置く。
    - crates.io: 5 クレートそれぞれに Trusted Publisher を設定する。
    - Azure: ユーザー割り当てのマネージド ID と、GitHub 用のフェデレーション資格情報を作る。
    - Marketplace: publisher（発行者）`ekicyou` にそのマネージド ID を Contributor として追加する。
    - GitHub: environment と variables（client・tenant・subscription の ID）を作る。
  - 利用者・開発者向け文書の更新（`crates/pasta_sample_ghost/RELEASE.md`、pasta-check スキルの release 手順の記述など）。
- **Out**:
  - バージョンの決定と bump のコミット（`release-workflow` 側に残す）。
  - マニュアルの公開（`manual.yml` の既存の仕組みのまま）。
  - pasta_lsp の独立リリース。
  - Open VSX への公開。
  - クロスプラットフォームのビルド。
  - 新しいクレートの初回公開（Trusted Publishing では作れない。その時に classic トークンで手動で行う）。

## Boundary Candidates

- タグの検証と公開前の関門（verify）
- 成果物のビルド（DLL zip・nar・VSIX）と git 追跡の解除
- 公開（crates.io・Marketplace）と、公開済みかの判定
- GitHub Release とリリースノート
- 一回限りのセットアップ（OIDC の信頼設定・Azure・environment）の手順書

## Out of Boundary

- `release-workflow` spec の書き直し（エージェントの手順を「版の決定 → bump → PR のマージ → タグの push → CI の結果確認・失敗時の再実行」へ縮める）。roadmap の「既存 spec の更新」として別に扱う。
- `build.yml` の PR・main 向けの検証内容の変更（必要なら共通部分を参照するだけにとどめる）。
- GitHub のブランチ保護・Immutable Releases の設定そのもの。

## Upstream / Downstream

- **Upstream**:
  - `build.yml` のツールチェーン構成（crt-static は `.cargo/config.toml`、LuaJIT は mlua の vendored、日本語ロケール）。
  - `release.ps1`、`pasta_check release`、`editors/vscode` の package スクリプト。
  - `release-workflow` design.md の公開順・冪等判定・リリースノートの分類方式。
- **Downstream**: `release-workflow` の縮小（この spec の完成が前提）。今後のすべてのリリース。

## Existing Spec Touchpoints

- **Extends**: なし（新しい境界）。`release-workflow` は書き直しの対象だが、この spec の完了後に別に更新する。
- **Adjacent**:
  - `release-workflow`（常駐 spec。現行の境界外に「CI/CD パイプラインへの統合」と書いてある）。
  - `completed/alpha05-build-ci`（`build.yml`）、`completed/vscode-extension-release`（VSIX）、`completed/release-workflow-dll-zip-fix`（dll.zip の中身）。

## Constraints

- **期限**: Marketplace の global PAT は 2026-12-01 に廃止される。それまでに Entra ID 連携による公開が動いている必要がある。
- **Windows 前提**: DLL・nar・VSIX のビルドは windows-latest で行う（VS 2026 のイメージ。壊れたら `windows-2022` に退避する）。npm の native addon が node-gyp 経由で VS 2026 を検出できない場合に注意する。
- **公開順**: crates は依存順。GitHub Release の作成は crates の公開に成功した後にする（今の設計と同じ）。
- **Trusted Publishing の一致条件**: crates.io の設定のワークフロー名・environment 名と、`release.yml` の実際の名前を一致させる。publish job には `id-token: write` を付ける。
- **成果物の中身は今と同じ**: dll.zip は `pasta.dll` と `THIRD_PARTY_LICENSES.txt`。nar の中身も `pasta_check release` が作るものと同じにする。
- 再実行は冪等にする。公開済みかどうかは、記憶でなく各公開先の実際の状態で判定する。
