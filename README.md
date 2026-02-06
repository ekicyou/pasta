# pasta
Memories of pasta twine together—now and then a knot, yet always a delight.

## Pasta DSL サンプル

```pasta
＊OnBoot
　＠挨拶：ごきげんよう、お待ちしておりましたわ、まあまあ
　％ぱすた、ラザニア
　ラザニア：＠挨拶　！
　　　　　：べ、別にあなたを待っていたわけではありませんのよ？
　　ぱすた：素直じゃないなあ……
　　　　　：ようこそ！一緒に楽しもうね。
```

> **Pasta** は里々/Ren'Pyにインスパイアされた対話スクリプト言語/SHIORI.DLLです。
> 日本語フレンドリーな全角マーカー、前方一致によるランダム選択、Luaランタイムによる拡張性を特徴とします。
> 上のサンプルのように、シンプルな記述で自然な対話を実現できます。

---

## 関連ドキュメント

| ドキュメント             | 説明                                                         |
| ------------------------ | ------------------------------------------------------------ |
| [GRAMMAR.md](GRAMMAR.md) | Pasta DSL 文法リファレンス（学習用クイックガイド）           |
| [doc/spec/](doc/spec/)   | Pasta DSL 正式言語仕様書（章別分割、実装判断の権威的ソース） |
| [AGENTS.md](AGENTS.md)   | AI開発支援ドキュメント（Kiro仕様駆動開発）                   |

## アーキテクチャ

Pastaは、「伺か」などのデスクトップマスコットアプリケーション、あるいはノベルゲーム用途に向いたスクリプトエンジンです。

### レイヤー構成

```
Engine (上位API) → Cache/Loader
    ↓
Transpiler (2pass) ← Parser (Pest)
    ↓
Runtime (Lua VM) → IR Output (ScriptEvent)
```

### パーサー/トランスパイラーアーキテクチャ

Pastaは現行の `parser` + `transpiler` スタックを使用しています：

| モジュール              | 文法ファイル   | 状態     | 用途                  |
| ----------------------- | -------------- | -------- | --------------------- |
| `pasta_core::parser`    | `grammar.pest` | **現行** | engine.rsで使用       |
| `pasta_lua::transpiler` | -              | **現行** | 2-pass トランスパイル |

#### 使用方法

```rust
// 現行スタック
use pasta_core::parser::{parse_str, parse_file};
use pasta_lua::transpiler::Transpiler;
```

### parserについて

`parser`モジュールは、`grammar.pest`文法に基づいています。

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
- [doc/spec/](doc/spec/) - 言語仕様書（章別分割、権威的ソース）
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
1. [doc/spec/](doc/spec/) - 言語仕様の理解（必要な章のみ）
2. [pasta_core/README.md](crates/pasta_core/README.md) - コアアーキテクチャ
3. [.kiro/steering/structure.md](.kiro/steering/structure.md) - プロジェクト構造
4. [.kiro/steering/workflow.md](.kiro/steering/workflow.md) - 開発ワークフロー

### AI開発支援向け（推定所要時間: 1時間）
1. [AGENTS.md](AGENTS.md) - AI開発支援概要
2. [.kiro/steering/](.kiro/steering/) - ステアリングファイル群
3. [doc/spec/README.md](doc/spec/README.md) - 正式仕様（章別インデックス）

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
