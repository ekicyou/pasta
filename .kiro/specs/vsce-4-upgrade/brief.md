# Brief: vsce-4-upgrade

> **ステータス**: 未着手（2026-10-24 の `/kiro-discovery` で起票。`/update` の提案（PR #187）で「起票を勧める」とした 1 本）。版は未定（0.4.3 には入れない。2026-10-24 開発者の決定）。要件定義は Opus。

## Problem

VSCode 拡張のパッケージと Marketplace への公開には、`@vscode/vsce` 3.x を使っている。vsce 4.0.0 が出ていて、3.x は追いかけるだけになる。

- 4.0 は Node.js 22 を要る。リリース CI は Node 20 で動いていて、Node 20 は 2026-04 に寿命が切れている。
- 4.0 は Marketplace への OIDC の公開の口（trusted publishing）を足し、トークンの交換の手順を Marketplace の取り決めに合わせて変えた（4.0.1 系）。pasta のリリースは `--azure-credential`（Entra ID のワークロード ID 連携）で公開している。この道が 4.x でも通るかは、公開してみるまで分からない。

勧告の面では急がない。3.x の配下の勧告（undici・form-data など）は、PR #187 の `npm audit fix` で 3.9.2 のまま消えた。

## Current State

2026-10-24（main `f472fc27`）に確かめた。

- **版**: `editors/vscode/package.json` 316 行 `"@vscode/vsce": "^3.0.0"`。lock は 3.9.2。
- **使っている所**:
  - `editors/vscode/package.json` 310 行 `"package": "vsce package"`（手元と CI の VSIX のビルド）
  - `.github/workflows/release.yml` 391 行〜の job `publish-vsce`。`npm ci` で lock の版を入れ、Azure/login の OIDC の後に `vsce publish --azure-credential` で公開する（389 行の説明）
  - `.github/workflows/release-setup-check.yml`。`vsce verify-pat ekicyou --azure-credential`（74 行）と `vsce show ekicyou.pasta-vscode --json`（89 行）で、公開の権限と拡張の有無を、公開せずに確かめられる
- **Node の版**: `release.yml` 25 行 `NODE_VERSION: '20'`（115・421 行で使う）、`release-setup-check.yml` 35 行 `'20'`。`manual.yml` は PR #187 で 22 にした（jsdom 30 のため）。
- **手元**: Node 26。非対話では `npm run package` の `build:wasm` が AllSigned で落ちるので、`build-wasm.ps1` を pwsh で先に走らせて `vsce package` を直接叩く。
- **vsce の ECONNRESET**: `vsce show`／`publish` が `read ECONNRESET` で切れるときは `NODE_OPTIONS=--dns-result-order=ipv4first`（手元の IPv6 の経路の問題）。

## Desired Outcome

- VSIX のビルドと Marketplace への公開が vsce 4.x で通る。
- リリース CI の Node が 22 になる。
- 公開の道が 4.x で通ることを、リリースの前に `release-setup-check.yml` で確かめてある。

## Approach

Node 22 への移行を先に済ませ（`ponytail-manual-ci` が「Node 22 に揃える」を持つ）、その後で vsce を 4.x にする。公開の道は `release-setup-check.yml` を `workflow_dispatch` で回して、`verify-pat --azure-credential` と `show` が 4.x で通ることを確かめる。通らなければ、4.x の OIDC の口（trusted publishing）へ移すかを要件で比べる。

## Scope

- **In**:
  - `editors/vscode/package.json`・`package-lock.json` の vsce の版
  - `release.yml`・`release-setup-check.yml` の Node の版（`ponytail-manual-ci` が先に済ませていなければ）
  - `--azure-credential` の公開の道の確認。だめなら公開の認証の手順の直し（`release.yml` の `publish-vsce`・`release-setup-check.yml`・`release` スキル・`tech.md` のリリースの節）
- **Out**:
  - 拡張の中身（`engines.vscode`・`@types/vscode`・`@types/node` の版は動かさない）
  - typescript 7（typescript-eslint 8.72 が対応するのは `<6.1.0` まで。`/update` で見送った）
  - Marketplace の Members の管理（画面からしかできない）

## Boundary Candidates

- ツールの版（package.json と lock）
- リリース CI の Node と公開の認証

## Out of Boundary

- crates.io の公開（trusted publishing のまま）
- GitHub Release の添付

## Upstream / Downstream

