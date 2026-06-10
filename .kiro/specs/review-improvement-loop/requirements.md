# Requirements Document

## Project Description (Input)

リポジトリ全域のコード品質（テスト網羅性・簡潔性・安全性）を、**外部から観測可能な挙動を変えずに**継続的に底上げする「自己発見ループ型」の再実行可能プロセス仕様。

1回の実装指示（`/kiro-impl review-improvement-loop`）で、(1) レビュー領域の自己発見（ギャップ分析）と `レビュー領域 × レビュー内容（7次元）` マトリクスの動的生成、(2) 各セルのサブエージェント委譲による改善実行、(3) サイクルごとの破壊検知とコミット、(4) デバッグ不能セルの巻き戻しとスキップによる全完走保証、(5) 完走後の改善内容レポート生成、を中断なく完走する。

`release-workflow` と同様の再実行型 spec として `.kiro/specs/` 直下に留まり、別プロジェクトへ spec ディレクトリをコピーして再実行するだけで、そのプロジェクトの領域を自動再発見し同等の効果が得られる移植性を持つ。

（詳細は `.kiro/specs/review-improvement-loop/brief.md` を参照）

---

## Introduction

本ドキュメントは、コード総合レビュー＆改善ループ（Review Improvement Loop）の要件を定義する。本ループは LLM エージェントが実装指示のたびに繰り返し実行するプロセス仕様であり、対象リポジトリのレビュー領域を実行時に自己発見し、確定 7 次元のレビュー観点とのマトリクスを生成して、各セルを改善・検証・コミットしながら全セルを完走する。

### 仕様の特殊性

本仕様は通常の機能仕様と異なり、以下の特性を持つ：

- **再実行型**: `/kiro-impl review-improvement-loop` が実行されるたびにタスク状態はリセットされ、新たな改善ループとして実行される。本仕様は `completed/` へ移行せず、常に `.kiro/specs/` 直下に留まる
- **プロセス仕様**: 新機能の追加を伴わず、既存コードの品質改善（挙動保存）のみを行う。改善対象（ワークリスト）は仕様に固定されず、実行時のギャップ分析で動的に決定される
- **移植可能**: 仕様ディレクトリを別プロジェクトへコピーして実装指示を再実行するだけで、コピー先の領域を自動再発見し同等のループが成立する

## Boundary Context

- **In scope**: 全レビュー領域（Rust クレート・Lua ランタイム資産・VSCode 拡張・`book/tools` を含む全ソース資産）× 全 7 次元のマトリクス改善を 1 実装指示で完走するループ。領域の自己発見（ギャップ分析）、サブエージェント委譲、サイクル毎コミット、破壊検知、巻き戻し、最終レポート。既存の品質検証インフラ（本リポジトリでは Rust=`cargo test`/`cargo clippy`/`cargo-audit`/`cargo-deny`/`insta`、Lua=`luacheck`/`lua_test`、TypeScript/JS=型検査・lint・`npm` テスト、横断=`TEST_COVERAGE.md`・`book` drift-check）の活用と必要に応じた追加
- **Out of scope**: 新機能の追加・外部仕様の変更（挙動保存が前提）。完了済み `audit-pasta-*` spec の改変・再オープン。性能チューニングを主目的とする作業（簡素化の副次効果は可）。CI 設定そのものの再設計（既存ワークフローとの整合確認は可）。Kiro ワークフロー自体（spec/skill 群）の改修。pasta DSL 文法・Lua API の仕様変更。リリース・配布フロー（`release-workflow` の領分）。マニュアル `book/` の内容拡充（ドキュメント整合次元での同期確認は In）
- **Adjacent expectations**: 既存スキル `karpathy-guidelines`（簡素化基準）・`kiro-review`（敵対的レビュー）・`kiro-debug`（根本原因デバッグ）・`kiro-verify-completion`（完了前検証）が利用可能であることを前提とする。`workflow.md` の DoD・回帰責任・危険 Git 操作禁止・MVP 禁止ルールに整合する。本リポジトリでは `cargo` 実行前に環境変数 `NoDefaultCurrentDirectoryInExePath` の解除が必要（既知のビルド環境制約）

