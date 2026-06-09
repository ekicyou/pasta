# Gap Analysis: pasta-debug-lua-view-toggle

> 既存コードベースと要件 (requirements.md R1–R8) のギャップ分析。設計判断ではなく、設計フェーズへ運ぶ情報・選択肢・研究課題を提示する。

## 分析サマリ

- **バックエンド基盤は既に大半が実装済み**。提示モード切替に必要な可変共有状態 `SharedSourceMode`（`Arc<AtomicU8>`・`set()`/`get()`）、フリップ時に再実行されるレゾルバ差し替え `attach_pasta_resolver()`、毎行で実効モードを読む `effective_mode()`、提示モード連動のステップ粒度（`pasta_step_should_stop`）が揃う。残る不足は **「カスタム DAP リクエスト経路」** と **「フリップ後の再描画イベント発火」** の 2 点に集約される。
- **VSCode 拡張側が最大のギャップ**。`contributes.commands` / `menus`（デバッグツールバー）が皆無、`vscode.commands.registerCommand` も `activeDebugSession.customRequest` 呼び出しも未実装。ただしデバッグ型 `"pasta"` 登録・`sourcePresentation` 正規化プロバイダ・attach 接続経路は既に存在し、追加は確立済みパターンに乗る。
- **マニュアル**は `book/src/debug/source-level.md` に「提示モードの切替」節が既存だが、attach 固定の初期解決のみ記述。実行時トグル手順・初期値上書き関係の追記が必要。
- **総合難易度 M（3–7 日）・リスク Low〜Medium**。バックエンドはシーム拡張、フロントエンドは新規 UI だが既知 API、再描画イベントの DAP セマンティクスのみ要検証。
- **主要な設計判断**: (a) カスタムリクエスト名・引数形（`pasta/setSourcePresentation` の引数 `mode`）、(b) フリップ後の再描画手段（`stopped` 再送 vs `invalidated` イベント vs クライアント主導 re-fetch）、(c) トグルの「実行中受理（次停止に反映）」と「停止中即時再描画」の両立方法。

## Requirement → Asset マップ（ギャップタグ）

