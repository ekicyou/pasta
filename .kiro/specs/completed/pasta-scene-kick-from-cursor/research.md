# Research & Gap Analysis: pasta-scene-kick-from-cursor

## Summary
- **Feature**: `pasta-scene-kick-from-cursor`
- **Discovery Scope**: Extension（既存 3 仕様 `pasta-scene-kick` / `pasta-source-map` / `pasta-vscode-lua-debug` の拡張・置換）
- **Key Findings**:
  - kick backend は完成・再利用可能。`pasta/playScene` の取次点（`wiring/inbound.rs::try_play_scene_kick`）は `KickRequest { scene: String }` を受け取る。位置→シーン確定さえできれば、確定したシーン名を既存 sink に渡すだけで再生まで繋がる。
  - **最大ギャップは「位置→シーン」逆引き索引の不在**。ソースマップは行マッピング（`.lua` 行 ↔ `.pasta` 行）のみを保持し、シーン境界・シーン識別子を一切持たない。`SceneRegistry` もシーンの `.pasta` 上の (file, 行範囲) を保持していない。両者を結ぶ索引を新設する必要がある。
  - VSCode 拡張・DAP custom request の追加は `pasta-debug-lua-view-toggle`（`pasta/sourcePresentation`）という明確な前例があり、4 層テンプレート（decode → wiring dispatch → VSCode command → pure helper）に沿って実装可能。リスクは低い。
  - 旧 `pasta.debug.playScene` UI 動線（`showInputBox` ＋ `commandPalette`/`debug/toolBar` 貢献）は所在が特定済みで、削除は局所的。

---

## Research Log

### Topic 1: 既存 kick backend の再利用可能性
- **Context**: 本機能は kick 実行本体を再設計しない（Out of scope）。確定済みシーンを既存取次点へ渡せるかを確認。
- **Sources Consulted**:
  - `crates/pasta_lua/src/debug/wiring/inbound.rs`（`try_play_scene_kick`、約 295-337 行）
  - `crates/pasta_lua/src/debug/kick.rs`（`KickRequest { scene: String }`、約 24-27 行）
  - `crates/pasta_lua/src/debug/dap/decode.rs`（`play_scene_response` / `play_scene_error`、約 311-339 行）
- **Findings**:
  - 取次点は `sink(KickRequest { scene: name })` を呼ぶだけ。入力はシーン名文字列で、ソース位置情報は持たない。
  - 成功／エラーの応答ビルダ（`play_scene_response` / `play_scene_error`）が既にある。新リクエスト用に同型のビルダを並置すればよい。
- **Implications**: 位置→シーン確定後は、確定したシーン名で既存 `KickSink` を呼べば Requirement 1.5 / 2.3 / 8.x を満たす。kick セマンティクスの継承は実装上「同じ sink を使う」ことで自然に達成される。

### Topic 2: ソースマップの現状と逆引き索引の不在
- **Context**: Requirement 2/3（位置→シーン解決とシーン同一性索引）の実現可能性。
- **Sources Consulted**:
  - `crates/pasta_lua/src/debug/source_map/mod.rs`（`ChunkSourceMap.forward: BTreeMap<u32, PastaPos>`、`SourceMap.reverse`、`resolve_lua_to_pasta` / `resolve_pasta_to_lua` / `nearest_pasta_line_with_mapping`、約 71/307/404-439 行）
  - `crates/pasta_lua/src/code_gen/source_map.rs`（`SourceMapSink` trait: `record_line` / `record`、`PastaPos`、約 40-68 行）
- **Findings**:
  - 保持しているのは `.lua` 行 → `PastaPos { file, line }` の前方マップと、その逆（`.pasta` (file,line) → `.lua` 行群）の `reverse` 索引のみ。
  - **シーン境界・シーン識別子は記録されていない**。`SourceMapSink` は行マッピングだけを記録し、トランスパイル時にシーン情報が sink へ流れていない。
- **Implications**:
  - 新たに「シーン同一性索引」（`(pasta_file, 行範囲) → scene 識別子`）を sink 経由で記録する拡張が必須（Requirement 3.1/3.2）。
  - 既存の行マッピング API は後方非破壊で維持する（Requirement 3.4）。

### Topic 3: SceneRegistry のシーン識別子生成と位置情報の不在
- **Context**: Requirement 3.3（索引のシーン識別子を `SceneRegistry` 規則と一致させる、二重実装回避）。
- **Sources Consulted**:
  - `crates/pasta_core/src/registry/scene_registry.rs`（`SceneRegistry` / `SceneEntry { id, name, fn_path, fn_name, parent, attributes }`、`register_global` / `register_local` / `sanitize_name`、約 40-170 行）
  - `crates/pasta_core/src/registry/scene_types.rs`
