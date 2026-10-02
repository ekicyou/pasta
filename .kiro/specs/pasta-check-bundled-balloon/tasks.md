# Implementation Plan

> 各タスクの完了時に `cargo clippy --all-targets --workspace -- -D warnings` と `cargo test -p pasta_check` が通る状態を保つ（CI と同じ基準）。

- [ ] 1. 基盤: 後方互換の固定と UTF-8 宣言付きフィクスチャ
- [x] 1.1 同梱バルーンが無い配布フォルダのゴースト用 updates.txt を特性化テストで固定する
  - 固定フィクスチャには `ghost/master`・入れ子のフォルダと、既存の除外対象（`profile/`・`var/`・`updates2.dau`・`developer_options.txt`・既存の `updates.txt`）を含める
  - 同梱バルーンの無いそのフィクスチャから生成したルートと `ghost/master` の updates.txt の全バイトを、`date=` の値だけ伏せて期待値と比べる
  - 除外の仕組みを変える前に置き、以降の変更で後方互換が崩れたら落ちるようにする
  - 現行の実装のままでテストが通る
  - _Requirements: 8.1, 8.4_

- [x] 1.2 (P) 既存の release 統合テストと CLI の E2E テストの install.txt フィクスチャを UTF-8 宣言付きにする
  - 対象は release の統合テスト（target の install.txt を書く箇所と、その内容を照合する箇所）と CLI の E2E テストの install.txt フィクスチャ
  - フィクスチャの install.txt の 1 行目を `charset,UTF-8`（CRLF）にする。フィクスチャの内容に結び付いた期待値はそれに合わせて直し、何を検証するかは変えない
  - 現行の実装のままで、変更したテストがすべて通る（宣言の必須化を入れた後も通る形になっている）
  - _Requirements: 8.6_
  - _Boundary: pasta_check tests_

- [x] 1.3 (P) サンプルゴースト hello-pasta の install.txt に UTF-8 宣言を足し、サンプルのテストで固定する
  - hello-pasta の install.txt の 1 行目に `charset,UTF-8`（CRLF）を足す。他の行と改行コードは変えない
  - サンプルゴーストの ukadoc ファイル検証に「install.txt の 1 行目が `charset,UTF-8`」の確認を加える
  - cargo のビルド・テスト前に環境変数 `NoDefaultCurrentDirectoryInExePath` を外す（LuaJIT のビルドが落ちるため）
  - サンプルゴーストのテストが通る
  - _Requirements: 8.7_
  - _Boundary: sample ghost_

- [ ] 2. 同梱バルーンの判定（読み取り専用）
- [x] 2.1 UTF-8 の key,value ファイルの読み取りと charset 宣言の確認を実装する
  - 新しい判定モジュールを用意してクレートに登録する。release から呼ばれるまでの間 clippy の dead_code で落ちないよう、登録箇所にテスト以外での `dead_code` 許可を一時的に付ける（4.2 で外す）
  - 先頭の BOM を除き、UTF-8 として正しくないバイト列はエラーにする。文字コード変換は行わず、依存も足さない
  - 1 行目がキー・値とも大文字小文字無視・前後空白無視で `charset,UTF-8` でなければ、ファイル名を示す「UTF-8 でない」エラーにする
  - 2 行目以降をキーの大文字小文字無視・値の trim で読み、カンマの無い行は無視する
  - 単体テスト: `Charset,UTF-8`・BOM 付き・前後空白を受け入れ、宣言無し・`charset,Shift_JIS`・不正バイト（`0x82 0xA0`）をエラーにする
  - _Requirements: 1.8, 1.9, 1.10, 1.11_

- [x] 2.2 バルーン指定の探索を実装する
  - 番号なし → `balloon0` → `balloon1` … の順に探し、`source.directory` の行があれば（値が空でも）それを、無ければ `directory` を採る（階層付きの値を許すかを区別して持つ）
  - 番号なしの有無は番号付きの探索に影響させず、番号付きは両方の行が無い最初の番号で打ち切る
  - 打ち切った番号より後ろの番号の指定があれば、そのキーと欠番を示すエラーにする
  - バルーン指定のキーが複数行（大文字小文字違いを含む）あれば重複キーのエラーにし、それ以外のキーは最初の行を採る
  - 単体テスト: `source.directory` 優先・`directory` 代用・番号なし＋0＋1 の 3 件・番号なし無しで `balloon0`・`balloon0`＋`balloon2` の欠番エラー・キーの大文字小文字無視・`Balloon.Directory` を含む重複エラーがそれぞれ期待どおりになる
  - _Requirements: 1.1, 1.2, 1.4, 2.7_

