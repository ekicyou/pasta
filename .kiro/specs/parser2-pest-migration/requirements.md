# Requirements Document

## Project Description (Input)
pasta2.pestに基づいた実装を行う。pasta2.pestを憲法とし、新たなパーサー層を構築せよ。現在の実装（mod parser）はまだ削除せず、置き換えの準備として（mod parser2）を作る方向で進めよ。pasta2.pestは「parser2」ディレクトリに移動・リネームしてよいが、内容の書き換えは行わないこと。

## Introduction
本仕様は、既存の`src/parser/pasta.pest`ではなく`pasta2.pest`を権威的文法定義として採用し、新たなパーサー層を構築するための要件を定義します。レガシーコード（`mod parser`）を保持しつつ、並行して新実装（`mod parser2`）を作成し、段階的な移行を可能にします。

## Requirements

### Requirement 1: Grammar File Preservation and History Tracking
**Objective:** 開発者として、pasta2.pestファイルを移動してもその内容を**絶対に変更しない**こと、かつファイル履歴を保全することを保証したい。pasta2.pestは既に検証済みの権威的文法定義であり、一切の変更を認めない。

#### Acceptance Criteria
1. When pasta2.pestを`src/parser2/grammar.pest`に移動する、the Parser2移行プロセス shall ファイル内容を一切変更せずに保全する（空白・コメント・フォーマットも含めて完全にそのまま）
2. The Parser2モジュール shall grammar.pestを**不変の**構文規則の唯一の真実として扱う
3. The Parser2実装 shall オリジナルのpasta2.pest仕様から逸脱するgrammar.pestへの手動編集を拒否する
4. When grammar.pestが作成される、the ファイル shall オリジナルのpasta2.pestとバイト単位で同一である（`git diff`またはチェックサムで検証可能）
5. When pasta2.pestを移動する、the 移行プロセス shall ファイル履歴を保全するために`git mv`コマンドを使用する
6. The gitコミットメッセージ shall conventional commitsフォーマットに従う：`refactor(parser2): Move pasta2.pest to parser2/grammar.pest`
7. The コミットメッセージ shall 操作を明確にするために"no content changes"を明示的に記述する

### Requirement 2: 新しいパーサーモジュール（parser2）の作成
**Objective:** 開発者として、既存parserとは独立した新しいparser2モジュールを作成したい。これにより、段階的移行とリグレッションリスク軽減を実現できる。

#### Acceptance Criteria
1. The Pastaプロジェクト shall 独立した名前空間を持つ新しいモジュール`src/parser2/`を作成する
2. The Parser2モジュール shall レガシーparserと同じ命名の公開API関数を公開する：`parse_file`, `parse_str`（モジュールパス`pasta::parser2::parse_str`経由で名前空間分離）
3. When lib.rsが公開APIを公開する、the Pastaクレート shall `parser2`モジュールをpublicとして公開する（`pub mod parser2;`）、`pasta::parser2::*`経由で使用可能にする
4. The Parser2モジュール shall 完全な独立性を保証するため、レガシーparserモジュールとAST型定義を共有しない
5. When グローバルシーン名が省略される（`＊`または`*`のみ）、the Parser2 shall 直前の名前付きグローバルシーン名を継承する（レガシーparser実装と同様）
6. When ファイル先頭で名前なしグローバルシーンが出現する、the Parser2 shall ParseErrorを返す（"Unnamed global scene at start of file. A named global scene must appear before any unnamed scenes."）

### Requirement 3: pasta2.pest文法に基づくAST型定義
**Objective:** 開発者として、**検証済み**のpasta2.pest文法規則を**すべて**正確に反映したAST型を定義したい。これにより、文法と実装の完全な一貫性を保証できる。

#### Acceptance Criteria
1. The Parser2 ASTモジュール shall grammar.pest内の**すべての**終端・非終端規則に対応するRust構造体を定義する（pasta2.pestは既に検証済みであり、文法の妥当性は保証されている）
2. The Parser2 AST型 shall grammar.pestで定義されたUnicode識別子（XID_START, XID_CONTINUE）と予約IDパターン（`__name__`）の検証をサポートする
3. The Parser2 AST型 shall 階層的スコープ構造を表現する：`FileScope` → `GlobalSceneScope` → `LocalSceneScope`
4. The Parser2 AST型 shall 言語識別子付きコードブロック（例：` ```rune ... ``` `）をサポートする

### Requirement 4: Pest parser生成の統合
**Objective:** 開発者として、grammar.pestからPest parserを生成し、Rustコードに統合したい。これにより、型安全なパース処理を実現できる。