- **Findings**:
  - シーン識別子（`fn_name` 等）は `sanitize_name(name)` ＋連番で生成。global/local の階層情報（`parent`）も保持。
  - **`SceneEntry` は `.pasta` の (file, 行範囲) を保持していない**。位置情報はコード生成時に AST から得られるが、レジストリにも sink にも渡っていない。
- **Implications**:
  - シーン位置（span）を、コード生成時に `SceneRegistry` 由来の識別子と紐付けて source-map sink へ受け渡す経路が必要（Requirement 3.1/3.3）。`SceneEntry` 拡張 or 別索引のいずれを採るかは設計判断（後述 Option）。

### Topic 4: DAP custom request 追加の前例
- **Context**: Requirement 4（位置ベーストランスポート）の実装テンプレート。
- **Sources Consulted**:
  - `crates/pasta_lua/src/debug/dap/decode.rs`（`decode_request` の `pasta/sourcePresentation` arm、約 208-234 行；`pasta/playScene` arm、約 236 行）
  - `crates/pasta_lua/src/debug/wiring/inbound.rs`（`try_source_presentation_toggle` ディスパッチ、約 95-99 行）
  - `editors/vscode/src/extension.ts`（`pasta.debug.toggleSourcePresentation` / `pasta.debug.playScene` のコマンド登録、約 128-186 / 245-276 行）
  - `editors/vscode/src/sourcePresentationToggle.ts` / `editors/vscode/src/playSceneRequest.ts`（pure helper）
- **Findings**:
  - カスタムリクエストは command 文字列で分岐し、`Decoded` に引数を載せる → wiring で応答・イベント処理という 4 層構成。
  - VSCode 側は `customRequest(command, payload)` ＋ `onDidReceiveDebugSessionCustomEvent` ＋ pure helper（vscode 非依存）の定型。
- **Implications**: `pasta/playSceneAt`{uri, line} を同テンプレートで追加できる。新規性が低くリスクは小さい。

### Topic 5: 旧 playScene UI 動線の所在
- **Context**: Requirement 5（旧動線の廃止）の影響範囲。
- **Sources Consulted**:
  - `editors/vscode/package.json`（`contributes.commands` の `pasta.debug.playScene`、`contributes.menus` の `commandPalette` / `debug/toolBar`、`when: debugType == 'pasta'`、約 36-68 行）
  - `editors/vscode/src/extension.ts`（`registerCommand('pasta.debug.playScene', ...)` ＋ `showInputBox`、約 245-276 行）
  - `editors/vscode/src/playSceneRequest.ts`（`requestCommand = 'pasta/playScene'`、`setPayload` / `validateSceneName`）
- **Findings**: 旧 UI 動線は package.json とコマンド登録の 2 箇所に局在。`editor/context` メニュー貢献は現状存在せず、新規追加となる。カーソル位置（`editor.selection`）アクセスも未使用で新規。
- **Implications**: 削除は局所的。新コマンド `pasta.runSceneAtCursor` を `editor/context` の `group: navigation@1` に追加し、カーソル行取得 → `customRequest('pasta/playSceneAt', {uri, line})` の薄い glue を実装する。

---

## Requirement-to-Asset Map

| 要件 | 必要な技術要素 | 既存資産 | ギャップ判定 |
|------|----------------|----------|--------------|
| R1 右クリック動線 | `editor/context` メニュー貢献 + 新コマンド + カーソル位置取得 | 旧 `playScene` コマンド／`toggleSourcePresentation` 前例 | **Missing**（editor/context 貢献・cursor アクセスは新規） |
| R2 位置→シーン解決 | (uri,line)→scene 確定ロジック（最内 local 優先） | ソースマップ・`SceneRegistry`（ただし両者に境界情報なし） | **Missing**（解決ロジック・索引参照が新規） |
| R3 シーン同一性索引 | (file,行範囲)→scene 識別子 索引 + transpile 時受け渡し | `SourceMapSink`（行マッピングのみ） | **Missing**（シーン情報の sink 経路・索引が新規） |
| R4 位置ベーストランスポート | `pasta/playSceneAt`{uri,line} | `pasta/sourcePresentation` 前例・`play_scene_*` 応答ビルダ | **Missing**（新リクエストだが前例で低リスク） |
| R5 旧動線廃止 | 旧コマンド／メニュー／helper 削除 | `pasta.debug.playScene`（所在特定済み） | **Constraint**（局所削除・後方互換方針は要判断） |
| R6 表示条件・未接続時挙動 | `when` 句・セッション判定・警告 | `isPastaSession` ヘルパ・`debugType=='pasta'` パターン | **Constraint**（既存パターン流用） |
| R7 staleness 整合 | ロード済み基準解決・未解決時の安全側 | ソースマップ（ロード時版を保持） | **Unknown**（自動検知/リロード要否は OPEN QUESTION 4） |
| R8 既存制約継承 | 同一 `KickSink` 経由 | `KickRequest`/`try_play_scene_kick` | **再利用**（セマンティクス自然継承） |

