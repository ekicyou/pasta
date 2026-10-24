# Requirements Document

## Project Description (Input)
`pasta_check release` を使うゴーストの作者が困っている（brief.md を参照）。`--target`・`--release`・`--copy`・`--nar` の関係を確かめていないため、組み合わせによって作者のゴーストのフォルダを丸ごと消す、自分自身を無限に入れ子に写す、書きかけの .nar をその .nar の中に入れたまま「成功」と報告する。2026-10-19 の `/ponytail:ponytail-audit` は、ほかに、読み取りの誤りの握りつぶし・決して働かない守り・写し・作られない名前の DLL を探すテストを指摘した。危ない組み合わせはファイルに触れる前に英語の誤りで止まり、読み取りの誤りは起きた段で報告され、働かない守りと写しが無くなり、本当に起きる場合を守るテストがある状態にする。

## Introduction

本仕様は、`pasta_check release` の引数の組み合わせで作者のフォルダが壊れる不具合を直し、同じ領域（`pasta_check` の `release`・`copy`・`nar`・`update_files`・`main` と、見本のゴースト `pasta_sample_ghost` の試験）の ponytail 監査の指摘を残らず片づける。

今は、次の呼び出しがどれも作者のフォルダを壊す（2026-10-24 にスクラッチの一時フォルダで再現した。`research.md` を参照）。

```text
pasta_check release --target C:\dev\myghost --release C:\dev     ... C:\dev ごと消える（myghost も）
pasta_check release --target . --release .                        ... 開発フォルダの中身が消える
pasta_check release --target G --release G\out                    ... G\out\out\out… を写し続け、スタックあふれで落ちる
pasta_check release --target G --release out --nar out\g.nar      ... 書きかけの g.nar を g.nar に入れて「Release complete!」
```

本仕様の後は、これらはどれも、ファイルに触れる前に次のような英語の誤りで止まる（文言は設計で決める）。

```text
Error: --release (C:\dev) must not be the same as or contain --target (C:\dev\myghost)
```

`release.ps1` の段 4 の形（`--target ghosts\hello-pasta --release <ルート>\release\hello-pasta --nar <ルート>\release\hello-pasta.nar`）のような正しい使い方は、今までどおり 6 段を進んで nar を作る。

用語:
- **開発フォルダ**: `--target` で渡すフォルダ。作者が書いているゴーストの一式。
- **上書きフォルダ**: `--copy` で渡すフォルダ（0 個以上）。段 3 で配布フォルダに重ねて写す。
- **配布フォルダ**: `--release` で渡すフォルダ。段 1 で消して作り直し、段 2・3 で写し、段 5 で更新ファイルを書き、段 6 で nar にまとめる。
- **nar**: `--nar` で渡すファイル。段 1 で前回のものを消し、段 6 で作る。
- **入力のフォルダ**: 開発フォルダと上書きフォルダ。
- **「A が B の中にある」**: A が B の下の階層にある（A と B が同じ場合は含まない）。
- **「A が B と同じか B を中に含む」**: A と B が同じフォルダか、B が A の中にある。
- **同じ場所**: パスの書き方（相対・絶対、`.`・`..`、末尾の区切り、大文字と小文字、8.3 の短い名前、junction）が違っても、ファイルシステムの上で同じものを指すこと。
- **引数の確かめ**: 段 1 の前に、上の関係を調べる手順。
- **監査の指摘**: `research.md` の「監査のやり直し」に並べた、この領域の指摘（brief の 1〜8 と、やり直しで増えたもの）。

## Boundary Context

- **In scope**:
  - `pasta_check release` の引数の確かめ（危ない組み合わせを段 1 の前に断る）。
  - `pasta_check release` の段 5（更新ファイルの生成）の読み取りの誤りの報告。
  - `pasta_check` の `release`・`copy`・`nar`・`update_files`・`main` の、働かない守り・写し・要らない作りの整理（外から見える出力は変えない）。
  - `pasta_sample_ghost` の試験（`tests/` の下と `src/scripts.rs` の試験）の、決して落ちない試験・写しの整理。
  - 本当に起きる場合を守る試験（壊れる組み合わせ・`release.ps1` の段 4 の形・読み取りの誤り）。
  - 引数の関係と、断る組み合わせの説明（`crates/pasta_check/README.md`、スキル `pasta-check` の `SKILL.md` と `references/nar-spec.md`、マニュアル `book/src/debug/validate.md` の「release との関係」）。
