# ギャップ分析: pasta-runtime-internals-doc

- **対象**: `requirements.md`（Requirement 1〜10、2026-10-02 生成）
- **分析日**: 2026-10-02
- **分類**: Extension（既存 mdBook 基盤・生成機構・完了ゲート・定期レビューへの追加）＋大量の新規執筆
- **判断方針**: 情報提供が目的。最終決定は要件ディスカッション／設計フェーズで行う。

## 1. サマリー

- **基盤側の新規実装はほぼ不要**。mdBook のビルド・検索・ハイライト・Pages 公開・生成機構・鮮度チェック・リンク検証はすべて既存で、内部設計パートは `SUMMARY.md` へのパート追加で乗る。生成機構は「1 章 → 1 生成ファイル」の対応表に 1 行追加するだけで `internal-modules.md` を生成できる（manual-ssot-authority design.md L66 の想定どおり）。
- **本仕様の作業量の大半は執筆**である。6 題材は合計で Rust 約 2.5 万行（debug だけで約 7.6k 行）、Lua 約 3.9k 行に及び、各題材を確立した完了 spec は 8〜11 件ずつある。完了 spec の design.md は旧パスの参照が多く（後述 §3.3）、「コードを正」とする照合コストが高い。
- **主要な制約の衝突が 3 つある**。(1) スキル自己完結規則（`crates/`・`book/src` の語を禁止）と、内部設計章にソースの所在を書く要件。(2) 生成機構が求める章構造（`---` 2 本・本体の口調不在）と、内部設計パートの文体規約が未定であること。(3) `internal-modules.md` に利用者向け API 使用法が混在しており、brief の Out of Boundary と衝突し得ること。
- **鮮度維持の組み込み先は特定済み**。完了ゲートは `workflow.md` DoD（権威）と `kiro-complete` SKILL.md（発火・チェックリスト）の 3 箇所を編集する。定期総点検は `review-improvement-loop` の brief / requirements / design / tasks の固定文言（D7）を編集する。manual-ssot-authority のコミット da8e1d7c に同型の前例がある。
- **推奨は Option C（ハイブリッド）**。基盤は既存拡張（Option A）とし、執筆は題材ごとの独立章として新規作成する。工数 L、リスク Medium。

## 2. 現状調査

### 2.1 マニュアル基盤（`book/`）

| 資産 | 現状 | 本仕様との関係 |
|---|---|---|
| `book/src/SUMMARY.md` | 5 パート（入門 / 文法 / Lua / デバッグ / リファレンス）と「はじめに」。内部設計パートは無い | 末尾にパートを追加する（R1.1） |
| `book/book.toml` | `title = "pasta 利用者マニュアル"`、description も利用者向け | 書名・説明の扱いが未定（Q4） |
| `book/AUTHORING.md` | 全内容章に Claudia 令嬢ボイス（導入→普通文体の本体→締め、`---` で区切り）を規定 | 内部設計パートへの適用が未定（Q1） |
| `book/tools/gen-skill-refs.mjs` | `GENERATION_MAP` は 21 エントリの `{chapter, skill}` 凍結配列。1 章 → 1 ファイル。出力名は章のベース名（`index.md` は `<親>-index.md`）。本体は最初と最後の `---` の間で、口調マーカーがあれば `voice-in-body` エラー。リンクは同スキル生成章なら兄弟ファイル名へ、それ以外は `https://ekicyou.github.io/pasta/…html` へ書き換える | 内部設計の章を `internal-modules.md` という名前で作れば、コード変更なしで既存名の生成物になる（R6.2） |
| `book/tools/gen-skill-refs-test.mjs` L100 | `HANDWRITTEN` に `internal-modules.md` を列挙 | 生成化に伴い更新が必要 |
| `book/tools/link-check.mjs` | 対象は `book/src/**/*.md` と 2 スキル。`FORBIDDEN_SKILL_TOKENS = ['doc/spec','GRAMMAR.md','book/src','crates/']` はフェンス内・コメントを含めて全文に適用される。クレート README・ルート md は走査しない | 生成対象章に `crates/` を書けない（Q3）。README のリンクは検証されない |
| `book/tools/verify-content.mjs` L249 | ボイス検査（D-voice）は列挙ディレクトリ（`''`・grammar・lua・lua/modules・getting-started・reference）のみ。A-summary は生成対象章の目次到達を検査する | 新ディレクトリは自動では口調検査されない。生成対象に加えれば A-summary の対象になる |
| `book/tools/verify-static.mjs` | SUMMARY 由来の全章について HTML 生成・目次・前後ナビゲーションを検査 | 新パートも自動で検査対象になる（R1.4） |
| `.github/workflows/manual.yml` | `book/**`・2 スキル・自身の変更で起動。順序は `--check` → build → highlight → bigram → link-check → verify → Pages | `crates/**` の変更では起動しない（鮮度は完了ゲート頼み。§4 参照） |

