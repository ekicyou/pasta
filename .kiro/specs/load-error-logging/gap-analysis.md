# Gap Analysis: load-error-logging

## 1. 現状調査

### 関連ファイル・モジュール

| ファイル | 役割 | 変更対象 |
|---------|------|---------|
| `crates/pasta_shiori/src/shiori.rs` | `PastaShiori` 構造体、`Shiori::load()`/`request()` 実装 | ✅ 主要 |
| `crates/pasta_shiori/src/error.rs` | `MyError` 列挙型、`to_shiori_response()` | ✅ 主要 |
| `crates/pasta_shiori/src/windows.rs` | `RawShiori<T>` DLLエントリ、`load_impl()`/`request_impl()` | ✅ 要変更 |
| `crates/pasta_lua/src/logging/logger.rs` | `PastaLogger::new()` + `RollingFileAppender` | ✅ 主要 |
| `crates/pasta_lua/src/loader/mod.rs` | `PastaLoader::load()`, `process_incremental()` | ⚠️ 調査対象 |
| `crates/pasta_lua/src/loader/config.rs` | `LoggingConfig`, `rotation_days` | ✅ 要変更 |
| `crates/pasta_lua/src/loader/error.rs` | `LoaderError`, `PartialTranspileError`, `TranspileFailure` | ⚠️ 調査対象 |
| `crates/pasta_lua/src/logging/registry.rs` | `GlobalLoggerRegistry`, `LoadDirGuard` | 参照のみ |

### 既存アーキテクチャパターン

- **レイヤー構造**: `pasta_shiori`（SHIORI DLL） → `pasta_lua`（Luaバックエンド） → `pasta_dsl`（パーサー）
- **エラー変換**: `LoaderError` → `MyError::Load(String)` でフォーマットされたメッセージを保持
- **ロギング**: `tracing` + `tracing-appender`（`GlobalLoggerRegistry` + `PastaLogger` での多インスタンスルーティング）
- **トランスパイル失敗処理**: `process_incremental()` は失敗しても `Ok(...)` を返す（`warn!` で報告のみ）

### 統合ポイント

- `PastaShiori::load()` は `PastaLoader::load()` を呼び出し、成功後にのみ tracing subscriber を初期化
- `RawShiori<T>::load_impl()` が `shiori.load()` を呼び、結果を `*guard = Some(shiori)` で保持
- `RawShiori<T>::request_impl()` は `guard` が `None` なら `MyError::NotInitialized` を返す
- `MyError::to_shiori_response()` が `X-ERROR-REASON` ヘッダを生成

---

## 2. 要件別ギャップ分析

### Requirement 1: load エラーメッセージの SHIORI 応答への伝搬

**現状**: 
- `PastaShiori::load()` の `Err(e)` ブランチで `Ok(false)` を返すが、エラー情報 `e` を保持しない
- `PastaShiori` 構造体に `last_error` フィールドが存在しない
- `request()` は `MyError::NotInitialized`（固定メッセージ `"not initialized error"`）を返す

**ギャップ**: Missing — エラー保持メカニズムが存在しない

**対応**: `PastaShiori` に `last_load_error: Option<String>` フィールドを追加し、load 失敗時に設定。`request()` で `NotInitialized` の代わりに保持されたエラーを使用。

### Requirement 2: load 失敗時のログファイル出力保証

**現状**:
- `init_tracing_with_config()` は `PastaLoader::load()` **成功後**にのみ呼ばれる（[shiori.rs L149-153](crates/pasta_shiori/src/shiori.rs)）
- load 失敗時、tracing subscriber は未初期化 → `error!()` マクロが実質 no-op
- `PastaLoader::load()` 内部で `Phase 6: Creating instance logger` としてロガーを作成するが、フェーズ4（トランスパイル）の前

**ギャップ**: Missing — load 前のログ初期化パスが存在しない

**対応**:
- `PastaShiori::load()` で `PastaLoader::load()` の**前に**最低限のログ初期化を行う
- 2つのアプローチが考えられる（Option A/B 参照）

**重要な発見**: `PastaLoader::load()` **内部**のフェーズ6でロガーが作成される。しかしフェーズ4（トランスパイル）のエラーは、フェーズ6より前に発生するため、ロガーが未作成の状態。ただし `tracing::warn!()` はLoader内部で呼ばれており、subscriber さえ初期化されていれば記録される。

### Requirement 3: ログファイル名の簡素化

**現状**:
- `PastaLogger::new()` で `Rotation::DAILY` + `filename_prefix()` を使用
- これにより `pasta.log.2026-03-16` 形式のファイル名が生成される
- `LoggingConfig` に `rotation_days: usize` フィールドが存在

**ギャップ**: Constraint — `Rotation::DAILY` → `Rotation::NEVER` への変更が必要

