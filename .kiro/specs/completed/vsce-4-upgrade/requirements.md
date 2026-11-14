# Requirements Document

## Project Description (Input)
**誰が困っているか**: pasta のリリースを回す開発者（とリリース CI）。VSCode 拡張 `ekicyou.pasta-vscode` の VSIX のビルドと Marketplace への公開に、パッケージの道具 `@vscode/vsce` を使っている。

**今の状況**: `editors/vscode/package.json` は `"@vscode/vsce": "^3.0.0"`、lock は 3.9.2。vsce は 4.x（最新 4.1.1・Node 22 以上が要る）が出ていて、3.x は追いかけるだけになる。3.x の配下に残る勧告のうち `glob` GHSA-3g3m-2fjr-6pw4（moderate）は 4.x でしか消えない（`minimatch` GHSA-g3hv-75wg-m249 は、vsce 4.1.1・`eslint`・`typescript-eslint` が `minimatch ^10` を要り、直った版が 11.0.1 以上だけなので残る。2026-11-14 に測った）。リリース CI の Node は `ponytail-manual-ci` が 22 にした。公開は Azure/login の OIDC の後に `vsce publish --azure-credential`（`.github/scripts/release/publish-vsix.ps1`）で行っていて、この道が 4.x でも通るかは、公開してみるまで本当には分からない。`release-setup-check.yml`（`verify-pat --azure-credential` と `show`）は environment の保護規則で main からしか走らない。

**何を変えたいか**: VSIX のビルドと Marketplace への公開が vsce 4.x で通るようにし、公開の道が 4.x で通ることを次のリリースの前に `release-setup-check.yml` で確かめてある状態にする。公開の認証は OIDC のまま（長期のトークン `VSCE_PAT` を置かない）。拡張の中身（`engines.vscode`・`@types/vscode`・`@types/node`）と typescript 7 は動かさない。

## Introduction

VSCode 拡張のパッケージと公開の道具 `@vscode/vsce` を 3.9.2 から 4.x（4.1.1）へ上げる。上げた後も、手元と CI で作る VSIX の名前と中身は変わらず、リリース CI の `publish-vsce` は今と同じ認証（Azure のマネージド ID・`--azure-credential`）と同じ「公開済みなら飛ばす」振る舞いで公開できる。公開の道が 4.x で通ることは、公開そのものの前に `release-setup-check.yml` の 2 つの確かめ（`verify-pat --azure-credential`・`show`）で確かめる。

## Boundary Context

- **In scope**:
  - `editors/vscode/package.json` の `@vscode/vsce` の版の指定と、`editors/vscode/package-lock.json`
  - VSIX のビルド（`npm run package`。手元・リリース CI の build）が 4.x で同じ VSIX を作ること
  - 公開の補助スクリプト `publish-vsix.ps1` と、その試験 `release-scripts-test.ps1` の項目 12 が 4.x で同じに振る舞うこと（4.x で振る舞いが変わる所だけを直す）
  - 公開の道の確かめ（`release-setup-check.yml` を main で `workflow_dispatch`）と、その結果の記録
  - 確かめがだめだったときの公開の認証の直し（`release.yml` の `publish-vsce`・`release-setup-check.yml`・`release` スキル・`tech.md` のリリースの節・`.github/release-ci-setup.md`）
- **Out of scope**:
  - 拡張の中身（`engines.vscode`・`@types/vscode`・`@types/node`・拡張のコード）
  - typescript 7（typescript-eslint 8.72 が `<6.1.0` までしか対応しない）
  - Node の版（`release.yml`・`release-setup-check.yml`・`build.yml` は既に 22。`ponytail-manual-ci` が済ませた）
  - Marketplace の Members の管理（画面からしかできない）
  - crates.io の公開（Trusted Publishing のまま）・GitHub Release の添付
  - 開発機に入っている大域の `vsce`（`release` スキルの `release-state.ps1` が `show` だけに使う。既に 4.0.0 で、`show` の振る舞いは 3.x・4.x で同じ。lock の版とは別物）
- **Adjacent expectations**:
  - `ponytail-manual-ci`（完了）が、リリース CI の Node を 22 にし、公開 job の第三者の action を SHA で固定して行末にタグのコメントを置く形にした。この spec はその形を保つ
  - `build.yml` の `vscode-test`（ubuntu・Node 22・`npm ci` → `npm test`）が、lock の変更を main で試す
  - リリースの席: リリースの実行中は `release.yml` を触る変更を main へ入れない。`pasta-dll-multiarch`・`pasta-recipe` と `release.yml` を共有する
  - `release-setup-check.yml` は environment `release-setup-check` の保護規則で main からしか走らない。公開の道の確かめは、版を上げる変更が main へ入った後・次のリリースタグの前にしかできない。そのため版を上げる変更だけを spec の完了より前に単独の PR で main へ入れる

## Requirements

