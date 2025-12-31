# 調査・設計判断ログ

---
**目的**: 発見事項、アーキテクチャ調査、設計判断の根拠を記録する。

**用途**:
- 発見フェーズの調査活動と成果をログ化
- `design.md` に詳細すぎる設計判断のトレードオフを記録
- 将来の監査や再利用のための参照とエビデンスを提供
---

## 概要
- **機能**: `pasta-lua-unit-test-framework`
- **発見スコープ**: Extension（既存システム拡張）
- **主要な発見事項**:
  1. Busted は Lua デファクトスタンダード テストフレームワーク（1.6k★）で、RSpec/Mocha 風の自然な記述が可能
  2. pasta_lua は既に mlua クレート統合済み、Rust 統合テストのみ存在し Lua スクリプト層テストが欠落
  3. Busted は luarocks 経由インストール可能、GitHub Actions 統合も対応済み

## 調査ログ
トピックごとに主要な調査ステップと成果を記録。

### テストフレームワークの選定
- **背景**: Lua スクリプト層のユニットテストフレームワークとして、デファクトスタンダードで保守性の高いものを選定する必要があった
- **調査ソース**:
  - [Busted GitHub](https://github.com/lunarmodules/busted) - 1.6k★、75+ contributors
  - [Busted 公式ドキュメント](https://lunarmodules.github.io/busted/)
  - LuaUnit, lua-TestMore なども検討したが、現在非推奨またはアーカイブ済み
- **発見内容**:
  - **Busted**: Lua 5.1, LuaJIT, Lua 5.5 対応、Moonscript もサポート
  - **RSpec/Mocha 風 BDD スタイル**: `describe`, `it`, `before_each`, `after_each` で自然な記述
  - **Mock/Spy 機能**: `spy.on()`, `stub()`, `mock()` で関数呼び出し検証可能
  - **CI/CD 統合**: GitHub Actions (`lunarmodules/busted@v0`)、Docker イメージ提供
  - **出力形式**: UTF/Plain Terminal、JSON、TAP（CI サーバー統合用）
  - **多言語対応**: 日本語含む 11 言語対応
- **影響**:
  - Busted を pasta_lua テストフレームワークとして採用
  - luarocks 経由でインストール可能、CI/CD 統合も容易

### pasta_lua 既存構成の分析
- **背景**: pasta_lua クレートの現在の構造と Rust 統合テストの状況を把握
- **調査内容**:
  - `crates/pasta_lua/Cargo.toml`: mlua クレート（ekicyou fork）導入済み
  - `crates/pasta_lua/src/`: transpiler, code_generator, context, error, config, string_literalizer
  - `crates/pasta_lua/tests/`: transpiler_integration_test.rs（Rust 統合テスト）、fixtures/ に sample.pasta, sample.lua
- **発見内容**:
  - Rust 側の統合テストのみ存在、Lua スクリプト層そのものの単体テストなし
  - transpiler_integration_test.rs は Rust から Lua コード生成を検証（824 行）
  - Lua VM 実行層のテストが欠落している
- **影響**:
  - Lua スクリプト層テストを tests/ に Busted ベースで追加
  - Rust 統合テストと Lua ユニットテストの役割を分離

### Busted インストール・セットアップ方式
- **背景**: Rust プロジェクトに Lua テストフレームワークを統合する方法を検討
- **調査ソース**:
  - [Busted 公式 Installation](https://lunarmodules.github.io/busted/#usage)
  - [GitHub Actions 統合例](https://github.com/lunarmodules/busted#use-as-a-ci-job)
- **発見内容**:
  - **方式 A: luarocks 経由（ローカル開発）**
    ```bash
    luarocks install busted
    busted spec/
    ```
  - **方式 B: Docker コンテナ（CI/CD）**
    ```yaml
    - name: Run Busted
      uses: lunarmodules/busted@v0
      with:
        args: --verbose
    ```
  - **設定ファイル**: `.busted` ファイルでタスク定義可能（verbose, coverage, tags など）
  - **テストファイル命名**: デフォルト `*_spec.lua`、`--pattern` で変更可能
- **影響**:
  - ローカル開発では luarocks、CI/CD では GitHub Actions を使用
  - `.busted` 設定ファイルを crates/pasta_lua/ に配置
  - テストファイル命名規約を `*_spec.lua` に統一

### テストディレクトリ構成
- **背景**: pasta プロジェクトの既存テスト構成に合わせた Lua テストレイアウトを設計
- **調査内容**:
  - `.kiro/steering/structure.md` のテスト構成規約を確認
  - pasta_rune の tests/ 構成を参照
- **発見内容**:
  - Rust 側: `tests/<feature>_test.rs`、`tests/fixtures/*.pasta`
  - Lua 側: `tests/lua_specs/*_spec.lua` でディレクトリ分離
  - 共通フィクスチャ: `tests/fixtures/` を共有可能
- **影響**:
  - `crates/pasta_lua/tests/lua_specs/` ディレクトリを新規作成
  - Busted テストと Rust 統合テストを明確に分離
  - `.busted` で `ROOT = {"tests/lua_specs"}` を指定

### サンプルテスト範囲
- **背景**: 要件 R2「サンプルテスト作成」で最小 3 個のテスト実装が求められている
- **調査内容**:
  - transpiler_integration_test.rs の既存テストケースを分析
  - Lua 側で検証すべき機能を特定
- **発見内容**:
  - **Transpiler 基本動作**: Pasta AST → Lua コード変換の正常系
  - **Code Generator**: シーン定義、アクター定義の Lua 出力検証
  - **Context 管理**: トランスパイル中の状態管理（変数スコープなど）
  - **エラーハンドリング**: 不正な AST を渡した場合のエラー検出
- **影響**:
  - 以下 3 つのサンプルテストを作成:
    1. `transpiler_spec.lua`: 基本変換動作
    2. `code_generator_spec.lua`: コード生成機能
    3. `context_spec.lua`: コンテキスト管理

## アーキテクチャパターン評価
検討した候補パターンまたはアプローチをリスト化。

| オプション | 説明 | 強み | リスク/制約 | 備考 |
|-----------|------|------|------------|------|
| luarocks + ローカル Busted | luarocks で Busted をローカルインストール | 開発環境で即座にテスト実行可能 | 開発者が luarocks セットアップ必要 | ローカル開発推奨 |
| Docker コンテナ Busted | `ghcr.io/lunarmodules/busted:latest` 使用 | 環境差異なし、CI/CD 統合容易 | ローカル開発では Docker 起動オーバーヘッド | CI/CD のみで使用 |
| GitHub Actions 統合 | `lunarmodules/busted@v0` アクション使用 | 最小設定で CI 統合完了 | GitHub Actions 限定 | CI/CD で採用 |

## 設計判断
大きなトレードオフを伴う主要判断を記録。

### 判断: Busted テストディレクトリを Rust tests/ 配下に配置
- **背景**: Rust プロジェクトに Lua テストを統合する際、ディレクトリ配置を決定する必要があった
- **検討した代替案**:
  1. `tests/lua_specs/` - Rust tests/ 配下にサブディレクトリ作成
  2. `lua_tests/` - ワークスペースルートに独立ディレクトリ作成
  3. `crates/pasta_lua/lua/tests/` - クレート内に独立した Lua ディレクトリ
- **選択したアプローチ**: オプション 1 - `tests/lua_specs/`
- **根拠**:
  - pasta プロジェクトのステアリングルール（structure.md）に従い、テストは `tests/` 配下に集約
  - Rust 統合テストと Lua ユニットテストを明確に分離しつつ、テスト関連ファイルを一箇所に集約
  - `.busted` 設定ファイルで `ROOT = {"tests/lua_specs"}` 指定で Lua テストのみ実行可能
- **トレードオフ**:
  - **メリット**: テストファイルの場所が統一され、探しやすい
  - **デメリット**: Rust と Lua で異なる実行コマンド必要（cargo test vs busted）
- **フォローアップ**: `.busted` 設定で verbose, pattern 設定を明示

### 判断: サンプルテストを 3 個に限定
- **背景**: 要件 R2 で「最小 3 個のテスト実装」が求められているが、全機能カバーも可能
- **検討した代替案**:
  1. 最小 3 個（transpiler, code_generator, context）
  2. 全モジュールカバー（5+ 個、error, config, string_literalizer 含む）
- **選択したアプローチ**: オプション 1 - 最小 3 個
- **根拠**:
  - Phase 0（基盤構築）の目標は「テストフレームワーク導入」であり、全機能カバーは Phase 1 以降
  - 3 個のサンプルテストでテストパターンを確立し、後続開発者が拡張可能にする
  - Lua 側のビジネスロジック実装は「非要件」として明示されている
- **トレードオフ**:
  - **メリット**: 導入コストを最小化、テストパターンの確立に集中
  - **デメリット**: 初期段階での全機能カバレッジは低い
- **フォローアップ**: README.md でテスト追加方法を記載し、拡張性を確保

### 判断: CI/CD 統合を Optional（P2）として扱う
- **背景**: 要件 R4 で CI/CD 統合が Optional として定義されている
- **検討した代替案**:
  1. P1（必須）: GitHub Actions 統合を必須とする
  2. P2（Optional）: ローカル実行のみ必須、CI は後続フェーズ
- **選択したアプローチ**: オプション 2 - P2（Optional）
- **根拠**:
  - ローカル開発環境でのテスト実行が最優先（開発者の即時フィードバック）
  - GitHub Actions 統合は pasta プロジェクト全体の CI/CD 戦略に依存する
  - 設計フェーズで CI 統合パスを準備しておけば、実装フェーズで追加可能
- **トレードオフ**:
  - **メリット**: 導入スコープを絞り、確実にローカル環境を整備
  - **デメリット**: CI で自動テスト実行されない期間が発生
- **フォローアップ**: タスクフェーズで「CI 統合タスク（Optional）」を明記

## リスク & 軽減策
- **リスク 1: luarocks インストール未対応環境** — 軽減策: README.md で luarocks セットアップ手順を明記、Docker コンテナ代替案を提示
- **リスク 2: Lua と Rust テストコマンドの混在** — 軽減策: `.busted` 設定で明確にディレクトリ分離、README でコマンド例を記載
- **リスク 3: Lua VM 実行層の検証不足** — 軽減策: サンプルテストで基本パターンを確立、後続フェーズで拡張可能な構造を提供

## 参考文献
公式ドキュメント、標準、ADR、内部ガイドラインへのリンクを提供。

- [Busted 公式ドキュメント](https://lunarmodules.github.io/busted/) - テストフレームワーク全体像
- [Busted GitHub Repository](https://github.com/lunarmodules/busted) - ソースコード、Issues
- [luassert Library](https://github.com/lunarmodules/luassert) - アサーションライブラリ
- [luarocks](https://luarocks.org/) - Lua パッケージマネージャ
- [GitHub Actions - Busted Action](https://github.com/marketplace/actions/lua-busted) - CI/CD 統合