| Req | 要件要旨 | 既存アセット | ギャップ |
| --- | -------- | ------------ | -------- |
| **R1** 実行時切替（制御経路） | `SharedSourceMode.set()` (`mod.rs:148-171`)、`attach` 分岐の `sourcePresentation` 解釈 (`dap.rs` attach arm)、`SourceMode::parse()` の不正値→Pasta フォールバック | **Missing**: `decode_request()` の `match command` (`dap.rs:349-499`) にカスタムリクエスト分岐なし（未知は `_ => Decoded::default()` で黙殺）。**Missing**: R1.3「受理応答」を返すカスタムレスポンス。R1.4 不正値は `parse()` 既存挙動で吸収可。R1.5 実行中受理は VM 非同期更新（共有セル）で自然に満たせるが要検証 |
| **R2** VSCode トグル UI | デバッグ型 `"pasta"` 登録、`registerDebugAdapterDescriptorFactory`/`ConfigurationProvider` (`extension.ts:38-50`)、`SOURCE_PRESENTATION_KEY` 定数 (`debugAttachTarget.ts:38`) | **Missing**: `contributes.commands`・`contributes.menus.debug/toolBar`（`package.json` に両キー不在）。**Missing**: `registerCommand` ハンドラ、`activeDebugSession.customRequest` 呼び出し。R2.4 非アクティブ無効化（`when` 句）、R2.5 現在モード表示の手段（toolbar アイコン状態 / 通知 / statusbar）未設計 |
| **R3** 提示への即時反映 | レゾルバseam `set_source_resolver()`、`encode_frames()` が毎フレーム resolver 参照 (`dap.rs:644-662`)、`attach_pasta_resolver()` 再実行 (`wiring.rs:155-196`) | **Missing**: R3.3「停止中の現在提示の再描画」を起こす **イベント発火が皆無**。socket bridge は attach フリップ後に共有セル＋レゾルバを更新するが `SessionEvent::Stopped`/`invalidated` 等を送出しない。クライアントが再 `stackTrace`/`source` を発火する契機が必要（**Research Needed**） |
| **R4** 初期値と実行時トグルの整合 | 優先順位 `attach > env > file > 既定` (`mod.rs:275-279`)、`from_env()` 解決 (`mod.rs:304-327`)、attach override seam | **Constraint**: 「実行時トグルで初期値を上書き」は共有セル `set()` で実現可。attach の override 経路と同一セルを共有するため、トグル＝attach 後段の追加書き込みとして自然。R4 はほぼ既存基盤の再利用で充足 |
| **R5** 提示モードとステップ粒度の整合 | `effective_mode()` (`session.rs:279-307`)、`pasta_step_should_stop()` (`session.rs:487-510`)、`on_line_impl` が毎行 `effective_mode()` 参照 (`session.rs:688-777`) | **充足見込み**: 共有セルを毎行読むため、停止中フリップ後の次ステップは新粒度で動く（R5.3）。コルーチン跨ぎ（R5.4）も `current_thread_and_depth` ベースで継続。**検証のみ必要**（新規実装ほぼ不要） |
| **R6** 既存挙動への無回帰 | OFF ゼロコストゲート `enable()` の `if !cfg.enabled return Ok(None)` (`mod.rs`)、BP は `pasta_active()` ゲートで提示モード非依存に維持 | **Constraint**: カスタムリクエスト処理は有効時のみ走り、OFF 経路は不変（バイト不変・ゼロコスト維持）。R6.3 BP 維持は既存の BP 翻訳ロジックが提示モード切替後も有効。**回帰テストで担保** |
| **R7** 実 DAP-over-TCP E2E | 既存 transport (`transport.rs`)、attach `sourcePresentation` の DAP テスト (`dap.rs:1079-1133`)、E2E ヘルパ (`tests/common/e2e_helpers.rs`) | **Missing**: 「`.pasta` BP のまま `.lua` 表示へ往復」のカスタムリクエスト E2E テスト。既存 attach テストの隣に並列スイートを新設 |
| **R8** マニュアル更新 | `book/src/debug/source-level.md` の「提示モードの切替」節（attach 固定の初期解決を記述）、`SUMMARY.md` の章エントリ | **Missing**: 実行時トグル手順（VSCode コマンド/ボタン）、初期値→トグル上書き関係、停止中即時再描画・実行中次停止反映のタイミング、粒度連動。drift-check 非対象章のため `manual-sources.toml` 変更は不要 |

## 主要な技術的不足（集約）

1. **カスタム DAP リクエストハンドラ不在**（R1/R3 中核）
   - `dap.rs:349-499` の `decode_request()` match は DAP 最小サブセットのみ。`pasta/setSourcePresentation`（仮）は `_ => Decoded::default()` で黙殺される。
   - 追加に必要: (a) match へ分岐追加、(b) 引数 `mode`（`"pasta"`/`"lua"`）を `SourceMode::parse()` で解釈、(c) 受理レスポンス（R1.3）、(d) `Decoded` に mode を載せて socket bridge へ伝播、(e) `wiring.rs:283-296` 付近で `source_map.source_mode.set(mode)` + `attach_pasta_resolver()` を呼ぶ（attach 経路と同じ後段処理を再利用）。

2. **フリップ後の再描画イベント非発火**（R3.3 中核・最大の不確実性）
   - 現状、共有セル＋レゾルバ更新後にクライアントへ「再取得せよ」と伝える信号がない。停止中はクライアントが自発的に `stackTrace` を再発行しないため、提示が古いまま残る懸念。
   - 候補手段: DAP `invalidated` イベント（`areas: ["stacks"]`）／`stopped` 再送（同一 reason で再フォーカス）／VSCode 拡張側で `customRequest` 応答後にクライアント側 refresh をトリガ。**いずれが VSCode で確実に再描画を起こすかは Research Needed**。

