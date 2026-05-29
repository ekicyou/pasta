# 実装計画

- [ ] 1. 静的解析ベースライン確立
- [ ] 1.1 clippy警告の収集と分類 (P)
  - `cargo clippy -p pasta_dsl -- -D warnings` を実行し、全警告を記録する
  - 警告カテゴリ別（unused_imports, dead_code, unreachable_patterns, complexity等）に分類する
  - ベースライン記録完了後、以降のタスクで警告ゼロを目標とする
  - 完了条件: 現在の警告リストが作成され、各警告の対処方針が決定されている
  - _Requirements: 3, 4_
  - _Boundary: 全ファイル（読み取りのみ）_

- [ ] 1.2 unwrap/expect/panicの全数調査 (P)
  - `crates/pasta_dsl/src/` 配下の全ファイルで `unwrap()`, `expect()`, `panic!()` をgrep検索する
  - ドキュメントコメント内（サンプルコード）と実行パスを区別する
  - parse_scene.rs:388 の `unwrap()` を含む実行パス上の使用箇所を特定する
  - 完了条件: 実行パス上のパニック可能箇所のリストが作成されている
  - _Requirements: 1_
  - _Boundary: 全ファイル（読み取りのみ）_

- [ ] 2. 入力検証の堅牢性強化
- [ ] 2.1 parse_scene.rsのunwrap排除
  - parse_scene.rs:388 の `raw[colon_pos..].chars().next().unwrap()` をmatch式またはif-let式に置換する
  - コロン位置が不正な場合にパニックではなくParseErrorを返すようにする
  - 完了条件: `cargo test -p pasta_dsl` が全パスし、grepで実行パス上のunwrapが0件
  - _Requirements: 1, 6_
  - _Boundary: parse_scene.rs_

- [ ] 2.2 partial.rsの防御的コーディング強化
  - 空文字列入力時の動作を検証し、必要に応じてガード条件を追加する
  - スコープ境界分割（split_by_scope_markers）の境界条件を検証する
  - infer_rule_from_line の入力バリデーションを確認する
  - 完了条件: 空入力・境界入力でパニックしないことをテストで確認
  - _Requirements: 1, 7_
  - _Boundary: partial.rs_

- [ ] 3. デッドコード除去
- [ ] 3.1 未使用importの除去 (P)
  - clippy警告で特定された未使用use宣言を全ファイルから除去する
  - 完了条件: `cargo clippy -p pasta_dsl` でunused_imports警告が0件
  - _Requirements: 3, 6_
  - _Boundary: 全ファイル_

- [ ] 3.2 pub可視性の最小化 (P)
  - クレート外部から参照されないpub関数・メソッド・型を `pub(crate)` または非pubに縮小する
  - pasta_lua、pasta_lsp からの参照を確認し、外部参照のある項目はpubを維持する
  - 完了条件: `cargo test --workspace` が全パスし、不要なpub項目が縮小されている
  - _Requirements: 3, 6_
  - _Boundary: 全ファイル_

- [ ] 3.3 到達不能コード・未使用関数の除去 (P)
  - clippy警告で特定されたdead_code、unreachable_patterns を除去する
  - AST型の未使用メソッドを特定し、外部参照がなければ除去またはpub(crate)化する
  - 完了条件: `cargo clippy -p pasta_dsl` でdead_code関連警告が0件
  - _Requirements: 3, 6_
  - _Boundary: 全ファイル_

- [ ] 4. 冗長表現の削減
- [ ] 4.1 parse_scene.rsの冗長パターン統合
  - 同一の属性マージロジックや変換パターンの重複を特定し、ファイル内ヘルパー関数に抽出する
  - 冗長なmatch分岐（同一処理の複数アーム）を統合する
  - 不要な中間変数やクロージャを簡素化する
  - 完了条件: 重複パターンが共通ヘルパーに統合され、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 4, 5, 6_
  - _Boundary: parse_scene.rs_

- [ ] 4.2 parse_action.rsの冗長パターン統合
  - 同一の変換パターンの重複を特定し統合する
  - 冗長なmatch分岐を統合する
  - 完了条件: 重複パターンが統合され、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 4, 5, 6_
  - _Boundary: parse_action.rs_

- [ ] 4.3 parse_elements.rs・AST型ファイルの冗長パターン統合 (P)
  - parse_elements.rs内の冗長パターンを統合する
  - ast/scene.rs、ast/action.rs、ast/cue.rs の冗長なimpl定義を統合する
  - ast/mod.rs の未使用re-exportを整理する
  - 完了条件: 冗長パターンが統合され、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 4, 6_
  - _Boundary: parse_elements.rs, ast/*_

- [ ] 5. パーサー複雑度の削減
- [ ] 5.1 parse_scene.rsの大規模関数分割
  - 50行を超える関数を特定し、論理的なサブ処理をファイル内ヘルパー関数として抽出する
  - 深いネスト（3段階以上のインデント）を早期リターンパターンで簡素化する
  - 完了条件: 主要関数が概ね50行以下になり、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 5, 6_
  - _Boundary: parse_scene.rs_

- [ ] 5.2 parse_action.rsの大規模関数分割
  - 50行を超える関数を特定し、ファイル内ヘルパー関数として抽出する
  - 深いネストを早期リターンパターンで簡素化する
  - 完了条件: 主要関数が概ね50行以下になり、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 5, 6_
  - _Boundary: parse_action.rs_

- [ ] 5.3 mod.rsの簡素化
  - build_file_ast等の大規模関数があれば分割する
  - 内部ヘルパー関数の整理と簡素化を行う
  - 完了条件: 関数が概ね50行以下に収まり、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 5, 6_
  - _Boundary: mod.rs_

- [ ] 6. 最終検証
- [ ] 6.1 全体回帰テストと静的解析
  - `cargo test --workspace` を実行し全テストパスを確認する
  - `cargo clippy -p pasta_dsl -- -D warnings` で警告ゼロを確認する
  - 実行パス上の `unwrap()`/`expect()`/`panic!()` がゼロであることをgrepで確認する
  - pub可視性が適切に縮小されていることを確認する
  - 完了条件: 全テストパス、clippy警告ゼロ、パニック箇所ゼロ
  - _Requirements: 1, 2, 3, 4, 5, 6, 7_

- [ ] 6.2 エラーハンドリング一貫性の最終確認
  - ParseErrorの全バリアントがDisplay/Debugを適切に実装していることを確認する
  - MultipleErrorsバリアントによるエラー集約が正しく動作することを確認する
  - 完了条件: エラー型の一貫性が検証済み
  - _Requirements: 2, 6_