---

## Requirements

### Requirement 1: レビュー領域の自己発見とマトリクス生成

**Objective:** As a 開発者, I want 実装指示のたびに対象リポジトリのレビュー領域を自動発見し、レビューマトリクスを動的生成したい, so that 手作業や暗黙知に依存せず、網羅的かつ再現可能に改善対象を定義できる

#### Acceptance Criteria

1. When 改善ループが開始される, the Review Improvement Loop shall 対象リポジトリの構造をギャップ分析し、レビュー領域の一覧を自己発見する
2. The Review Improvement Loop shall レビュー領域を、対象プロジェクトの全ソース資産カテゴリ（コンパイル単位に限らずスクリプト資産・拡張・ツール群を含む）にわたり、最低でも各トップレベル構成単位（本リポジトリでは Rust クレート・Lua ランタイム資産（`pasta_scripts/`・`tests/lua_specs/`）・VSCode 拡張（TypeScript）・`book/tools`（JS））の粒度で識別する
3. If 単一領域の規模がサブエージェント 1 回で安全に処理できる粒度を超える, the Review Improvement Loop shall 当該領域をサブモジュール等のより小さな領域へ細分化する
4. The Review Improvement Loop shall 領域一覧を事前定義の固定リストからではなく、実行時のリポジトリ分析結果から決定する
5. When 領域一覧が確定する, the Review Improvement Loop shall `レビュー領域 × レビュー 7 次元` のマトリクスをワークリストとして生成し、開発者が確認可能な形式で記録する

### Requirement 2: レビュー 7 次元の実施

**Objective:** As a 開発者, I want 各レビュー領域に対して確定された 7 つの観点で点検と改善を実施したい, so that レビュー観点の抜け漏れが領域ごとに生じない

#### Acceptance Criteria

1. The Review Improvement Loop shall 各レビュー領域に対し、次の 7 次元をレビュー内容として適用する: (1) テスト網羅性 (2) コード簡素化 (3) 脆弱性対策 (4) lint 徹底 (5) デッドコード/未使用除去 (6) パニック経路削減 (7) ドキュメント/依存整合
2. When テスト網羅性次元のセルを実行する, the Review Improvement Loop shall 不足しているテストを特定して追加する
3. If テスト網羅性次元で不要なテストを除外する, the Review Improvement Loop shall 慎重に判断したうえで除外の根拠を明記する
4. When コード簡素化次元のセルを実行する, the Review Improvement Loop shall 簡素化ガイドライン（`karpathy-guidelines`）への準拠を検証し、逸脱を是正する
5. When 脆弱性対策次元のセルを実行する, the Review Improvement Loop shall 脆弱性レビューを実施し、必要な対策コードを投入する
6. When lint 徹底次元のセルを実行する, the Review Improvement Loop shall 対象プロジェクト標準の lint（本リポジトリでは `cargo clippy`）を警告ゼロ相当の水準で通過させ、慣用句からの逸脱を是正する
7. When デッドコード/未使用除去次元のセルを実行する, the Review Improvement Loop shall 未使用の公開エクスポート・関数・依存を検出して除去する
8. When パニック経路削減次元のセルを実行する, the Review Improvement Loop shall 回復不能停止に至る経路（`unwrap`/`expect`/インデックスパニック等）を明示的なエラー処理へ置換し、FFI/SHIORI 等の外部境界を特に重視する
9. When ドキュメント/依存整合次元のセルを実行する, the Review Improvement Loop shall テストマッピング台帳・README・仕様ドキュメントの同期確認、およびサプライチェーン監査（本リポジトリでは `cargo-audit`/`cargo-deny`）を実施する
10. Where レビュー領域が特定の資産種別に属する, the Review Improvement Loop shall 各次元を当該資産種別に適合したツール・手法で実現し（本リポジトリでは Rust=`cargo clippy`/`cargo-audit`/`cargo-deny`、Lua=`luacheck`/`lua_test`、TypeScript/JS=型検査・lint・`npm` テスト等）、当該資産種別に適用不能な次元は「該当なし」として記録する

