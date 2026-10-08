# リリース CI の一回限りのセットアップ

リリース CI（`.github/workflows/release.yml`）が crates.io と VSCode Marketplace へ認証できるようにするための設定の手順書。節の順が実施の順になっている。

## 1. この手順書の位置づけ

- ここに書く設定は、リリース CI のワークフローの外で **1 度だけ** 行う。リリースのたびに実行するものではない（毎回のリリース手順は `crates/pasta_sample_ghost/RELEASE.md`）。
- 設定をやり直すとき（リポジトリの改名・移管、マネージド ID の作り直し、新しいクレートの追加など）にも、この手順書を使う。
- 設定に入れる名前（リポジトリ・ワークフローのファイル名・environment の名前・クレート名）は、末尾の「12. 名前の対応表」にまとめてある（正本は design の「認証名の契約」で、この表はその写し）。各公開先の設定画面には、表の値をそのまま写す。
- **ID の値はリポジトリに書かない。** このリポジトリは公開されている。Azure の client・tenant・subscription の ID は GitHub のリポジトリ variables（`AZURE_CLIENT_ID`・`AZURE_TENANT_ID`・`AZURE_SUBSCRIPTION_ID`）にだけ置き、Marketplace の profile ID も含めて、ファイル・Issue・PR・コミットメッセージに書かない。本書の `<クライアント ID>` などは、各自の値に読み替える。
- GitHub の secrets は使わない。crates.io は Trusted Publishing（OIDC から 30 分のトークン）、Marketplace は Microsoft Entra ID のワークロード ID 連携（OIDC からアクセストークン）で認証し、長期の認証情報をどこにも置かない。

全体の流れ:

1. Azure でサブスクリプション・予算アラート・マネージド ID を用意する（2 節）
2. マネージド ID に GitHub 用のフェデレーション資格情報を 2 件作る（3 節）
3. GitHub に environment 2 つと variables 3 つを登録する（4 節）
4. ワークフローを main へ入れ、確認ワークフローで profile ID を得て Marketplace の Members に追加する（5〜7 節）
5. crates.io の 5 クレートに Trusted Publisher を設定する（8 節）
6. 初回のリリースを行い、成功したら必須の後始末をする（9・10 節）

## 2. Azure: サブスクリプション・予算アラート・マネージド ID

Marketplace の publisher `ekicyou` の Owner と同じ Microsoft アカウントで Azure にサインアップする。

### 従量課金のサブスクリプションを用意する

- サブスクリプションは従量課金（「Azure プラン」）を使う。
- マネージド ID とフェデレーション資格情報には課金が無いので、実際の請求は ¥0 の見込みである。それでもサブスクリプションは要る（マネージド ID はサブスクリプションのリソースグループに属し、`Azure/login` はサブスクリプションへのログインを既定で求める）。

### 無料試用版・Visual Studio のサブスクリプションを使わない理由

- **無料試用版のままにしない。** 無料試用版のサブスクリプションは期限が切れると無効になり、その中のマネージド ID も使えなくなる。そうなると、ある日のリリースから Marketplace への公開が認証の失敗で止まる。試用版から始めた場合は、必ず従量課金へ切り替え、次の予算アラートを設定する。従量課金への切り替えと予算アラートは、どちらも省けない。
- **Visual Studio のサブスクリプション（MSDN 特典）も使わない。** 同じテナントにあっても、Visual Studio の契約が切れるとサブスクリプションも無効になる。また、規約で用途が開発・テストに限られ、公開の運用には向かない。

### 予算アラートを設定する

想定外の課金に早く気づくため、¥0 に近い額で予算アラートを設定する。

1. Azure ポータルの「コストの管理」→「予算」で、サブスクリプションに予算を追加する。
2. 名前 `budget-pasta`、金額 ¥100／月にする。
3. アラートの条件を「実績が 1%」にし、通知先に自分のメールアドレスを入れる。
4. 有効期限は遠く（例: 2036-12-31）にしておく。期限を延長する手間を省ける。

### リソースグループとマネージド ID を作る

