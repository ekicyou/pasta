# Gap Analysis: pasta-debug-break-coalesce

生成日時: 2026-06-08T13:54:07Z

## 分析サマリ

- **バグは局所的で原因確定済み**: `on_line_impl` の「Breakpoint-first」分岐（`crates/pasta_lua/src/debug/session.rs:639`）が、提示モードに関わらず**全 `.lua` 行で `should_pause()` を呼び、`.pasta` 行集約を持たない**。同等の集約はステップ経路（`pasta_step_should_stop`）には既にある。
- **統合の縫い目が既存**: `DebugSession` は `source_map: Option<Arc<SourceMap>>`・`effective_mode() -> SourceMode`・`resolve_current_pasta(source, line) -> Option<PastaPos>` を保持。`mode` は `RefCell`（VMスレッド単一・`&self` 内部可変）。抑制アンカー用の `RefCell` フィールド追加が自然・レースフリー。
- **推奨は Option A（`on_line_impl` の BP 分岐を拡張）**: ステップ側の集約パターンを BP/Continue 経路へ対称適用。新規ファイル不要・既存ヘルパー再利用・モード gating（`effective_mode()==Pasta && source_map.is_some()`）を流用して `.lua`/map無し/OFF を不変に保てる。
- **設計の肝はアンカーのライフサイクル**: 「直前停止の `.pasta` 行を*離れるまで*抑制、別の対応 `.pasta` 行へ移ったら解除（ループ・再帰で再訪したら再停止）」。永久抑制にしない／同一訪問内の未対応行挟みで誤再停止しない、の両立が唯一の非自明点。
- **検証資産あり**: 実 DAP-over-TCP E2E ハーネス（`wiring.rs` の統合テスト群）・`runtime/debug_integration_test.rs`・`session.rs` 単体テスト群が揃い、R6 の証跡を既存様式で取得可能。
- **規模/リスク**: Effort **S**、Risk **Low〜Medium**（唯一の不確実性はアンカーのフレーム/コルーチン修飾要否＝下記「要決定」）。

## 1. 現状調査（Current State）

### 関連モジュール（`crates/pasta_lua/src/debug/`）
| ファイル | 役割 | 本仕様との関係 |
| --- | --- | --- |
| `session.rs` | 停止状態機械（protocol 非依存）。`on_line_impl`・`stop_loop`・`RunMode`・ステップ集約 | **主変更点**。BP/Continue 経路へ `.pasta` 行集約を追加 |
| `breakpoints.rs` | 共有 BP ストア。二段キー（present_source / 実行座標 `(chunk, lua_line)`）。`should_pause` は実行座標一致のみ判定 | 参照。`.pasta` 1行→複数 `.lua` 行を全登録（8.2、テスト済み）する正常仕様の前提 |
| `source_map.rs` | `.lua`↔`.pasta` 双方向写像。`resolve_lua_to_pasta`・`canonicalize_chunk_name`・`PastaPos` | 参照。集約判定の `.pasta` 位置解決に使用 |
| `mod.rs` | `SourceMode {Pasta, Lua}`・`SharedSourceMode`・`enable` 配線 | 参照。モード gating の源 |
| `wiring.rs` | DAP 配線・実ソケット E2E テスト群 | 検証資産（R6.2/R6.3） |
| `dap.rs` | DAP source/stackTrace（提示レゾルバ） | **本仕様の関心外**（隣接 `pasta-debug-lua-view-toggle`） |

### 確定した制御フロー（`session.rs:634` `on_line_impl`）
1. `let (source, line) = source_and_line(debug)`。
2. **Breakpoint-first（639）**: `if self.breakpoints.should_pause(&source, line) { return stop_loop(StopReason::Breakpoint) }` — **モード非依存・全行・集約なし**。← バグの所在。
3. **Stepping（648–686）**: `step_should_stop`（`.lua` 粒度）が true のとき、`effective_mode()==Pasta && source_map.is_some()` なら `pasta_step_should_stop`（`.pasta` 粒度集約）で `take_stop` を精製。← 既に正しい集約あり。
4. `Continue`（500–503）: `*mode = RunMode::Running` に戻すのみ（ステップ状態を解除）。

