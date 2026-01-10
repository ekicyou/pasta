# Implementation Gap Analysis

## 分析概要

**スコープ**: pasta_luaの起動シーケンス確立 - ディレクトリ探索、設定ファイル解釈、トランスパイル、ランタイム初期化の統合

**主要な課題**:
- ファイルシステム探索機能が未実装（`dic/*/*.pasta` パターン）
- TOML設定ファイル解析機能が未実装（`pasta.toml`）
- 複数ファイル統合トランスパイル機能が未実装（現在は単一ファイルのみ）
- ディレクトリ自動作成とLuaモジュール検索パス設定が未実装
- 統合起動API（`PastaLoader`）が未実装

**推奨アプローチ**: Option B（新規コンポーネント作成）- `loader` モジュールを新設し、既存の `transpiler` と `runtime` を統合

---

## 1. 現状分析

### 既存アセット

**コア機能（実装済み）:**
- `LuaTranspiler` - 単一PastaFileのトランスパイル（`transpiler.rs`）
- `PastaLuaRuntime` - Lua VM初期化とモジュール登録（`runtime/mod.rs`）
- `TranspileContext` - シーン/単語レジストリ管理（`context.rs`）
- `TranspileError` - エラー型（thiserror統合済み、`error.rs`）
- `pasta_core::parse_str/parse_file` - Pastaパーサー

**設定管理:**
- `TranspilerConfig` - トランスパイラー設定（コメント、改行、`config.rs`）
- `RuntimeConfig` - ランタイム設定（stdlib有効化、`runtime/mod.rs`）

**依存関係:**
- `pasta_core` - パーサー、レジストリ
- `mlua` - Lua VM
- `mlua-stdlib` - @assertions, @testing, @regex等
- `thiserror` - エラー型定義

**欠落している依存関係:**
- `glob` - ファイル探索（Cargo.tomlに未登録）
- `toml` / `serde` - TOML設定解析（Cargo.tomlに未登録）

### アーキテクチャパターン

**レイヤー構成（tech.md準拠）:**
```
Engine (統合API層) ← 今回実装対象
  ↓
Transpiler, Runtime (既存)
  ↓
pasta_core（再エクスポート）
```

**命名規約:**
- モジュール: スネークケース（例: `loader.rs`）
- 公開API: パスカルケース（例: `PastaLoader`）
- エラー型: `{Context}Error`（例: `LoaderError`）

**エラーハンドリング:**
- `thiserror` による型階層
- `Result<T, {Context}Error>` パターン
- `Display` trait実装で人間可読メッセージ

**テスト戦略:**
- 統合テスト: `tests/loader_integration_test.rs`
- フィクスチャ: `tests/fixtures/loader/`

---

## 2. 要件と実装ギャップ

### Requirement 1: 起動ディレクトリ探索

**必要な機能:**
- `dic/*/*.pasta` globパターン探索
- `profile/**` 除外
- エラーハンドリング（ディレクトリ不存在、権限エラー）

**ギャップ:**
- ❌ **Missing**: ファイル探索機能全体
- ✅ **Resolved**: globクレート依存関係追加済み（Cargo.toml更新完了）

**実装候補:**
- `glob` クレート（既にpasta_runeで使用中、パターン実績あり）
- または `walkdir` + 手動フィルタリング

### Requirement 2: 設定ファイル解釈

**必要な機能:**
- `pasta.toml` 読み込み・デシリアライズ
- `RuntimeConfig` へのマッピング
- ディレクトリ自動作成（`profile/pasta/save/lua/` 等）

**ギャップ:**
- ❌ **Missing**: TOML解析機能全体
- ✅ **Resolved**: `toml`, `serde` 依存関係追加済み（Cargo.toml更新完了）
- ❌ **Missing**: 設定ファイルスキーマ定義
- ❌ **Missing**: ディレクトリ作成ロジック

**実装候補:**
- `toml` + `serde` （標準的な組み合わせ）
- `std::fs::create_dir_all` でディレクトリ作成

