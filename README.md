<img src="/img/pasta.svg" alt="Pasta logo" width="120" align="left" style="margin-right: 1em;">

# pasta
Memories of pasta twine together—now and then a knot, yet always a delight.

[![Build](https://github.com/ekicyou/pasta/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/ekicyou/pasta/actions/workflows/build.yml)
[![Release](https://github.com/ekicyou/pasta/actions/workflows/release.yml/badge.svg)](https://github.com/ekicyou/pasta/actions/workflows/release.yml)

<br clear="both">

**pasta** は、「伺か」のようなデスクトップマスコットや、シナリオ型ゲームを実現するための対話スクリプトエンジンです。
日本語で書ける Pasta DSL の辞書を Lua にトランスパイルし、SHIORI（`pasta.dll`）としてゴーストの頭脳になります。
全角マーカー、前方一致によるランダム選択、宣言的な Call／Jump、Lua による拡張を特徴とします。

```pasta
＊OnBoot
　＠挨拶：ごきげんよう、お待ちしておりましたわ、まあまあ
　％ぱすた、ラザニア
　ラザニア：＠挨拶　！
　　　　　：べ、別にあなたを待っていたわけではありませんのよ？
　　ぱすた：素直じゃないなあ……
　　　　　：ようこそ！一緒に楽しもうね。
```

---

## 📖 ゴーストを作りたい方へ

### 👉 **[pasta マニュアル](https://ekicyou.github.io/pasta/)**

案内役の Claudia と執事のアンソニーが、ゴーストづくりを最初から最後まで案内します。日本語で検索できます。

| パート | 内容 |
| ------ | ---- |
| [入門ガイド](https://ekicyou.github.io/pasta/getting-started/index.html) | 辞書を 1 枚ずつ足しながら、13 段でゴーストを育てて配布するまで |
| [Pasta DSL 文法](https://ekicyou.github.io/pasta/grammar/index.html) | マーカー・シーン・アクション行・変数・単語・アクター辞書 |
| [Lua API / コーディング](https://ekicyou.github.io/pasta/lua/index.html) | 公開モジュール・SHIORI イベント・`scripts/` の書き方 |
| [デバッグ](https://ekicyou.github.io/pasta/debug/index.html) | VSCode から `.pasta` の行で止めて調べる |
| [pasta.toml リファレンス](https://ekicyou.github.io/pasta/reference/pasta-toml.html) | 設定の全セクション・全キー |

最新版の配布物は、次のリンクから直接ダウンロードできます（過去の版は [リリースページ](https://github.com/ekicyou/pasta/releases)）。

- [`hello-pasta.nar`](https://github.com/ekicyou/pasta/releases/latest/download/hello-pasta.nar) — サンプルゴースト（SSP へドロップして入れる。`pasta.dll` とシェルを含む）
- [`pasta.dll.zip`](https://github.com/ekicyou/pasta/releases/latest/download/pasta.dll.zip) — SHIORI の本体（`pasta.dll` とライセンス表示）
- [Pasta DSL VSCode 拡張](https://marketplace.visualstudio.com/items?itemName=ekicyou.pasta-vscode) — 構文ハイライト・診断・デバッガ接続

---

## 🛠️ 開発者・コントリビュータの方へ

pasta 本体（パーサ・トランスパイラ・ランタイム・SHIORI）の開発に関わる方向けの入口です。

### ドキュメント

| ドキュメント | 説明 |
| ------------ | ---- |
| [SOUL.md](SOUL.md) | ビジョン・コアバリュー・設計原則 |
| [マニュアルの内部設計パート](https://ekicyou.github.io/pasta/internals/index.html)（[`book/src/internals/`](book/src/internals/)） | 現行実装の内部構造（トランスパイル・レジストリ・実行モデル・ローダ・SHIORI 層・トーク出力・デバッグ基盤） |
| [マニュアルの利用者向けパート](https://ekicyou.github.io/pasta/)（[`book/src/`](book/src/)） | 文法・公開 Lua API・`pasta.toml` の**唯一の権威**。スキル `references/` の規範部分は、マニュアルの章から生成する（`node book/tools/gen-skill-refs.mjs`） |
| [book/AUTHORING.md](book/AUTHORING.md) | マニュアルの執筆規約（文体・台詞部品・パートごとの決まり） |
| [.kiro/steering/](.kiro/steering/) | プロジェクト構造・技術スタック・開発ワークフロー・[ロードマップ](.kiro/steering/roadmap.md) |
| [CLAUDE.md](CLAUDE.md) | AI 開発支援と Kiro 仕様駆動開発の概要（プロジェクト指示・コマンド一覧） |
| [TEST_COVERAGE.md](TEST_COVERAGE.md) | テストと機能の対応 |

### クレート構成

| クレート | 役割 |
| -------- | ---- |
| [pasta_dsl](crates/pasta_dsl/README.md) | Pasta DSL の pest 文法・パーサと AST 定義 |
| [pasta_core](crates/pasta_core/README.md) | 言語に依存しないシーン・単語レジストリと、前方一致の検索表 |
| [pasta_lua](crates/pasta_lua/README.md) | トランスパイラ、ローダ、Lua VM（LuaJIT 2.1）とランタイムモジュール、トーク出力、デバッグ基盤 |
| [pasta_shiori](crates/pasta_shiori/README.md) | SHIORI DLL（`pasta.dll`）としての FFI 境界 |
| [pasta_lsp](crates/pasta_lsp/README.md) | 言語サーバ（VSCode 拡張が WASM で使う） |
| [pasta_check](crates/pasta_check/README.md) | ゴーストの配布物を作る CLI（更新ファイル・NAR） |
| [pasta_sample_ghost](crates/pasta_sample_ghost/README.md) | サンプルゴースト hello-pasta（入門ガイドの題材。公開しないクレート） |

データの流れ（詳しくは [内部設計の概要](https://ekicyou.github.io/pasta/internals/index.html)）:

```text
.pasta 辞書 → パース（pasta_dsl）→ AST → トランスパイル（pasta_lua）→ Lua コード
           → Lua VM で実行（pasta_lua）→ さくらスクリプト → SHIORI 応答（pasta_shiori）
```

依存の向きは `pasta_shiori` → `pasta_lua` → `pasta_dsl`・`pasta_core` です。

### クイックスタート

```bash
cargo build --workspace                                      # ビルド
cargo test --all                                             # テスト
cargo clippy --all-targets --workspace -- -D warnings        # lint
cargo build --release --target i686-pc-windows-msvc -p pasta_shiori   # SSP 用の pasta.dll（32bit）
```

前提: Rust 2024 edition / cargo。`pasta.dll` は Windows 用です。詳しいプロジェクト構造は [.kiro/steering/structure.md](.kiro/steering/structure.md) を参照してください。

---

## ライセンス

[LICENSE](LICENSE) ファイルを参照してください。
