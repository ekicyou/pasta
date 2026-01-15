# Implementation Gap Analysis

## 分析概要

**スコープ**: PastaShiori::load()関数の実装でPastaLoader::load()を統合し、PastaLuaRuntimeインスタンスを保持する。

**主要な課題**:
- PastaShioriはArc<Mutex<Option<T>>>パターンで既存のライフサイクル管理を実装済み
- pasta_luaはPastaLoader::load()とPastaLuaRuntime::from_loader()の統合APIを提供済み
- エラー型の相互変換（LoaderError → MyError）が必要
- tracingロガーの初期化タイミング（DllMainで実施済みか確認必要）

**推奨アプローチ**: Option A（既存コンポーネント拡張）- PastaShiori構造体にランタイムフィールドを追加し、load実装を更新

---

## 1. 現状調査

### 既存アセット

**pasta_shiori クレート構造**:
```
crates/pasta_shiori/src/
├── lib.rs                # クレートエントリーポイント
├── shiori.rs             # Shioriトレイト、PastaShiori実装
├── error.rs              # MyError型、MyResult型
├── windows.rs            # Windows FFI、RawShiori<T>ラッパー
└── util/                 # HGLOBAL文字列変換など
```

**既存パターン・規約**:
- **ライフサイクル管理**: `RawShiori<T>(hinst, Arc<Mutex<Option<T>>>)`パターン
  - DllMainでOnceLockに初期化
  - load()でMutex内にSome(T)を設定
  - unload()でNoneに設定
- **エラーハンドリング**: MyResult<T>、MyError型で統一
  - `to_shiori_response()`でSHIORI 500エラーレスポンス生成
  - tracingマクロ（error!）でログ出力
- **依存関係**: pasta_core, pasta_lua, tracing, thiserror, windows-sys
- **テスト配置**: 現時点でテストなし（統合が必要）

**統合サーフェス**:
- **PastaLoader::load(base_dir)** → `Result<PastaLuaRuntime, LoaderError>`
- **LoaderError型**: `DirectoryNotFound`, `Config`, `Parse`, `Transpile`, `Runtime(mlua::Error)`, `Io`
- **PastaLuaRuntime**: Lua VMホスト、exec()メソッドでスクリプト実行

### 技術的ニーズと既存の対応

| 要件 | 技術的ニーズ | 既存対応 | ギャップ |
|------|------------|---------|---------|
| Req 1: Runtimeインスタンス管理 | PastaLuaRuntimeフィールド保持 | Option<T>パターン実装済み | **Missing**: PastaShioriにruntime: Option<PastaLuaRuntime>フィールド追加 |
| Req 2: load_dirパス処理 | PathBuf変換、存在確認 | load_dir: Option<PathBuf>フィールド実装済み | **Missing**: PastaLoader::load()呼び出し実装 |
| Req 3: エラーハンドリング | LoaderError → MyError変換 | MyError型定義済み、tracingログ出力パターン実装済み | **Missing**: From<LoaderError> for MyError実装 |
| Req 4: hinstパラメータ保持 | hinst: isizeフィールド保持 | ✅ 実装済み（shiori.rsのhinst: isize） | - |
| Req 5: Runtime状態管理 | Option<Runtime>、Drop実装 | ✅ Option<T>パターン実装済み、Drop trait実装済み | **Missing**: request()でランタイム未初期化チェック |

### 制約

- **Windowsプラットフォーム専用**: windows-sys依存、cfg(windows)ゲート
- **FFI境界**: load()はRawShiori経由でC ABI互換の関数から呼ばれる
- **スレッドセーフティ**: Arc<Mutex<Option<T>>>によるスレッドセーフな状態管理
- **エラー情報制限**: SHIORI 500エラーレスポンスに変換するため、詳細情報はX-ERROR-REASONに格納

---

## 2. 実装アプローチオプション

### Option A: 既存コンポーネント拡張（推奨）

**変更対象**:
- `crates/pasta_shiori/src/shiori.rs`
  - PastaShiori構造体にruntime: Option<PastaLuaRuntime>フィールド追加
  - load()実装を更新してPastaLoader::load()を呼び出し
  - request()実装でランタイム未初期化チェック追加
- `crates/pasta_shiori/src/error.rs`
  - From<LoaderError> for MyError実装追加
  - MyError::Loadバリアントの詳細化（現状は単純なLoad）

