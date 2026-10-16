# Implementation Plan

> 実装の順番は design.md の Migration Strategy に従う: 核（1〜2）→ 一致テスト（3）→ `validate`（4）→ `release` の関門（5）→ WASM の入口と CI（6）→ VSCode 拡張（7）→ 文書（8）→ 最終確認（9）。核の公開 API（1.3）が決まった後は、CLI の側（3〜5）とエディタの側（6〜7）を並行できる。

- [x] 1. 土台: pasta_check をライブラリとバイナリに分け、検査の核の骨組みを作る
- [x] 1.1 pasta_check をライブラリとバイナリの 2 つにし、依存を feature で分ける
  - バイナリだけが使う依存（引数の解析・ハッシュ・zip）を既定で有効な feature `cli` の後ろに移し、バイナリはその feature を要るようにする
  - ライブラリの入口を足し、構文・名前の規則の部品・TOML・パターン照合を依存に足す。wasm32-unknown のときだけ乱数源の `wasm_js` を有効にする（`pasta_core` は触らない）
  - テストのためだけに `pasta_lua` を版の無い path の dev-dependency に足す
  - ルートの workspace の依存の行で `pasta_check` の既定の feature を切り、`pasta_lsp` が `cli` を引き込まないようにする
  - 完了の状態: `cargo build -p pasta_check` と、ライブラリだけを feature 無しで wasm32-unknown-unknown に向けた `cargo check` が通り、通常の依存の木（`cargo tree -p pasta_check -e normal`）に `mlua`・`pasta_lua` が現れない
  - _Requirements: 1.7, 9.1, 11.1_

- [x] 1.2 報告の型・分類・文言・並び順・出力の形を 1 か所に作る
  - 報告の種類 23 個と、種類ごとに固定の誤り／注意の対応、診断用の kebab-case の code を作る
  - 1 件の問題（code・分類・相対パス・行・列・理由）、まとめの数（辞書ファイル数・誤り・注意・飛び先を検査しなかった Call・飛び先の省略）を作る
  - 1 行の形（`ghost/master/{path}:{line}:{col}: 誤り|注意: {理由}`、列・行が無いときは省く）、パス→行（無いものが先）→列→code→理由の並び、飛び先を省いたときの案内の行、まとめの行（省いたときは最後の項を出さない）を作る
  - design.md の文言の表どおりの日本語の理由を作る
  - 報告の型を `pasta_check::validate` から公開し、`tests/` の結合テストから使えるようにする
  - 完了の状態: 報告のテストで、1 行の形（列あり・なし・行なし）、並び順、まとめの行の 2 通り、全種類の分類の表を固定するテストが通る
  - _Requirements: 2.9, 6.7, 6.8, 7.1, 7.2, 7.3, 7.4, 7.7, 7.8_

- [x] 1.3 検査の核の公開 API と検査の順番の骨組みを作る
  - 入力の型（`pasta.toml` の状態 3 通り・ファイルの一覧・ファイルの中身 3 通り）を、JS からも読める camelCase とタグ `kind` の形で作る
  - 辞書ファイルを選び出す関数と、ゴースト全体を検査する関数を公開し、design.md の順番（設定 → 集め方 → モジュール名・構文 → トランスパイル・シーン → 飛び先）で各段を呼ぶ骨組みを置く。各段は後のタスクで中身を入れる
  - 検査の関数に渡されなかった辞書ファイルは「読み込まれていません」の読めないファイルとして扱う。乱数源を使わず、シーン表には空のモック選択子を渡す
  - 完了の状態: `pasta.toml` と正しい `.pasta` 1 つを持つゴーストを渡すと panic せずに報告が返り、同じ入力を 2 回渡して同じ報告になるテストが通る（後のタスクで規則が増えてもこの見本は誤りにならない）。ライブラリはファイルシステムに触れない
  - _Requirements: 1.1, 1.7, 11.1_