3. **VSCode UI コントリビューション一式**（R2 中核）
   - `commands`・`menus.debug/toolBar`・`registerCommand`・`activeDebugSession.customRequest` をゼロから追加。`when` 句で `debugType == 'pasta'` かつセッション状態で活性制御（R2.4）。現在モード表示（R2.5）は toolbar アイコン二態 or 通知 or status bar の選択が必要。

## 実装アプローチ選択肢

### Option A: 既存シームを拡張（バックエンド）＋ 新規 UI 追加（フロント）— 推奨
- **バックエンド**: `decode_request()` にカスタムリクエスト分岐を追加し、attach が既に使う `SharedSourceMode.set()` + `attach_pasta_resolver()` の後段処理を**そのまま再利用**。`Decoded` 構造体の既存 `attach_source_mode: Option<SourceMode>` フィールドを汎用化するか、新フィールドを足す。
- **フロント**: `package.json` に `commands`/`menus`、`extension.ts` に `registerCommand` + `customRequest` を新規追加。
- **トレードオフ**: ✅ 既存の可変共有状態・レゾルバ差し替え・粒度連動を最大活用（新規ロジック最小）。✅ attach override と完全に同一経路で整合（R4 自然充足）。❌ 再描画イベントの設計は別途必要（どの選択肢でも避けられない）。
- **適合度**: 高。brief の Approach（`SharedSourceMode` を実行時更新する DAP カスタムリクエスト）と一致。

### Option B: 新規「SourceMode 制御」サブモジュールを新設
- カスタムリクエスト解釈・モード状態遷移・再描画トリガを `debug/source_mode_control.rs`（仮）に集約。
- **トレードオフ**: ✅ 責務分離・テスト容易。❌ 既存 `wiring.rs`/`dap.rs`/`session.rs` に散在する共有セル参照と二重管理になり、かえって整合が複雑化。既存基盤が既に薄く正しく結線されているため過剰。
- **適合度**: 低〜中。既存設計が既にシーム化されており、新モジュールの便益が薄い。

### Option C: ハイブリッド（バックエンドは A、再描画のみ段階導入）
- フェーズ1: カスタムリクエスト＋共有セル更新（実行中→次停止反映、R1.5）まで実装。停止中即時再描画（R3.3）は最小手段（`stopped` 再送 or `invalidated`）で後追い。
- フェーズ2: VSCode 側のモード表示（R2.5）と E2E（R7）を仕上げる。
- **トレードオフ**: ✅ 不確実性（再描画イベント）を切り出して段階検証。✅ R1/R5 を先に確定できる。❌ R3.3 を後回しにすると受け入れ基準が割れるため、フェーズ間の DoD 管理が必要。
- **適合度**: 中〜高。再描画イベントの不確実性が高いと判明した場合の保険。

## 工数・リスク

| 領域 | 工数 | リスク | 根拠 |
| ---- | ---- | ------ | ---- |
| バックエンド カスタムリクエスト（R1/R4） | S | Low | `SharedSourceMode.set()` + attach 後段処理が既存。match 分岐追加と Decoded 伝播のみ |
| 再描画イベント（R3.3） | S〜M | **Medium** | DAP `invalidated`/`stopped` の VSCode 再描画挙動が未検証。手段選定に実験が必要 |
| ステップ粒度整合（R5） | S | Low | `effective_mode()` 毎行読みで既に動作見込み。検証主体 |
| VSCode UI（R2） | M | Low〜Medium | 新規 commands/menus/registerCommand/customRequest。既知 API だが toolbar 活性制御・モード表示の UX 設計あり |
| 無回帰（R6） | S | Low | OFF ゼロコスト維持・BP 翻訳は既存。回帰テストで担保 |
| 実 DAP E2E（R7） | M | Medium | TCP 越しの往復＋BP 維持検証。既存 e2e_helpers を流用しつつ新シナリオ構築 |
| マニュアル（R8） | S | Low | 既存節の更新。drift-check 非対象 |
| **総合** | **M（3–7 日）** | **Low〜Medium** | 不確実性は再描画イベント手段と VSCode UX に局在 |

