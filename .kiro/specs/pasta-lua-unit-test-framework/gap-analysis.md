# 実装ギャップ分析：pasta_lua ユニットテストフレームワーク導入

## 分析サマリー

- **スコープ**: pasta_lua クレートに Busted テストフレームワークを導入し、Lua スクリプト層のユニットテスト環境を構築
- **主要課題**:
  - 現在 Rust 統合テストのみ存在し、Lua レイヤー単体テストが完全に欠落
  - Lua と Rust の 2 つのテストランナーを並行運用する必要あり
  - Lua モジュール（require 可能な API）が未実装で、テストから呼び出す実体が不在
  - luarocks/Busted の開発者ローカル環境セットアップが必要
- **推奨アプローチ**: **オプション C（ハイブリッドアプローチ）** - Phase 0 では新規ディレクトリとドキュメント追加のみ、Phase 1 以降で Lua モジュール実装と統合
- **複雑度 & リスク**: **S（1-3 日）/ 低リスク** - テストフレームワーク導入のみで既存コード変更なし、Lua モジュール実装は Phase 1 以降で対応

---

## 1. 現状調査

### ドメイン関連アセット

**ディレクトリレイアウト**:
```
crates/pasta_lua/
├── Cargo.toml                  # mlua 依存あり
├── src/
│   ├── lib.rs                  # 公開 API（LuaTranspiler, Config, Context, Error）
│   ├── transpiler.rs           # LuaTranspiler - メイン変換 API
│   ├── code_generator.rs       # LuaCodeGenerator - Lua コード生成
│   ├── context.rs              # TranspileContext - 状態管理
│   ├── config.rs               # TranspilerConfig - 設定
│   ├── error.rs                # TranspileError - エラー型
│   └── string_literalizer.rs   # StringLiteralizer - 文字列リテラル最適化
├── scripts/                    # ✅ 自作Luaコード・スクリプト層
│   ├── hello.lua               # サンプルスクリプト
│   ├── transpiler.lua          # 自作実装
│   ├── code_generator.lua      # 自作実装
│   ├── context.lua             # 自作実装
│   ├── helpers/                # ヘルパー関数
│   │   ├── string_utils.lua
│   │   └── table_utils.lua
│   ├── examples/               # 使用例スクリプト
│   ├── init.lua                # エントリーポイント
│   └── README.md               # スクリプト層ドキュメント
├── scriptlibs/                 # ✅ 外部Luaライブラリ専用
│   ├── busted/                 # Busted テストフレームワーク
│   └── (外部ライブラリ)
└── tests/
    ├── transpiler_integration_test.rs  # Rust 統合テスト (824 行)
    ├── lua_specs/              # Lua ユニットテスト
    └── fixtures/
        ├── sample.pasta        # テスト用 Pasta スクリプト
        └── sample.lua          # 期待される Lua 出力 (241 行)
```

