# ギャップ分析: vsce-4-upgrade

- 日付: 2026-11-14（worktree の base `c8ec6331`）
- 対象: `requirements.md`（要件 1〜6）
- 外部の調べ: `@vscode/vsce` の GitHub のリリース 2 ページ・main のソース（`main.ts`・`publish.ts`・`oidc.ts`・`auth.ts`・`show.ts`・`package.ts`・`store.ts`）・v3.9.1 のタグのソース（`main.ts`・`publish.ts`・`auth.ts`・`show.ts`）・PR #1291・#1305・#1308・#1309・#1337・#1345・npm の registry（`latest` = 4.1.1・`engines.node` `>= 22`）。4.x のソースは v4.1.1 のタグでなく main で読んだ（4.1.1 は 11/12 の版で、その後の main の変更が混ざりうる。**Research Needed**: 設計で v4.1.1 のタグで読み直す → 6 節で 4.1.1 の配布物を読んで閉じた）

## 1. 今の状態

### vsce に触れる所

| 場所 | 何をするか | 4.x で効くか |
|------|-----------|-------------|
| `editors/vscode/package.json` 241 行 `"@vscode/vsce": "^3.0.0"`・235 行 `"package": "vsce package"`・`prepackage`（`build:wasm` → `compile`） | 手元と CI の VSIX のビルド | 版の指定を変える。`prepackage` は npm が走らせる（vsce は `vscode:prepublish` だけを走らせ、`prepackage` には関わらない）ので変わらない |
| `editors/vscode/package-lock.json` 1518 行 `node_modules/@vscode/vsce` 3.9.2 | 版の固定。配下に `keytar`・`glob` 13・`minimatch` 10・`cheerio`・`markdown-it`・`form-data`・`tmp` など | 入れ替わりが大きい（4.x は keytar → `@napi-rs/keyring`、glob → tinyglobby、markdown-it → marked、cheerio → parse5、tmp・chalk → Node の標準、form-data → Node の streams） |
| `.github/workflows/release.yml` build（windows・Node 22）129〜135 行 `npm ci` → `npm run package` | リリースの VSIX を作る | 4.x の `package` が最初に CI で動くのはここ（下の「穴」）。195〜215 行の中身の検査（名前・`extension/package.json`・WASM・`THIRD_PARTY_LICENSES.txt`・版）が意図しない落ちを捕まえる |
| `release.yml` `publish-vsce`（ubuntu・Node 22・environment `release`）452 行 `npm ci --ignore-scripts` → Azure/login（SHA 固定）→ 466 行 `publish-vsix.ps1` | 公開 | `@napi-rs/keyring` はプラットフォーム別の optional の依存で入るので `--ignore-scripts` でも困らないはず（**Research Needed**: ubuntu で `npm ci --ignore-scripts` の後に vsce が keyring を読み込めずに落ちないか。CI は PAT を保存しないので keyring に触れないはず） |
| `.github/scripts/release/publish-vsix.ps1` | `VSCE_PAT` を消す → `show --json` で公開済み判定 → `publish --azure-credential --packagePath --skip-duplicate` → `show` に版が現れるまで待つ | 調べた範囲では全部そのまま通る（下の表） |
| `.github/scripts/release/release-scripts-test.ps1` 項目 12 | 偽の vsce が `show` に `{"versions":[...]}` を返す → `status=skipped` | 偽の vsce なので vsce の版に関係しない。4.x の `show --json` の形が変わらない限り直さない |
| `.github/workflows/release-setup-check.yml`（ubuntu・Node 22・environment `release-setup-check`・main だけ） | `verify-pat ekicyou --azure-credential`（`continue-on-error`）と `show ekicyou.pasta-vscode --json` | 両方 4.x に残る。`verify-pat` に `--oidc` は無い |
| `build.yml` `vscode-test`（ubuntu・Node 22）`npm ci` → `npm test` | lock の変更を main と PR で試す | `npm ci` が 4.x の lock で通ることはここで分かる。ただし `vsce package` は走らせない |
| `.claude/skills/release/scripts/release-state.ps1` 65 行 `vsce show ekicyou.pasta-vscode --json` | リリースの状態の照会（開発機の大域の `vsce`） | 開発機の大域の `vsce` は既に **4.0.0**（`npm` の大域）。`show` は 3.x と同じなので影響しない。lock の版とは別物 |
| `.kiro/steering/tech.md` 222 行・`.github/release-ci-setup.md` 57・131・132 行 | 認証の道（マネージド ID・`--azure-credential`）の説明 | 認証の道が変わらなければ直さない。vsce の版の数字は書いていない |

