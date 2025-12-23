# 要件定義: parser2 FileScope複数出現バグ修正

## イントロダクション

### 背景
parser2の現在の実装は、Pest文法定義 `file = ( file_scope | global_scene_scope )*` に違反しており、複数の`file_scope`ブロックが出現した場合、最後のブロックのみを保持し、それ以前のブロックを消失させるクリティカルなバグが存在する。このバグは、ファイルレベルの属性継承と単語定義の両方に影響を与え、transpiler2-layer-implementationの実装を阻害している。

### スコープ
本仕様は、parser2のAST構造とパーサーロジックを修正し、grammar.pestの仕様に完全準拠させることを目的とする。具体的には、複数の`file_scope`と`global_scene_scope`の任意順序・複数回出現を正しく処理し、ファイル記述順序を保持したデータ構造を提供する。

### 想定する利用者
- **transpiler2開発者**: ファイルレベル属性継承とシーンコンテキスト解決を実装
- **Pastaスクリプト作成者**: 中規模以上のスクリプトで複数の`file_scope`を使用
- **メンテナー**: parser2の文法準拠性を検証・保守

---

## 要件

### 要件1: 複数FileScope保持
**目的**: Pastaスクリプト作成者として、ファイル内で複数の`file_scope`ブロックを記述できるようにし、すべてのブロックが消失せずに保持されることを保証する。

#### 受入基準
1. When Pastaファイルに複数の`file_scope`ブロックが記述された場合、parser2 shall すべての`file_scope`ブロックをAST構造内に保持する
2. When `file_scope`ブロックが2回以上出現する場合、parser2 shall 最初のブロックから最後のブロックまで、ファイル記述順序を保持する
3. If 後続の`file_scope`ブロックが先行ブロックと同じ属性名を定義する場合、parser2 shall 両方のブロックを個別に保持し、マージや上書きを行わない
4. The parser2 shall ファイル中の任意の位置（先頭、中間、末尾）に出現する`file_scope`を等しく扱う

### 要件2: FileScope/GlobalSceneScope交互出現対応
**目的**: 開発者として、grammar.pestの `( file_scope | global_scene_scope )*` 仕様に完全準拠し、`file_scope`と`global_scene_scope`の交互出現順序を正確に保持する。

#### 受入基準
1. When `file_scope`と`global_scene_scope`が交互に出現する場合、parser2 shall ファイル記述順序を保持したデータ構造を生成する
2. When `file_scope`がグローバルシーンの間に挟まれている場合、parser2 shall その`file_scope`の位置情報を正確に保持する
3. The parser2 shall `file_scope`と`global_scene_scope`の出現順序をAST構造から復元可能にする
4. The parser2 shall 順序保持のため、`PastaFile`に統一的なアイテムリスト（`items: Vec<FileItem>`）を提供する

### 要件3: ファイルレベル属性の分離保持
**目的**: transpiler2開発者として、各`file_scope`ブロックの属性を個別に取得し、シーンごとの属性コンテキストを正確に解決できるようにする。

#### 受入基準
1. When 1つ目の`file_scope`に `＆season：winter` が定義され、2つ目の`file_scope`に `＆season：summer` が定義される場合、parser2 shall 両方の属性定義を個別の`FileScope`インスタンスとして保持する
2. When `file_scope`間に`global_scene_scope`が挟まれる場合、parser2 shall 各`file_scope`がどのグローバルシーンの前に位置するかを判別可能な情報を提供する
3. The parser2 shall 各`FileScope`インスタンスに、属性リスト（`attrs: Vec<Attr>`）を保持する
4. The parser2 shall `FileScope`インスタンスにSpan情報を含め、ソースコード位置をトレース可能にする

### 要件4: ファイルレベル単語定義の累積保持
**目的**: Pastaスクリプト作成者として、ファイル途中で単語を定義し、後続のシーンで使用できることを保証する。

#### 受入基準
1. When 複数の`file_scope`にそれぞれ異なる単語定義（`＠word1`, `＠word2`）が記述される場合、parser2 shall すべての単語定義を個別の`FileScope`インスタンス内に保持する
2. The parser2 shall 単語定義の消失を防ぎ、すべての`＠`単語定義を各`FileScope`の`words`フィールドに格納する
3. The parser2 shall 単語定義の出現順序を保持し、transpiler側での順次処理を可能にする
4. If ファイル内に3つの`file_scope`が存在し、それぞれ異なる単語を定義する場合、parser2 shall 3つの個別の`FileScope`インスタンスとして保持する

### 要件5: AST構造の破壊的変更
**目的**: メンテナーとして、grammar.pest準拠のためにAST構造を変更し、既存コードへの影響を最小化する移行パスを提供する。

#### 受入基準
1. The parser2 shall `PastaFile`構造体に`items: Vec<FileItem>`フィールドを導入する
2. The parser2 shall `enum FileItem { FileScope(FileScope), GlobalSceneScope(GlobalSceneScope) }`を定義する
3. When 既存の`file_scope: FileScope`フィールドが廃止される場合、parser2 shall コンパイルエラーを発生させ、依存コードの修正を強制する
4. The parser2 shall `PastaFile`に`file_scope`フィールドの代わりに`items`フィールドを使用するヘルパーメソッド（`file_scopes()`, `global_scenes()`）を提供する
5. While AST構造を変更する期間、parser2 shall すべてのユニットテストと統合テストを更新し、リグレッションを防止する

### 要件6: パーサーロジックの修正
**目的**: 開発者として、`src/parser2/mod.rs`のパーサーループを修正し、`file_scope`の上書き代入を廃止する。

