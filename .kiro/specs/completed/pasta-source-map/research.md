# Gap Analysis: pasta-source-map

調査日: 2026-06-08 / 対象: requirements.md R1〜R8 / ブランチ: feat/pasta-source-map

先行仕様 `pasta-vscode-lua-debug`（完了）が「実現可能性確定・設計シーム実装・代表経路1本のE2E実証」までを出荷済み。本分析は、その遺産（producer シーム／consumer シーム／薄い実証スライス）に対し本番化の差分を洗い出す。

---

## 1. 現状調査（Current State）

### 1.1 producer 側（code_gen）
- 配置: `crates/pasta_lua/src/code_gen/`（`mod.rs` 197行 / `element_gen.rs` 520行+ / `scope_gen.rs` 323行 / `source_map.rs` 55行）
- **出力行カウンタ + シーム**: `LuaCodeGenerator` に `out_line: u32`（mod.rs:44）、`source_map: Option<&mut dyn SourceMapSink>`（mod.rs:49）、`record_span(span)`（mod.rs:109-115）を実装済み。`writeln`/`write_blank_line`/`write_raw`/`write_line_terminator` の4経路で `out_line` を厳密管理（mod.rs:128-164）。`set_source_map()`（mod.rs:81）で sink 装着。
- **record 呼び出しの現状**: 全11個の `generate_*` のうち **`generate_action` のみ**が `record_span` を呼ぶ（element_gen.rs:345）。残り10経路は span を破棄。
- **SourceMapSink trait**: `record(&mut self, lua_line: u32, span: Span)`（source_map.rs:51-54）、`PastaPos { file: String, line: u32 }`（source_map.rs:33-38）。
- **normalize_output**: `normalize.rs:30-86`。CRLF→LF、`end` 直前の空行削除、末尾空白削除、単一LF付与。`out_line` は normalize **前**のバッファ行を数える。talk 行は `end` より前に出るため代表経路では前後不変（debug/source_map.rs:21-30 に根拠コメント）。

### 1.2 DSL span 可用性（pasta_dsl）
- `Span`（pasta_dsl/.../ast/span.rs:31-44）: `start_line/col`・`end_line/col`（1-based）＋ `start_byte/end_byte`（0-based）。`is_valid()` = `end_byte > 0`（:109）。
- **span を保有**: `Action`（全variant）・`VarSet`・`CallScene`・`CodeBlock`・`ActionLine`・`ContinueAction`・`KeyWords`。code_gen に完全に渡っている。
- **span を保有しない（⚠ギャップ）**: `ActorScope`・`GlobalSceneScope`・`LocalSceneScope`（スコープ定義3型）。

### 1.3 consumer 側（debug / DAP）
- 配置: `crates/pasta_lua/src/debug/`（mod.rs 636行 / dap.rs 1071行 / breakpoints.rs 266行 / session.rs / hook.rs / inspect.rs / transport.rs / wiring.rs / types.rs / source_map.rs[gate]）。
- **DAP source seam**: `SourceResolver = Box<dyn Fn(&str,u32)->ResolvedSource + Send>`（dap.rs:113）、`default_source_resolver()` は生成 `.lua` をそのまま返す（dap.rs:119-124）、`set_source_resolver()` で差替（dap.rs:249-251）。`encode_frames` がフレーム毎に `resolver(&f.source, f.line)` を呼ぶ（dap.rs:562-580、:570）。
- **setBreakpoints**: DAP→`SessionCommand::SetBreakpoints{source, lines}` に decode（dap.rs:319-328、parse は :512-530）。SourceRef は現状 `.lua` パス前提。
- **breakpoints.rs**: `BreakpointSet = Arc<Mutex<HashSet<Breakpoint>>>`、`should_pause(source,line)->bool`、`set_breakpoints()` は実行中書込可・per-source。`SourceRef{path:String}` は単なる文字列で解釈は呼び元次第 → `.pasta` パス受理は構造上可能。
- **本番 enable 経路は既定 resolver のまま**: `set_source_resolver` はスライスE2Eテストでのみ呼ばれ、`mod.rs` の `enable()` は既定（`.lua`）を使用。