### 4.x の振る舞い（publish-vsix.ps1 が頼る所）

| 頼る所 | 3.9.x | 4.x（main で読んだ） |
|--------|-------|-----------|
| `--pat` の既定が `VSCE_PAT` で `--azure-credential` より優先 | そう | **そのまま**（`getPAT` の順は oidc → pat → azureCredential → 保存済み）。`Remove-Item Env:VSCE_PAT` は 4.x でも要る |
| `--azure-credential` の取得（ChainedTokenCredential: Environment → AzureCli → ManagedIdentity → …・scope `499b84ac-…/.default`） | — | `auth.ts` は 3.9.1 と同じ。非推奨の文言は無い |
| `publish --packagePath --skip-duplicate`・文言 `Version X is already published. Skipping publish.` | そう | **そのまま** |
| `show --json` で拡張が無いと `undefined` を出して 0 で終える | そう | `show.ts` は同じ（exit code は推定。**Research Needed**: 手元で無い拡張に `show --json` して確かめる） |
| `package` の名前 `<name>-<version>.vsix` | そう | **そのまま** |
| ファイルの集め方 | glob | tinyglobby。`.vscodeignore`・`files` の照合は minimatch のまま。差はシンボリックリンクの扱いと、大文字小文字の区別がプラットフォーム依存（win32・darwin は区別しない）になったこと |
| Node | 20 で動く | `engines.node` `>= 22`（CLI 自体は版を見ない。20 では EBADENGINE の警告） |
| 新しく増えたもの | — | `publish --oidc`（GitHub Actions だけ・audience `marketplace.visualstudio.com`・`/_apis/gallery/token?api-version=7.2-preview.1` で交換・最長 15 分）。Marketplace 側の trusted publishing の設定が要る。PR #1291 で作者が 10/5 に「まだ使えない・試験中」と答えている（**Research Needed**: Marketplace の画面で今使えるか）。4.1.0 からすべての Marketplace の要求に `X-Market-CI-Provider` ヘッダを付ける（統計用） |

## 2. 要件ごとの対応と穴

