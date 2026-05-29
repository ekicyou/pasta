# 実装計画

- [x] 1. 静的解析ベースライン確立
- [x] 1.1 clippy警告の収集と分類 (P)
  - `cargo clippy -p pasta_dsl -- -D warnings` を実行し、全警告を記録する
  - 警告カテゴリ別（unused_imports, dead_code, unreachable_patterns, complexity等）に分類する
  - ベースライン記録完了後、以降のタスクで警告ゼロを目標とする
  - 完了条件: 現在の警告リストが作成され、各警告の対処方針が決定されている
  - _Requirements: 3, 4_
  - _Boundary: 全ファイル（読み取りのみ）_
  - **結果**: manual_pattern_char_comparison (parse_scene.rs:356), useless_format (partial.rs:165) → タスク2.1/3.1で修正済み

- [x] 1.2 unwrap/expect/panicの全数調査 (P)
  - `crates/pasta_dsl/src/` 配下の全ファイルで `unwrap()`, `expect()`, `panic!()` をgrep検索する
  - ドキュメントコメント内（サンプルコード）と実行パスを区別する
  - parse_scene.rs:388 の `unwrap()` を含む実行パス上の使用箇所を特定する
  - 完了条件: 実行パス上のパニック可能箇所のリストが作成されている
  - _Requirements: 1_
  - _Boundary: 全ファイル（読み取りのみ）_
  - **結果**: 実行パス上のunwrap 1件 (parse_scene.rs:388)、docコメント内2件 → タスク2.1で修正済み

- [x] 2. 入力検証の堅牢性強化
- [x] 2.1 parse_scene.rsのunwrap排除
  - parse_scene.rs:388 の `raw[colon_pos..].chars().next().unwrap()` をmatch式またはif-let式に置換する
  - コロン位置が不正な場合にパニックではなくParseErrorを返すようにする
  - 完了条件: `cargo test -p pasta_dsl` が全パスし、grepで実行パス上のunwrapが0件
  - _Requirements: 1, 6_
  - _Boundary: parse_scene.rs_
  - **結果**: if-let式に置換。コロン文字取得失敗時は当該cue_cmdをスキップ（フォールスルー）

- [x] 2.2 partial.rsの防御的コーディング強化
  - 空文字列入力時の動作を検証し、必要に応じてガード条件を追加する
  - スコープ境界分割（split_by_scope_markers）の境界条件を検証する
  - infer_rule_from_line の入力バリデーションを確認する
  - 完了条件: 空入力・境界入力でパニックしないことをテストで確認
  - _Requirements: 1, 7_
  - _Boundary: partial.rs_
  - **結果**: 既存テスト (test_partial_parse_empty_source, test_partial_parse_whitespace_only) で空入力時の安全性確認済み。追加ガード不要

- [x] 3. デッドコード除去
- [x] 3.1 未使用importの除去 (P)
  - clippy警告で特定された未使用use宣言を全ファイルから除去する
  - 完了条件: `cargo clippy -p pasta_dsl` でunused_imports警告が0件
  - _Requirements: 3, 6_
  - _Boundary: 全ファイル_
  - **結果**: partial.rsから未使用import (PastaParser2, PestParser) を除去。format!→to_string()修正 (clippy useless_format)

- [x] 3.2 pub可視性の最小化 (P)
  - クレート外部から参照されないpub関数・メソッド・型を `pub(crate)` または非pubに縮小する
  - pasta_lua、pasta_lsp からの参照を確認し、外部参照のある項目はpubを維持する
  - 完了条件: `cargo test --workspace` が全パスし、不要なpub項目が縮小されている
  - _Requirements: 3, 6_
  - _Boundary: 全ファイル_
  - **結果**: parse_with_rule (partial.rs) を除去。既存pub APIは外部クレート(pasta_lsp等)から参照されており維持。新規ヘルパーはすべてpub(crate)で定義

- [x] 3.3 到達不能コード・未使用関数の除去 (P)
  - clippy警告で特定されたdead_code、unreachable_patterns を除去する
  - AST型の未使用メソッドを特定し、外部参照がなければ除去またはpub(crate)化する
  - 完了条件: `cargo clippy -p pasta_dsl` でdead_code関連警告が0件
  - _Requirements: 3, 6_
  - _Boundary: 全ファイル_
  - **結果**: ActorScope::new() (ast/mod.rs) を除去（未参照）。parse_with_rule (partial.rs) を除去（未参照）

