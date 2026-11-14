# Requirements Document

## Project Description (Input)

pasta-recipe（VS Code を使わない pasta DSL のエディタ）は、Windows x64 の `pasta-recipe.zip` 1 つしか配っていない。arm64 の Windows（Snapdragon の機械など）を使うゴースト作者は、x64 の exe をエミュレーションで動かすしかなく、動くかどうかも確かめていない。開発者は、x64 版と arm64 版をどちらもリリース CI で作り、`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip` として配りたい。x64 も含めて、リリース CI のビルド・検査・添付が 2 つの zip でそろったところまでを、この spec の完成とする（`brief.md`。2026-11-14 の `/kiro-discovery`）。

## Introduction

この spec は、pasta-recipe の配布物を arch ごとの 2 つの zip（`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip`）に分け、手元の作成・リリース CI の作成と検査・GitHub Release の添付・リリースの状態の集計・main の CI の見張り・利用者向けの文書を、2 つの新しい名前にそろえる。`pasta-recipe.zip` の名前は一度も出さない。

**前提（仮置き。OPEN QUESTION 1）**: 2 つの zip を初めて添付する版は **0.5.3** とする（`brief.md` の推奨）。`pasta-recipe.zip` は 0.5.2 までのどの版にも添付されておらず、0.5.3 から添付する予定だった。0.5.3 に入れれば、古い名前の互換と、過去の版の添付の数え方の追加は要らない。

用語:

- **arch**: 配る Windows の種類。`x64`（64 ビットの Intel・AMD）と `arm64`（64 ビットの ARM）の 2 つ。
- **2 つの zip**: `pasta-recipe.x64.zip` と `pasta-recipe.arm64.zip`。
- **配布物の作成スクリプト**: `editors/recipe/scripts/package.ps1`（手元とリリース CI が使う）。
- **リリース CI**: リリースタグの push で動く `.github/workflows/release.yml`。
- **main の CI**: push・PR・手動の起動で動き、リリース CI の関門としても呼ばれる `.github/workflows/build.yml`。
- **機械の種類**: exe の PE ヘッダにある、その exe が動く CPU の種類（x64 = `0x8664`、arm64 = `0xAA64`）。

## Boundary Context

- **In scope**:
  - arch ごとの pasta-recipe の exe とライセンスの表示の作成と、2 つの zip への詰め合わせ（手元とリリース CI）。
  - arm64 版の exe も VC++ ランタイムに頼らない（静的 CRT）こと。
  - リリース CI の 2 つの zip の検査（中身・版・機械の種類・大きさ）と、GitHub Release への添付（添付は 8 つ）。
  - `release` スキルの状態の集計と手順書の添付の数え方（8 つ）。
  - main の CI で arm64 のビルドの壊れを見つけること。
  - タグを打たずに、2 つの zip の作成と検査をリリース CI と同じ形で確かめられること。
  - `pasta-recipe.zip` の名前を使っていた所（`completed/` の spec の記録を除く）を、2 つの新しい名前にそろえること。マニュアルと README での arch の選び方。
- **Out of scope**:
  - x86（32 ビット）・mac・Linux の pasta-recipe。
  - インストーラ・コード署名・自動更新。
  - pasta.dll・見本ゴーストの `.nar`・`release.ps1` の arch（`pasta-dll-multiarch` が持つ）。
  - pasta-recipe の画面と機能の変更（`pasta-recipe`・`pasta-recipe-dev-actions` が作ったもの）。
  - リリースタグを打って本当のリリースを行うこと（`release` スキルが持つ）。
  - arm64 の exe を arm64 の機械で起動して確かめること（**仮置き**。範囲に入れるかは OPEN QUESTION 3 で決める）。
- **Adjacent expectations**:
  - `pasta-dll-multiarch` も、ルートの静的 CRT の設定に arm64 の 1 段を要る。先に入った方が足し、後の方は在るのを確かめる。この spec は、その段がどちらの spec で入っても、arm64 版の exe が静的 CRT で作られることを求める。
  - `completed/` の下の spec の記録（`completed/pasta-recipe/` など）は、過去の決定の記録として書き換えない。
  - 実際の 0.5.3 のリリースでの添付の確かめは、`release` スキルが行う。