1. リソースグループ `rg-pasta-release` を作る（リージョンは Japan East）。
2. `rg-pasta-release` に、ユーザー割り当てのマネージド ID `id-pasta-release` を作る。
3. `rg-pasta-release` の「アクセス制御 (IAM)」で、`id-pasta-release` に「閲覧者」のロールを割り当てる。マネージド ID がどのサブスクリプションにもロールを持たないと、`Azure/login` がログインするサブスクリプションを見つけられない。
4. `id-pasta-release` の「概要」で、次の 3 つの値を控える（4 節で GitHub の variables に登録する。リポジトリには書かない）。
   - クライアント ID（`AZURE_CLIENT_ID` に入れる。「オブジェクト (プリンシパル) ID」と取り違えない）
   - テナント ID（`AZURE_TENANT_ID`。Entra ID の「概要」や `az account show --query tenantId` でも確かめられる）
   - サブスクリプション ID（`AZURE_SUBSCRIPTION_ID`）

### サービスプリンシパル（アプリ登録）を使わない理由

Entra ID のアプリ登録（サービスプリンシパル）なら、サブスクリプションなしでもフェデレーション資格情報を作れる。しかしこの方法は採らない。サービスプリンシパルで `vsce verify-pat` は通るのに、`vsce publish` が「corporate credentials」のエラーで失敗する報告があり（microsoft/vscode-vsce#1023）、原因が示されないまま not_planned で閉じられているためである。Microsoft の文書が示すマネージド ID の経路を使う。

## 3. Azure: フェデレーション資格情報を 2 件作る

`id-pasta-release` に、GitHub Actions 用のフェデレーション資格情報を 2 件作る。公開する job（environment `release`）と確認ワークフロー（environment `release-setup-check`）とで OIDC の subject が違うためである。

マネージド ID のフェデレーション資格情報は subject の完全一致でしか照合しない（ワイルドカードは使えない）。environment を付けない job ではタグごとに subject が変わってしまうので、リリース CI は公開する job に必ず environment を付けている。

Azure ポータルで `id-pasta-release` →「設定」→「フェデレーション資格情報」→「資格情報の追加」を開き、シナリオに「GitHub Actions による Azure リソースのデプロイ」を選んで、次の 2 件を作る。資格情報の名前は自由に付けてよい（必須の欄）。

| 欄 | 1 件目 | 2 件目 |
|----|--------|--------|
| 組織 | `ekicyou` | `ekicyou` |
| リポジトリ | `pasta` | `pasta` |
| エンティティ | 環境 | 環境 |
| 環境名 | `release` | `release-setup-check` |
| subject（自動で入る） | `repo:ekicyou/pasta:environment:release` | `repo:ekicyou/pasta:environment:release-setup-check` |
| issuer（自動で入る） | `https://token.actions.githubusercontent.com` | `https://token.actions.githubusercontent.com` |
| 対象ユーザー（audience） | `api://AzureADTokenExchange`（既定のまま） | `api://AzureADTokenExchange`（既定のまま） |

Azure CLI で作るなら次のとおり（2 件目は `--subject` の environment 名を `release-setup-check` にする）。自分のアカウントで作成系のコマンドを使うには MFA でのログインが要る。同じテナントに別のサブスクリプションがあるので、`--subscription` で 2 節のサブスクリプションを明示する。

```bash
az identity federated-credential create \
  --subscription <サブスクリプション ID> \
  --name <資格情報の名前> \
  --identity-name id-pasta-release \
  --resource-group rg-pasta-release \
  --issuer https://token.actions.githubusercontent.com \
  --subject repo:ekicyou/pasta:environment:release \
  --audiences api://AzureADTokenExchange
```

## 4. GitHub: environment と variables

リポジトリ `ekicyou/pasta` の Settings で、environment を 2 つと、リポジトリ variables を 3 つ登録する。

### environment を 2 つ作る

Settings →「Environments」→「New environment」で作り、「Deployment branches and tags」を「Selected branches and tags」にして、実行できる ref を限る。

