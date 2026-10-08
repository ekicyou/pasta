# Requirements Document

## Project Description (Input)
**誰が困っているか**: pasta をリリースする開発者（およびリリース手順を代行するエージェント）。

**現状**: リリースは `release-workflow` spec の手順（Stage A〜D）を手元の Windows で毎回たどって行っている。手順が長く、セッションを開いたまま再試行を粘る必要がある。公開に使う認証情報（`CARGO_REGISTRY_TOKEN`・`VSCE_PAT`）を開発機の環境変数に置いている。ビルドした成果物（`release/hello-pasta/**`・`release/hello-pasta.nar`・サンプルゴーストの `pasta.dll` と `THIRD_PARTY_LICENSES.txt`）を git にコミットしている。さらに Marketplace の公開に必要な Azure DevOps の global PAT は 2026-12-01 に廃止され、このままでは Marketplace へ公開できなくなる。

**何を変えるか**: バージョンを上げたコミットを main に入れて `vX.Y.Z` タグを push すれば、その後は人手なしで crates.io（5 クレート）・VSCode Marketplace・GitHub Release（`pasta.dll.zip`・`hello-pasta.nar`・VSIX 添付）への公開が終わるようにする。失敗時は Actions の「失敗した job の再実行」だけで、公開済みのものを飛ばして最後まで進める（各公開先の実際の状態で判定する冪等な再実行）。長期の認証情報はリポジトリに置かず、crates.io は Trusted Publishing（OIDC）、Marketplace は Microsoft Entra ID のワークロード ID 連携で認証する。ビルドした成果物は git で追跡せず、タグ時点のソースから CI が作る。詳細は `brief.md` を参照。

## Introduction

本仕様は、`vX.Y.Z` タグの push を契機に、公開前の検査・成果物のビルド・3 つの公開先への公開までを GitHub Actions 上で人手なしに行う「リリース CI」を定義する。

開発者（またはエージェント）がすることは、バージョンを上げたコミットを main に入れ、そのコミットに `vX.Y.Z` タグを付けて push することだけになる。その後の検査・ビルド・公開はリリース CI が行う。途中で失敗したときは、GitHub Actions の「失敗した job の再実行」を押すだけで、公開済みのものを飛ばして最後まで進む。

あわせて、ビルドした成果物の git 追跡をやめる。成果物はタグが指すソースから CI が毎回作る。

期限がある。Marketplace の公開に使っている Azure DevOps の global PAT は 2026-12-01 に廃止される。それまでに、PAT を使わない Marketplace への公開がリリース CI で動いている必要がある。

本文書の中の **【仮定】** は、brief.md だけでは決めきれず、要件ディスカッションで確かめる前提を示す。対応する論点は末尾の「未決事項（要件ディスカッションへの申し送り）」に番号付きでまとめた。

用語:
- **リリース CI**: 本仕様が作る、タグの push を契機に動く GitHub Actions のワークフロー。
- **リリースタグ**: `v` の後に `X.Y.Z`（数字 3 つをドットでつないだもの）が続くタグ。例: `v0.3.8`。
- **公開先**: crates.io・VSCode Marketplace・GitHub Release の 3 つ。
- **公開対象のクレート**: `pasta_core`・`pasta_dsl`・`pasta_lua`・`pasta_shiori`・`pasta_check` の 5 つ。`pasta_sample_ghost` と `pasta_lsp` は公開しない。
- **公開済み**: その版が公開先に実際に存在する状態。リリース CI の過去の実行記録ではなく、公開先に問い合わせた結果で決める。
- **配布物**: `pasta.dll.zip`・`hello-pasta.nar`・VSIX（VSCode 拡張のパッケージ）の 3 つ。
- **一回限りのセットアップ**: リリース CI が公開先へ認証できるようにするために、人が 1 度だけ行う設定（crates.io・Azure・Marketplace・GitHub）。

## Boundary Context