### 既存の集約ロジック（再利用すべき手本）`pasta_step_should_stop`（`session.rs:433`）
- `(1)` 現 `.lua` 行が `.pasta` 未対応（`cur_pasta==None`）→ 通過。
- `(2)` 同一起点フレーム（`cur_thread==thread && depth==base_depth`）かつ `origin_pasta==Some(cur)` → 同一 `.pasta` 行を消化（継続）。
- `(3)` それ以外 → 停止。
- 起点 `.pasta` は `RunMode::Stepping` の `origin_pasta` に保持。フレーム同一性は `(thread, base_depth)` で判定。

### 規約・テスト配置
- Rust スネークケース、`Result<T, PastaError>`、テストは同ファイル `#[cfg(test)]` ＋ `crates/pasta_lua/tests/` 配下。
- 既存テスト: `session.rs:1484+`（単体）、`tests/runtime/debug_integration_test.rs`、`tests/runtime/source_map_handoff_test.rs`、`wiring.rs` 実ソケット E2E。
- OFF 経路: デバッグ無効時はラインフック自体が未設置（`on_line_impl` は実行されない）→ R4.3 バイト不変は構造的に保証。変更は `on_line_impl` 内に閉じる。

## 2. 要件→資産マップ（ギャップ）

| 要件 | 必要能力 | 既存資産 | ギャップ |
| --- | --- | --- | --- |
| R1 Continue 離脱 | BP/Continue 経路の `.pasta` 行集約 | `resolve_current_pasta`・`effective_mode`・`mode(RefCell)` | **Missing**: BP 分岐に集約判定が無い／「現在抑制中の `.pasta` 行」状態の保持が無い |
| R2 1訪問1停止・再訪再停止 | 抑制アンカーのライフサイクル（離脱で解除・再訪で再停止） | ステップ側 `origin_pasta` パターン | **Missing**: BP 経路にはアンカー状態が無い（新規 `RefCell` フィールド）。**要決定**: アンカーのフレーム/コルーチン修飾要否（再帰エッジ） |
| R3 停止位置・理由一貫性 | `.pasta` ソース・行提示＋Breakpoint 理由 | `stop_loop(StopReason::Breakpoint)`・既存提示レゾルバ | **Constraint**: 既存提示は不変。集約は「停止を抑制（イベント不発生）」のみ行い提示経路に触れない |
| R4 モード直交・後方互換 | `.lua`/map無し/OFF 不変 | `effective_mode()==Pasta && source_map.is_some()` gating（ステップ側で実績） | **Low**: 同一 gating を流用すれば充足 |
| R5 ステップとの一貫性 | Continue とステップで「現在の `.pasta` 行」概念を共有 | `pasta_step_should_stop`（既存） | **Unknown**: 共通ヘルパー抽出で一貫させるか、独立実装にするか（設計判断） |
| R6 無回帰・検証 | 多対1構成・ループ構成の実 DAP-over-TCP E2E | `wiring.rs` E2E ハーネス・`debug_integration_test.rs` | **Low**: 既存ハーネスに多対1＋ループの fixture シナリオを追加 |

## 3. 実装アプローチ選択肢

### Option A: `on_line_impl` の BP 分岐を拡張（推奨）
- **内容**: `DebugSession` に `last_break_pasta: RefCell<Option<PastaPos>>`（命名は設計時）を追加。`on_line_impl` の BP-first 分岐で、`effective_mode()==Pasta && source_map.is_some()` のとき `cur_pasta = resolve_current_pasta(source,line)` を求め、
  - `should_pause` が true でも `cur_pasta == Some(anchor)` なら停止を抑制（継続）、
  - `cur_pasta` が `Some(別の対応行)` に変化したらアンカーを解除/更新（ループ・再訪で再停止可能化）、
  - 実際に停止する時はアンカーを停止行に設定。
  - `.lua` モード/map無し時は従来どおり（gating で素通し）。
