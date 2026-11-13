# ギャップ分析: vsce-4-upgrade

- 日付: 2026-11-14（worktree の base `c8ec6331`）
- 対象: `requirements.md`（要件 1〜6）
- 外部の調べ: `@vscode/vsce` の GitHub のリリース 2 ページ・main のソース（`main.ts`・`publish.ts`・`oidc.ts`・`auth.ts`・`show.ts`・`package.ts`・`store.ts`）・v3.9.1 のタグのソース（`main.ts`・`publish.ts`・`auth.ts`・`show.ts`）・PR #1291・#1305・#1308・#1309・#1337・#1345・npm の registry（`latest` = 4.1.1・`engines.node` `>= 22`）。4.x のソースは v4.1.1 のタグでなく main で読んだ（4.1.1 は 11/12 の版で、その後の main の変更が混ざりうる。**Research Needed**: 設計で v4.1.1 のタグで読み直す）

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
