# Requirements Document

## Project Description (Input)

hello-pasta のネットワーク更新の中身が、辞書を直しても変わらない。開発者の指示は「ネットワーク更新は book に同期する」である。完了済みの `hello-pasta-network-update` は、置き場所の中身を最新 Release の `hello-pasta.nar` の展開にしたため、main の辞書を直しても、リリースを出すまで置き場所は変わらない。そのため開発者は、辞書を直した直後に実機でネットワーク更新を確かめられない。

変えたいこと（開発者 2026-11-19）: 辞書が変わったら、`pasta.dll` が変わったら、ネットワーク更新に出る。置き場所は「main のリポジトリに在る hello-pasta のファイル」と「最新 Release の nar にだけ在るファイル（`pasta.dll` など）」を重ねて作る。`hello-pasta.nar` は今のままリリースのときにだけ作る。

詳細な背景・退けた案・制約は `brief.md` を参照。

## Introduction

この仕様は、hello-pasta のネットワーク更新を、main の中身に合わせる。辞書・シェル・設定など、リポジトリに在るファイルは、main に入ったときに置き場所に出る。`pasta.dll` など、ビルドで作るファイルは、リリースのときに置き場所に出る。更新はファイル単位で、変わったファイルだけが、変わったときに届く。

開発者は、辞書を直してマージした後、手元の SSP の更新確認で、その変更が届くことを確かめられる。利用者は、新しい `pasta.dll` が要る辞書を、古い `pasta.dll` の手元に受け取らない。

## Boundary Context

- **In scope**:
  - 置き場所の中身を、2 つの出どころ（main のファイル・最新 Release の nar にだけ在るファイル）を重ねて作ること。
  - 重ねた結果から `updates.txt` を作り直し、公開前に照合すること。
  - マニュアル公開 CI の起動の条件に、hello-pasta の配下を足すこと。
  - 新しい `pasta.dll` が要る辞書を、古い dll と組で出さないこと。
  - 置き場所を古い中身に戻さないこと。
  - main で消した・改名したファイルを、置き場所に置かないこと。
  - リリース CI の公開後の見比べを、新しい作り方に合わせること。
  - 手順書と、`hello-pasta-network-update` の置き換わる項の記録。
  - 実機の SSP での確認（PR #288 の台詞の、最初の本番の確認を兼ねる）。
- **Out of scope**:
  - `hello-pasta.nar`・`pasta-starter.nar` と Release の添付の作り方・時機（リリースのときのまま）。
  - `pasta.dll` を main のコミットごとにビルドして配ること。
  - ゴースト専用のリリース・タグ。
  - 作者が自分のゴーストをネットワーク更新に対応させる方法の案内。
  - `pasta_check release` の `updates.txt` の作り方。
  - マニュアルの中身と、入門の章と辞書の照合（`tutorial-check`）。
  - hello-pasta の台詞・シェル。
  - `pasta.dll` の arch ごとの配布（`pasta-dll-multiarch`）。
  - 利用者の手元に残った、消した・改名したファイルを消させること（`delete.txt`）。`hello-pasta-network-update` の決定のまま、手順書の注意にとどめる（2026-11-19 議題 2）。
- **Adjacent expectations**:
  - `hello-pasta-network-update`（完了）: 置き場所の URL・`homeurl`・入口のページ・照合・公開の経路・報告の表・失敗からの回復は、そのまま使う。置き場所の中身を「最新 Release の nar の展開」とする項と、公開後の見比べの項だけを、この仕様が置き換える。
  - `pasta-check-bundled-balloon`（完了）: `updates.txt` の書式と、載せるファイルの決まり。この仕様はそれに合わせる側で、変えない。
  - `release-ci`（完了）: Release の公開と、その後にマニュアル公開 CI を main で起動する流れ。この仕様は流れを変えない。
  - `pasta-dll-multiarch`（未完了）: nar に arch ごとの dll を足す。足されたファイルは、この仕様の変更なしに「nar にだけ在るファイル」として置かれる。`release.yml` を触るので、同じウェーブに置かない。
  - 公開サイト（GitHub Pages）は、サイト全体を 1 回で丸ごと入れ替える。置き場所とマニュアルは同じサイトに載る。

## Requirements

### Requirement 1: 置き場所の中身（2 つの出どころを重ねる）

**Objective:** As a 開発者, I want main の辞書を直してマージしたら、置き場所の辞書もそのコミットのものになること, so that リリースを待たずに、実機でネットワーク更新を確かめられる

#### Acceptance Criteria