### 2.2 吸収元の文書

| 文書 | 規模 | 現状と陳腐化 |
|---|---|---|
| `OPTIMIZATION.md` | 145 行。「最終更新 2026-01-25 / Phase 0 完了」 | 参照先の `crates/pasta_lua/src/code_generator.rs#L320-L460` は存在しない（現在は `code_gen/`。TCO は `element_gen.rs` の `generate_call_scene(…, is_tail_call)`）。§3「アクター最適化」は不正確で、`last_actor` は切替最小化ではなく継続行の話者引継ぎ（無ければ `invalid_continuation`）。§4.2「Unicode を含む場合」も不正確で、`string_literalizer.rs` は `\`・`"` の場合のみロングブラケットにする。§6 ビルドプロファイルは `Cargo.toml` L83-88 と一致。§5 は将来候補 |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` | 796 行・手書き（生成マーカー無し） | STORE / ACT（トーク・`get_property`・スポット・検索・`call`・`yield`・`choice`・PROXY）/ SCENE / WORD / GLOBAL / SAVE / `finalize_scene` / `pasta.buf`・`pasta.lua_version`。純内部事項と、ゴースト作者がスクリプトで使う API 使用法が混在している（Q2） |
| `crates/pasta_lua/README.md` | 521 行 | 内部: アーキテクチャ（L14）、ソースモジュール構成（L26-51）、SHIORI 統合 / `pasta.shiori.res` / sakura_builder（L353-482）、ファイル検出・モジュール名生成（L483-511）。利用者向け重複: `pasta.toml`（L119-186）、検索パス・UTF-8 契約・起動失敗（L190-243）、組み込みモジュール API（L244-295）。crates.io 向け: 概要・Rust API・関連・ライセンス |
| `crates/pasta_shiori/README.md` | 219 行 | 内部: アーキテクチャ（L12）、ディレクトリ構成（L27）、FFI 境界の安全性（L88-103）。半内部: プロトコルフロー（L45-87）。残りは crates.io 向け |
| `crates/pasta_core/README.md` | 132 行 | 内部: アーキテクチャ（L11）、ディレクトリ構成（L25-47） |
| `crates/pasta_dsl/README.md` | 115 行（英語） | 内部: Architecture（L75-112） |
| その他 README | `pasta_lsp` 128、`pasta_check` 108、`pasta_sample_ghost` 151 | 題材外のクレート。移設対象とするかは Q5 次第 |
| `book/src/reference/startup.md` | 185 行・生成対象 | 利用者向けの権威。内部章は実装詳細のみを書き、ここへリンクする（R4.2） |

`OPTIMIZATION.md` への参照元: `SOUL.md` L33・L478、`TEST_COVERAGE.md` L203・L295（R5.5 で修正対象）。ルートに `docs/`・`ARCHITECTURE*.md` の類は無い。

### 2.3 鮮度維持の組み込み先

- **`.kiro/steering/workflow.md`**: DoD（L27-36）は Gate 1〜5 と条件付きの Gate 6「Manual Sync Gate」（L38-54）。L40 で「ルール本体はこのゲートに置く（権威）」とし、L54 で既存 Gate の意味・順序を変えない「条件付き追加」の前例を示す。スキル更新検討表は L66-87。
- **`.claude/skills/kiro-complete/SKILL.md`**: Step 1 で workflow.md のゲートを列挙・発火する（L111、L127-134 は Manual Sync Gate の発火条件・コマンド・中断・スキップ）。完了チェックリストは L288-290。workflow.md を権威とする宣言（L33-37）があるが、実際には L127-134 と L290 で発火条件を再記述している。
- **`.kiro/specs/review-improvement-loop/`**: phase `tasks-generated`、反復 spec（completed へ移さない）。初回完走済みで全タスク `[x]`。次元⑦の定義は brief.md L54、requirements.md L55・R2.9（L63）、design.md L293（横断 D7 集約）・L318-326（G5 群）、tasks.md L41（Task 2 固定文）・L797-798（Task 5.1 文書整合チェックリスト）にある。tasks.md L770-779 のセル 3.64 は生成物（再実行時に再生成）。ループを実装するスキルは無く、`/kiro-impl review-improvement-loop` で回す。
- **前例**: コミット da8e1d7c（manual-ssot-authority）が D7 の文言を 4 ファイル・7 行で更新している。