- **トレードオフ**: ✅ 局所的・新規ファイル不要・既存ヘルパー再利用・ステップ集約と対称で理解容易。✅ BP 全登録（8.2）の信頼性を保ったまま停止側だけ集約。❌ `on_line_impl` に分岐が増える（ただしステップ側と同型）。
- **Effort S / Risk Low〜Medium**。

### Option B: BP 登録を `.pasta` 行の代表 `.lua` 行に絞る
- **内容**: `breakpoints.rs` の `.pasta` 登録経路で、1 `.pasta` 行につき代表1 `.lua` 行のみ登録。
- **トレードオフ**: ✅ 再ヒットの根を断つ。❌ **重大リスク**: `should_pause` は実行座標一致なので、代表 `.lua` 行が分岐等で実行されないと BP が**一度も発火しない**。❌ 既存テスト `one_present_line_registers_multiple_execution_coords`（8.2）と衝突。❌ どの行を代表にするか判定が脆い。**非推奨**。

### Option C: ハイブリッド（A ＋ 共通集約ヘルパー抽出）
- **内容**: Option A を採りつつ、「現在 `.pasta` 行 vs アンカー」の判定を Continue とステップ（`pasta_step_should_stop`）で共有できる小ヘルパーへ括り出し、R5 の一貫性を構造で担保。
- **トレードオフ**: ✅ R5 を明示的に満たす・将来の挙動ぶれ防止。❌ 既存ステップ集約のフレーム同一性条件（`(thread, base_depth)`）と BP 経路（フレーム情報なし）の差異を吸収する抽象が必要で、過度な共通化は逆に複雑化。**A を基本に、共通化は設計で費用対効果を判断**。

## 4. 規模・リスク

- **Effort**: **S（1〜3日）**。変更は `session.rs` の 1 フィールド＋ `on_line_impl` BP 分岐＋単体/統合/E2E テスト。既存パターン（ステップ集約）を踏襲。
- **Risk**: **Low〜Medium**。技術的不確実性は低い（縫い目が全て既存）。唯一の Medium 要因はアンカーのライフサイクル設計（下記要決定）の正しさ。

## 5. 設計フェーズへの申し送り

### 推奨アプローチ
- **Option A** を基本線とする。R5 の一貫性は Option C の共通ヘルパー抽出を「費用対効果が見合えば」採用（過度な共通化は避ける）。

### 要決定（設計で確定すべき事項）
1. **アンカーのフレーム/コルーチン修飾**: BP/Continue 経路にはステップ側の `(thread, base_depth)` フレーム同一性が無い。**【要件ディスカッション 2026-06-08 決定】保証対象は「ループ再訪」のみ（R2.2）。「同一 `.pasta` 行への直接再帰」「同一 `.pasta` 行の別コルーチン実行」での訪問ごと再停止はベストエフォート（R2.3・厳密保証なし）。** → `.pasta` 行のみのアンカー（フレーム/コルーチン修飾なし）を基本線として採用可能。実用上、関数呼び出しで実行 `.pasta` 行は一旦別行へ遷移するためアンカーは解除され、再帰でも多くの場合は再停止する見込みだが、これは保証ではなく結果論として扱う。設計はフレーム修飾を避ける最小実装を優先し、ベストエフォートで足りるか確認する。
2. **アンカー解除条件**: 「`cur_pasta` が `Some(別行)` に変化したときのみ解除」を基本とする（同一 `.pasta` 行の展開に未対応行 `None` が挟まる場合に同一訪問内で誤再停止しないため、`None` では解除しない）。この方針の妥当性とエッジを設計で確認。
3. **アンカーの確立タイミングと初期到達**: 「初めて当該 `.pasta` 行に到達した BP ヒットでは停止し、その時アンカーを設定」「Continue 後の同一行は抑制」を、`mode`（Running/Stepping）と組み合わせてどう実現するか（停止時設定 vs 到達時設定）を確定。
4. **ステップ⇔Continue の状態共有**: アンカーとステップの `origin_pasta` を統合するか分離するか（R5・Option C）。