### 1.4 薄い実証スライス（debug/source_map.rs・feature `pasta-source-map-slice`・cfg(test)）
- `LineMap`: `BTreeMap<u32, PastaPos>`（:44-93）。`pasta_for_lua`（:76, R4.4）、`lua_lines_for_pasta`（:86, 逆引き R4.5・BTreeMap反復で決定的）。
- `SliceSink`（:104-158）: `record` 受領時に `span.start_byte` → `pasta_line_at(byte)`（:138-144、`\n` 数えで1-based行・UTF-8境界チェック）で `.pasta` 行へ変換。
- `resolve_lua_to_pasta`（:160-168）公開関数。
- E2E: `slice_e2e_pasta_breakpoint_hits_and_reports_pasta_line`（:436-517、task 8.3）。実トランスパイル→LineMap→BP逆引き→実 DebugSession 停止→DapAdapter が `.pasta` 行2を stackTrace で報告。
- **ゼロコスト regression**: `tests/transpiler/source_map_seam_test.rs`（feature OFF でバイト一致を検証、`CapturingSink` mock）。

### 1.5 VSCode 拡張（editors/vscode）
- `package.json`: debugger type `pasta`（:189-231、attach・host/port 既定 127.0.0.1:9276）。`breakpoints` に `pasta`/`lua` 両言語登録済み（:181-187）。
- `debugAdapterFactory.ts`: `DebugAdapterServer(port, host)` を返す attach-only（:28-34）。`extension.ts:40-42` で factory 登録。
- **提示モード切替（.pasta/.lua）の設定項目は未存在**。

---

## 2. Requirement → Asset マップ（ギャップ種別: Missing / Unknown / Constraint）

| Req | 必要能力 | 既存資産 | ギャップ |
|-----|---------|---------|---------|
| R1 全経路網羅 | 全 `generate_*` で `record_span` | `generate_action` のみ実装 | **Missing**: 残10経路へ record 挿入。scope 3型は span 不在 → **Missing(DSL)**: AST に span 追加が必要 |
| R2 行ズレ補正 | normalize 後行への一般補正 | 代表経路のみ不変保証 | **Missing**: `end` より前に出る要素（var_set/choice/code_block 等）の normalize 行ズレ補正アルゴリズム |
| R3 保持＋任意出力 | メモリ既定＋ディスクサイドカー | LineMap（メモリ・test gate） | **Missing**: 本番 LineMap 構築の本番経路結線、サイドカー出力器（フォーマット未定 → **Unknown**） |
| R4 `.pasta` BP | `.pasta`→`.lua` 逆引き＋登録 glue | `lua_lines_for_pasta` 存在 | **Missing**: setBreakpoints の `.pasta` SourceRef 受理→逆引き→`.lua` 行登録の glue。対応行なし時の最近接調整 |
| R5 停止/スタック `.pasta`提示 | `.pasta` SourceResolver + 結線 | `set_source_resolver` seam 存在 | **Missing**: `.pasta` resolver 実装と `enable()` 本番経路での装着 |
| R6 提示モード切替 | `.pasta`/`.lua` 切替 | seam で resolver 差替可能 | **Missing**: DebugConfig への mode 追加・VSCode 設定項目・launch.json 受け渡し |
| R7 後方互換/ゼロコスト | feature OFF バイト不変・Lua デバッグ継続・暫定ハーネス除去 | regression テスト・slice は cfg(test)+gate | **Constraint**: 本番化に伴い slice を本番経路へ昇格 or 除去。ゼロコスト regression を本番マップ生成に拡張 |
| R8 エッジケース | 多対多・安定順序・集約 | BTreeMap で決定的順序 | **Unknown→確定要**: 多対多時の代表 `.pasta` 行選定規則、複数命令端の扱い |

---

## 3. 実装アプローチ案

