# Requirements Document

## Project Description (Input)

見本ゴースト hello-pasta は、SSP のネットワーク更新に対応していない。利用者は新しい版が出るたびに、`hello-pasta.nar` を取り直して入れ直すしかない。開発者は hello-pasta をネットワーク更新で配りたく、公開の手間はリリース CI に任せたい。

現状、`ghost/master/descript.txt` の `homeurl` はリポジトリのページを指していて、更新の置き場所ではない。SSP は `homeurl` の下に `updates.txt` と、`ghost/master/pasta.dll` のようなフォルダ構造のままのファイル一式を探す。GitHub Release の添付は平らな名前だけで、フォルダ構造を置けない。

変えたいこと: GitHub Pages（`https://ekicyou.github.io/pasta/hello-pasta/`）に、最新リリースの `hello-pasta.nar` の中身をそのまま展開して置き、`homeurl` をそこへ向ける。置き場所はリリースのたびにリリース CI が自動で最新にし、中身が nar と同じであることを公開前と公開後の両方で確かめる。hello-pasta をコピーして作った派生ゴーストが、うっかり hello-pasta の更新で自分の辞書を上書きされないようにする。

詳細な背景・退けた案・制約は `brief.md` を参照。

## Introduction

この仕様は、見本ゴースト hello-pasta を SSP のネットワーク更新で最新版にできるようにする。利用者は SSP の更新確認 1 回で、辞書・`pasta.toml`・シェル・`pasta.dll` を、1 つのリリースの版でそろえて入れ替えられる。開発者は、リリースタグを push する以外の手作業なしに、更新の置き場所を最新にできる。

## Boundary Context

- **In scope**:
  - hello-pasta の `homeurl` を、更新の置き場所に向けること。
  - 更新の置き場所の中身（最新リリースの `hello-pasta.nar` の展開）と、その `updates.txt` との整合の検査。
  - リリース CI が、リリースのたびに置き場所を最新にし、公開後の実際の URL を Release の版と見比べ、結果を報告の表に出すこと。
  - 置き場所の入口（ブラウザで開いたときの案内）。
  - 派生ゴーストの事故の防止（マニュアルの注意・`pasta_check validate` の警告）。
  - リリース手順書（失敗時の回復を含む）の更新。
  - 実機の SSP での確認。
- **Out of scope**:
  - 作者が自分のゴーストをネットワーク更新に対応させる方法の案内（`pasta_check release` が `updates.txt` を作るところまでは既にある）。
  - 同梱バルーンの `homeurl`（バルーンの持ち主が決める。`bal/updates.txt` は既存のまま）。
  - hello-pasta の中に、更新を知らせる台詞やメニューを足すこと。
  - `pasta.dll` の arch ごとの配布（`pasta-dll-multiarch`）。
  - マニュアルの章立て・見た目の変更（触れるのは、ダウンロードのページへの案内と、派生ゴーストの注意の 1 節だけ）。
  - `pasta_check release` の `updates.txt` の作り方、Release の添付の名前・数。
  - nar から消えたファイルを利用者の手元から消す仕組み（`delete.txt`）。手順書に注意を 1 行書くだけにとどめる。
  - VS Code 拡張・LSP・pasta-recipe での派生ゴースト警告。
  - この仕様を入れる版の決定（ロードマップ側で決める。`pasta-dll-multiarch` と同じウェーブに置かない）。
- **Adjacent expectations**:
  - `release-workflow`・`release-ci`（完了）: リリースタグの push で Release が公開される流れとジョブ構成。この仕様はその結果（公開された `hello-pasta.nar`）を使う側で、公開の流れ自体は変えない。
  - `manual-download-page`（完了）: ダウンロードのページと、最新版の固定リンクの名前を Release の添付の名前と照らす検査。この仕様の変更がその検査を壊さない。
  - `pasta-check-bundled-balloon`（完了）: `updates.txt` の作り。この仕様は出力を使う側で、作り方を変えない。
  - `pasta-dll-multiarch`（未完了）: リリース CI と `release.ps1` を共有する。arch ごとの `pasta.dll` が nar に入れば、そのまま置き場所にも出るよう、この仕様は nar の中身の構成に依らない。同じウェーブに置かない。
  - 公開サイト（GitHub Pages）は、サイト全体を 1 回の公開で丸ごと入れ替える。置き場所とマニュアルは同じサイトに載るため、どちらの公開でも、もう一方が消えてはならない。