- [x] 2. 検査の核: 規則を 1 つずつ入れる
- [x] 2.1 pasta.toml の [loader] と [scene] を pasta.dll と同じ厳しさで読む
  - TOML として読めない（行・列）、`[loader]` の 4 つのキーの型の誤り（行が取れれば行）、`[scene]`・`[scene.alias]` の型の誤りと別名表の拒否（空の名前・重複・連鎖、該当する別名の行を全部）を誤りにする
  - `[loader]`・`[scene]` 以外のセクションは見ない。誤りがあっても既定値（`dic/**/*.pasta`・既定の別名表）で続ける
  - `pasta.toml` が無いときは既定値で集めたうえで `config-missing` を誤りにする
  - 完了の状態: 設定のテストで、上の各誤りの code・行と、無いときの `config-missing`、既定値で検査が続くことが確かめられる
  - _Requirements: 2.2, 2.4, 2.5, 2.10_

- [x] 2.2 pasta.dll と同じ辞書ファイルと辞書モジュールを選び出す
  - パターンを成分に分け、Windows の `glob::glob` を写す: ワイルドカードの無い成分は大文字・小文字を区別せず、ある成分は区別し、`**` は 0 個以上の成分に一致する。パターンとして読めなければ `config-pattern-invalid`（誤り）
  - 一致したファイルごとに「pasta.dll が見るパス」（ワイルドカードの無い成分はパターンの綴り）を作り、報告のパスは実物の綴りにする
  - `..`・ルートを含むパターンは使わず注意、`.pasta` で終わらないパターンは `.lua` を作らず注意。`.pasta` のパターンから `.lua` のパターンを作って辞書モジュールを集める
  - `profile/` を除き、重複をまとめ、相対パスの昇順で返す。`init.pasta`・`init.lua` は誤り、辞書が 1 つも無ければ注意
  - 一致テストのために、「pasta.dll が見るパス」の一覧を返す関数を `#[doc(hidden)] pub` で公開する
  - 完了の状態: 設定のテストで、`Dic/` のフォルダが既定のパターンに一致し、`dic/X.PASTA` は一致せず、`profile/` が除かれ、上の注意と誤りが出ることが確かめられる
  - _Requirements: 2.1, 2.3, 2.6, 2.7, 2.8, 11.8_

- [x] 2.3 モジュール名を作り、重なりを分類する
  - 「pasta.dll が見るパス」から、先頭の `dic` を外す → 最後の拡張子を外す → `.` と `-` を `_` → 区切りを `.` → `pasta.scene.` を前に付ける、でモジュール名を作る
  - `.pasta` どうし・`.lua` どうしの重なりは関わる各ファイルに誤り、`.lua` と `.pasta` の重なりはその `.lua` に注意。判定はモジュール名だけで行う
  - 一致テストのために、モジュール名を作る関数を `#[doc(hidden)] pub` で公開する
  - 完了の状態: 規則のテストで、`a-b`・`a.b`・`a_b` が誤り 3 件、`.lua` と `.pasta` が注意、`.lua` どうしが誤り、`dic/` の外と階層の違いの例が期待どおりになる
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 2.4 各辞書ファイルの構文を pasta.dll と同じ関数で読む
  - 本文を pasta.dll と同じ構文解析で読み、最初の構文エラーを行・列つきで誤りにする。理由は解析器が前に付ける位置の文言を外した残りにする
  - UTF-8 で読めないもの・読めないものを誤りにする。どのファイルで失敗しても残りを続け、読めた構文木を次の段に渡す
  - 完了の状態: 規則のテストで、構文エラーのある 2 ファイルの両方が報告され、位置が解析器の結果と同じで、UTF-8 で読めないファイルが誤りになる
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 2.5 トランスパイルで失敗する 3 つの書き方を構文木の上で見つける
  - シーンの引数への代入、式の中のプロパティ参照（代入の右辺がそのままのとき・台詞の中・プロパティへの代入の左辺は許す）、ローカルシーンの中で前にアクター行が無い継続行を誤りにする
  - 構文エラーと同じ 1 行の形にする
  - 完了の状態: 規則のテストで、3 つの誤りと、許される形（代入の右辺・台詞の中）が誤りにならないことが確かめられる
  - _Requirements: 3.5_