### Requirement 3: 挙動保存ポリシー

**Objective:** As a 開発者, I want 改善が外部から観測可能な挙動を変えないことを保証したい, so that 利用者への影響ゼロでコード品質を底上げできる

#### Acceptance Criteria

1. The Review Improvement Loop shall 正常系（妥当な入力）の外部観測挙動を厳密に保存する
2. Where 脆弱性対策が不正入力・攻撃面のハードニングに該当する, the Review Improvement Loop shall 当該箇所に限り挙動変化を許容する
3. When 攻撃面ハードニングによる挙動変化を許容する, the Review Improvement Loop shall 変化の境界を回帰テストで明示し、最終レポートに記録する
4. When 挙動等価を意図した改善（簡素化・lint 是正・デッドコード除去・パニック経路置換のうち正常系で観測不能なもの）を行う, the Review Improvement Loop shall 既存テストおよびスナップショットテストによって等価性を検証する
5. The Review Improvement Loop shall 新機能の追加および外部仕様の変更を行わない
6. Where 到達可能なパニック経路（不正・異常入力で発火しうる `unwrap`/`expect`/インデックスパニック等）を明示的なエラー処理へ置換する, the Review Improvement Loop shall 当該置換を許容ハードニング（R3.2）として扱い、異常系の挙動変化を許容したうえで R3.3 に従い境界を回帰テストで明示・記録する
7. The Review Improvement Loop shall FFI/SHIORI 境界を越えるパニックを未定義動作とみなし、その削減を安全性修正として扱う

### Requirement 4: サイクル実行・破壊検知・コミット

**Objective:** As a 開発者, I want セル単位の改善サイクルごとに破壊検知とコミットを行いたい, so that リグレッションを即座に検出し、改善を安全に積み上げられる

#### Acceptance Criteria

1. When 改善ループが開始される, the Review Improvement Loop shall 改善着手前にベースライン検証（本リポジトリでは `cargo test --workspace` 等の全体テスト）を実行し、グリーン状態を確認する
2. If 開始時のベースライン検証が失敗する, the Review Improvement Loop shall 改善作業に着手せず、失敗内容を開発者へ報告して実行を中断する
3. When 1 つのセルの改善が完了する, the Review Improvement Loop shall ベースライン検証を再実行し、改善前後で破壊（リグレッション）が無いことを確認する
4. When セルの検証がグリーンである, the Review Improvement Loop shall 当該セルの変更をコミットする（サイクル完了＝グリーン確認＝コミットを 1 単位とする）
5. If セルの検証で破壊が検出される, the Review Improvement Loop shall 根本原因特定に基づくデバッグ（`kiro-debug` プロトコル）により修復を試みる
6. When セルの点検で改善すべき事項が検出されない, the Review Improvement Loop shall 当該セルを「確認済み（改善不要）」として記録し、空コミットを作成せずに次のセルの処理へ進む
7. Where 対象プロジェクトに既知のビルド環境制約が記録されている（本リポジトリでは検証コマンド実行前の環境変数 `NoDefaultCurrentDirectoryInExePath` 解除）, the Review Improvement Loop shall 検証コマンドの実行時にその制約に従う

### Requirement 5: 巻き戻し・スキップと完走保証

**Objective:** As a 開発者, I want デバッグ不能に陥ったセルを安全に巻き戻して次へ進みたい, so that 1 回の実装指示で全セルの完走が保証され、途中放棄が起こらない

#### Acceptance Criteria

