# 実装計画

進め方は設計の Migration Strategy の 4 段（`@toml` を足す → `OnPastaSetSaveVariable` を型で受ける → 6 つと依存を外す → 文書）に従う。どの段の後でもテストが通る。`cargo build`・`cargo test` の前に `NoDefaultCurrentDirectoryInExePath` を外す。`pasta_lua` のテストが書き換える `sample.generated.lua` の改行の差分は `git checkout --` で戻す。

- [ ] 1. `@toml` を足す（6 つのモジュールはまだ残す）
- [x] 1.1 TOML の文字列を Lua の表に読む `@toml` を、設定によらず初期化の時から登録する
  - `decode` だけを持つ表を `@pasta_log` の隣で必ず登録し、ほかの登録の順は変えない
  - 値の変換は `@pasta_config` と同じ既存の変換を使い、日付と時刻は RFC 3339 の形の文字列になる
  - 読めない文字列（UTF-8 として正しくないものを含む）は `nil` とエラーメッセージを返し、文字列でない引数（数・`nil`・表）は Lua のエラーにする
  - 完了: `RuntimeConfig::minimal()` の VM で `require("@toml").decode('a = 1')` が `{a = 1}` を返し、`cargo build -p pasta_lua` が通る
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 6.1_
- [x] 1.2 `@toml` の結合テストを足す
  - 新しいテストのファイルを runtime のテストの入口に登録する
  - 正しい TOML（文字列・整数・小数・真偽値・日付・配列・入れ子の表）が `@pasta_config` と同じ値になり、日付は `1979-05-27T07:32:00` の形になる
  - 読めない文字列で `nil` と空でない文字列、`encode` が `nil` で鍵が `decode` だけ、2 回の `require` が同じ表で続けた `decode` が混ざらない、`decode(42)`・`decode(nil)`・`decode({})` が `pcall` で `false`
  - 読み込んだゴーストの `main.lua` の先頭で `require "@toml"` できる
  - 完了: 新しいテストがすべて通る
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 6.4_

- [x] 2. `OnPastaSetSaveVariable` を Reference2 の型で受け、`@json` を使わなくする
  - 失敗の種類の「JSON として読めない値」の席を「型の名前にならない値」（鍵は `BAD_TYPE`）に置き換える。`NOT_NUMBER`・`NOT_ASSIGNABLE` はそのまま、種類は 12 のまま。失敗の種類の Lua のテストを、12 種・`BAD_TYPE` の文言・`NOT_JSON` が無いことに直す
  - 既定ハンドラの確かめの順は 門 → Reference0 → Reference2 → `delete` なら消す → Reference1 があるか → 型ごとに読む。型が無い・空・`string` は届いた文字列をそのまま、`number` は Lua の標準の `tonumber` で読み無限大・非数を断る、`boolean` は `true`・`false` の完全一致、`delete` は Reference1 を見ずに消す
  - 失敗は設計の失敗の表のとおりの種類・表記・ログの警告（`not a number`・`not a boolean`・`unknown type`・`no value`）を、既存の失敗の出口から出し、変数に触れない。`@json` を読まず、冒頭の説明を新しい形に直す。`OnPastaCallFunction` と共有の局所関数の働きは変えない
  - 外部操作のイベントの Lua のテストを書き直す（試験の助けを型を受ける形にし、「土台の `@json`」の試験を消す）: 型を省く・空・`string` で `01`・`"01"`・`60`・`null`・空の文字列が文字列のまま、`number` で `60`・`-1.5`・` 60 `・`0x10`・`1e3` が数値、`boolean` で `true`・`false`（`false` が消えない）、`delete` で Reference1 が無い・空・何かの文字列でも消える、戻り値が `nil`、名前の `a.b` と予約変数。失敗の表の各行で表記・警告 1 行・前の値が残ること、`Number`・`数値` が知らない型、`balloon_warning = false` で表記なし・ログあり。門と Reference0 の試験は今のまま通す
  - Rust の通しのテストの `&["会員番号", "\"01\""]` を `&["会員番号", "01"]` にし、その後のシーンが `01` を話すことを確かめる
  - 失敗の種類・ハンドラ・2 つのテストは途中で分けるとテストが赤くなるので、1 つの段として一緒に直してコミットする
  - 完了: ハンドラのファイルに `@json` が無く、失敗の種類と外部操作のイベントの Lua のテストと Rust のテストがすべて通る
  - _Requirements: 1.3, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 6.2, 6.4_