## Requirements

### Requirement 1: 更新の置き場所の公開内容

**Objective:** As a hello-pasta の利用者, I want SSP が更新の置き場所から、ゴースト一式を最新の版でそろえて取れること, so that 辞書・`pasta.toml`・シェル・`pasta.dll` の版がずれず、1 回の更新確認で最新版になる

#### Acceptance Criteria

1. The 更新の置き場所 shall `https://ekicyou.github.io/pasta/hello-pasta/` とし、直下に `updates.txt` を、`ghost/master/`・`shell/master/` などを nar のフォルダ構造のままで置く。
2. The 更新の置き場所の中身 shall 最新の GitHub Release の `hello-pasta.nar` を展開したものと、ファイルの一覧・内容とも同じである。
   - 置き換わった: `hello-pasta-book-sync` で、`main` のリポジトリのファイルに nar にだけ在るファイルを重ねたものになった。
3. The 更新の置き場所 shall `.dll`・`.pasta`・`.png`・`.txt`・`.toml`・`.csv` など nar に含まれる種類のファイルを、変換せずそのままの内容で返す。
4. When 新しいリリースが公開される, the 更新の置き場所 shall 公開後に、その版の `hello-pasta.nar` の中身へ入れ替わる。
   - 置き換わった: `hello-pasta-book-sync` で、nar の側だけが入れ替わり、辞書などは `main` への push で変わる。
5. Where nar に `bal/`（同梱バルーン）が含まれる, the 更新の置き場所 shall `bal/` を含めて nar 全体を展開したものにし、除外の規則を持たない。（現在の hello-pasta は `bal/` を含まない。）
   - 置き換わった: `hello-pasta-book-sync` で、リポジトリのファイルに nar にだけ在るファイルを重ね、`updates.txt`・`updates2.dau`・`developer_options.txt` は除外する。

### Requirement 2: ゴースト側の設定（`homeurl`）

**Objective:** As a hello-pasta の利用者, I want hello-pasta が自分の更新の置き場所を知っていること, so that SSP の更新確認が正しい場所を見に行く

#### Acceptance Criteria

1. The hello-pasta shall `ghost/master/descript.txt` の `homeurl` を `https://ekicyou.github.io/pasta/hello-pasta/`（末尾の `/` を含む）にする。
2. When `pasta_check release` が hello-pasta の nar を作る, the `pasta_check release` shall 変更前と同じ手順で、新しい `descript.txt` の md5 と大きさを nar の `updates.txt` に書く。
3. When SSP の更新確認が、`homeurl` を持つ版の hello-pasta で行われる, the 更新の置き場所 shall その `updates.txt` とその行が指すファイル一式を、SSP が取れる形で返す。
4. The リリース手順書とマニュアルの注意 shall `homeurl` を足した版の nar を入れた利用者から更新が効くこと（既存の利用者の nar の `homeurl` は、版が上がるまで古いまま）を書く。

### Requirement 3: 公開前の整合の検査

**Objective:** As a 開発者, I want 置き場所に出す前に、中身が `updates.txt` と合っていることを機械で確かめたい, so that 壊れた置き場所を利用者の SSP に配らない

#### Acceptance Criteria

1. When 置き場所の中身を作る, the マニュアル公開 CI shall 展開した各ファイルの md5 と大きさを、`updates.txt` の対応する行と照らす。
   - 置き換わった: `hello-pasta-book-sync` で、重ねた一式を最新 Release の `pasta_check release` で検査し、`updates.txt` を作り直して照らす。
2. If `updates.txt` の行が指すファイルが展開結果に無い、または md5・大きさが合わない, then the マニュアル公開 CI shall どのファイルが合わないかを表示して失敗し、公開しない。
   - 置き換わった: `hello-pasta-book-sync` で、照らす相手は Release の nar の展開結果でなく、`pasta_check release` が重ねて作った nar になった。
