# Requirements Document

## Project Description (Input)
pastaエンジンは複数同時に初期化して同時に実行しても、それぞれ独立して動作するか。関連するテストは整備されているか。

## Introduction

本要件定義では、pastaスクリプトエンジンの複数インスタンス同時実行における独立性と、それを保証するテスト体制を定義します。pastaエンジンは、Pasta DSLスクリプトを解析・実行し、会話制御イベントを生成するコンポーネントです。複数のエンジンインスタンスが同一プロセス内で同時に動作する場合、各インスタンスが完全に独立して動作し、相互に干渉しないことが求められます。

対象範囲：
- `PastaEngine`インスタンスの完全な独立性（変数スコープ、ラベルテーブル、実行状態、キャッシュ）
- インスタンス間でのグローバル状態の共有禁止
- 並行実行時の動作保証
- テストカバレッジと自動化テスト基盤

設計原則：
- **エンジン内封じ込め**: すべての状態とキャッシュはエンジンインスタンス内に保持
- **グローバル状態ゼロ**: プロセス全体で共有される`static`変数を一切持たない（定数を除く）
- **完全な所有**: `Arc`などの共有ポインタは不要、エンジンが全データを所有
- **純粋関数的実装**: パース・トランスパイル処理は副作用を持たない純粋関数
- **完全な独立性**: あるエンジンの動作が他のエンジンに影響を与えない

## Requirements

### Requirement 1: インスタンス完全独立性の保証

**Objective:** As a pasta library user, I want 複数のPastaEngineインスタンスを同時に使用したい, so that 各インスタンスが他のインスタンスの状態に一切影響を与えず完全に独立して動作できる

#### Acceptance Criteria

1. When 複数の`PastaEngine`インスタンスが同一プロセス内で作成される, the pastaシステム shall 各インスタンスが全データを所有し（Rune Unit、RuntimeContext、ラベルテーブル、変数マネージャー、キャッシュ）、`Arc`や参照カウントによる共有を行わない
2. When あるエンジンインスタンスがラベルを実行する, the pastaシステム shall その実行状態（変数値、スピーカー、実行位置）を他のインスタンスと共有しない
3. When 2つの異なるエンジンインスタンスが同じ名前のグローバル変数（`@*変数名`）を設定する, the pastaシステム shall 各インスタンス内で独立した変数空間を保持し、相互に干渉しない
4. The pastaシステム shall 各`PastaEngine`インスタンスが独自の`RandomSelector`実装を保持し、乱数生成が独立している
5. The pastaシステム shall `static`変数（`static mut`、`OnceLock`、`LazyLock`、グローバルキャッシュなど）を一切使用しない（定数`const`を除く）

### Requirement 2: エンジン内部キャッシュの独立性

**Objective:** As a pasta library developer, I want パースキャッシュが各エンジンインスタンス内に閉じている, so that エンジン間での状態共有を完全に排除し、独立性を保証できる

#### Acceptance Criteria

1. The pastaシステム shall パースキャッシュ（AST、トランスパイル結果）を各`PastaEngine`インスタンスのフィールドとして保持し、所有権によって管理する
2. When あるエンジンインスタンスがスクリプトをパース・トランスパイルする, the pastaシステム shall その結果を自身のインスタンス内キャッシュにのみ保存し、`Arc`などで共有しない
3. When 複数のエンジンインスタンスが同一スクリプトをパースする, the pastaシステム shall 各インスタンスが独立してパース・トランスパイルを実行し、結果を各自が完全に所有する
4. The pastaシステム shall パース・トランスパイル関数を純粋関数として実装し、グローバル状態への依存や副作用を持たない
5. When エンジンインスタンスが破棄される, the pastaシステム shall 所有権システムにより自動的にすべてのデータ（キャッシュ含む）を解放する

### Requirement 3: 並行実行時の動作保証（補助的要件）

**Objective:** As a pasta library user, I want 複数のPastaEngineインスタンスを異なるスレッドで実行できる, so that マルチスレッド環境でも安全に使用できる

**Note**: 主要要件はReq 1-2（インスタンス独立性）。本要件はグローバル状態不在の副次的効果として、マルチスレッド安全性が構造的に保証されることを確認するもの。

#### Acceptance Criteria

1. When 2つの`PastaEngine`インスタンスが異なるスレッドで同時に`execute_label`を呼び出す, the pastaシステム shall 各スレッドで独立したVMを実行し、グローバル状態を持たないため互いに干渉しない
2. When 複数スレッドがそれぞれ`PastaEngine`を作成する, the pastaシステム shall 各エンジンインスタンスが完全に独立したデータを所有し（Rune Unit、RuntimeContext、キャッシュ）、共有ポインタを使用しない
3. When あるスレッドのエンジンがRuneランタイムエラーを発生させる, the pastaシステム shall そのエラーを該当スレッド内にとどめ、グローバル状態が存在しないため他のスレッドに影響を与えない
4. The pastaシステム shall エンジンインスタンスが`Send`トレイトを実装し、スレッド境界を越えて移動可能にする（共有は不要）
5. When 複数スレッドが同時に異なるエンジンインスタンスを操作する, the pastaシステム shall グローバル状態が存在しないため、データ競合やデッドロックを発生させない