## Research Needed（設計フェーズへ持ち越し）

1. **再描画トリガ手段の確定**: 停止中にモードをフリップした際、VSCode に現在フレームを新座標で再描画させる最確手段は何か。候補: DAP `invalidated` イベント（`areas: ["stacks"]` / `["sources"]`）、`stopped` イベント再送、または拡張側 `customRequest` 応答後のクライアント refresh。実 VSCode で再 `stackTrace`/`source` が発火するかを検証する。
   - **要件ディスカッション確定（2026-06-09）**: R3.3「停止中の即時再描画」は**ハード受け入れ基準**（利用者の追加操作なしで即座に再描画）として確定。フォールバック（次ステップ/フレーム再選択での反映）は許容しない。したがって本項目は単なる調査ではなく、**設計着手前に小さな spike（実証）で確実な再描画手段を 1 つ確定させる**ことが必須前提となる。spike で確実な手段が見つからない場合は設計でその不確実性を最優先に扱う。
2. **カスタムリクエストのプロトコル形**: リクエスト名（`pasta/setSourcePresentation` 等）・引数キー（`mode` か `sourcePresentation` か—既存 attach と命名統一すべきか）・レスポンス形（現在モードのエコーバックで R1.3 と R2.5 を同時充足できるか）。
3. **R2.5 現在モード表示の UX**: デバッグツールバーアイコンの二態表示か、status bar item か、通知トーストか。VSCode のツールバーボタンで状態表現が可能かを確認。
4. **実行中（非停止）受理の反映タイミング**（R1.5）: 共有セルは即時更新されるが、VM スレッドが次に停止するまで提示は変わらない。この「次停止で反映」を E2E でどう検証するか。
5. **`Decoded.attach_source_mode` の汎用化可否**: 既存 attach 専用フィールドをトグルでも流用するか、別フィールド/別経路にするか（命名と後段処理の重複回避）。

## 設計フェーズへの推奨

- **推奨アプローチ: Option A**（既存シーム拡張 + 新規 UI）。不確実性が高い場合のみ Option C のフェーズ分割を保険とする。
- **最優先で解消すべき設計判断**: 再描画イベント手段（Research Needed #1）。これが R3.3 受け入れ基準の成否を左右する。
- **持ち越し研究**: 上記 Research Needed #1〜#5。とりわけ #1 は設計着手前に小さな実証（spike）で確度を上げる価値がある。
- **既存資産の最大活用**: `SharedSourceMode` / `attach_pasta_resolver` / `effective_mode` / `pasta_step_should_stop` を再利用し、新規ロジックを最小化する方針を設計に明記する。

---

# 設計フェーズ discovery / synthesis（2026-06-09）

> `/kiro-spec-design` の Light Discovery（Extension）＋ synthesis 結果。上記ギャップ分析を前提に、R3.3 即時再描画手段を確定するための精密調査を実施。