- **完成の定義**: x64 と arm64 の 2 つの zip が、タグを打たない確かめの実行（Requirement 7）で作成と検査を通った記録と、GitHub Release の添付の自己テスト（4.5・4.6）が通った記録がそろったとき、この spec は完成とする。
- **作業の制約**: `.claude/skills/` の下（`release` スキル・`pasta-check` スキル）の書き換えが作業の環境の分類器に拒まれたときは、回避の手段を探さず、入った分と残りを示して開発者の許可を求める。

## Requirements

### Requirement 1: arch ごとの 2 つの配布物

**Objective:** As a ゴースト作者, I want 自分の Windows の arch に合う pasta-recipe の zip を落とせること, so that arm64 の機械でもエミュレーションに頼らずに pasta-recipe を使える

#### Acceptance Criteria

1. When リリースタグ `vX.Y.Z` が push されたとき, the リリース CI shall `pasta-recipe.x64.zip` と `pasta-recipe.arm64.zip` の 2 つを、そのタグのソースから作る。
2. The リリース CI shall 2 つの zip のどちらにも、`pasta-recipe.exe` と `THIRD_PARTY_LICENSES.txt` の 2 つだけを入れる。
3. The リリース CI shall 各 zip の `THIRD_PARTY_LICENSES.txt` を、その zip の arch の exe に入っているネイティブの部品と、画面の部品の両方の表示にする。
4. The リリース CI shall 2 つの zip の `pasta-recipe.exe` の版を、タグの版（`X.Y.Z`）にする。
5. The リリース CI shall `pasta-recipe.x64.zip` の exe を機械の種類が x64 の実行ファイルに、`pasta-recipe.arm64.zip` の exe を機械の種類が arm64 の実行ファイルにする。
6. The リリース CI shall 2 つの zip の exe を、VC++ の再頒布可能パッケージを入れていない Windows でも起動できる形（ランタイムを exe に静的に入れた形）で作る。
7. The リリース CI shall `pasta-recipe.zip` の名前の配布物を作らず、どこにも添付しない。

### Requirement 2: リリース CI の配布物の検査

**Objective:** As a 開発者, I want 2 つの zip の取り違えや壊れをリリース CI が公開の前に見つけること, so that 間違った arch や版の exe を作者に配らない

#### Acceptance Criteria

1. When リリース CI が 2 つの zip を作り終えたとき, the リリース CI shall 公開の job へ進む前に、2 つの zip のそれぞれについて、中身・exe の版・exe の機械の種類・ライセンスの表示・大きさを検査する。
2. If どちらかの zip が無い、または中身が `pasta-recipe.exe`・`THIRD_PARTY_LICENSES.txt` の 2 つでないとき, the リリース CI shall その zip の名前と見つけた中身を示して失敗する。
3. If どちらかの zip の exe の版がタグの版と違うとき, the リリース CI shall その zip の名前と exe の版を示して失敗する。
4. If どちらかの zip の exe の機械の種類が、その zip の名前の arch と違うとき, the リリース CI shall その zip の名前と、期待した機械の種類と実際の機械の種類を示して失敗する。
5. If どちらかの zip の `THIRD_PARTY_LICENSES.txt` に、ネイティブの部品（`tauri`）と画面の部品（`monaco-editor`）のどちらかの名前が無いとき, the リリース CI shall その zip の名前と欠けた名前を示して失敗する。
6. If どちらかの zip が 30 MB（31,457,280 バイト）を超えるとき, the リリース CI shall その zip の名前と大きさを示して失敗する。上限は zip ごとに当てる。
7. When 検査で複数の問題が見つかったとき, the リリース CI shall 最初の 1 つで止めずに、見つけた問題をすべて示してから失敗する。
8. If 配布物の検査が失敗したとき, the リリース CI shall crates.io・Marketplace・GitHub Release のどこにも公開しない。
9. If 2 つの zip を作る間に、ルートまたはアプリの `Cargo.lock` が書き換わったとき, the リリース CI shall 失敗する。
10. When 2 つの zip が検査を通ったとき, the リリース CI shall 2 つの zip を、ほかの配布物と同じく後続の公開の job へ渡す。
11. If どちらかの zip の exe が VC++ ランタイムの DLL（`vcruntime*.dll`・`msvcp*.dll` など）を読み込む形で作られているとき, the リリース CI shall その zip の名前と読み込む DLL の名前を示して失敗する。（**仮置き**。検査に入れるかは OPEN QUESTION 2 で決める）

### Requirement 3: 手元での 2 つの zip の作成