- **In scope**:
  - リリースタグの push を契機に動くリリース CI の新設。
  - 公開前の関門（タグと版の一致、タグのコミットが main から到達できること、テストと lint）。
  - 配布物のビルド。中身は今の手順で作るものと同じにする。
  - crates.io・Marketplace への公開と、公開済みかの判定。
  - GitHub Release の作成、配布物の添付、リリースノートの生成（今の分類方式を引き継ぐ）。
  - 失敗した job の再実行による続行（冪等性）。
  - 長期の認証情報を使わない認証（crates.io は Trusted Publishing、Marketplace は Entra ID のワークロード ID 連携）。
  - ビルドした成果物の git 追跡の解除と、無視の設定。
  - 成果物を作るスクリプト（`release.ps1`・VSIX 用の WASM ビルドスクリプト）の、CI で動かすための調整。
  - 一回限りのセットアップの手順書。
  - 利用者・開発者向け文書のリリース手順の記述の更新。
- **Out of scope**:
  - 版の決定と、版を上げるコミット（`release-workflow` 側に残す）。
  - `release-workflow` spec の書き直し（エージェントの手順を「版の決定 → bump → PR のマージ → タグの push → 結果の確認・失敗時の再実行」へ縮める）。本仕様の完了後に別に扱う。
  - マニュアルの公開（`manual.yml` の今の仕組みのまま）。
  - `build.yml` の PR・main 向けの検査内容の変更。
  - pasta_lsp の独立したリリース、Open VSX への公開、Windows 以外の配布物。
  - 新しいクレートの初回公開（Trusted Publishing では作れないため、その時に手で行う）。
  - GitHub のブランチ保護・Immutable Releases の設定そのもの。
  - 一回限りのセットアップの実施そのもの（人が手順書に従って行う）。
- **Adjacent expectations**:
  - `release-workflow`（常駐 spec）: 版の決定・bump・タグの作成と push は、引き続き開発者（エージェント）が `release-workflow` の手順で行う。本仕様はタグが push された後だけを受け持つ。本仕様の完了後、`release-workflow` の Stage A〜D のうち CI へ移ったもの（ローカルビルド・公開の 2 トラック・Resume・ScheduleWakeup による再試行・main の CI 全緑の確認）は `release-workflow` の更新で取り除く。
  - `build.yml`: ツールチェーンの構成（`.cargo/config.toml` の crt-static、mlua の vendored LuaJIT、日本語ロケールの設定）を前提として引き継ぐ。`build.yml` の検査内容は変えない。
  - `pasta_check release`: nar の作成と `updates.txt` の生成は、今の `pasta_check release` の挙動のまま使う。
  - `release-workflow` design.md: 公開順（core → dsl → lua → shiori → check）、公開済みの判定の考え方、リリースノートの分類方式は、そこに書かれたものを引き継ぐ。
  - `hello-pasta-shell-art`（後続）: 同じ `release.ps1` と生成物の追跡の扱いを触る。本仕様はシェルの画像の追跡の扱いを変えない（【仮定】未決事項 7）。

## Requirements

### Requirement 1: リリースタグの push による起動

**Objective:** 開発者として、リリースタグを push するだけでリリースが始まってほしい。そうすれば、手元で長い手順をたどったり、セッションを開いたまま待ったりしなくて済む。

#### Acceptance Criteria

1. When リリースタグがリポジトリへ push されたとき, the Release CI shall そのタグのコミットを対象にリリースを開始する。
2. When リリースタグの形に合わないタグ（例: `v0.3`・`test-1`。`v1.0.0-rc.1` のようなプレリリースの形を含む）が push されたとき, the Release CI shall リリースを開始しない（プレリリースの版は扱わない。brief の `vX.Y.Z` のとおり）。
3. When ブランチへの push や PR が行われたとき, the Release CI shall 起動しない。
4. While あるリリースタグのリリースが実行中であるとき, the Release CI shall 同じタグのリリースをもう 1 つ並行して走らせない。
5. The Release CI shall リポジトリへコミット・ブランチ・タグを書き戻さない（GitHub Release の作成に伴うものを除く）。