**互換性評価**:
- ✅ Shioriトレイト互換性保持（シグネチャ不変）
- ✅ RawShiori<T>ラッパーは型パラメータなので影響なし
- ✅ 既存のDrop実装がOption<PastaLuaRuntime>も自動解放

**複雑性・保守性**:
- ✅ PastaShiori構造体の責務は明確（SHIORI DLLライフサイクル管理）
- ✅ ファイルサイズ小規模（現在shiori.rs: 31行 → 推定50行程度）
- ✅ 単一責任原則維持（pasta_luaロード＆保持のみ）

**トレードオフ**:
- ✅ 最小限のファイル変更、既存パターン活用
- ✅ pasta_luaとの統合は公開APIのみ使用（PastaLoader::load()）
- ❌ エラー型変換コードが必要（From実装で対応可能）
- ❌ shiori.rsがpasta_lua依存を直接持つ（すでにCargo.tomlで宣言済み）

---

### Option B: 新規コンポーネント作成

**新規作成対象**:
- `crates/pasta_shiori/src/runtime_manager.rs`
  - RuntimeManagerトレイトとPastaLuaRuntimeManager実装
  - load_dir → PastaLoader::load() → ランタイム保持ロジック
  - PastaShioriはRuntimeManagerトレイトに委譲

**統合ポイント**:
- PastaShiori::load()がRuntimeManager::load()を呼び出し
- PastaShiori::request()がRuntimeManager::execute_request()を呼び出し

**責務境界**:
- **RuntimeManager**: pasta_luaロード、ランタイムライフサイクル管理
- **PastaShiori**: SHIORI DLLインターフェース、FFI境界処理

**トレードオフ**:
- ✅ 関心の分離が明確（SHIORI層とランタイム管理層）
- ✅ 将来的なpasta_rune統合時にRuntimeManagerを差し替え可能
- ❌ 小規模機能のために新規ファイル追加（過剰設計の可能性）
- ❌ トレイト定義とジェネリック実装のオーバーヘッド

---

### Option C: ハイブリッドアプローチ

**組み合わせ戦略**:
- Phase 1: Option Aで最小限実装（MVP）
  - PastaShiori::load()にPastaLoader::load()統合
  - エラー変換実装
- Phase 2: Option Bで抽象化（将来のpasta_rune統合時）
  - RuntimeManagerトレイト導入
  - pasta_lua/pasta_runeの選択的ロード機能

**段階的実装**:
1. **初期フェーズ**: Option A実装で動作検証
2. **リファクタリングフェーズ**: pasta_rune統合が必要になった時点でOption B移行

**リスク軽減**:
- 最小限の変更で動作確認
- 将来の拡張性を設計レベルで考慮
- 段階的移行によるリグレッションリスク低減

**トレードオフ**:
- ✅ 段階的実装により早期フィードバック可能
- ✅ 過剰設計を回避しつつ拡張性確保
- ❌ Phase 2でリファクタリングコスト発生
- ❌ 設計意図の文書化が必要（Phase 2移行時の指針）

---

## 3. 実装複雑性・リスク評価

### 工数見積もり: **S（1-3日）**

**根拠**:
- 既存パターン（Option<T>、Arc<Mutex<>>）を活用
- pasta_luaは統合API（PastaLoader::load）を提供済み
- 変更範囲が限定的（shiori.rs、error.rs、テスト追加）
- 外部依存なし（すべてワークスペース内クレート）

**タスク分解**:
1. エラー変換実装（1-2時間）: From<LoaderError> for MyError
2. PastaShiori構造体更新（2-3時間）: runtimeフィールド追加、load()実装
3. テスト作成（3-4時間）: 単体テスト（load成功/失敗）、統合テスト（SHIORI呼び出し）
4. ドキュメント更新（1時間）: コードコメント、SHIORI.md更新

### リスク評価: **Low（低）**

**根拠**:
- ✅ 確立されたパターン使用（Arc<Mutex<Option<T>>>）
- ✅ 既知の技術スタック（Rust、mlua、pasta_lua）
- ✅ 明確なスコープ（load関数のみ）
- ✅ pasta_luaの統合APIが既にテスト済み（loader_integration_test.rs）

**特定リスクと軽減策**:

| リスク | 影響 | 確率 | 軽減策 |
|--------|------|------|--------|
| tracingロガー未初期化 | ログ出力されない | 低 | DllMainでtracing_subscriber初期化確認、テストで検証 |
| LoaderError詳細情報損失 | デバッグ困難 | 中 | MyError::Load(String)に詳細メッセージを格納、X-ERROR-REASONに出力 |
| パフォーマンス劣化 | ロード時間増加 | 低 | PastaLoader::load()は最適化済み（キャッシュ機構あり） |
| Windowsパス互換性 | \\?\プレフィックス問題 | 低 | LoaderContext::strip_windows_prefix()で既に対処済み |

---

## 4. 設計フェーズへの推奨事項

### 優先アプローチ: **Option A（既存コンポーネント拡張）**

**理由**:
- 最小限の変更で要件を満たす
- 既存パターンとの一貫性維持
- 早期フィードバックサイクル実現
- pasta_rune統合は将来の別仕様で対応（YAGNIの原則）

### 設計で決定すべき事項

1. **エラー変換戦略**:
   - From<LoaderError> for MyError実装の詳細設計
   - LoaderError各バリアントのマッピング方針
   - X-ERROR-REASONフィールドに含める情報レベル

2. **tracingロガー初期化**:
   - **決定**: RawShiori::new()で初期化（シングルトンライフサイクルに合致）
   - tracing_subscriber、tracing_appender依存関係追加
   - ログ出力先: `ghost/master/profile/pasta/logs/pasta_shiori.log`
   - ローテーション: なし（起動時に上書き、最後の起動ログのみ保持）
   - profile配下以外は動的ファイルで汚染しない原則

3. **テスト戦略**:
   - 単体テスト: PastaShiori::load()の成功/失敗パス
   - 統合テスト: Windows FFI経由の呼び出しシミュレーション
   - フィクスチャ: テスト用pasta.toml、.pastaファイル配置

4. **ドキュメント更新**:
   - shiori.rs内のドキュメントコメント
   - util/shiori.md（SHIORI DLL仕様）の更新
   - examples/の追加（オプショナル）

### 要調査項目（Research Needed）

なし（全議題完了）

### 確認済み事項

1. **pasta.toml配置規約** ✅:
   - load_dirパラメータ: `ghost/master/`
   - pasta.toml配置: `ghost/master/pasta.toml`
   - スクリプト配置: `ghost/master/dic/*.pasta`

2. **パフォーマンス要件** ✅:
   - load()実行時間許容範囲: 1-3秒以内
   - ユーザー体感を重視（ゴースト起動時の待ち時間として許容範囲）
   - 現状のPastaLoader::load()実装で要件を満たす見込み

---

## 5. 要件とアセットのマッピング

| 要件 | 対応アセット | ギャップ | 実装アプローチ |
|------|------------|---------|--------------|
| **Req 1: Runtimeインスタンス管理** | PastaLoader::load(), Option<T>パターン | PastaShiori::runtimeフィールド追加 | Option A: フィールド追加、load()実装 |
| **Req 2: load_dirパス処理** | PathBuf変換、PastaLoader::load(base_dir) | load()実装でPastaLoader呼び出し | Option A: load()内でPastaLoader::load()統合 |
| **Req 3: エラーハンドリング** | MyError型、tracing、to_shiori_response() | From<LoaderError> for MyError | Option A: エラー変換実装 |
| **Req 4: hinstパラメータ保持** | PastaShiori::hinst: isize | ✅ 実装済み | - |
| **Req 5: Runtime状態管理** | Option<T>、Drop trait | request()未初期化チェック | Option A: request()でNoneチェック追加 |

---

## まとめ

**実装の実現可能性**: ✅ **高**（既存パターンとpasta_lua統合APIを活用）

**推奨実装戦略**: **Option A - 既存コンポーネント拡張**
- shiori.rs、error.rsを更新
- PastaLoader::load()統合
- エラー変換実装
- テスト追加

**次フェーズで設計すべき内容**:
1. From<LoaderError> for MyError実装の詳細設計
2. tracingロガー初期化戦略
3. テスト構成とフィクスチャ配置
4. ドキュメント更新範囲

**調査が必要な項目**:
- tracingロガー初期化タイミング（DllMainでの初期化可否）
- pasta.tomlディレクトリ配置規約
- パフォーマンス基準設定
