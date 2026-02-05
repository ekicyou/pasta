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
│   └── pasta_lua/           # Lua言語バックエンド層
│       ├── Cargo.toml       # pasta_lua設定（pasta_core依存）
│       ├── src/
│       │   ├── lib.rs       # クレートエントリーポイント
│       │   ├── config.rs    # 設定管理
│       │   ├── code_generator.rs # Lua コード生成
│       │   ├── context.rs   # トランスパイルコンテキスト
│       │   ├── error.rs     # エラー型
│       │   ├── runtime/     # ランタイムレイヤー
│       │   └── stdlib/      # Lua標準ライブラリ
│       └── tests/           # pasta_lua統合テスト
│           ├── common/      # テスト共通ユーティリティ
│           └── fixtures/    # テスト用Pastaスクリプト
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

**注**: 
- ルートクレート (`src/`) は削除済み。すべての実装コードは `crates/*/src/` 配下に配置。
- 各クレートは独自の `tests/` ディレクトリを持つことができる（例: pasta_lua, pasta_sample_ghost）
- ワークスペースレベルの `tests/` は複数クレートにまたがる統合テスト用

## ファイル命名規則

### ソースファイル
- モジュールエントリー: `mod.rs`
- 単一機能モジュール: `<feature>.rs`（例: `engine.rs`, `cache.rs`）
- サブモジュール: ディレクトリ作成し`mod.rs`配置

### テストファイル
- 統合テスト: `crates/<crate>/tests/<feature>_test.rs`（アンダースコア区切り、単数形）
- フィクスチャ: `crates/<crate>/tests/fixtures/<scenario>.pasta`
- 共通ユーティリティ: `crates/<crate>/tests/common/mod.rs`
- クレート専用テスト: 各クレート配下の `tests/` に配置可能

### 文法定義
- Pest文法: `src/parser/pasta.pest`

## モジュール構成

### ワークスペース構成

```
pasta (workspace)
├── pasta_core          # 言語非依存層（パーサー、レジストリ）
└── pasta_lua           # Luaバックエンド層（pasta_core依存）
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

**pasta_lua:**
```
loader (スクリプト読み込み)
  ↓
transpiler (AST→Lua)
  ↓
runtime (Lua VM)
  ↓
pasta_core（再エクスポート）
```

### 公開API (`pasta_core/lib.rs`)
- **Parser**: `parse_str()`, `parse_file()`, AST型（PastaFile, Statement, Expr等）
- **Registry**: `SceneRegistry`, `WordDefRegistry`, `SceneTable`, `WordTable`
- **Random**: `RandomSelector`, `DefaultRandomSelector`
- **Error**: `ParseError`, `SceneTableError`, `WordTableError`



## テスト構成

| カテゴリ     | 対象                | ファイル例                       |
| ------------ | ------------------- | -------------------------------- |
| Parser       | 文法パース、エラー  | `span_byte_offset_test.rs`       |
| Transpiler   | Lua変換、シーン管理 | `transpiler_integration_test.rs` |
| Runtime      | Lua VM、シーン解決  | `runtime_e2e_test.rs`            |
| Loader       | スクリプト読み込み  | `loader_integration_test.rs`     |
| Registry     | 型管理、独立性      | `scene_search_test.rs`           |
| Control Flow | Call、最適化        | `transpiler_snapshot_test.rs`    |

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