- **Out of scope**:
  - `pasta_check` の `validate/`（辞書の検査の規則。`syntax.rs`・`report.rs` は `ponytail-editor-tooling`、`scenes.rs` は `ponytail-core-registry` が持つ）。
  - 同梱バルーンの判定（`balloon.rs`）と、`validate` サブコマンドの入口（`validate_cmd.rs`）の振る舞い。段 4 の検査の出力が日本語であることも変えない。
  - 配布フォルダが、前回の配布フォルダでない中身の入ったフォルダかどうかの判定（「毎回クリーンされる」は今の約束のまま）。
  - リリース CI（`ponytail-manual-ci`）と `release.ps1`・`release.bat` の中身。
  - 新しいオプション（確かめを省く・消さずに写す など）。
- **Adjacent expectations**:
  - `release.ps1` の段 4 は、段 3 で作った x64 の `pasta_check.exe` で `pasta_check release` を今の引数の形のまま呼ぶ。本仕様はその形を断らず、試験で守る。
  - リリース CI の版の検査は `pasta_check --version` の出力（`pasta_check X.Y.Z`）と完全一致で比べる。本仕様はその出力を変えない。
  - VSCode 拡張は `validate` の核だけを使う。本仕様の変更は拡張に届かない。
  - 後の `mlua-0-12-upgrade` が `self_deploy_integration_test.rs` を触る。本仕様が先に入る。
  - 同じウェーブの他の spec とソースのファイルを共有しない（約束: `pasta_check` の `validate/` に触らない）。

## Requirements

### Requirement 1: 危ない組み合わせを、何にも触れる前に断る

**Objective:** ゴーストの作者として、引数を取り違えても自分の開発フォルダや上書きフォルダが消えたり、ごみの木ができたりしないようにしたい。そうすれば、何か月もの作業を一度の打ち間違いで失わずに済む。

#### Acceptance Criteria

1. If 配布フォルダが、開発フォルダと同じか開発フォルダを中に含む, the pasta_check release shall 引数の確かめで失敗する。
2. If 配布フォルダが、どれかの上書きフォルダと同じかその上書きフォルダを中に含む, the pasta_check release shall 引数の確かめで失敗する。
3. If 配布フォルダが、開発フォルダの中にある, the pasta_check release shall 引数の確かめで失敗する。
4. If 配布フォルダが、どれかの上書きフォルダの中にある, the pasta_check release shall 引数の確かめで失敗する。
5. If nar が、配布フォルダと同じ場所か配布フォルダの中にある, the pasta_check release shall 引数の確かめで失敗する。
6. If nar が、開発フォルダかどれかの上書きフォルダと同じ場所か、その中にある, the pasta_check release shall 引数の確かめで失敗する（brief の 3 つの関係に加える。2026-10-24 開発者の決定。再現で、`--nar` が開発フォルダの中の作者のファイルを指すと、段 1 でそのファイルを消し、段 6 で nar に置き換えて成功と報告した）。
7. If 開発フォルダかどれかの上書きフォルダが無い（またはフォルダでない）, the pasta_check release shall 引数の確かめで失敗する（2026-10-24 開発者の決定。今は段 1 で配布フォルダと前回の nar を消してから段 2・3 で失敗する）。
8. When 引数の確かめで失敗したとき, the pasta_check release shall どのファイルもフォルダも消さず、作らず、書き換えない（配布フォルダの中身と前回の nar もそのまま残す）。
9. When 引数の確かめで失敗したとき, the pasta_check release shall 段の表示（`[1/6]` 以降）を出さずに終わる。

### Requirement 2: パスの書き方に左右されない判定

**Objective:** ゴーストの作者として、同じフォルダを別の書き方で渡しても確かめをすり抜けないようにしたい。そうすれば、相対パスや短い名前を使うバッチファイルでも守られる。

#### Acceptance Criteria

1. The pasta_check release shall Requirement 1 の関係を、パスを同じ場所として比べて判定する（相対・絶対、`.`・`..`、末尾の区切り、大文字と小文字の違い、8.3 の短い名前、junction の違いに左右されない）。
2. While 配布フォルダか nar がまだ無い, the pasta_check release shall 在る場合と同じように Requirement 1 の関係を判定する（まだ無いパスでも、在る親フォルダの書き方の違いに左右されない）。
3. The pasta_check release shall 引数の確かめのために、ファイル・フォルダを作らない。
4. The pasta_check release shall 段の表示・成功の表示・誤りの表示に出すパスを、作者が渡した書き方のまま出す（判定のために変換した形を出さない）。

### Requirement 3: 断る理由の表示

**Objective:** ゴーストの作者として、断られたときに、どの 2 つの引数のどんな関係が悪いのかを一目で知りたい。そうすれば、直す引数がすぐ分かる。

