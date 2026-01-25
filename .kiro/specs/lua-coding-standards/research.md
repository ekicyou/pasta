# リサーチ & 設計決定ログ

## サマリー
- **フィーチャー**: `lua-coding-standards`
- **ディスカバリースコープ**: Extension（既存コードベースへの規約策定・リファクタリング）
- **主要な発見事項**:
  1. 既存のLuaコードは既に高品質で、モジュールパターンとEmmyLua注釈が部分的に導入済み
  2. 命名規約はほぼ統一されているが、クラスメタテーブルにPascalCaseが使用されている（`WordBuilder`等）
  3. クラス実装でコロン構文が使用されており、`MODULE_IMPL`パターンへの移行が必要
  4. luacheck v1.2.0は既に`scriptlibs/`に導入済み、設定ファイルの作成が必要

## リサーチログ

### 既存のLuaモジュール構造分析

- **コンテキスト**: 8つのLuaファイル（`scripts/pasta/*.lua`）の現状パターンを調査
- **参照ソース**: 
  - [scripts/pasta/actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua)
  - [scripts/pasta/act.lua](../../../crates/pasta_lua/scripts/pasta/act.lua)
  - [scripts/pasta/ctx.lua](../../../crates/pasta_lua/scripts/pasta/ctx.lua)
- **発見事項**:
  - 全ファイルで`@module`注釈が使用済み
  - `require`文がファイル先頭に配置（準拠）
  - モジュールテーブルはUPPER_CASE（`ACTOR`, `ACT`, `CTX`など）
  - `return MOD`がファイル末尾に配置（準拠）
- **影響**: モジュール構造規約は既に良好。ドキュメント化が主な作業

### クラスパターンの現状

- **コンテキスト**: Rust風クラス設計パターンへの移行を検討
- **参照ソース**: gap-analysis.md、actor.lua、ctx.lua
- **発見事項**:
  - `CTX.new()`パターンは既に使用中（良好）
  - `ACT.new()`も使用中だが、`__index = ACT`で自己参照
  - `WordBuilder`、`ActorWordBuilder`がPascalCaseで、`_IMPL`サフィックスなし
  - メソッド定義でコロン構文（`function ACT:__index(key)`）が使用されている
- **影響**: 
  - PascalCaseを`_IMPL`サフィックスに変更必要（`WORD_BUILDER_IMPL`等）
  - 実装内のコロン構文をドット構文+明示的selfに変換必要
  - 呼び出し側のコロン構文は許容（利便性）

### EmmyLua型注釈の互換性

- **コンテキスト**: sumneko.lua（Lua Language Server）との互換性確認
- **参照ソース**: VS Code拡張機能リスト、既存コード
- **発見事項**:
  - sumneko.lua (ID: `sumneko.lua`) がワークスペースにインストール済み
  - EmmyLuaフォーマットは正式サポート: `@module`, `@class`, `@param`, `@return`, `@field`
  - `@vararg`は非標準→`@param ... type`が正式
  - 可変長引数: `--- @param ... string`形式に統一必要
- **影響**: 型注釈規約はEmmyLua標準に従う。2箇所の`@vararg`を修正必要

### luacheckの設定

- **コンテキスト**: Lua静的解析ツールの設定要件
- **参照ソース**: 
  - luacheck公式ドキュメント
  - [scriptlibs/luacheck/](../../../crates/pasta_lua/scriptlibs/luacheck/)
- **発見事項**:
  - luacheck v1.2.0が`crates/pasta_lua/scriptlibs/luacheck/`に配置済み
  - `.luacheckrc`設定ファイルが未作成
  - グローバル変数（`PASTA`, `ACTOR`など）のホワイトリスト必要
  - 日本語識別子のサポート確認必要
- **影響**: `.luacheckrc`設定ファイルを作成し、pastaプロジェクト用にカスタマイズ

### テストフレームワークの構造

- **コンテキスト**: lua_testフレームワークの使用パターン
- **参照ソース**: 
  - [scriptlibs/lua_test/test.lua](../../../crates/pasta_lua/scriptlibs/lua_test/test.lua)
  - [scriptlibs/lua_test/expect.lua](../../../crates/pasta_lua/scriptlibs/lua_test/expect.lua)