### Requirement 2: 公開前の関門

**Objective:** メンテナーとして、取り消せない公開の前に、タグの指すコミットがリリースしてよい状態かを確かめてほしい。そうすれば、版の食い違いや壊れたコミットを公開してしまうことを防げる。

#### Acceptance Criteria

1. When リリースが開始されたとき, the Release CI shall どの公開先への公開よりも先に、本要件の検査をすべて行う。
2. If リリースタグの版（先頭の `v` を除いた部分）が、タグのコミットのワークスペースの版と一致しないとき, the Release CI shall 公開を一切行わずに失敗し、タグの版とワークスペースの版の両方を示す。
3. If タグのコミットの VSCode 拡張の版（`editors/vscode/package.json`）がリリースタグの版と一致しないとき, the Release CI shall 公開を一切行わずに失敗し、食い違う版を示す（Marketplace にタグと食い違った版が出るのを防ぐため）。
4. If タグのコミットがリポジトリの main ブランチから到達できないとき, the Release CI shall 公開を一切行わずに失敗し、その理由を示す。
5. When 版と到達性の検査が通ったとき, the Release CI shall タグのコミットそのもので、`build.yml` が PR・main で行う検査をすべて、`build.yml` と同じ構成（ツールチェーン・ターゲット・日本語ロケール）で実行する。
6. The Release CI shall 関門の検査の一覧を `build.yml` と別に持たない。`build.yml` に検査が足されたり変わったりしたとき、関門も同じ検査を行う（実現の方法は設計で決める。例: `build.yml` を呼び出す）。
7. If 関門の検査のいずれかが失敗したとき, the Release CI shall 公開を一切行わずに失敗する。
8. The Release CI shall 本要件の関門を、今のリリース手順の「main の CI がすべて緑か」の確認に代わるものとする。リリース CI は main の CI の結果を参照しない。

### Requirement 3: 配布物のビルド

**Objective:** 利用者として、各リリースの配布物が、今と同じ中身で、タグの指すソースから作られていてほしい。そうすれば、リリースページから入手したものがその版のソースと対応していると信頼できる。

#### Acceptance Criteria

1. When 公開前の関門が通ったとき, the Release CI shall タグのコミットのソースから `pasta.dll.zip`・`hello-pasta.nar`・VSIX の 3 つの配布物を作る。
2. The Release CI shall `pasta.dll.zip` に、x86（32bit）向けのリリースビルドの `pasta.dll` と、その第三者ライセンス表示 `THIRD_PARTY_LICENSES.txt` の 2 つを、今の手順で作るものと同じ構成で入れる。
3. The Release CI shall `hello-pasta.nar` を `pasta_check release` で作り、中身（ファイルの一覧と配置、`updates.txt` を含む）を今の手順で作るものと同じにする。
4. The Release CI shall VSIX を、タグの版を持つ VSCode 拡張のパッケージとして作り、言語サーバーの WASM と、その第三者ライセンス表示を同梱する（【仮定】WASM のビルドの種類（デバッグ/リリース）は今の `npm run package` と同じにする。未決事項 6）。
5. The Release CI shall 配布物のビルドに必要なツールを、ビルドのたびに同じ版になるよう版を固定して用意する。
6. If いずれかの配布物のビルドに失敗したとき, the Release CI shall どの公開先への公開も行わずに失敗し、どの配布物のビルドが失敗したかを示す。
7. The Release CI shall 配布物を Windows の環境でビルドする。
8. The Release CI shall ビルドした配布物を、同じリリースの後続の公開の処理（再実行を含む）から使えるよう保持する。
9. The Release CI shall 同じタグのソースから配布物を作り直したとき、依存クレートの版の解決結果まで同じ配布物が得られるようにする。
10. The repository shall `Cargo.lock` を git で追跡し、main の CI・公開前の関門・配布物のビルドが同じ依存の解決結果で動くようにする。

