# Implementation Plan

> 進め方：設計の「Migration Strategy」の 11 手順に沿う。1 つ直すごとに確かめて 1 コミットにし、どのコミットでも品質の関門（`cargo fmt --check`・`cargo clippy --all-targets` 警告 0・`cargo test -p pasta_dsl -p pasta_lua -p pasta_check -p pasta_lsp`）が通る。挙動を変える直しは、直す前の誤りを再現するテストを先に書き、直す前に失敗する（あふれはテストのプロセスが落ちる）ことを確かめてから、直しと一緒にコミットする。
>
> - cargo の前に `NoDefaultCurrentDirectoryInExePath` を外す。`pasta_lua` のテストが `sample.generated.lua` の改行だけを書き換えたら `git checkout --` で戻す。
> - 変えないもの：`crates/pasta_dsl/src/partial.rs`・`grammar.pest`・`crates/pasta_lua/src/code_gen/`・ルートの `Cargo.toml`・`Cargo.lock`。
> - 直す前の AST の控えは `target/ast-baseline/`（gitignore 済み）に置く。セッションのスクラッチには置かない（タスクごとに別のセッションで走るため）。
> - 設計の手順からの 3 つのずれ（どれも「どのコミットでも関門が通る」を守るため）：
>   - 速さのテストは、手順 1 でなく手順 3（タスク 3）で入れる。手順 1 では比のテストが落ちる。
>   - `filename` の受け渡しと上限の定数は、手順 2（タスク 2）で一度に入れず、使う関数・使うタスク（4.2・5.x・6・7.1）で足す。手順 2 で入れると、使われない引数と定数で clippy の警告になる。
>   - `PastaFile.span` の撤去（タスク 9）では、`pasta_lua` のテストのコード 2 ファイルも直す。設計は「`pasta_lua` に使い手は無い」と書いているが、構造体の字句のテストが `span:` を書いている。どちらも `code_gen/` の外のテストのコードである。

- [x] 1. 振る舞いを固めるテストを置き、直す前の AST の控えを取る
  - リポジトリの全 `.pasta`（`crates/` と `.kiro/` の下。`target`・`node_modules` を除き、解析に成功したもの）と、端の辞書（CRLF・BOM つき・絵文字と全角・単独の `\r` を含む台詞・最後の改行が無い辞書）で、AST のすべての span の行と桁が、バイトの位置から pest の `Position::line_col` で出した値と一致することを確かめる。拾った span が 1 つ以上あることも確かめる
  - `#[ignore]` のテストで、環境変数のフォルダへ全 `.pasta` の `items`（誤りなら `Err`）を書き出せるようにする
  - 直す前の AST を `target/ast-baseline/` に書き出し、タスク 11.2 まで消さずに置く
  - 直す前のコードで span の正解のテストが通り、`target/ast-baseline/` に全 `.pasta` の書き出しがある。この手順だけの 1 コミットになっている
  - _Requirements: 1.3, 3.3, 8.1, 8.3_

- [x] 2. 位置つきの誤りの補助を置き、pest の誤りの写しをそれに寄せる（挙動は変えない）
  - 新しい誤りと pest の誤りを、同じ `SyntaxError` の形と `Parse error in {file} at {l}:{c}: ` の前置きで作る補助を置き、`parse_str` の pest の誤りの写しをそれで作る（文言は 1 文字も変えない）
  - `filename` の受け渡しと上限の定数は、ここでは足さない（使うタスクで足す）
  - 既存のテストと span の正解のテストが変わらず通り、`error_api_test.rs` の `parse_str` の誤りの位置と文言のテストが通る。clippy の警告が 0 である
  - _Requirements: 1.4, 4.3, 8.3_

- [ ] 3. span を数え直さずに作り、解析の時間を行数に比例させる
  - 速さのテスト（ほかと並んで走らないよう、1 本だけの別のテストのファイル）を先に書く：同じ形の辞書 2,000 行と 8,000 行を 3 回ずつ解析し、最短の時間どうしの比が 8 倍以内。`#[ignore]` で、release の 8,000 行が 1 秒未満と、32,000 行／8,000 行の比が 8 倍以内
  - 3 回の最短を取ることで、機械の揺れを抑える。直す前の比は約 16 倍、直した後は約 4 倍の見込みで、8 倍の線の両側に 2 倍の幅がある
  - 直す前に、debug の比のテストが失敗することを確かめる
  - span の部品を足す。始まりは pest の行の索引から取り、終わりは節の文の中だけで数える。17 か所の span の作り方をそれに置き換え、古い変換を消す
  - span の正解のテストと速さのテストが通り、`cargo test -p pasta_dsl --release --test parse_speed_test -- --ignored` で 8,000 行が 1 秒未満になる
  - _Requirements: 1.1, 1.2, 1.3, 8.3_