### Option A: 既存シームの拡張（Extend）— **推奨**
先行 spec が遺した seam（`SourceMapSink`/`SourceResolver`/`LineMap`/`lua_lines_for_pasta`）をそのまま本番化する。
- **producer**: 残10 `generate_*` に `record_span` を挿入。scope 3型は pasta_dsl AST に `span` を追加して伝播。
- **map**: slice の `LineMap`/`SliceSink` を test gate から本番モジュールへ昇格、本番 `record_span`（byte→line 変換は parser の `Span.start_line` を直接使えば `pasta_line_at` 再計算を回避できる可能性 → 設計検討）。
- **normalize**: `out_line`→最終 `.lua` 行への補正テーブルを normalize に持たせ、確定後に LineMap を rebase。
- **consumer**: `.pasta` SourceResolver を実装し `enable()` で mode に応じ装着。setBreakpoints に `.pasta`→`.lua` 逆引き glue を追加。
- **VSCode**: package.json に提示モード設定＋launch.json 受け渡し。
- ✅ seam 設計が既に検証済みで手戻り小・依存追加ゼロ・ゼロコスト維持が容易 / ❌ code_gen 全経路と pasta_dsl AST への横断改修（広い波及）

### Option B: 独立ソースマップ層を新設（New）
code_gen から独立した「span 収集→正規化→LineMap」専用モジュールを新規作成し、code_gen は span イベントを emit するだけにする。
- ✅ 責務分離・テスト容易・normalize 補正を一箇所に集約 / ❌ 既存 seam（既に record ベース）と二重化リスク・先行 spec の検証資産を捨てる・XL 化

### Option C: ハイブリッド（段階導入）
Phase 1: producer 全網羅＋normalize 補正＋本番 LineMap（メモリのみ）で `.pasta` BP/停止提示を成立（slice 昇格）。Phase 2: ディスクサイドカー出力・提示モード切替・エッジケース確定・暫定ハーネス除去。
- ✅ 早期に中核体験（`.pasta` BP）を出荷・検証マトリクス（保持×提示）を後段に隔離 / ❌ 2段階の計画管理

---

## 4. Effort / Risk

| 区分 | 評価 | 根拠 |
|------|------|------|
| 全体 | **L（1〜2週）** | seam は既存だが code_gen 全経路＋pasta_dsl AST＋DAP glue＋VSCode＋テストへ横断波及 |
| R1 producer 網羅 | M / Risk: Medium | 10経路への record 挿入は機械的だが、scope 3型の span 追加は pasta_dsl パーサ改修で波及 |
| R2 normalize 補正 | M / Risk: **High** | 出力行ズレの一般化は誤ると全 `.pasta` 行が1行ずれる中核リスク。`end` 前要素の検証が要 |
| R3 保持/出力 | S〜M / Risk: Medium | メモリ結線は軽量。サイドカー出力フォーマットが未定（Unknown） |
| R4/R5 BP/提示 | M / Risk: Medium | seam 差込で素直。対応行なし時の調整・多対多の代表選定が UX 判断 |
| R6 提示モード | S / Risk: Low | resolver 差替＋設定追加 |
| R7 互換/ハーネス除去 | S / Risk: Medium | regression 拡張＋slice 昇格時に cfg(test) 依存の解体に注意 |
| R8 エッジケース | S / Risk: Medium | 規則確定が主。BTreeMap で順序は既に決定的 |

---

## 5. 設計フェーズへの申し送り（Recommendations）

### 推奨アプローチ
**Option A（既存シーム拡張）＋ Option C の段階化**。先行 spec が seam を検証済みのため新設は不要。中核体験（`.pasta` BP/停止提示）を先に成立させ、サイドカー出力・提示モード・エッジケース確定を後段へ隔離する。

### 設計で決める主要判断
1. **byte→line 変換の出所**: slice は `span.start_byte` から再計算（`pasta_line_at`）。本番は `Span.start_line`（パーサ既算・1-based）を直接使えば再計算とUTF-8走査を回避できる可能性。代表行の定義（要素の開始行 vs 行頭）を確定。
2. **normalize 行ズレ補正の方式**: (a) normalize 時に旧行→新行の rebase テーブルを作り LineMap を写像、(b) normalize を行削除しない方針へ変更（出力差リスク）。(a) 推奨だが要設計。
3. **scope 3型の span 追加範囲**: パーサ改修の最小範囲（定義行のみ記録で足りるか）。
4. **`.pasta` SourceRef の識別**: setBreakpoints/stackTrace で `.pasta` か `.lua` かをパス拡張子で判定するか、明示フラグか。
5. **提示モードの受け渡し**: DebugConfig フィールド／VSCode 設定キー名／launch.json スキーマ。
6. **エッジケース規則（R8）**: 多対多時の代表 `.pasta` 行（最小行 or 最初に record された行）、集約行の確定提示。
7. **対応行なし時のBP調整（R4-3）**: 「後続最近接へ調整＋調整位置提示」を採用予定（要件で既定済・設計で DAP の breakpoint verified/line 返却に落とす）。

