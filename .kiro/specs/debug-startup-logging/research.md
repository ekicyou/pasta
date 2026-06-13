# ギャップ分析: debug-startup-logging

> 既存コードベースと要件 (R1〜R4) の差分分析。設計フェーズの判断材料を提供する（最終決定は下さない）。
> 調査日: 2026-06-13 / 対象: `crates/pasta_lua/src/debug/`

## 1. 現状調査（Current State）

### 対象コードと挿入点
- **`enable()`**: `crates/pasta_lua/src/debug/mod.rs:564-692`。デバッグバックエンドの単一エントリポイント兼有効化ゲート。
  - **ゼロコスト無効パス**: `mod.rs:569-574`。`if !cfg.enabled { return Ok(None); }`。フック未装着・ポート未開放・スレッド未生成・`std_debug` 非公開。
  - **トランスポート起動**: `mod.rs:638` `let transport = Transport::start(cfg.listen)?;` → `mod.rs:639` `let local_addr = transport.local_addr();`。
- **`Transport::start`**: `transport.rs:223-257`。`Some(addr)` 時に `TcpListener::bind`（失敗は `?` で `DebugError::Bind` に伝播 = `mod.rs:638` で早期 return）、`listener.local_addr()` で**実バインドアドレスを読み戻し**て保持（`transport.rs:239`）。`None` 時は何も開かず `local_addr = None`。
- **`Transport::local_addr`**: `transport.rs:260-262`。戻り値は `Option<SocketAddr>`（無効時 `None`）。`Some(addr)` 時の成功経路でのみ `Some`。

### 既存ロギング慣習
- `debug` モジュールに**情報ログ (`info!`) は皆無**。警告 2 件のみ:
  - `mod.rs:98` `tracing::warn!`（不正な `sourcePresentation` フォールバック）。
  - `source_map.rs:523` `tracing::warn!`（サイドカー書込失敗）。
- ロギング基盤: `tracing 0.1` は `pasta_lua` の**既存直接依存**（`tech.md`）。`pasta.log` 出力は `logging/tracing_init.rs` / `tracing-appender` で構成済み。→ **新依存ゼロで `tracing::info!` を 1 行追加するだけ**。

### テスト基盤（テスト可能性 = 本仕様の最大の論点）
- **`tracing-test` が dev-dependency に存在**: `Cargo.toml:43` `tracing-test.workspace = true`。`tests/log/integration_test.rs:8` で `use tracing_test::traced_test;` の実績あり。
  - → `#[traced_test]` + `logs_contain("...")` で「ログが出る／出ない」を**ユニットテストで直接検証可能**。
- **既存テストアンカー**（`mod.rs` 内 `#[cfg(test)] mod tests`、いずれもポート 0 使用）:
  - `enable_enabled_returns_handle`（`mod.rs:829`）— 有効パス。R1/R2 ログ検証の拡張先。
  - `enable_disabled_returns_none_and_no_trace`（`mod.rs:813`）— 無効パス。R3 の「ログを出さない」検証の拡張先（現状はトレース捕捉していない＝名前倒れ。`#[traced_test]` 化で実効化できる）。
  - `enable_bind_failure_surfaces_debug_error_bind`（`mod.rs:867`）— バインド失敗。R2.5「失敗時は info ログを出さない」検証の拡張先。

## 2. 要件→アセット対応表（Requirement-to-Asset Map）