- [x] 2.6 シーンを登録し、照合用の名前の重なりと組み込みの名前を見つける
  - pasta.dll のトランスパイラと同じ手順で、別名を置き換えた名前でグローバルシーンを、名前のあるローカルシーンをその配下に、全ファイルで 1 つの登録表に登録する。構文エラーのあるファイルは登録しない
  - 組み込みの飛び先の名前の一覧（GLOBAL・act・SHIORI 用の act のメソッド）を定数で持ち、一致テストのために `#[doc(hidden)] pub` で公開する
  - 書いた名前が違い照合用の名前が同じグローバルシーン・ローカルシーンを、宣言ごとに注意にする（ほかの宣言の位置を示す）。同名・別名で同じになったもの・前方一致は報告しない。組み込みの名前と同じグローバルシーンを注意にする
  - 完了の状態: シーンのテストで、`＊会話·A` と `＊会話_A` が注意 2 件、`＊会話` と `＊OnTalk`・同名・`＊挨拶` と `＊挨拶朝` が報告されず、ローカルの重なりと `＊yield` が注意になる
  - _Requirements: 2.4, 5.1, 5.2, 5.3, 5.4_

- [x] 2.7 静的な飛び先の Call と選択肢を辞書のシーンと組み込みの名前だけで探す
  - Call はローカルの前方一致 → 組み込みの名前の完全一致 → 別名を置き換えたグローバルの前方一致の順で探し、無ければ注意。文字列リテラルだけの動的な Call も静的として扱う
  - 選択肢はローカル → 別名を置き換えたグローバルの順で探し、無ければ注意。`On`・`script:` で始まる先は見ない
  - それ以外の動的な Call は数えるだけにする。Lua の中身は候補にしない。どれかのファイルに構文エラーがあれば飛び先の検査を丸ごと省き、そのことをまとめに残す
  - 完了の状態: シーンのテストで、ローカルが先に見つかる・別名を置き換えたグローバル・組み込み（`transfer_req_to_var`）・見つからない Call と選択肢・`＠？OnBoot`・`＠？script:…`・`＞＄変数` の数・構文エラーで省いて案内の行が出ることが確かめられる
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8, 6.9_

- [x] 3. 一致テスト: 規則が同じ版の pasta.dll と同じことを守る
- [x] 3.1 モジュール名・選び出し・読み込みの成否・トランスパイルの成否を pasta_lua と照らす
  - `pasta_lua` の読み込みの部品の、ファイルの集め方の関数を `#[doc(hidden)]` で 1 行公開する（`loader/config/` には触れない。振る舞いは変えない）
  - 2.2・2.3 で `#[doc(hidden)] pub` にした関数を使い、(a) モジュール名、(b) 一時のフォルダ（`Dic/` のフォルダを含む）での選び出し（pasta.dll が見るパスどうしを比べる）、(c) 読み込みの成否と `pasta.toml`（`[loader]`・`[scene]` の誤りの見本）・`init.pasta`・`pasta.toml` が無いこと、(d) トランスパイルの成否と 3 つの書き方、を核の結果と照らす
  - 読み込みを呼ぶテストには `PASTA_DEBUG` を中和するガードを置く
  - 完了の状態: `cargo test -p pasta_check --test validate_parity_test` で (a)〜(d) が通り、`pasta_lua` のテストの振る舞いが変わらない
  - _Requirements: 9.2, 9.3, 9.4_