| 要件 | 今ある物 | 穴 |
|------|---------|----|
| 1.1・1.2 版を 4.1.1 に | `package.json` 241 行・lock | **Missing**: 1 行と lock の作り直しだけ |
| 1.3・1.4 勧告 | 2026-11-13 の `/update` の記録 | **Missing**: 上げた後の `npm audit` の結果の照合。4.x の新しい推移的な依存（`@napi-rs/keyring`・`marked`・`parse5`・`tinyglobby`）の勧告は未確認（**Unknown**） |
| 1.5・1.6 他の版を動かさない | — | **Constraint**: `npm install @vscode/vsce@^4.1.1` は lock の他の項目まで動かすことがある。差分を見て vsce の配下の入れ替わりだけであることを確かめる |
| 2.1・2.4 VSIX の名前と必須の中身 | `release.yml` 195〜215 行の検査（リリースの build の中だけ） | **Constraint**: この検査はリリースのときにしか走らない。手元で同じ検査をする道具は無い（`Get-ZipEntries` は `release.yml` の中のインラインの関数） |
| 2.2・2.3 3.9.2 と同じファイル一覧 | 無い | **Missing**: 3.9.2 と 4.1.1 で同じソースから VSIX を作り、zip の項目の一覧を比べる手順。`.vscodeignore` の否定 `!wasm/*.d.ts` と、windows で大文字小文字を区別しない照合になる点が差の候補 |
| 2.5 `vscode-test` | `build.yml` 205 行 | 無し（lock を変えれば PR で走る） |
| 2.6 Node 22 | 3 つのワークフローとも 22 | 無し |
| 3.1〜3.6 補助スクリプトの振る舞い | `publish-vsix.ps1` | 調べた範囲では直す所が無い。**Research Needed**: `show --json` の無い拡張の exit code を 4.x で実測 |
| 3.7・3.8 項目 12 | `release-scripts-test.ps1` | 無し（偽の vsce の答えは 4.x の `show --json` と同じ形） |
| 3.9 SHA の固定 | `ponytail-manual-ci` の形 | 無し（`publish-vsce` の job を触らなければ保たれる） |
| 4.1〜4.4 公開の前の確かめ | `release-setup-check.yml` | **Constraint**: main からしか走らない。ブランチで 4.x を入れても統合の前には確かめられない。**Missing**: 「確かめるまでリリースタグを push しない」を守らせる仕組み（今は人の手順だけ。`release` スキルの手順に 1 行足すか、この spec の完了の手順で走らせて記録するか） |
| 4.6 手元の `show` | 開発機の Node 26・`NODE_OPTIONS=--dns-result-order=ipv4first` の知見 | 無し（`npx --no-install vsce show` を `editors/vscode` で） |
| 5.1〜5.4 だめなときの扱い | 3.9.2 へ戻すのは lock を戻すだけ（可逆） | **Unknown**: `--oidc` へ移す道は Marketplace 側が使えるか未確認。移すなら `release.yml`・`release-setup-check.yml`・`publish-vsix.ps1`・`release` スキル・`tech.md`・`release-ci-setup.md`・Azure 側の資格情報の後始末まで広がる |
| 6.1〜6.3 文書 | `publish-vsix.ps1` のコメント「vsce の publish.js の getPAT」 | 4.x でも `publish.ts` を js にしたものの `getPAT` なので正しい。版の数字を書いた文書は見つからなかった。ロードマップの `vsce-4-upgrade` の項（10・206・343・350・352 行）は完了の時に動かす |

### 穴のまとめ

- **VSIX のビルドは、リリースの build でしか CI で 4.x の `vsce package` が動かない。** `vscode-test` は `npm ci` と試験だけで、パッケージは作らない。手元で作って比べなければ、ファイルの落ちは次のリリースの build の中身の検査で初めて分かる（必須の 3 つ以外の落ちはそこでも分からない）
- **公開の道は main へ入った後にしか確かめられない。** `verify-pat` と `publish` は同じ `getPAT`（Entra ID の取得・`auth.ts` は 3.9.1 と同じ）を通る。違いは呼ぶ Marketplace の API（`getRoleAssignments` と公開）だけ。2 つの確かめで、トークンの取得と publisher の権限までは確かめられ、公開の API の受け付けは確かめられない
- **4.x の新しい依存の勧告と、`npm ci --ignore-scripts` での keyring の読み込み**は未確認

## 3. 進め方の候補

### 案 A: 版を上げるだけ（今の道を延ばす）

- `package.json` 1 行・lock の作り直し。`publish-vsix.ps1`・項目 12・ワークフローは触らない
- 手元で 3.9.2 と 4.1.1 の VSIX のファイル一覧を比べ、`show --json` の振る舞いを実測する（記録は spec に）
- マージの後に `release-setup-check.yml` を main で走らせ、結果を記録する
- ✅ 差分が最小（2 ファイル）。戻すのも lock を戻すだけ
- ✅ `release.yml` を触らないので、リリースの席・`pasta-dll-multiarch`・`pasta-recipe` とぶつからない
- ❌ 公開の API の受け付けは次のリリースまで分からない（2 つの確かめで代える前提）

### 案 B: 案 A ＋ 確かめを強める（`release-setup-check.yml` に足す）

- `release-setup-check.yml` に `publish-vsix.ps1 -DryRun` の step を足す（`show` での判定と、行うはずの `publish` のコマンドを、lock の vsce で通して見せる）。あるいは `vsce package` を確かめのワークフローか `vscode-test` に足して、パッケージの道も CI で毎回通す
- ✅ リリースの前に、補助スクリプトの判定の道まで 4.x で通る
- ❌ `-DryRun` は認証も公開の API も通らないので、確かめが増える割に分かることは少ない。`vsce package` を CI に足すと WASM のビルド（wasm-pack）が要り、`vscode-test` が重くなる
- ❌ `release-setup-check.yml` を触る（リリースの席）