### 持ち越す調査項目（Research Needed）
- 実 DAP-over-TCP E2E に「1 `.pasta` 行→複数 `.lua` 行」かつ「同一 `.pasta` 行をループ再訪」する fixture を追加する具体シナリオ設計（R6.2/R6.3）。**確定: 既存ハーネスは `crates/pasta_lua/tests/runtime/debug_integration_test.rs`（`DapClient`・`enabled_runtime_persists_breakpoint_across_requests` が BP→continue→再ヒットの様式）。ここへ多対1＋ループ fixture を追加する。**
- コルーチン跨ぎ（シーンコルーチン）での BP 再訪挙動が既存ステップ集約（コルーチン跨ぎ実証済み）と矛盾しないことの確認シナリオ。

---

## 設計合成（Design Synthesis・2026-06-08）

### 1. 一般化（Generalization）
- R1（Continue で行を抜ける）・R2（1訪問1停止・ループ再訪再停止）・R3.2（後続 `.lua` 行の停止イベント抑制）・R5（ステップとの一貫性）は、いずれも「**`.pasta` 行粒度で停止し、同一 `.pasta` 行の連続ヒットを消化する**」という単一能力の変奏。ステップ経路は `RunMode::Stepping.origin_pasta`（フレーム修飾済み）で既に実現。一般化された能力＝「直前停止の `.pasta` 行を離れるまで BP 再ヒットを消化するアンカー」を **Running/BP 経路へ追加**する（`pasta_break_anchor`）。
- インターフェースのみ一般化し実装は最小：アンカーは `Option<PastaPos>` 1個。

### 2. Build vs Adopt
- 外部ライブラリ不要。既存資産を全面再利用：`resolve_current_pasta`・`effective_mode`・`PastaPos`・`source_map`・`RefCell` 内部可変（`mode` と同型のスレッドモデル）。
- **ステップ `origin_pasta` との状態共有は行わない（要決定#4 確定＝分離）**。理由：ステップ側は `(thread, base_depth)` フレーム同一性で修飾するが、BP/Continue 経路は**フレーム修飾なし**（要件ディスカッション決定＝ループ保証・再帰ベストエフォート）。両者を1つの抽象へ強制共通化すると、フレーム修飾の有無差を吸収する条件分岐が増え、かえって複雑化する。R5.2 の一貫性は「どちらの再開でも同一 `.pasta` 行を離れるまで再停止しない」という**振る舞いの一致**で満たし、コード共有では満たさない。

### 3. 簡素化（Simplification）
- 新規コンポーネント・新規ファイルなし。`DebugSession` に **1 フィールド**（`pasta_break_anchor: RefCell<Option<PastaPos>>`）＋ **1 ヘルパー**（アンカー維持＆抑制適格判定）＋ `on_line_impl` の BP-first 分岐への数行。
- 共有ヘルパー抽出（Option C）は採らない（上記 Build vs Adopt）。

### 要決定の確定（設計合成で解決）
- **#1 フレーム/コルーチン修飾**: なし（pasta 行のみのアンカー）。ループ保証・再帰/コルーチン跨ぎはベストエフォート（要件ディスカッション決定済み）。
- **#2 アンカー解除条件**: 現行 `.lua` 行が**別の対応 `.pasta` 行**へマップされた時のみ解除（`Some(別行)`）。**未対応行（`None`）では解除しない**（同一 `.pasta` 行展開に未対応 `.lua` 行が挟まる場合に同一訪問内で誤再停止しないため）。
- **#3 アンカー確立タイミング**: 実際に BP 停止する直前に `anchor = 現在の .pasta 位置` を設定（停止時設定）。初回到達は `anchor` 未一致のため停止し、その時設定。Continue/Step 後の同一行は一致により抑制。
- **#4 ステップとの状態共有**: 分離（上記）。
- **既知リスク（自己修復）**: セッションは複数実行（SHIORI リクエスト）を跨いで存続。前回実行がアンカー設定行のまま終了し、次回実行の最初の BP が偶然同一 `.pasta` 位置だと1回だけ抑制され得る。極めて稀・次行で解除され自己修復。ベストエフォート方針の許容範囲。