- [ ] 4. 数の字句を 1 つの規則で値にする
- [ ] 4.1 式の中の誤りを飲まずに上へ返す形にする
  - 式の項・式の並び・シーン引数の参照・動的な名前を読む関数の戻り値を、「誤りか、項になるか・ならないか」の形にする。関数呼び出しの中の誤りを黙って捨てていた 3 か所は、上へ返す
  - 「項にならない」の意味は今のまま保つ（今の関数呼び出しの読み方は誤りを作らないので、挙動は変わらない）
  - 既存のテストと span の正解のテストが通り、AST の書き出しが `target/ast-baseline/` と同じである
  - _Requirements: 2.4, 8.3_
- [ ] 4.2 数の値の規則を 1 か所に置き、式・属性値・キューの引数で使う
  - 数の値のテスト（新しいテストのファイル）を先に書き、直す前に失敗することを確かめる。確かめる中身：
    - i64 の上限・下限・上限＋1、20 桁・400 桁の整数、400 桁の小数
    - 要件 2.4 の 4 つの例
    - 同じ字句が、式・位置の引数・名前つきの引数・Call の動的ターゲット・括弧の中・属性値・キューの引数で同じ値になること
    - 誤りの行・桁（全角の字句では、桁が文字の数）・前置き・文
  - 規則を 1 つの部品にする：小数点が無く i64 に収まれば整数、収まらなければいちばん近い小数、小数点があれば小数、有限でなければ字句の位置の構文の誤り。3 か所の使い手をそれに寄せ、属性値だけの古い読み方を消す
  - その部品から上の呼び出しの道に、位置つきの誤りに要る `filename` を渡す
  - マニュアルを直す：
    - `grammar/literals.md` に、要件 7.1 で確定した文を足す
    - `internals/transpiler.md` に、数の値の規則を足す
    - `node book/tools/gen-skill-refs.mjs` で参照を作り直す
  - 数の値のテストが通り、`＄x＝10000000000000000000` が `1e19`、`＄y＝1＋10000000000000000000` が `1 ＋ 1e19` になる。`gen-skill-refs.mjs --check` に差分が無い
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 4.3, 7.1, 7.3, 8.3_

- [ ] 5. 番号の範囲の外を構文の誤りにする
- [ ] 5.1 アクターの番号の範囲を検査する
  - 数の値のテストに次の 2 つを足し、直す前に失敗することを確かめる：`％a、b＝4294967296` の誤り（`digit_id` の位置・文）と、`％a＝4294967295、b` の採番（b は 4294967295）
  - 0〜4294967295 に収まらない番号を、その番号の位置の構文の誤りにする。範囲の中の番号と、省いた番号の採番は今のまま。アクター指定行を読む関数に `filename` を渡す
  - `grammar/actor-dictionary.md` と `internals/transpiler.md` を直し、参照を作り直す
  - 足したテストが通り、範囲の中の番号の AST が `target/ast-baseline/` と同じである
  - _Requirements: 2.7, 4.3, 7.1, 7.3, 8.3_
- [ ] 5.2 シーン引数の番号の範囲を検査する
  - 数の値のテストに次を足し、直す前に失敗することを確かめる：`＄２５５` は通り、`＄２５６` はアクション行・式の項・`＠＄２５６` で `＄` の次の位置の誤りになる
  - 0〜255 に収まらない番号を、その番号の位置の構文の誤りにする。シーン引数を読む関数まで `filename` を渡す
  - `grammar/variables.md` の「番号は 0〜255」の行と `internals/transpiler.md` を直し、参照を作り直す
  - 足したテストが通り、`gen-skill-refs.mjs --check` に差分が無い
  - _Requirements: 2.8, 4.3, 7.1, 7.3, 8.3_

