# Project Structure Steering

## ディレクトリ構造

```
pasta/
├── src/                      # ソースコード
│   ├── lib.rs               # クレートエントリーポイント、公開API定義
│   ├── engine.rs            # PastaEngine - 上位API層
│   ├── cache.rs             # ParseCache - パース結果キャッシュ
│   ├── loader.rs            # DirectoryLoader - スクリプト読み込み
│   ├── error.rs             # PastaError - エラー型定義
│   ├── ir.rs                # ScriptEvent - IR出力型
│   ├── parser/              # パーサーレイヤー
│   │   ├── mod.rs           # パーサーAPI公開
│   │   ├── ast.rs           # AST定義（Statement, Expr, LabelDef等）
│   │   └── pasta.pest       # Pest文法定義
│   ├── transpiler/          # トランスパイラレイヤー
│   │   ├── mod.rs           # Transpiler API
│   │   └── label_registry.rs # ラベル管理・モジュール生成
│   ├── runtime/             # ランタイムレイヤー
│   │   ├── mod.rs           # ランタイムAPI
│   │   ├── generator.rs     # ScriptGenerator - Rune VM実行
│   │   ├── variable.rs      # VariableManager - 変数管理
│   │   ├── label_table.rs   # LabelTable - ラベル検索テーブル
│   │   └── random.rs        # RandomSelector - ランダム選択
│   └── stdlib/              # Pasta標準ライブラリ（Rune側関数）
│       └── mod.rs           # stdlib API登録
├── tests/                    # 統合テスト
│   ├── common/              # テスト共通ユーティリティ
│   ├── fixtures/            # テスト用Pastaスクリプト
│   │   ├── simple_hello.pasta
│   │   ├── comprehensive_control_flow.pasta
│   │   └── ...
│   ├── parser_tests.rs      # パーサーテスト
│   ├── transpile_comprehensive_test.rs
│   ├── engine_integration_test.rs
│   └── ...
├── examples/                 # サンプルコード（将来追加）
├── benches/                  # ベンチマークコード（将来追加）
├── .kiro/                    # Kiro Spec-Driven Development設定
│   ├── steering/            # ステアリング（プロジェクト規約）
│   │   ├── product.md       # プロダクトビジョン
│   │   ├── tech.md          # 技術スタック・原則
│   │   └── structure.md     # このファイル
│   └── specs/               # 仕様管理
│       ├── completed/       # 完了した仕様
│       │   ├── pasta-engine-independence/
│       │   ├── pasta-transpiler-pass2-output/
│       │   └── ...（計11件）
│       ├── pasta-yield-propagation/  # 進行中仕様
│       ├── pasta-word-definition-dsl/
│       └── ...（計9件）
├── Cargo.toml               # Cargo設定
├── README.md                # プロジェクト概要
├── GRAMMAR.md               # Pasta DSL文法リファレンス
├── LICENSE                  # ライセンス
└── AGENTS.md                # AI開発支援ドキュメント
```

## ファイル命名規則

### ソースファイル
- モジュールエントリー: `mod.rs`
- 単一機能モジュール: `<feature>.rs`（例: `engine.rs`, `cache.rs`）
- サブモジュール: ディレクトリ作成し`mod.rs`配置

### テストファイル
- 統合テスト: `tests/<feature>_test.rs`（アンダースコア区切り、単数形）
- フィクスチャ: `tests/fixtures/<scenario>.pasta`
- 共通ユーティリティ: `tests/common/mod.rs`

### 文法定義
- Pest文法: `src/parser/pasta.pest`

## モジュール構成

### レイヤー分離原則
各レイヤーは上位レイヤーのみに依存：

```
engine (上位API)
  ↓
cache, loader
  ↓
transpiler ← parser
  ↓
runtime
  ↓
stdlib, ir
```

### 公開API (`lib.rs`)
- **Parser**: `parse_str`, `parse_file`, AST型
- **Transpiler**: `Transpiler`, `TranspileContext`
- **Runtime**: `ScriptGenerator`, `LabelTable`, `VariableManager`
- **Engine**: `PastaEngine`（統合API）
- **IR**: `ScriptEvent`, `ContentPart`
- **Error**: `PastaError`, `Result`