### Requirement 1: vsce の版を 4.x にする
**Objective:** As a リリースを回す開発者, I want パッケージと公開の道具を保守の続く 4.x にしたい, so that 3.x の配下にしか残らない勧告が消え、道具が追いかけるだけの版にならない

#### Acceptance Criteria
1. The 拡張の依存の定義 shall `@vscode/vsce` の版を 4.x の範囲で指定し、lock で 4.1.1 に固定する（指定の範囲は `^4.1.1`。4.0.0 は OIDC のトークン交換が壊れていた版なので下限で外す）
2. When `editors/vscode` で `npm ci` を行ったとき, the 拡張の開発環境 shall lock の 4.x の vsce を入れ、`npx --no-install vsce --version` が 4.1.1 を返す
3. When 4.x へ上げた lock で `npm audit` を行ったとき, the 拡張の依存 shall `glob` GHSA-3g3m-2fjr-6pw4 を報告せず、開発者は残る `minimatch` GHSA-g3hv-75wg-m249 の理由（vsce 4.1.1・`eslint`・`typescript-eslint` が `minimatch ^10` を要り、直った版が 11.0.1 以上だけ。配る VSIX には入らない）を記録する。`overrides` で上げない
4. If 4.x へ上げた lock に、3.9.2 の lock には無かった勧告が現れたとき, the 開発者 shall その勧告を一覧にして、互換の範囲で消せるものは消し、消せないものは理由を残す
5. The 拡張の依存の定義 shall `engines.vscode`・`@types/vscode`・`@types/node`・`typescript` の版の指定を変えない
6. The 拡張の依存の定義 shall vsce 以外の直接の依存の版の指定を変えない（lock の中の推移的な依存の入れ替わりは、vsce を上げたことによるものだけ）

### Requirement 2: VSIX のビルドが 4.x で同じ VSIX を作る
**Objective:** As a リリースを回す開発者, I want 4.x でも今と同じ名前と中身の VSIX ができてほしい, so that 利用者に届く拡張が道具の更新で変わらない

#### Acceptance Criteria
1. When `editors/vscode` で VSIX のビルドを行ったとき, the VSIX のビルド shall `pasta-vscode-<package.json の版>.vsix` の名前の VSIX を 1 つ作る
2. When 4.x で作った VSIX と 3.9.2 で作った VSIX を同じソースから作って比べたとき, the VSIX のビルド shall 同じファイルの一覧（パス）を持つ VSIX を作る
3. If 2 の比較でファイルの一覧に差が出たとき, the 開発者 shall 差の 1 つずつについて原因を確かめ、意図しない差（入るべきものが落ちた・入るべきでないものが入った）は `.vscodeignore` か依存の定義の直しで無くし、残す差は理由を記録する
4. The VSIX のビルド shall 少なくとも `extension/package.json`・`extension/wasm/pasta_lsp_wasm_bg.wasm`・`extension/wasm/THIRD_PARTY_LICENSES.txt` を VSIX に含める（リリース CI の build の既存の中身の検査が通る）
5. While lock の vsce が 4.x である間, the `build.yml` の `vscode-test` shall `npm ci` と拡張の試験を成功させる
6. The リリース CI・`build.yml`・`release-setup-check.yml` shall vsce 4.x を Node 22 以上だけで動かす（今どれも 22。この spec は Node の版を変えない）

### Requirement 3: 公開の補助スクリプトの振る舞いを保つ
**Objective:** As a リリースを回す開発者, I want 4.x でも `publish-vsce` の判定と結果が今と同じであってほしい, so that 失敗した job の再実行で続きから回復する冪等な公開が崩れない

#### Acceptance Criteria
1. When `publish-vsce` が Marketplace へ公開するとき, the 公開の補助スクリプト shall Azure のマネージド ID（`--azure-credential`）で認証し、長期のトークン（PAT）を使わない
2. While 環境に `VSCE_PAT` が在る間, the 公開の補助スクリプト shall その値を vsce に渡さない（4.x でも `--pat` の既定は `VSCE_PAT` で、`--azure-credential` より優先されるため）
3. When 公開しようとする版が Marketplace に既に在るとき, the 公開の補助スクリプト shall 公開せずに `status=skipped`・終了コード 0 で終える
4. If `vsce show` の結果から版の一覧を読めないとき（拡張が無いときに 4.x も `undefined` を出して 0 で終えることを含む）, the 公開の補助スクリプト shall 公開済みと見なさず `status=failed`・`reason=transient` で終える
5. When `vsce publish --skip-duplicate` が「既に公開済み」を検出して 0 で終えたとき, the 公開の補助スクリプト shall `status=skipped` で終える（4.x の文言 `is already published. Skipping publish` に合う）
6. If `vsce publish` が認証で失敗したとき, the 公開の補助スクリプト shall `status=failed`・`reason=auth` で終え、照合する手順書を示す
7. When `release-scripts-test.ps1` を走らせたとき, the 試験 shall 項目 12（偽の vsce が公開済みと答える → `status=skipped`・終了コード 0）を含めて全項目が合格する
8. If 4.x で `show`・`publish` の出力や終了コードが 3.9.2 と変わっていたとき, the 開発者 shall 公開の補助スクリプトと項目 12 の偽の vsce の答えを 4.x の実際の振る舞いに合わせる
9. The `publish-vsce` の job shall 第三者の action を SHA で固定し行末にタグのコメントを置く今の形を保つ

