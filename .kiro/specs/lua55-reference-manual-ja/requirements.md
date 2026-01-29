# Requirements Document

## Introduction

Lua 5.5公式リファレンスマニュアル（https://www.lua.org/manual/5.5/）を日本語に翻訳し、`crates/pasta_lua/doc/`に配置するドキュメンテーションプロジェクト。Lua言語仕様を日本語で参照できるようにすることを目的とする。

**独立性の原則**: 本翻訳はLua 5.5公式リファレンスの日本語版として完全に独立した文書であり、pasta_lua固有の用語・概念・リンクを一切含まない。

## Project Description (Input)
Lua 5.5 Reference Manual をフェッチして取り込み、日本語に訳したうえで、「pasta_lua/doc/」あたりに配置して欲しい。
マニュアルURLはこちら
https://www.lua.org/manual/5.5/contents.html

markdownか、HTMLかは任せますが、マークダウンの方が使い勝手がよいかな？リンクも確認して、マニュアル部分についてはすべて日本語化して欲しい。

## Requirements

### Requirement 1: ドキュメント構造とフォーマット

**Objective:** As a Lua開発者, I want Lua 5.5リファレンスマニュアルをMarkdown形式で参照できること, so that VS Codeやgitリポジトリ上で快適に閲覧・検索できる

#### Acceptance Criteria
1. The ドキュメントシステム shall 全マニュアルをMarkdown形式（`.md`）で提供する
2. The ドキュメントシステム shall `crates/pasta_lua/doc/lua55-manual/`ディレクトリに配置する
3. The ドキュメントシステム shall 目次ファイル（`README.md`または`index.md`）を提供する
4. The ドキュメントシステム shall 各セクション間のリンクをMarkdown相対リンクで実装する
5. When セクションが大きい場合, the ドキュメントシステム shall 章ごとに分割したファイル構成を採用する

### Requirement 2: マニュアルセクション網羅性

**Objective:** As a Lua開発者, I want 公式マニュアルの全セクションが日本語で参照できること, so that 言語仕様の全体像を把握できる

#### Acceptance Criteria
1. The ドキュメントシステム shall 以下の全章を含むこと：
   - 1 – イントロダクション
   - 2 – 基本概念（値と型、スコープ、エラー処理、メタテーブル、ガベージコレクション、コルーチン）
   - 3 – 言語仕様（字句規約、変数、文、式）
   - 4 – アプリケーションプログラムインターフェース（C API）
   - 5 – 補助ライブラリ
   - 6 – 標準ライブラリ（基本関数、coroutine、package、string、utf8、table、math、io、os、debug）
   - 7 – スタンドアロンLua
   - 8 – 前バージョンとの非互換性
   - 9 – Lua完全構文
2. The ドキュメントシステム shall 索引（Index）セクションを提供する

### Requirement 3: 翻訳品質

**Objective:** As a Lua開発者, I want 技術的に正確な日本語翻訳を参照できること, so that 実装時の理解に支障がない

**Translation Method (確定):** AI多段階品質改善方式
- Phase 1: AI一括翻訳（LLM使用）
- Phase 2: AI品質レビュー（用語統一、技術的正確性）
- Phase 3: AIブラッシュアップ（リンク整合性、表現改善）
- Phase 4: AI最終検証（全体整合性、用語対応表更新）

**用語対応表の方針 (確定):**
- 網羅的な用語対応表（100語以上）をAI翻訳Phase 1で初版作成
- Phase 2-4で段階的に洗練
- 後から人間が修正可能な原本として機能

#### Acceptance Criteria
1. The 翻訳 shall 技術用語を一貫して使用する（用語集の整備を含む）
2. The 翻訳 shall API名・関数名・予約語は原文のまま維持する（例: `lua_pushstring`, `local`, `function`）
3. The 翻訳 shall コード例は原文のまま維持する
4. The 翻訳 shall 曖昧さを避け、原文の意味を正確に伝える
5. Where 訳語に複数の選択肢がある場合, the ドキュメントシステム shall 用語対応表を提供する
6. The 翻訳プロセス shall AI多段階品質改善を実施し、人間レビューは不要とする

### Requirement 4: ナビゲーションと検索性

**Objective:** As a Lua開発者, I want 必要な情報に素早くアクセスできること, so that 開発効率が向上する

#### Acceptance Criteria
1. The ドキュメントシステム shall 階層的な目次構造を提供する
2. The ドキュメントシステム shall 各ページにパンくずリスト的なナビゲーションを含める
3. The ドキュメントシステム shall 関数・型の一覧ページを提供する（索引機能）
4. When 関連するセクションがある場合, the ドキュメントシステム shall 相互リンクを提供する
5. The ドキュメントシステム shall 各ドキュメントにセクションアンカー（見出しリンク）を設定する

### Requirement 5: メンテナンス性

**Objective:** As a ドキュメントメンテナ, I want ドキュメントの更新・管理が容易であること, so that 将来のLuaバージョン更新に対応できる

#### Acceptance Criteria
1. The ドキュメントシステム shall 原文バージョン（Lua 5.5）と翻訳日時を記録する
2. The ドキュメントシステム shall ライセンス情報（Lua license: MIT）を明記する
3. The ドキュメントシステム shall 翻訳元URLを各ファイルに記載する
4. The ドキュメントシステム shall ファイル構成をシンプルに保つ（1章1ファイルまたは論理的分割）
5. Where 将来の更新が予想される場合, the ドキュメントシステム shall 差分確認しやすい構造を採用する

### Requirement 6: ライセンス遵守

**Objective:** As a プロジェクト管理者, I want Luaライセンスに準拠した形でドキュメントを配布できること, so that 法的リスクを回避できる

#### Acceptance Criteria
1. The ドキュメントシステム shall Luaのライセンス（MIT License）の原文を含める
2. The ドキュメントシステム shall 翻訳が公式のものではなく、非公式な日本語翻訳であることを明記する
3. The ドキュメントシステム shall 原著作権表示（Copyright © 2020–2025 Lua.org, PUC-Rio）を保持する
4. The ドキュメントシステム shall pasta_lua固有の用語・概念・リンクを一切含まない（Lua 5.5翻訳として完全に独立）

## Non-Requirements (スコープ外)

以下は本仕様のスコープ外とする：

- HTML形式での出力生成
- 自動翻訳システムの構築
- 公式Luaサイトへの貢献・マージ
- Lua 5.4以前のバージョンのマニュアル翻訳
- オンラインホスティング・Web公開

## 想定ファイル構成

```
crates/pasta_lua/doc/lua55-manual/
├── README.md                    # 目次・イントロダクション
├── LICENSE.md                   # Luaライセンス
├── GLOSSARY.md                  # 用語対応表
├── 01-introduction.md           # 1章: イントロダクション
├── 02-basic-concepts.md         # 2章: 基本概念
├── 03-language.md               # 3章: 言語仕様
├── 04-c-api.md                  # 4章: C API
├── 05-auxiliary-library.md      # 5章: 補助ライブラリ
├── 06-standard-libraries.md     # 6章: 標準ライブラリ（分割可）
├── 07-standalone.md             # 7章: スタンドアロンLua
├── 08-incompatibilities.md      # 8章: 非互換性
├── 09-complete-syntax.md        # 9章: 完全構文
└── index.md                     # 索引（関数・型一覧）
```