3. If 展開した `updates.txt` が無い、または読めない, then the マニュアル公開 CI shall 失敗し、公開しない。
   - 置き換わった: `hello-pasta-book-sync` で、`updates.txt` は `pasta_check release` が作り直したものになった。
4. While PR またはブランチで実行されている, the マニュアル公開 CI shall 公開を行わずに、置き場所の中身の作成と整合の検査までを実行する。
5. The マニュアル公開 CI shall 置き場所の中身を、マニュアルの静的性・検索・コンテンツの検査の対象に入れず、それらの検査の後で公開物に足す。
6. If マニュアルの検査または置き場所の整合の検査が失敗する, then the マニュアル公開 CI shall サイトを公開しない。
7. When リリース CI が `hello-pasta.nar` を作る, the リリース CI shall Release を公開する前に、作った nar を置き場所の組み立てと同じ照合に通し、合わなければ Release を公開しない。（公開済みの Release は差し替えられず、壊れた nar が最新になるとマニュアルの公開まで止まるため。）

### Requirement 4: 公開の経路と自動化

**Objective:** As a 開発者, I want リリースタグを push するだけで、置き場所が最新になること, so that 更新の公開に手作業を挟まない

#### Acceptance Criteria

1. When リリース CI が GitHub Release を公開する, the リリース CI shall マニュアル公開 CI を `main` で起動し、その完了を待つ。
2. The 公開サイトへの公開 shall マニュアル公開 CI の 1 か所だけで行い、リリース CI は公開サイトへ直接公開しない。
3. When 置き場所の更新と無関係なマニュアルの変更が `main` へ入り、マニュアルが再公開される, the マニュアル公開 CI shall その時点の最新 Release の `hello-pasta.nar` から置き場所の中身も作り直して公開し、置き場所をサイトから消さない。
   - 置き換わった: `hello-pasta-book-sync` で、`main` のリポジトリのファイルに最新 Release の nar にだけ在るファイルを重ねて作り直す。
4. If 最新の Release が無い、または最新の Release に `hello-pasta.nar` が無い、または取得できない、または取得した nar に `updates.txt` が無い, then the マニュアル公開 CI shall 失敗し、置き場所を欠いたサイトを公開しない。PR・ブランチでの実行でも同じく失敗にし、警告にとどめない。（欠いたままの公開は、すでに配られた利用者の更新確認を壊すので止める。再実行で直せる。）
5. The リリース CI shall 置き場所を最新にするための追加の権限を、その処理を行うジョブにだけ最小限で与え、既存のジョブの権限を広げない。
6. The マニュアル公開 CI shall 公開サイトの公開に必要な権限を、既存どおり公開を行うジョブにだけ与える。
7. If マニュアル公開 CI の起動または完了の待ち合わせが失敗する, then the リリース CI shall 失敗を報告の表に出し、公開済みの Release の状態を変えない。
8. While `homeurl` を足した版がまだ公開されていない, the マニュアル公開 CI shall 最新の Release の（前の版の）`hello-pasta.nar` を置き場所に置く。
   - 置き換わった: `hello-pasta-book-sync` で、辞書などは `main` のもの、nar にだけ在るファイルは最新の Release のものを置く。

### Requirement 5: 公開後の検証と報告

**Objective:** As a 開発者, I want 公開した実際の URL の中身が Release と同じであることを CI が確かめて報告すること, so that 公開後に利用者の更新が壊れていないと分かる

#### Acceptance Criteria

1. When マニュアル公開 CI の公開が完了する, the リリース CI shall 実際の URL（`https://ekicyou.github.io/pasta/hello-pasta/updates.txt`）を取り、その Release の `hello-pasta.nar` が含む `updates.txt` と内容が同じであることを確かめる。
   - 置き換わった: `hello-pasta-book-sync` で、`updates.txt` の nar の側の行だけを、Release の nar の `updates.txt` の行と md5・大きさで比べる見比べになった。
2. The リリース CI shall 結果を、公開先ごとの結果を並べる報告の表に 1 行（置き場所）として加え、status と、失敗のときは reason を出す。
3. If 実際の URL の `updates.txt` が Release 版と違う、または取れない, then the リリース CI shall 結果を `failed` として報告の表に出す。
   - 置き換わった: `hello-pasta-book-sync` で、「違う」は nar の側の行の md5・大きさが Release の nar の `updates.txt` と合わないことになった。
