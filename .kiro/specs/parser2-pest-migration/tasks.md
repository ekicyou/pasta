# Implementation Plan

## Task Format

- 全9要件を網羅するタスクリスト
- 各サブタスクは1〜3時間で完了可能なサイズ
- `(P)` は並列実行可能なタスクを示す

---

## Tasks

- [ ] 1. 文法ファイルの準備と移動
- [ ] 1.1 parser2ディレクトリ作成と文法ファイル移動
  - `src/parser2/`ディレクトリを作成する
  - pasta2.pestをgrammar.pestとして移動する（`git mv`で履歴保全）
  - 移動後、ファイル内容が完全に保全されていることを`git diff`で検証する
  - コミットメッセージにconventional commits形式と"no content changes"を明記する
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7_

- [ ] 2. AST型定義
- [ ] 2.1 (P) コアAST型の定義
  - PastaFile, FileScope, GlobalSceneScope, LocalSceneScope構造体を定義する
  - 3層スコープ階層（File ⊃ Global ⊃ Local）を型で表現する
  - Span型でソース位置情報を全ノードに付与する
  - Debug, Clone traitを導出する
  - _Requirements: 3.1, 3.3_

- [ ] 2.2 (P) シーンアイテムと式の型定義
  - LocalSceneItem列挙型（VarSet, CallScene, ActionLine, ContinueAction）を定義する
  - Action列挙型（Talk, WordRef, VarRef, FnCall, SakuraScript, Escape）を定義する
  - Expr列挙型（Integer, Float, String, VarRef, FnCall, Binary等）を定義する
  - 継続行（ContinueAction）を独立したAST型として実装する
  - _Requirements: 3.4, 3.5_

- [ ] 2.3 (P) 補助型の定義
  - VarSet, CallScene, Attr, KeyWords, Args, Arg, CodeBlock構造体を定義する
  - VarScope, FnScope, BinOp列挙型を定義する
  - AttrValue列挙型（Integer, Float, String, AttrString）を定義する
  - _Requirements: 3.1, 3.2_

- [ ] 2.4 AST型のユニットテスト作成
  - Span::from_pestの座標変換テストを作成する
  - FileScope::default()のテストを作成する
  - 全AST型のClone/Debug動作確認テストを作成する
  - _Requirements: 3.1_

- [ ] 3. パーサー実装
- [ ] 3.1 PastaParser構造体とPest統合
  - mod.rsを作成し、モジュールエントリーポイントとする
  - `#[derive(Parser)]`と`#[grammar = "parser2/grammar.pest"]`でPastaParser構造体を生成する
  - ast.rsからAST型を`pub use ast::*`で再エクスポートする
  - _Requirements: 4.1, 4.2, 6.1, 6.3, 6.4_

- [ ] 3.2 parse_str関数の実装
  - ソース文字列を受け取り、Pest parserでパースする
  - パース結果（Pairs<Rule>）からAST構築処理を呼び出す
  - file_scope、global_scene_scope、local_scene_scopeを順次パースする
  - PastaFileを返す
  - _Requirements: 2.2, 4.3_

- [ ] 3.3 全角数字正規化関数の実装
  - 全角数字（'０'..'９'）を半角に変換する関数を実装する
  - 全角マイナス（'－'）、全角小数点（'．'）も変換対象とする
  - 小数点の有無でInteger/Floatを判定する
  - 境界値（i64::MAX, i64::MIN）を正しく処理する
  - _Requirements: 3.1_

- [ ] 3.4 未名シーン継承ロジックの実装
  - last_global_scene_nameをNoneで初期化する
  - グローバルシーン登場時に名前を更新する
  - 未名シーン（global_scene_continue_line）登場時に直前のグローバルシーン名を継承する
  - ファイル先頭での未名シーンはParseErrorを返す
  - _Requirements: 2.5, 2.6_

- [ ] 3.5 parse_file関数の実装
  - ファイルパスを受け取り、内容を読み込む
  - parse_str関数に処理を委譲する
  - IOエラーをPastaError::IoErrorに変換する
  - _Requirements: 2.2_