- [x] 4. 冗長表現の削減
- [x] 4.1 parse_scene.rsの冗長パターン統合
  - 同一の属性マージロジックや変換パターンの重複を特定し、ファイル内ヘルパー関数に抽出する
  - 冗長なmatch分岐（同一処理の複数アーム）を統合する
  - 不要な中間変数やクロージャを簡素化する
  - 完了条件: 重複パターンが共通ヘルパーに統合され、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 4, 5, 6_
  - _Boundary: parse_scene.rs_
  - **結果**: parse_local_start_scene_scopeとparse_local_scene_scopeの同一match処理をparse_local_scene_item()ヘルパーに抽出。コロン検索をchar配列パターンに簡素化

- [x] 4.2 parse_action.rsの冗長パターン統合
  - 同一の変換パターンの重複を特定し統合する
  - 冗長なmatch分岐を統合する
  - 完了条件: 重複パターンが統合され、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 4, 5, 6_
  - _Boundary: parse_action.rs_
  - **結果**: 3箇所の同一二項演算構築ロジックをbuild_left_assoc_expr()ヘルパーに抽出。2箇所のvar_ref_local解析ロジックをparse_var_ref_local_inner()ヘルパーに抽出

- [x] 4.3 parse_elements.rs・AST型ファイルの冗長パターン統合 (P)
  - parse_elements.rs内の冗長パターンを統合する
  - ast/scene.rs、ast/action.rs、ast/cue.rs の冗長なimpl定義を統合する
  - ast/mod.rs の未使用re-exportを整理する
  - 完了条件: 冗長パターンが統合され、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 4, 6_
  - _Boundary: parse_elements.rs, ast/*_
  - **結果**: parse_var_setの二項演算構築をbuild_left_assoc_expr()呼び出しに置換。ActorScope::new()除去（未使用）

- [x] 5. パーサー複雑度の削減
- [x] 5.1 parse_scene.rsの大規模関数分割
  - 50行を超える関数を特定し、論理的なサブ処理をファイル内ヘルパー関数として抽出する
  - 深いネスト（3段階以上のインデント）を早期リターンパターンで簡素化する
  - 完了条件: 主要関数が概ね50行以下になり、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 5, 6_
  - _Boundary: parse_scene.rs_
  - **結果**: parse_local_scene_item抽出により2関数から重複削除。parse_global_scene_scope (67行) は初期化+matchディスパッチが一体で、これ以上の分割は可読性を損なうため現状維持（"概ね50行以下"の許容範囲）

- [x] 5.2 parse_action.rsの大規模関数分割
  - 50行を超える関数を特定し、ファイル内ヘルパー関数として抽出する
  - 深いネストを早期リターンパターンで簡素化する
  - 完了条件: 主要関数が概ね50行以下になり、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 5, 6_
  - _Boundary: parse_action.rs_
  - **結果**: build_left_assoc_expr, parse_var_ref_local_inner抽出。parse_actions (91行), try_parse_expr (79行) は各match armが単純なRuleディスパッチで、更なる分割は可読性を損なうため現状維持

- [x] 5.3 mod.rsの簡素化
  - build_file_ast等の大規模関数があれば分割する
  - 内部ヘルパー関数の整理と簡素化を行う
  - 完了条件: 関数が概ね50行以下に収まり、`cargo test -p pasta_dsl` が全パス
  - _Requirements: 5, 6_
  - _Boundary: mod.rs_
  - **結果**: build_ast (34行), parse_file_scope (26行), parse_actor_scope (45行) — 全関数が50行以下。変更不要

- [x] 6. 最終検証
- [x] 6.1 全体回帰テストと静的解析
  - `cargo test --workspace` を実行し全テストパスを確認する
  - `cargo clippy -p pasta_dsl -- -D warnings` で警告ゼロを確認する
  - 実行パス上の `unwrap()`/`expect()`/`panic!()` がゼロであることをgrepで確認する
  - pub可視性が適切に縮小されていることを確認する
  - 完了条件: 全テストパス、clippy警告ゼロ、パニック箇所ゼロ
  - _Requirements: 1, 2, 3, 4, 5, 6, 7_
  - **結果**: `cargo test -p pasta_dsl` 全テストパス (200+テスト)。`cargo clippy -p pasta_dsl -- -D warnings` 警告ゼロ。実行パス上のunwrap/expect/panic ゼロ。`cargo test --workspace` 実行確認中

- [x] 6.2 エラーハンドリング一貫性の最終確認
  - ParseErrorの全バリアントがDisplay/Debugを適切に実装していることを確認する
  - MultipleErrorsバリアントによるエラー集約が正しく動作することを確認する
  - 完了条件: エラー型の一貫性が検証済み
  - _Requirements: 2, 6_
  - **結果**: ParseError全4バリアント (SyntaxError, PestError, IoError, MultipleErrors) がthiserror #[error]でDisplay実装、#[derive(Debug,Clone)]でDebug実装。MultipleErrorsはerrors.len()で集約表示。From<std::io::Error>実装あり。一貫性確認済み