- [ ] 6. 1 行の括弧の重なりを 64 段に抑える
  - 深さのテスト（新しいテストのファイル）を先に書く。1 MiB のスタックのスレッドで、解析と片付けまで走らせる。確かめる中身：
    - `（`・`＠f（`・`＠＄f（` の 64 段は通り、65 段は 65 段目の位置の誤りになる。1 万段も落ちずに誤りになる
    - 台詞・コメント・Lua ブロックの行の 65 段も誤りになる
    - 閉じ括弧の多い行で、深さが負にならない
    - 部分解析に 65 段の行を含む文書を渡しても落ちない
  - 直す前に、65 段・1 万段のテストが失敗する（プロセスが落ちる）ことを確かめる
  - 括弧の重なりの上限の定数（64）を置く。`parse_str` で BOM を外した後、pest の前に、各行の括弧の重なりを数える走査を入れる（行と桁は、pest の行の索引と同じ数え方）
  - `grammar/variables.md` の「優先順位と括弧」に確定した上限の文を足し、`internals/transpiler.md` を直し、参照を作り直す
  - 深さのテストの括弧の分と、span の正解のテストが通る
  - _Requirements: 1.4, 3.1, 3.3, 3.4, 3.5, 3.6, 4.3, 7.1, 7.3, 8.3_

- [ ] 7. 1 つの式の演算子を 256 個に抑える
- [ ] 7.1 演算子の数の検査を入れる
  - 深さのテストに次を足し、直す前に 1 万個のテストが失敗することを確かめる：
    - 演算子 256 個（括弧 64 段の内側に置いた、上限ちょうどの形を含む）は通り、257 個は 257 個目の位置の誤りになる
    - 括弧をまたいで合わせて 257 個も誤りになる。2 つの関数呼び出しの引数に 200 個ずつは通る
    - 1 万個も落ちずに誤りになる
    - 部分解析に 1 万個の行を含む文書を渡しても落ちない
  - 演算子の数の上限の定数（256）を置く。式の鎖を組む 3 つの関数の頭で、渡された節の子孫の二項演算子を記述順に数え、257 個目でその位置の誤りを返す（鎖を組む前に止める）。その 3 つの関数まで `filename` を渡す
  - `grammar/variables.md` と `internals/transpiler.md` の上限の記述が演算子の分もそろっていることを確かめ、参照を作り直す
  - 深さのテストが全部通る
  - _Requirements: 3.2, 3.3, 3.5, 3.6, 4.3, 7.1, 7.3, 8.3_
- [ ] 7.2 トランスパイルと pasta.dll の読み込みで落ちないことを確かめる
  - `pasta_lua` に新しいテストのファイルを置く。上限ちょうどの 5 つの形を、1 MiB のスレッドでトランスパイルまで走らせる。5 つの形は、関数呼び出し 64 段・括弧 64 段・動的な関数呼び出し 64 段・アクション行の関数呼び出し 64 段・Call の引数の中の 63 段で、どれも演算子 256 個とする
  - 同じファイルに、ローダのテストを 1 本置く。65 段の括弧の行を含む辞書のゴーストを、既存のローダのテストと同じ形で読み込む。プロセスが落ちず、`parse_str` と同じ行・桁の構文の誤りの読み込みエラーで返ることを確かめる。`PASTA_DEBUG` を中和する、既存のテストと同じ守りを使う
  - `pasta_lua` のソースは変えない
  - `cargo test -p pasta_lua --test parser_limit_stack_test` が debug と `--release` で通る
  - _Depends: 6, 7.1_
  - _Requirements: 3.4, 3.6_

- [ ] 8. 作られない誤りの型を消す
  - 誤りの型を `SyntaxError` と `IoError` だけにする。作られない 2 つの型・誤りの情報の型・結果の別名・3 つのコンストラクタを消す。ファイルの読み込みの誤りからの変換は残す
  - 再輸出を誤りの型 1 つだけにする
  - `error_api_test.rs` の、消した型のテストを消す。`SyntaxError` の形のテストは、`parse_str` が返す誤りで確かめる形にする
  - `crates/pasta_dsl/README.md` の誤りの型の一覧と、`lib.rs` の説明を今の型に書き直す。公開 API の断り書きは足さない
  - `cargo test -p pasta_dsl -p pasta_lua -p pasta_check -p pasta_lsp` が通り、`pasta_dsl` の外に消した名前の使い手が無い
  - _Requirements: 4.1, 4.2, 7.2, 7.4, 8.3_

- [ ] 9. ファイル全体の span を消す
  - `PastaFile` から、ファイル全体の span の欄とその計算を消す
  - `pasta_dsl` のテストを直す：
    - `error_api_test.rs` の該当テストを、「空行だけ・コメントだけの辞書の解析が成功し、項目が無い」に直す
    - `span_byte_offset_test.rs` の 2 本を、各節（グローバルシーン・アクション行）で確かめる形に直す。`start_byte < end_byte` と、`extract_source` が元の文に合うことを確かめる
  - `pasta_lua` のテストのコードも直す。`PastaFile` を字句で作っている所から `span:` の行を消し、使われなくなった import も消す。所は `src/transpiler_tests.rs`（6 か所）と `tests/transpiler/comparison_test.rs`（3 か所）で、どちらも `code_gen/` の外である
  - 直したテストが通り、clippy の警告が 0 で、ファイル全体の span を読み書きするコードがリポジトリに無い
  - _Requirements: 5.1, 5.2, 8.3_