| environment | 許可する ref | 使うもの |
|-------------|--------------|----------|
| `release` | タグ `v*`（「Add deployment branch or tag rule」で Ref type を Tag にする） | `release.yml` の publish-crates・publish-vsce |
| `release-setup-check` | ブランチ `main` | `release-setup-check.yml` |

- `release` をタグからの実行に限ることで、crates.io のトークンを得られるのはリリースタグの push で動く実行だけになる。Marketplace のアクセストークンは、同じマネージド ID の資格情報を持つ確認ワークフロー（main からの実行に限る）でも得られる。そのため、main を PR 経由でしか変えられないこと（ブランチ保護）が前提になる。
- environment の保護規則は、GitHub Free では public リポジトリでだけ使える。
- environment の secrets と variables は使わない（ID は次のリポジトリ variables に 1 組だけ置く）。

### リポジトリ variables を 3 つ登録する

Settings →「Secrets and variables」→「Actions」→「Variables」タブ →「New repository variable」で、2 節で控えた値を登録する。

| 名前 | 値 |
|------|----|
| `AZURE_CLIENT_ID` | `<クライアント ID>`（`id-pasta-release` のクライアント ID） |
| `AZURE_TENANT_ID` | `<テナント ID>` |
| `AZURE_SUBSCRIPTION_ID` | `<サブスクリプション ID>` |

secrets ではなく variables に置く（秘密の値ではないため）。ただし公開リポジトリなので、値をファイルに書き写さない。

## 5. ワークフローを main へ入れる

`release.yml` と `release-setup-check.yml` を含む PR を main へマージする。

`release-setup-check.yml` は手動起動（`workflow_dispatch`）だけのワークフローで、手動起動できるのは既定のブランチ（main）にあるワークフローだけである。そのため、6・7 節の Marketplace の設定は、初回のリリースより前に 1 度 main へマージしてからになる。

## 6. 確認ワークフローを実行する（1 回目: profile ID を得る）

確認ワークフロー「Release setup check」（`release-setup-check.yml`）は、**どの公開先にも公開せずに**、セットアップの配線を確かめる。

- Azure へ OIDC でログインできるか（3 節の資格情報・4 節の environment と variables）
- マネージド ID の Marketplace の profile ID と表示名（7 節で Members に追加する値）
- publisher `ekicyou` のメンバーになっているか（`vsce verify-pat ekicyou --azure-credential`）
- 拡張 `ekicyou.pasta-vscode` が Marketplace にあるか（認証なしの `vsce show`）
- 名前の対応表（設定画面と目で照合するため）

crates.io のトークン交換は行わない（確認のために公開できる認証を発行しない）。crates.io 側の設定は、8 節の設定画面の値を名前の対応表と照合することと、9 節の初回のリリースで確かめる。

手順:

1. GitHub の「Actions」→「Release setup check」→「Run workflow」で、ブランチ `main` を選んで実行する。
2. 実行の Summary を開き、「Marketplace の profile（マネージド ID）」の表の **profile ID** を控える。profile ID は秘密ではないが、ログには出さず summary にだけ書いている。リポジトリには書かない。
3. この時点では「publisher `ekicyou` のメンバー確認」が失敗する（まだ Members に追加していないため。想定どおり）。この step は失敗しても止まらない設定なので、後続の確認は続き、Summary にも結果が書かれる。

`Azure login (OIDC)` の step で失敗したら、3 節の subject（`repo:ekicyou/pasta:environment:release-setup-check`）、4 節の environment 名と variables を見直す。

## 7. Marketplace: マネージド ID を Members に追加する

1. Marketplace の publisher 管理ページ（Manage Publishers & Extensions）で publisher `ekicyou` を開く。
2. 「Members」で「Add」を選び、6 節で控えた profile ID を入れ、ロールを **Contributor** にして追加する。
3. 確認ワークフロー「Release setup check」をもう 1 度実行し、Summary の「publisher `ekicyou` のメンバー確認」が「成功（Members に追加済み）」になることを確かめる。これで Marketplace への公開の配線が通っている。

