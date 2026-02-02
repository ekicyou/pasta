# Requirements Document

## Introduction

本仕様は、pasta.tomlの`[logging]`セクションを通じてGlobalLoggerRegistryのログ出力を制御する機能を定義します。`tracing-subscriber`の`EnvFilter`ディレクティブ構文を活用し、ターゲット別・レベル別のログフィルタリングをTOML設定で宣言的に記述できるようにします。

**アーキテクチャ決定:** tracing subscriber初期化を`PastaLoader::load()`完了後に遅延させることで、pasta.toml設定を即時反映する。`PASTA_LOG`環境変数はデバッグ用のオーバーライド手段として提供する。

## Project Description (Original Input)

GlobalLoggerRegistryのログ出力

### pasta.toml設定によるログレベル指定
- `[logging]`セクション
- デフォルトでDEBUG
- GlobalLoggerRegistryに対してTOMLで比較的簡単に指定できるログ出力内容

### ログレベル調整対象
- **TRACEに変更**: SHIORI関数キャッシュ・リクエスト処理詳細
- **WARNに変更**: 永続化ファイル未検出
- **INFOに変更**: SHIORI.unload成功
- **DEBUGログ追加**: 200 OKレスポンス時のリクエスト/レスポンス文字列

---

## Requirements

### Requirement 1: LoggingConfig拡張

**Objective:** As a ゴースト開発者, I want pasta.tomlの`[logging]`セクションでログレベルを宣言的に設定, so that コード変更なしにログ出力を調整できる

#### Acceptance Criteria

1. The LoggingConfig shall デフォルトレベルとして`debug`を使用
2. When `[logging].level`が設定されている, the LoggingConfig shall 指定されたレベル（error/warn/info/debug/trace）をデフォルトレベルとして使用
3. When `[logging].filter`が設定されている, the LoggingConfig shall EnvFilter互換のディレクティブ文字列としてパース
4. The LoggingConfig shall `filter`未設定時にデフォルトフィルタ文字列を生成

### Requirement 2: EnvFilterベースのログフィルタリング

**Objective:** As a ゴースト開発者, I want ターゲット（モジュールパス）別にログレベルを指定, so that 特定モジュールのログを抑制または詳細化できる

#### Acceptance Criteria

1. The pasta_shiori shall `tracing_subscriber::filter::EnvFilter`を使用してログをフィルタリング
2. When pasta.tomlに`filter = "debug,pasta_shiori=info"`が設定されている, the pasta_shiori shall デフォルトDEBUGレベルでpasta_shioriモジュールはINFO以上のみ出力
3. The EnvFilter shall カンマ区切りで複数のディレクティブを受け入れ（例: `"debug,target1=trace,target2=warn"`）
4. If `filter`設定が無効な場合, the pasta_shiori shall デフォルトフィルタにフォールバックしwarningをログ出力

### Requirement 3: デフォルトログレベル調整

**Objective:** As a ゴースト開発者, I want 頻出する内部ログが適切なレベルに設定されている, so that 通常運用時のログが見やすくなる

#### Acceptance Criteria

1. The pasta_shiori shall 以下のログをTRACEレベルで出力:
   - `SHIORI.load function cached`
   - `SHIORI.request function cached`
   - `SHIORI.unload function cached`
   - `SHIORI.load returned true`
   - `Processing SHIORI request`
   - `SHIORI.request completed`
2. The pasta_lua::runtime::persistence shall `Persistence file not found`をWARNレベルで出力
3. The pasta_shiori shall `SHIORI.unload called successfully`をINFOレベルで出力

### Requirement 4: SHIORIリクエスト/レスポンスログ

**Objective:** As a ゴースト開発者, I want 200 OKレスポンス時のリクエスト・レスポンス内容をDEBUGログで確認, so that 動作確認・デバッグが容易になる

#### Acceptance Criteria

1. When SHIORIリクエストが200 OKレスポンスを返す, the pasta_shiori shall リクエスト文字列をDEBUGレベルでログ出力
2. When SHIORIリクエストが200 OKレスポンスを返す, the pasta_shiori shall レスポンス文字列をDEBUGレベルでログ出力
3. The ログ出力 shall リクエスト/レスポンス文字列の長さ制限を設けない（フル出力）

#### Design Rationale

DEBUGレベルを採用することで、通常運用時（INFOレベル）ではログ出力を抑制し、デバッグ時のみフル詳細を確認できる。これによりログファイル肥大化を防ぎつつ、必要時には完全な情報が得られる。