### Requirement 3: 複数ファイルトランスパイル

**必要な機能:**
- 複数PastaFileの統合トランスパイル
- 単一TranspileContextへのレジストリ統合
- 生成コードの `profile/pasta/cache/lua/` への保存

**ギャップ:**
- ⚠️ **Constraint**: 既存 `LuaTranspiler::transpile()` は単一ファイルのみ対応
- ✅ **Existing**: `TranspileContext` は複数ファイルの統合をサポート（レジストリは追加可能）
- ❌ **Missing**: ループで複数ファイルをトランスパイルするロジック
- ❌ **Missing**: ファイル保存ロジック

**実装アプローチ:**
- 複数回 `transpiler.transpile()` を呼び出し、同一Contextを共有
- 各ファイルの生成コードを連結してキャッシュに保存

### Requirement 4: ランタイム初期化

**必要な機能:**
- 4階層Luaモジュール検索パス設定（絶対パス）
- キャッシュファイルのロード

**ギャップ:**
- ❌ **Missing**: `package.path` 設定ロジック（既存ランタイムは設定しない）
- ✅ **Existing**: テストコード `lua_unittest_runner.rs` に `package.path` 設定の実装例あり
- ❌ **Missing**: 起動ディレクトリの絶対パス取得・保持

**実装アプローチ:**
- `PastaLuaRuntime::with_config()` に起動ディレクトリパスを追加
- または `PastaLoader` が設定した後にランタイムを初期化

### Requirement 5: 統合起動API

**必要な機能:**
- `PastaLoader::load(path)` - ディレクトリ探索 → トランスパイル → ランタイム初期化
- `tracing` ログ出力

**ギャップ:**
- ❌ **Missing**: `PastaLoader` 型全体
- ✅ **Existing**: `tracing` 依存関係あり（Cargo.toml登録済み）

### Requirement 6: エラーハンドリング

**必要な機能:**
- 構造化エラー型（`LoaderError`, `StartupError`）
- ファイルパス、行番号情報を含むエラー

**ギャップ:**
- ❌ **Missing**: `LoaderError` 型（エラー種別の統一命名）
- ✅ **Existing**: `TranspileError` のパターンを踏襲可能（`error.rs`）

---

## 3. 実装アプローチオプション

### Option A: 既存コンポーネント拡張（非推奨）

**拡張対象:**
- `LuaTranspiler` に `transpile_directory()` メソッド追加
- `PastaLuaRuntime` に `package.path` 設定機能追加

**トレードオフ:**
- ✅ 新規ファイルなし
- ❌ `LuaTranspiler` の責務が肥大化（ファイルシステム操作 + トランスパイル）
- ❌ `PastaLuaRuntime` がディレクトリ構造に依存（単体テスト困難）

**リスク**: **High**（単一責任原則違反、既存APIの複雑化）

---

### Option B: 新規コンポーネント作成（推奨）

**新規モジュール:**
- `loader` モジュール（`src/loader/mod.rs`）
  - `PastaLoader` 構造体
  - `LoaderConfig` 構造体（設定ファイルスキーマ）
  - `LoaderError` 型

**責務分離:**
- `PastaLoader`: ファイル探索、設定読み込み、ディレクトリ作成、統合フロー
- `LuaTranspiler`: トランスパイル（変更なし）
- `PastaLuaRuntime`: Lua VM実行（パス設定機能を追加）

**統合ポイント:**
```rust
// loader/mod.rs
pub struct PastaLoader {
    base_dir: PathBuf,
    config: LoaderConfig,
}

impl PastaLoader {
    pub fn load(path: impl AsRef<Path>) -> Result<PastaLuaRuntime, LoaderError> {
        // 1. ディレクトリ探索
        // 2. 設定ファイル読み込み
        // 3. トランスパイル（既存API使用）
        // 4. ランタイム初期化（拡張API使用）
    }
}
```