### 案 C: OIDC の口（`--oidc`）へ移す

- `publish-vsix.ps1` の `--azure-credential` を `--oidc` に替え、`publish-vsce` の Azure/login を外し、Marketplace 側に trusted publishing の設定を足す。`release-setup-check.yml` の確かめも替える（`verify-pat` に `--oidc` は無いので、確かめの形から作り直し）
- ✅ Azure のマネージド ID・サブスクリプション（従量課金）・フェデレーション資格情報が要らなくなる
- ❌ Marketplace 側が使えるか未確認（10/5 時点で試験中）。PR #1337 は「公開を通しで試していない」と書く
- ❌ 公開の前に確かめる道が無い（`verify-pat` が `--oidc` を持たない）。触るファイルが 6 つ以上に広がる
- brief の Approach どおり、`--azure-credential` の確かめがだめだったときの代わりとして比べる位置づけ

## 4. 規模と危うさ

- **規模: S**（1〜3 日）。決まった道の版の更新で、直すファイルは 2 つ（案 A）。比べる手順と記録が作業の大半
- **危うさ: Medium**。道具の振る舞いは調べた範囲で変わらないが、公開の道は main へ入った後・本当には公開の job でしか確かめられない。戻すのは lock を戻すだけで可逆

## 5. 設計への申し送り

- 推奨の候補: 案 A。4.x で CLI の口と `publish-vsix.ps1` が頼る振る舞いが変わっていないので、補助スクリプトとワークフローは触らない。確かめは手元の比べ（VSIX の一覧・`show`）＋ マージの後の `release-setup-check.yml`。だめなら lock を戻し、案 C を比べる
- 要件ディスカッションで決めたこと（2026-11-14）: 公開の確かめは 2 つの確かめで代える／版を上げる変更だけを実装の途中に単独の PR で main へ入れ、`release-setup-check.yml` を回して結果をこのファイルに記録してから完了する／だめなら先に lock を戻し、同じ spec で直すか `--oidc` を比べ、どちらも無理なら spec ごと見送る／版の指定は `^4.1.1`／開発機の大域の `vsce` は範囲外
- **Research Needed**（設計で）:
  1. v4.1.1 のタグのソースで `show.ts`・`publish.ts`・`main.ts` を読み直す（今回は main で代用）
  2. 4.x で無い拡張に `show --json` したときの終了コードを手元で実測する
  3. ubuntu・`npm ci --ignore-scripts` の後に `vsce publish`・`verify-pat` が `@napi-rs/keyring` の読み込みで落ちないか（PAT を保存しない道で keyring に触れるか）
  4. 4.x の lock の `npm audit` の結果（新しく増える推移的な依存の勧告）
  5. windows で 4.x の照合が大文字小文字を区別しなくなることが、`.vscodeignore` の行（`**/*.ts`・`!wasm/*.d.ts`・`wasm/README.md` など）の結果を変えないか（VSIX の一覧の比べで確かめる）
  6. Marketplace の trusted publishing（`--oidc`）が今使えるか（案 C を比べるときだけ）
  - → 1〜5 は設計フェーズで閉じた（6 節）。6 は調べていない（要件 5.2 の分岐に入ったときだけ）

---

## 6. 設計フェーズの調べ（2026-11-14）

### Summary