## Discovery Summary
- **Discovery Scope**: Extension（既存 DAP バックエンド＋VSCode 拡張への機能追加）
- **Key Findings**:
  1. **source 提示は実ファイルパス**: `default_source_resolver`/`pasta_source_resolver` はいずれも `{"path": <file>}` を返し、`encode_frames` (`dap.rs:644-662`) がそのまま frame.source に載せる。仮想 `sourceReference` も `source` リクエストハンドラも存在しない（`decode_request` は `source` 未対応）。→ `.lua` 提示は attach 時 `sourcePresentation: lua`（既存機能）と同一の実ファイル addressability に依存し、**新たな source 配信機構は不要**。これは上流 pasta-vscode-lua-debug / pasta-source-map が既に解決済みの領域。
  2. **`invalidated` 経路は皆無、`stopped` は全配線済み**: `supportsInvalidatedEvent` は initialize で未通知（`dap.rs:350-367` は `supportsConfigurationDoneRequest: true` のみ）。クライアント capability も未読。一方 `SessionEvent::Stopped` → `run_event_encoder` (`wiring.rs:489-507`) → `drain_outbound` (`wiring.rs:469-480`) → wire の経路は完全配線。`stopped` 再送は DAP クライアント普遍の確実な再フェッチ契機。
  3. **ブリッジは VM 停止状態を不可視**: socket-bridge は `RunMode` を観測できない（`wiring.rs:213-259`）。「停止中のみ再描画」を成立させるには、停止状態を所有する VM スレッド側 `DebugSession`（stop_loop）に再描画判断を委ねる必要がある。

## Architecture Pattern Evaluation（再描画手段）

| Option | 説明 | 強み | リスク/限界 | 採否 |
|--------|------|------|-------------|------|
| A. `stopped` 再送（停止中のみ） | トグル適用後、停止中なら現在の `SessionEvent::Stopped` を再送し VSCode に stackTrace 再フェッチを促す | 既存経路を完全再利用・全 DAP クライアントで確実・capability 交渉不要 | 「再停止」の意味論（同一位置で再 stop 通知）。実質無害だが厳密には pause の再通知 | **採用** |
| B. `invalidated` イベント（areas: stacks） | DAP `invalidated` を送出し stacks のみ無効化 | 意味論的に正確（再 stop を装わない） | initialize でクライアント capability `supportsInvalidatedEvent` の読取＋ゲートが必要・新イベント型新設・対応クライアント限定 | 不採用（交渉コスト・普遍性で A に劣る） |
| C. ブリッジ側で停止状態共有フラグを新設し同期 emit | `SharedStopState` を追加しブリッジが直接再描画 emit | 同期・即時 | 新規共有状態の追加・停止スナップショット二重管理 | 不採用（C は状態増。停止状態所有はセッションが自然） |

## Design Decisions

### Decision: 再描画は「停止中限定の `stopped` 再送」をセッション経路で行う
- **Context**: R3.3 は停止中トグルでの即時再描画をハード必須（フォールバック不可、ディスカッション #2 確定）。ブリッジは停止状態を知らない。
- **Alternatives Considered**: A（stopped 再送）/ B（invalidated）/ C（ブリッジ側共有フラグ）。
- **Selected Approach**: VM スレッド `DebugSession` に `SessionCommand::RefreshPresentation` を新設。ブリッジは custom request 受信時に (1) `SharedSourceMode.set()` で提示モードを更新、(2) `attach_pasta_resolver()` でレゾルバ差し替え（attach 経路の後段処理を再利用）、(3) `RefreshPresentation` をセッションへ転送。セッションは stop_loop で受信した際、**停止中なら直近の停止スナップショット `(StopReason, thread_id)` を用いて `SessionEvent::Stopped` を再送**（resume はしない）。非停止時は無視（次の自然停止が新モードで提示される＝R1.5）。
- **Rationale**: 既存の stopped 全配線＋attach レゾルバ差し替えを最大再利用。停止状態の所有者（セッション）が再描画を判断するため、ブリッジへの新規共有状態が不要。capability 交渉も不要で全クライアントで確実。
- **Trade-offs**: ✅ 新規面最小・確実・OFF 経路不変。❌ 「再 stop」意味論（無害）。spike で VSCode 再フェッチ＋新 source 再描画を着手前に実証する（R3.3 ハード基準ゆえ必須前提）。
- **Follow-up**: spike で stopped 再送 → stackTrace 再フェッチ → 新ファイル/行で再描画を確認。確認後に本実装着手。