## 8. crates.io: Trusted Publisher を 5 クレートに設定する

公開対象の 5 クレートそれぞれで、crates.io のクレートの設定画面（Settings →「Trusted Publishing」）から GitHub の Trusted Publisher を追加する。初回のリリースより前ならいつ行ってもよい。

| クレート（公開順） |
|--------------------|
| `pasta_core` |
| `pasta_dsl` |
| `pasta_lua` |
| `pasta_shiori` |
| `pasta_check` |

5 件とも、次の値を入れる。

| 欄 | 値 |
|----|----|
| Repository owner | `ekicyou` |
| Repository name | `pasta` |
| Workflow filename | `release.yml`（パスを付けず、ファイル名だけ。完全一致で照合される） |
| Environment | `release` |

- environment を入れたので、environment `release` 以外からの実行ではトークンを得られない。
- owner は数値の ID で照合される。リポジトリを改名・移管したら、5 件とも作り直す。
- 設定が合っているかは、公開しないと確かめられない。確認ワークフローの Summary の名前の対応表と、5 件の設定画面の値を目で照合しておく。

## 9. 初回のリリース

2〜8 節がすべて済んだら、初回のリリースを行う。

1. いつものリリースの手順（`release-workflow` の手順）で版を上げ、そのコミットを main へ入れて、リリースタグ `vX.Y.Z` を push する。
2. GitHub の「Actions」→「Release」の実行を開き、report job の Summary を見る。crates.io の 5 クレート・Marketplace・GitHub Release がすべて `published` になり、GitHub Release の URL が出ていれば成功である。
3. publish-crates の Summary で、クレートごとの所要秒を確かめる（トークンの期限 30 分に対する余裕の判断材料）。
4. 同じ実行を再実行（Re-run all jobs）し、すべての公開先が `skipped`（公開済みのため飛ばした）になることを確かめる。

認証で失敗したとき（`reason` が `auth`）は、12 節の名前の対応表と各公開先の設定を照合する。crates.io なら 8 節の Trusted Publisher、Marketplace なら 3 節のフェデレーション資格情報と 7 節の Members である。crates.io で auth の step は通ったのに、そのクレートの公開が権限のエラー（403）で失敗した場合も、そのクレートの 8 節の Trusted Publisher の設定を見直す。

### auth の取り直しが拒否されたとき（落とし先）

publish-crates は、トークンの期限（30 分）に当たらないよう、クレートごとに `rust-lang/crates-io-auth-action` を呼び直している。ただし、同じ job で何度も呼べることは公式の文書に書かれていない。

初回のリリースで、`pasta_core` が `published` か `skipped` になったのに、2 つ目以降の `Auth crates.io (...)` の step が失敗した（そのクレートが `failed`・`auth`）場合は、Trusted Publisher の設定ではなく、この呼び直しが拒否されたと見る。次のように落とす（`release.yml` の publish-crates の先頭のコメントと同じ内容）。

1. `release.yml` の publish-crates を、auth の step を先頭の 1 回だけにし、5 つの publish step すべてにそのトークン（`steps.auth_core.outputs.token`）を渡す形に直す。
2. 版を上げて出し直す。その時点で `pasta_core` の版は公開済みなので、同じタグを付け直して使うことはできない。
3. 1 回取得の形で 30 分を超えて失敗したら、「失敗した job の再実行」で続ける（公開済みのクレートは `skipped` で飛ばされ、新しいトークンで残りを公開する）。

`pasta_core` の auth から失敗する場合は、呼び直しの問題ではない。8 節の Trusted Publisher の設定を見直す。

## 10. 初回の成功後の必須手順

初回のリリース CI が成功したら、次の 3 つを **必ず** 行う。どれも、手元に残った長期の認証情報での公開の経路を閉じるためのものである。