1. When マニュアル公開 CI が main のコミットから置き場所を作る, the 更新の置き場所 shall リポジトリの `crates/pasta_sample_ghost/ghosts/hello-pasta/` に在るファイルを、そのコミットの内容のまま、同じフォルダ構造で置く。
2. The 更新の置き場所 shall 最新の Release の `hello-pasta.nar` に在り、その Release のタグのツリーに無いファイル（ビルドで作ったもの。2026-11-19 時点では `ghost/master/pasta.dll`・`ghost/master/THIRD_PARTY_LICENSES.txt`・`ghost/master/scripts/README.md` の 3 本）を、nar の中身のまま置く。
3. If nar に在るファイルが、その Release のタグのツリーにも在る, then the 更新の置き場所 shall それをリポジトリの側とし、公開するコミットに在ればその内容を置き、無ければ置かない。
4. The マニュアル公開 CI shall どちらの出どころから置くかを「最新の Release のタグのツリーに在るか」だけで決め、ファイル名の一覧を決め打ちで持たない。（公開するコミットで消したファイルが、nar の側として置かれないため。）（nar に arch ごとの `pasta.dll` などが足されても、この仕様の変更なしに nar の側として置かれる。）
5. If リポジトリに在るファイルが、`pasta_check release` が `updates.txt` に載せない種類のもの（`updates.txt`・`updates2.dau`・`developer_options.txt`）である, then the 更新の置き場所 shall それを置かず、`updates.txt` にも載せない。nar の `updates.txt`・`ghost/master/updates.txt` も、nar の側として置かない（ルートの `updates.txt` は Requirement 2 で作り直す。`ghost/master/updates.txt` の扱いは設計で決める）。
6. While 2 つのリリースの間で、リポジトリに在るファイルだけが変わる, the 更新の置き場所 shall nar の側のファイル（`pasta.dll` など）を、最新の Release の nar と同じ内容のまま置き続ける。（利用者が、辞書の更新のたびに `pasta.dll` を取り直さない。）
7. The 入口のページ（`hello-pasta-network-update` 要件 7） shall 今のとおり置き場所の直下に置き、`updates.txt` に載せない。
8. The 更新の置き場所 shall リポジトリの側の文字のファイルを、nar と同じ改行（CRLF）で置く。（リリースのコミットから作った置き場所では、リポジトリの側の各ファイルの md5 が、その Release の nar と一致する。改行が違うと、中身が同じでも利用者が全部を取り直す。）

### Requirement 2: `updates.txt` の作り直しと公開前の照合

**Objective:** As a hello-pasta の利用者, I want 置き場所の `updates.txt` が、置かれたファイルと正しく合っていること, so that SSP が変わったファイルだけを取り、壊れた一式を受け取らない

#### Acceptance Criteria

1. When 置き場所の中身を重ねて作る, the マニュアル公開 CI shall 重ねた結果から `updates.txt` を作り直す。
2. The 作り直した `updates.txt` shall 同じファイル一式から `pasta_check release` が作るものと、載るファイル・各行の md5・大きさが同じで、同じ書式である。日付の欄の違いは、違いとしない。
3. Where 作り直しを `pasta_check release` 以外の処理で行う, the 置き場所の組み立ての自己テスト shall 同じファイル一式で、作り直した `updates.txt` と `pasta_check release` の出力が 2.2 の意味で一致することを確かめる。（作り方が 2 つになるので、ずれを見逃さないため。）
4. When `updates.txt` を作り直す, the マニュアル公開 CI shall 公開の前に、今と同じ照合（全行の md5・大きさと、置いたファイルの照らし合わせ）を通す。
5. If 照合で、行が指すファイルが無い、または md5・大きさが合わない, then the マニュアル公開 CI shall どのファイルが合わないかを表示して失敗し、サイトを公開しない。
6. If 最新の Release が無い、または `hello-pasta.nar` が無い・取れない、または nar に `updates.txt` が無い, then the マニュアル公開 CI shall 今のとおり失敗し、置き場所を欠いたサイトを公開しない。PR・ブランチでの実行でも同じく失敗にする。
7. While PR またはブランチで実行されている, the マニュアル公開 CI shall 公開を行わずに、重ねる・作り直す・照合するところまでを実行する。

### Requirement 3: 公開のきっかけ

**Objective:** As a 開発者, I want 辞書が変わっても、`pasta.dll` が変わっても、置き場所に出ること, so that どちらの変更も、利用者の更新確認で届く

#### Acceptance Criteria