---

## Implementation Approach Options

### Option A: シーン位置を `SceneRegistry`（`SceneEntry`）へ拡張記録し、source-map sink へ受け渡す
- **概要**: `SceneEntry` に `.pasta` の (file, 行範囲) を追加。コード生成時にシーン span を sink へ流し、ソースマップ側にシーン同一性索引を構築。
- **Trade-offs**:
  - ✅ シーン識別子の SSOT が `SceneRegistry` に集約され、Requirement 3.3（二重実装回避）と整合しやすい。
  - ✅ 既存 sink トレイトに `record_scene(span, scene_id)` 的なメソッドを追加する自然な拡張。
  - ❌ `pasta_core` の公開型変更が下流（テスト・スナップショット）へ波及する可能性。
  - ❌ 行マッピングとシーン索引を同じソースマップ内に共存させる設計が必要。

### Option B: シーン同一性索引を独立データ構造として新設（レジストリ非改変）
- **概要**: `SceneEntry` は変えず、コード生成時に別途 `(file, 行範囲) → scene 識別子` の独立索引を構築し、デバッグ用ソースマップと並置。
- **Trade-offs**:
  - ✅ `pasta_core` 公開型を変更せず影響範囲を局所化。
  - ✅ デバッグ専用機能（既定無効・ゼロコスト）の方針と整合しやすい。
  - ❌ シーン識別子規則の整合を実装側で担保する必要（`SceneRegistry` の生成結果を参照する形にすれば二重実装は回避可能）。
  - ❌ 索引の生成タイミング・所有者（loader/runtime のどこが保持するか）の設計が必要。

### Option C: ハイブリッド（span は code_gen で得て、識別子は `SceneRegistry` の生成結果を参照）
- **概要**: シーン span はコード生成フェーズで AST から取得。シーン識別子は `SceneRegistry` の登録結果（`fn_name` 等）を参照して紐付け、ソースマップ側の新索引へ記録。レジストリ自体は最小変更。
- **Trade-offs**:
  - ✅ 識別子は `SceneRegistry` 由来で一致（Requirement 3.3）、span は code_gen で確実に取得。
  - ✅ レジストリの破壊的変更を最小化しつつ SSOT を維持。
  - ❌ code_gen フェーズでシーン登録結果と span を突き合わせる結線が必要（実装の慎重さを要する）。

**設計フェーズへの推奨**: Option C を軸に検討（識別子の SSOT 維持と影響局所化の両立）。Option A は SceneEntry 拡張の波及を許容できる場合の代替。

---

## Implementation Complexity & Risk

| 領域 | Effort | Risk | 根拠 |
|------|--------|------|------|
| シーン同一性索引（R3：transpile 経路・sink 拡張・索引構築） | M (3-7日) | Medium | 新規のシーン span 受け渡し経路。pasta_core/code_gen にまたがり、span と識別子の突合に正確性が要る。 |
| 位置→シーン解決ロジック（R2：最内 local 優先・未検出処理） | M (3-7日) | Medium | 行範囲包含・local 優先の判定と境界ケース（シーン外・staleness）の安全側処理。 |
| 位置ベーストランスポート（R4：`pasta/playSceneAt`） | S (1-3日) | Low | `pasta/sourcePresentation` の前例に沿った定型追加。 |
| VSCode `editor/context` 動線（R1/R6） | S (1-3日) | Low | カーソル位置取得＋薄い glue。前例（toggle コマンド）あり。`editor/context` 貢献のみ新規。 |
| 旧動線廃止（R5） | S (1-3日) | Low | 局所削除。後方互換方針の確定が前提。 |

**全体**: Effort = M〜L（索引と解決ロジックが主因）、Risk = Medium（中核は索引の正確性と境界ケース処理）。

---