### Requirement 4: crates.io への公開

**Objective:** Rust 開発者として、各リリースで 5 つのクレートが依存順に crates.io へ公開されてほしい。そうすれば、`cargo add` でその版をすぐ使える。

#### Acceptance Criteria

1. When 公開前の関門と配布物のビルドが通ったとき, the Release CI shall 公開対象の 5 クレートを `pasta_core` → `pasta_dsl` → `pasta_lua` → `pasta_shiori` → `pasta_check` の順に、タグのコミットのソースから公開する。
2. When あるクレートのその版が crates.io で既に公開済みであるとき, the Release CI shall そのクレートの公開を飛ばし、次のクレートへ進む。
3. The Release CI shall 公開済みかどうかを、各クレートを公開する直前に crates.io へ問い合わせた結果で判定する。
4. If あるクレートの公開に失敗したとき, the Release CI shall それより後のクレートを公開せずに失敗し、公開済みのクレートと未公開のクレートを示す。
5. When あるクレートの公開が終わり、crates.io の索引への反映の待ちが時間切れになったとき, the Release CI shall それを失敗とせず次のクレートの公開へ進み、次のクレートの公開が依存先の未反映で失敗した場合はその公開を再試行する。
6. If 公開対象のクレートが crates.io にまだ 1 度も公開されていないとき, the Release CI shall そのクレートを公開せずに失敗し、初回の公開は手で行う必要があることを示す。
7. The Release CI shall `pasta_sample_ghost` と `pasta_lsp` を crates.io へ公開しない。

### Requirement 5: VSCode Marketplace への公開

**Objective:** VSCode の利用者として、各リリースで拡張の新しい版が Marketplace に出てほしい。そうすれば、VSCode の拡張機能の更新で最新版を受け取れる。

#### Acceptance Criteria

1. When 公開前の関門と VSIX のビルドが通ったとき, the Release CI shall ビルドした VSIX を publisher `ekicyou` の拡張として Marketplace へ公開する。
2. When その版の拡張が Marketplace で既に公開済みであるとき, the Release CI shall 公開を飛ばし、失敗とせずに終える。
3. The Release CI shall 公開済みかどうかを、Marketplace へ問い合わせた結果で判定する。
4. If Marketplace への公開に失敗したとき, the Release CI shall 失敗を示し、crates.io への公開の結果には影響を与えない。
5. The Release CI shall Marketplace への公開に Azure DevOps の PAT を使わない。2026-12-01 の global PAT の廃止の後も公開が続けられることを完了の条件とする。
6. The Release CI shall Marketplace への公開と crates.io への公開を、互いの成否を待たずに進められるようにする（【仮定】今の設計の「2 トラックを互いにブロックしない」を引き継ぐ）。

### Requirement 6: GitHub Release とリリースノート

**Objective:** 利用者として、各リリースの配布物と変更点を GitHub のリリースページで 1 か所にまとめて見たい。そうすれば、ゴーストの DLL・サンプルゴースト・拡張を同じ版でそろえて入手できる。

#### Acceptance Criteria

