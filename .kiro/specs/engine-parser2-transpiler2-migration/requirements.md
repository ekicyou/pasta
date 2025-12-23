# Requirements Document

## Project Description (Input)
ランタイム層のAPI切り替えについて、パーサー２、トランスパイラー２を利用したバージョンに完全に切り替えます。

## Introduction
PastaEngineは現在、旧parser/transpilerスタック(`src/parser`および`src/transpiler`)を使用していますが、新しいparser2/transpiler2スタック(`src/parser2`、`src/transpiler2`、`src/registry`)が完成しており、全611テストが成功しています。この移行により、PastaEngineは新しいアーキテクチャを活用し、モジュール間の独立性と保守性を向上させます。

移行スコープ:
- `src/engine.rs`: parser/transpilerからparser2/transpiler2へのAPI切り替え
- AST構造の変換: 旧PastaFile(flat構造)から新PastaFile(items構造)へ
- トランスパイル呼び出しの更新: 単一pass(`transpile_with_registry`)から2-pass(`transpile_pass1` + `transpile_pass2`)へ
- ランタイム層は変更不要(互換性維持)

## Requirements

### Requirement 1: Parser2への移行
**目的:** 開発者として、PastaEngineが新しいparser2モジュールを使用してPastaスクリプトをパースできるようにし、文法の一貫性と保守性を向上させたい

#### Acceptance Criteria
1. When PastaEngineが初期化される時、PastaEngineは`crate::parser2`をインポートするものとする
2. When Pastaファイルをパースする時、PastaEngineは`parser2::parse_file()`を呼び出すものとする
3. When パース結果を取得する時、PastaEngineは`parser2::PastaFile`構造体(items-based)を使用するものとする
4. When 旧parser APIへの参照が残っている時、PastaEngineはそれらをすべて削除するものとする
5. The PastaEngineは旧`parser::parse_file`を使用してはならない

### Requirement 2: Transpiler2への移行
**目的:** 開発者として、PastaEngineが新しいtranspiler2モジュールの2-pass戦略を使用してRuneコードを生成できるようにし、コード品質を向上させたい

#### Acceptance Criteria
1. When PastaEngineが初期化される時、PastaEngineは`crate::transpiler2::Transpiler2`をインポートするものとする
2. When トランスパイルを実行する時、PastaEngineは`Transpiler2::transpile_pass1()`と`transpile_pass2()`を順次呼び出すものとする
3. When Pass 1を実行する時、PastaEngineはシーン登録とモジュール生成を完了するものとする
4. When Pass 2を実行する時、PastaEngineはscene_selectorとpastaラッパーを生成するものとする
5. When 旧transpiler APIへの参照が残っている時、PastaEngineはそれらをすべて削除するものとする
6. The PastaEngineは旧`Transpiler::transpile_with_registry`を使用してはならない

### Requirement 3: AST構造変換
**目的:** 開発者として、PastaEngineが複数のPastaFileをマージする際に新しいitems-based構造を使用できるようにし、データ統合の一貫性を確保したい

#### Acceptance Criteria
1. When 複数のPastaFileをマージする時、PastaEngineは`parser2::PastaFile::items`からFileItemを抽出するものとする
2. When FileItemを処理する時、PastaEngineは`FileAttr`、`GlobalWord`、`GlobalSceneScope`の各バリアントを識別するものとする
3. When マージされたASTを構築する時、PastaEngineは新しいPastaFileのitemsベクターを作成するものとする
4. When 旧AST構造(global_words、scenes)への参照が残っている時、PastaEngineはそれらをitems-based構造に置き換えるものとする
5. The PastaEngineは旧PastaFileのflat構造(global_words、scenesフィールド)を直接参照してはならない

### Requirement 4: Registry統合
**目的:** 開発者として、PastaEngineが共有registryモジュールを使用してシーンと単語定義を管理できるようにし、transpiler間の一貫性を確保したい

#### Acceptance Criteria
1. When レジストリを初期化する時、PastaEngineは`crate::registry::SceneRegistry`と`WordDefRegistry`を使用するものとする
2. When トランスパイル時にレジストリを渡す時、PastaEngineは両方のレジストリへの可変参照を提供するものとする
3. When レジストリからシーン情報を取得する時、PastaEngineは`SceneRegistry::all_scenes()`を使用するものとする
4. When 旧transpiler内部レジストリへの参照が残っている時、PastaEngineはそれらを共有registryへの参照に置き換えるものとする
5. The PastaEngineは`crate::transpiler::{SceneRegistry, WordDefRegistry}`ではなく`crate::registry::{SceneRegistry, WordDefRegistry}`をインポートするものとする