- [x] 2.3 バルーン指定の値の正規化と検証を実装する
  - 設計の順（空 → `directory` 代用時の区切り → 先頭区切り → `:` → `..` → 正規化後の空 → `profile`・`var`）で検査し、最初に該当した理由とキー・値を示すエラーにする
  - `source.directory` の値は `/`・`\` どちらの区切りでも階層付きの相対パスとして要素列にする（空要素と `.` は捨てる）
  - 単体テスト: `extra\bal1`・`extra/bal1/` を受け入れ、空・`/abs`・`\\server\x`・`C:\x`・`C:x`・`a/../b`・`.`・`directory` 代用時の `a/b`・`profile/bal`・`x/Var/bal` が各理由のエラーになる
  - _Requirements: 2.1, 2.2, 2.8_

- [x] 2.4 実在のフォルダ名への解決・重複除去・重なり検出を実装する
  - 要素ごとに親フォルダを列挙して完全一致のフォルダ（シンボリックリンクでないもの）を採り、無ければファイルシステムが同じとみなす場合に限り大文字小文字違いの実在名を採る
  - 見つからない・同名のファイル・シンボリックリンクはキーとフォルダを示す不在エラーにする。結合には検証済みの要素と列挙で得た実在名だけを使い、配布フォルダの外を指さない
  - 解決後の文字列が同じ指定は最初の 1 件だけ残す
  - `ghost/master` と同じ・上位・配下（比較は大文字小文字無視）、および同梱バルーンどうしの入れ子を、両方のフォルダを示す重なりエラーにする
  - 単体テスト: 不在・同名ファイル → 不在エラー、`ghost`・`ghost/master`・`ghost/master/x`・バルーン同士の入れ子 → 重なりエラー、同じフォルダを指す 2 件 → 1 件。Windows 限定テストで `EMO2-KAKUKAKU` が実在名 `emo2-kakukaku` に解決される
  - _Requirements: 1.5, 2.3, 2.4, 2.5, 2.6_

- [x] 2.5 同梱バルーンの descript.txt の確認と判定全体の入口を実装する
  - 配布フォルダ直下の install.txt だけを読み、無ければ同梱バルーン無し・警告無しを返す。宣言の確認は指定の有無より先に行う
  - 各同梱バルーンの descript.txt が無ければフォルダを示すエラー、UTF-8 規則違反はそのファイルを示すエラーにする
  - `homeurl` が無い・空のときだけ、フォルダを示す警告を 1 件加える（接頭辞は付けない）
  - 判定はファイルを一切書かない
  - 単体テスト: install.txt 無し・指定無し → 空で警告なし、descript.txt 不在・Shift_JIS 宣言 → エラー、`homeurl` 無し・空 → 警告 1 件、`HomeURL,https://…` → 警告なし、サブフォルダの install.txt は判定に使われない、判定の前後で配布フォルダの内容が変わらない
  - _Requirements: 1.3, 1.6, 5.2, 7.1, 7.2, 7.5, 8.3, 8.6_

- [ ] 3. 更新ファイルの生成
- [ ] 3.1 (P) ゴースト用 updates.txt の収集で、指定した相対フォルダを丸ごと外せるようにする
  - 既存の名前一致の除外に加え、基準フォルダからの相対パス（`/` 区切り）が完全一致したフォルダには入らない
  - 外すフォルダが無いときは従来と同じ処理経路を通る
  - 単体テスト: `bal` を外すとルートの一覧に `bal/` の行が無く、`ghost/master/bal/` の同名フォルダは残り、それ以外のファイルは従来どおり載る。1.1 の特性化テストが通ったまま
  - 判定モジュール（2.x）とは別ファイルで独立しており、2.x と並行して進められる
  - _Requirements: 2.6, 3.1, 3.3, 3.4, 8.1_
  - _Boundary: UpdateFilesGenerator_
  - _Depends: 1.1_

