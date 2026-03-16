# Research & Design Decisions: load-error-logging

## Summary
- **Feature**: `load-error-logging`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. `tracing-appender` 0.2.4 の `Rotation::NEVER` + `filename_prefix` で正確な固定ファイル名が得られる
  2. `init_tracing_with_config()` は `try_init()` ベースのため、早期呼び出し後の再初期化は安全に無視される
  3. `RawShiori<T>::load_impl()` が `load()` の成否に関わらず `*guard = Some(shiori)` を設定する — エラー保持型にするには `request_impl()` のフロー変更が必要
  4. `PartialTranspileError` の `Display` にファイルパスを含めることで `X-ERROR-REASON` 経由でファイル名が伝搬する（設計レビュー後の議論で確定）

## Research Log

### tracing-appender 0.2.4 の `Rotation::NEVER` ファイル名挙動

- **Context**: Req3 — ログファイル名を `pasta.log.2026-03-16` → `pasta.log` に変更するため、`Rotation::NEVER` の正確な動作を確認
- **Sources Consulted**: `tracing-appender` 0.2.4 ソースコード（`rolling.rs` L619-642, `builder.rs`）
- **Findings**:
  - `join_date()` 関数（L630-642）の分岐:
    ```
    (&Rotation::NEVER, Some(filename), None) => filename.to_string()
    (&Rotation::NEVER, Some(filename), Some(suffix)) => format!("{}.{}", filename, suffix)
    (&Rotation::NEVER, None, Some(suffix)) => suffix.to_string()
    ```
  - `Rotation::NEVER` + `filename_prefix("pasta.log")` + suffix なし → ファイル名は正確に `"pasta.log"`
  - `max_log_files` は `Rotation::NEVER` では実質無効（ローテーションしないため古いファイルが発生しない）
  - Builder のデフォルト rotation が `Rotation::NEVER` であるため、`.rotation()` 呼び出しは省略可能だが明示性のため記載推奨
- **Implications**:
  - 現行の `.rotation(Rotation::DAILY).filename_prefix(log_file_name)` を `.rotation(Rotation::NEVER).filename_prefix(log_file_name)` に変更するだけで OK
  - `.max_log_files()` は不要になる（削除可能）
  - `config.rotation_days` フィールドは互換性のため残すが、`Rotation::NEVER` では使用しない

### `init_tracing_with_config()` の早期呼び出し安全性

- **Context**: Req2 — `PastaLoader::load()` 前にロガーを初期化しても、成功後の再初期化と衝突しないか
- **Sources Consulted**: `shiori.rs` L20-49, `tracing-subscriber` ドキュメント
- **Findings**:
  - `init_tracing_with_config()` は `tracing_subscriber::registry().try_init()` を使用（L38）
  - `try_init()` は既に subscriber が設定済みの場合、`Err` を返すだけで panic しない（ `let _ =` で無視）
  - つまり最初の呼び出しのみが有効、以降は安全に無視される
  - ただし、`PastaLoader::load()` 内部の Phase 6 でロガー（`PastaLogger`）が作成され `GlobalLoggerRegistry` に登録される。subscriber 初期化とロガー登録は独立した処理
- **Implications**:
  - 早期の `init_tracing_with_config()` 呼び出しは安全
  - ただし早期呼び出し時点では `PastaLogger`（ファイルライター）が未登録 → `GlobalLoggerRegistry` がファイルに書けない
  - **解決策**: `PastaShiori::load()` で `PastaLoader::load()` 前に `PastaLogger` を先に作成・登録し、その後 `init_tracing_with_config()` を呼ぶ

### `RawShiori<T>::load_impl()` のエラー保持問題