#### Acceptance Criteria
1. The Parser2モジュール shall pest_derive用に`#[grammar = "parser2/grammar.pest"]`ディレクティブを使用する（src/ディレクトリからの相対パス）
2. The Parser2モジュール shall `#[derive(Parser)]`マクロを使用して`PastaParser2`構造体を生成する
3. The Parser2 shall `PastaParser2::parse(Rule::file, source)`を使用して有効なPastaスクリプトのパースに成功する

### Requirement 5: レガシーparserとの共存
**Objective:** 開発者として、既存のmod parserを削除せずに稼働させたい。これにより、新旧パーサーの比較テストとリスク管理を可能にする。

#### Acceptance Criteria
1. The Pastaプロジェクト shall `src/parser/`と`src/parser2/`の両モジュールを同時に維持する
2. When lib.rsがインポートを宣言する、the Pastaクレート shall 異なるインポートパスを提供する：`pasta::parser`と`pasta::parser2`
3. The 既存のテストスイート shall 変更なしで`pasta::parser`を使い続ける
4. The Parser2モジュール shall レガシーparserモジュールとのコンパイルエラーやランタイム競合を引き起こさない

### Requirement 6: Module Structure
**Objective:** 開発者として、parser2モジュールを標準的なRustモジュール構成で実装したい。これにより、保守性と拡張性を確保できる。

#### Acceptance Criteria
1. The Parser2モジュール shall モジュールエントリーポイントとして`mod.rs`ファイルを定義する
2. The Parser2モジュール shall AST型定義用の`ast.rs`ファイルを定義する
3. The Parser2モジュール shall Pest文法仕様として`grammar.pest`ファイルを定義する
4. When `mod.rs`が公開APIを公開する、the Parser2モジュール shall `pub use ast::*`を使用してAST型を再公開する

### Requirement 7: エラーハンドリング統合
**Objective:** 開発者として、parser2のエラーを既存のPastaError型で扱いたい。これにより、統一的なエラー処理を維持できる。

#### Acceptance Criteria
1. The Parser2モジュール shall すべてのパース操作で`Result<T, PastaError>`を返す
2. When Pestパースエラーが発生する、the Parser2 shall それらを`PastaError::PestError(String)`バリアントでラップする（既存parser実装と同様）
3. When IOエラーが発生する、the Parser2 shall `From<std::io::Error>`トレイトを使用して自動変換する
4. The Parser2エラーメッセージ shall ファイル名とソース位置のコンテキストを含む（format!("Parse error in {}: {}", filename, e)形式）

### Requirement 8: 完全なテストカバレッジ
**Objective:** 開発者として、pasta2.pest文法の**すべての機能**を検証するテストを用意したい。これにより、実装の完全性を保証できる。

#### Acceptance Criteria
1. The Pastaプロジェクト shall grammar.pestで定義された**すべての**文法規則（約60規則）をカバーするテストファイルを作成する（式展開を含む全規則を網羅し、将来のtranspiler2実装の基盤とする）
2. The テストスイート shall すべてのスコープ構造を検証する：file_scope、global_scene_scope、local_scene_scope
3. The テストスイート shall 4階層すべての括弧レベルで入れ子文字列リテラルを検証する（`「text」`、`「「text」」`、`「「「text」」」`、`「「「「text」」」」`）
4. The テストスイート shall 予約IDパターンの拒否を検証する（`__name__` shall パース失敗する）
5. The テストスイート shall 言語識別子付きコードブロックを検証する（例：` ```rune ... ``` `、` ```rust ... ``` `）
6. The テストスイート shall `space_chars`で定義された14種類のUnicode空白文字すべてを検証する
7. The テストスイート shall `tests/fixtures/`ディレクトリのfixtureを使用し、parser2固有機能用の新しい包括的fixtureを作成する
8. The テストスイート shall すべての文法規則についてparser2がpest_consumeデバッグ出力と同一の結果を生成することを検証する

### Requirement 9: Documentation
**Objective:** 開発者として、parser2モジュールの目的と使用方法を文書化したい。これにより、将来の開発者が意図を理解し、適切にメンテナンスできる。

#### Acceptance Criteria
1. The Parser2の`mod.rs` shall 移行目的を説明するモジュールレベルのdocコメント（`//!`）を含む
2. The Parser2のdocコメント shall grammar.pestを権威的仕様として参照する
3. The Parser2の公開API関数 shall 使用例を含むdocコメントを含む
4. When README.mdが更新される、the Pastaプロジェクト shall 並行パーサーアーキテクチャを文書化する（parser vs parser2の使い分け、移行計画）