| 要件 | 既存アセット | ギャップ | タグ |
|---|---|---|---|
| R1 有効化 info ログ | 挿入点 `mod.rs:574` 直後（ゲート通過＝有効確定） | `tracing::info!` 1 行を追加するのみ | Missing（軽微） |
| R2.1/2.2 待ち受け info（実アドレス） | `mod.rs:639` `local_addr` 取得直後 | `info!` 1 行追加。`Option` を `if let Some`/`unwrap` で扱う設計判断 | Missing（軽微） |
| R2.3 OS割当(port 0)時は実値 | `transport.rs:239` で実バインド値を読み戻し済み | **能力は既存**。`local_addr` を出力すれば自動充足 | Satisfied |
| R2.5 バインド失敗時は出さない | `mod.rs:638` の `?` がログ前に短絡 | **配置で自動充足**（info を `:639` 以降に置く） | Constraint（配置順） |
| R3 無効時ゼロコスト・無言 | `mod.rs:569-574` 早期 return | 早期 return に**触れない**こと。ログは必ずゲート後に置く | Constraint |
| R4.1〜4.4 非回帰・新依存なし | `tracing` 既存依存・全テスト緑 | 機能挙動・ログ基盤不変。`cargo test --all` 維持 | Constraint |

## 3. 実装アプローチ選択肢

### Option A: `enable()` 内に `tracing::info!` を 2 行インライン追加（推奨）
- **内容**: ゲート通過直後（`mod.rs:574` 後）に R1 ログ、`local_addr` 取得後（`mod.rs:639` 後）に R2 ログを直接記述。既存 `tracing::warn!` と同じ慣習。
- **Trade-offs**:
  - ✅ 最小差分・既存パターン踏襲・新ファイル/関数なし・配置だけで R2.5/R3 を自動充足。
  - ✅ ゼロコスト無効パスに一切触れない（R3 厳守が自明）。
  - ❌ ほぼ無し（2 行追加のため）。

### Option B: ログ整形を小ヘルパー関数に切り出し
- **内容**: `fn log_listening(addr: Option<SocketAddr>)` 等に整形ロジックを分離。
- **Trade-offs**:
  - ✅ ログ文言の単体テストが容易。
  - ❌ 2 行のために関数を作るのは過剰。`#[traced_test]` で直接検証できるため不要。**非推奨**。

### Option C: ハイブリッド（将来の attach/切断ログ拡張を見据えた構造化）
- **内容**: 起動ログを構造化フィールド (`tracing::info!(addr = %addr, ...)`) で出し、将来の逐次ログと整合する形にする。
- **Trade-offs**:
  - ✅ 将来拡張に整合。構造化フィールドはコスト無し。
  - ❌ 本仕様の Out of scope（attach/切断逐次ログ）に踏み込むと過設計。**起動ログ限定なら Option A に構造化フィールドだけ採り入れるのが妥当**。

## 4. 複雑度・リスク

- **Effort: S（1 日未満）** — 既存パターン（`tracing::warn!`）に倣い `info!` を 2 行追加＋検証テスト 3 件拡張。新依存・新ファイルなし。
- **Risk: Low** — 挿入点・実バインドアドレス読み戻し・テスト基盤すべて既存で確認済み。無効パスは早期 return で物理的に隔離されている。

## 5. 設計フェーズへの申し送り

### 推奨アプローチと主要判断
- **Option A** を基線に、R2 ログは Option C 由来の構造化フィールド（`addr = %addr`）を 1 つ採用。
- **配置で要件を満たす**: R1 ログは `mod.rs:569-574` のゲート**通過後**、R2 ログは `mod.rs:639`（`local_addr` 取得）**以降**に置く。これだけで R2.5（失敗時は出さない）と R3（無効時は無言）が構造的に保証される。

> **注意（2026-06-13 要件ディスカッション反映）**: 要件は「2 段ログ（有効化＋待ち受け）」から**待ち受け開始 info ログ 1 本に統合**し、**バインド失敗時は warn ログ 1 件**を出す方針へ変更された。これに伴い、本節・上記マップ・Option 群の「R1（有効化専用ログ）」前提は無効化されている。最新の正は requirements.md（R1=待ち受け info、R2=失敗 warn、R3=無効時無言、R4=非回帰）。下記の確定すべき判断はこの新方針に読み替えること。