### Research Needed（設計フェーズで詰める）
- ディスクサイドカーのフォーマット（独自JSON / Source Map v3 互換 / 行ペア列）と出力先・命名。
- LuaJIT の `currentline` が複数バイトコードに跨る端ケースで報告する行の実挙動（実機確認）。
- slice の cfg(test)+feature gate を本番モジュールへ昇格する際のコンパイル構成（feature 名の去就・default 化）。

---

## 要件ディスカッション反映（2026-06-08）

- **議題1（scope定義行）= 「含める」決定**: scope定義3型（`ActorScope`/`GlobalSceneScope`/`LocalSceneScope`）への span 追加を**本仕様のスコープに正式に昇格**（旧ギャップ分析では「設計境界・span不在」と記載していたが消費側限定から拡大）。`.pasta` の `＊シーン名`/`＠アクター` 等の定義ヘッダ行をブレークポイント対象にする。
  - 影響: pasta_dsl パーサ改修（3型への span 追加＋伝播）が必須化。R1 producer の Effort/Risk が上振れ（Medium→Medium寄りだが pasta_dsl 横断のため波及増）。設計フェーズで「定義ヘッダ行の代表 span（定義行＝開始行）」と「Lua 側の停止アンカー（`function ...` 定義行 vs 本体先頭）」の対応規則を確定する必要あり。
- **議題2（未対応行BP）= 「後続最近接へ自動調整」決定**: R4-3 の既定どおり。DAP `setBreakpoints` レスポンスで `verified` ＋ 調整後 `line` を返す（設計で確定）。要件変更なし。
- **議題3（ステップ粒度）= 「`.pasta` 粒度ステップも含める」決定**: step over/into/out を `.pasta` 行単位で進める要件（R9）を新設し**本仕様のスコープへ追加**。
  - 影響: 全体 Effort が **L → XL 寄り**へ上振れ。先行 spec の `session.rs` StepController（over/into/out は coroutine identity ＋ stack depth で判定・`.lua` 粒度）に、「現在 `.pasta` 行に対応する `.lua` 行群を消化するまで内部 step を継続」するループ層を追加実装する必要あり。設計フェーズで、StepController を改修するか上位にラッパを設けるか、`.pasta` 対応なし行のスキップ条件、提示モード（`.pasta`/`.lua`）でのステップ粒度切替を確定する。consumer 側（debug 基盤）への波及が当初想定（seam 差込のみ）より増える。

## 6. 境界の再確認（Out of scope）
- Lua デバッグ基盤本体（transport/hook/inspect/session/DAP プロトコル本体）は先行 spec 所有・改修は seam 装着の最小限。
- `.pasta` の span 生成そのものはパーサ責務（本仕様は scope 3型への span 追加を「依存先への必要改修」として持つが、span の意味論設計は最小限）。
- `.pasta` 編集時ラウンドトリップ・`.lua` 以外の生成ターゲットは対象外。

---

# 設計フェーズ Discovery（2026-06-08 / kiro-spec-design）

> 上記 §1〜§6 は `/kiro-validate-gap` 由来のギャップ分析。以下は設計フェーズで 5 本の並列 subagent によりコードを再精査した結果と設計判断。**§1.2 / §2 の「scope定義3型は span を保有しない」という記述は、本セクションの実コード検証により誤りと確定した（下記 D-1）。**

## D-0. Discovery 手法
Light discovery（Extension）。並列精査領域: producer 側（code_gen）/ consumer 側（debug・DAP）/ pasta_dsl span + normalize / feature gate + VSCode / ローダのチャンク命名・ライフサイクル。外部依存の新規追加なしのため WebSearch は不要と判断。