### 内部モジュール
- `loader`: ディレクトリスキャン・ファイル読み込み
- `cache`: パース結果メモリキャッシュ
- `transpiler::label_registry`: ラベル登録・ID管理・モジュール生成
- `runtime::generator`: Rune VM実行・Generator継続管理

## テスト構成

### テストカテゴリ

#### パーサーテスト
- `parser_tests.rs`: 基本文法パース
- `parser_error_tests.rs`: エラーケース
- `parser_line_types.rs`: 行タイプ分類
- `sakura_script_tests.rs`: さくらスクリプトエスケープ

#### トランスパイラテスト
- `transpile_comprehensive_test.rs`: 総合トランスパイル
- `two_pass_transpiler_test.rs`: 2パス処理
- `label_registry_test.rs`: ラベル管理
- `actor_assignment_test.rs`: アクター変数生成

#### ランタイムテスト
- `comprehensive_rune_vm_test.rs`: Rune VM基本動作
- `rune_block_integration_test.rs`: Runeブロック統合
- `label_resolution_runtime_test.rs`: ラベル解決
- `function_scope_tests.rs`: 関数スコープ解決

#### エンジンテスト
- `engine_integration_test.rs`: E2E統合テスト
- `engine_independence_test.rs`: UI層独立性確認
- `engine_two_pass_test.rs`: エンジン2パス処理
- `directory_loader_test.rs`: スクリプトローダー

#### 制御フローテスト
- `comprehensive_control_flow_test.rs`: Call/Jump文総合
- `concurrent_execution_test.rs`: 並行実行

#### その他
- `error_handling_tests.rs`: エラーハンドリング
- `persistence_test.rs`: 永続化（シリアライゼーション）
- `stdlib_integration_test.rs`: 標準ライブラリ

### テストフィクスチャ
`tests/fixtures/` 配下のPastaスクリプト：
- `simple_hello.pasta`: 基本的な挨拶
- `comprehensive_control_flow.pasta`: Call/Jump文総合
- `actor_switch.pasta`: アクター切り替え
- `label_continuation.pasta`: ラベル継続

## ドキュメント構成

### プロジェクトルート
- **README.md**: プロジェクト概要、クイックスタート
- **GRAMMAR.md**: Pasta DSL完全文法リファレンス（10セクション、600行超）
- **AGENTS.md**: AI開発支援・コンテキスト情報
- **LICENSE**: MIT OR Apache-2.0

### Kiro仕様管理
- **`.kiro/steering/`**: プロジェクトステアリング（規約・原則）
- **`.kiro/specs/completed/`**: 完了仕様11件
  - `pasta-engine-independence`: UI層独立性確立
  - `pasta-transpiler-pass2-output`: 2パス出力実装
  - `pasta-label-resolution-runtime`: ラベル解決ランタイム
  - `pasta-script-loader`: スクリプトローダー
  - `pasta-declarative-control-flow`: 宣言的制御フロー
  - `pasta-serialization`: シリアライゼーション
  - `pasta-transpiler-actor-variables`: アクター変数
  - `pasta-chain-token-ir-check`: チェイントークンIRチェック
  - `pasta-test-missing-entry-hash`: エントリーハッシュテスト修正
  - `pasta-engine-doctest-fix`: Doctest修正
  - `areka-P0-script-engine`: Arekaスクリプトエンジン統合基盤
- **`.kiro/specs/`**: 進行中仕様9件
  - `pasta-yield-propagation`: Yield伝搬問題解決（実装中）
  - `pasta-word-definition-dsl`: 単語定義DSL（設計段階）
  - `pasta-local-rune-calls`: ローカルRune呼び出し
  - `pasta-label-continuation`: ラベル継続
  - `pasta-call-resolution-priority`: Call解決優先度
  - `pasta-conversation-inline-multi-stage-resolution`: インライン多段解決
  - `pasta-dialogue-continuation-syntax`: 対話継続構文
  - `pasta-jump-function-calls`: Jump関数呼び出し
  - `ukagaka-desktop-mascot`: 伺かデスクトップマスコット統合メタ仕様（32子仕様管理）

### コードドキュメント
- `///`ドキュメントコメント: 公開API全体
- `//!`モジュールコメント: 各モジュール先頭
- Doctestサンプル: `lib.rs`、主要API

### 将来追加予定
- **ARCHITECTURE.md**: 詳細アーキテクチャ設計
- **CHANGELOG.md**: バージョン履歴
- **examples/**: 使用例スクリプト