- **Upstream**: `ponytail-manual-ci`（`release.yml`・`release-setup-check.yml` の Node 22）
- **Downstream**: 次のリリースの `publish-vsce`

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-manual-ci`（`release.yml`・`release-setup-check.yml` を共有する。同じウェーブに置かない。Node 22 はあちらが持つ）
  - `pasta-dll-multiarch`（`release.yml`）
  - リリースの席: リリースの実行中は `release.yml` を触る spec を main へ入れない

## Constraints

- 公開の認証は OIDC のまま（長期のトークン `VSCE_PAT` を開発機にも GitHub の secret にも置かない。2026-10-10 に失効させた）。
- `vsce publish` は Bash からは分類器が拒否し、PowerShell からは通る（手元で叩くとき）。
- 公開の道は、リリースの公開 job でしか本当には確かめられない。`release-setup-check.yml` の 2 つの確かめで代わりにする。それで足りるかを要件で決める。

## 申し送り（ponytail-manual-ci より・2026-11-03）

- Node 22 への移行は済んだ。`release.yml` の `NODE_VERSION` は `'22'`、`release-setup-check.yml` の `node-version` も `'22'`。`actions/setup-node` はどちらも v7。この spec で Node を上げる手間は無くなった（Scope の「Node の版」と Current State の Node 20 の行は古い）。
- 公開の job の第三者の action は SHA で固定し、行末にタグのコメントを置く形になった（`Azure/login@a641126d… # v3` など。`release-setup-check.yml` も同じ）。`publish-vsce` の job を変えるときは、この形を保つ。
- `publish-vsix.ps1` の「公開済みなら飛ばす」判定は、`release-scripts-test.ps1` の項目 12（偽の vsce が公開済みと答える）が試験するようになった。4.x で `vsce show` の出力が変わるなら、この項目の偽の vsce の答えも合わせる。

## 2026-11-12 棚卸の再測定（main 2381bf52）

- **前提の変化**: `vscode-mcp-client`（#231）が入り、`editors/vscode/package.json` は 316 → 249 行になった。vsce は今も `^3.0.0`（241 行）、`"package": "vsce package"` は 235 行、lock は 3.9.2（`package-lock.json` 1528 行）。その後の `package.json` の変化は版の数字だけ。`release.yml`・`release-setup-check.yml`・`publish-vsix.ps1`・`release-scripts-test.ps1` は 2026-11-03 から変わっていない（`NODE_VERSION: '22'` 27 行・`publish-vsce` 391 行・`Azure/login` 430 行・`publish-vsix.ps1` を呼ぶのは 438 行・`verify-pat` 74 行）。Node 22 は済んだ（上の申し送り）。
- **触るファイル**: `editors/vscode/package.json`（241 行の 1 行）・`package-lock.json`。4.x で振る舞いが変われば `publish-vsix.ps1`（120 行）と `release-scripts-test.ps1`（276 行）の項目 12。公開の道がだめなら `release.yml` の `publish-vsce`・`release-setup-check.yml`（122 行）・`release` スキル・`tech.md` のリリースの節。
- **規模**: 3〜6 タスク（前のまま）。
- **先に要るもの**: 働きの上では無い。ファイルの重なり（未完了）: `pasta-recipe-dev-actions`（`editors/vscode/package.json` の試験のスクリプト・`package-lock.json`）、`pasta-recipe`（`release.yml`）、`pasta-dll-multiarch`（`release.yml`）、CI に clang を入れるなら `pest-2-9-4-upgrade`（`release.yml`）。リリースの実行中は `release.yml` を main へ入れない。
- **種別**: 基盤（道具の版を上げる。開発者の依頼で `/update` から起票）。
- **要件定義のモデル**: Opus（決めることは「公開の道を `release-setup-check.yml` の 2 つの確かめで代えてよいか」と「だめなら OIDC の口へ移すか」の 2 つで、どちらも後から戻せる）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - Current State・Scope の Node 20 の行と、`package.json` の行番号（316・310 行）は古い（上の行が正しい）。`package.json` を共有する相手は `vscode-mcp-client`（完了）から `pasta-recipe-dev-actions` に変わった。
  - 前回から残る: `vsce publish --azure-credential` を実際に叩くのは `publish-vsix.ps1` で、Current State と Scope にこのスクリプトが無い。4.x で 2 つの振る舞い（`--pat` が既定で `VSCE_PAT` を読み `--azure-credential` より勝つこと・拡張が無いと `vsce show` が `undefined` を出して 0 で終えること）が変わっていないかを要件の前に確かめる。手元の `editors/vscode/node_modules` の vsce が lock と違う版のことがあるので、手元で確かめるときは `npm ci` し直す。

## 申し送り（pasta-recipe-dev-actions より）

- `pasta-recipe-dev-actions` は `editors/vscode/package.json`・`package-lock.json` を変えなかった（上の「先に要るもの」に書いた重なりは無くなった）。拡張の開発の操作の流れは `editors/shared/` へ移り、拡張は `nodeGhostIO` で差し込む（同 spec の design.md の「File Structure Plan」「共有の流れ（`editors/shared/`）」）。
- `build.yml` に job `vscode-test`（ubuntu-latest・Node 22・`editors/vscode` で `npm ci` → `npm test`。`PASTA_VSCODE_NO_WASM=1`）が入った。vsce を上げて lock が変われば、この job が main で `npm ci` と拡張の試験を走らせる（同 design.md の「Testing Strategy」の拡張の行）。