#### Acceptance Criteria

1. When 引数の確かめで失敗したとき, the pasta_check release shall `Error: ` で始まる英語の理由を 1 行、標準エラー出力に出し、終了コード 1 で終わる。
2. When Requirement 1 の 1〜6 のどれかで失敗したとき, the pasta_check release shall 理由に、関係する 2 つのオプションの名前（`--release`・`--target`・`--copy`・`--nar` のうち 2 つ）と、それぞれに渡されたパスと、関係（同じ・含む・中にある）を示す。
3. When Requirement 1 の 7 で失敗したとき, the pasta_check release shall 理由に、オプションの名前と渡されたパスと、無い・フォルダでないのどちらかを示す。
4. If 引数が複数の関係に当たる, the pasta_check release shall 決まった順に調べて最初に見つけた 1 つを理由に出す（同じ引数なら毎回同じ理由になる。順は設計で決める）。

### Requirement 4: 今の正しい使い方を変えない

**Objective:** ゴーストの作者と pasta の開発者として、今動いている `release.bat`・`release.ps1` とリリース CI が、この直しの後もそのまま動いてほしい。そうすれば、直しのために手元の手順を書き換えずに済む。

#### Acceptance Criteria

1. When 引数が `release.ps1` の段 4 の形（配布フォルダと nar が同じ親フォルダに並び、開発フォルダがその外にある）のとき, the pasta_check release shall 今と同じ 6 段を進み、同じ中身の配布フォルダと nar を作る。
2. When 引数が Requirement 1 のどの関係にも当たらないとき, the pasta_check release shall 段の表示・更新ファイル・nar のエントリと中身を、今と同じに保つ。
3. The pasta_check shall `--version` の出力を今の形（`pasta_check <版>`）のまま保つ。
4. The pasta_check shall `--help` で出す使い方の行（オプションの一覧）を、今の 4 つのオプションのまま保つ。
5. When 引数の確かめを通ったとき, the pasta_check release shall 段 1 で配布フォルダを作り直し、前回の nar を消し、段 6 で失敗したときは作りかけの nar を消す、今の約束を保つ。

### Requirement 5: 読み取りの誤りを、起きた段で報告する

**Objective:** ゴーストの作者として、配布フォルダの中の一部が読めないときに、その分が黙って `updates.txt` から抜けた nar を配らないようにしたい。そうすれば、ネットワーク更新で一部のファイルが届かない配布物を出さずに済む。

#### Acceptance Criteria

1. If 段 5 で、配布フォルダか同梱バルーンのフォルダの中のフォルダの一覧を読めない, the pasta_check release shall 段 5 で失敗し、`updates.txt` と nar を作らない。
2. If 段 5 で、フォルダの一覧の中の 1 項目（名前・種類・大きさ・更新日時・中身）を読めない, the pasta_check release shall 段 5 で失敗し、`updates.txt` と nar を作らない。
3. When 段 2・3・5・6 でファイルの読み書きに失敗したとき, the pasta_check release shall `Error: ` で始まる英語の理由に、失敗したファイルかフォルダのパスを含める（2026-10-24 開発者の決定。今は OS の文言だけ（例: `Error: 指定されたパスが見つかりません。 (os error 3)`）で、どのパスか分からない）。
4. When 段 5 で失敗したとき, the pasta_check release shall 今の約束どおり、段 1 で消した前回の nar を残さない。

### Requirement 6: 監査の指摘の解消

**Objective:** pasta の開発者として、この領域の監査の指摘が残らず片づいていてほしい。そうすれば、決して働かない守りや写しに惑わされずに、本当の守りだけを読んで直せる。

#### Acceptance Criteria