**Objective:** As a 開発者, I want 手元でもリリース CI と同じ 2 つの zip を作れること, so that リリースの前に手元で中身を確かめられる

#### Acceptance Criteria

1. When 開発者が配布物の作成スクリプトを arch `x64` を指定して実行したとき, the 配布物の作成スクリプト shall リポジトリの根の `release/pasta-recipe.x64.zip` を作る。
2. When 開発者が配布物の作成スクリプトを arch `arm64` を指定して実行したとき, the 配布物の作成スクリプト shall リポジトリの根の `release/pasta-recipe.arm64.zip` を作る。
3. When 開発者が配布物の作成スクリプトを arch を指定せずに実行したとき, the 配布物の作成スクリプト shall 2 つの zip をどちらも作る。（**仮置き**。OPEN QUESTION 4）
4. The 配布物の作成スクリプト shall 作った zip の中身が `pasta-recipe.exe`・`THIRD_PARTY_LICENSES.txt` の 2 つだけであることを自分で確かめる。
5. If 作成のどの段かが失敗したとき, the 配布物の作成スクリプト shall 終了コード 1 で止まり、その arch の作りかけの zip を残さない。
6. If `x64`・`arm64` 以外の arch が指定されたとき, the 配布物の作成スクリプト shall 受け付ける arch を示して終了コード 1 で止まり、zip を作らない。
7. When 一方の arch だけを作ったとき, the 配布物の作成スクリプト shall もう一方の arch の zip を消さない。
8. The 配布物の作成スクリプト shall 手元で作った zip と、リリース CI が作る zip を、同じ名前・同じ中身の並びにする。

### Requirement 4: GitHub Release の添付

**Objective:** As a 開発者, I want GitHub Release に 2 つの zip がそろって添付されること, so that 作者がリリースのページと固定のリンクから arch ごとに落とせる

#### Acceptance Criteria

1. When リリース CI が GitHub Release を作るとき, the GitHub Release の作成スクリプト shall `pasta.dll.zip`・`hello-pasta.nar`・`pasta-vscode-X.Y.Z.vsix`・`pasta_check.zip`・`pasta-skills.zip`・`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip` の 7 つの配布物と、詳しいリリースノート `release-notes-vX.Y.Z.md` の、合わせて 8 つを添付する。
2. If 添付する 8 つのうちどれかが手元に無いとき, the GitHub Release の作成スクリプト shall GitHub の操作より前に、無いものの名前を示して終了コード 1 で止まる。
3. While GitHub Release が公開済みで 8 つがそろって添付されているとき, the GitHub Release の作成スクリプト shall 何もせずに飛ばしたと示す。
4. The GitHub Release の作成スクリプト shall 添付の数を示す出力（飛ばした理由など）を、8 つ（配布物 7 つと詳しいリリースノート）に合わせる。
5. The リリースのスクリプトの自己テスト shall `pasta-recipe.x64.zip` だけが無いとき、`pasta-recipe.arm64.zip` だけが無いときのそれぞれで、GitHub Release の作成スクリプトが終了コード 1 で止まり、無い方の名前だけを示すことを確かめる。
6. The リリースのスクリプトの自己テスト shall ほかの配布物の 1 つだけが無いときの既存の試験を、8 つの添付に合わせて保つ。

### Requirement 5: リリースの状態の集計と手順書

**Objective:** As a リリースを行うエージェント, I want リリースの状態の集計が 8 つの添付で数えること, so that 0.5.3 以降のリリースの完了と未完了を正しく見分けられる

#### Acceptance Criteria

1. When `release` スキルの状態の集計が版 V の GitHub Release を調べるとき, the 状態の集計 shall 4.1 の 8 つの名前のうち添付済みのものを数え、「公開・添付 n/8」の形で示す。
2. When 版 V の GitHub Release が公開済みで 8 つがそろっているとき, the 状態の集計 shall 「あり（公開・添付 8/8）」と示し、公開済みに数える。
3. While 調べる版が 0.5.2 以下のとき, the `release` スキルの手順書 shall 2 つの zip の無い過去の版（0.4.2〜0.5.2 は 6/8、0.4.1 以下は 4/8）を、完了したリリースとして読み替える決まりを示す。
4. The `release` スキルの手順書 shall 状態の表と「公開済み」の数え方の説明を、8 つの添付（配布物 7 つと詳しいリリースノート）と、`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip` は 0.5.3 から添付することに合わせる。

