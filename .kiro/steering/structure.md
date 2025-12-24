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
│   ├── parser/              # パーサーレイヤー（PEG → AST変換）
│   │   ├── mod.rs           # パーサーAPI公開
│   │   ├── ast.rs           # AST定義（Statement, Expr, LabelDef等）
│   │   └── grammar.pest     # Pest文法定義
│   ├── transpiler/          # トランスパイラレイヤー（AST → Rune、2pass）
│   │   ├── mod.rs           # Transpiler API
│   │   ├── code_generator.rs # Runeコード生成
│   │   ├── context.rs       # トランスパイルコンテキスト
│   │   └── error.rs         # トランスパイルエラー型
│   ├── registry/            # 型管理レイヤー（独立）
│   │   ├── mod.rs           # Registry API
│   │   ├── scene_registry.rs # SceneRegistry - シーン管理
│   │   └── word_registry.rs  # WordDefRegistry - 単語辞書
│   ├── runtime/             # ランタイムレイヤー
│   │   ├── mod.rs           # ランタイムAPI
│   │   ├── generator.rs     # ScriptGenerator - Rune VM実行
│   │   ├── variable.rs      # VariableManager - 変数管理
│   │   ├── label_table.rs   # LabelTable - シーン検索テーブル
│   │   └── random.rs        # RandomSelector - ランダム選択
│   └── stdlib/              # Pasta標準ライブラリ（Rune側関数）
│       └── mod.rs           # stdlib API登録
├── tests/                    # 統合テスト
│   ├── common/              # テスト共通ユーティリティ
│   ├── fixtures/            # テスト用Pastaスクリプト
│   │   ├── simple_hello.pasta
│   │   ├── comprehensive_control_flow.pasta
│   │   └── ...
│   ├── parser2_integration_test.rs  # パーサー統合テスト
│   ├── pasta_transpiler2_*.rs       # トランスパイラーテスト群
│   ├── pasta_engine_*.rs            # エンジンテスト群
│   ├── pasta_integration_*.rs       # E2E統合テスト群
│   └── ...
├── examples/                 # サンプルコード（将来追加）
├── benches/                  # ベンチマークコード（将来追加）
├── .kiro/                    # Kiro Spec-Driven設定
│   ├── steering/            # ステアリング規約
│   └── specs/               # 仕様管理
│       ├── completed/       # 完了仕様（アーカイブ）
│       └── <spec-name>/     # 進行中仕様
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
parser, transpiler (2pass)
  ↓
runtime
  ↓
registry (公開型) → stdlib, ir
```

### 公開API (`lib.rs`)
- **Parser**: `parse()`, AST型（Statement, Expr等）
- **Transpiler**: `transpile()`, `TranspileContext`
- **Registry**: `SceneRegistry`, `WordDefRegistry`, `WordEntry`
- **Runtime**: `ScriptGenerator`, `LabelTable`, `VariableManager`
- **Engine**: `PastaEngine`（統合API）
- **IR**: `ScriptEvent`, `ContentPart`
- **Error**: `PastaError`, `Result`

### 内部モジュール
- `loader`: ディレクトリスキャン・ファイル読み込み
- `cache`: パース結果メモリキャッシュ
- `registry`: シーン登録・単語辞書・型管理（独立層）
- `runtime::generator`: Rune VM実行・Generator継続管理

## テスト構成

| カテゴリ     | 対象                  | ファイル例                                  |
| ------------ | --------------------- | ------------------------------------------- |
| Parser       | 文法パース、エラー    | `parser2_integration_test.rs`               |
| Transpiler   | 2パス変換、シーン管理 | `pasta_transpiler2_*.rs`                    |
| Runtime      | Rune VM、シーン解決   | `pasta_engine_rune_*.rs`                    |
| Engine       | E2E統合、スコープ     | `pasta_engine_*.rs`                         |
| Registry     | 型管理、独立性        | `pasta_stdlib_call_jump_separation_test.rs` |
| Control Flow | Call/Jump、並行実行   | `pasta_integration_control_flow_test.rs`    |

### テストファイル配置
- `tests/<feature>_test.rs`: 統合テスト
- `tests/fixtures/*.pasta`: テスト用スクリプト
- `tests/common/`: 共通ユーティリティ

**注**: 旧parser/transpiler実装に依存していたテスト21ファイルは削除済み（2024-12-24 legacy-parser-transpiler-cleanup完了）

## ドキュメント構成

| ファイル         | 用途                |
| ---------------- | ------------------- |
| README.md        | プロジェクト概要    |
| GRAMMAR.md       | DSL文法リファレンス |
| SPECIFICATION.md | 言語仕様書          |
| AGENTS.md        | AI開発支援          |

### Kiro仕様管理
- `.kiro/steering/`: 規約・原則
- `.kiro/specs/completed/`: 完了仕様アーカイブ
- `.kiro/specs/<name>/`: 進行中仕様

### コードドキュメント
- `///`: 公開APIドキュメント
- `//!`: モジュール概要
- Doctest: 使用例をドキュメント内に記述