1. The pasta_check shall ファイルシステムが返さない値（ディレクトリの一覧の名前の `..`・区切り文字）を調べる守りを持たない（監査の指摘 6。本当の守りは Requirement 1・2 が置き換える。audit-pasta-check の Req 1.3・1.4 を、開発者が 2026-10-19 に了承した形で置き換える）。
2. The pasta_check shall Unicode として正しくない名前の誤りを作る処理を 1 か所に持ち、nar と更新ファイルの両方で同じ英語の文言（`file name is not valid Unicode: <パス>`）を出す（監査の指摘 7）。
3. The pasta_check shall research.md の「監査のやり直し」の N5（要らない確かめ）・N6（要らない作り）・N7（古い注記と試験の名前）・N8（一つだけの欄の包み）を持たない（N6・N8 の小さな整理も範囲に入れる。2026-10-24 開発者の決定）。
4. The pasta_sample_ghost の試験 shall 作られない名前の DLL を探す道具と、決して落ちない試験を持たない（監査の指摘 4）。
5. The pasta_sample_ghost の試験 shall ゴーストの `master/` を一時フォルダへ写す処理と、試験の前に `PASTA_DEBUG`・`PASTA_DEBUG_PORT` を外す処理を、それぞれ 1 か所に持つ（監査の指摘 8）。
6. The pasta_sample_ghost の試験 shall 別の試験が同じことを確かめている試験を持たない（監査の指摘 8 の `dist_src_validation_test.rs`）。
7. The pasta_sample_ghost の試験 shall 写しを 1 か所にまとめた後も、今と同じものを確かめる（`profile/` と `pasta.dll` を写さず、コミット済みのゴーストをその場で読み込まず、写すときの読み取りの誤りを握りつぶさない）。
8. When 直しを入れた後, the pasta_check と pasta_sample_ghost shall `cargo fmt --check`、clippy の警告 0、`cargo test -p pasta_check -p pasta_sample_ghost` を通る。
9. The pasta_sample_ghost の `src/scripts.rs` の試験 shall 別の試験が確かめている内容だけの試験（N11）を持たない（roadmap の持ち場は `tests/**` だが、brief の領域はクレート全体なので範囲に入れる。2026-10-24 開発者の決定）。

### Requirement 7: 本当に起きる場合を守る試験

**Objective:** pasta の開発者として、壊れる組み合わせと正しい使い方の両方を試験で固めたい。そうすれば、後の変更で作者のフォルダを消す不具合が戻っても、手元の試験で気づける。

#### Acceptance Criteria

1. The pasta_check の試験 shall Requirement 1 の 1〜7 の各関係を、一時フォルダの中で実際の引数として渡し、失敗することと、開発フォルダ・上書きフォルダ・配布フォルダの中身・前回の nar が残ることを確かめる。
2. The pasta_check の試験 shall 少なくとも、`--release` を開発フォルダの親にする場合・開発フォルダと同じにする場合（`.` 同士を含む）・開発フォルダの中にする場合・`--nar` を配布フォルダの中にする場合を含む。
3. The pasta_check の試験 shall 同じ場所を別の書き方（相対と絶対、`..` を含む形、大文字と小文字の違い）で渡しても断ることを確かめる。
4. The pasta_check の試験 shall `release.ps1` の段 4 の形の引数で、最後まで進んで nar を作ることを確かめる。
5. The pasta_check の試験 shall 段 5 の読み取りの誤りが段 5 の失敗になり、`updates.txt` と nar が残らないことを確かめる。
6. The pasta_check と pasta_sample_ghost の試験 shall 一時フォルダのパスに 8.3 の短い名前が混じる環境（GitHub のランナーの `%TEMP%`）でも通る。
7. The pasta_check と pasta_sample_ghost の試験 shall 一時フォルダの外のファイル・フォルダを消さない・書き換えない。

### Requirement 8: 説明の文書を合わせる

**Objective:** ゴーストの作者として、どの組み合わせが断られるかと、断られたときに何が残るかを、README・スキル・マニュアルで同じように知りたい。そうすれば、どれを読んでも同じ答えになる。

#### Acceptance Criteria

1. The `crates/pasta_check/README.md` の release の節 shall 断る組み合わせ（Requirement 1）と、引数の確かめで止まったときは何も消さないこと（前回の nar も残ること）を書く。
2. The `crates/pasta_check/README.md` の仕様メモ shall 外した守り（ZIP のエントリ名の `..` の検査）の記述を残さない。
3. The スキル `pasta-check` の `SKILL.md` shall 選択肢の表と段の説明に断る組み合わせを書き、トラブルシューティングの表に新しい誤りの行を足す（編集は分類器が止めるので、実装のその場面で、入った分と残りを示して開発者の許可を得る）。
4. The スキル `pasta-check` の `references/nar-spec.md` shall 段 1 の前回の nar の削除の説明を、引数の確かめで止まったときの扱い（入力のフォルダが無いときも、前回の nar を残す）と合わせる（許可の扱いは 3 と同じ）。
5. The マニュアル `book/src/debug/validate.md` の「release との関係」 shall 1 段目の前に引数を確かめ、危ない組み合わせでは何にも触れずに止まることを書く。
6. When 振る舞いを変える直しを入れるとき, the 開発者 shall 同じ変更で上の文書を直す。
7. When 文書を直した後, the 開発者 shall `gen-skill-refs` の `--check` を走らせ、生成ファイルにずれが無いことを確かめる。