1. If セルのデバッグが収束しない（`kiro-debug` プロトコルによっても修復を完遂できないと判断される）, the Review Improvement Loop shall 当該セルで加えた変更のみを直前コミット時点の状態へ復元する
2. When セルを巻き戻す, the Review Improvement Loop shall スキップ理由を記録したうえで次のセルの処理へ進む
3. When セルを巻き戻す, the Review Improvement Loop shall 当該セル以外の変更（コミット済みの成果および他セッションの未コミット作業）を巻き込まない
4. The Review Improvement Loop shall 巻き戻しにおいて、無関係な変更を一括破棄しうる破壊的 Git 操作（`workflow.md` で禁止された操作）を使用しない
5. The Review Improvement Loop shall ワークリスト上の全セルが処理完了（改善コミット済み・確認済み（改善不要）として記録済み・スキップ記録済みのいずれか）となるまで実行を継続する
6. The Review Improvement Loop shall 途中中断および部分完了の完成宣言（「MVP」「部分実装」等）を行わない

### Requirement 6: サブエージェント委譲とオーケストレーション

**Objective:** As a 開発者, I want 重い分析・改善・レビュー作業をサブエージェントへ委譲したい, so that メインエージェントのコンテキストが枯渇せず、全セルを完走できる

#### Acceptance Criteria

1. When ギャップ分析・各セルの改善・自己レビュー・レポート集約を実行する, the Review Improvement Loop shall それぞれの作業をサブエージェントへ委譲する
2. The Review Improvement Loop shall メインエージェントの責務をオーケストレーション（ワークリスト管理・コミット・巻き戻し判断・進捗追跡）に限定する
3. When セルの改善が完了する, the Review Improvement Loop shall 改善実施者とは独立した自己レビュー（`kiro-review` の敵対的レビュー観点）により、破壊の有無と改善の妥当性を検証する
4. When ループ全体の完了を宣言する, the Review Improvement Loop shall 新しい証拠に基づく完了検証（`kiro-verify-completion`）を実施したうえで宣言する

### Requirement 7: 改善内容レポート

**Objective:** As a 開発者, I want 全完走後に改善内容の統合レポートを得たい, so that 改善の全容・許容した挙動変化・スキップ箇所を把握し、今後のリファクタリングやテスト戦略の入力にできる

#### Acceptance Criteria

1. When 全セルの処理が完了する, the Review Improvement Loop shall 改善内容レポートを生成し開発者へ報告する
2. The Review Improvement Loop shall レポートに各セルの実施結果（領域・次元・改善内容・対応コミット）を含める
3. The Review Improvement Loop shall レポートに許容した挙動変化（攻撃面ハードニング）とその境界を明示した回帰テストへの参照を含める
4. The Review Improvement Loop shall レポートにスキップしたセルの一覧とスキップ理由を含める
5. The Review Improvement Loop shall レポートに「確認済み（改善不要）」だったセルの一覧を含め、改善実施セルと区別する

### Requirement 8: 再実行型運用と移植性

**Objective:** As a 開発者, I want 本仕様を繰り返し再実行でき、別プロジェクトへコピーするだけで移植できるようにしたい, so that 継続的な品質底上げと他プロジェクトへの横展開が低コストで実現できる

#### Acceptance Criteria

1. The Review Improvement Loop shall 再実行型仕様として運用され、完走後も仕様ディレクトリを `completed/` へ移動しない
2. When 実装指示が再実行される, the Review Improvement Loop shall タスク実行状態を初期化し、領域の自己発見からループを再走する
3. When 本仕様ディレクトリが別プロジェクトへコピーされ実装指示が実行される, the Review Improvement Loop shall コピー先プロジェクトのレビュー領域を自動再発見し、同等の改善ループを実行する
4. When 別プロジェクトで実行される, the Review Improvement Loop shall コピー先プロジェクトに存在する品質検証インフラ（テスト・lint・監査ツール等）を発見して活用する