- **Feature**: `vsce-4-upgrade`
- **Discovery Scope**: Extension（決まった道の道具の版の更新。light discovery）
- **使ったスキル・規則**: `kiro-spec-design`（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）、`kiro-validate-design`（`design-review.md`）、ponytail（変更を最小にする方針）。並列のサブエージェントは使わなかった（調べる所が狭く、手元で測るほうが確か）
- **Key Findings**:
  1. 5 節の Research Needed 1〜5 は手元で測って閉じた。`publish-vsix.ps1` が頼る振る舞い（`getPAT` の順・`VSCE_PAT` の既定・`--skip-duplicate` の文言・無い拡張の `show --json` が `undefined` で 0 終了・`auth.js`・`show.js`）は 4.1.1 の配布物でも 3.9.2 と同じ。VSIX のファイルの一覧・`extension.vsixmanifest` も同じ（仮の WASM で）
  2. **要件 1.3 の半分は 4.1.1 では満たせない。** `glob` GHSA-3g3m-2fjr-6pw4 は消えるが、`minimatch` GHSA-g3hv-75wg-m249 は残る。直った版は `minimatch` 11.0.1 以上だけで、vsce 4.1.1 自身が `minimatch ^10.2.6` を要り、`eslint` 10.14.0・`typescript-eslint` 8.72.2 も `^10` を要るため
  3. lock の差分は vsce の配下だけ（消える 174・増える 17・版が変わる 3 = `@vscode/vsce`・`parse5`・`entities`）。要件 1.6 は `npm install --save-dev @vscode/vsce@^4.1.1` 1 回で満たせる

### Research Log

#### 4.1.1 の配布物のソース（Research Needed 1）

- **Context**: ギャップ分析は 4.x を main のソースで読んでいた
- **Sources Consulted**: npm から入れた `@vscode/vsce@4.1.1` の `out/*.js`（公開された物そのもの。タグの ts を js にしたもの）と、同じく `3.9.2` の `out/*.js`
- **Findings**:
  - `publish.js` の `getPAT` の順は 4.1.1 で `oidc → pat → azureCredential → 保存済み`、3.9.2 で `pat → azureCredential → 保存済み`。`--oidc` を付けない限り同じ
  - `main.js` の `publish`・`verify-pat` の `-p, --pat` の既定は 4.1.1 も `process.env['VSCE_PAT']`。`--azure-credential` は `publish`・`unpublish`・`verify-pat` に残る
  - `publish.js` の重複の文言は `Version ${version} is already published. Skipping publish.`（2 か所）で 3.9.2 と同じ
  - `auth.js` と `show.js` は 3.9.2 と 4.1.1 で同じ内容（`diff` で差なし）。認証の失敗の文言 `Can not acquire a Microsoft Entra ID access token` は `publish-vsix.ps1` の `$AuthPattern` の `Entra ID access token` に合う
  - `verify-pat` の失敗の文言 `The Personal Access Token verification has failed` は 4.1.1 も同じで、`$AuthPattern` の `Personal Access Token` に合う
- **Implications**: `publish-vsix.ps1`・`release-scripts-test.ps1` の項目 12・`release-setup-check.yml` は直さない（要件 3.8 の「変わっていたら合わせる」は発動しない）

#### 無い拡張の `show --json`（Research Needed 2）

- **Context**: `publish-vsix.ps1` の `Get-Versions` は、拡張が無いと `undefined` が出て 0 で終わる前提で、`ConvertFrom-Json` の失敗を transient に落とす
- **測った物**: 手元（Windows・Node 26.12.1・npm 12.2.1・`NODE_OPTIONS=--dns-result-order=ipv4first`）で、使い捨ての写し 2 つ（3.9.2 の lock と、4.1.1 に上げた lock）から
  - `vsce show ekicyou.pasta-vscode --json`: どちらも exit 0・`versions` 42 件（最新 0.5.2）・キーの並びも同じ
  - `vsce show ekicyou.pasta-vscode-nonexistent-zz --json`: どちらも exit 0・標準出力 `undefined`・標準エラーなし
- **Implications**: 要件 3.4 は今の `publish-vsix.ps1` のまま満たす。要件 4.6 の手元の `show` は 2026-11-14 に一度通した（実装でもう一度取る）

#### `npm ci --ignore-scripts` と `@napi-rs/keyring`（Research Needed 3）

- **Findings**:
  - lock の `@napi-rs/keyring` とプラットフォーム別の 13 個は、どれも `hasInstallScript` が無い。プラットフォーム別の物は `optional`・`os`/`cpu` 付きの作り済みのバイナリで、`--ignore-scripts` でも入る（Windows で `keyring-win32-x64-msvc` が入ったのを確かめた。ubuntu では `keyring-linux-x64-gnu` が選ばれる）
  - 4.1.1 は `@napi-rs/keyring/keytar.js` を `store.js` の `KeytarStore.open` の既定引数の中でだけ `require` する（遅延読み込み）。`--azure-credential` の道は `getPAT` が保存済みの資格情報に届く前に返るので、keyring を読まない