- [x] 3.2 組み込みの名前の一覧と静的な Call の検索の結果を実行時と照らす
  - (e) 読み込んだランタイムで GLOBAL と act・SHIORI 用の act の関数の名前を列挙し、2.6 で `#[doc(hidden)] pub` にした組み込みの名前の一覧と照らす
  - (f) 見本の辞書の静的な Call について、実行時のシーン検索の有無と核の判定が同じであることを照らす
  - 完了の状態: 一致テストの (e)(f) が通り、一覧と実物がずれるとテストが落ちる
  - _Requirements: 5.4, 6.1, 6.3, 9.2_

- [x] 4. `pasta_check validate` コマンド
- [x] 4.1 ゴーストのフォルダから一覧と中身を集めて核を呼ぶ殻を作る
  - `--target` が無い・存在しない・ディレクトリでない・`ghost/master` を含まないときは理由つきで失敗する（`ghost/master` そのものは読み替えない）
  - `ghost/master` から再帰で一覧を作る（最上位の `profile` に入らず、シンボリックリンクをたどらない、区切りは `/`）。中身はバイトで読み、UTF-8 で読めない・読めないを区別する
  - 報告の行を標準出力に出し、誤りがあれば 1、無ければ 0 で終わる。`release` からも使える、`ghost/master` を直接受ける入口を持つ。何も書き込まない
  - 完了の状態: 殻の関数が、誤りあり・誤りなし・フォルダの誤りの 3 通りを区別して返す
  - _Requirements: 1.1, 1.2, 1.3, 1.6, 3.4, 7.5, 7.6_

- [x] 4.2 サブコマンド validate を引数の解析と使い方に足す
  - `validate --target <path>` を受け付け、`--release`・`--nar`・`--copy` などは理由と使い方を出して失敗にする
  - 使い方に `validate` とそのオプションを `release` と並べる。`release` に検査を省くオプションは足さない
  - 完了の状態: `pasta_check --help` に validate が並び、`pasta_check validate --target <ゴースト>` が 4.1 の殻を通って終了コードを返す
  - _Requirements: 1.4, 1.5, 8.4_

- [x] 4.3 validate のコマンドのテストと hello-pasta の確認
  - 誤りあり→1、誤りなし（注意あり）→0、フォルダが無い・`ghost/master` を直接渡す・知らないオプション→1、出力が 2 回とも同じ、パスが `ghost/master/` から始まる
  - 実行の前後でゴーストのフォルダの一覧と更新時刻が変わらない
  - 完了の状態: CLI のテストが通り、hello-pasta（`crates/pasta_sample_ghost/ghosts/hello-pasta`）が誤り 0 で終了コード 0 になる
  - _Requirements: 1.1, 1.3, 1.4, 1.6, 7.2, 7.5, 7.6, 9.5_

- [x] 5. release の関門
- [x] 5.1 release に検査の段を足し、誤りがあれば更新ファイルと nar の前で止める
  - 上書きコピーの後に `[4/6] Validating dictionaries...` を足し、配布フォルダの `ghost/master` を validate と同じ殻で検査して同じ行を出す
  - 誤りがあれば英語の文言（`dictionary validation found N error(s); update files and NAR were not created`）で失敗させ、既存の `Error:` と終了コード 1 の処理に任せる。注意だけなら続ける
  - 段の表示を `[1/6]`〜`[6/6]` に振り直す（既存の段の中身とオプションは変えない）
  - 既存の release のテスト（`release.rs` の中のテストと `cli_test.rs`）の見本のゴーストは `pasta.toml` が無く関門で止まるので、最小の `pasta.toml` と正しい `.pasta` を足す
  - 完了の状態: release のテストで、誤りのある辞書では終了コード 1・`updates.txt` と nar が無い・出力に問題の行があり、注意だけでは nar ができ、表示が `[1/6]`〜`[6/6]` になり、既存の release のテストが通る
  - _Requirements: 7.5, 8.1, 8.2, 8.3, 8.4, 8.5_