1. When hello-pasta の配下（`crates/pasta_sample_ghost/ghosts/hello-pasta/` の下のどのファイルでも）を変えるコミットが main に入る, the マニュアル公開 CI shall 走り、そのコミットの中身で置き場所を作り直して公開する。（今の起動の条件は `dic/**` と `calendar.csv` だけである。シェル・`pasta.toml`・`descript.txt`・`install.txt` などを足す。）
2. When hello-pasta の配下を変える PR が出る, the マニュアル公開 CI shall 走り、公開せずに Requirement 2 の作り直しと照合までを行う。
3. When hello-pasta と無関係な、今の起動の条件のファイル（book など）を変えるコミットが main に入る, the マニュアル公開 CI shall 今のとおり走り、置き場所も、そのコミットの hello-pasta の中身で作り直して公開する。
4. When リリース CI が新しい Release を公開する, the リリース CI shall 今のとおりマニュアル公開 CI を main で起動し、置き場所の nar の側のファイルを、新しい Release の nar のものに入れ替える。
5. When マニュアル公開 CI が main で手動実行される, the マニュアル公開 CI shall その時点の main のコミットと、最新の Release の nar から置き場所を作り直して公開する。

### Requirement 4: 新しい `pasta.dll` が要る辞書

**Objective:** As a hello-pasta の利用者, I want 手元の `pasta.dll` で読めない辞書が、更新で届かないこと, so that 更新の後に、ゴーストが起動しない・誤りを出すことがない

#### Acceptance Criteria

1. When 置き場所の中身を作る, the マニュアル公開 CI shall 重ねた中身を、最新の Release の版の `pasta_check` で検査する。
2. If 重ねた中身が、最新の Release の版の検査で誤りになる, then the マニュアル公開 CI shall どのファイルの何が誤りかを表示して失敗し、サイト（マニュアルと置き場所）を公開しない。（置き場所とマニュアルは 1 回の公開で丸ごと入れ替わるので、置き場所だけを止めることはしない。古い中身で出し直すと、すでに届いた辞書を古いものに戻すので、その道も採らない。）
3. When その辞書を読める `pasta.dll` を持つ Release が公開される, the マニュアル公開 CI shall 手作業なしに検査に通り、置き場所を公開する。
4. While PR またはブランチで実行されている, the マニュアル公開 CI shall 同じ検査を行い、誤りなら失敗にする。
5. The 検査 shall 最新の Release の版の pasta で読める辞書の変更を止めない。
6. The 手順書 shall この検査が Lua を動かさない検査で、実行の結果だけが変わる違い（新しい日時変数の値が空になるなど）は止められないことを書く。

### Requirement 5: 古い中身に戻さない

**Objective:** As a hello-pasta の利用者, I want 更新で、手元にあるものより古い中身に戻されないこと, so that 入れた版や、すでに届いた辞書が、更新で巻き戻らない

#### Acceptance Criteria

1. When 最新の Release の `hello-pasta.nar` で入れた直後の利用者が更新確認をする, the 更新の置き場所 shall その nar より古いファイルを返さない。（置き場所のリポジトリの側は、その Release 以降の main のコミット、nar の側は、その Release のものである。）
2. If マニュアル公開 CI が公開しようとする中身が、main の最新のコミットの中身より古い（古い run の再実行・追い越された run など）, then the マニュアル公開 CI shall それを公開せず、理由を表示して、失敗にしない。
3. The 更新の置き場所 shall nar の側のファイルを、最新の Release より古い Release のものにしない。
4. When 5.2 で公開を飛ばす, the マニュアル公開 CI shall main の最新の中身が公開されないままにならないようにする。（新しいコミットがマニュアル公開 CI の起動の条件に当たらないときも、取りこぼさない。）

### Requirement 6: リリース CI の公開後の見比べ

**Objective:** As a 開発者, I want リリースの後、実際の URL の置き場所に、その Release の `pasta.dll` などが出たことを CI が確かめて報告すること, so that 公開後に、利用者が新しい dll を受け取れると分かる

#### Acceptance Criteria

1. When リリース CI が起動したマニュアル公開 CI の公開が完了する, the リリース CI shall 実際の URL の `updates.txt` を取り、その Release の nar の側のファイル（Requirement 1.2 の決め方で決まるもの）の行が、nar の `updates.txt` の行と md5・大きさで一致することを確かめる。
2. The リリース CI shall リポジトリの側のファイルについては、md5 の一致を求めない。（Release のタグの後に main へ入った辞書の変更で、違いうるため。リポジトリの側は、公開前の照合 Requirement 2 で確かめ済み。）
3. The リリース CI shall `updates.txt` の日付の欄の違いを、不一致としない。
4. If 見比べる行が一致しない、または実際の URL の `updates.txt` が取れない, then the リリース CI shall 結果を `failed` として報告の表に出し、他の公開先の status を変えない。
5. The リリース CI shall 報告の表の置き場所の行の書式・取り直しの回数と間隔・最新ではないタグのときの `skipped` を、今のとおりに保つ。