#### 受入基準
1. When パーサーが`Rule::file_scope`をマッチする場合、parser2 shall `file.items.push(FileItem::FileScope(parse_file_scope(pair)?))` を実行する
2. When パーサーが`Rule::global_scene_scope`をマッチする場合、parser2 shall `file.items.push(FileItem::GlobalSceneScope(...))` を実行する
3. If パーサーループ内で`file.file_scope = ...`のような上書き代入が検出される場合、parser2 shall コンパイルエラーを発生させる（フィールド削除により）
4. The parser2 shall ループ内で`items`ベクターに順次push操作を行い、ファイル記述順序を保持する

### 要件7: テストケースの追加
**目的**: 品質保証担当者として、複数`file_scope`シナリオをカバーする回帰テストを追加し、将来のバグ再発を防止する。

#### 受入基準
1. The parser2 shall 複数`file_scope`属性定義シナリオのテストケースを含む
2. The parser2 shall `file_scope`と`global_scene_scope`の交互出現シナリオのテストケースを含む
3. The parser2 shall 単語定義が複数`file_scope`に分散されるシナリオのテストケースを含む
4. When テストケースを実行する場合、parser2 shall すべての`file_scope`が順序通りに保持されることを検証する
5. When テストケースを実行する場合、parser2 shall 1つ目の`file_scope`と2つ目の`file_scope`の属性が個別に取得可能であることを検証する

### 要件8: transpiler2互換性の確保
**目的**: transpiler2開発者として、修正後のAST構造からファイルレベル属性とシーンコンテキストを正確に抽出できることを保証する。

#### 受入基準
1. When transpiler2が`PastaFile.items`を順次処理する場合、transpiler2 shall 各`FileScope`の属性を累積的にマージできる
2. When transpiler2が`GlobalSceneScope`に到達する場合、transpiler2 shall 直前の`FileScope`群から累積されたファイルレベル属性を取得できる
3. The parser2 shall `FileItem`列挙型からパターンマッチで`FileScope`と`GlobalSceneScope`を識別可能にする
4. The parser2 shall transpiler2が必要とするSpan情報を各`FileItem`に含める
5. When parser2がAST構造を変更する場合、parser2 shall transpiler2の既存実装（存在する場合）への影響を文書化する

### 要件9: エラーハンドリングの保持
**目的**: 開発者として、AST構造変更後もエラーメッセージの品質と詳細度を維持する。

#### 受入基準
1. When パース中にエラーが発生する場合、parser2 shall 各`FileScope`または`GlobalSceneScope`のSpan情報を含むエラーメッセージを生成する
2. If 無効な`file_scope`ブロックが検出される場合、parser2 shall そのブロックの行番号と位置を含むエラーを返す
3. The parser2 shall AST構造変更後も、既存のエラーハンドリング機構（`PastaError`）を使用する
4. The parser2 shall エラー報告において、複数の`file_scope`が存在する場合でも、エラー発生箇所を一意に特定可能にする

### 要件10: ドキュメント更新
**目的**: メンテナーとして、AST構造変更の影響範囲と移行ガイドを文書化し、将来の開発者を支援する。

#### 受入基準
1. The parser2 shall `PastaFile`構造体の変更内容をdocコメントに記載する
2. The parser2 shall `FileItem`列挙型の使用例をdocコメントまたは統合テストに含める
3. When 破壊的変更が導入される場合、parser2 shall CHANGELOG.mdまたは該当仕様の`design.md`に移行ガイドを記載する
4. The parser2 shall grammar.pest仕様との対応関係を`src/parser2/mod.rs`のモジュールコメントに明記する

---

## 制約条件

### 技術制約
- **Pest文法準拠**: `file = ( file_scope | global_scene_scope )*` 仕様に完全準拠すること
- **Rust所有権**: `FileItem`列挙型は`FileScope`と`GlobalSceneScope`を所有すること
- **後方互換性**: `PastaFile`のパブリックAPIを変更する場合、セマンティックバージョニング（メジャーバージョンアップ）を適用すること

### 非機能制約
- **パフォーマンス**: 複数`file_scope`の処理時間は、単一`file_scope`と比較して線形増加を超えないこと
- **メモリ効率**: `Vec<FileItem>`のメモリ使用量は、file_scopeとglobal_scene_scopeの合計数に比例すること
- **テストカバレッジ**: 新規追加コードのテストカバレッジは80%以上とすること

---

## 成功基準

### 機能検証
1. ✅ 複数`file_scope`を含むPastaファイルを正常にパース可能
2. ✅ すべての`file_scope`ブロックがAST構造内に順序保持されて格納
3. ✅ transpiler2が各`file_scope`の属性と単語定義を個別に取得可能
4. ✅ 既存のparser2ユニットテストがすべて合格（リグレッション0件）

### 品質検証
1. ✅ 新規テストケース（複数`file_scope`シナリオ）を3件以上追加
2. ✅ `cargo test --package pasta --lib parser2` がすべて合格
3. ✅ AST構造変更が文書化され、移行ガイドが提供される

### 依存仕様の解除
1. ✅ transpiler2-layer-implementation仕様のブロッキング状態が解除
2. ✅ transpiler2の要件11（FileScope Attribute Inheritance）が実装可能な状態

---

## 参考資料

### 関連仕様
- **SPECIFICATION.md**: Pasta DSL言語仕様（第5章: File Scope）
- **grammar.pest**: Pest文法定義（`file`ルール）
- **transpiler2-layer-implementation**: 依存先仕様（要件11、15がブロック中）

### 関連ファイル
- `src/parser2/mod.rs`: パーサーロジック実装
- `src/parser2/ast.rs`: AST構造定義
- `tests/pasta_parser2_*.rs`: parser2統合テスト群

### バグレポート
- `.kiro/specs/parser2-filescope-bug-fix/bug-report.md`: 詳細バグ分析と再現ケース