- [x] 6. WASM の入口と CI
- [x] 6.1 (P) pasta_lsp に核を呼ぶ WASM の入口 2 つを足す
  - `pasta_lsp` に `pasta_check`（既定の feature 無し）を依存に足す
  - 辞書ファイルの選び出しと検査の入口を、既存の解析の入口と同じく panic を捕まえて `null` を返す形で足す。核の結果をそのまま返し、文言を作り直さない
  - 完了の状態: `pasta_lsp` のネイティブのテストで入出力の JS の形（camelCase・タグ `kind`）が照らされ、`cargo build -p pasta_lsp --target wasm32-unknown-unknown` が通る
  - _Boundary: WasmExports_
  - _Depends: 2.7_
  - _Requirements: 11.1, 11.2, 11.8_

- [x] 6.2 (P) CI の WASM の確認とリリースの公開の依存を足す
  - `build.yml` の wasm の job に、ライブラリだけを feature 無しで wasm32-unknown-unknown に向けた `cargo check` を足す
  - `release.yml` の `pasta_check` の公開の段に `-DependsOn pasta_core,pasta_dsl` を足す
  - 完了の状態: 2 つのワークフローの差分がこの 2 か所だけで、YAML として読める
  - _Boundary: CI_
  - _Depends: 1.1_
  - _Requirements: 9.1, 11.1_

- [x] 7. VSCode 拡張のゴースト全体の診断
- [x] 7.1 WasmBridge に型つきの呼び出し 2 つと形の検査を足す
  - 入力・ファイルの中身・報告の型を足し、2 つの呼び出しが WASM の結果の形を確かめて、違えば throw する
  - 完了の状態: 既存の bridge のテストと同じやり方で、正しい形を返し、形が違えば throw することが確かめられる
  - _Depends: 6.1_
  - _Requirements: 11.2, 11.8_

- [x] 7.2 拡張のテストの土台を足す
  - vscode のモックに `workspace.fs`・保存の通知・ファイルの見張り・`FileType`・`Uri.joinPath` を足す
  - `test:unit` に新しいテストのファイルを足す
  - 完了の状態: 空のゴースト検査のテストのファイルが `npm run test:unit` で走り、既存のテストが通ったままになる
  - _Requirements: 11.3_

- [x] 7.3 ゴーストのファイルを集めて検査し、診断を出す
  - 祖先の `ghost/master` を探し、`workspace.fs` で再帰に一覧を作る（最上位の `profile` に入らず、シンボリックリンクを含めない）。中身は厳格な UTF-8 で読み、読めない・UTF-8 でないを区別する。選び出しは bridge に任せ、TypeScript はパターンを見ない
  - 診断の集合 `pasta-ghost` に、誤りを Error・注意を Warning、`source`・`code` つきで、開いていないファイル・`pasta.toml` にも出す。前回あって今回無い URI の診断を消す
  - 開いている文書の `syntax` は出さず、閉じたら出し直し、開いたら外して出し直す。WASM の呼び出しが throw したらそのゴーストの診断を消して出力チャンネルに 1 行残す
  - 完了の状態: 拡張のテストで、開いていないファイルと `pasta.toml` に Error・Warning が出る、消えた問題の診断が消える、開いた文書の `syntax` が出ず閉じると出る、が確かめられる
  - _Requirements: 3.4, 11.2, 11.4, 11.5, 11.7, 11.8, 11.9, 11.11_

- [x] 7.4 保存とファイルの増減で検査を起動し、WASM が読めたときだけ登録する
  - `.pasta`・`ghost/master/pasta.toml` の保存、`ghost/master` の配下の `.pasta`・`.lua` の作成・削除で検査を起動する。`ghost/master` の外なら何もしない
  - ゴーストごとに 300ms で束ね、走行中に来たら終わった後に 1 回だけ走らせる。起動直後の全体検査はしない
  - `extension.ts` で WASM が読めたときだけ登録し、読めないときは作らない
  - 完了の状態: 拡張のテストで、配下の保存で 1 回走り、配下に無い保存では走らず、`pasta.toml` の保存と作成・削除で走り、WASM が読めないときは登録されないことが確かめられる
  - _Requirements: 11.3, 11.6, 11.10, 11.12_