- **Context**: Req1 — load 失敗時のエラーをどこに保持し、`request()` でどう返すか
- **Sources Consulted**: `windows.rs` L140-152, `shiori.rs` L105-185
- **Findings**:
  - `load_impl()` の流れ:
    1. `*guard = None` で既存インスタンスを破棄
    2. `T::default()` で新規インスタンス作成
    3. `shiori.load(hinst, dir)?` を呼び出し（`?` で `MyResult<bool>` をアンラップ）
    4. `*guard = Some(shiori)` で保持
    5. `Ok(rc)` を返す（`rc` が `false` でも `Some(shiori)` が設定される）
  - `PastaShiori::load()` は `Err(e)` を `Ok(false)` に変換（L181-186）して返すため、`load_impl()` の `?` では `Err` に到達しない
  - つまり `*guard = Some(shiori)` は**常に設定される**（load 失敗時も）
  - `request_impl()` では `guard == None` のときだけ `NotInitialized` を返す → load 失敗した `PastaShiori` でも `Some` なので `NotInitialized` ではなく別のエラー経路に入る
  - **しかし** `runtime` が `None` のため `self.runtime.as_ref().ok_or(MyError::NotInitialized)?` で `request()` 内部で `NotInitialized` が返る
- **Implications**:
  - `PastaShiori` に `last_load_error: Option<String>` フィールドを追加
  - `load()` の `Err(e)` ブランチでエラーメッセージを保持
  - `request()` で `runtime.is_none()` かつ `last_load_error.is_some()` の場合、`MyError::Load(msg)` を返す

### `PastaLoader` ロガー作成の分離可能性

- **Context**: Req2 — ロガーを `PastaLoader` 内部（Phase 6）から外部に分離できるか
- **Sources Consulted**: `loader/mod.rs` L160-165, `logging/logger.rs`, `logging/registry.rs`
- **Findings**:
  - `PastaLogger::new(base_dir, config)` は `base_dir` と `LoggingConfig` のみに依存
  - `LoggingConfig` のデフォルト値は `Default::default()` で取得可能
  - Phase 6 のロガー作成は `PastaConfig` の `logging()` セクションを使用
  - `PastaConfig` は Phase 1 で読み込まれる
  - **問題**: `PastaLoader::load()` の本体で `PastaConfig` が確定するのは Phase 1 完了後
  - **解決案A**: `PastaShiori::load()` でデフォルト `LoggingConfig` でロガーを先に作成（`pasta.toml` 読み込み前）
  - **解決案B**: `PastaLoader::load()` 内の Phase 順序を変更し、Phase 1（config読み込み）直後にロガーを作成
- **Implications**:
  - 解決案B が適切: Phase 1 の config 読み込み直後（Phase 2 の前）にロガーを作成すれば、`pasta.toml` の `[logging]` 設定が反映される
  - ただし `init_tracing_with_config()` は `PastaShiori::load()` 側で呼ぶ必要がある（subscriber の初期化は1回のみ）
  - **最終方針**: `PastaShiori::load()` で `PastaLoader::load()` 前にデフォルト config でロガーとsubscriberを初期化。`PastaLoader` 内部の Phase 6 でロガーが再作成されるが、`GlobalLoggerRegistry` に登録済みのロガーが上書きされるだけ

### `tracing_subscriber::reload::Layer` によるフィルター動的更新

- **Context**: 2段階初期化アプローチでは、Stage 1（デフォルト config）のフィルターを Stage 1.5（pasta.toml 読み込み後）で更新する必要がある。`try_init()` では subscriber 全体を再作成できないため、フィルターのみを動的更新する手段が必要
- **Sources Consulted**: `tracing-subscriber` 0.3 ドキュメント、`tracing_subscriber::reload` モジュール
- **Findings**:
  - `reload::Layer::new(inner_layer)` が `(reload::Layer<L, S>, reload::Handle<L, S>)` を返す
  - `reload::Handle::modify(f)` でレイヤーの内容をクロージャで動的更新可能
  - `reload::Handle::reload(new_layer)` でレイヤー全体を差し替えることも可能
  - `Handle` は `Send + Sync` を実装しているため `OnceLock` での保存が可能
  - 具体的な型 `reload::Handle<L, S>` の `S` は購読者構成によって決まるため、型エイリアスまたは `Box<dyn FilterUpdater>` trait で消去するアプローチを検討