- **Implications**: `publish-vsce`・`release-setup-check.yml` の `npm ci --ignore-scripts` は直さない。ubuntu での読み込みは CI で初めて分かるが、読まない道なので危うさは低い

#### `npm audit`（Research Needed 4）

| | 3.9.2 の lock | 4.1.1 の lock |
|---|---|---|
| 件数 | moderate 11 | moderate 10 |
| `glob` GHSA-3g3m-2fjr-6pw4 | 在る（`glob` 13.0.7、vsce の配下） | **無い**（`glob` が lock から消えた） |
| `minimatch` GHSA-g3hv-75wg-m249（`>=7.0.0 <11.0.1`） | 在る（`minimatch` 10.2.6。vsce・`glob`・`eslint`・`@eslint/config-array`・`@typescript-eslint/typescript-estree` から） | **在る**（`minimatch` 10.2.6。vsce 4.1.1 は `minimatch ^10.2.6` を要る。`eslint`・`typescript-eslint` も同じ） |
| 新しく増えた勧告 | — | 無い（`@napi-rs/keyring`・`marked` 18.2.0・`parse5` 8.1.1・`proper-lockfile` 4.1.2 などに勧告なし） |

- 残りの 10 件は全部 `minimatch` の 1 件から辿ったもの（`eslint`・`typescript-eslint` の系統と vsce 自身）
- `minimatch` の 10 系に直った版は無い（`latest` は 11.0.8。`legacy-v10` の tag も無い）
- **Implications**: 要件 1.3 の `minimatch` の半分は vsce を上げても満たせない。消すには `minimatch` 11 への `overrides` が要り、vsce・`eslint`・`typescript-eslint` の 3 つの道具の振る舞いを確かめる手間が増える（要件 1.6 の「vsce 以外の直接の依存を動かさない」の精神にも反する）。設計では「`glob` は消える・`minimatch` は残り、理由を記録する」と仮に置き、未決事項 Q1 としてディスカッションへ回す。brief の「4.x でしか消えない」は `glob` には正しく、`minimatch` には正しくなかった

#### Windows での VSIX のファイルの一覧（Research Needed 5・要件 2.2 の下見）

- **測った物**: 使い捨ての写しに `editors/vscode` の git の管理下のファイル＋手元の `out/`＋`wasm/`（`pasta_lsp_wasm_bg.wasm` と `pasta_lsp_wasm_bg.wasm.d.ts` は仮の中身、`wasm/package.json` は `.vscodeignore` の確かめのために仮に置いた）を置き、3.9.2 と 4.1.1 で `vsce ls --no-dependencies` と `vsce package --no-dependencies` をした
- **Findings**:
  - `vsce ls` の 16 件、VSIX の zip の項目 18 件（`[Content_Types].xml`・`extension.vsixmanifest`＋`extension/` の 16 件）、`extension.vsixmanifest` の中身が、3.9.2 と 4.1.1 で全部同じ。`!wasm/*.d.ts`・`wasm/README.md`・`wasm/package.json`・`out/test/**`・`**/*.map` の照合の結果も同じ（Windows の大文字小文字を区別しない照合は差を生まなかった）
  - 必須の 3 つ（`extension/package.json`・`extension/wasm/pasta_lsp_wasm_bg.wasm`・`extension/wasm/THIRD_PARTY_LICENSES.txt`）は入る。名前は `pasta-vscode-0.5.2.vsix`
  - 気づいたこと（この spec の外）: `eslint.config.mjs` が VSIX に入っている（3.9.2 から同じ。`.vscodeignore` に無い）