- [x] 8. 文書
- [x] 8.1 マニュアルに検査コマンドの章を足し、目次と章の数の検査を合わせる
  - 「デバッグ」の部に、使い方・検査の項目・誤りと注意・各報告の意味と直し方・読むだけでは決まらない範囲・終了コード・release との関係（関門で止まると nar が残らないこと）・エディタの診断（保存のとき）を載せた章を足す
  - 目次に足し、章の数を決め打ちしている検査 4 つを 60 から 61 にする
  - 完了の状態: `mdbook build` と章の数の検査 4 つが通る
  - _Requirements: 10.1, 10.5, 10.6_

- [x] 8.2 pasta.toml のリファレンスを直し、スキル references を再生成する
  - 「エディタの言語機能と pasta_check は別名表を読まない」を、`pasta_check validate` と VSCode 拡張のゴースト全体の診断が別名表と `pasta_patterns` を読むことに直す
  - 完了の状態: スキル references の再生成の後、生成物の差分の検査が通る
  - _Requirements: 10.2_

- [x] 8.3 (P) README・スキル・steering を実態に合わせる
  - `crates/pasta_check/README.md` に validate の使い方、release の 6 段と関門、ソース構成と依存を載せる。`editors/vscode/README.md` に保存のときのゴースト全体の診断を載せる
  - スキル `pasta-check` に validate と関門を載せる（スキルの編集は分類器が止めることがあるので、止まったら開発者の許可を取る）
  - steering の `tech.md`・`structure.md` に pasta_check のライブラリ・依存・pasta_lsp からの依存・拡張の新しいファイルを書く
  - 完了の状態: 4 種の文書の記述が実装の段の表示・コマンドの使い方と一致する
  - _Boundary: Docs_
  - _Depends: 5.1, 7.4_
  - _Requirements: 10.3, 10.4, 10.6_

- [x] 9. 最終確認
- [x] 9.1 全体のテスト・静的検査・WASM のビルドを通す
  - `cargo test --all`、`cargo clippy`、`cargo fmt --check`、ライブラリの wasm32 の `cargo check`、`npm run test:unit`、マニュアルの検査を走らせる
  - `pasta_lua` のテストが書き換える生成物の改行の差分はコミットに混ぜない
  - 完了の状態: 上のすべてが通り、作業ツリーにこの spec の変更以外が残らない
  - _Requirements: 9.1, 9.5, 11.1_

## Implementation Notes