- **Implications**:
  - `init_tracing_with_config()` を `init_tracing_with_reload()` に更新し、`reload::Layer` を使用して handle を `pasta_lua::logging` の module-level `OnceLock` に保存
  - `update_tracing_filter(config: &LoggingConfig)` 関数を `pasta_lua::logging` に追加し、`OnceLock` から handle を取得してフィルターを更新
  - `pasta_shiori` は `init_tracing_with_reload()` を Stage 1 で呼び出し、`pasta_lua` 内の `PastaLoader` は `update_tracing_filter()` を Stage 1.5 で呼び出す

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 既存拡張 | `PastaShiori` にフィールド追加 + 早期ログ初期化 + `process_incremental` エラー伝搬 | 変更最小、既存パターン準拠 | `PastaShiori` の責務微増 | **採用** |
| B: ロガー分離 | `PastaLoader` から Phase 6 を外部に移動 | ライフサイクル明確 | API変更が大きい、`from_loader_with_scene_dic` の引数変更 | 不採用 |

## Design Decisions

### Decision: 早期ロガー初期化のタイミング（2段階アプローチ）

- **Context**: load 失敗時でもログファイルに記録を残しつつ、`pasta.toml` の `[logging].level/filter` 設定を正しく反映させたい
- **Alternatives Considered**:
  1. `pasta.toml` 読み込み後にのみ初期化 → `pasta.toml` パース失敗がファイルログに残らない
  2. デフォルト config で早期初期化のみ（`try_init()` ロック） → `pasta.toml` のフィルター設定が永久に無効
  3. **2段階アプローチ**: Stage 1（早期デフォルト初期化） + Stage 1.5（`pasta.toml` 読み込み後に writer/filter を両更新） ← 採用
- **Selected Approach**: 案３ — 2段階アプローチ
  - Stage 1: `PastaShiori::load()` でデフォルト `LoggingConfig` を使い早期初期化（`reload::Layer` 使用、handle を OnceLock に保存）
  - Stage 1.5: `PastaLoader` Phase 1 成功後、`GlobalLoggerRegistry` で writer を差し替え、`reload::Handle` 経由でフィルターを更新
- **Rationale**:
  - `GlobalLoggerRegistry` は `MakeWriter` 実装のため、subscriber 初期化後でも writer 差し替えが可能
  - `tracing_subscriber::reload::Layer` により、subscriber 初期化後でもフィルターの動的更新が可能
  - `pasta.toml` パース失敗もファイルログに記録され、かつ成功時は `[logging]` 設定が完全に反映される
- **Trade-offs**: `OnceLock<FilterHandle>` の追加と handle 型の複雑性（型エイリアスまたはボックス化が必要）
- **Follow-up**: `pasta.toml` で `[logging].file_path` をカスタマイズしている場合、早期ロガーのファイルパスと異なる可能性がある → 影響は軽微（早期ロガーは load 失敗時のみ重要）

### Decision: トランスパイル部分失敗時のロード中止

- **Context**: 実運用で .pasta 7件中3件がトランスパイル失敗 → scene_dic.lua が全7件 require → Phase 7 クラッシュ → ログなし
- **Alternatives Considered**:
  1. 中断（Err で即座にロード停止）
  2. 続行（現行動作 — 部分成功で起動）
  3. 設定可能（pasta.toml で切替）
- **Selected Approach**: 案1 — 中断
- **Rationale**: 構造的に部分成功での起動は不可能（`scene_dic.lua` が全モジュールを require するため）。明示的なエラー中止が正しいユーザー体験
- **Trade-offs**: 1件でも構文エラーがあるとゴーストが起動しなくなる → 開発中のデバッグを妨げる可能性あるが、エラー内容がログに記録されるため原因特定は容易
- **Follow-up**: なし（確定済み）

### Decision: `module_names` への失敗ファイル登録の扱い

- **Context**: `process_incremental()` が失敗ファイルの `module_name` も `scene_dic.lua` に含めている
- **Alternatives Considered**:
  1. 失敗ファイルを `module_names` から除外（部分成功起動を可能にする）
  2. 失敗時は即座に `Err` を返す（`module_names` は使われない）
- **Selected Approach**: 案2 — 即座に `Err` を返す
- **Rationale**: Req4 で部分失敗→ロード中止が確定しているため、`module_names` の修正は不要。`Err` が返された時点で `scene_dic.lua` 生成には到達しない
- **Trade-offs**: なし（シンプルな変更）
- **Follow-up**: なし

