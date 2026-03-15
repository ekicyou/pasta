# Implementation Plan

- [x] 1. `KeyWords` ASTを複数キー対応に拡張し、既存参照箇所をすべて修正する
  - `KeyWords` 構造体の `name: String` フィールドを `names: Vec<String>` に変更する
  - `pub fn name(&self) -> &str` ヘルパーメソッドを追加し `&self.names[0]` を返す
  - `parse_key_words()` のコンストラクタを `names: vec![name]` 形式に更新する（文法変更前の暫定対応）
  - `pasta_lua` 内で `kw.name` / `word.name` / `word_def.name` と書かれた7箇所をすべて `kw.name()` 等のメソッド呼び出しに修正する
  - テストコード内の構造体リテラル計10箇所を `names: vec![...]` 形式に修正し、フィールドアクセス1箇所（`actor_code_block_test.rs`）を `.name()` 呼び出しに修正する
  - この時点で `cargo test --all` が通ること
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

- [x] 2. PEG文法に複数キー構文を追加し、パーサーを更新する
- [x] 2.1 PEG文法に `key_list` ルールを追加し `key_words` ルールを更新する
  - `key_list = { id ~ ( comma_sep ~ id )* }` ルールを追加する
  - `key_words` ルールを `key_list ~ s ~ kv_marker ~ s ~ words` 形式に変更する
  - 既存の `comma_sep` ルールを再利用することで全角・半角カンマを自動サポートする
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 3.1, 3.2_
- [x] 2.2 `parse_key_words()` を `key_list` 内の全 `id` を `names` に収集するよう更新し、テストを追加する
  - `Rule::id` の直接マッチを `Rule::key_list` 内のイテレーションに変更し、全キーを `names` に収集する
  - 複数キー入力（2キー・3キー）のパーステストを追加する
  - 単一キー入力が従来と同一のAST（`names.len() == 1`）を返すことを確認するテストを追加する
  - この時点で `cargo test --all` が通ること
  - _Requirements: 1.1, 1.2, 1.3, 2.1_

- [x] 3. (P) pasta_lua のレジストリ登録を全キー対応にする
  - `context.rs` の `register_global_words()` / `register_local_words()` ヘルパーメソッドを削除する
  - `transpiler.rs` の LocalWord 登録パスをヘルパー経由から `word_registry` 直接呼び出しに変更する
  - `transpiler.rs` の GlobalWord / ActorScope 各パスに `names.iter()` ループを追加し、全キーに対して登録を実行する
  - `context.rs` の `test_register_global_words` / `test_register_local_words` テストを `word_registry` 直接呼び出し形式に改修し、複数キーの登録ケースを追加する
  - 複数キーを持つ `KeyWords` をトランスパイルした場合に全キーがレジストリに登録されることを検証する統合テストを追加する
  - _Requirements: 4.1, 4.3, 4.4_

- [x] 4. (P) pasta_lua のコード生成を全キー対応にする
  - `generate_global_word()` / `generate_local_word()` に `names.iter()` ループを追加し、各キーに対して `PASTA.create_word(key):entry(...)` / `SCENE:create_word(key):entry(...)` を出力する
  - `generate_actor()` に `names.iter()` ループを追加し、各キーに対して `ACTOR:create_word(key):entry(...)` を出力する
  - 単一キー入力時に既存と同一の出力を生成することを確認する
  - 複数キー入力時に全キー分の `create_word` 呼び出しが出力されることを検証するスナップショットテストを追加する
  - `cargo test --all` が通ること
  - _Requirements: 4.2, 4.3_