1. When crates.io への 5 クレートの公開がすべて公開済みになったとき, the Release CI shall リリースタグの GitHub Release を作成する。Marketplace への公開の成否は待たず、VSIX は Marketplace の成否によらず添付する（brief の Constraints「公開順」のとおり）。
2. The Release CI shall GitHub Release に `pasta.dll.zip`・`hello-pasta.nar`・VSIX の 3 つの配布物を添付する。
3. The Release CI shall GitHub Release の題名を `pasta vX.Y.Z` とする（今の手順と同じ）。
4. When そのタグの GitHub Release が既に存在し、3 つの配布物がすべて添付済みであるとき, the Release CI shall 作成を飛ばし、失敗とせずに終える。
5. If そのタグの GitHub Release が既に存在するが、添付されていない配布物があるとき, the Release CI shall 足りない配布物だけを添付する。配布物が欠けた Release を公開状態で残さない（Immutable Releases が有効な場合の作成の順序は設計で決める。未決事項 8）。
6. The Release CI shall リリースノートを、1 つ前のリリースタグからリリースタグまでのコミット（マージコミットを除く）から作る。1 つ前のリリースタグが無いときは、リリースタグまでの全コミットから作る。
7. The Release CI shall リリースノートのコミットを Conventional Commits の種類で分類し、`feat`（✨ Features）・`fix`（🐛 Bug Fixes）・`refactor`（♻️ Refactoring）・`docs`（📝 Documentation）・`test`（🧪 Tests）・`chore`（🔧 Maintenance）の見出しの下に並べる。
8. The Release CI shall スコープが `spec` のコミットをリリースノートから除き、コミットが 1 つも無い見出しを出さない。
9. The Release CI shall リリースノートの末尾に、1 つ前のリリースタグとの比較のリンク（Full Changelog）を付ける。
10. If crates.io への公開が終わっていないとき, the Release CI shall GitHub Release を作成しない。

### Requirement 7: 失敗からの再実行と冪等性

**Objective:** 開発者として、リリースが途中で失敗しても、GitHub Actions の「失敗した job の再実行」を押すだけで最後まで進んでほしい。そうすれば、手元で手順を組み立て直したり、公開済みのものを二重に公開しようとしたりせずに済む。

#### Acceptance Criteria

1. When 失敗した job が再実行されたとき, the Release CI shall 公開済みの公開先・クレートを飛ばし、未公開のものだけを公開して最後まで進む。
2. The Release CI shall 公開済みかどうかを、リリース CI の過去の実行の記録ではなく、各公開先の実際の状態で判定する。
3. When すべての公開先で公開済みの状態でリリースが実行されたとき, the Release CI shall 何も公開せずに成功で終える。
4. If 公開先への問い合わせが一時的な障害で失敗したとき, the Release CI shall 公開済みと見なさず、その job を失敗として終える（再実行で続行できる状態を保つ）。
5. The Release CI shall 再実行のために、開発者が手元でコマンドを実行したり、リリース CI の設定を書き換えたりする必要を生じさせない（認証の設定の誤りなど、一回限りのセットアップに起因する失敗を除く）。
6. While 公開の一部が失敗した状態にあるとき, the Release CI shall 公開済みのものを取り消したり、上書きしたりしない。

### Requirement 8: 実行結果の報告

**Objective:** 開発者として、リリースの実行が終わったときに、どの公開先が公開された・飛ばされた・失敗したかを一目で知りたい。そうすれば、再実行が要るかどうかと、何が残っているかをすぐ判断できる。

#### Acceptance Criteria

1. When リリースの実行が終わったとき, the Release CI shall 実行結果の画面に、公開先ごと（crates.io はクレートごと）に「今回公開した」「公開済みのため飛ばした」「失敗した」「前の段の失敗で行わなかった」のいずれかを示す。
2. When GitHub Release を作成したとき, the Release CI shall 実行結果の画面に GitHub Release の URL を示す。
3. If 公開前の関門で失敗したとき, the Release CI shall どの検査で失敗したかと、どの公開先にも公開していないことを示す。

### Requirement 9: 長期の認証情報を置かない認証

**Objective:** メンテナーとして、公開に使う長期の認証情報をリポジトリにも開発機にも置かずに済ませたい。そうすれば、認証情報の漏洩や期限切れの管理から解放され、PAT の廃止にも影響されない。

#### Acceptance Criteria

