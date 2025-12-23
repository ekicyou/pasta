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

### 要件1: FileItem3バリアント構造によるファイルスコープ管理
**目的**: parser2が`(file_scope | global_scene_scope)*`文法を正確に実装し、複数のファイルレベルアイテム（属性、単語、シーン）を交互出現させながら、記述順序を完全に保持すること。

#### 受入基準
1. When `PastaFile`を構築する場合、parser2 shall `Vec<FileItem>`フィールドを使用し、FileAttr・GlobalWord・GlobalScopeScope の3つのバリアントを交互に任意回数格納できる
2. When パーサーが `file_scope` ブロックをマッチする場合、parser2 shall そのブロック内の `attrs` を `FileItem::FileAttr` として items に追加する
3. When パーサーが `file_scope` ブロックをマッチする場合、parser2 shall そのブロック内の `words` を `FileItem::GlobalWord` として items に追加する
4. When パーサーが `global_scene_scope` ブロックをマッチする場合、parser2 shall それを `FileItem::GlobalSceneScope` として items に追加する
5. When複数の`file_scope`ブロック間に`global_scene_scope`が挟まれる場合、parser2 shall その出現順序をitems配列の並序で正確に保持し、ファイル記述順序から復元可能にする
6. If `file_scope` 内の属性または単語が複数回定義される場合、parser2 shall 各定義を個別の `FileItem` として保持し、マージや上書きを行わない

### 要件2: FileItem列挙型とAST構造の導入
**目的**: grammar.pest準拠のAST構造を実装し、破壊的変更によって既存コードの修正を強制するメカニズムを提供する。

#### 受入基準
1. The parser2 shall `pub enum FileItem { FileAttr(Attr), GlobalWord(KeyWords), GlobalSceneScope(GlobalSceneScope) }` を定義する
2. The parser2 shall `PastaFile` 構造体に `items: Vec<FileItem>` フィールドを追加する
3. The parser2 shall 既存の `file_scope: FileScope` フィールドを廃止し、コンパイルエラーにより依存コードの修正を強制する
4. The parser2 shall 既存の `global_scenes: Vec<GlobalSceneScope>` フィールドを廃止する
5. The parser2 shall ヘルパーメソッド `file_attrs()` と `words()` と `global_scene_scopes()` を提供し、transpiler2での利便性を確保する

### 要件3: パーサーロジックの修正
**目的**: `src/parser2/mod.rs` の解析ループを修正し、複数出現するアイテムの累積処理を実現する。

#### 受入基準
1. When パーサーが `Rule::file_scope` をマッチする場合、parser2 shall 上書き代入を廃止し、parse_file_scope() で取得した属性・単語を個別の `FileItem` として `file.items.push()` する
2. When パーサーが `Rule::global_scene_scope` をマッチする場合、parser2 shall `file.items.push(FileItem::GlobalSceneScope(...))` を実行する
3. The parser2 shall ループ内で、`file_scope` と `global_scene_scope` が出現する順序を items 配列に反映する
4. The parser2 shall パーサーループ内で上書き代入操作を排除し、すべてのアイテムをpush操作で累積する

### 要件4: テストケースの追加と既存テストの移行
**目的**: 複数ファイルアイテム交互出現シナリオをカバーする統合テストを追加し、既存テストを新AST構造に対応させる。

#### 受入基準
1. The parser2 shall ファイルスコープと複数グローバルシーンが交互に3回以上出現するテストフィクスチャ（`comprehensive_control_flow2.pasta` など）を含む
2. When テストを実行する場合、parser2 shall `file.items` が正確に6個以上のFileItem（FileAttr・GlobalWord・GlobalSceneScope の混在）を格納していることを検証する
3. When テストを実行する場合、parser2 shall items の各インデックスで期待される順序（ファイル記述順）に従っていることを検証する
4. The parser2 shall `tests/parser2_integration_test.rs` 内の既存テスト6箇所を新 items ベースの検証に修正する
5. The parser2 shall パターンマッチまたはヘルパーメソッドを使用して、特定の `FileItem` バリアントを抽出可能にする

### 要件5: transpiler2互換性インターフェース
**目的**: transpiler2が `PastaFile.items` を順次処理し、ファイルレベルコンテキストを積算しながらシーンを処理できるAPIを提供する。

#### 受入基準
1. When transpiler2が `file.items` をイテレートする場合、transpiler2 shall パターンマッチで `FileItem::FileAttr`, `FileItem::GlobalWord`, `FileItem::GlobalSceneScope` を識別し処理できる
2. The parser2 shall 各 `FileItem` に Span 情報を含め、エラー報告やログにおいて発生箇所をトレース可能にする
3. When transpiler2 が items をスキャンする場合、transpiler2 shall 直前の `FileAttr` と `GlobalWord` をバッファリングし、次の `GlobalSceneScope` 到達時に属性コンテキストとして適用できる
4. The parser2 shall ヘルパーメソッド（`file_attrs()`, `words()`, `global_scene_scopes()`）を通じて、全アイテムを型別に取得可能にする

### 要件6: エラーハンドリングとドキュメント整備
**目的**: AST構造変更後も診断品質を維持し、破壊的変更の影響を文書化する。

#### 受入基準
1. The parser2 shall パース中にエラーが発生する場合、各 `FileItem` のSpan情報を使用してエラーメッセージに行番号・位置を含める
2. The parser2 shall `PastaFile` 構造体と `FileItem` 列挙型に対して、移行ガイドと使用例を docコメントに記載する
3. The parser2 shall `src/parser2/mod.rs` のモジュールコメントに「grammar.pest `( file_scope | global_scene_scope )*` 準拠」と明記する
4. The parser2 shall 既存transpiler（legacy parser使用）への影響がないことを確認し、その旨を仕様書に記載する

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
