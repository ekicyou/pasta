# Requirements Document

## Project Description (Input)
（brief.md より）バルーンを同梱して配るゴーストの作者と利用者は、`pasta_check release` がゴーストの `updates.txt`（ルートと `ghost/master` の両方）に同梱バルーンのファイルまでゴースト相対のパスで載せてしまうため、ネットワーク更新のたびにゴーストフォルダ内へゴミのフォルダが増え、本来のバルーンは更新されないという問題を抱えている。現状 `pasta_check` は `install.txt` を読まず、同梱バルーンの概念を持たない。`install.txt` の `balloon.source.directory`（無ければ `balloon.directory`）で同梱バルーンのフォルダを決め、そのフォルダ配下をゴースト用 `updates.txt` から除外し、同じ形式のバルーン用 `updates.txt` をバルーンのフォルダ直下に生成して nar に封入する。`homeurl` 欠落は警告。同梱バルーンが無いゴーストの出力は date を除き不変とする。

## Introduction

`pasta_check release` は、配布フォルダ（`--release`）の全ファイルをゴーストの `updates.txt` に載せている。ゴーストと一緒に配るバルーン（同梱バルーン）のファイルもゴースト相対のパスで載るため、ネットワーク更新でゴーストを更新すると、ゴーストのフォルダの中にバルーンのファイルが誤って置かれ、インストール済みのバルーン本体は更新されない（2026-10-02、areka alpha の実機ラップ A3r 項目 8 で発覚。emo2 の nar で 20 ファイルが `ghost/emo2/emo2-kakukaku/` へ誤配置）。

本仕様は、`pasta_check` が配布フォルダ直下の `install.txt` を読んで同梱バルーンのフォルダを判定し、そのフォルダ配下をゴーストの `updates.txt` から外し、同梱バルーンのフォルダ直下にバルーン用の `updates.txt` を生成して nar に入れるようにする。判定の根拠を、ベースウェアがインストール時の振り分けに使う `install.txt` と同じにすることで、配布物の形とネットワーク更新の対象を食い違わせない。

用語:
- **配布フォルダ**: `--release` で指定されたフォルダ。`--target` のコピーと `--copy` の上書きが済んだ後の状態を指す。
- **バルーン指定**: `install.txt` の番号なしの `balloon.source.directory`／`balloon.directory` の組と、番号付きの `balloon0.*`・`balloon1.*`… の各組。1 つのバルーン指定が 1 つの同梱バルーンを表す（ukadoc「install.txt」）。
- **同梱バルーン**: バルーン指定が指す、配布フォルダ内のフォルダ（配布フォルダからの相対パス。SSP 2.9.00 以降は `extra\bal1` のような階層付きの指定も許される）。
- **ゴースト用 `updates.txt`**: 配布フォルダ直下の `updates.txt` と、その複製である `ghost/master/updates.txt`。
- **バルーン用 `updates.txt`**: 同梱バルーンのフォルダ直下（そのバルーンの `descript.txt` と同じ階層。ukadoc「ネットワーク更新」の配置どおり）に生成する `updates.txt`。

## Boundary Context

- **In scope**:
  - 配布フォルダ直下の `install.txt` からの同梱バルーンのフォルダの読み取り（`balloon.source.directory`、無ければ `balloon.directory`。番号付きの `balloonN.*` も同じ規則）
  - ゴースト用 `updates.txt`（ルート・`ghost/master`）からの同梱バルーン配下の除外
  - 同梱バルーンのフォルダ直下へのバルーン用 `updates.txt` の生成と nar への封入
  - 同梱バルーンの `descript.txt` に `homeurl` が無いときの警告
  - 指定フォルダが存在しないとき・値が不正なときの扱い
  - 同梱バルーンが無いゴーストの出力が変わらないことの回帰テスト
  - `install.txt`（と同梱バルーンの `descript.txt`）の UTF-8 宣言の確認と、リポジトリのサンプル hello-pasta の `install.txt` への宣言の追加
  - スキル `pasta-check`（`SKILL.md`・`references/updates-txt-spec.md`・`references/nar-spec.md`）と `crates/pasta_check/README.md` の記述更新