- [ ] 3.6 エラーハンドリングの実装
  - PestエラーをPastaError::PestErrorでラップする
  - エラーメッセージにファイル名とソース位置を含める
  - IOエラーのFrom変換を実装する
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [ ] 4. lib.rs統合とレガシー共存
- [ ] 4.1 lib.rsへのparser2モジュール追加
  - `pub mod parser2;`を追加する（エイリアスや再エクスポートなし）
  - 既存のparser, transpiler, runtime等への影響がないことを確認する
  - `pasta::parser2::parse_file`, `pasta::parser2::parse_str`経由でアクセス可能にする
  - _Requirements: 2.1, 2.3, 2.4, 5.1, 5.2, 5.3, 5.4, 5.5_

- [ ] 5. テストカバレッジ
- [ ] 5.1 スコープ構造テストの作成
  - file_scope、global_scene_scope、local_scene_scopeの3層構造テストを作成する
  - 単純なグローバルシーンのパーステストを作成する
  - 継続行（continue_action_line）のテストを作成する
  - _Requirements: 8.2_

- [ ] 5.2 未名シーン継承テストの作成
  - 未名シーン（global_scene_continue_line）の名前継承テストを作成する
  - ファイル先頭での未名シーンエラーテストを作成する
  - 連続未名シーンのテストを作成する
  - _Requirements: 8.2_

- [ ] 5.3 文字列リテラルとコードブロックテストの作成
  - 4階層すべての括弧レベルで入れ子文字列リテラルをテストする（`「」`〜`「「「「」」」」`）
  - 言語識別子付きコードブロック（rune, rust等）のテストを作成する
  - _Requirements: 8.3, 8.5_

- [ ] 5.4 (P) 識別子と空白テストの作成
  - 予約IDパターン（`__name__`）の拒否テストを作成する
  - space_charsで定義された14種類のUnicode空白文字テストを作成する
  - Unicode識別子（XID_START, XID_CONTINUE）のテストを作成する
  - _Requirements: 8.4, 8.6_

- [ ] 5.5 (P) 数値リテラルテストの作成
  - 全角/半角数字の変換テストを作成する（計14ケース以上）
  - 全角マイナス、全角小数点の変換テストを作成する
  - 混在パターン（`３.１４`、`3．14`）のテストを作成する
  - 境界値（i64::MAX, i64::MIN）テストを作成する
  - _Requirements: 8.6.5_

- [ ] 5.6 fixtureファイルの作成
  - parser2専用fixtureディレクトリ（`tests/fixtures/parser2/`）を作成する
  - 各規則に対応するfixtureファイルを作成する
  - comprehensive_control_flow2.pastaを最終統合テストケースとして使用する
  - _Requirements: 8.1, 8.7, 8.8_

- [ ] 5.7 全規則カバレッジの検証
  - 65規則（Normal 39 + Atomic 26）すべてにテストケースがあることを確認する
  - テストファイル内コメントで規則名を明記する
  - `cargo test --all`が成功することを確認する
  - _Requirements: 8.1_

- [ ] 6. ドキュメント
- [ ] 6.1 (P) mod.rs docコメント追加
  - モジュールレベルのdocコメント（`//!`）を追加し、移行目的を説明する
  - grammar.pestを権威的仕様として参照する
  - parse_file, parse_str関数に使用例付きdocコメントを追加する
  - _Requirements: 9.1, 9.2, 9.3_

- [ ] 6.2 (P) ast.rs docコメント追加
  - 各AST型に目的と用途を説明するdocコメントを追加する
  - ContinueActionにpasta.pestとの仕様変更を明記する
  - _Requirements: 9.1_

- [ ] 6.3 README.md更新
  - 並行パーサーアーキテクチャを説明するセクションを追加する
  - parser vs parser2の使い分けを明記する
  - 将来の移行計画を記載する
  - _Requirements: 9.4_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1 (Grammar File Preservation) | 1.1 |
| 2 (parser2モジュール作成) | 3.1, 3.2, 3.4, 3.5, 4.1 |
| 3 (AST型定義) | 2.1, 2.2, 2.3, 2.4, 3.3 |
| 4 (Pest parser統合) | 3.1, 3.2 |
| 5 (レガシー共存) | 4.1 |
| 6 (Module Structure) | 3.1 |
| 7 (エラーハンドリング) | 3.6 |
| 8 (テストカバレッジ) | 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7 |
| 9 (Documentation) | 6.1, 6.2, 6.3 |

全9要件を20サブタスクで網羅