- **手元の落とし穴**: npm 12 は UUID に見える文字列を出力で `***` に伏せる。スクラッチパッドのパスには UUID が入るので、`vsce` が中で叩く `npm list --production --parseable` の出力のパスが壊れ、依存を集める道（`--no-dependencies` を付けない `vsce ls`・`vsce package`）がファイルを 0 件と見て、`package` は「entrypoint missing」で落ちる（3.9.2 も 4.1.1 も同じ）。拡張の `package.json` には `dependencies` が無いので、`--no-dependencies` を付けても VSIX の中身は同じ。worktree のパス（UUID を含まない）で作るなら起きない
- **Implications**: 下見では差は無い。ただし本物の WASM でなく、`npm run package` の道そのものでもないので、要件 2.2 の比べは実装で本物の WASM を作ってもう一度行う

#### Marketplace の trusted publishing（Research Needed 6）

- 調べなかった（案 C は `verify-pat` がだめだったときだけ比べる。要件 5.2 の分岐に入ったら調べる）

#### 前の確かめの実績（gh の照会・読むだけ）

- `release-setup-check.yml` の最後の実行は 2026-10-08（run 37777674500・success・Node 20 のころ）。ログに `The Personal Access Token verification succeeded for the publisher 'ekicyou'.` が在る
- `release.yml` の v0.5.2（2026-11-07・run 42354018129）の `publish-vsce` は success（Node 22・vsce 3.9.2）
- `release-setup-check.yml` の `verify-pat` の step は `continue-on-error: true` なので、run が緑でも `verify-pat` が通ったとは限らない。判定は job summary の行（とログの文言）で読む

### 測らなかったもの

- 本物の WASM（`build-wasm.ps1 -Release`）での VSIX の比べ（実装の task。wasm-pack・cargo-about が要る）
- `npm run package` を通しての VSIX の作成（`build:wasm` を含むため）
- ubuntu での `npm ci --ignore-scripts` の後の vsce の起動（`vscode-test` と `release-setup-check.yml` が main で確かめる）
- `release-scripts-test.ps1`（偽の vsce を使い vsce の版に関係しないので、実装で走らせる）
- `verify-pat`・`publish`・`login`（資格情報が要る・公開につながる。手元では叩かない）
- Marketplace の trusted publishing が今使えるか

## 7. 合成（design-synthesis）

- **一般化**: 要件 2・3・4 はどれも「3.9.2 で頼っていた振る舞いが 4.1.1 で変わらないことを、測って記録する」の変種。1 つの照合表（design.md の「vsce の CLI の契約」）にまとめ、手元で測れる物は手元で、main でしか測れない物は `release-setup-check.yml` で埋める
- **作るか採るか**: 確かめは全部、今ある道具を使う（`release-setup-check.yml`・`release.yml` の build の中身の検査・`build.yml` の `vscode-test`・`release-scripts-test.ps1`）。新しいスクリプト・step・ワークフローは作らない。VSIX の比べは一回限りの手作業で、結果を記録するだけ（繰り返す道具にしない）
- **単純化**: ギャップ分析の案 B（`release-setup-check.yml` に `-DryRun` や `vsce package` を足す）は採らない。分かることが少なく、リリースの席を触ることになる。公開の補助スクリプトとワークフローは、測った振る舞いが変わっていないので触らない

## 8. 設計の決定

### Decision: 案 A（版を上げるだけ）

- **Context**: 要件 1〜3 と、要件ディスカッション（2026-11-14）の合意
- **Alternatives Considered**: 案 A（版だけ）・案 B（確かめを強める）・案 C（`--oidc` へ移す）。3 節を参照
- **Selected Approach**: `editors/vscode/package.json` の 1 行（`"@vscode/vsce": "^4.1.1"`）と lock だけを変える
- **Rationale**: 4.1.1 の配布物で、`publish-vsix.ps1` が頼る振る舞いが全部同じだと測れた
- **Trade-offs**: 公開の API の受け付けは次のリリースの `publish-vsce` まで分からない（要件 4.3 でそれを受け入れた）
- **Follow-up**: 本物の WASM での VSIX の比べ、main での `release-setup-check.yml`

### Decision: 版を上げる PR は spec の文書を含まない別のブランチから