### Requirement 5: ランタイム互換性の維持
**目的:** 開発者として、新しいparser2/transpiler2への移行後もランタイム層が変更なく動作することを確認し、リスクを最小化したい

#### Acceptance Criteria
1. When Runeコードを生成した後、PastaEngineは既存の`runtime::SceneTable`と`WordTable`を使用するものとする
2. When Rune VMを実行する時、PastaEngineは既存の`ScriptGenerator`を使用するものとする
3. When pasta_stdlib関数を呼び出す時、PastaEngineは既存のstdlib APIを使用するものとする
4. When ScriptEventを出力する時、PastaEngineは既存のIR型を使用するものとする
5. The PastaEngineはランタイム層(`src/runtime/`、`src/stdlib/`)のコードを変更してはならない

### Requirement 6: 後方互換性の確保
**目的:** ユーザーとして、既存のPastaスクリプトがparser2/transpiler2への移行後も同じ動作をすることを保証し、移行の透明性を確保したい

#### Acceptance Criteria
1. When 既存の統合テストを実行する時、PastaEngineはすべてのテストに合格するものとする
2. When 既存のフィクスチャファイルをロードする時、PastaEngineは同じIR出力を生成するものとする
3. When シーン呼び出しを実行する時、PastaEngineは同じシーン解決動作を提供するものとする
4. When 単語参照を展開する時、PastaEngineは同じランダム選択動作を提供するものとする
5. The PastaEngineは既存のengine統合テスト(`tests/pasta_integration_engine_test.rs`等)を破壊してはならない

### Requirement 7: テストカバレッジの維持
**目的:** 開発者として、parser2/transpiler2への移行が完全にテストされていることを確認し、リグレッションを防止したい

#### Acceptance Criteria
1. When engine.rsを変更した後、PastaEngineは既存の全611テスト(3 ignored除く)を合格するものとする
2. When 新しいparser2/transpiler2統合を追加する時、PastaEngineは既存のengineテストスイートで検証されるものとする
3. If テストが失敗する時、PastaEngineは失敗理由を明確なエラーメッセージで報告するものとする
4. When 移行が完了した後、PastaEngineは新規のリグレッションテストを0件に保つものとする
5. The PastaEngineは移行前と同じテストカバレッジレベル(611 passed)を維持するものとする

### Requirement 8: 段階的移行の実施
**目的:** 開発者として、parser2/transpiler2への移行を段階的に実施し、各ステップで検証することで安全性を確保したい

#### Acceptance Criteria
1. When 移行を開始する時、PastaEngineはまずimport文を更新するものとする
2. When import更新後、PastaEngineはコンパイルエラーを修正するものとする
3. When AST変換ロジックを実装する時、PastaEngineは単一の関数またはメソッドに変更を局所化するものとする
4. When 各変更ステップ後、PastaEngineは`cargo test`を実行して回帰を検出するものとする
5. The PastaEngineは一度にすべての変更を適用せず、検証可能な小さなステップで移行するものとする

### Requirement 9: ドキュメントの更新
**目的:** 開発者として、PastaEngineのドキュメントがparser2/transpiler2アーキテクチャを正確に反映することで、将来のメンテナンスを容易にしたい

#### Acceptance Criteria
1. When engine.rsのドキュメントコメントを更新する時、PastaEngineは使用するparser2/transpiler2モジュールを明記するものとする
2. When アーキテクチャ図を更新する時、PastaEngineは新しい2-pass戦略を反映するものとする
3. When README.mdを更新する時、PastaEngineは最新のモジュール構成を記載するものとする
4. When 旧parserへの参照がドキュメントに残っている時、PastaEngineはそれらをparser2への参照に更新するものとする
5. The PastaEngineのドキュメントは旧parser/transpilerアーキテクチャへの参照を含んではならない

### Requirement 10: 旧モジュールの非推奨化計画
**目的:** プロジェクト管理者として、旧parser/transpilerモジュールの将来的な削除計画を明確にし、技術的負債を管理したい

#### Acceptance Criteria
1. When 移行が完了した後、PastaEngineは旧parserモジュールへのアクティブな参照を0件にするものとする
2. When 移行が完了した後、PastaEngineは旧transpilerモジュールへのアクティブな参照を0件にするものとする
3. When 非推奨化戦略を文書化する時、PastaEngineはREADMEまたはCHANGELOGに計画を記載するものとする
4. If 旧モジュールを並行保持する場合、PastaEngineは明確な非推奨マーカー(`#[deprecated]`)を追加するものとする
5. The PastaEngineは移行完了後、旧parser/transpilerモジュールの削除タイムラインを提示するものとする