- [ ] 10. 使われない部品と隠れた拾い方を消す
- [ ] 10.1 式の項として扱っていない規則を黙って拾わない
  - `parser/mod.rs` の単体テストを先に足す。項として扱わない規則の節を項の読み方に渡すと、位置つきの `内部の誤り` になることを確かめる
  - 項の読み方の、演算子の腕と既定の腕を、位置つきの誤りを返す 1 つの腕にする
  - 足した単体テストが通り、AST の書き出しが `target/ast-baseline/` と同じである
  - _Requirements: 6.1, 8.3_
- [ ] 10.2 キューのスコープの分け方と、同語反復の分岐を整える
  - キューのスコープを、子の文を 1 回の区切りで分ける形にする。起きない場合の守りと、届かない戻り値を消す
  - AST を組むときの、ファイルの終わりの規則を何もせずに読み飛ばす腕を消し、既定の腕だけにする
  - 既存のキューのテストが通り、AST の書き出しが `target/ast-baseline/` と同じである
  - _Requirements: 6.3, 6.4, 8.3_
- [ ] 10.3 テストだけが使う span の長さを消す
  - span の長さの関数と、そのテストを消す。`extract_source` と `SpanError` は残す
  - `cargo test -p pasta_dsl` が通り、span の長さの関数がリポジトリに無い
  - _Requirements: 6.2, 8.3_

- [ ] 11. 全体の確かめと品質の関門
- [ ] 11.1 (P) `pasta_check validate` が pasta.dll と同じ結果を出すことを確かめる
  - `validate_rules_test.rs` にテストを 1 本足す：
    - 大きすぎる数・アクターの番号・シーン引数の番号・括弧の重なり・演算子の数の辞書で、`syntax` の誤りが 1 つずつ出る。位置は `parse_str` と同じ行・桁で、文は `構文エラー: {文}`
    - 20 桁の整数の辞書は、誤り無しになる
  - `pasta_check` のソースは変えない
  - `cargo test -p pasta_check` が通る
  - _Boundary: pasta_check tests_
  - _Requirements: 2.9, 4.4_
- [ ] 11.2 (P) 直した後の AST を控えと比べる
  - 全部の直しの後に AST を書き出し、`target/ast-baseline/` と `diff -r` で比べる
  - 差分が無いという出力を、タスクの証拠に残す（要件 2・3 で決めた変化は、リポジトリの辞書に現れない）
  - _Boundary: SpanOracleTest_
  - _Requirements: 3.3, 8.1_
- [ ] 11.3 (P) x86 と x64 の、debug と release で深さのテストを走らせる
  - 先に準備を確かめる：`rustup target list --installed` に `i686-pc-windows-msvc` があること（無ければ足す）。`NoDefaultCurrentDirectoryInExePath` を外してあること（x86 の LuaJIT のビルドに要る）
  - 次の 2 つを、`i686-pc-windows-msvc`・`x86_64-pc-windows-msvc` のそれぞれで、debug と `--release` の 4 通り走らせる。x86 は省かない
    - `cargo test -p pasta_dsl --test nesting_guard_test --test span_oracle_test --test number_literal_test`
    - `cargo test -p pasta_lua --test parser_limit_stack_test`
  - 4 通りすべてで通る出力が残る
  - _Boundary: NestingGuardTest, ParserLimitStackTest_
  - _Depends: 7.2_
  - _Requirements: 3.6_
- [ ] 11.4 品質の関門を通し、変えないものを確かめる
  - 関門を通す：`cargo fmt --check`・`cargo clippy --all-targets`（警告 0）・`cargo test -p pasta_dsl -p pasta_lua -p pasta_check -p pasta_lsp`・`node book/tools/gen-skill-refs.mjs --check`
  - `expr_parse_test.rs` の括弧式の回帰テストが残っていて、通る
  - main からの差分に、`partial.rs`・`grammar.pest`・`pasta_lua/src/code_gen/`・ルートの `Cargo.toml`・`Cargo.lock` が無い
  - README と `.kiro/steering/tech.md` に、公開 API の断り書きを足していない
  - すべての関門が通る出力と、変えないファイルの差分が空の出力が残る
  - _Requirements: 7.4, 8.2, 8.4, 8.5_