**再利用可能なコンポーネント**:
- **LuaTranspiler**: Pasta AST → Lua コード変換（Rust FFI 経由のみアクセス可能）
- **TranspileContext**: シーン/単語レジストリ、変数スコープ管理
- **fixtures/**: 既存の sample.pasta/sample.lua がテストパターンとして流用可能

**主要アーキテクチャパターン**:
- **Rust 統合テスト**: transpiler_integration_test.rs で Rust から LuaTranspiler を呼び出し、出力検証
- **テストヘルパー関数**: `get_global_scene_scopes()`, `get_actor_scopes()` で AST 抽出
- **テスト命名規則**: `test_transpile_<feature>_<aspect>` 形式

### 規約抽出

**命名規約**:
- **Rust テストファイル**: `<feature>_test.rs`（pasta_rune でも同様）
- **Fixture ファイル**: `tests/fixtures/*.pasta`, `*.lua`
- **テスト関数**: `#[test] fn test_<feature>_<case>()`

**レイヤー構成**:
- **src/**: Rust ライブラリコード（Lua VM と通信しない）
- **tests/**: Rust 統合テスト（`#[test]` マクロ使用）
- **fixtures/**: テストデータ（Pasta/Lua サンプル）

**依存関係パターン**:
- **Workspace 依存**: `Cargo.toml` で `mlua.workspace = true` を使用
- **テスト専用依存**: `[dev-dependencies]` セクションで tempfile など
- **外部ツール依存**: Rust エコシステム外のツール（luarocks）は未使用

**テスト配置アプローチ**:
- pasta_rune: `crates/pasta_rune/tests/` に 27 個の統合テストファイル
- 全テスト Rust ベース、Lua/Rune スクリプト層の単体テストなし

### 統合面

**データモデル/スキーマ**:
- **AST**: pasta_core の `PastaFile`, `GlobalSceneScope`, `ActorScope` など
- **Registry**: `SceneRegistry`, `WordDefRegistry`（TranspileContext 内）
- **Lua 出力**: 文字列として生成、Lua VM で実行可能（mlua 経由）

**API クライアント**:
- なし（外部 API 統合なし）

**認証メカニズム**:
- なし

---

## 2. 要件実現性分析

### 技術的必要事項

**R1: テストスイート導入**
- **データモデル**: なし（テストフレームワーク導入のみ）
- **API/サービス**: Busted CLI、luarocks パッケージマネージャ
- **UI/コンポーネント**: なし
- **ビジネスルール**: `.busted` 設定ファイルで ROOT, pattern 指定
- **非機能要件**:
  - パフォーマンス: Rust ビルド不要で高速フィードバック
  - セキュリティ: luarocks 公式パッケージ使用
  - スケーラビリティ: テストファイル数増加に対応
  - 信頼性: Busted の豊富な実績（1.6k★）

**R2: サンプルテスト作成**
- **データモデル**: なし（テストコードのみ）
- **API/サービス**: Lua モジュール（require 可能な API）→ **Missing**
- **UI/コンポーネント**: なし
- **ビジネスルール**: describe/it/assert パターン（Busted BDD スタイル）
- **非機能要件**: テスト実行速度、保守性

**R3: ローカル実行環境**
- **データモデル**: なし
- **API/サービス**: luarocks インストール、busted コマンド
- **UI/コンポーネント**: なし
- **ビジネスルール**: README.md でセットアップ手順提供
- **非機能要件**: 開発者エクスペリエンス、セットアップ容易性

**R4: CI/CD 統合（Optional）**
- **データモデル**: なし
- **API/サービス**: GitHub Actions, `lunarmodules/busted@v0` アクション
- **UI/コンポーネント**: なし
- **ビジネスルール**: ワークフロー YAML 定義
- **非機能要件**: CI 実行速度、Docker コンテナ起動オーバーヘッド

### ギャップ & 制約

**Missing（実装欠落）**:
1. **Lua モジュール API（R2 の前提条件）**
   - **問題**: pasta_lua は現在 Rust FFI のみでアクセス可能、Lua から `require("pasta_lua")` 不可
   - **影響**: サンプルテスト（transpiler_spec.lua など）が実体を持てない
   - **対応**: Phase 1 で Lua モジュール実装（mlua の `UserData` / `Module` 使用）

2. **`.busted` 設定ファイル**
   - **問題**: 未作成
   - **影響**: テスト実行コマンドが不明瞭
   - **対応**: Phase 0 で作成

3. **tests/lua_specs/ ディレクトリ**
   - **問題**: 未作成
   - **影響**: Lua テストファイルの配置場所なし
   - **対応**: Phase 0 で作成

4. **README.md (tests/)**
   - **問題**: 未作成
   - **影響**: 開発者がセットアップ方法を知らない
   - **対応**: Phase 0 で作成

5. **GitHub Actions ワークフロー**
   - **問題**: 未作成（Optional）
   - **影響**: CI で自動テスト実行されない
   - **対応**: Phase 0 では実装しない選択肢あり

**Unknown（調査必要）**:
- なし（すべての技術選定は research.md で完了）

**Constraint（既存制約）**:
1. **Rust 中心のテストエコシステム**
   - **制約**: pasta プロジェクト全体が Rust ベース、Lua テストは異質
   - **影響**: `cargo test` で Lua テストが実行されない、別コマンド必要
   - **対応**: README.md で明示、CI ワークフローで両方実行

2. **mlua の Lua 5.4 依存**
   - **制約**: `mlua.workspace` で `lua54` feature 固定
   - **影響**: Busted も Lua 5.4 対応必要（Busted は Lua 5.1+ 対応済みで問題なし）
   - **対応**: なし（既に互換性あり）

3. **pasta_rune パターンとの差異**
   - **制約**: pasta_rune は Rune VM スクリプト層テストなし
   - **影響**: pasta_lua が先駆的事例となり、パターン確立の責任
   - **対応**: README.md で再利用可能なパターンを明示

### 複雑度シグナル

**分類**: **Simple（シンプル）**
- テストフレームワーク導入のみ、外部統合なし
- アルゴリズムロジックなし、ワークフローなし
- 既存コードへの影響なし（新規追加のみ）

**ただし注意点**:
- Lua モジュール実装は Medium 複雑度（mlua の UserData/Module 理解必要）
- Phase 0 ではモジュール実装を除外しているため Simple

---

## 3. 実装アプローチオプション

### オプション A: 既存コンポーネント拡張

**適用対象**: なし

**理由**: 
- 既存の Rust 統合テストを拡張しても Lua レイヤー単体テストは実現不可
- LuaTranspiler 自体は変更不要（テストフレームワーク導入のみ）

### オプション B: 新規コンポーネント作成

**適用対象**: 完全新規導入

**拡張するファイル/モジュール**: なし

**新規作成の根拠**:
- Lua テストフレームワークは Rust エコシステム外の完全新規ツール
- 既存の Rust 統合テストと責務が異なる（Rust FFI vs Lua スクリプト層）
- テストディレクトリ分離が明確（`tests/lua_specs/` vs `tests/*.rs`）

**統合ポイント**:
- **ローカル開発**: `busted` コマンド（Rust とは独立）
- **CI/CD**: GitHub Actions で `cargo test` と `busted` を両方実行
- **ドキュメント**: README.md で両者の使い分けを説明

**責務境界**:
- **Rust 統合テスト**: Rust から LuaTranspiler を呼び出し、出力検証（E2E）
- **Lua ユニットテスト**: Lua モジュール API の動作を Lua 内で検証（単体）
- **データフロー**: 両者は独立、共有フィクスチャ（`fixtures/*.pasta`）のみ共用可能

**トレードオフ**:
- ✅ Rust と Lua テストの責務が明確に分離
- ✅ Lua VM 実行層の品質保証が可能
- ✅ Rust ビルド不要で高速フィードバック
- ❌ 2 つのテストランナーを管理する必要（`cargo test` + `busted`）
- ❌ 開発者が luarocks セットアップ必要
- ❌ CI で 2 つのコマンド実行が必要

### オプション C: ハイブリッドアプローチ（推奨）

**組み合わせ戦略**:
- **Phase 0**: 新規ファイル/ディレクトリのみ作成（Lua モジュール実装なし）
  - `.busted` 設定ファイル
  - `tests/lua_specs/` ディレクトリ
  - `tests/README.md`
  - サンプルテストファイル骨組み（pending 状態）
- **Phase 1**: Lua モジュール実装と統合
  - mlua の `UserData`/`Module` で LuaTranspiler を Lua から呼び出し可能に
  - サンプルテストを実装レベルに拡充
- **Phase 2 以降**: CI/CD 統合、全機能カバレッジ拡充

**段階的実装**:
1. **Phase 0（基盤確立、1-2 日）**:
   - luarocks/Busted インストール手順を README.md に記載
   - `.busted` 設定ファイルで ROOT/pattern 定義
   - `tests/lua_specs/` ディレクトリ作成
   - transpiler_spec.lua, code_generator_spec.lua, context_spec.lua を pending 状態で作成
   - **ゴール**: `busted --list` でテストファイルが認識される状態
2. **Phase 1（Lua モジュール実装、3-5 日）**:
   - mlua の `UserData` で LuaTranspiler をラップ
   - `require("pasta_lua")` で Lua からアクセス可能に
   - サンプルテストの pending を解除し、実装
   - **ゴール**: `busted --verbose` でテストが実行され、成功する状態
3. **Phase 2（CI/CD 統合、1 日）**:
   - GitHub Actions ワークフローで `busted` 実行
   - テスト失敗時に CI が失敗するように設定
   - **ゴール**: PR で自動テスト実行

**リスク軽減**:
- **Phase 0**: 既存コード変更なし、リスクゼロ
- **Phase 1**: Lua モジュール実装が失敗しても Phase 0 の成果物は残る
- **Phase 2**: CI 統合が不要なら省略可能（Optional）

**トレードオフ**:
- ✅ 段階的に価値を提供、各フェーズで成果確認可能
- ✅ リスク分散、早期失敗検出
- ✅ Phase 0 で基盤確立、後続開発者が拡張可能
- ❌ 複数フェーズの計画・管理が必要
- ❌ Phase 0 単独では実行可能なテストなし（pending のみ）

---

## 4. 実装複雑度 & リスク

### 工数見積もり

**S（1-3 日）**
- **Phase 0 のみ**: 新規ファイル作成、ドキュメント整備、pending テスト骨組み
- **根拠**:
  - 既存コード変更なし
  - `.busted` 設定ファイルは 10 行程度
  - README.md は既存パターン参照可能
  - サンプルテスト骨組みは describe/it ブロックのみ

**M（3-7 日）**
- **Phase 0 + Phase 1**: 上記 + Lua モジュール実装
- **根拠**:
  - mlua の UserData/Module API 学習が必要
  - LuaTranspiler の Lua ラッパー実装
  - サンプルテストを実装レベルに拡充
  - デバッグとトラブルシューティング

**L（1-2 週間）**
- **Phase 0 + Phase 1 + Phase 2**: 上記 + CI/CD 統合 + 全機能カバレッジ
- **根拠**:
  - GitHub Actions ワークフロー作成
  - 複数モジュール（error, config, string_literalizer）のテスト追加
  - CI デバッグとパフォーマンス最適化

### リスク評価

**低リスク**
- **根拠**:
  - Phase 0 は既存コード変更なし、リグレッション可能性ゼロ
  - Busted は実績豊富（1.6k★、75+ contributors）
  - luarocks は Lua 公式パッケージマネージャ、安定性高い
  - pasta_lua の API 設計は明確、Lua ラッパー実装は定型パターン

**ただし注意点**:
- **Phase 1 の Lua モジュール実装**: mlua の UserData API 習熟が必要 → **Medium リスク**
  - 軽減策: mlua の公式ドキュメント参照、既存事例（mlua examples）を学習
- **開発者環境セットアップ**: luarocks 未経験者が躓く可能性 → **Low リスク**
  - 軽減策: README.md で主要 OS（macOS, Linux, Windows）の手順を明記
- **CI/CD 統合**: Docker コンテナ起動でビルド時間増加 → **Low リスク**
  - 軽減策: Phase 2 を Optional とし、必要に応じて実装

---

## 5. 要件-アセットマップ

| 要件 | 既存アセット | ギャップ | 推奨アプローチ |
|------|-------------|---------|---------------|
| **R1: テストスイート導入** | - Cargo.toml（mlua 依存あり）<br>- tests/ ディレクトリ | **Missing**: `.busted` 設定ファイル<br>**Missing**: tests/lua_specs/ ディレクトリ<br>**Constraint**: luarocks 依存なし | **Option C-Phase0**: 新規作成 |
| **R2: サンプルテスト作成** | - tests/fixtures/sample.pasta<br>- tests/fixtures/sample.lua | **Missing**: Lua モジュール API<br>**Missing**: transpiler_spec.lua など | **Option C-Phase0**: pending テスト骨組み作成<br>**Option C-Phase1**: Lua モジュール実装 |
| **R3: ローカル実行環境** | - 既存 Rust テストパターン | **Missing**: README.md<br>**Missing**: luarocks セットアップ手順 | **Option C-Phase0**: README.md 作成 |
| **R4: CI/CD 統合（Optional）** | - GitHub リポジトリ | **Missing**: GitHub Actions ワークフロー | **Option C-Phase2**: ワークフロー作成（Optional） |

---

## 6. 設計フェーズへの推奨事項

### 推奨アプローチ

**オプション C（ハイブリッドアプローチ）** を推奨

**理由**:
- Phase 0 で基盤確立、リスクゼロで即座に価値提供
- Phase 1 で Lua モジュール実装、段階的にリスク管理
- Phase 2 は Optional、プロジェクト状況に応じて柔軟に対応

### 主要判断ポイント

1. **Lua モジュール実装のタイミング**
   - **判断**: Phase 0 では実装しない、Phase 1 で対応
   - **根拠**: 既存コード変更を避け、リスクを最小化

2. **CI/CD 統合の優先度**
   - **判断**: Optional（Phase 2）
   - **根拠**: ローカル実行環境が最優先、CI は後続フェーズで対応可能

3. **テストディレクトリ配置**
   - **判断**: `tests/lua_specs/` に配置
   - **根拠**: Rust 統合テストと明確に分離、structure.md の規約に準拠

4. **サンプルテスト範囲**
   - **判断**: 3 個（transpiler, code_generator, context）に限定
   - **根拠**: Phase 0 で基本パターン確立、後続フェーズで拡充

### 調査項目（設計フェーズで実施）

**Research Needed（すべて research.md で完了済み）**:
- なし

**設計フェーズで詳細化すべき項目**:
1. **Lua モジュール API 設計**
   - mlua の UserData/Module どちらを使用するか
   - LuaTranspiler の Lua ラッパーメソッド定義
   - エラーハンドリング戦略（Lua error() vs Result 型）

2. **`.busted` 設定の詳細**
   - タスクベース設定（default, verbose, coverage など）
   - pattern の最適化（`*_spec.lua` vs `**/*_spec.lua`）
   - 出力フォーマット選択（utfTerminal, JSON, TAP）

3. **README.md の内容構成**
   - セクション構成（セットアップ、実行、追加方法、トラブルシューティング）
   - サンプルコマンド例
   - OS 別インストール手順

4. **CI/CD ワークフロー設計（Optional）**
   - `cargo test` と `busted` の実行順序
   - キャッシュ戦略（Rust ビルドキャッシュ、luarocks キャッシュ）
   - テスト並列実行の可否

---

## 分析アプローチ

本分析は **Gap Analysis Framework** に基づき、以下の手順で実施しました：

1. **現状調査**: Grep/Read ツールで pasta_lua の既存実装を深掘り
2. **要件実現性分析**: requirements.md の各要件を技術的必要事項に分解
3. **ギャップ特定**: Missing/Unknown/Constraint を明確化
4. **複数オプション評価**: Option A/B/C のトレードオフを比較
5. **工数・リスク見積もり**: S/M/L スケールで複雑度評価

**分析品質保証**:
- ✅ 既存コードベースを徹底調査（20+ ファイル参照）
- ✅ steering 全体（5 ファイル）との整合性確認
- ✅ 複数の実装アプローチを客観的に比較
- ✅ リスクと軽減策を明示

---

## 次のステップ

設計フェーズに進んでください：

```bash
# 設計書を生成（要件自動承認）
/kiro-spec-design pasta-lua-unit-test-framework -y
```

設計フェーズでは、以下を詳細化します：
- Lua モジュール API の具体的設計（Phase 1 準備）
- `.busted` 設定ファイルの完全な定義
- README.md のセクション構成とサンプルコマンド
- GitHub Actions ワークフローの YAML 定義（Optional）

**推奨**: 本ギャップ分析の「オプション C（ハイブリッドアプローチ）」を設計書に反映し、Phase 0 → Phase 1 → Phase 2 の段階的実装を計画してください。