## D-1.【重要訂正】scope定義3型は span を既に保有している
- **検証（自己確認・file:line）**:
  - `ActorScope.span: Span`（`ast/mod.rs:140`）、`GlobalSceneScope.span: Span`（`ast/scene.rs:58`）、`LocalSceneScope.span: Span`（`ast/scene.rs:117`）が**定義済み**。
  - パーサが**有効な span を実際に格納**: `parse_global_scene_scope` が `Span::from(&pair.as_span())`（`parse_scene.rs:11`）を `GlobalSceneScope.span` へ（`parse_scene.rs:68`）。ローカルシーンは `parse_scene.rs:186-188, 199-201`。ActorScope は `parser/mod.rs:236, 283`。
  - 値は scope 全体（ヘッダ＋body）を覆う pest Pair span。`start_line` は定義ヘッダ行（`＊シーン名`/`＠アクター名`）を指す。
- **帰結**: R1.5（定義ヘッダ行を BP 対象に）は**既存 `span.start_line` で達成可能**。`record(header_lua_line, scope.span)` を各 scope ヘッダの `.lua` 出力行で呼べばよい。**pasta_dsl パーサ改修は baseline では不要**。
- **要件・ギャップ分析との差分**: requirements の Boundary Context「scope定義3型には span が未提供のため本仕様で追加する／pasta_dsl パーサ改修」および要件ディスカッション議題1の前提（span 不在）は**不正確**。本設計はパーサ改修を baseline から外す（スコープ縮小）。ヘッダ行*単独*の精密 span が将来必要になった場合のみ、ヘッダ Pair span をパーサに追加可能（Pest Pair から `Span::from`、波及小）。R1.5 の WHAT（ヘッダ行を停止対象に）は保持される。

## D-2.【設計核心】本番チャンク名 = `@<キャッシュ .lua の絶対パス>`
- 本番は `scene_dic.lua` の `require("pasta.scene.<module>")` → Lua 標準 `package.path` 検索（`loadfile`）でロード（`loader/mod.rs:185,460`、`loader/cache.rs:203-345`、`loader/context.rs:98-113`）。LuaJIT がチャンク名に**絶対 `.lua` ファイルパス**を付与する（Rust 側は未設定・未正規化）。
- ラインフックの `source` は `lua_Debug.source` を素通し（`@` 除去・正規化なし。`session.rs:180-190`、`inspect.rs:174-183`）。BP 突合は `(source, line)` 完全一致（`breakpoints.rs:73-79`）。
- スライステストは `set_name("@slice.lua")`（`source_map.rs`）でチャンク名固定 → **本番と形が根本的に異なる**。
- トランスパイル単位は **1 `.pasta` = 1 Lua チャンク**（結合なし。`loader/mod.rs:384-461`）。
- **帰結**: 本番 `SourceMap` は「チャンク名（絶対 `.lua` パス・`@` 付き）」をキーにする。ローダは `cache::source_to_cache_path` で同一の絶対パスを producer 側で算出できる。**実機（特に Windows）のチャンク名文字列形は早期タスクで `debug.getinfo` 実測確認する（Validation Hook・最重要リスク）。**

## D-3. その他の設計核心
- **本番ローダから sink 到達不能**: `record_span` 配線は `generate_action` の 1 箇所のみ。`LuaTranspiler::transpile()` / `TranspileContext` は sink を通さず、装着は `LuaCodeGenerator` 直接構築のみ（`transpiler.rs:57`、`code_gen/mod.rs:81`）。→ transpile API への sink 受け渡し追加が必須。
- **`Span.start_line` 直使用で byte 走査廃止可**: スライスの `SliceSink` は `start_byte`＋ソース走査だが、`span.start_line`（1 始まり）を直接使えば `.pasta` ソーステキスト不要（簡素化）。移行時に既存スライステストの byte 結果と一致確認。
- **normalize 行ズレ要因は 1 つ**: 「`end` 直前空行削除」（＋末尾空白削除）のみ・単一パス・行削除のみ（増加/マージ/挿入なし・`normalize.rs:30-86`）。削除対象は空行＝`.pasta` 由来なし。補正 = 「最終行(old) = old − (old より前で削除された行数)」の単調写像。現 `normalize_output` は削除写像を破棄しているため、削除行リスト（または old→new 写像）を返す拡張が必要。
- **session は LineMap 非依存**: `.pasta` 粒度ステップ（R9）には session への `Arc<SourceMap>`＋`SourceMode` 注入が必要（`session.rs` の `RunMode::Stepping`/`step_should_stop` 拡張）。
- **提示モードは器のみ**: `DebugConfig.source_map_slice: bool` は常時 `false` のデッド予約。`SourceMode` enum 由来へ置換。VSCode は DAP 報告値表示のみで変換は Rust `SourceResolver` 側（`editors/vscode` は `sourcePresentation` 素通しのみ）。
- **DebugConfig 供給**: pasta.toml `[debug]`＋env、合成は `DebugConfig::resolve`（env>file>default・`debug/mod.rs:122-159`）。提示モードはここ＋DAP attach 引数で供給。