1. The Release CI shall crates.io への公開に、リポジトリやその secrets に保存した長期のトークンを使わず、実行のたびに短い期限で発行される認証を使う（crates.io の Trusted Publishing）。
2. The Release CI shall Marketplace への公開に、リポジトリやその secrets に保存した長期のトークン・パスワードを使わず、実行のたびに短い期限で発行される認証を使う（Microsoft Entra ID のワークロード ID 連携）。
3. The Release CI shall 公開先の認証に必要な権限を、公開を行う処理だけに与え、検査やビルドの処理には与えない。
4. The Release CI shall 公開を行う処理を、GitHub の environment に属させ、リリースタグからの実行に限る。
5. If 認証の設定（crates.io の Trusted Publisher、Azure のフェデレーション資格情報、Marketplace の publisher のメンバー）がリリース CI の実際の名前（ワークフローのファイル名・environment の名前・リポジトリ）と一致しないとき, the Release CI shall その公開先への公開を行わずに失敗し、認証で失敗したことを示す。
6. The Release CI shall 開発機の環境変数 `CARGO_REGISTRY_TOKEN`・`VSCE_PAT` を必要としない。

### Requirement 10: ビルドした成果物の git 追跡の解除

**Objective:** 開発者として、ビルドした成果物がリポジトリにコミットされない状態にしたい。そうすれば、リリースのたびに大きなバイナリの差分が履歴に積もらず、成果物とソースの食い違いも起きない。

#### Acceptance Criteria

1. The repository shall `release/` 配下（`release/hello-pasta/**`・`release/hello-pasta.nar`）と、サンプルゴーストの `ghost/master/pasta.dll`・`ghost/master/THIRD_PARTY_LICENSES.txt` を git で追跡しない。
2. When 開発者が手元で配布物を作るスクリプトを実行したとき, the repository shall 作られた成果物を git の未追跡の変更として示さない（無視の設定に含まれる）。
3. The repository shall 成果物の追跡を解除した後も、`cargo test --all` と clippy がクリーンなチェックアウトで成功する状態を保つ。
4. The repository shall サンプルゴーストの手書きの正本（`descript.txt`・`pasta.toml`・`dic/`・`install.txt` など）の追跡を続ける。
5. The repository shall サンプルゴーストのシェルの画像（`surface*.png`・`surfaces.txt`）の追跡の扱いを変えない（【仮定】未決事項 7）。
6. The Release CI shall 配布物を、git で追跡している成果物からではなく、タグのソースからのビルドだけで作る。

### Requirement 11: 一回限りのセットアップの手順書

**Objective:** メンテナーとして、リリース CI が公開先へ認証できるようにするための設定を、手順書に従って 1 度で漏れなく行いたい。そうすれば、初回のリリースが認証の設定漏れで失敗することを防げ、将来の設定のやり直しにも使える。

#### Acceptance Criteria

1. The 手順書 shall crates.io で、公開対象の 5 クレートそれぞれに Trusted Publisher を設定する手順を示し、設定に入れる値（リポジトリの owner と名前・ワークフローのファイル名・environment の名前）をリリース CI の実際の名前と一致する形で示す。
2. The 手順書 shall Azure で、従量課金のサブスクリプションを用意する手順、予算アラート（¥0 に近い額）を設定する手順、ユーザー割り当てのマネージド ID を作る手順、GitHub 用のフェデレーション資格情報を作る手順を示す。
3. The 手順書 shall 無料試用版のサブスクリプションのままでは期限切れで公開が止まること、従量課金への切り替えと予算アラートの設定が必須であることを明記する。
4. The 手順書 shall サブスクリプションの要らないアプリ登録（サービスプリンシパル）による方法を採らない理由（公開が「corporate credentials」のエラーで失敗する報告があること）を記す。
5. The 手順書 shall Marketplace の publisher `ekicyou` に、作ったマネージド ID を Contributor として追加する手順を示す。
6. The 手順書 shall GitHub で environment を作り、リリースタグからの実行に限る設定と、variables（client・tenant・subscription の ID）を登録する手順を示す。
7. The 手順書 shall セットアップが済んだことを、公開を行わずに確かめる方法を示す（【仮定】確かめ方の具体は設計で決める）。
8. The 手順書 shall 新しいクレートを追加したときの初回の公開は手で行い、その後に Trusted Publisher を設定する必要があることを示す。
9. The 手順書 shall リリース CI のワークフローの外に置き、リリースのたびに実行するものではないことを明記する。