**トレードオフ:**
- ✅ 明確な責務分離
- ✅ 既存コンポーネント変更最小
- ✅ 単体テスト容易
- ❌ 新規ファイル追加（`loader/`）

**リスク**: **Low**（確立されたパターン、pasta_runeの `PastaEngine` と類似）

---

### Option C: ハイブリッドアプローチ（中間案）

**フェーズ1: 最小限の新規API**
- `PastaLoader::load_simple(path)` - 設定ファイルなし、デフォルト動作のみ

**フェーズ2: 設定ファイル対応**
- `LoaderConfig` 導入、`pasta.toml` サポート

**トレードオフ:**
- ✅ 段階的実装
- ✅ MVP早期提供
- ❌ 2段階の設計・実装コスト

**リスク**: **Medium**（スコープ管理が必要）

---

## 4. 研究項目（設計フェーズで詳細化）

以下は設計フェーズで詳細調査が必要な項目：

1. **TOML設定ファイルスキーマ**: RuntimeConfig以外にどの設定項目を含めるか？
2. **キャッシュファイル命名規則**: `<hash>.lua` のハッシュアルゴリズムは？（MD5, Blake3等）
3. **package.path設定の詳細**: Luaの`?.lua`と`?/init.lua`パターンをどう組み合わせるか？
4. **エラーリカバリ戦略**: 一部のpastaファイルが破損している場合の挙動は？

---

## 5. 実装規模とリスク評価

### 工数見積もり: **M（3-7日）**

**理由:**
- 新規モジュール作成（`loader`）: 2-3日
- TOML設定解析: 1日
- ファイル探索・保存ロジック: 1日
- 統合テスト・ドキュメント: 1-2日

**内訳:**
- ディレクトリ探索（glob）: S（既存パターンあり）
- TOML設定解析: S（標準的な実装）
- 複数ファイルトランスパイル: M（既存API統合）
- package.path設定: S（テストコードに実装例あり）
- 統合API設計: M（新規エントリーポイント）

### リスク評価: **Low**

**理由:**
- 既存の `LuaTranspiler`, `PastaLuaRuntime` が安定
- glob, toml/serdeは枯れた依存関係
- 類似実装（pasta_runeの`PastaEngine`）が存在
- 明確な責務分離により影響範囲が限定的

**潜在的リスク:**
- Luaモジュール検索パスの絶対パス化（Windowsパス区切り `\` のエスケープ処理）
- 設定ファイルスキーマの拡張性（将来の機能追加に対応）

---

## 6. 推奨事項

### 設計フェーズへの提言

1. **Option B（新規コンポーネント）を推奨**
   - `src/loader/mod.rs` に `PastaLoader` を実装
   - 既存APIへの影響を最小化
   - pasta_runeの `PastaEngine` パターンを踏襲

2. **依存関係追加**
   - `glob = "0.3"` をCargo.tomlに追加
   - `toml = "0.9.8"`, `serde = { version = "1", features = ["derive"] }` を追加

3. **設定ファイルスキーマの初期設計**
   ```toml
   [runtime]
   enable_std_libs = true
   enable_assertions = true
   # ... RuntimeConfig相当

   [loader]
   script_pattern = "dic/*/*.pasta"  # オプション（将来の拡張用）
   ```

4. **PastaLuaRuntime拡張**
   - `with_base_dir(context, config, base_dir: PathBuf)` メソッドを追加
   - 内部で `package.path` を絶対パスで設定

5. **テスト戦略**
   - `tests/fixtures/loader/` に最小限のディレクトリ構造を作成
   - 統合テストで `PastaLoader::load()` のE2Eテストを実施

---

## 次のステップ

ギャップ分析完了。以下のコマンドで設計フェーズに進んでください：

```
/kiro-spec-design pasta-lua-startup-sequence
```

設計フェーズでは、以下を詳細化します：
- `PastaLoader` の詳細API設計
- `LoaderConfig` / `LoaderError` 型定義
- ファイル探索・保存ロジックのアルゴリズム
- package.path設定の具体的な実装戦略
