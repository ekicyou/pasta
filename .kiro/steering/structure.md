# Project Structure Steering

## ディレクトリ構造

```
pasta/                        # Cargo ワークスペースルート（Pure Virtual Workspace）
├── Cargo.toml               # ワークスペース設定のみ（[package] セクションなし）
├── crates/                  # クレート群
│   ├── pasta_core/          # 言語非依存層
│   │   ├── Cargo.toml       # pasta_core設定
│   │   └── src/
│   │       ├── lib.rs       # クレートエントリーポイント
│   │       ├── error.rs     # ParseError, SceneTableError, WordTableError
│   │       ├── parser/      # パーサーレイヤー（PEG → AST変換）
│   │       │   ├── mod.rs   # パーサーAPI公開
│   │       │   ├── ast.rs   # AST定義（Statement, Expr, LabelDef等）
│   │       │   └── grammar.pest # Pest文法定義
│   │       └── registry/    # 型管理レイヤー（独立）
│   │           ├── mod.rs   # Registry API
│   │           ├── scene_registry.rs # SceneRegistry - シーン管理
│   │           ├── word_registry.rs  # WordDefRegistry - 単語辞書
│   │           ├── scene_table.rs    # SceneTable - シーン検索
│   │           ├── word_table.rs     # WordTable - 単語検索
│   │           └── random.rs         # RandomSelector - ランダム選択
│   ├── pasta_rune/          # Rune言語バックエンド層（公開API）
│   │   ├── Cargo.toml       # pasta_rune設定（pasta_core依存）
│   │   ├── src/
│   │   │   ├── lib.rs       # クレートエントリーポイント、公開API定義
│   │   │   ├── engine.rs    # PastaEngine - 上位API層
│   │   │   ├── cache.rs     # ParseCache - パース結果キャッシュ
│   │   │   ├── loader.rs    # DirectoryLoader - スクリプト読み込み
│   │   │   ├── error.rs     # PastaError - ランタイムエラー型定義
│   │   │   ├── ir/          # ScriptEvent - IR出力型
│   │   │   │   └── mod.rs
│   │   │   ├── transpiler/  # トランスパイラレイヤー（AST → Rune、2pass）
│   │   │   │   ├── mod.rs   # Transpiler API
│   │   │   │   ├── code_generator.rs # Runeコード生成
│   │   │   │   ├── context.rs # トランスパイルコンテキスト
│   │   │   │   └── error.rs  # トランスパイルエラー型
│   │   │   ├── runtime/     # ランタイムレイヤー
│   │   │   │   ├── mod.rs   # ランタイムAPI
│   │   │   │   ├── generator.rs # ScriptGenerator - Rune VM実行
│   │   │   │   └── variables.rs # VariableManager - 変数管理
│   │   │   └── stdlib/      # Pasta標準ライブラリ（Rune側関数）
│   │   │       ├── mod.rs   # stdlib API登録
│   │   │       └── persistence.rs # 永続化API
│   │   └── tests/           # pasta_rune統合テスト
│   │       ├── common/      # テスト共通ユーティリティ
│   │       └── fixtures/    # テスト用Pastaスクリプト
│   └── pasta_lua/           # Lua言語バックエンド層
│       ├── Cargo.toml       # pasta_lua設定（pasta_core依存）
│       └── src/
│           ├── lib.rs       # クレートエントリーポイント
│           ├── config.rs    # 設定管理
│           ├── code_generator.rs # Lua コード生成
│           ├── context.rs   # トランスパイルコンテキスト
│           ├── error.rs     # エラー型
│           ├── runtime/     # ランタイムレイヤー
│           └── stdlib/      # Lua標準ライブラリ
├── tests/                    # ワークスペースレベル統合テスト
│   ├── common/              # テスト共通ユーティリティ
│   └── fixtures/            # テスト用Pastaスクリプト
│       ├── simple_hello.pasta
│       ├── comprehensive_control_flow.pasta
│       └── ...
│   ├── parser2_integration_test.rs  # パーサー統合テスト
│   ├── pasta_transpiler2_*.rs       # トランスパイラーテスト群
│   ├── pasta_engine_*.rs            # エンジンテスト群
│   ├── pasta_integration_*.rs       # E2E統合テスト群
│   └── ...
├── examples/                 # サンプルコード
│   ├── scripts/              # Pastaスクリプト例
│   ├── rune_module_test.rs   # Rune統合例
│   └── test_*.rs             # 動作確認例
├── benches/                  # ベンチマークコード
├── .kiro/                    # Kiro Spec-Driven設定
│   ├── steering/            # ステアリング規約
│   ├── settings/            # テンプレート・ルール
│   └── specs/               # 仕様管理
│       ├── completed/       # 完了仕様（アーカイブ）
│       └── <spec-name>/     # 進行中仕様
├── .vscode/                 # VS Code 設定
├── .github/                 # GitHub Actions, PR テンプレート
├── README.md                # プロジェクト概要
├── GRAMMAR.md               # Pasta DSL文法リファレンス
├── SPECIFICATION.md         # 言語仕様書
├── LICENSE                  # ライセンス
└── AGENTS.md                # AI開発支援ドキュメント
```