### Requirement 12: リリース手順の文書の更新

**Objective:** 開発者・エージェントとして、リリースの手順を記した文書が、タグの push で公開が終わる新しい手順と一致していてほしい。そうすれば、古い手順（手元でのビルドと公開、PAT の設定）に従って誤った操作をしなくて済む。

#### Acceptance Criteria

1. The 文書 shall `crates/pasta_sample_ghost/RELEASE.md` のリリース手順を、「版を上げたコミットを main に入れる → リリースタグを push する → リリース CI の結果を確かめる → 失敗したら失敗した job を再実行する」の流れに書き換える。
2. The 文書 shall pasta-check スキルのリリース手順の記述のうち、成果物の置き場所や GitHub Release の作成の手順など、本仕様で変わる部分を新しい手順に合わせる。
3. The 文書 shall 手元で配布物を作るスクリプトの説明を、作った成果物がコミットの対象ではないこと（動作確認用であること）に合わせる。
4. The 文書 shall `CARGO_REGISTRY_TOKEN`・`VSCE_PAT` を開発機に設定する手順を、リリース手順の前提から外す。
5. The 文書 shall `release-workflow` spec の本体を書き換えない（本仕様の範囲外。完了後に別に更新する）。

## 未決事項（要件ディスカッションへの申し送り）

本文の **【仮定】** に対応する。brief.md だけでは決めきれなかったため、最善の仮定で要件を書いた。