**対応**: 
- `logger.rs`: `Rotation::DAILY` → `Rotation::NEVER`、`filename_prefix()` → `filename_suffix()` の空指定 もしくは直接 `File::create` に置き換え
- `config.rs`: `rotation_days` フィールドの扱い（残すか削除か）

**注意**: `Rotation::NEVER` の場合でも `RollingFileAppender` は使用可能だが、`filename_prefix` が付くと `pasta.log.` のようなファイル名になる可能性がある。`tracing-appender` 0.2 の `Rotation::NEVER` の挙動を確認する必要あり。 → **Research Needed**

### Requirement 4: トランスパイルエラーの詳細記録

**現状**:
- `process_incremental()` が失敗ファイルを `Vec<TranspileFailure>` に収集
- `warn!()` で各失敗を報告（L490-496）
- しかし `process_incremental()` は `Ok(...)` を返す — 失敗があっても `LoaderError` に変換しない
- `stats.failed` がカウントされるが、呼び出し元でチェックしていない

**ギャップ**: Partial — 情報は収集されているが、呼び出し元への伝搬が不足

**対応**: `process_incremental()` で `stats.failed > 0` の場合に `Err(LoaderError::PartialTranspileError {...})` を返すか、`Ok` のまま返して呼び出し元で `stats.failed` を確認するか。部分失敗時の起動方針（継続 or 中止）は設計フェーズで決定。

### Requirement 5: ~~エラーメッセージの可読性~~ → Requirement 1 に統合済み

Req1-AC3/AC4/AC5 として吸収。

---

## 3. 実装アプローチ

### Option A: 既存コンポーネント拡張（推奨）

**概要**: `PastaShiori`, `MyError`, `PastaLogger` を最小限に拡張

**変更箇所**:

1. **`PastaShiori` 構造体** (`shiori.rs`):
   - `last_load_error: Option<String>` フィールドを追加
   - `load()` の `Err(e)` ブランチでエラーを保持
   - `request()` で `last_load_error` がある場合は `MyError::Load(msg)` を返す

2. **`init_tracing_with_config()`** (`shiori.rs`):
   - `PastaLoader::load()` の前にデフォルト設定でロガー初期化を呼ぶ
   - `try_init()` なので、成功後の再初期化は安全に無視される

3. **`PastaLogger::new()`** (`logger.rs`):
   - `Rotation::DAILY` → `Rotation::NEVER`
   - ファイル名生成ロジックの簡素化

4. **`LoggingConfig`** (`config.rs`):
   - `rotation_days` を非推奨化（互換性のため残す）

5. **`process_incremental()`** (`loader/mod.rs`):
   - 失敗がある場合に `Err(LoaderError::PartialTranspileError)` を返す

**トレードオフ**:
- ✅ 変更ファイル数が最小（4-5ファイル）
- ✅ 既存パターンに完全準拠
- ✅ 後方互換性を維持
- ❌ `PastaShiori` の責務がわずかに増加

### Option B: 早期ロガー分離

**概要**: load 前のロギングを `PastaLoader` の外に分離し、`PastaShiori` がロガーを先に作成

**変更箇所**:
- `PastaShiori::load()` で先に `PastaLogger` を作成・登録
- `PastaLoader` の Phase 6 を削除し、外部から注入

**トレードオフ**:
- ✅ ロガーのライフサイクルが明確
- ❌ `PastaLoader` のAPI変更が必要（`PastaLuaRuntime::from_loader_with_scene_dic` の引数変更）
- ❌ 変更範囲が大きい

### Option C: ハイブリッド（不採用）

Option Aで十分カバーできるため、ハイブリッドは不要。

---

## 4. 複雑さ・リスク評価

**工数**: **S**（1-3日）  
既存パターンの拡張のみ。新しいアーキテクチャパターンや依存ライブラリの追加なし。

**リスク**: **Low**  
- 全変更が既存インターフェースの拡張
- `try_init()` による安全な二重初期化防止
- `tracing-appender` の `Rotation::NEVER` の挙動確認のみ調査が必要

---

## 5. Research Needed（設計フェーズで確認）

1. **`tracing-appender` 0.2 の `Rotation::NEVER` のファイル名挙動**: `filename_prefix` / `filename_suffix` が空の場合にどのようなファイル名が生成されるか
2. **`process_incremental` のエラー伝搬戦略**: 部分失敗時に `Err` で返すか、`Ok` + warning で返すか（既存の設計意図を確認）

---

## 6. 推奨事項

- **推奨アプローチ**: Option A（既存コンポーネント拡張）
- **キー決定事項**: load前ロガーはデフォルト `LoggingConfig` で `init_tracing_with_config()` を早期呼び出し
- **設計フェーズへのキャリーオーバー**: Research Needed の2項目