## Risks & Mitigations

- **早期ロガーのパス不一致**: `pasta.toml` でカスタムの `[logging].file_path` を設定している場合、早期ロガーのパスと異なる → load 失敗時のログは デフォルトパスに記録される。影響軽微（デフォルトの `profile/pasta/logs/pasta.log` は既知の場所）
- **subscriber の一度きり初期化**: tracing subscriber は `try_init()` で1回のみ初期化される。`reload::Layer` によりフィルターは動的更新可能だが、subscriber 構造自体は変更できない → 意図通りの制約

## Deep Dive Investigation（リファインメント追加調査）

### `reload::Layer` のフィルター配置変更

- **Context**: 現行の `init_tracing_with_config()` では `EnvFilter` が `fmt::layer().with_filter(filter)` として per-layer フィルターになっている
- **Finding**: `reload::Layer` の `reload()` メソッドで差し替えるには、フィルターを subscriber-global レイヤー（`registry().with(filter_layer)`）に昇格させる必要がある
- **Implication**: `fmt::layer()` はフィルターなしで全イベントを受け取り、フィルタリングは `reload::Layer` が担う。動作上の差異はなし（per-layer filter → global filter の移動のみ）

### Cargo.toml の `reload` feature 追加

- **Context**: `tracing-subscriber` の workspace dependency 確認
- **Finding**: 現行は `features = ["env-filter"]` のみ。`reload::Layer` を使用するには `"reload"` feature の追加が必須
- **Change**: `tracing-subscriber = { version = "0.3", features = ["env-filter", "reload"] }`

### `.lua` ファイル失敗の `failures` Vec 未収集問題

- **Context**: `process_incremental()` の `.lua` ファイルループ（loader/mod.rs L437-461）
- **Finding**: `.pasta` ファイルの失敗は `failures.push(TranspileFailure { ... })` で収集されるが、`.lua` ファイルの読み込み/コピー失敗は `stats.failed++` と `warn!()` のみで `failures` に追加されない
- **Implication**: `PartialTranspileError` で返す際に `.lua` ファイルの失敗詳細が欠落する。修正として `.lua` 読み込み失敗と `.lua` キャッシュ書き込み失敗を `failures.push()` に追加する
- **Note**: `TranspileFailure` 型名は `.pasta` トランスパイル固有ではなく汎用的なため、`.lua` 失敗を含めても構造的に問題ない

### `init_tracing_with_config()` の責務移動

- **Context**: `init_tracing_with_config()` は現在 `pasta_shiori/src/shiori.rs` L15-49 に定義されている
- **Finding**: `init_tracing_with_reload()` と `update_tracing_filter()` は `OnceLock<FilterHandle>` を共有する必要がある。`update_tracing_filter()` は `PastaLoader`（`pasta_lua` クレート内）から呼ばれるため、`OnceLock` は `pasta_lua` に置く必要がある
- **Decision**: 両関数を `pasta_lua::logging::tracing_init` モジュール（新規）に配置。`pasta_shiori` からは `pasta_lua::init_tracing_with_reload` として呼び出す
- **Benefit**: 循環参照を回避し、ロギング関連の責務を `pasta_lua::logging` に集約
- **既存テストへの影響**: `PastaLogger` の `Rotation::DAILY` → `Rotation::NEVER` への変更がテストで不具合を起こす可能性 → テストは `PastaLogger::new()` を直接使っており、Rotation への依存は軽微

## References

- [tracing-appender 0.2.4 rolling.rs](https://docs.rs/tracing-appender/0.2.4/src/tracing_appender/rolling.rs.html) — `Rotation::NEVER` のファイル名生成ロジック（L619-642）
- [tracing-subscriber try_init](https://docs.rs/tracing-subscriber/0.3/tracing_subscriber/util/trait.SubscriberInitExt.html#method.try_init) — 二重初期化の安全性
- [gap-analysis.md](gap-analysis.md) — 要件別ギャップ分析と実装アプローチ評価