### 設計で確定すべき判断（Decisions for design）
1. **バインド失敗時 warn の発火点（R2.2）**: `mod.rs:638` の `Transport::start(cfg.listen)?` は失敗を `?` で短絡するため、warn を出すには `.map_err(|e| { tracing::warn!(...); e })?` 等で失敗事由（`io::Error`）と試行アドレス（`cfg.listen`）を握ってからログする。`info`（成功）は従来どおり `:639` 以降に置く。配置順で R2.1（失敗時 info を出さない）が自動充足される。
2. **`Option` 処理（R1.4 成功側 / R2.3 失敗側）**: 成功 `info` の `local_addr` は成功経路で必ず `Some` → `if let Some(addr)` で握る（防御的）。失敗 `warn` の `cfg.listen` も `Option<SocketAddr>` で `%`(Display)不可 → 有効ゲート通過済み＝`Some` 確定を前提に分割代入してから `%addr` で出す（`?cfg.listen` の `Some(...)` 表記は非推奨）。両側とも Display を効かせるため Option を先に剥がす。
3. **ログ文言（R1.3 / R2.3）**: 待ち受け info は「デバッグモードで待ち受け開始」と識別でき、loopback `host:port` のみ含む（秘密情報非混入）。失敗 warn は試行アドレス＋失敗事由のみ。**言語は簡潔な英語に確定**（議題 2 / 2026-06-13。既存 tracing ログと一貫）。例: info `debug backend listening on 127.0.0.1:9276` / warn `debug transport bind failed on 127.0.0.1:9276: {err}`。具体文言を design で最終確定。

### Research Needed（設計時に確認）
- **`#[traced_test]` × 既存 env ガードの整合**: `enable()` を叩くテストは `PASTA_DEBUG` 環境変数の汚染で固定ポート枯渇する既知問題あり（crate 全体に `#[ctor]` 中和ガードが入っているため新テストも自動適用される想定だが、新規ログ検証テスト追加時に再確認）。テストはポート 0 を使用すること。
- **`#[traced_test]` のグローバルサブスクライバ**: `pasta.log` 用の本番サブスクライバ初期化と衝突しないか（ユニットテストは本番初期化を経由しないため通常問題ないが念のため確認）。

---

## 6. 設計 synthesis（2026-06-13 / design フェーズ）

要件群に対し generalization / build-vs-adopt / simplification の 3 レンズを適用した結果。

### Generalization
- 旧 R1（有効化検知ログ）と旧 R2（待ち受け開始ログ）は「起動観測性」という同一問題の変種だった。バインド成功＝有効化確定であるため、**待ち受け開始 info ログ 1 本に一般化**し、有効化とポートの双方をそれで兼ねる（議題 1 の決定と一致）。失敗系のみ独立要件（warn）として分離。

### Build vs. Adopt（すべて Adopt）
- ログ出力: 既存 `tracing 0.1`（`pasta_lua` 直接依存）を採用。新規ロギング機構は構築しない。
- 実バインドアドレス取得: 既存 `Transport::local_addr()`（OS 割り当て値の読み戻し済み）を採用。新規取得経路を作らない。
- テスト: 既存 dev-dependency `tracing-test`（`#[traced_test]` / `logs_contain`）を採用。カスタムログ捕捉を作らない。

### Simplification
- **Option B（ログ整形ヘルパー関数の切り出し）を却下**: 2 箇所の `tracing` マクロ呼び出しに集約し、新規関数・モジュール・抽象を導入しない。`#[traced_test]` で文言を直接検証できるためヘルパー不要。
- 新コンポーネント・新ファイルはゼロ。`enable()` 内の既存制御フロー上の 2 点への観測挿入に限定（最小差分）。
- 設計判断: 成功 `info` は `if let Some(addr)` で実アドレスを握る／失敗 `warn` は `Transport::start(...).map_err(|e| { warn!; e })?` で伝播挙動を変えずにログのみ追加。