- 1.2: 理由の文言は `validate::msg` の関数にまとめた。`config-scene-alias` は重複の文言だけ用意済み（空の名前・連鎖・型の違いは 2.1 で足す）。並び順の code は FindingKind の宣言の順。serde の kebab-case と `code()` が同じことは 6.1 の JSON の形のテストで 1 行確かめる。
- 1.3: 各段の関数は crate 内の骨組み。2.1 で design の `read_config(src) -> (GhostConfig, Vec<Finding>)` に切り替え、GhostConfig を discovery と scenes に渡す（TOML を 2 度読まない）。参照を省くかは Syntax の Finding の有無で決める。
- 2.1: `PastaConfig::from_str` は純粋な文字列の解析で PASTA_DEBUG のガードは要らない（ガードは `PastaLoader::load` を呼ぶテストだけ）。読めない pasta.toml は `file-unreadable`（パス pasta.toml）。
- 2.2: pasta_lua の `discover_files` はソートも重複除去もしない（パターンごとに glob の順で連結）。3.1 (b) の一致テストは集合（並べ替え・重複除去の後）で比べる。`pasta_dll_paths(input) -> (Vec<String>, Vec<String>)` が .pasta と .lua の pasta.dll が見るパス。
- 2.4: 読めた構文木は `Vec<(String, PastaFile)>`（相対パスと組）で 2.5・2.6 へ渡る。飛び先の検査を省くのは要件 6.8 どおり `Syntax` があるときだけ（NotUtf8・Unreadable では省かないので、そのファイルのシーンへの Call が「見つからない」注意になり得る。どちらにせよ誤りで終わる）。
- 2.5: トランスパイルの規則はファイル内の該当箇所を全部報告する（pasta_lua は最初で止まる）。3.1 (d) は件数でなく有無で比べる。
- 2.6: 組み込みの名前の一覧は `__index` を含めない（実行時は L3 で届くが design が外している）。3.2 (e) の列挙でも `__index` を除く。`SceneModel.globals`・`GlobalScene.registered` の `#[allow(dead_code)]` は 2.7 で外す。単独の `＊` もローカルの重なりは検査する。
- 2.7: `＠？script:…` は DSL では書けない（choice の飛び先は id）ので判定関数の単体テストで確かめた。`＞（「X」）` は今は動的として数える（実行時は静的と同じ）。
- 3.1: 名前が `.pasta` で終わるフォルダ（`dic/x.pasta/`）は既定のパターンに一致し pasta.dll は読み込みに失敗するが、核の入力はファイルだけなので報告しない（要件 9.4 の範囲外として受け入れ）。一致テスト (b) は `is_file()` で絞る。`.lua` パターンの読み替えはテストに写した（`discover_all_files` は pub(super)）。
- 3.2: (f) は design の「SCENE.search の有無」でなく、実際の Call と同じ `find_act_handler`（current_scene をそのグローバルシーンの表にした SHIORI 用の act）で照らす（組み込みも含めて照らせるので強い）。
- 4.1: 殻は `validate_cmd::run(target: &Path)` と `run_on_master(master) -> Result<ValidateOutcome, String>`（サブフォルダが読めないときは Err。5.1 では io のエラーに写す）。main.rs の `mod validate_cmd;` の `#[allow(dead_code)]` は 4.2 で外す。`--target` が無いときは 4.2 の解析で扱う。
- 5.1: `ghost/master` の無い配布フォルダは段 4 で失敗する（理由は `run()` の「`ghost/master` がありません」ではなく、`run_on_master` の walk の「{path} を読めません: {OS のエラー}」）。前回の nar は既存の段 1 で消えるので、関門で止まると nar は残らない。
- 6.1: WASM の入口の戻り値は `Serializer::json_compatible()`（None を null にするため。既存の wasm_analyze は既定のまま）。
- 7.1: vscodeModules のテストは事前の WASM のビルド（`pwsh -NoProfile -File scripts/build-wasm.ps1 -Release`）が要る（既存の activate のテストも同じ）。
- 7.2: モックの `createDiagnosticCollection` は作った集合を記録しない。`outputLines`・`createdFileSystemWatchers` を空に戻す関数も無い（7.3 で集合を外から渡すか記録を足す）。
- 7.3: `GhostValidator(bridge, outputChannel)`、公開は `register()`（今は開く・閉じるだけ）・`validate(master)`・`dispose()`。7.4 で保存・見張り・300ms の束ね（design の `requestValidation`）を足し、extension.ts から出力チャンネルを渡す。列は核のコードポイント単位を -1 しただけ（design どおり）。
- 完了時にその場で解決: エディタの診断の列をコードポイントから UTF-16 に換算（同じ行の手前の絵文字などでずれない）。
- 完了時にその場で解決: `＞（「X」）`（括弧で囲んだ文字列リテラル）を静的な Call として探す（実行時の call_key と同じ）。
- 完了時に残した端のケース（裁定が要るので直していない）: (1) UTF-8 で読めない・読めない辞書があっても飛び先の検査は省かない（要件 6.8 は構文エラーだけ）。(2) 名前が `.pasta` で終わるフォルダは報告しない（核の入力はファイルだけ）。(3) VSCode 拡張は小文字の `ghost/master` だけを探す（CLI は Windows で大文字・小文字を区別しない）。