4. While 公開の反映が遅れている（公開サイトの配信に時間がかかる）, the リリース CI shall 一定の回数・時間の範囲で取り直してから結果を決める。回数・間隔・上限は設計で決める。見比べるのは `updates.txt` 1 本とし、全ファイルの md5 の取り直しは行わない（公開前に全ファイルを照合済みで、無変換配信の確認は実機確認 Requirement 10.4 で行う）。
5. If 置き場所の更新が失敗する, then the リリース CI shall 他の公開先（crates.io・Marketplace・GitHub Release）の status を、その失敗で変えない。
6. The 報告の表の置き場所の行 shall 他の公開先の行と同じ書式（公開先・status・reason）で並ぶ。

### Requirement 6: 失敗からの回復

**Objective:** As a 開発者, I want 公開が失敗しても、再実行で直せること, so that 手で公開サイトをいじらずに復旧できる

#### Acceptance Criteria

1. When 置き場所の更新が失敗する, the リリース手順書 shall リリース CI の失敗したジョブの再実行、またはマニュアル公開 CI の手動実行（`main`）で復旧できることを示す。
2. When 置き場所の更新を再実行する, the 置き場所の更新 shall すでに公開済みの Release を変えず、最新の Release の `hello-pasta.nar` から置き場所を作り直して公開する。
3. If 最新ではない Release のタグでリリース CI が再実行される, then the リリース CI shall マニュアル公開 CI を起動せず置き場所を更新せず、報告の表の置き場所の行に `skipped`（理由: 最新ではないタグ）を出す。最新かどうかの判定は起動の前に行い、ほかの run に影響させない。
4. The リリース手順書 shall 置き場所の更新の失敗時の読み方と、リリースの実行中は `release.yml` を `main` へ入れないという注意を載せる。
5. The リリース手順書 shall 最新の Release の nar が壊れていたときの手順（新しい版を切る）と、置き場所の行が `failed`（reason `dispatch`）のとき（初回の実リリースを含む）の手順を載せる。

### Requirement 7: 置き場所の入口

**Objective:** As a 利用者, I want SSP の「配布元 URL を開く」で 404 を見ないこと, so that ゴーストの配布元から入手方法へ迷わず進める

#### Acceptance Criteria

1. When ブラウザが `https://ekicyou.github.io/pasta/hello-pasta/` を開く, the 更新の置き場所 shall 404 でなく、ダウンロードのページ（`https://ekicyou.github.io/pasta/download.html`）へ案内するページを返す。ページは短い案内文とダウンロードのページへのリンクだけの静的なものとし、自動では移さない。
2. The 入口のページ shall `updates.txt` に載るファイル（SSP の更新の対象）に含まれず、`updates.txt` との整合の検査（Requirement 3）の結果に影響しない。
3. The 入口のページ shall リリースの版や nar の中身の構成に依らず、同じ内容で置ける。

### Requirement 8: 派生ゴーストの事故の防止

**Objective:** As a hello-pasta をコピーして自分のゴーストを作る作者, I want hello-pasta の更新で自分の辞書を上書きされないこと, so that 自分のゴーストを作り込んだ後でも、更新確認で作ったものが失われない

#### Acceptance Criteria