## Research Needed（設計フェーズへ持ち越し）
1. シーン同一性索引の所有者・生存期間（loader/runtime のどこがロード済み索引を保持するか。debug 既定無効・ゼロコスト方針との両立）。
2. シーン span の取得元（AST のシーンノードに行範囲があるか、code_gen でどう露出するか）の確定。
3. local シーンの「行範囲」定義（宣言行のみか、内包する子要素まで含むか）と、最内判定アルゴリズムの厳密化。
4. staleness 検知の要否・方式（OPEN QUESTION 4 と連動。ロード済み版マーカーと編集中バッファの照合可否）。
5. 旧内部トランスポート `pasta/playScene`{scene 名} の保持／撤去（OPEN QUESTION 3 と連動。位置ベース実装が名前ベース取次点を内部再利用する設計なら名前ベース経路は内部的に残す選択肢あり）。

---

## OPEN QUESTIONS（要件ディスカッションで解決）
1. **メニュー表示条件・セッション未接続時の挙動**: `editor/context` 項目を `resourceLangId == pasta` で常時表示するか、要デバッグセッション（`debugType == 'pasta'`）で gate するか。常時表示なら未接続時は警告のみか、自動アタッチ案内まで行うか（requirements.md R6.2 暫定＝常時表示＋警告）。
2. **シーン外位置の扱い**: どのシーンにも属さない位置（シーン間・空行）でのキック要求を、シーン未検出エラーとするか、最近接シーンへフォールバックするか（requirements.md R2.5 暫定＝未検出エラー）。
3. **旧内部トランスポートの後方互換**: 旧 UI 動線（コマンド・showInputBox）廃止は確定。旧内部リクエスト `pasta/playScene`{scene 名} を完全撤去するか、位置ベース実装が再利用するため内部的に残すか（requirements.md R5.4 暫定＝設計フェーズ判断）。
4. **staleness／リロード整合**: 編集後・未リロードで行ずれが生じ得る場合、ベストエフォート解決のみとするか、staleness を検知してリロード要求／警告を出すか（requirements.md R7.2 暫定＝ベストエフォート、検知方式は設計判断）。

---

## 設計フェーズ追記（discovery 検証・synthesis・設計判断）

### Discovery 検証（コードベース実地確認）
research 初版の所在情報を実コードで再確認し、以下を確定した（設計の根拠）。
- **時系列制約（最重要）**: シーン登録（`register_global`/`register_local`）は **Lua ランタイム（SHIORI finalize）** で発生するのに対し、`.pasta` span は **code_gen 時**に `SourceMapSink` 経由で記録される。両者は別フェーズ。「span（code_gen 由来）と識別子（`SceneRegistry` 由来）の突合をどこで行うか」が設計の中核論点（Decision 1）。
- `SourceMapSink` は `record_line(lua_line, pasta_line)` ＋ sugar `record(lua_line, span)` のみ。`record_scene` を **デフォルト no-op で追加**すれば既存実装は後方非破壊（R3.4 を満たす）。
- `scope_gen.rs` は global/local シーン生成箇所で既に `record_span(scene.span)` を呼んでおり、`record_scene` の自然な挿入点が存在。
- kick 取次は `try_play_scene_kick` → `sink(KickRequest { scene })`（fire-and-forget）。`KickSink = Arc<dyn Fn(KickRequest) + Send + Sync>`。確定識別子を渡すだけで R8（セマンティクス継承）が自然達成。
- DAP 4 層（decode→wiring dispatch→VSCode command→pure helper）は `pasta/sourcePresentation` が完全な前例。`editor/context` 貢献のみ本機能が初出（既存メニューは `commandPalette`/`debug/toolBar` のみ）。
- `SourceMap` は loader `build_source_map` が `.pasta` ごとに parse→sink→transpile→`finish`→`insert_chunk` で集約し `Arc<SourceMap>` を構築。`enable()` が `Arc<SourceMap>`＋`KickSink` をブリッジへ供給。索引はこの `SourceMap` に同梱するのが自然。

