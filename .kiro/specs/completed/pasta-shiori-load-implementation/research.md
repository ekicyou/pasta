# Research & Design Decisions

## Summary
- **Feature**: `pasta-shiori-load-implementation`
- **Discovery Scope**: Extension（既存システム拡張）
- **Key Findings**:
  - tracingのグローバルSubscriber制約により、複数インスタンスのログファイル分割は標準的なアプローチでは困難
  - tracing_appenderのRollingFileAppenderはrotation_daysを直接サポートしない（daily/hourly/minutelyのみ）
  - PastaLoaderインスタンスごとの独立ログ実装には、tracingを使わずファイル直接書き込みが最もシンプル

## Research Log

### トピック1: tracingの複数ファイル出力アーキテクチャ

- **Context**: 複数のPastaLoaderインスタンスが独立したログファイルに出力する要件
- **Sources Consulted**: 
  - https://docs.rs/tracing-subscriber/latest/tracing_subscriber/
  - https://docs.rs/tracing-appender/latest/tracing_appender/
- **Findings**:
  - `tracing_subscriber::set_global_default()`はプロセス全体で1回のみ呼び出し可能
  - 複数ファイル出力にはLayerスタックとFilterを組み合わせる必要あり
  - 動的にLayerを追加する場合は`reload`機能が必要（複雑性増加）
  - SpanフィールドでフィルタリングしてWriter振り分けは技術的に可能だが複雑
- **Implications**: 
  - tracingをグローバルロガーとして使用しながら、インスタンスごとの独立ファイル出力は標準的ではない
  - 代替案: ファイル直接書き込み（`std::fs::File`）、またはSpanフィールドで識別して1ファイルに統合

### トピック2: tracing_appenderのローテーション戦略

- **Context**: rotation_days（N日保持）の実装方法
- **Sources Consulted**: 
  - https://docs.rs/tracing-appender/latest/tracing_appender/rolling/
- **Findings**:
  - `RollingFileAppender`はRotation::DAILY, HOURLY, MINUTELY, NEVERをサポート
  - max_log_files設定でファイル数上限を指定可能
  - rotation_days=7の場合、`Rotation::DAILY`と`max_log_files=7`の組み合わせで実現
  - ファイル名パターン: `prefix.log.YYYY-MM-DD`形式
- **Implications**: 
  - rotation_daysはmax_log_filesにマッピングして実装可能
  - 古いログの自動削除はtracing_appenderが処理

### トピック3: ログファイル分割アプローチ比較

- **Context**: 各PastaLoaderインスタンスが独立したログファイルを持つ実装方法
- **Findings**:
  
  **Option 1: tracingグローバル + Layer/Filter分割（複雑）**
  - グローバルSubscriberに動的Layerを追加
  - Spanフィールド（ghost_dir）でフィルタリング
  - 複雑性: 高、reload機能必要
  
  **Option 2: tracing使用せず、独自FileLogger（シンプル）**
  - 各PastaLoaderがファイルハンドルを保持
  - tracingマクロは使えない（info!, debug!等不可）
  - 複雑性: 中、独自ログAPIが必要
  
  **Option 3: tracing + インスタンスローカルWriter（推奨）**
  - 各インスタンスがMakeWriter実装を持つ
  - グローバルSubscriberは1回設定、Writerが動的に出力先を決定
  - 複雑性: 中、MakeWriter trait実装が必要

- **Implications**: Option 3を推奨。tracingエコシステムを活用しながらインスタンスごとの出力を実現

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: グローバルSubscriber + SpanFilter | Span情報でファイル振り分け | tracingの標準機能活用 | 複雑、reload必要、設定変更困難 | 複数ゴースト同時起動時に複雑化 |
| B: インスタンスごとFileLogger | 各インスタンスが独自ファイルハンドル | シンプル、独立性高 | tracingマクロ使用不可 | 既存コードのtracing!呼び出しが無効化 |
| C: ThreadLocal MakeWriter | スレッドローカルでWriter切り替え | tracingマクロ継続使用可 | スレッド境界の考慮必要 | 実装複雑性中 |
| **D: 独自Logger + tracingラッパー** | 内部でファイル書き込み、tracingはログレベルフィルタのみ | シンプル、独立性高 | 二重ロギング回避が必要 | **推奨** |

## Design Decisions

### Decision: ログ出力アーキテクチャ

- **Context**: 伺かは1プロセス内で複数ゴーストを同時起動。各ゴーストが独立したログファイルを持つ必要がある。
- **Alternatives Considered**:
  1. tracingグローバルSubscriber + Layer/Filter — 標準的だが動的追加が複雑
  2. tracingを使わず独自FileLogger — シンプルだがtracingエコシステム離脱
  3. インスタンスごとのPastaLogger構造体 — 各インスタンスがファイルハンドルとSpanを保持
- **Selected Approach**: Option 3（インスタンスごとのPastaLogger）
  - 各PastaLuaRuntimeがPastaLoggerインスタンスを所有
  - PastaLoggerがファイルハンドルとローテーション管理
  - tracingマクロからの呼び出しはSpan経由でPastaLoggerにディスパッチ
- **Rationale**: 
  - 各インスタンスの独立性を確保しながらtracing APIを活用
  - グローバルSubscriberへの依存を最小化
  - ローテーション実装が自己完結
- **Trade-offs**: 
  - MakeWriter実装が必要（中程度の複雑性）
  - グローバルログ（全インスタンス共通）は別途対応必要
- **Follow-up**: 
  - MakeWriter traitの詳細実装を設計フェーズで確定
  - WorkerGuard管理（非同期フラッシュ）の確認

### Decision: ローテーション実装

- **Context**: rotation_daysパラメータで古いログを自動削除する必要がある
- **Selected Approach**: tracing_appender::rolling::dailyとmax_log_files組み合わせ
- **Rationale**: 
  - tracing_appenderが標準でmax_log_filesをサポート
  - rotation_days=7の場合、max_log_files=7で7日間のログを保持
  - 古いログはtracing_appenderが自動削除
- **Trade-offs**: 
  - 日付ベースローテーションのみ（サイズベースは別途実装必要）
  - ファイル名パターンがtracing_appender標準に固定

### Decision: エラー変換戦略

- **Context**: LoaderError → MyError変換が必要
- **Selected Approach**: From<LoaderError> for MyError実装
  - 各LoaderErrorバリアントをMyError::Load(String)にマッピング
  - 詳細メッセージにエラー原因を含める
- **Rationale**: 
  - シンプルな実装で十分
  - X-ERROR-REASONにデバッグ情報を格納可能
- **Trade-offs**: 
  - LoaderErrorの構造情報は文字列化により損失
  - パターンマッチングによる詳細処理は不可（必要になれば拡張）

## Risks & Mitigations

- **複数インスタンスのログ競合** — 各インスタンスが独立したファイルパスを使用することで回避
- **ログファイル書き込み権限** — ディレクトリ自動作成、権限エラー時はログ出力をスキップ（panic回避）
- **ローテーション中のデータ損失** — tracing_appenderのnon_blocking + WorkerGuardで非同期フラッシュ保証
- **グローバルSubscriber競合** — OnceLockで初回のみ設定、2回目以降は既存Subscriberを再利用

## References

- [tracing-subscriber Layer documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html)
- [tracing-appender RollingFileAppender](https://docs.rs/tracing-appender/latest/tracing_appender/rolling/struct.RollingFileAppender.html)
- [tracing-subscriber Filter trait](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Filter.html)
- [MakeWriter trait documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/writer/trait.MakeWriter.html)