- [ ] 3.2 バルーン用 updates.txt の書き出しと、生成件数の要約を返す形への変更を実装する
  - 受け取った同梱バルーンのフォルダをゴースト用の収集から外し、ルートへ書いて `ghost/master` へ複製する（既存どおり）
  - 各同梱バルーンのフォルダを基準に同じ除外規則で収集し、0 件なら書かず、そうでなければ同じ形式でフォルダ直下に書く（既存のファイルは置き換え、`ghost/master` への複製はしない）
  - ゴースト用の件数と、生成したバルーンごとのフォルダ・件数を返す
  - release 側の呼び出しを新しい形に合わせ（同梱バルーンは当面空を渡す）、生成したバルーンごとに `Generated <フォルダ>/updates.txt (N entries)` を stdout に出す表示もここで加える（判定がつながる 4.2 までは空なので出力は変わらない）
  - 単体テスト: `bal/updates.txt` の各行がバルーン基準の相対パスで、自分自身・`profile/`・`var/`・`updates2.dau`・`developer_options.txt` を含まず、md5・size が実ファイルと一致する。既存の `bal/updates.txt` が置き換わり、0 件のバルーンには書かれず、`bal/ghost/master/` があっても複製されない。ルートと `ghost/master` の updates.txt が同内容
  - _Requirements: 3.2, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9, 5.1, 5.2_
  - _Boundary: UpdateFilesGenerator, ReleaseOrchestrator (call site only)_

- [ ] 4. release への組み込み
- [ ] 4.1 release の開始時に `--nar` の位置の前回の nar を消す
  - 配布フォルダの準備の直後に `--nar` のファイルを消し、存在しないときは成功とみなし、それ以外の削除エラーはそのまま止める
  - 消したときは消したパスを進捗表示に出す
  - 統合テスト: `--nar` の位置にダミーの nar を置き、後段を今ある手段（存在しない `--copy` フォルダ）で失敗させると、そのファイルが無くなっている。成功時は新しい nar に置き換わる（判定エラーでの同じ確認は 4.3 で足す）
  - _Requirements: 8.8_

- [ ] 4.2 段 4 で判定 → 生成 → 進捗表示 → 警告表示を行う
  - `--copy` の上書き後に判定を呼び、その結果の同梱バルーンのフォルダで更新ファイルを生成する。判定エラーは何も書かずにそのまま返し、nar を作らない
  - 段 4 の進捗表示の後に、警告を `Warning: ` 接頭辞付きで stderr に出して段 5 へ進む
  - 2.1 で付けた一時的な `dead_code` 許可を外す
  - 統合テスト: `homeurl` の無い同梱バルーン付きの配布フォルダで release が成功し、nar と `bal/updates.txt` が作られる
  - _Depends: 2.5, 3.2_
  - _Requirements: 7.3, 7.4_

- [ ] 4.3 同梱バルーン付き・判定エラー時・同梱バルーン無しの release 統合テストを追加する
  - 同梱バルーン付き: nar に `bal/updates.txt` があり、nar 内の各ファイルのバイト列の md5・size がバルーン用 updates.txt の記載と一致し、ルートと `ghost/master` の updates.txt に `bal/` の行が無く、nar のエントリのパス体系と `profile/` 除外が保たれる
  - 判定エラー時（宣言無しの install.txt・指定フォルダ不在）: nar が作られず、配布フォルダのルートに updates.txt が新しく書かれない
  - `--copy` の後の install.txt で判定する: target の install.txt には指定が無く `--copy` の上書きで指定が加わると `bal/updates.txt` が生成され、上書きで加わった指定のフォルダが無ければエラーになる
  - 判定エラー時に、`--nar` の位置に置いたダミーの nar が残らない
  - 同梱バルーン無し: nar のエントリ集合とエントリ名が従来と同じ
  - 追加したテストがすべて通る
  - _Requirements: 1.7, 1.10, 2.5, 3.2, 5.3, 6.1, 6.2, 8.2, 8.5, 8.8_

- [ ] 5. 検証: CLI とサンプルの写し
- [ ] 5.1 CLI の E2E テストで警告・エラー・後方互換の出力を確認する
  - `homeurl` 無しの同梱バルーン: exit 0、stderr に `Warning:` とフォルダ名、stdout に `Generated bal/updates.txt (N entries)`
  - 宣言の無い install.txt: exit 1、stderr に `Error:` と `UTF-8`、nar が無い
  - 同梱バルーン無し: stderr に `Warning:` が出ない
  - 追加した E2E テストがすべて通る
  - _Requirements: 1.10, 4.9, 7.3, 7.4, 8.3, 8.6_