### Requirement 4: 複数インスタンステストの整備

**Objective:** As a pasta library maintainer, I want 複数エンジンインスタンスの独立性を検証する自動テストが存在する, so that リグレッションを防止し、コードの変更が独立性を損なわないことを保証できる

#### Acceptance Criteria

1. The pastaテストスイート shall 同一プロセス内に2つの`PastaEngine`インスタンスを作成し、異なるスクリプトを実行しても互いに影響を与えないことを検証するテストケースを含む
2. The pastaテストスイート shall 2つのエンジンインスタンスが同名のグローバル変数を設定しても相互に干渉しないことを検証するテストケースを含む
3. The pastaテストスイート shall 同じスクリプト文字列から複数のエンジンインスタンスを作成し、各インスタンスが独立してパース・コンパイルを実行し、独立した実行状態を持つことを検証するテストケースを含む
4. The pastaテストスイート shall エンジンインスタンスが異なる`RandomSelector`実装を持ち、乱数選択が独立していることを検証するテストケースを含む
5. When テストスイートが実行される, the pastaテストスイート shall 全てのインスタンス独立性テストが成功し、`cargo test`コマンドで自動実行可能である

### Requirement 5: 並行実行テストの整備

**Objective:** As a pasta library maintainer, I want マルチスレッド環境での動作を検証する自動テストが存在する, so that スレッドセーフ性の保証と並行実行時の問題を早期に検出できる

#### Acceptance Criteria

1. The pastaテストスイート shall 複数スレッドでそれぞれ独立した`PastaEngine`を作成し、同時に`execute_label`を実行し、互いに影響を与えないことを検証するテストケースを含む
2. The pastaテストスイート shall 複数スレッドが同一スクリプト文字列から各自の`PastaEngine`を作成し、各インスタンスが独立してパース・コンパイルを実行することを検証するテストケースを含む
3. The pastaテストスイート shall グローバル状態が存在しないため、スレッド間でのデータ競合やデッドロックが構造的に発生しないことを検証するテストケースを含む
4. When 並行実行テストが実行される, the pastaテストスイート shall 全スレッドが完全に独立した結果を生成し、期待通りの`ScriptEvent`列を返す
5. The pastaテストスイート shall `cargo test --release`および`cargo test`の両方で並行実行テストが成功し、Miriなどのメモリ安全性チェッカーでも問題がない

### Requirement 6: グローバル状態不在の検証

**Objective:** As a pasta library maintainer, I want エンジンがグローバル状態を持たないことを検証する自動テストが存在する, so that 完全な独立性を保証し、将来の変更でグローバル状態が導入されないことを確認できる

#### Acceptance Criteria

1. The pastaテストスイート shall 同一スクリプトから2つの`PastaEngine`を作成し、一方のエンジンを破棄した後も他方のエンジンが正常に動作し、所有権により安全にメモリが解放されることを検証するテストケースを含む
2. The pastaテストスイート shall 複数のエンジンインスタンスが同時にパース処理を実行しても、各インスタンスが独立したデータを所有しているため相互に影響を与えないことを検証するテストケースを含む
3. The pastaテストスイート shall エンジンインスタンスのドロップ（破棄）が所有権システムによりすべてのデータを解放し、他のエンジンインスタンスに影響を与えないことを検証するテストケースを含む
4. The pastaテストスイート shall コードレビューまたは`grep`による静的チェックで`static`変数（`static mut`、`OnceLock`、`LazyLock`など）の不在を確認する手順を含む
5. When グローバル状態不在テストが実行される, the pastaテストスイート shall 全てのエンジンインスタンスが完全に独立し、`Arc`などの共有ポインタを使用していないことを確認する

### Requirement 7: テスト実行とCI統合

**Objective:** As a pasta library maintainer, I want 全ての独立性・並行実行テストがCI/CDパイプラインで自動実行される, so that プルリクエストごとに品質を保証し、リグレッションを防止できる

#### Acceptance Criteria

1. The pastaプロジェクト shall `cargo test`コマンドで全ての独立性テスト、並行実行テスト、キャッシュテストを実行可能にする
2. The pastaプロジェクト shall テストが`crates/pasta/tests/`ディレクトリ内の適切なファイルに配置されている（例: `engine_independence_test.rs`, `concurrent_execution_test.rs`）
3. When CI/CDパイプラインが実行される, the pastaプロジェクト shall 全てのテストケースが成功することを検証条件とする
4. If いずれかのテストケースが失敗する, then the CI/CDパイプライン shall ビルドを失敗とし、プルリクエストのマージを防止する
5. The pastaプロジェクト shall テスト実行時にRust標準のテストフレームワークを使用し、追加の外部依存を導入しない
