<img src="/img/pasta.svg" alt="Pasta logo" width="120" align="left" style="margin-right: 1em;">

# pasta
Memories of pasta twine together—now and then a knot, yet always a delight.

<br clear="both">

**pasta** は「伺か」などのデスクトップマスコット、あるいはノベルゲーム向けの対話スクリプトエンジン／SHIORI.DLL です。
日本語フレンドリーな全角マーカー、前方一致によるランダム選択、Lua ランタイムによる拡張性を特徴とします。

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

## 📖 ゴーストを作りたい方へ — 利用者マニュアル

### 👉 **[pasta 利用者マニュアル](https://ekicyou.github.io/pasta/)**

Pasta DSL の文法、Lua API/コーディング、ゼロからの入門チュートリアルを、**日本語で検索可能な単一サイト**にまとめています。
ゴースト作者の方は、まずこちらからどうぞ。

> 以前 `GRAMMAR.md` や `doc/spec/` に分散していた利用者向けの知識は、上記マニュアルへ集約しました。
> この README は、pasta **本体の開発**に関わる方のための入口です。

---

## 🛠️ 開発者・コントリビュータの方へ

pasta 本体（パーサ・トランスパイラ・ランタイム）の開発に関わる方向けの入口です。

### ドキュメント

| ドキュメント | 説明 |
| ------------ | ---- |
| [doc/spec/](doc/spec/) | Pasta DSL 正式言語仕様（章別分割・実装判断の**権威的ソース**） |
| [GRAMMAR.md](GRAMMAR.md) | 文法クイックリファレンス（開発時の手元参照用） |
| [CLAUDE.md](CLAUDE.md) | AI 開発支援・Kiro 仕様駆動開発の概要（プロジェクト指示・コマンド一覧） |
| [SOUL.md](SOUL.md) | コアバリュー・設計原則 |
| [.kiro/steering/](.kiro/steering/) | プロジェクト構造・技術スタック・開発ワークフローのステアリング |

### クレート構成

| クレート | 役割 |
| -------- | ---- |
| [pasta_dsl](crates/pasta_dsl/README.md) | DSL パーサー（Pest PEG → AST） |
| [pasta_core](crates/pasta_core/README.md) | レジストリ・ユーティリティ（言語非依存層） |
| [pasta_lua](crates/pasta_lua/README.md) | Lua トランスパイラ・ランタイム（2-pass） |
| [pasta_shiori](crates/pasta_shiori/README.md) | SHIORI DLL 統合 |

レイヤー構成:

```
Engine (上位API) → Cache/Loader
    ↓
Transpiler (2pass) ← Parser (pasta_dsl, Pest)
    ↓
Runtime (LuaJIT VM) → IR Output (ScriptEvent)
```

### クイックスタート

```bash
cargo build --workspace   # ビルド
cargo test --workspace    # テスト
```

前提: Rust 2024 edition / cargo。詳しいプロジェクト構造は [.kiro/steering/structure.md](.kiro/steering/structure.md) を参照してください。

### エディタ拡張

- [Pasta DSL VSCode 拡張](https://marketplace.visualstudio.com/items?itemName=ekicyou.pasta-vscode) — 構文ハイライト・LSP 連携

---

## ライセンス

[LICENSE](LICENSE) ファイルを参照してください。