### Decision: 提示モードは「設定 custom request ＋ push カスタムイベント」で制御する
- **Context**: R1.3 は受理応答、R2.5/R2.6 は現在モードの常時・正確な表示を要求。拡張は launch.json 未指定時のバックエンド解決値（env/toml/既定）を知らないため、初期表示が不正確になりうる。
- **Alternatives Considered**:
  1. set/get 統合の単一 request（pull）— セッション開始時に省略形 query を送って初期表示を初期化。`onDidStartDebugSession` 時点でアダプタが custom request を処理できる readiness/timing リスクが残る。
  2. set request ＋ push カスタムイベント — バックエンドが初期/変更モードを能動通知。
  3. 既定値仮定 — 初期は launch.json 値 or `.pasta` と仮定（env/toml 上書き時に不正確）。
- **Selected Approach**（設計ディスカッション #1, 2026-06-09 でユーザ確定）: 設定は DAP custom request `pasta/sourcePresentation`（引数 `{ mode }`・受理応答 `{mode}`）。表示は同名の **DAP カスタムイベント**（body `{mode}`）で push。バックエンドは (a) attach 完了時に解決済み初期モード、(b) トグルでの変更後に新モードをイベント送出。拡張は `onDidReceiveDebugSessionCustomEvent` を表示の単一真実源とし、query を発行しない。
- **Rationale**: 初期表示の正確性（R2.5）を query の timing/readiness から独立させ、最も堅牢。他要因での変更も拡張へ確実に届く。受理応答（R1.3）は request 側、表示更新（R2.6）はイベント側に責務分離。
- **Trade-offs**: ✅ timing 非依存・堅牢・表示の単一真実源。❌ カスタムイベント契約が増える（request＋event の 2 契約）。
- **Follow-up**: 不正 `mode` 値（R1.4）は無変更＋現在モードをエコー、セッション継続。イベント送出は attach・トグル両経路で共通化。

### Decision: VSCode 側は「ツールバーボタン＝操作」「ステータスバー＝常時表示」に分離
- **Context**: R2.2 はデバッグツールバーのトグルボタン、R2.5 は現在モードの常時判別表示。ツールバーボタンはテキスト状態の持続表示に不向き。
- **Selected Approach**: `contributes.menus.debug/toolBar` にトグルボタン（操作）、`vscode.window.createStatusBarItem` で `$(eye) 提示: .pasta/.lua` を pasta デバッグセッション中のみ常時表示（状態）。コマンドパレットコマンドも提供（R2.1）。さらにステータスバー item の `command` をトグルコマンドへ束ね、クリックでもトグル可能にする（第3導線・設計ディスカッション #3 確定）。
- **Rationale**: 操作と状態表示の責務分離。ステータスバーは持続表示の VSCode 標準手段。
- **Trade-offs**: ✅ R2.2/R2.5 を各々最適手段で充足。❌ UI 要素 2 箇所。
- **Follow-up**: `when: debugType == 'pasta'` で非 pasta セッション時は非表示/無効（R2.4）。純ロジック（次モード算出・ペイロード生成）は vscode 非依存モジュールへ分離しユニットテスト（既存 `debugAttachTarget.ts` の踏襲）。

## Synthesis 結果
- **Generalization**: attach と実行時トグルが「モード適用 → レゾルバ差し替え → RefreshPresentation → `pasta/sourcePresentation` イベント送出」の同一後段経路を共有。設定は request、表示は push イベントに責務分離（上記 Decision）。
- **Build vs Adopt**: 再描画は DAP 標準（`stopped` イベント）を adopt。新規プロトコル/ライブラリ導入なし。
- **Simplification**: 新規共有状態を増やさず（停止スナップショットはセッションローカル）、`SharedSourceMode`/`attach_pasta_resolver`/`SessionEvent::Stopped`/`pasta_step_should_stop` を再利用。再描画経路は `RefreshPresentation` 1 本（停止中ゲートはセッションが判断）。