- [ ] 3. mlua-stdlib の 6 つのモジュールと依存を外す
- [ ] 3.1 6 つの登録と mlua-stdlib への依存を外し、それを使っていたテストを直す
  - VM の初期化から 6 つの登録を消し、`new()` の文書を `@toml` の 1 行に直す
  - ルートと `pasta_lua` の `Cargo.toml` から mlua-stdlib を消し、`Cargo.lock` はビルドで落とす
  - Lua のテストの走らせ役の `@json` の登録を消し、応答のテストの `@assertions` を素の `assert` に直す
  - 6 つを確かめていた 2 つのテストのファイルを消し、runtime のテストの入口から外す
  - 完了: `Cargo.lock` に `name = "mlua-stdlib"` が無く、`cargo tree -i mlua-stdlib` が見つからず、`cargo test -p pasta_lua` が通る
  - _Requirements: 1.1, 1.2, 6.5_
- [ ] 3.2 Rust の設定から 6 つの名前と `env` の警告を外し、誤りの文を `std_` の名前だけにする
  - `default_libs()`・`new()`・`minimal()` は `["std_all"]`、`full()` は `["std_all_unsafe"]`。`should_enable_module` を消す
  - `validate_and_warn` から `env` の分岐を消す（`std_debug`・`std_all_unsafe` の警告は残す）。設定の文書と注釈から mlua-stdlib を消す
  - `UnknownLibrary` の文を 13 の `std_` の名前だけにする
  - 単体テスト: 既定の名前、`std_` でない名前（6 つの名前）を飛ばすこと、`env` を並べても警告せず `std_debug` は警告すること、誤りの文に 6 つの名前が無いこと。`should_enable_module` と `env` の試験は消す
  - 完了: `pasta_lua` の単体テストが通り、`crates/pasta_lua/src` に `should_enable_module` が無い
  - _Requirements: 1.4, 1.5, 1.6, 1.7_
- [ ] 3.3 6 つのモジュールが入らないことの結合テストを足す
  - `RuntimeConfig::new()` と、6 つの名前を並べた `from_libs` の VM のどちらでも、6 つの `pcall(require, …)` が `false`
  - 完了: 足したテストが通り、`pasta_scripts` と `crates/pasta_lua/src` を 6 つのモジュール名で検索して 0 件
  - _Requirements: 1.1, 1.3, 1.5_

- [ ] 4. マニュアル・スキル・文書を今の姿にする
- [ ] 4.1 (P) mlua-stdlib の章を `@toml` の章に置き換え、Lua 標準ライブラリの節を一覧の頁へ移す
  - `@toml` の章は、読み込み方・`decode` の使い方と値の対応の表・失敗の返し方・`encode` を持たないこと・JSON・YAML・正規表現の部品が無く文字列は Lua の標準の関数で調べること。冒頭と末尾にほかの章と同じ会話を置く
  - 目次の行を置き換え、章の数は 63 のまま。古い章のファイルを消す
  - モジュールの一覧の頁に `@toml` の行を足し、mlua-stdlib の段落を置き換え、Lua 標準ライブラリの節を移す（`ffi` は `[loader] ffi = true` で開けることを書き、pasta.toml の頁へリンク）。Lua の章の入口のリンクを新しい節に直す
  - references を作る道具の照合（`gen-skill-refs.mjs --check`）は、章の対応を直す 4.5 まで落ちたままでよい
  - 完了: `link-check.mjs` と章の会話の検査が通り、新しい節へのリンクが届く
  - _Boundary: ManualDocs（toml.md・SUMMARY.md・lua/modules/index.md・lua/index.md）_
  - _Requirements: 5.1, 5.3, 5.5_
- [ ] 4.2 (P) SHIORI イベントの章の `OnPastaSetSaveVariable` の節と既定ハンドラーの表の行を、型で受ける形に書き直す
  - 見出しを変えずに、Reference0〜2 の表、型の表（型を省けば文字列）、失敗の表、MCP からの呼び方の例 3 つを書き、トークの間隔の変数は `number` を付けて送ると書く
  - JSON・引用符の二重の包み・`null`・配列とオブジェクト・`@json` の行を消す。表は `OnPastaSetSaveVariable` の行の文だけを直す（行の数と「9 つのイベント」の文は触らない）
  - 完了: 節に「JSON」と `@json` が無く、`grammar/variables.md`・`internals/shiori.md` からのリンクが届く
  - _Boundary: ManualDocs（lua/shiori-events.md）_
  - _Requirements: 5.4_