### Requirement 7: 消えたファイル

**Objective:** As a hello-pasta の利用者, I want main で消した・改名したファイルが、更新で新しく届かないこと, so that 消したはずの辞書を新しく受け取らない

#### Acceptance Criteria

1. When main で hello-pasta のファイルを消す・改名する, the 更新の置き場所 shall 消したファイル・改名の前の名前のファイルを置かず、`updates.txt` に載せない。
2. The 更新の置き場所 shall 利用者の手元のファイルを消させる指示（SSP の `delete.txt`）を出さない。（`hello-pasta-network-update` の決定のまま。利用者の手元に残った古いファイルは、pasta に読まれ続けうる。）

### Requirement 8: 既存の約束を保つ

**Objective:** As a リリースとマニュアルに関わる者, I want この仕様の変更で、既存の配布物・権限・検証を崩さないこと, so that ネットワーク更新を直しても、別の保証が壊れない

#### Acceptance Criteria

1. The `hello-pasta.nar`・`pasta-starter.nar`・Release の添付 shall 名前・数・中身・作る時機（リリースのときだけ）を変えない。
2. The `pasta_check release` shall `updates.txt` の作り方を変えない。
3. The マニュアル公開 CI shall 公開サイトへの公開を `deploy` ジョブ 1 か所で行い、ワークフロー全体の権限を `contents: read` のまま保ち、公開の権限を `deploy` ジョブにだけ与える。
4. The 公開サイト shall 1 回の公開で丸ごと入れ替わり、マニュアルと置き場所のどちらかを欠いた形で公開されない。
5. The マニュアル公開 CI shall 置き場所を作るために、`pasta.dll` のビルドや Windows の実行環境を要さない。
6. While キャッシュが効いている, the マニュアル公開 CI shall 置き場所の組み立てと Requirement 4 の検査で増える所要の時間を、今（40〜50 秒）から 1 分以内にとどめる。
7. The この仕様の変更 shall マニュアルの中身・章立てと、入門の章と辞書の照合を変えない。

### Requirement 9: 手順書と記録の更新

**Objective:** As a 開発者または AI, I want 手順書と前の仕様が、新しい置き場所の作り方を正しく書いていること, so that 次のリリースや、止まったときに迷わない

#### Acceptance Criteria

1. The `crates/pasta_sample_ghost/RELEASE.md` shall 置き場所が「main のリポジトリのファイル＋最新の Release の nar にだけ在るファイル」で作られること、出る 2 つのきっかけ（main への push・リリース）、Requirement 4 で止まったときの読み方と直し方（リリースを出す・辞書を戻す）と、検査で止められない違い（Requirement 4.6）、Requirement 6 の見比べの読み方を書く。
2. The `release` スキル shall リリースの確認と最終報告の内容を、Requirement 6 の見比べに合わせる。
3. The `.github/release-ci-setup.md` shall 置き場所の作り方の記述を、新しい作り方に合わせる。
4. The `hello-pasta-network-update` の要件 shall 置き換わる項（置き場所を最新の Release の nar の展開とする項と、公開後の見比べの項。要件 1.2・1.4・4.3・4.8・5.1・9.6 など）に、この仕様で置き換わったことを記録する。
5. The 手順書 shall 消した・改名したファイルが利用者の手元に残り、pasta に読まれ続けうることを、今の注意のまま書く（Requirement 7.2）。

### Requirement 10: 実機（SSP）での確認

**Objective:** As a 開発者, I want 実際の SSP で、main の辞書の変更が更新で届くことを確かめたい, so that 机上でなく、利用者の更新が動くと言える

#### Acceptance Criteria

1. When v0.6.2 の `hello-pasta.nar` を入れた SSP（別ポートの別のインスタンス）で、この仕様の後の置き場所に対して更新確認を行う, the hello-pasta shall main の辞書に入れ替わり、`OnUpdateComplete`（`changed`）の台詞を出す。（PR #288 の台詞の、最初の本番の確認を兼ねる。）
2. The 実機確認 shall その更新で取られたファイルが変わったファイルだけで、`pasta.dll` を取り直していないことを、SSP の更新のログで確かめて記録する。
3. When 置き場所と手元の中身が同じ状態で更新確認を行う, the 実機確認 shall SSP が `No Update` になることを確かめる。
4. The 実機確認 shall ローカルの HTTP サーバーの置き場所での確認をマージの前に、公開サイト（GitHub Pages）の実際の URL での確認を main へのマージの直後に行って記録する。（公開は main からしかできないため。）
5. If 実機確認が通らない, then the 完了の判定 shall この仕様を完了としない。（部分的な出荷はせず、原因を直すかリジェクトする。）