- [ ] 5.2 リポジトリの release/hello-pasta の写しを新しい pasta_check で再生成する
  - 先に hello-pasta のフォルダに追跡外・無視対象のファイル（`profile/` など）が無いことを git で確認する（あれば写しに混ざるため、止めて報告する）
  - pwsh からリリーススクリプトを `-SkipSetup` で直接呼ぶ（`-ExecutionPolicy Bypass` は使わない）か、同じ引数で `cargo run -p pasta_check -- release` を実行し、写しの install.txt・updates.txt・`ghost/master/updates.txt` と nar を再生成する（手で直さない）
  - 写しの install.txt の 1 行目が `charset,UTF-8` で、updates.txt の install.txt 行の md5・size が実ファイルと一致する
  - 差分が install.txt の行・`date=`・nar に限られることを git の差分で確認する。それ以外（改行コードの違いによる md5 の変化など）が出たら手で直さず、止めて報告する
  - _Requirements: 8.7_

- [ ] 6. 参照文書の更新
- [ ] 6.1 (P) スキルの updates.txt 仕様と nar 仕様に同梱バルーンの扱いを書く
  - updates.txt 仕様: 判定方法（キー・優先順位・番号付き指定と欠番・階層付きの値）、不正な値・重複キー・重なりのエラー、ゴースト用からの除外、バルーン用の位置と相対パスの基準、適用する除外規則、`homeurl` 警告を記載する
  - nar 仕様: バルーン用 updates.txt が入ることと、同梱バルーンがあるときの内部構造の例を記載する
  - 同じ文書の現行実装との食い違い（除外ファイル表の `updates2.dau`、nar 構造例の `ghost/master/pasta_scripts/` など）を正す
  - 記載内容が実装のエラー文言・挙動と一致している
  - _Requirements: 9.1, 9.2, 9.5_
  - _Boundary: skill docs (references)_
  - _Depends: 4.3_

- [ ] 6.2 (P) スキルの SKILL.md に同梱バルーンの扱いを反映する
  - 実行フローの更新ファイル生成の段の説明とディレクトリ構成例に同梱バルーンを加える
  - トラブルシューティングに `homeurl` 警告、install.txt・descript.txt が UTF-8 でないエラー、不正な値・指定フォルダ不在・descript.txt 不在・重なりのエラーを加え、「updates.txt が Shift_JIS でない」の記述を UTF-8 限定の方針に合わせて正す
  - 記載内容が実装のエラー文言・挙動と一致している
  - _Requirements: 9.3, 9.5_
  - _Boundary: skill docs (SKILL.md)_
  - _Depends: 4.3_

- [ ] 6.3 (P) pasta_check の README と steering の構成一覧を更新する
  - README の仕様メモに同梱バルーンの判定・除外・バルーン用 updates.txt、install.txt の `charset,UTF-8` 宣言の必須化、release 開始時の前回の nar の削除を書き、ソース構成に判定モジュールを足す
  - steering の structure.md の pasta_check のソース一覧に判定モジュールを 1 行足す
  - 記載内容が実装の挙動と一致している
  - _Requirements: 9.4_
  - _Boundary: README, steering_
  - _Depends: 4.3_

## Implementation Notes

- 1.3: リポジトリに .gitattributes は無く `core.autocrlf=true`。テキストは index が LF・作業ツリーが CRLF（`git ls-files --eol` で `i/lf w/crlf`）。5.2 で写しを再生成するとき、md5 は作業ツリーの CRLF のバイト列で計算される点に注意する。
- 2.3: Windows では `bal.`・`bal `・`...`・`.. ` も `is_dir()` が真になる（`...` は親フォルダ自身を指す）。2.4 の解決は「`is_dir()` が真」だけで採らず、必ず `read_dir` の実在名との一致（完全一致または小文字一致）で採り、結合には実在名だけを使う。
- 2.4: 重複除去は `dedup_dirs`（設計の判定規則 5）として独立させた。2.5 では解決 → `dedup_dirs` → `check_overlaps` の順に呼ぶ（逆だと同じフォルダが重なりと誤判定される）。大文字小文字違いの照合は Unicode の `to_lowercase()`（`is_dir()` が真の場合に限る）。