- **Out of scope**:
  - 同梱シェル・プラグインなど、バルーン以外の同梱物（`install.txt` の他のキー）
  - `delete.txt` の生成・編集
  - nar の構造の変更（エントリのパス体系・既存の除外規則）
  - `updates.txt` の形式そのものの変更（Version 3・`charset,UTF-8` 行・CRLF・SOH 区切りは不変）
  - 既存の除外規則の拡張（例: SSP の生成機能が自動で外す `desktop.ini`・`thumbs.db`・`.DS_Store`、`developer_options.txt` の `noupdate`・`nonar` 指定の解釈）
  - ベースウェアのネットワーク更新の挙動
  - マニュアル（`book/`）への `pasta_check` の章の新設（現在マニュアルは `pasta_check` を扱っておらず、スキル `pasta-check` はマニュアルからの生成対象外の手書きファイルである）
  - crates.io へのリリース作業そのもの、ghost_dev 側の `release/emo2` の再生成、ghost_dev 側にあるスキル `pasta-check` の写しの同期
- **Adjacent expectations**:
  - 完了済み spec `pasta-check`・`audit-pasta-check` が定めた release の 5 段の流れ（準備 → target コピー → `--copy` 上書き → 更新ファイル生成 → nar 作成）、更新ファイルの形式、既存の除外規則、パストラバーサル防御、`--target` を変更しないことを前提とし、これらを壊さない。
  - 新しい `pasta_check` は、本仕様の完了後に `release-workflow` で crates.io へ公開され、emo2 開発セッション（ghost_dev）へバージョンが連絡される。

## Requirements

### Requirement 1: 同梱バルーンの判定

**Objective:** As a バルーンを同梱して配るゴーストの作者, I want `pasta_check` がベースウェアと同じ根拠（`install.txt`）で同梱バルーンのフォルダを判定してほしい, so that 配布物の振り分けとネットワーク更新の対象が食い違わない

#### Acceptance Criteria
1. When release の更新ファイル生成の段に入ったとき, the `pasta_check` shall 配布フォルダ直下の `install.txt` のバルーン指定に `source.directory` の行（例: `balloon.source.directory`）があれば、その値を同梱バルーンのフォルダとする
2. If バルーン指定に `source.directory` の行が無く `directory` の行（例: `balloon.directory`）がある, then the `pasta_check` shall `directory` の値を同梱バルーンのフォルダとする
3. If 配布フォルダ直下に `install.txt` が無い、または `install.txt` が 9 の確認を通り、バルーン指定が 1 つも無い, then the `pasta_check` shall 同梱バルーンは無いものとして、警告もエラーも出さずに従来どおりの更新ファイルを生成する
4. Where `install.txt` に番号付きのバルーン指定（`balloon0.source.directory`・`balloon0.directory`・`balloon1.*`…）がある, the `pasta_check` shall ベースウェアと同じ順序（番号なし → 0 → 1 → 2 …、見つからない番号が出た時点で打ち切る）でバルーン指定を探し、見つかった各指定を 1 つの同梱バルーンとして 1・2 と同じ規則で扱う（ukadoc「install.txt」の同時インストールの規則に合わせる。番号なしと番号付きは別の指定として併存しうる。ディスカッション #1 で決定）。打ち切った番号より後ろの番号の指定（例: `balloon0` と `balloon2` だけがあり、`balloon2` がベースウェアに読まれない）がある場合は、その指定のキーを示すエラーを表示してゼロ以外の終了コードで終了し、nar を作成しない（ukadoc「欠番を作ってはいけない」。読まれないバルーンのフォルダはゴースト用 `updates.txt` に載り、元の不具合が再発するため。作成ツールは問題があれば止める。設計ディスカッションで決定）
5. If 複数のバルーン指定が同じフォルダを指す, then the `pasta_check` shall そのフォルダを 1 つの同梱バルーンとして 1 回だけ扱う
6. The `pasta_check` shall 同梱バルーンの判定に、配布フォルダ直下の `install.txt` だけを使い、サブフォルダにある `install.txt`（例: 同梱バルーンのフォルダ内にあるバルーン自身の `install.txt`）は判定に使わない
7. The `pasta_check` shall `--copy` による上書きが済んだ後の配布フォルダの `install.txt` を判定に使う
8. The `pasta_check` shall `install.txt` のキー名を大文字小文字の違いを無視して照合し（例: `Charset`・`charset` を同じキーとみなす）、値の前後の空白を無視する（ukadoc はキーの大文字小文字に触れていないが、実在する `install.txt` に `Charset,UTF-8` の表記があり、ベースウェアで動作している。ディスカッションで決定）
9. When release の更新ファイル生成の段に入ったとき, the `pasta_check` shall 配布フォルダ直下に `install.txt` があれば、同梱バルーンの有無にかかわらず、その 1 行目（UTF-8 の BOM があればその直後）が `charset,UTF-8` の宣言（キーと値の大文字小文字、前後の空白は問わない）であることを確かめる（`pasta_check` は UTF-8 に限って動くツールとする。ukadoc では `charset` 行は 1 行目に書き、省略すると Shift_JIS とみなされる。ディスカッションで決定）
10. If `install.txt` の 1 行目が `charset,UTF-8` の宣言でない（宣言が無い・UTF-8 以外を指定している）、または `install.txt` に UTF-8 として正しくないバイト列がある, then the `pasta_check` shall `install.txt` が UTF-8 でない旨のエラーを表示してゼロ以外の終了コードで終了し、nar を作成しない
11. The `pasta_check` shall 文字コードの変換を行わず、UTF-8 以外の文字コードの `install.txt`・`descript.txt` を処理する機能を持たない