1. **「Trusted Publishing のみ」を有効にする。** 5 クレートそれぞれの crates.io の設定画面（Trusted Publishing の項）で、Trusted Publishing だけで公開を受け付け、トークンでの公開を拒む設定（`trustpub_only`）を有効にする。
2. **`CARGO_REGISTRY_TOKEN` を失効させる。** 開発機に置いていた crates.io の API トークンを、crates.io の「Account Settings」→「API Tokens」で失効（Revoke）させる。あわせて開発機の環境変数 `CARGO_REGISTRY_TOKEN` を消し、`cargo login` で保存していたなら `cargo logout` で消す。
3. **`VSCE_PAT` を失効させる。** 開発機に置いていた Marketplace 用の Personal Access Token を、Azure DevOps の「User settings」→「Personal access tokens」で失効（Revoke）させ、開発機の環境変数 `VSCE_PAT` を消す（この PAT は Azure DevOps の global PAT にあたり、2026-12-01 以降は使えなくなる見込み）。

**緊急のとき**（CI が長く使えず、手で公開するしかないとき）は、クレートの owner が crates.io の設定画面で「Trusted Publishing のみ」を無効に戻せる。その場合は新しい API トークンを作って手で公開し、済んだらトークンを失効させ、「Trusted Publishing のみ」を有効に戻す。

## 11. 新しいクレートの初回公開

Trusted Publishing のトークンでは、crates.io にまだ無いクレートを作れない。新しいクレートを公開対象に加えるときは、初回だけ手で公開してから Trusted Publisher を設定する。リリース CI はクレートが crates.io に無いことを検出すると、そのクレートを `failed`・`not-registered` にして止まる。

1. crates.io で、新しいクレートを公開できる API トークン（スコープ `publish-new`）を一時的に作る。
2. 開発機で、そのトークンを使って新しいクレートを手で公開する（`cargo publish -p <クレート名>`）。依存先のクレートの同じ版が crates.io に公開済みである必要がある。
3. 公開できたら、1 のトークンをすぐに失効させる。
4. 新しいクレートに、8 節と同じ値で Trusted Publisher を設定し、10 節の「Trusted Publishing のみ」を有効にする。
5. `release.yml` の publish-crates に、そのクレートの auth step と publish step の組を依存順の位置に足し、job の outputs・集約 step・report job の一覧にも加える。`release-setup-check.yml` の名前の対応表と、12 節の表（正本は design の「認証名の契約」）の crates の並びも更新する。

## 12. 名前の対応表

各公開先の設定画面に入れる名前と、リリース CI の実際の名前の対応。確認ワークフローの Summary にも同じ名前が出る。設定をやり直すときは、この表と設定画面を照合する。

| 項目 | 値 | 使う場所 |
|------|----|----------|
| リポジトリ | owner `ekicyou`・name `pasta` | crates.io Trusted Publisher、Azure フェデレーション資格情報の subject |
| ワークフローのファイル名 | `release.yml` | crates.io Trusted Publisher（完全一致） |
| 公開用 environment | `release`（保護規則: Selected branches and tags → タグ `v*`） | publish-crates・publish-vsce の `environment:`、crates.io の environment 欄、Azure FIC subject `repo:ekicyou/pasta:environment:release` |
| 確認用 environment | `release-setup-check`（保護規則: ブランチ `main` のみ） | release-setup-check.yml の `environment:`、Azure FIC subject `repo:ekicyou/pasta:environment:release-setup-check` |
| Azure の ID | リポジトリ variables `AZURE_CLIENT_ID`・`AZURE_TENANT_ID`・`AZURE_SUBSCRIPTION_ID`（値はリポジトリに書かない） | `Azure/login@v3` の `client-id`・`tenant-id`・`subscription-id` |
| OIDC audience | `api://AzureADTokenExchange`（Azure/login の既定） | FIC の Audience 欄 |
| Marketplace | publisher `ekicyou`・拡張 `pasta-vscode`（`ekicyou.pasta-vscode`） | `vsce show`・`vsce publish` |
| crates | `pasta_core` → `pasta_dsl` → `pasta_lua` → `pasta_shiori` → `pasta_check` | publish-crates の step 列・Trusted Publisher の設定（5 件） |