1. ~~**プレリリースの版**~~ → **確定（自明修正）**: 扱わない。brief の `vX.Y.Z` のとおり。<br>旧: **プレリリースの版**（R1.2）: `v1.0.0-rc.1` のようなプレリリースのタグを扱うか。仮定: 扱わない（数字 3 つの形だけを起動の対象にする）。
2. ~~**拡張の版の検査**~~ → **確定（自明修正）**: 検査する（R2.3）。<br>旧: **拡張の版の検査**（R2.3）: brief は「タグと `Cargo.toml` の版の一致」だけを挙げる。`editors/vscode/package.json` の版も一致を検査するか。仮定: 検査する（Marketplace に食い違った版が出るのを防ぐため）。
3. ~~**関門の範囲**~~ → **確定（議題 2）**: `build.yml` が PR・main で行う検査（test・clippy の x86・x64、cargo-deny、luacheck、WASM ビルド）をすべて、タグのコミットで通す。検査の一覧を要件に書き写さず、`build.yml` に従わせる（R2.5・R2.6）。<br>旧: **関門の範囲**（R2.6）: `build.yml` は test・clippy のほかに cargo-deny・luacheck・WASM ビルドも回す。「main の CI が全部緑か」の確認を置き換えるなら、これらも関門に含めるか。仮定: test と clippy は x86・x64 の両方で回す。残りは未定。
4. ~~**GitHub Release が待つもの**~~ → **確定（自明修正）**: crates だけを待つ。brief の Constraints「公開順」のとおり（R6.1）。<br>旧: **GitHub Release が待つもの**（R6.1）: GitHub Release の作成は crates の公開の成功だけを待つか、Marketplace の公開の成功も待つか。仮定: crates だけを待つ（今の設計と同じ）。VSIX は Marketplace の成否によらず添付する。
5. ~~**依存の固定（再現性）**~~ → **確定（議題 3）**: `Cargo.lock` を追跡する（R3.9・R3.10）。依存の更新は `cargo update` のコミットとして明示的に行う。crates.io の利用者は自分の lock で解決し直すので、公開するクレートへの影響は無い。<br>旧: **依存の固定（再現性）**（R3.9）: `.gitignore` は `Cargo.lock` を無視している（「ライブラリクレートなので」）。タグのソースから成果物を再現する要件を、依存クレートの解決結果まで含めて満たすには `Cargo.lock` の追跡が要る。追跡するか、ビルドの構成の再現だけで足りるとするか。
6. **VSIX の WASM のビルドの種類**（R3.4）: 今の `npm run package` は `build-wasm.ps1` を `-Release` なしで呼ぶため、VSIX に入る WASM はデバッグビルドになっている。今と同じにするか、リリースビルドに変えるか。
7. **シェルの画像の追跡**（R10.5）: `ghosts/hello-pasta/shell/master/surface*.png`・`surfaces.txt` は `cargo run -p pasta_sample_ghost` が作る生成物だが、git で追跡している。brief は追跡の解除の対象に挙げていない。後続の `hello-pasta-shell-art` が画像を「追跡する素材」に変えるため、本仕様では今のまま追跡を続ける、で良いか。同様に、`release.ps1` が `crates/pasta_lua/scripts` から写す `ghost/master/scripts/README.md` も追跡されている生成物である。これを追跡の解除の対象に含めるか。
8. **→ 設計へ**: 要件は R6.5「配布物が欠けた Release を公開状態で残さない」とし、作成の順序（下書き → 添付 → 公開）は設計で決める。<br>旧: **GitHub Release の作成済み・添付漏れ**（R6.5）: Immutable Releases を有効にすると、公開後の Release に配布物を足せない。作成の途中で失敗して添付が欠けた Release が残った場合の扱い（作成と添付を一度に済ませる・下書きで作ってから公開する等）を、要件としてどこまで求めるか。
9. **ワークフロー自体の不具合の修正**（R7.5）: 「失敗した job の再実行」は、タグのコミットにあるワークフローの定義で再実行する。ワークフローの定義そのものに不具合があった場合、再実行では直せない。その場合の回復の手段（同じタグで手動起動できる入口を設けるか、版を上げて出し直すか）を要件に含めるか。
10. **→ 設計へ**: 置き場所は設計で決める。<br>旧: **手順書の置き場所**（R11）: 一回限りのセットアップの手順書をどこに置くか（`crates/pasta_sample_ghost/RELEASE.md` の一節・リポジトリの開発者向け文書・spec 配下など）。仮定: 設計で決める。
11. ~~**手元のスクリプトの扱い**~~ → **確定（自明修正）**: 残す。成果物の生成は `release.ps1` を CI からも呼ぶ形で流用するため（research.md Option C）。<br>旧: **手元のスクリプトの扱い**（R12.3）: `release.ps1`・`release.bat` を、CI が使う部品として残しつつ手元での動作確認用にも残すか。仮定: 残す。
12. ~~**Marketplace の新しい OIDC 公開**~~ → **確定（議題 1）**: Entra ID のワークロード ID 連携を本線にする（R5・R9.2・R11 は今のまま）。`--oidc` は preview で Marketplace 側の手順書も無いため、本仕様では採らない。正式に提供されたら、その時に乗り換えを検討する。2 本の経路を並べて持つことはしない。<br>旧: **Marketplace の新しい OIDC 公開（`vsce publish --oidc`）**（R5・R9.2・R11）: brief の確定後の調査で、vsce に Marketplace 自身の Trusted Publishing（GitHub の OIDC トークンを Marketplace のセッショントークンに交換。Azure のサブスクリプションが要らない）が隠しオプションとして入ったことが分かった（PR microsoft/vscode-vsce#1291、2026-07 マージ、2026-09 に契約確定のコミット）。ただし preview 扱いで、Marketplace 側の設定の手順書が見当たらない。brief どおり Entra ID のワークロード ID 連携で進めるか、OIDC 公開を本線または予備にするか。仮定: brief どおり Entra ID で進め、OIDC 公開は設計で再評価する。
13. **crates.io の「Trusted Publishing のみ」設定**（R9・R11）: crates.io にはクレートごとにトークンでの公開を拒む設定（`trustpub_only`）がある。手順書で、セットアップの確認後にこれを有効にする手順を必須にするか。有効にすると、新しいクレートの初回公開（手で行う）以外の手作業の公開ができなくなる。仮定: 任意の手順として載せる。