### Synthesis（3 レンズ適用結果）
- **Generalization**: 位置→シーン解決は「行→所属シーン」の一般問題。`SceneIdentityIndex` を「行範囲→識別子（入れ子レベル付き）」の汎用索引として設計し、最内優先・後方フォールバックを resolver 側のポリシーとして分離。将来の `pasta-authoring-window`（別境界）も同索引を再利用可能なインタフェースとする（実装は現要件範囲に限定）。
- **Build vs. Adopt**: シーン識別子規則は `SceneRegistry::sanitize_name` ＋連番が既存 SSOT。**Adopt**（参照）し二重実装しない（R3.3）。トランスポートは独自プロトコルを作らず既存 DAP カスタムリクエストチャネルを **Adopt**。索引データ構造は標準 `BTreeMap`（行範囲昇順）で十分（O(log n)・R8.3）。
- **Simplification**: 新規 transport は `playScene` の引数拡張でなく専用 `pasta/playSceneAt` を新設（旧口は撤去するため拡張より置換が単純・R5.4）。索引は別クレート化せず既存 `SourceMap` 内に同梱（debug 既定無効・ゼロコスト方針と整合）。resolver は単一実装で抽象レイヤを足さない。

### Design Decisions

#### Decision 1: span と識別子の突合を code_gen で行う（research Option C 採用）
- **Context**: span は code_gen 時、識別子は `SceneRegistry` 由来で別フェーズ。R3.3（二重実装回避）と影響局所化の両立が必要。
- **Alternatives**: A) `SceneEntry` に span を追加（`pasta_core` 公開型変更が下流へ波及） / B) 独立索引をレジストリ非改変で新設（識別子整合を実装側で担保） / C) span は code_gen で取得し識別子は `SceneRegistry` 登録結果を参照して `record_scene` へ流す。
- **Selected**: **Option C**。code_gen がシーン生成時に `SceneRegistry` の確定識別子を引き、`record_scene(scene_id, span)` を呼ぶ。
- **Rationale**: 識別子 SSOT を `SceneRegistry` に維持しつつ `pasta_core` 破壊的変更を最小化。`scope_gen` の既存 `record_span` 近傍に自然挿入できる。
- **Trade-offs**: ✅ 影響局所・SSOT 維持 / ❌ code_gen でシーン登録結果と span を突き合わせる結線に正確性が要る。
- **Follow-up**: OQ-1。code_gen フェーズで識別子が確実に得られるか（登録順と生成順の一致）を実装時に検証。

#### Decision 2: 専用トランスポート `pasta/playSceneAt` を新設し旧外部口を撤去
- **Context**: R5.4（外部口を位置ベース一本に統一）。
- **Selected**: `pasta/playSceneAt`{uri, line} を新設、旧 `pasta/playScene`{scene 名} arm と `Decoded.kick_scene` を削除。内部取次（`KickSink`）は名前ベースのまま保持（外部非露出・R5.5）。
- **Rationale**: 「唯一の動線」方針。引数拡張より置換の方がワイヤ・コードとも単純。
- **Trade-offs**: ✅ 動線一本化 / ❌ 旧口に依存する外部利用があれば破壊的（ただし作者向け動線は廃止確定）。

#### Decision 4: staleness は best-effort ＋ 未解決＝未検出（暫定）
- **Context**: R7.2/7.5。ロード時索引で解決するため未リロードで誤解決し得る。
- **Selected（暫定）**: best-effort 解決とし、解決不能は未検出扱い（R7.6）。staleness の積極検知（mtime/ハッシュ照合）は OQ-2 として discussion へ持ち越し。kick 経路での自動リロードは行わない（R7.3、手動「SHIORIリロード」へ一本化）。
- **Trade-offs**: ✅ 実装単純・安全側 / ❌ 行ずれ時に誤シーンを再生し得る（検知導入で軽減可能）。

#### Decision 5: 再アタッチは固定待機→1回（暫定）
- **Context**: R9.4/9.5。リロードでデタッチ→自動再アタッチ。
- **Selected（暫定）**: 「リロード指示→数秒固定待機→`vscode.debug.startDebugging`（type pasta）で1回再アタッチ」。再アタッチ後のキック自動再実行なし。「SHIORIリロード」は接続中のみ表示。具体値・リトライ/タイムアウトは OQ-3 として持ち越し。

### Risks & Mitigations（設計フェーズ追加）
- **uri 正規化** — VSCode uri → 索引キー（chunk 名/ファイルパス）の正規化（Windows パス・URI エンコード・8.3 短縮名）。`std::path::absolute` ベースで正規化し、`fs::canonicalize` は避ける（既知 CI バグ回避）。
- **end_line 確定** — シーン範囲終端（「次の同レベル以上のシーン宣言の直前」/ファイル末尾）の算出と入れ子レベルの受け渡し。`finish` 時に確定する設計とし特性化テストで担保。
- **後方非破壊の担保** — `record_scene` デフォルト no-op ＋ 既存 `resolve_*` の挙動不変を回帰テストで固定（R3.4）。