1. The マニュアル shall hello-pasta をコピーして自分のゴーストを作るときは `descript.txt` の `homeurl` を消すか自分の更新の置き場所に直すことを、hello-pasta から写す手順に近い場所に書く。
2. The マニュアル shall hello-pasta のネットワーク更新で、手元で書き換えた辞書・`pasta.toml` が上書きされうることを、実機確認（Requirement 10.3）の結果に合わせて、ダウンロードのページのサンプルゴーストの節と、hello-pasta から写す手順（`getting-started/setup.md`）の 2 か所に短く書く。
3. When `pasta_check validate` が、`name` が `hello-pasta` でないゴーストの `descript.txt` の `homeurl` を調べ、それが hello-pasta の更新の置き場所を指していることを見つける, the `pasta_check validate` shall 警告（注意）を出す。
4. If `descript.txt` の `name` が `hello-pasta` である、または `homeurl` が無い、または `homeurl` が hello-pasta の更新の置き場所を指さない、または `descript.txt` が無い・読めない, then the `pasta_check validate` shall この警告を出さず、読めないことを誤りにしない。`name` が無いゴーストは `hello-pasta` でないものとして扱う。
5. The `pasta_check validate` shall この警告を、検査の結果の分類の 1 つとして固定の重さ（注意）で出し、オプションや状況で変えない。
6. The `pasta_check validate` shall `homeurl` が hello-pasta の置き場所を指すかの判定で、末尾の `/` の有無・大文字小文字・`http` と `https` の違いを同じとして扱い、置き場所の配下のパスも指しているものとみなす。
7. If この警告だけが出る, then the `pasta_check validate` shall 終了コードを、既存の警告（注意）だけのときと同じにする。
8. The この警告 shall `pasta_check validate`（および内部で `validate` を行う `pasta_check release`）で出し、VS Code 拡張・LSP・pasta-recipe では出さない。

### Requirement 9: 手順書とマニュアルの更新

**Objective:** As a 開発者または AI, I want 手順書が置き場所を含むリリースの流れを正しく書いていること, so that 次のリリースで迷わず進められる

#### Acceptance Criteria

1. The `crates/pasta_sample_ghost/RELEASE.md` shall リリース CI のジョブ表に置き場所の更新のジョブを加え、報告の表の新しい行の読み方と、失敗時の回復（Requirement 6）を書く。
2. The `release` スキル shall 置き場所の更新を、リリースの確認の対象と最終報告の内容に含める。
3. The `.github/release-ci-setup.md` shall 置き場所の更新に必要な権限と、`github-pages` の environment が `main` だけに公開を許すことを書く。
4. The マニュアルのダウンロードのページ shall サンプルゴーストの節に、hello-pasta がネットワーク更新に対応していることを書く。
5. The マニュアル shall 章立て・見た目を変えず、触れる範囲をダウンロードのページへの案内と、派生ゴーストの注意の 1 節に限る。
6. The `crates/pasta_sample_ghost/RELEASE.md` shall ファイルを消す・改名する版を出すときは（SSP の更新が古いファイルを消さないため）別途検討することを 1 行で書く。
   - 置き換わった: `hello-pasta-book-sync` の手順書の「hello-pasta の更新の置き場所」に、置き場所の作りの変更に合わせて書き直した。

### Requirement 10: 実機（SSP）での確認

**Objective:** As a 開発者, I want 実際の SSP で更新が通ることを確かめたい, so that 机上でなく実際の利用者の更新が動くと言える

#### Acceptance Criteria

1. When ローカルの HTTP サーバーに更新の置き場所を作り、別ポートの別の SSP インスタンスで更新確認を行う, the hello-pasta shall `pasta.dll` を含む一式を、置き場所の版に差し替える。
2. While SSP が `pasta.dll` を読み込んでいる, the SSP shall 更新確認で `pasta.dll` を差し替えられる。*（pasta では未確認。この実機確認で確かめる。）*
3. When 手元で書き換えた辞書または `pasta.toml` がある状態で更新確認を行う, the 実機確認 shall 上書きされるかどうかの結果を記録し、Requirement 8.2 の注意の根拠にする。
4. The 実機確認 shall 公開サイト（GitHub Pages）の実際の URL で `.dll`・`.pasta`・`.png` が変換されずに返ることも確かめる。公開は `main` からしかできないので、この確認は PR のマージ直後に行って記録する。
5. If 実機確認（10.1〜10.3）で更新が通らない, then the 完了の判定 shall この仕様を完了としない。*（部分的な出荷はせず、原因を直すかリジェクトする。）*
6. If マージ直後の実 URL の確認（10.4）が通らない, then the リリース手順 shall 次のリリースを出さず、直す PR を先に入れる。（マージ直後の公開は前の版の nar の展開で、新しい `homeurl` を持つ利用者がいないため、利用者には影響しない。リリース手順書のリリース前の確認にもこの順を書く。）