### Requirement 4: 公開の道を公開の前に確かめる
**Objective:** As a リリースを回す開発者, I want 4.x での公開の道が通ることを次のリリースの前に知りたい, so that リリースの公開 job で初めて失敗することを避ける

#### Acceptance Criteria
1. When この spec の実装で、vsce を 4.x へ上げる変更（`package.json` と lock）を spec の完了より前に単独の PR で main へ入れたとき, the 開発者 shall 次のリリースタグを push する前に、main で `release-setup-check.yml` を `workflow_dispatch` で走らせる
2. When `release-setup-check.yml` が 4.x の lock で走ったとき, the 確かめのワークフロー shall `vsce verify-pat ekicyou --azure-credential` の成功と、`vsce show ekicyou.pasta-vscode --json` から版の一覧を読めたことを job summary に書く
3. When 2 の 2 つの確かめがどちらも成功したとき, the 開発者 shall 公開の道が 4.x で通ると判断し、その結果（走らせた日・run の URL・vsce の版）を spec の `research.md` に記録する（公開そのものの確かめはこの 2 つで代える。本当に公開して確かめるのは次のリリースの `publish-vsce` である）
4. While 公開の道の確かめが済んでいない間, the 開発者 shall vsce 4.x の入った main からリリースタグを push しない
5. The 確かめのワークフロー shall どの公開先にも公開せず、crates.io のトークン交換も行わない（今のまま）
6. When この spec の変更を main へ入れる PR を作る前, the 開発者 shall 手元で 4.x の `vsce show ekicyou.pasta-vscode --json` が版の一覧を返すことを確かめる（`read ECONNRESET` のときは IPv4 を優先して取り直す）
7. While 公開の道の確かめが済んでいない間, the 開発者 shall この spec を完了（`/kiro-complete`）しない

### Requirement 5: 確かめがだめだったときの扱い
**Objective:** As a リリースを回す開発者, I want 4.x で公開の道が通らなかったときの次の手が決まっていてほしい, so that リリースが止まったまま放っておかれない

#### Acceptance Criteria
1. If `release-setup-check.yml` の `verify-pat --azure-credential` が 4.x で失敗し、同じ設定の 3.9.2 では成功していたとき, the 開発者 shall vsce 4.x の変更を main から戻す（lock を 3.9.2 に戻す）か、まず vsce 4.x の変更を main から戻す PR（lock を 3.9.2 に戻す）を入れて main をリリースできる状態に戻し、そのうえでこの spec の中で公開の認証の直しか OIDC の口への移行を比べる
2. Where 公開の認証を 4.x の OIDC の口（`--oidc`・Marketplace の trusted publishing）へ移すとき, the 公開の認証 shall 長期のトークンを開発機にも GitHub の secret にも置かない OIDC のままである
3. Where 公開の認証を移すとき, the 開発者 shall `release.yml` の `publish-vsce`・`release-setup-check.yml`・`publish-vsix.ps1`・`release` スキル・`tech.md` のリリースの節・`.github/release-ci-setup.md` を、新しい認証の道に揃えて直す
4. If `verify-pat` の失敗の原因が Members の設定など vsce の版と関係の無いものだったとき, the 開発者 shall vsce の版を戻さず、手順書 `.github/release-ci-setup.md` に沿って設定を直してから確かめ直す
5. If 公開の認証の直しも OIDC の口への移行もできないとき, the 開発者 shall 部分的に出さずこの spec を見送り、vsce を 3.9.2 のままにして、勧告が残る理由を記録する

### Requirement 6: 文書を実際に合わせる
**Objective:** As a リリースを回す開発者, I want 手順書と steering が使っている道具の版と認証の道に合っていてほしい, so that 次のリリースで手順書を読んで迷わない

#### Acceptance Criteria
1. When 公開の認証の道が変わらなかったとき（`--azure-credential` のまま）, the 文書 shall `release` スキル・`tech.md`・`.github/release-ci-setup.md` の認証の記述を変えない
2. If 文書や公開の補助スクリプトのコメントに vsce 3.x に固有の記述（版の数字・3.x のソースの場所）があるとき, the 開発者 shall それを 4.x で正しい記述に直す
3. When この spec を完了するとき, the ロードマップ shall `vsce-4-upgrade` を版の決まっていないもの・依存の更新の一覧から完了へ移す