### Requirement 5: pasta.toml設定スキーマ

**Objective:** As a ゴースト開発者, I want `[logging]`セクションの設定が明確に文書化されている, so that 設定方法を容易に理解できる

#### Acceptance Criteria

1. The `[logging]`セクション shall 以下のフィールドをサポート:
   - `file_path`: ログファイルパス（既存、デフォルト: `profile/pasta/logs/pasta.log`）
   - `rotation_days`: ログローテーション日数（既存、デフォルト: 7）
   - `level`: デフォルトログレベル（新規、デフォルト: `debug`）
   - `filter`: EnvFilterディレクティブ文字列（新規、オプション）
2. When `level`と`filter`の両方が設定されている, the LoggingConfig shall `filter`を優先
3. The pasta.toml shall `[logging]`セクション未設定時にデフォルト値を使用

### Requirement 6: tracing subscriber遅延初期化

**Objective:** As a ゴースト開発者, I want pasta.toml設定が起動時に即時反映される, so that 設定変更後の再起動で新しいログ設定が有効になる

#### Acceptance Criteria

1. The pasta_shiori shall `init_tracing()`を`PastaLoader::load()`完了後に実行
2. The init_tracing shall LoggingConfigからEnvFilterを構築
3. When `PASTA_LOG`環境変数が設定されている場合, the init_tracing shall 環境変数を優先（デバッグ用オーバーライド）
4. When EnvFilterの構築に失敗した場合, the init_tracing shall デフォルトのDEBUGレベルフィルタにフォールバック
5. The EnvFilter shall `tracing_subscriber::registry().with(fmt::layer().with_filter(env_filter))`パターンで適用

#### Design Decision

- DllMain時点ではtracing subscriberを設定しない（load_dirが不明でpasta.tomlが読めないため）
- DllMain〜load()間のログは出力されない（正常系では問題なし、エラー時のみ影響）
- init_tracing_with_config()完了直後に、load_dirをINFOログとして出力し、ロードディレクトリ情報を確実に記録

### Requirement 7: ロードディレクトリ情報の確実な記録

**Objective:** As a ゴースト開発者, I want tracing subscriber初期化前に失われたログ情報（特にロードディレクトリ）を確実に記録, so that ログファイルにゴースト起動情報が必ず残る

#### Acceptance Criteria

1. The pasta_shiori shall `init_tracing_with_config()`完了直後に、load_dirパスをINFOレベルでログ出力
2. The ログメッセージ shall ロードディレクトリの絶対パスを含む
3. The ログ出力 shall tracing subscriber初期化成功後の最初のログとして出力される

#### Design Rationale

DllMain〜init_tracing_with_config()間のログ（`info!("Starting PastaShiori load")`など）は失われるため、最低限の起動情報（ロードディレクトリ）をsubscriber初期化直後に出力することで、ログファイルに必ず記録されるようにする。

### Requirement 8: 後方互換性

**Objective:** As a 既存ゴースト開発者, I want 既存のpasta.toml設定が引き続き動作, so that 移行コストが発生しない

#### Acceptance Criteria

1. The LoggingConfig shall `level`/`filter`未設定時に既存の動作（全DEBUGログ出力）を維持
2. The `file_path`および`rotation_days`フィールド shall 既存の動作を変更しない
3. When 既存のpasta.tomlに`[logging]`セクションがない場合, the システム shall デフォルト設定で動作

## Technical Notes

### EnvFilterディレクティブ構文

```
target[span{field=value}]=level
```

**Examples:**
- `debug` - 全体をDEBUGレベル
- `warn,pasta_shiori=debug` - デフォルトWARN、pasta_shioriはDEBUG
- `debug,pasta_shiori::cache_lua_functions=trace` - 特定関数のみTRACE

### pasta.toml設定例

```toml
[logging]
file_path = "profile/pasta/logs/pasta.log"
rotation_days = 7
level = "debug"
# または詳細フィルタ
filter = "debug,pasta_shiori=info,pasta_lua::runtime::persistence=warn"
```

### 影響を受けるファイル

- `crates/pasta_lua/src/loader/config.rs` - LoggingConfig拡張
- `crates/pasta_shiori/src/windows.rs` - init_tracing EnvFilter統合
- `crates/pasta_shiori/src/shiori.rs` - ログレベル調整
- `crates/pasta_lua/src/runtime/persistence.rs` - ログレベル調整