### Requirement 11: 既存の保証の維持

**Objective:** As a リリースに関わる者, I want 既存の検査・配布物を壊さないこと, so that この仕様の変更で別のリリースの保証が崩れない

#### Acceptance Criteria

1. The リリース CI の変更 shall `release.yml` の 1 行 1 つの `release/<名前>` の添付の行の書き方を保ち、最新版の固定リンクの名前の検査を壊さない。
2. The Release の添付 shall 名前と数を変えない。
3. The `pasta_check release` shall `updates.txt` の作り方を変えない。
4. The 置き場所の更新 shall nar の中身の構成（ファイルの数・種類・arch ごとの `pasta.dll`）に依らず、nar を展開して `updates.txt` と照らす形で動く。
5. When `release.yml` を変更する, the マニュアル公開 CI shall 既存の起動条件のとおり検証（ビルド・検査）を実行し、失敗したら公開しない。

---

## 決定事項（要件ディスカッション 2026-11-15・全議題を推奨案で確定）

1. **展開の範囲**: nar 全体を展開する。`bal/` の除外規則は持たない（Requirement 1.5）。
2. **Release・nar が取れないとき**: 失敗して公開を止める。PR・ブランチでも同じく失敗にする。`updates.txt` の無い nar も失敗。`homeurl` を足した版が出る前は、前の版の nar を置く（Requirement 4.4・4.8）。
3. **上書き注意の置き場所**: ダウンロードのページのサンプルゴースト節と `setup.md` の 2 か所。文言は実機確認の結果に合わせる（Requirement 8.2）。
4. **公開後の見比べ**: `updates.txt` 1 本の一致。失敗は報告の表の 1 行を `failed` にするだけで、他の公開先は変えない。取り直しの値は設計（Requirement 5）。
5. **古いタグの再実行**: 起動の前に判定し、更新せず `skipped`（理由: 最新ではないタグ）を表に出す（Requirement 6.3）。
6. **派生ゴースト警告**: `pasta_check validate`（と `release`）だけ。エディタ系は対象外。`name` 無しは警告・`descript.txt` が読めなければ黙る・正規化は末尾の `/`・大文字小文字・http/https・配下のパス（Requirement 8）。
7. **入口ページ**: 短い案内とリンクだけの静的なページ。自動では移さず、`updates.txt` に載せない（Requirement 7）。
8. **`delete.txt`**: 対象外。手順書に別途検討の 1 行（Requirement 9.6）。
9. **版**: 要件に書かず、ロードマップで決める。`pasta-dll-multiarch` と同じウェーブに置かない。
10. **確認事項のまま進めるもの**: SSP が動作中の `pasta.dll` を差し替えられるか（Requirement 10.2 の実機で確かめる。通らなければ未完了）。`actions` 権限を新ジョブだけに足せるか（設計で確かめる。足せないときも公開を 1 か所に保つ制約は譲らない）。

## 設計フェーズへ回す事項（`research.md` の「設計判断の項目」と同じ。`/kiro-design` で解決する）

展開・検査のロジックの置き場所と言語（公開前と公開後で共有するか）／起動と待ちの方式と run の特定／起動した run が `cancelled` になったときの扱い／新ジョブの構成と `report` への組み込み／見比べの取り直しの値／`release` スキルの完了判定（7 → 8 つか別枠か）／`index.html` の元ファイルの置き場所／照合の細部（`ghost/master/updates.txt`・`date=`・パスの安全性）／警告の実装（`descript.txt` を読む入口・`FindingKind`・URL 定数の持ち方・`validate.md` とスキル生成物）／`integration_test.rs:173` の直し／`manual.yml` の起動条件／`homeurl` の方式（`descript.txt` のみ）。

## 設計ディスカッションでの追加決定（2026-11-15）

- Release 公開前に、作った nar を置き場所と同じ照合に通す関門を `release.yml` の `build` ジョブに足す（Requirement 3.7・6.5）。
- 実 URL の確認 10.4 は `main` へのマージ直後に行う。通らなければ次のリリースを出さず修正 PR を先に入れる（Requirement 10.4・10.6）。実機確認 10.1〜10.3 はマージ前（Requirement 10.5）。