### Requirement 6: main の CI での arm64 の見張り

**Objective:** As a 開発者, I want arm64 の pasta-recipe が作れなくなったことを main の CI で見つけること, so that リリースの日に初めて arm64 の壊れに気づくことが無い

#### Acceptance Criteria

1. When main の CI が push・PR・手動の起動で動くとき, the main の CI shall arm64 の pasta-recipe の exe を作り、作れなければ失敗する。
2. The main の CI shall pasta-recipe の x64 の型の検査・テスト・静的解析・exe の作成を、今のとおり続ける。
3. When リリース CI が関門として main の CI を呼ぶとき, the main の CI shall 6.1 の arm64 の作成も関門に含める。
4. Where arm64 のランナーで pasta-recipe の検査を行う形が選ばれたとき, the main の CI shall arm64 のランナーで pasta-recipe の Rust のテストを走らせる。（**仮置き**。範囲に入れるかは OPEN QUESTION 3 で決める）

### Requirement 7: タグを打たない確かめ

**Objective:** As a 開発者, I want リリースタグを打たずに、2 つの zip の作成と検査をリリース CI と同じ形で確かめられること, so that x64 も含めた CI の作成と検査がそろったことを、本当のリリースの前に確かめて spec を完了できる

リリース CI は今はリリースタグの push でしか動かない。どの CI で、どう起動するかは OPEN QUESTION 5 で決める。

#### Acceptance Criteria

1. When 開発者が作業のブランチで確かめの実行を手動で起動したとき, the CI shall リリースタグを要らずに、2 つの zip を作り、Requirement 2 と同じ検査（中身・exe の版・機械の種類・ライセンスの表示・大きさ）を行う。版はその時点のリポジトリの版を使う。
2. The 確かめの実行 shall crates.io・Marketplace・GitHub Release のどこにも公開せず、GitHub Release を作らない。
3. The 確かめの実行とリリース CI shall 2 つの zip の作成と検査に、同じ手順を使う（片方だけを直すと食い違う形にしない）。
4. When 確かめの実行が成功したとき, the CI shall 2 つの zip を、開発者が落として中身を見られる形で残す。
5. The 確かめの実行 shall 作業のブランチで成功した実行の記録（実行の URL）を、spec の完了の証拠として残せる形にする。

### Requirement 8: 名前の付け替えと利用者向けの文書

**Objective:** As a ゴースト作者, I want マニュアルと README で、自分の Windows に合う zip を選んで落とせること, so that 迷わずに正しい arch の pasta-recipe を入れられる

#### Acceptance Criteria

1. The マニュアルの pasta-recipe のページ shall 2 つの zip の固定のダウンロードのリンク（`https://github.com/ekicyou/pasta/releases/latest/download/pasta-recipe.x64.zip` と `.../pasta-recipe.arm64.zip`）を示す。
2. The マニュアルの pasta-recipe のページ shall 自分の Windows の arch の調べ方（Windows の設定の「システムの種類」の見方）と、x64 の Windows は x64 版、arm64 の Windows は arm64 版を選ぶことを示す。
3. The マニュアルの pasta-recipe のページ shall 配布の対象を x64 と arm64 の Windows と書き、32 ビットの Windows・mac・Linux 向けが無いことを示す。
4. The README とマニュアルのはじめの章（`introduction.md`） shall pasta-recipe の配布物の固定のリンクを、2 つの zip の名前にする。
5. The リポジトリ shall `pasta-recipe.zip` を配布物の名前として使う所（リリースの手順書 `RELEASE.md`・steering の `tech.md`・`structure.md`・`roadmap.md`・`pasta-check` スキル・リリース CI・リリースのスクリプトとその自己テスト・`release` スキル・配布物の作成スクリプト）を、2 つの新しい名前にそろえる。
6. When 名前の付け替えが終わったとき, the リポジトリ shall 過去の記録（`completed/` の下の spec・各 spec の `brief.md` の調査の記録）と、名前を変えたことの説明（この spec の文書・`roadmap.md` のこの spec の行）を除き、`pasta-recipe.zip` の名前をどこにも含まない。
7. The steering の `tech.md`・`structure.md` shall リリース CI の配布物の数（7 つ）と添付の数（8 つ）、2 つの zip の中身と検査の項目（機械の種類を含む）を、変わった後の形で書く。