- **Context**: 要件 4.1 は「版を上げる変更だけを単独の PR で」と言う。spec のブランチには spec の文書のコミットが載っている
- **Selected Approach**: `origin/main` から作業用のブランチを切り、`package.json`・`package-lock.json` の 2 つだけをコミットして PR → squash マージ。その後 spec のブランチへ `git merge origin/main`
- **Trade-offs**: ブランチが 1 本増える。spec の文書を main へ先に入れずに済む

### Decision: 3.9.2 の基準の確かめを、版を上げる PR のマージの前に取る

- **Context**: 要件 5.1・5.4 は「同じ設定の 3.9.2 では成功していたか」で分岐する。最後の `release-setup-check.yml` は 2026-10-08（Node 20）で、今の main（Node 22・3.9.2）では走らせていない
- **Selected Approach**: マージの直前に main で一度 `release-setup-check.yml` を回して基準にする。v0.5.2 の `publish-vsce` の成功（2026-11-07）を代わりの基準にもできる
- **Trade-offs**: dispatch が 1 回増える（公開なし・30 秒ほど）
- **Decided**: 2026-11-14 の設計ディスカッションで推奨案（回す）に決めた。落ちたときに版のせいか設定のせいかを見分けるため

### Decision: 要件 1.3 の `minimatch` は残し、理由を記録する

- **Context**: 測った結果、4.1.1 では消えない
- **Selected Approach**: `glob` が消えることを確かめ、`minimatch` は「消せない勧告」として理由（直った版が 11 系だけ・vsce 4.1.1・`eslint`・`typescript-eslint` が `^10` を要る）を記録する。`overrides` は使わない
- **Decided**: 2026-11-14 の設計ディスカッションで要件 1.3 の文言を直した。配る VSIX には入らない道具の中の DoS で、4.x にする主な目的（保守の続く版・`glob` の解消）は果たせるため

## 9. Risks & Mitigations

- 4.1.1 の `verify-pat --azure-credential` が main で失敗する — 先に lock を戻す PR を入れ（要件 5.1）、同じ spec で直すか `--oidc` を比べる。戻すのは squash のコミットの revert 1 つ
- マージから確かめまでの間に誰かがリリースタグを push する — マージの直後に同じセッションで確かめを回す（数分）。`release` スキルは直さない（`.claude/skills` の書き換えは分類器が止める）。Q2
- 本物の WASM での VSIX に、下見で出なかった差が出る — 要件 2.3 に沿って差の 1 つずつを確かめ、`.vscodeignore` で直す
- `publish-vsce` の公開の API の受け付けだけが次のリリースまで分からない — 要件 4.3 の合意どおり受け入れる。`publish-vsix.ps1` の冪等な判定で、失敗の後の再実行で続きから回復する
- `minimatch` の勧告が残る — 配る VSIX には入らない、パッケージを作る道具の中の DoS（moderate）。理由を記録する

## 10. References

- npm の `@vscode/vsce@4.1.1`・`@vscode/vsce@3.9.2` の配布物（`out/publish.js`・`out/main.js`・`out/auth.js`・`out/show.js`・`out/store.js`）
- [GHSA-g3hv-75wg-m249](https://github.com/advisories/GHSA-g3hv-75wg-m249) — `minimatch` `>=7.0.0 <11.0.1`
- [GHSA-3g3m-2fjr-6pw4](https://github.com/advisories/GHSA-3g3m-2fjr-6pw4) — `glob` `>=9.0.0 <=14.0.1`
- 手元の測りの記録（スクラッチパッド。コミットしない）: `audit-v3.json`・`audit-v4.json`・`ls-v3.txt`・`ls-v4.txt`・`zip-v3.txt`・`zip-v4.txt`・`show-v3.json`・`show-v4.json`

## 11. 公開の道の確かめの記録（実装で埋める・要件 4.3）

| 項目 | 値 |
|------|----|
| 基準の run（3.9.2・マージの前） | （未） |
| 版を上げる PR | （未） |
| 確かめの run（4.1.1・マージの後） | （未） |
| 走らせた日 | （未） |
| lock の vsce の版 | （未） |
| `verify-pat ekicyou --azure-credential` | （未） |
| `show ekicyou.pasta-vscode --json` | （未） |
| 判断 | （未） |