### 2.4 題材ごとの資産（執筆対象のコード）

| 題材 | 主なソース | 主な完了 spec | 規模の目安 |
|---|---|---|---|
| 1 トランスパイル | `pasta_dsl/src/parser/`（`grammar.pest` 262、ast・parse_* 約 1.9k）、`pasta_lua/src/transpiler.rs`・`code_gen/`（mod・element_gen・scope_gen・source_map）・`context.rs`・`normalize.rs`・`string_literalizer.rs`、`loader/cache.rs`（482） | dsl-separation、pasta-lua-cache-transpiler、pasta-transpiler-variable-expansion、transpiler-ctx-instead-of-act、refine-talk-conversion、lua-passthrough ほか | 約 5k 行 |
| 2 レジストリ・検索 | `pasta_core/src/registry/`（scene_registry・scene_table（RadixMap）・word_*・random）、`pasta_lua/src/search/`、`runtime/finalize.rs`、Lua `pasta/scene.lua`・`pasta/word.lua` | pasta_search_module、scene-search-integration、pasta-scene-dictionary-finalization、word-multi-key ほか | 約 3k 行 |
| 3 実行モデル | Lua `shiori/event/init.lua`（`set_co_scene`）、`pasta/act.lua`・`pasta/shiori/act.lua`・`store.lua`・`save.lua`・`actor.lua`、Rust `runtime/`（mod・exec・factory・persistence・lifecycle・module_registry・runtime_config） | scene-coroutine-execution、coroutine-resume-loop、yield-continuation-token、store-save-*、act-impl-call ほか | 約 4k 行 |
| 4 ローダ・解決 | `loader/`（mod: Phase 1〜5.5、extract: 埋め込み zip・md5 マーカー・準アトミック展開、config/、process、discovery）、`build.rs`・`build_zip.rs`、`runtime/searcher.rs`（357） | pasta-scripts-self-deploy、lua-require-robustness、lua-module-path-resolution、lua-path-restructure、pasta-config-restructure ほか | 約 3k 行 |
| 5 SHIORI 層 | `pasta_shiori/src/windows.rs`（FFI・catch_unwind）、`shiori.rs`、`actor/`（mailbox・thread・marshaling・lifecycle・teardown）、`util/hglobal`・`util/parsers`、`pasta_lua/src/presentation/`・`runtime/renderer_injection.rs`、Lua `callback.lua`・`entry.lua`・`res.lua`・`sakura_builder.lua` | pasta-actor-runtime、shiori-async-talk、shiori-entry、shiori-event-module ほか | 約 4.5k 行 |
| 6 デバッグ | `pasta_lua/src/debug/`（dap/・transport/・session/・wiring/・source_map/・breakpoints・inspect・hook・enable、kick.rs・playscene.rs）、Lua `shiori/event/kick.lua`、`ActorMsg::Kick` | pasta-vscode-lua-debug、pasta-source-map、debug-transport-hardening、pasta-scene-kick(-from-cursor) ほか | 約 7.6k 行（テスト別に約 10k） |

**題材外でコードに存在する内部機構**（Q5 の材料）: `sakura_script/`（tokenizer・wait_inserter・line_breaker（budoux））、仮想イベントディスパッチャ（OnTalk/OnHour・トーク頻度）、`appearance.lua`（516 行・サーフェス復旧）、`encoding/`・`@enc`、`logging/`（約 780 行）・`@pasta_log`、設定読込、`pasta_lsp`（約 2.5k・`pasta_dsl::partial` 部分パース）、`pasta_check`（約 1.7k）、`pasta_sample_ghost`（約 1.1k）。

## 3. 要件 ↔ 資産マップ（ギャップ）