- **発見事項**:
  - BDD風テストフレームワーク: `describe`, `it`, `expect`
  - Jest風アサーション: `expect(value).to_equal(expected)`
  - テストファイル命名: 現在`*_spec.lua`→`*_test.lua`に統一予定
- **影響**: テスト規約をドキュメント化。ファイル名をRust（`*_test.rs`）と統一

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク/制約 | 備考 |
|------------|------|------|-------------|------|
| ドキュメントファースト | 規約作成→リファクタリング | 指針明確、レビュー可能 | 2段階作業 | **推奨** |
| 一括実装 | 規約とリファクタを同時 | 1回で完了 | 手戻りリスク | 非推奨 |
| ハイブリッド | 骨格作成→1ファイル検証→確定 | 早期検証 | パイロット選定必要 | 代替案 |

## 設計決定

### 決定: ドキュメントファーストアプローチ

- **コンテキスト**: 規約策定とリファクタリングの実施順序
- **検討した選択肢**:
  1. ドキュメントファースト — 規約作成後にリファクタリング
  2. 一括実装 — 同時実施
  3. ハイブリッド — 1ファイルでパイロット検証
- **選択したアプローチ**: ドキュメントファースト
- **根拠**: 
  - 規約が先にあることでリファクタリングの指針が明確
  - レビュー可能な中間成果物を生成
  - 規約承認後のコード修正で手戻りを削減
- **トレードオフ**: 2段階作業で期間は長いが、品質と確実性を優先
- **フォローアップ**: リファクタリング時に規約の実用性を検証

### 決定: `_IMPL`サフィックス統一

- **コンテキスト**: クラスメタテーブルの命名規約
- **検討した選択肢**:
  1. PascalCase維持（`WordBuilder`）
  2. `_IMPL`サフィックス（`WORD_BUILDER_IMPL`）
  3. `_mt`サフィックス（`WORD_BUILDER_mt`）
- **選択したアプローチ**: `_IMPL`サフィックス
- **根拠**: 
  - モジュールテーブルとクラス実装メタテーブルを明確に区別
  - UPPER_CASE統一でコードベースの一貫性向上
  - Rust影響の設計思想（明示性重視）と整合
- **トレードオフ**: 既存コードの修正が必要だが、長期的な保守性を優先

### 決定: 明示的self + ドット構文（実装側）

- **コンテキスト**: クラスメソッド定義の構文
- **検討した選択肢**:
  1. コロン構文維持（`function MODULE:method()`）
  2. ドット構文 + 明示的self（`function MODULE_IMPL.method(self)`）
- **選択したアプローチ**: ドット構文 + 明示的self（実装側）、コロン構文許可（呼び出し側）
- **根拠**: 
  - Rust影響: 実装側での明示性と利用側での利便性を両立
  - 暗黙のselfバグを防止
  - 呼び出し側は`instance:method(arg)`で自然な記述が可能
- **トレードオフ**: 実装側のコード量がわずかに増加するが、バグ予防効果を優先

## リスク & 緩和策

- **リスク1**: リファクタリングによるリグレッション
  - **緩和策**: 既存テスト（lua_specs）を全て実行して動作確認。機能変更なし（リファクタリングのみ）を徹底

- **リスク2**: 日本語識別子のluacheckエラー
  - **緩和策**: `.luacheckrc`で適切な設定を行い、日本語識別子を許可

- **リスク3**: 規約の過剰な複雑化
  - **緩和策**: 既存パターンを尊重し、必要最小限の変更にとどめる

## リファレンス

- [EmmyLua Annotations](https://github.com/EmmyLua/EmmyLua-LanguageServer/wiki/Annotations) — 型注釈の正式フォーマット
- [luacheck Documentation](https://luacheck.readthedocs.io/) — 静的解析設定リファレンス
- [sumneko.lua](https://github.com/LuaLS/lua-language-server) — Lua Language Server（EmmyLua互換）
- [gap-analysis.md](gap-analysis.md) — 現状調査と要件ギャップ分析