**注**: ルートクレート (`src/`) は削除済み。すべての実装コードは `crates/*/src/` 配下に配置。

## ファイル命名規則

### ソースファイル
- モジュールエントリー: `mod.rs`
- 単一機能モジュール: `<feature>.rs`（例: `engine.rs`, `cache.rs`）
- サブモジュール: ディレクトリ作成し`mod.rs`配置

### テストファイル
- 統合テスト: `crates/<crate>/tests/<feature>_test.rs`（アンダースコア区切り、単数形）
- フィクスチャ: `crates/<crate>/tests/fixtures/<scenario>.pasta`
- 共通ユーティリティ: `crates/<crate>/tests/common/mod.rs`

### 文法定義
- Pest文法: `src/parser/pasta.pest`

## モジュール構成

### ワークスペース構成

```
pasta (workspace)
├── pasta_core          # 言語非依存層（パーサー、レジストリ）
└── pasta_rune          # Runeバックエンド層（pasta_core依存）
```

### レイヤー分離原則
各レイヤーは上位レイヤーのみに依存：

**pasta_core:**
```
parser（AST生成）
  ↓
registry（シーン/単語テーブル）
  ↓
error（パースエラー）
```

**pasta_rune:**
```
engine (上位API)
  ↓
cache, loader
  ↓
transpiler (2pass)
  ↓
runtime
  ↓
stdlib, ir
  ↓
pasta_core（再エクスポート）
```

### 公開API (`pasta_core/lib.rs`)
- **Parser**: `parse_str()`, `parse_file()`, AST型（PastaFile, Statement, Expr等）
- **Registry**: `SceneRegistry`, `WordDefRegistry`, `SceneTable`, `WordTable`
- **Random**: `RandomSelector`, `DefaultRandomSelector`
- **Error**: `ParseError`, `SceneTableError`, `WordTableError`

### 公開API (`pasta_rune/lib.rs`)
- **Engine**: `PastaEngine`（統合API）
- **Transpiler**: `transpile()`, `TranspileContext`
- **Runtime**: `ScriptGenerator`, `VariableManager`
- **IR**: `ScriptEvent`, `ContentPart`
- **Error**: `PastaError`, `Result`, `Transpiler2Pass`
- **Core**: `pasta_core`の再エクスポート（`parser`, `core`エイリアス）

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
- `crates/<crate>/tests/<feature>_test.rs`: 統合テスト
- `crates/<crate>/tests/fixtures/*.pasta`: テスト用スクリプト
- `crates/<crate>/tests/common/`: 共通ユーティリティ

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