| 要件 | 既存資産 | ギャップ | 種別 |
|---|---|---|---|
| R1 パート新設 | SUMMARY・verify-static・bigram・highlight・Pages | パート追加のみ。書名・description・はじめに章の扱いは未定。新ディレクトリはボイス検査の対象外 | Missing（軽微）/ Unknown（Q1・Q4） |
| R2 題材網羅 | コード・完了 spec 群 | 6 章分の執筆が全て未着手。「2 パス」の現行定義が要る（§3.2） | Missing（大） |
| R3 正確性 | なし（照合の自動化は無い） | design.md の陳腐化が多く、コードとの手照合が必要 | Constraint |
| R4 役割分担 | `reference/startup.md`・lua 章・debug 章 | 内部章からの片方向リンクで足りる。逆リンクの可否は未定 | Unknown（Q4） |
| R5 OPTIMIZATION 吸収 | `OPTIMIZATION.md` | 4 箇所の誤り・陳腐化の修正、将来候補の記録先、参照元 4 箇所の修正 | Missing / Unknown（Q8） |
| R6 internal-modules 生成 | 生成機構・`--check`・SKILL.md 区分表 | 対応表 1 行、test の HANDWRITTEN、SKILL.md L79・L168、生成対象章の `crates/` 禁止、口調の章構造 | Missing（小）/ Constraint（Q3）/ Unknown（Q2） |
| R7 README 移設 | 4 README の内部節 | 移設と絶対 URL 化。README のリンクは link-check の対象外 | Missing / Constraint |
| R8 完了ゲート | workflow.md DoD・kiro-complete | 項目の新設と章↔ソース範囲の対応表 | Missing / Unknown（Q6） |
| R9 定期総点検 | review-improvement-loop D7 | 固定文言の 4 ファイル更新 | Missing（小） |
| R10 一括・品質 | 既存 CI・テスト | 特になし（記述のみ） | — |

### 3.1 制約の衝突（Constraint）

1. **スキル自己完結 vs ソースの所在（R2.7・R3.5 ↔ R6.4）**: link-check はスキル配下の全文で `crates/` の語を禁止する（フェンス内も対象）。`internal-modules.md` の生成元章にソースパスを書くと検査に落ちる。取り得る対応:
   - (a) 生成対象章ではパスを書かず、ソースの所在は別の内部章（例: 実行モデル章）に書く
   - (b) 生成時に特定ブロック（マーカーで囲んだ部分）を落とす機能を生成器へ追加する
   - (c) パスを `pasta_lua/src/...` のように `crates/` を含まない表記にする（規則の抜け穴になるため非推奨）
2. **章構造とボイス（R6.2 ↔ Q1）**: 生成対象章は H1・`---` 2 本・本体に口調なしが必須である。内部パートをボイス無しにする場合も、生成対象章だけは区切り構造を満たす必要がある。verify-content の D-voice は列挙ディレクトリのみ検査するため、ボイス無し方針でも既存検査とは衝突しない。
3. **README リンクの検証外（R7.5）**: クレート README は link-check・manual.yml の対象外で、Pages の URL 変更や章のリネームで README のリンクが黙って切れる。対応候補は、link-check の対象に README を加える、または完了ゲートの確認に含める（Research Needed）。

### 3.2 「2 パス」の現行定義（R2.1）

steering の「2 パス変換（Pass1: シーン登録、Pass2: コード生成）」は現行の pasta_lua トランスパイラとずれている。`transpiler.rs` は文書順の単一ループで登録と生成を行い、実質的な「確定」段は実行時の `finalize_scene`（`runtime/finalize.rs`）が担う。`pasta_core` の registry コメントにのみ 2 パスの語が残る。執筆時に現行の段階構成を定義し直す必要があり、steering `tech.md` の記述とずれる点の扱い（steering 再編は Out）も設計で判断する。

### 3.3 完了 spec の陳腐化した参照（R3 の照合コストの根拠）

- `code_generator.rs` → `code_gen/`（pasta-scene-dictionary-finalization・OPTIMIZATION.md）
- `scripts/pasta/*.lua` → `pasta_scripts/`（scene-search-integration・shiori-entry・lua-module-path-resolution）
- `loader/config.rs` → `loader/config/{mod,sections}.rs`（pasta-scripts-self-deploy ほか）
- `debug/{dap,session,source_map,wiring,transport}.rs` → 各ディレクトリ（pasta-source-map・debug-transport-hardening）
- `actor_poc/` 削除済み（pasta-actor-runtime。コード内 doc コメントにも残存）
- `vscode-debuggee.lua` 不在（pasta-vscode-lua-debug。現行は Rust ホストの DAP）
- `lib.rs` 冒頭の「pasta_rune と同構成・Lua 5.3+」（実際は LuaJIT）

これらは R3.4 / Q10 の「記録のみ」対象になる。

## 4. 実装アプローチの選択肢

### Option A: 既存拡張のみ（パートを 1〜2 ファイルの大章として追加）
- `SUMMARY.md` に内部パートと少数の大きな章を追加し、`internal-modules.md` を 1 章として対応表へ追加する。
- ✅ ファイル数が最小で、生成機構・検証の変更が最小。
- ❌ 6 題材×数千行規模を少数章に詰めると、ナビゲーション・検索の粒度が粗くなり、完了ゲートの「章ごとの対象領域」（R8.2）の対応付けも粗くなる。