## D-4. Synthesis（3 レンズ）
- **Generalization**: 全要件は「チャンクごとの双方向行マップ」1 抽象に集約できる。R4 は `.pasta`→`.lua`（1→N）、R5 は `.lua`→`.pasta`（N→1）、R8 はその多重度、R9 は `.lua`→`.pasta` 上の「同一 `.pasta` 行消化」。スライス `LineMap` を種に、マルチチャンク本番 `SourceMap` へ一般化（実装は現要件範囲・IF を一般化）。
- **Build vs Adopt**: Source Map v3（VLQ）は列指向・minified JS 向けで過剰＋外部 crate 依存（最小依存方針に反する）→ **不採択**。既存 `SourceMapSink`/`LineMap`/`SourceResolver` シームを **Build（拡張）**。サイドカーは既存依存 `serde_json` で簡素な行ペア JSON。チャンク命名は Lua 自身の `loadfile` チャンク名を **Adopt**（フック source と必ず一致）。
- **Simplification**: (a) scope 3型のパーサ改修を撤去（D-1）、(b) デッド `source_map_slice` bool を `SourceMode` へ統合、(c) `pasta-source-map-slice` feature gate を撤去し本番常時コンパイル化（R7.3）、(d) byte 走査を `start_line` 直使用へ。

## D-5. Design Decisions（設計フェーズ確定）
1. **SourceMap 2 段構造**: `SourceMap { chunks: HashMap<ChunkName, ChunkSourceMap>, reverse: PastaIndex }`。`ChunkSourceMap` = `BTreeMap<u32 lua_line, PastaPos>`。`reverse` = `.pasta` ファイル/行 → `[(ChunkName, lua_line)]`。`ChunkName` はフック報告値（`@<絶対 .lua パス>`）に一致。`Arc<SourceMap>` で不変共有。
2. **source map 生成は debug 有効時のみ**: 無効時 sink=None でバイト不変（R7.1）。ローダが debug 有効時のみ `MapBuilderSink` を transpile へ装着。
3. **提示モードは DAP attach 引数経由（既定 `.pasta`）**: `SourceMode { Pasta, Lua }`。launch.json `sourcePresentation` → attach 引数 → サーバが resolver/ステップ粒度を選択。pasta.toml/env もフォールバック供給。
4. **`.pasta` 粒度ステップは session に SourceMap 注入**: `RunMode::Stepping` に起点 `.pasta` 位置を追加。`step_should_stop` 後、現 `.lua` 行の `.pasta` 位置が起点と同一 or 未対応なら継続。
5. **BP 未対応行は後続最近接へ調整**（R4.3）: `reverse` index で「要求 `.pasta` 行以上で最初に対応を持つ行」を解決し、DAP レスポンスで `verified`＋調整後 `line` を返す。
6. **エッジケース規則**（R8）: `ChunkSourceMap` は 1 `.lua` 行 → 1 `PastaPos`（決定論的 codegen の last-write-wins ＝安定）。逆引きは昇順 `Vec`。`Arc` 不変共有で提示順安定。

## D-6. Risks（設計フェーズ更新）
- 本番チャンク名の実機文字列形（D-2）— 早期 Validation Hook で実測。最重要。
- 保持方式 × 提示モードの検証面増（brief 制約）— tasks フェーズで検証マトリクス明示。サイドカーは最小実装、メモリ既定を主検証。
- R7.1 回帰（バイト不変）— debug 無効時 sink=None のスナップショット回帰を必須化。
- 全 `generate_*` 配線漏れ — `record_span` を `writeln` 直前に集約し構文種別網羅テストで担保。
