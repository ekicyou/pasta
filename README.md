# pasta
Memories of pasta twine together—now and then a knot, yet always a delight.

## 関連ドキュメント

| ドキュメント                         | 説明                                               |
| ------------------------------------ | -------------------------------------------------- |
| [GRAMMAR.md](GRAMMAR.md)             | Pasta DSL 文法リファレンス（学習用クイックガイド） |
| [SPECIFICATION.md](SPECIFICATION.md) | Pasta DSL 正式言語仕様書（実装判断の権威的ソース） |
| [AGENTS.md](AGENTS.md)               | AI開発支援ドキュメント（Kiro仕様駆動開発）         |

## アーキテクチャ

Pastaは、areka デスクトップマスコットアプリケーション向けのスクリプトエンジンです。

### レイヤー構成

```
Engine (上位API) → Cache/Loader
    ↓
Transpiler (2pass) ← Parser (Pest)
    ↓
Runtime (Rune VM) → IR Output (ScriptEvent)
```

### パーサー/トランスパイラーアーキテクチャ

Pastaは `parser2` + `transpiler2` スタックを使用しています：

| モジュール           | 文法ファイル   | 状態               | 用途                  |
| -------------------- | -------------- | ------------------ | --------------------- |
| `pasta::parser2`     | `grammar.pest` | **現行**           | engine.rsで使用       |
| `pasta::transpiler2` | -              | **現行**           | 2-pass トランスパイル |
| `pasta::parser`      | `pasta.pest`   | レガシー（非推奨） | 後方互換性のため維持  |
| `pasta::transpiler`  | -              | レガシー（非推奨） | 後方互換性のため維持  |

#### 使用方法

```rust
// 現行スタック（推奨）
use pasta_core::parser::{parse_str, parse_file};
use pasta_lua::transpiler::Transpiler;
```

#### 移行履歴

1. **Phase 1**: ✅ parser2モジュールを作成、parser と並存
2. **Phase 2**: ✅ transpiler2を作成、parser2と連携
3. **Phase 3**: ✅ engine.rs を parser2/transpiler2 に完全移行
4. **Phase 4**: （保留）レガシースタックの削除は後方互換性確認後に実施

### parser2について

`parser2`モジュールは、検証済みの`pasta2.pest`文法（`grammar.pest`として配置）に基づいています。

主な特徴：
- 3層スコープ構造：`FileScope` ⊃ `GlobalSceneScope` ⊃ `LocalSceneScope`
- 未名グローバルシーンの自動名前継承
- 全角/半角数字の自動正規化
- 継続行アクション（`：`で始まる行）のサポート

---

## ドキュメントマップ

### Level 0: Entry Point
- [README.md](README.md) - プロジェクト概要（このドキュメント）

### Level 1: Core Docs
- [GRAMMAR.md](GRAMMAR.md) - Pasta DSL 文法リファレンス
- [SPECIFICATION.md](SPECIFICATION.md) - 言語仕様書（権威的ソース）
- [AGENTS.md](AGENTS.md) - AI開発支援ドキュメント

### Level 2: Crate Docs
- [pasta_core/README.md](crates/pasta_core/README.md) - パーサー・レジストリ
- [pasta_lua/README.md](crates/pasta_lua/README.md) - Luaトランスパイラ
- [pasta_shiori/README.md](crates/pasta_shiori/README.md) - SHIORI DLL統合

### Level 3: Steering
- [.kiro/steering/](.kiro/steering/) - AI/仕様駆動開発コンテキスト

### 開発者向けリソース
- [crates/pasta_lua/tests/fixtures/](crates/pasta_lua/tests/fixtures/) - テストフィクスチャ

---

## オンボーディングパス

### DSLユーザー向け（推定所要時間: 30分）
1. [GRAMMAR.md](GRAMMAR.md) - 基本文法を学ぶ
2. [crates/pasta_lua/tests/fixtures/sample.pasta](crates/pasta_lua/tests/fixtures/sample.pasta) - サンプルスクリプト
3. [クイックスタート](#クイックスタート) - ビルド・実行方法

### 開発者向け（推定所要時間: 2-3時間）
1. [SPECIFICATION.md](SPECIFICATION.md) - 言語仕様の理解
2. [pasta_core/README.md](crates/pasta_core/README.md) - コアアーキテクチャ
3. [.kiro/steering/structure.md](.kiro/steering/structure.md) - プロジェクト構造
4. [.kiro/steering/workflow.md](.kiro/steering/workflow.md) - 開発ワークフロー

### AI開発支援向け（推定所要時間: 1時間）
1. [AGENTS.md](AGENTS.md) - AI開発支援概要
2. [.kiro/steering/](.kiro/steering/) - ステアリングファイル群
3. [SPECIFICATION.md](SPECIFICATION.md) - 正式仕様

---

## クイックスタート

### 前提条件
- Rust 2024 edition
- cargo

### ビルド
```bash
cargo build --workspace
```

### テスト
```bash
cargo test --workspace
```

### プロジェクト構造
```
pasta/
├── crates/
│   ├── pasta_core/    # パーサー・レジストリ（言語非依存層）
│   ├── pasta_lua/     # Luaトランスパイラ・ランタイム
│   └── pasta_shiori/  # SHIORI DLL統合
└── tests/             # 統合テスト・フィクスチャ
```

---

## ライセンス

[LICENSE](LICENSE) ファイルを参照してください。