### Option B: 新規構成（内部パート専用の規約・検証を新設）
- `book/src/internals/` を新設し、内部パート専用の執筆規約（AUTHORING.md の節）、ソースパス実在検査などの新検証、README リンク検証を追加する。
- ✅ 鮮度の自動検出が強く、OPTIMIZATION.md の二の舞を構造的に防ぐ。
- ❌ brief の「鮮度維持は a＋b」を越える新機構になる（Q7）。生成機構・CI への変更が増える。

### Option C: ハイブリッド（推奨候補）
- 基盤は既存拡張とする: SUMMARY のパート追加、対応表 1 行、`HANDWRITTEN` 更新、SKILL.md 区分表更新。
- 執筆は題材ごとの独立章として新規作成する: 概要章＋6 題材章＋`internal-modules` 相当章（生成元）。完了ゲートの「章 ↔ ソース範囲」対応表は概要章または workflow.md に置く。
- 新規の自動検査は持たず、ディスカッションで要とされた場合のみ追加する（Q7）。
- 段階: (1) パート骨格＋対応表＋ゲート組み込み → (2) 題材章の執筆（題材ごとに独立・並行可） → (3) 吸収元の削除・README リンク化・参照修正。ただし R10.1 により出荷は一括。
- ✅ brief の境界候補（執筆／集約／運用組み込み）とそのまま対応し、題材ごとに並行して執筆できる。
- ❌ 執筆量は減らない。題材章間の重複（例: ACT は実行モデル章と internal-modules 章の両方に現れる）の線引きが必要。

## 5. 工数とリスク

- **工数: L（1〜2 週間）**。基盤側の変更は S 規模だが、6 題材の執筆と現行コードとの照合（Rust 約 2.5 万行・Lua 約 3.9k 行・完了 spec 約 60 件）、README 4 本の移設、internal-modules 796 行の照合が大きい。
- **リスク: Medium**。技術的に未知の要素は無い（既存機構の利用）。一方で、正確性を自動検証できず手照合に依存すること、Q2・Q3 のように権威と境界に関わる判断が設計を左右すること、生成物と自己完結規則の衝突があることがリスク要因である。

## 6. 設計フェーズへの申し送り

### 推奨と主要判断
- Option C を軸に、要件ディスカッションで Q1〜Q10 を確定させてから設計する。
- 先に決めるべき順序: Q2（internal-modules の帰属）→ Q3（自己完結とパス表記）→ Q1（文体）。この 3 つで生成対象章の書き方が決まる。

### Research Needed
- 生成対象章（`internal-modules` 相当）の章名と出力名の対応（`outName` がベース名を使うため、章ファイル名を `internal-modules.md` にするか、出力名の上書き機能を追加するか）。
- bigram 検索索引・構文ハイライトが新パートの HTML を自動で処理することの確認（ツールが SUMMARY 全章・出力 HTML 全体を対象にしているかの実地確認）。
- クレート README からのリンクの鮮度担保の手段（link-check の対象拡張か、ゲートでの目視確認か）。
- 完了ゲートの「内部設計章 ↔ ソース範囲」対応表の置き場所（workflow.md / 概要章 / 両方）と、kiro-complete の発火条件の書き方（Manual Sync Gate と同型のパス条件にするか）。
- `review-improvement-loop` の D7 文言を更新する際の、生成済みセル（tasks.md L770-779）を触らない範囲の確定。
- steering `tech.md` の「2 パス変換」記述との齟齬の扱い（steering 再編は Out。リンク追記で足りるか）。

### 要件ディスカッションで設計へ回した判断（カテゴリ B）
- 完了ゲートの形式（新規条件付き Gate 7 か Doc Gate 拡張か）と「章 ↔ ソース範囲」対応表の置き場所・発火条件の書き方（旧 Q6。R8.2 の「機械的に照合できる形」を満たすこと）。
- 生成対象章でソースパスを書かない方式（§3.1 の (a)/(b)/(c)。(c) は非推奨）。
- 題材章どうしの重複の線引き（例: ACT は実行モデル章と内部モジュール章の両方に現れる）。
- クレート README からマニュアルへの絶対 URL リンクの鮮度担保（link-check 対象拡張か、ゲートでの確認か）。
- steering `tech.md` の「2 パス変換」記述との齟齬の扱い（steering 再編は Out。リンク追記の要否）。
