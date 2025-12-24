# Requirements Document

## Project Description (Input)
ランタイム層のAPI切り替えについて、パーサー２、トランスパイラー２を利用したバージョンに完全に切り替えます。

## Introduction
PastaEngineは旧parser/transpilerスタックから新parser2/transpiler2スタックへの移行を完了させる。対象は`src/engine.rs`におけるパース・マージ・トランスパイル経路であり、itemsベースASTと2-passトランスパイルを採用しつつランタイム層と既存テスト行動を維持する。

## Requirements

### Requirement 1: Parser2統合
**目的:** 開発者として、PastaEngineがparser2を通じてPastaスクリプトをパースし、旧parser依存を排除したい

#### Acceptance Criteria
1. When PastaEngineがPastaファイルをパースする時、the PastaEngine shall `parser2::parse_file`を呼び出して結果を取得する。
2. While パース結果を保持する時、the PastaEngine shall `parser2::PastaFile`のitemsベース構造を用いてASTを管理する。
3. If パースに失敗する時、the PastaEngine shall `parser2::PastaError`をパス情報付きで`PastaError`へ変換して報告する。
4. The PastaEngine shall 旧`crate::parser`経由のパースAPIをインポートまたは呼び出さない。

### Requirement 2: ASTマージのitems化
**目的:** 開発者として、複数ファイルのASTを新items構造で正しく統合し、構造的整合性を確保したい

#### Acceptance Criteria
1. When 複数のPastaFileを統合する時、the PastaEngine shall `FileItem`列挙を識別して1つの`items`ベクターへ順序を保ったまま集約する。
2. When FileItemを処理する時、the PastaEngine shall `FileAttr`・`GlobalWord`・`GlobalSceneScope`を区別して統合結果に保持する。
3. While 統合ASTを生成する時、the PastaEngine shall 元ファイルの`path`および`span`情報を維持した`PastaFile`を構築する。
4. If 旧`global_words`または`scenes`フィールドへの依存が残存する時、the PastaEngine shall itemsベースの参照に置き換える。

### Requirement 3: Transpiler2二段トランスパイル
**目的:** 開発者として、2-passトランスパイルでRuneコードを生成し、出力を一貫したバッファに蓄積したい

#### Acceptance Criteria
1. When トランスパイルを開始する時、the PastaEngine shall `SceneRegistry`と`WordDefRegistry`を生成し、単一の可変バッファを用意する。
2. When Pass1を実行する時、the PastaEngine shall `Transpiler2::transpile_pass1`を`PastaFile`・両レジストリ・出力バッファに対して呼び出す。
3. When Pass2を実行する時、the PastaEngine shall 同一バッファに追記する形で`Transpiler2::transpile_pass2`を呼び出す。
4. If Pass1またはPass2が失敗する時、the PastaEngine shall `TranspileError`を段階情報付きで`PastaError`として返す。
5. The PastaEngine shall `Transpiler::transpile_with_registry`を呼び出さない。

### Requirement 4: Registry統合とランタイム生成
**目的:** 開発者として、共有registryを通じてシーン・単語定義を管理し、ランタイムテーブルを一貫して生成したい

#### Acceptance Criteria
1. When レジストリを初期化する時、the PastaEngine shall `crate::registry::{SceneRegistry, WordDefRegistry}`を使用する。
2. When トランスパイルが完了する時、the PastaEngine shall `SceneTable`と`WordTable`を各レジストリから生成する。
3. While レジストリを参照する時、the PastaEngine shall シーン列挙に`SceneRegistry::all_scenes`を用いてランタイム入力を取得する。
4. If レジストリが未初期化のままPass2実行を試みる時、the PastaEngine shall エラーとして処理し不整合出力を防止する。

### Requirement 5: 後方互換性と動作維持
**目的:** ユーザーとして、移行後も既存スクリプトが同じIRと実行結果を得られるようにしたい

#### Acceptance Criteria
1. When 既存フィクスチャをロードする時、the PastaEngine shall 旧スタックと同等の`ScriptEvent`系列を生成する。
2. When Rune VMを実行する時、the PastaEngine shall 既存`ScriptGenerator`で同じシーン解決とランダム選択動作を提供する。
3. When シーン呼び出しや単語展開を行う時、the PastaEngine shall 前方一致選択の確率分布と挙動を維持する。
4. The PastaEngine shall ランタイム層およびstdlibのコード変更を伴わずに移行を完了する。

### Requirement 6: テストおよび品質ゲート
**目的:** 開発者として、移行後も既存テスト網で回帰を即座に検知し、611テスト合格状態を維持したい

#### Acceptance Criteria
1. When 移行後にテストスイートを実行する時、the PastaEngine shall 全611テスト(ignored除く)を合格する。
2. While engine.rsの変更をレビューする時、the PastaEngine shall `cargo test`を含む自動チェックでparser2/transpiler2統合を検証する。
3. If 任意のテストが失敗する時、the PastaEngine shall エラーをparser2/transpiler2コンテキスト付きで報告し再現手順を示す。
4. The PastaEngine shall 新規回帰テストを不要にする形で既存テストカバレッジを維持する。

### Requirement 7: ドキュメント更新
**目的:** プロジェクト管理者として、移行後のアーキテクチャを文書化し、新スタック採用を明確にしたい

#### Acceptance Criteria
1. When ドキュメントを更新する時、the PastaEngine shall parser2/transpiler2採用と2-pass戦略をREADMEやengineドキュメントに反映する。
2. When 旧parser/transpilerへの記述が残存する時、the PastaEngine shall 新スタックの記述へ置き換える。
3. The PastaEngine shall engine.rs内の旧parser/transpiler参照をすべて削除し、新スタックのみ使用することを示す。