## Risks & Mitigations（設計フェーズ追補）
- **R-1 stopped 再送の再描画確実性（最重要）** — spike で着手前に実証。万一不確実なら invalidated（Option B）へ切替可能なよう、再描画 emit をセッション内の単一箇所に局所化。
- **R-2 `.lua` 提示の実ファイル可達性** — attach 時 lua モード（既存）と同一前提。新機構を導入しない（上流所有）。E2E（R7）で往復実証。
- **R-3 RefreshPresentation の非停止時残留** — 非停止で送られた command が次 stop_loop で冗長再送する可能性。無害だが、停止スナップショット無し時は即無視する実装で抑制。
- **R-4 OFF 経路不変（R6.2）** — custom request 処理は debug 有効時のみ走るブリッジ/アダプタ内に限定。OFF はバイト不変・ゼロコスト維持。

---

## Spike 結果（Task 1.1）

> 「停止中 `stopped` 再送による即時再描画」の実証（Requirement 3.3 のハード受け入れ基準ゲート）。本 spike は **非挙動（NON-BEHAVIORAL）** であり、ランタイムトグル機能の実装は含まない。確定した再描画手段とエビデンス、go/フォールバック判断を記録する。

### 確定: 再描画手段は「停止中 `stopped` 再送」（GO）

- **確定手段**: 停止中にモードをフリップした際、現在の停止スナップショット（`reason`/`thread_id`）で **DAP `stopped` イベントを再送** する。これにより DAP クライアント（VSCode）が `threads` → `stackTrace` → `scopes` → `variables` を **再フェッチ** し、差し替え済みレゾルバが提示する新モード座標で現在フレームが再描画される。
- **判断**: **GO**。`invalidated`（フォールバック）は不要。設計の Decision「再描画は『停止中限定の `stopped` 再送』をセッション経路で行う」をエビデンスで裏付け、本実装着手の前提条件を満たした。

### エビデンス #1: DAP 仕様 — `stopped` がクライアントの再フェッチ起点

DAP 公式 overview（VSCode チーム著・DAP の規範的解説）より逐語:

> "Whenever the program stops … the debug adapter sends a **stopped** event with the appropriate reason and thread id."
>
> "Upon receipt, the development tool first requests the `threads` … and then the _stacktrace_ (a list of `stack frames`) for the thread mentioned in the stopped event. If the user then drills into the stack frame, the development tool first requests the `scopes` for a stack frame, and then the `variables` for a scope."

すなわち `stopped` の受信 → 開発ツール（VSCode）が `threads`/`stackTrace`（→必要に応じ `scopes`/`variables`）を **必ず再要求** する。これが「停止中に `stopped` を再送すれば現在フレームが新座標で再描画される」根拠。

加えて、オブジェクト参照の寿命が再描画契機を補強する:

> "Once execution resumes, object references become invalid and DAP clients must not use them. When execution is paused again, object references no longer refer to the same objects."

→ クライアントは新たな停止通知のたびに古い参照を捨てて再フェッチする設計であり、`stopped` 再送が「同一位置の再 stop」であっても確実に再フェッチを誘発する。

- 出典: <https://microsoft.github.io/debug-adapter-protocol/overview>

### エビデンス #2: VSCode 固有挙動 — `stopped` 受信で stackTrace を要求

VSCode 拡張 API デバッグドキュメントより:

> "When the program stops … the debug adapter has to send a stopped event … Upon receipt VS Code will request the stacktrace (a list of stack frames) for the given thread."

→ VSCode が `stopped` 受信時に `stackTrace` を要求することの一次情報。

- 出典: <https://vscode-docs.readthedocs.io/en/stable/extensionAPI/api-debugging/>

### エビデンス #3: フォールバック手段 `invalidated` の制約（不採用根拠）

DAP 仕様（specification）より逐語:

> `invalidated` イベント: "This event signals that some state in the debug adapter has changed and requires that the client needs to re-render the data snapshot previously requested." / body の `areas`: "Set of logical areas that got invalidated"（例: stacks 等）。
>
> capability ゲート: "This event should only be sent if the corresponding capability `supportsInvalidatedEvent` is true."
>
> initialize の **クライアント** capability `supportsInvalidatedEvent`: "Client supports the `invalidated` event."

→ `invalidated`（`areas: ["stacks"]`）は意味論的には正確だが、(a) initialize **リクエスト** で来るクライアント capability `supportsInvalidatedEvent` を読み取り、(b) true のときのみ送出するゲートが必須。現状バックエンドは initialize で `supportsConfigurationDoneRequest` のみ通知し、クライアント capability を読んでいない（`dap.rs` `decode_request` の `"initialize"` アーム）。普遍性（全クライアントで確実）と交渉コスト不要の点で `stopped` 再送が優位。よって **不採用（フォールバック保留）**。

- 出典: <https://microsoft.github.io/debug-adapter-protocol/specification>

### エビデンス #4: アダプタ層の最小実証（コード）

既存トランスポート/アダプタが「停止中の `stopped` 再送 → 直後の `stackTrace` を継続提供」を **単発ガードなし・resume なし** で行えることを、アダプタ層のユニットテストで実証した。

- テスト: `crates/pasta_lua/src/debug/dap.rs` の `debug::dap::tests::stopped_can_be_resent_midpause_and_stacktrace_still_served`
- 検証内容:
  1. 1 回目の `Stopped`（breakpoint）→ `stackTrace`（req_seq=50）受理・`Stack` 応答が correlate。
  2. **resume せず** 2 回目の `Stopped`（同一 reason/thread）を `encode_event` → 単発ガードなしで再度 `stopped` フレームを発行。`seq` は単調増加（リプレイではなく新規フレーム＝クライアントは新規停止として再フェッチ）。
  3. 再送後の `stackTrace`（req_seq=51）も継続提供され correlate。
- 実行結果: `cargo test -p pasta_lua --lib debug::dap::tests::stopped_can_be_resent_midpause_and_stacktrace_still_served` → **1 passed**。`debug::dap::tests` 全体 → **27 passed; 0 failed**（無回帰）。
- スコープ注記: 「停止中のみ再送する」VM スレッド側のゲート判断（`SessionCommand::RefreshPresentation` を `session.rs` の stop_loop で停止中に限り処理）は **本 spike では未実装**（本実装タスクの範囲）。本テストは「アダプタ/トランスポートが再送＋継続提供を許す」という transport-layer の前提のみを実証する。停止中ゲートの実装可否は、stop_loop が既に `reason`/`thread_id` を引数保持していること（`session.rs:540-661`）から構造的に裏付けられる。

### フォールバック発動の基準（トリガ）

以下のいずれかが本実装中の E2E（Task 7・実 DAP-over-TCP / 実 VSCode）で観測された場合に限り、`stopped` 再送を `invalidated`（`areas: ["stacks"]` ＋ `supportsInvalidatedEvent` ゲート）へ切替える:

1. 実 VSCode が停止中の `stopped` 再送を受けても `stackTrace` を **再フェッチしない**（現在フレーム提示が更新されない）。
2. 再送 `stopped` が VSCode 側で副作用（フォーカス飛び・スクロール位置リセット・誤った「新規停止」通知の重複）を生み、利用者体験を損なう。
3. 同一位置再 stop が VSCode の内部状態（call stack ビュー）を不整合にする。

切替コストは設計どおり局所的: 再描画 emit はセッション内の単一箇所（`RefreshPresentation` アーム）に局所化されるため、`invalidated` へ差し替える場合も影響範囲は (a) initialize でのクライアント capability 読取＋ `supportsInvalidatedEvent` 保持、(b) emit 1 箇所の置換、に限定される。現時点のエビデンス（#1–#4）からは **フォールバック発動は不要** と判断する。