### Requirement 2: 不正な指定と存在しないフォルダの扱い

**Objective:** As a ゴーストの作者, I want `install.txt` の書き誤りや配置漏れを release の時点で知りたい, so that 壊れた配布物や意図しないファイル操作を避けられる

#### Acceptance Criteria
1. If 同梱バルーンのフォルダの値が空である、絶対パス（ドライブ名・ルート・UNC から始まるもの）である、`..` の要素を含む、または `source.directory` の行が無く代わりに使う `directory` の値がパス区切り（`/`・`\`）を含む, then the `pasta_check` shall 不正な値とその行のキーを示すエラーを表示してゼロ以外の終了コードで終了し、nar を作成しない（ukadoc「install.txt」: `..` による上位階層への参照はできない／`*.directory` は 1 階層のディレクトリ名だけでパス区切りは使えない。ディスカッションで決定）
2. Where バルーン指定の `source.directory` の値がパス区切り（`/`・`\` のどちらも可）を含む配布フォルダ内の相対パスである, the `pasta_check` shall それを配布フォルダからの階層付きの相対パスとして扱い、指すフォルダを同梱バルーンとする（ukadoc: SSP 2.9.00 以降の `*.source.directory` の仕様。ディスカッションで決定）
3. If 同梱バルーンのフォルダが `ghost/master` と同じ・その上位・その配下のいずれかである、または別の同梱バルーンのフォルダの上位・配下である, then the `pasta_check` shall 重なっているフォルダを示すエラーを表示してゼロ以外の終了コードで終了し、nar を作成しない（ゴースト本体や他のバルーンと入れ子の指定は配布物の誤り。作成ツールは問題があれば止める。ディスカッションで決定）
4. The `pasta_check` shall `install.txt` の値がどのようなものであっても、配布フォルダの外にあるファイルを読み取り・作成・変更しない
5. If 値は妥当だが、配布フォルダ内にそのフォルダが存在しない（同名のファイルがある場合を含む）, then the `pasta_check` shall 指定されたフォルダとその行のキーを示すエラーを表示してゼロ以外の終了コードで終了し、nar を作成しない（同梱を宣言したのに実体が無い nar はインストールに失敗する配布物であり、警告では見落とされるため。ディスカッションで決定）
6. When 値と配布フォルダ内のフォルダ名が大文字小文字だけ異なり、ファイルシステムがそれらを同じフォルダとして扱う, the `pasta_check` shall そのフォルダを同梱バルーンとして扱い、Requirement 3 の除外と Requirement 4 の生成を同じフォルダに対して一貫して行う
7. If バルーン指定のキー（`*.source.directory`・`*.directory`）が `install.txt` に複数行ある（キーの大文字小文字の違いだけのものを含む）, then the `pasta_check` shall 重複したキーを示すエラーを表示してゼロ以外の終了コードで終了し、nar を作成しない（ベースウェアがどの行を採るかは未確認で、食い違えば判定がずれるため。作成ツールは問題があれば止める。設計ディスカッションで決定）
8. If 同梱バルーンのフォルダの値の要素に `profile` または `var`（大文字小文字は問わない）がある, then the `pasta_check` shall 不正な値とその行のキーを示すエラーを表示してゼロ以外の終了コードで終了し、nar を作成しない（どちらも SSP の生成機能が配布物から外すフォルダで、`profile/` は nar にも入らず、インストールできない配布物になるため。設計ディスカッションで決定）

### Requirement 3: ゴースト用 updates.txt からの同梱バルーンの除外

**Objective:** As a 同梱バルーン付きゴーストの利用者, I want ゴーストのネットワーク更新でバルーンのファイルがゴーストのフォルダに置かれないでほしい, so that 更新のたびにゴーストのフォルダにゴミが増えない

#### Acceptance Criteria
1. While 同梱バルーンが判定されているとき, the `pasta_check` shall すべての同梱バルーンのフォルダ配下のファイルを、配布フォルダ直下の `updates.txt` に 1 行も載せない
2. While 同梱バルーンが判定されているとき, the `pasta_check` shall `ghost/master/updates.txt` を、配布フォルダ直下の `updates.txt` と同じ内容で出力する（同梱バルーン配下の行を含まない）
3. While 同梱バルーンが判定されているとき, the `pasta_check` shall 同梱バルーンのフォルダ以外のファイル（例: `install.txt`・`delete.txt`・`readme.txt`・`ghost/`・`shell/` 配下）を、従来どおりゴースト用 `updates.txt` に載せる
4. The `pasta_check` shall 名前が同梱バルーンのフォルダ名と一致するフォルダであっても、バルーン指定が指す位置以外にあるもの（例: `ghost/master/<同名>/`）をゴースト用 `updates.txt` から除外しない

### Requirement 4: バルーン用 updates.txt の生成

**Objective:** As a 同梱バルーン付きゴーストの利用者, I want 同梱バルーンがバルーンとしてネットワーク更新できるようにしてほしい, so that インストール済みのバルーン本体が新しい版に更新される

#### Acceptance Criteria
1. While 同梱バルーンが判定されているとき, the `pasta_check` shall 各同梱バルーンのフォルダ直下（そのフォルダの `descript.txt` と同じ階層）に、そのフォルダを基準とするバルーン用 `updates.txt` を 1 つずつ生成する
2. The `pasta_check` shall バルーン用 `updates.txt` の各行のパスを、同梱バルーンのフォルダからの相対パス（スラッシュ区切り。例: `file,arrow0.png…`）で書く
3. The `pasta_check` shall バルーン用 `updates.txt` を、ゴースト用 `updates.txt` と同じ形式（1 行目 `charset,UTF-8`、CRLF 改行、区切りのバイト値 1、32 文字小文字 16 進の md5、`size=`、`date=`、パスの辞書順）で書く
4. The `pasta_check` shall バルーン用 `updates.txt` に自分自身（同梱バルーンのフォルダ直下の `updates.txt`）を載せない
5. The `pasta_check` shall バルーン用 `updates.txt` の収集にも、ゴースト用と同じ除外規則（フォルダ `profile`・`var`、ファイル名 `updates2.dau`・`updates.txt`・`developer_options.txt`）を適用する（ukadoc の更新ファイルの配置と生成時の除外はゴーストとバルーンで共通。emo2 開発セッションの希望とも一致。ディスカッションで決定）
6. If 配布フォルダの同梱バルーンのフォルダ直下に `--target` または `--copy` 由来の `updates.txt` が既にある, then the `pasta_check` shall それを生成したバルーン用 `updates.txt` で置き換える
7. If 同梱バルーンのフォルダに除外規則を適用した後の対象ファイルが 0 件である, then the `pasta_check` shall そのバルーン用 `updates.txt` を生成しない（ゴースト用で 0 件のときと同じ扱い）
8. The `pasta_check` shall バルーン用 `updates.txt` を同梱バルーンのフォルダ直下以外（例: `ghost/master/`）へ書き出さない
9. When バルーン用 `updates.txt` を生成したとき, the `pasta_check` shall 更新ファイル生成の段の進捗表示に、同梱バルーンのフォルダと登録したエントリ数を表示する

### Requirement 5: 実ファイルとの一致（内容を変換しない）

**Objective:** As a ゴーストの利用者, I want `updates.txt` の md5 とサイズが配布されるファイルと必ず一致してほしい, so that ネットワーク更新が不一致で失敗したり無駄なダウンロードを繰り返したりしない

#### Acceptance Criteria
1. The `pasta_check` shall ゴースト用・バルーン用のどちらの `updates.txt` でも、md5 と size を配布フォルダ内の実ファイルのバイト列から計算する
2. The `pasta_check` shall 生成する `updates.txt` 以外の配布フォルダ内のファイル（同梱バルーンのファイルと `install.txt` を含む）の内容を、改行コードの変換を含めて一切変更しない
3. When nar を作成したとき, the `pasta_check` shall nar 内の各ファイルのバイト列を、`updates.txt` に記載した md5 と size に一致させる

### Requirement 6: nar への封入

**Objective:** As a 同梱バルーン付きゴーストの作者, I want 生成したバルーン用 `updates.txt` が配布物（nar）に入ってほしい, so that 初回インストール直後からバルーンのネットワーク更新が成り立つ

#### Acceptance Criteria
1. When バルーン用 `updates.txt` を生成したとき, the `pasta_check` shall nar に `<同梱バルーンのフォルダ>/updates.txt` のエントリとしてバルーン用 `updates.txt` を含める
2. The `pasta_check` shall 同梱バルーンの有無にかかわらず、nar のエントリのパス体系と既存の除外規則（`profile/` を入れない）を従来どおりに保つ

### Requirement 7: 同梱バルーンの homeurl 欠落の警告

**Objective:** As a ゴーストの作者, I want 同梱バルーンにネットワーク更新の前提が欠けているとき気づきたい, so that 更新できないバルーンを知らずに配らない

#### Acceptance Criteria
1. If 同梱バルーンのフォルダ直下の `descript.txt` に `homeurl` の行が無い、または値が空である, then the `pasta_check` shall 同梱バルーンのフォルダを示し、`homeurl` が無いためバルーンのネットワーク更新ができない旨の警告を表示する
2. If 同梱バルーンのフォルダ直下に `descript.txt` が無い, then the `pasta_check` shall 同梱バルーンのフォルダを示し、`descript.txt` が無い旨のエラーを表示してゼロ以外の終了コードで終了し、nar を作成しない（バルーンとして成り立たない配布物のため。作成ツールは問題があれば止める。ディスカッションで決定）
3. When 警告を表示したとき, the `pasta_check` shall 処理を止めずに、警告が無い場合と同じ生成物（ゴースト用・バルーン用 `updates.txt` と nar）を作成し、終了コード 0 で終了する
4. The `pasta_check` shall 警告を、進捗表示と区別できる形（警告であることが分かる接頭辞を付ける）で表示する
5. The `pasta_check` shall 同梱バルーンの `descript.txt` にも Requirement 1.9〜1.11 と同じ UTF-8 の規則を適用し（満たさなければ 1.10 と同じくエラーで止める）、`homeurl` を Requirement 1.8 と同じキー照合で読み取る

### Requirement 8: 同梱バルーンが無いゴーストの後方互換

**Objective:** As a 同梱バルーンを持たないゴーストの作者（例: pasta-in-windows・hello-pasta）, I want 新しい `pasta_check` でも出力が変わらないでほしい, so that 既存の配布物とネットワーク更新に影響が出ない

#### Acceptance Criteria
1. While 同梱バルーンが無いとき, the `pasta_check` shall ゴースト用 `updates.txt`（ルート・`ghost/master`）を、`date=` の値を除いて従来とバイト単位で同一に出力する
2. While 同梱バルーンが無いとき, the `pasta_check` shall nar に従来と同じファイルの集合を、同じエントリ名で含める
3. While 同梱バルーンが無いとき, the `pasta_check` shall 同梱バルーンに関する警告を表示しない
4. The `pasta_check` shall 同梱バルーンが無い配布フォルダについて 1 から 3 を検証する回帰テストを持つ
5. The `pasta_check` shall 同梱バルーンがある配布フォルダについて、ゴースト用 `updates.txt` に同梱バルーン配下の行が無いこと、バルーン用 `updates.txt` の各行の md5 と size が実ファイルと一致すること、nar にバルーン用 `updates.txt` が入ることを検証するテストを持つ
6. If 同梱バルーンが無いゴーストでも、配布フォルダ直下の `install.txt` が Requirement 1.9 の確認を通らない, then the `pasta_check` shall Requirement 1.10 のエラーで止める（1〜3 の後方互換の例外。ukadoc では宣言の無い `install.txt` は Shift_JIS であり、UTF-8 限定のツールとして扱わない）
7. The リポジトリのサンプルゴースト hello-pasta の `install.txt`（`crates/pasta_sample_ghost/ghosts/hello-pasta/install.txt`）shall 1 行目に `charset,UTF-8` を持ち、新しい `pasta_check` の release を通る（リポジトリに置かれた `release/hello-pasta` の写しも同じ内容にそろえる）
8. When release を開始したとき, the `pasta_check` shall `--nar` の位置に既にあるファイルを、配布フォルダの準備と同じ段で削除し、その後のどの段で失敗しても `--nar` の位置に nar が残らないようにする（失敗時に前回の nar が残ると、古い配布物を最新と誤って配りうるため。同梱バルーンの有無によらない既存の挙動の変更で、1〜3 の後方互換の例外。作成ツールは問題があれば止める。設計ディスカッション #1 で決定）

### Requirement 9: スキル・README の記述更新

**Objective:** As a `pasta_check` を使う作者と AI エージェント, I want 同梱バルーンの扱いが参照文書に正しく書かれていてほしい, so that 同梱バルーン付きゴーストを迷わず正しく配布できる

#### Acceptance Criteria
1. The スキル `pasta-check` の `references/updates-txt-spec.md` shall 同梱バルーンの判定方法（`install.txt` のキー・優先順位・番号付き指定・階層付きの値）、不正な値と重なりのエラー、ゴースト用 `updates.txt` からの除外、バルーン用 `updates.txt` の位置と相対パスの基準、適用する除外規則、`homeurl` 欠落の警告を記載する
2. The スキル `pasta-check` の `references/nar-spec.md` shall nar にバルーン用 `updates.txt` が入ることと、同梱バルーンがあるときの内部構造の例を記載する
3. The スキル `pasta-check` の `SKILL.md` shall 実行フローの更新ファイル生成の段の説明・ディレクトリ構成例・トラブルシューティング（`homeurl` 警告、`install.txt`・`descript.txt` が UTF-8 でないエラー、不正な値・指定フォルダ不在・`descript.txt` 不在・重なりのエラー）に同梱バルーンの扱いを反映する
4. The `crates/pasta_check/README.md` shall 仕様メモに同梱バルーンの扱い（判定・除外・バルーン用 `updates.txt`）と、`install.txt` に `charset,UTF-8` の宣言を必須とすること、release の開始時に `--nar` の位置の前回の nar を消すこと（Requirement 8.8）を記載する（crates.io の利用者が読む説明であり、どちらも利用者に見える変更のため。ディスカッションで決定）
5. When 上記の文書を更新するとき, the スキル `pasta-check` shall 同じ文書にある現行実装と食い違う既存の記述（例: 除外ファイル表に `updates2.dau` が無い、トラブルシューティングの「updates.txt が Shift_JIS でない」、nar 内部構造の例の `ghost/master/pasta_scripts/`）を現行実装に合わせて正す（触る文書の中の食い違いは同時に直す。特に Shift_JIS の記述は UTF-8 限定の方針と矛盾する。ディスカッションで決定）