- [ ] 4.3 (P) ほかのマニュアルの頁から、6 つのモジュールと mlua-stdlib の記述を無くす
  - DSL と Lua の使い分け・パターン集・`@pasta_log` の章・実行モデルとローダーの内部の頁を、設計の Modified Files のとおりに直す
  - 完了: `book/src` を 6 つのモジュール名と `mlua-stdlib` で検索して 0 件
  - _Boundary: ManualDocs（dsl-vs-lua.md・patterns.md・pasta-log.md・internals/execution-model.md・internals/loader.md）_
  - _Requirements: 5.2_
- [ ] 4.4 (P) README と steering と試験の一覧から mlua-stdlib を消す
  - `crates/pasta_lua/README.md` の組み込みモジュールの文と例の注釈・`from_libs` の例を直す
  - `tech.md` の依存の行と `@env` の行を消し `@toml` の行を足す。`structure.md`・`TEST_COVERAGE.md` の消したテストの 2 行を `toml_module_test.rs` の 1 行にする
  - 完了: 4 つのファイルを `mlua-stdlib`・`@env`・`stdlib_modules_test`・`stdlib_regex_test` で検索して 0 件
  - _Boundary: ReadmeSteering_
  - _Requirements: 5.7_
- [ ] 4.5 スキル `pasta-lua-coding` を直し、references を作り直す
  - references を作る道具とその自己テストの章の対応を `lua/modules/toml.md` にする
  - 道具で references を作り直し、消えた章の写しを手で消す
  - `SKILL.md` の表・references の表・統合モジュールの表・モジュールの一覧とリンク・`OnPastaSetSaveVariable` の説明を直す（分類器に止められたら、入った分と残りを示して開発者の許可を求める）
  - 完了: `node book/tools/gen-skill-refs.mjs --check` と `gen-skill-refs-test.mjs` が通り、`references/mlua-stdlib.md` が無い
  - _Depends: 4.1, 4.2, 4.3_
  - _Requirements: 5.6, 5.8_

- [ ] 5. 全体の確かめと申し送り
- [ ] 5.1 全体のテストと検査を走らせて、残りの動きが変わっていないことを確かめる
  - `cargo test --workspace`・`cargo clippy`・触ったファイルの `cargo fmt --check`
  - マニュアルの検査（`gen-skill-refs.mjs --check`・`link-check.mjs`・章の数を決め打ちする自己テスト・会話の検査）。`book/node_modules` が無いワークツリーで落ちる自己テストは spec のせいではないと分けて記録する
  - `crates/`・`book/src`・`.claude/skills/pasta-lua-coding`・`crates/pasta_lua/README.md`・`.kiro/steering` を 6 つのモジュール名と `mlua-stdlib` で検索し、残るのは `tests/loader/config_test.rs` の `env` のテストだけであること（`.kiro/specs` と `roadmap.md`・出した版のリリースノートは記録なので対象外。spec の名前 `mlua-stdlib-removal` を挙げた行も数えない）
  - 第三者のライセンスの一覧は git で追っておらず、リリースの CI が依存の木から作り直すので、`cargo tree -i mlua-stdlib` が見つからないことでその一覧からも消えると確かめたことにする
  - 完了: すべての検査が通り（または node_modules による既知の落ち方だけで）、hello-pasta を含む既存の通しのテストが通る
  - _Requirements: 1.1, 1.2, 1.3, 5.8, 6.1, 6.2, 6.3, 6.5_
- [ ] 5.2 破壊的変更の知らせ方を申し送りに残す
  - PR の題名に `!` を付ける案（6 つのモジュールを外すことと `OnPastaSetSaveVariable` の形を変えることの 2 つ）と、`release` スキルへの「動きが変わる点」2 点と `RuntimeConfig` の 1 行を、`kiro-complete` が読める所（research.md の申し送りの節）に書く
  - 完了: research.md に PR の題名の案と申し送りの 3 行がある
  - _Requirements: 6.6_

## Implementation Notes

- `from_loader` は main.lua を読まない。main.lua を通す試験は `PastaLoader::load`（`from_loader_with_scene_dic`）で読み込む（1.2）
