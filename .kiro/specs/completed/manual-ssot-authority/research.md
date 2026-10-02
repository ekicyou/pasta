# Research: manual-ssot-authority

## ギャップ分析（2026-10-01・要件ドラフト時点）

要件（`requirements.md`・未承認ドラフト）と既存リポジトリの差分を調査した。本書は判断材料の提示であり、最終決定は設計フェーズで行う。

### 1. 現状調査

#### 1.1 関連資産と規模

| 資産 | 規模 | 現在の役割 |
| --- | --- | --- |
| `doc/spec/`（12 章 + README） | 1745 行 | 「実装判断の権威」を自称。ch08（属性・65 行）と ch12（将来仕様・145 行）は mdBook に対応章なし |
| `GRAMMAR.md` | 831 行 | 学習用クイックリファレンス。冒頭で doc/spec を権威と案内 |
| `book/src/grammar/`（10 章） | 約 1230 行 | 各章末に `doc/spec` への「権威的仕様」リンク（GitHub 絶対 URL） |
| `book/src/lua/`（5 章） | 679 行 | `modules.md` 224 行・`patterns.md` 196 行 |
| `book/src/reference/` | `startup.md` 174 行・`external-links.md` 67 行 | `pasta.toml` の章は無い |
| `pasta-ghost-authoring/references/` | 文法 7 ファイル 1041 行 + `pasta-toml.md` 409 行 + `authoring-patterns.md` 348 行 | 手書き。各ファイルが「外部参照不要」と自称 |
| `pasta-lua-coding/references/` | `runtime-api` 710・`shiori-handlers` 442・`internal-modules` 794・`coding-conventions` 407・`testing-lint` 307 行 | 手書き |
| `book/tools/drift-check.mjs`（401 行）＋テスト（533 行）、`verify-drift-gate.mjs`（332 行）、`book/manual-sources.toml`（10 マッピング） | — | doc/spec 追従のドリフト検出。**リンク切れ検出も同居** |

#### 1.2 章構造（生成の前提）

- 文法・Lua・リファレンスの全章が「見出し → キャラ口調の導入 → `---`（7〜8 行目）→ 規範的本文 → `---` → キャラ口調の締め」という構造を持つ。
- 文法章は締めの後に `> **権威的仕様**: ... doc/spec/...` の引用ブロックを持つ。
- `lua/modules.md` は本文内にも `---` を 7 本持つ（節区切り）。「最初の `---` まで」「最後の `---` 以降」は機械的に判定できるが、本文内 `---` と締め区切りの区別は「最後の 1 本」という位置規約に依存する。
- 章間の相対リンクが多数ある（例: `grammar/action-line.md` → `variables.md`、`grammar/variables.md` → `../lua/patterns.md`、`lua/patterns.md` → `../grammar/variables.md#エンジンが値を入れる変数`、`lua/basics.md` → `../reference/external-links.md`）。
- 文法・Lua・リファレンス章に mdBook 固有記法（`{{#include}}` 等）や画像は無い。デバッグ章のみ mp4 を持つ。

#### 1.3 既存の規約・ツールチェーン

- `book/tools/` は依存なしの素の Node スクリプト（`.mjs`）＋同居の `*-test.mjs` 自己テスト。CI がテストを `find` で一括実行する。
- `book/package.json` の依存は構文ハイライト用（`vscode-textmate` 等）のみ。
- 改行の LF 正規化は `drift-check.mjs`・`tutorial-check.mjs` に既存実装（`/\r\n?/g`）があり、流用できる。
- `.github/workflows/manual.yml` は **`book/**` と自身の変更時のみ起動**。`build.yml`（Windows・cargo）は全 push/PR で起動。

#### 1.4 参照箇所の全数（`.kiro/specs/completed/` を除く）

- **`doc/spec` 参照**: steering（`grammar.md` 27 件・`workflow.md` 12・`product.md` 3・`structure.md` 3・`tech.md` 2・`roadmap.md` 2）、`README.md` 2、`SOUL.md` 2（「衝突時は doc/spec を優先」ルールを含む）、`OPTIMIZATION.md` 1、`kiro-complete/SKILL.md` 3、`pasta-ghost-authoring/references/*` 7 ファイル各 1（`<!-- source: ... -->` コメント）、`book/AUTHORING.md` 13、`book/CONTENT-REVIEW.md` 6、`book/src/grammar/*` 全章、`book/src/introduction.md` 1、`book/src/reference/external-links.md` 11、`book/tools/verify-content.mjs` 6、進行中 spec（`review-improvement-loop` の brief/matrix/tasks、`pasta-runtime-internals-doc/brief.md`）。
- **`GRAMMAR.md` 参照**: steering（`grammar.md`・`structure.md`・`workflow.md`・`roadmap.md`）、`README.md`、`SOUL.md`、`book/AUTHORING.md`、`crates/pasta_lua/tests/runtime/syntax_test.rs`（コメント 1 件）、`review-improvement-loop`（matrix/tasks）。
- **drift 機構参照**: `manual.yml`、`kiro-complete/SKILL.md`（Manual Sync Gate）、steering（`workflow.md`・`tech.md`・`structure.md`・`roadmap.md`）、`book/AUTHORING.md`、`book/CONTENT-REVIEW.md`、`book/tools/verify-content.mjs`・`verify-scripts-test.mjs`・`tutorial-check*.mjs`（コメントのみ）、`review-improvement-loop`（design/requirements/research/matrix/tasks/reports）。
- **スキルファイルをコードから読む箇所**: `crates/pasta_lua/tests/loader/config_defaults_test.rs:226` の 1 箇所のみ（`pasta-toml.md` の「キー名を含む行に `` `値` `` がある」ことを行単位で検査）。

### 2. 要件ごとの資産対応とギャップ

| 要件 | 既存資産 | ギャップ | 種別 |
| --- | --- | --- | --- |
| R1 文法の一本化 | `book/src/grammar/` 10 章 | doc/spec（1745 行）に対し mdBook 文法章は約 1230 行。例: `02-markers.md` 458 行に対し `markers.md` 129 行。規範的詳細の厚み付けが必要。`GRAMMAR.md` のチェイントーク記述は book にも doc/spec にも無い。ch08/ch12 の行き先が無い | Missing |
| R2 doc/spec・GRAMMAR.md 廃止 | — | 削除と案内化のみ。`verify-content.mjs` が doc/spec リンクを必須としているため同時に改修が必要 | Constraint |
| R3 Lua API | `lua/modules.md` 224 行・`patterns.md` 196 行 | スキル側 1152 行（`runtime-api`＋`shiori-handlers`）の内容が大半未収録。SHIORI イベント一覧・仮想ディスパッチャの章が無い。新章には導入・締めのキャラ口調の書き下ろしが必要 | Missing |
| R4 pasta.toml 章 | スキル `pasta-toml.md` 409 行 | mdBook に章なし。テストは行単位の文字列一致に依存するため、移設先でも「キー名と `` `値` `` が同一行」という表形式を保つ必要がある | Missing / Constraint |
| R5 生成 | LF 正規化・章走査の既存実装 | 生成スクリプトが無い。章間相対リンクの書き換え規則、章→スキルファイルの対応表、生成物ヘッダの仕様が未定 | Missing |
| R6 区分明示 | 両 `SKILL.md` に references 一覧あり | 生成／手書きの区分表記なし。`pasta-lua-coding/SKILL.md:40` が `../../../book/src/reference/startup.md` を参照（既存の自己完結違反）。`runtime-api`・`shiori-handlers` 末尾の「関連リファレンス」は手書きの `internal-modules.md` を指す | Missing |
| R7 鮮度チェック | CI の自己テスト一括実行、kiro-complete の Manual Sync Gate | チェック本体なし。`manual.yml` の起動条件が `book/**` のみで、スキル側だけの手編集では起動しない | Missing / Constraint |
| R8 drift 撤去 | — | `drift-check.mjs` はリンク切れ検出（book 内相対 `.md` リンク・GitHub blob URL）も担う。丸ごと削除するとリンク切れ検出が消える。`verify-content.mjs` の A 節（権威リンク・manual-sources 整合）と `verify-scripts-test.mjs` も連動改修が必要 | Constraint |
| R9 参照修正 | — | 1.4 のとおり広範。`steering/grammar.md` は doc/spec の要約であり、参照付け替えだけでは「写し」が残る。`SOUL.md` の優先ルール書き換えが必要 | Missing |
| R10 一括完了 | kiro-complete（PR ベース squash マージ） | 単一 PR に全変更を載せる運用で満たせる。変更量が大きくレビュー負荷が高い | Constraint |

### 3. 主要な統合上の課題

1. **粒度の不一致**: mdBook 文法は 10 章、スキル文法 references は 7 ファイル（`grammar-model.md` が markers / block-structure / literals / 属性 / キューコマンドを束ねる）。`SKILL.md` は現行ファイル名へ 30 箇所以上リンクしている。
2. **情報量の逆転**: スキル側にしか無い内容（属性の配置ルール、キューコマンド構文、永続化と SAVE、SHIORI イベント一覧 等）を先に mdBook へ移さないと、生成に切り替えた時点でスキルの情報が減る。
3. **キャラ口調の除去範囲**: 導入に加えて締め段落と「権威的仕様」引用がある。brief の記述（導入のみ）どおりだと締めのキャラ口調がスキルに混入する。
4. **章間リンクの自己完結化**: 生成対象外の章（例: `external-links.md`、`getting-started`）へのリンクは、スキル内で解決できない。
5. **鮮度チェックの起動条件**: スキル手編集を捕捉するには CI の起動範囲拡大が必要。
6. **リンク切れ検出の同居**: drift-check 撤去とリンク切れ検出維持の切り分け。
7. **既定値整合テストの実行環境**: `build.yml`（Windows）で走る cargo テストが `book/src/` を読むことになる。パス解決は既存の `repo_root()` で足りる。

### 4. 実装アプローチの選択肢

#### 4.1 生成物の構成

- **A. 章と 1:1 で生成**（スキル references を mdBook の章構成に合わせて作り直す）
  - ✅ 対応表が自明、生成スクリプトが最小、章間リンクは同名ファイルへの置換で解決
  - ❌ `SKILL.md` のリンク全面書き換え、持ち出し先から見てファイル名が変わる
- **B. 現行ファイル名を維持し、複数章を連結して生成**（対応表で `grammar-model.md` ← index+markers+block-structure+literals 等）
  - ✅ `SKILL.md`・持ち出し先への影響が小さい
  - ❌ 対応表と見出しレベル調整・リンク書き換えが複雑化。アンカー衝突の扱いが必要
- **C. ハイブリッド**（文法は 1:1 に改め、Lua API / pasta.toml は現行ファイル名を維持）
  - ✅ 文法は章構成が既に細かく 1:1 が自然、Lua 側は mdBook の章を新設する段階でファイル名を合わせられる
  - ❌ 規則が 2 種類になる

#### 4.2 drift-check の撤去方法

- **A. 丸ごと削除し、リンク切れ検出は捨てる** — 最小。ただし既存の検証能力が減る。
- **B. リンク切れ検出部のみを残して改名・縮小** — 既存テストの該当部を流用可能。ドリフト部・`manual-sources.toml`・`verify-drift-gate.mjs` は削除。
- **C. リンク切れ検出を `verify-content.mjs` へ移す** — ファイル数は減るが `verify-content.mjs`（429 行）が肥大。

#### 4.3 鮮度チェックの置き場所

- **A. `manual.yml` の起動条件にスキルのパスを追加** — ワークフロー追加なし。スキルだけの変更で mdBook ビルド一式が走る。
- **B. `build.yml` に Node ステップを追加** — 全 PR で必ず走る。Windows ランナーで CRLF 耐性も同時に検証できるが、Node セットアップが増える。
- **C. 生成スクリプトに `--check` を持たせ、cargo テストから呼ばず CI と完了ゲートの両方で同じコマンドを実行** — A/B いずれとも併用可能。ローカル再現（R7.7）を満たす。

#### 4.4 作業順序（一括完了の内訳）

内容移設（doc/spec 吸収・Lua API 拡充・pasta.toml 章）→ 生成機構と対応表 → スキル切替と `SKILL.md` 更新 → 鮮度チェックと CI → drift 機構撤去と `verify-content` 改修 → 旧文書削除と参照修正、の順が依存上自然。すべて同一ブランチで行い、最後に単一 PR で統合する。

### 5. 規模とリスク

- **Effort: L（1〜2 週間）** — 生成スクリプト自体は小さいが、約 1700 行の doc/spec 吸収、約 1500 行のスキル内容の mdBook 化（キャラ口調の導入・締めの書き下ろしを含む）、40 以上のファイルにまたがる参照修正がある。
- **Risk: Medium** — 技術的に未知の要素は無い。リスクは (1) 移設時の規範内容の欠落、(2) 文法の記述変更が意図せず「仕様変更」に見えること、(3) 単一 PR の大きさによるレビュー漏れ。欠落は移設前後の見出し・キーワード対照表で機械的に抑えられる。

### 6. 設計フェーズへの申し送り

**検討が必要な決定事項**

- 章 → スキルファイルの対応表の持ち方（4.1 の A/B/C）と、対応表の置き場所。
- 生成時に落とす範囲の規約（導入・締め・「権威的仕様」引用・章中コラム）と、それを執筆規約 `book/AUTHORING.md` にどう書くか。
- 章間リンクの書き換え規則（生成対象章 → スキル内ファイル、対象外章 → 公開マニュアル URL か削除か）。
- 生成物ヘッダの文言と位置（R5.6）。
- ch08 / ch12 の収録先。
- リンク切れ検出の存続方法（4.2）。
- 鮮度チェックの CI 配置（4.3）と kiro-complete の Manual Sync Gate の置き換え文面。
- `steering/grammar.md` を参照付け替えに留めるか縮小するか。

**Research Needed**

- doc/spec 各章と mdBook 文法章の節単位の差分（どの規範的記述が未収録か）の全数洗い出し。
- `GRAMMAR.md` 固有内容（チェイントーク・選択肢定義・キューコマンド行）の現行実装との整合確認。
- `runtime-api.md` / `shiori-handlers.md` / `pasta-toml.md` と現行実装の食い違いの有無（移設時に誤りを権威化しないため）。
- `authoring-patterns.md` に含まれる規範的事実（§6.4 時報変数・§6.6 シャッフル消費・§6.7 チェイントーク等）の切り分け。
- 公開マニュアルの外から `doc/spec/` を指す既存リンク（crates.io の README、VS Code 拡張の説明等）の有無。リポジトリ内 grep では該当なし。

---

## 設計フェーズの調査と決定（2026-10-01）

### Summary

- **Feature**: `manual-ssot-authority`
- **Discovery Scope**: Extension（既存の mdBook 基盤・`book/tools`・CI への統合）。新規外部依存なし（Light discovery）。
- **Key Findings**:
  - 全 17 章（文法 10・Lua 5・リファレンス 2）が「最初と最後の `---` 行」で導入・本文・締めに分かれる。`lua/modules.md` は本文内にも `---` を 7 本持つが、最初／最後の位置規約は全章で成立する。生成器の抽出規則はこれで足りる。
  - 吸収元（doc/spec・book・スキル・GRAMMAR.md）の記述に、実装と食い違う箇所が約 20 件ある（行継続の `：` 必須、ローカル／グローバル候補の非統合、引数のカンマ区切り、REG ハンドラの引数が `act`、`RES.ok_with` が存在しない、さくらスクリプトのウェイト設定キーと既定値 等）。そのまま権威化・生成すると誤りがスキルへ伝播する。
  - スキルを読むコードは `config_defaults_test.rs` の 1 箇所のみで、照合規則は「キー名と `` `値` `` が同一行」（節スコープなし、4 キー）。読み先パスを変えるだけで移設できる。

### Research Log

#### 章構造と章間リンク
- **Sources**: `book/src/**/*.md` の `---` 行位置、相対リンク抽出。
- **Findings**: 区切り行は文法章で 7（index は 8）行目と末尾 5〜6 行前。文法章は締めの後に `> **権威的仕様**` 引用（doc/spec の GitHub URL）を持つ。パート間の相対リンクは 6 本のみ（`grammar/variables→lua/patterns`、`lua/basics・lua/index→reference/external-links`、`lua/patterns→grammar/variables#…`、`reference/startup→lua/modules`、`reference/startup→debug/troubleshooting`）。対象章に画像・`{{#include}}` は無い。
- **Implications**: 「同一スキルの生成ファイルへは兄弟リンク、それ以外は公開 URL」という 2 規則で全リンクを処理できる。画像等は出現時にエラーとすればよい。

#### 吸収元と実装の食い違い（文法）
- **Sources**: `crates/pasta_dsl/src/parser/grammar.pest`、`code_gen/element_gen.rs`・`scope_gen.rs`、`pasta_core/src/registry/scene_table.rs`・`word_table.rs`、`pasta_scripts/pasta/act.lua`。
- **Findings**: 行継続（pest:237 `continue_action_line` は `：` 必須）、継続内空行は無出力（pest:222,257）、ローカル優先で統合しない（scene_table.rs:318-381、word_table.rs:90-107）、Call 検索 5 段（act.lua:299-350）、Call フィルターは構文なし（pest:161）、引数はカンマ区切り・位置引数可（pest:104-107）、文字列エスケープなし（pest:110-130）、単語値はカンマ区切り（pest:144-146）、属性行はグローバルシーン初期部のみ（pest:153-157 ほか）、Lua フェンスは 3 個以上＋任意識別子（pest:217）、`!select` は `act:choice_timeout` を生成（scope_gen.rs:385-404）、単語値にさくらスクリプト可（pest:145）。どの文書にも無い実装事実（単独 `＊` 行、`％a＝0、b`、`＄０`、末尾 `#` コメント等）もある。
- **Implications**: 移し替えは「実装照合つき」で行う必要がある。吸収台帳に照合位置の列を設ける。

#### 吸収元と実装の食い違い（Lua・設定）
- **Sources**: `pasta_scripts/pasta/shiori/event/init.lua`・`register.lua`・`res.lua`・`second_change.lua`・`callback.lua`・`boot.lua`、`crates/pasta_lua/src/sakura_script/mod.rs`・`wait_inserter.rs`、`loader/config/sections.rs`。
- **Findings**: ハンドラは `handler(act)`（init.lua:183-184）、`RES.ok_with` は無く `RES.ok(value, dic)` 等 9 関数（res.lua:39-138）、シーン関数フォールバックは `SCENE.co_exec` で 200（init.lua:161-199）、OnSecondChange 既定ハンドラが仮想ディスパッチャを駆動（second_change.lua:15-24）、`CALLBACK.resume_pending` は存在しない、ウェイトはアクター表直下の `script_wait_*`・既定 50/1000/500/500/200・挿入値 `値-50`・連続句読点は最大値（sections.rs:232-246、wait_inserter.rs:55-61）。book の `lua/modules.md`・`lua/patterns.md` も同じ誤りを持つ。`pasta-toml.md` の `[talk]` 記述は正しい。`@pasta_search`・`@pasta_persistence`・`@enc`・`@pasta_log`・仮想ディスパッチャの API は一致。
- **Implications**: 生成対象外の `lua/patterns.md` と `pasta-lua-coding/SKILL.md` 早見表も同時訂正が必要。

#### 既定値整合テスト
- **Sources**: `crates/pasta_lua/tests/loader/config_defaults_test.rs:213-247`。
- **Findings**: `repo_root()`（`CARGO_MANIFEST_DIR` の 2 階層上）＋固定相対パス。`talk_interval_min/max`・`hour_margin`・`spot_newlines` の 4 キーを全行走査で照合し、失敗時にキー名と期待値を出す。
- **Implications**: パス文字列の置換のみで 4.3–4.5 を満たす。マニュアル章側に「キー名と `` `値` `` の同一行」の表形式を規約として課す。

#### CI と完了ゲート
- **Sources**: `.github/workflows/manual.yml`・`build.yml`、`workflow.md` DoD、`kiro-complete/SKILL.md`、`verify-drift-gate.mjs`。
- **Findings**: `manual.yml` は `book/**` のみで起動、`deploy` は `needs: build`。`verify-drift-gate.mjs` は `workflow.md` と `kiro-complete/SKILL.md` の文言を検査しているため、ゲート文面の変更と同時に削除が必要。`.agents/skills/` を指す記述（SOUL.md・tech.md・workflow.md・GRAMMAR.md）は実在しないパス。
- **Implications**: `paths` に 2 スキルを足すだけで 7.5 を満たす。ゲート文面の自己検査は撤去し、機械検証はコマンド（`--check`・link-check）に一本化する。

#### doc/spec ch08・ch12
- **Findings**: ロードマップにはどの項目も未記載。属性（行は受理、ファイルレベル属性は統合まで実装済みで未利用）とフィルター（構文未受理）は具体的な構文定義があり brief 化できる。`＠＄` は文法定義のみでパーサ未実装・優先度低。§12.9 は `ctx.local/global` 前提で実装（`var`/`save`）に置換済み。§12.2 は `＞チェイントーク` の実装で事実上置換。他の §12.x の多くは現行挙動の事実でありマニュアルへ移せる。

### Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 生成物コミット＋`--check` 照合 | 生成ファイルをリポジトリに置き、CI で再生成結果と比較 | スキルをディレクトリコピーだけで持ち出せる。照合と生成が同一関数 | 生成忘れは CI で落ちる（意図どおり） | 採用 |
| ビルド時生成（コミットしない） | CI・リリース時にのみ生成 | 差分が出ない | 持ち出し時に生成手順が必要、5.3 の自己完結と両立しない | 却下 |
| 章ごとマーカーコメントで抽出範囲を指定 | `<!-- skill:begin -->` 等 | 位置規約に依存しない | 全章に執筆負担、既存規約と二重 | 却下（`---` 規約で足りる） |

### Design Decisions

#### Decision: 対応表は生成器内の定数、出力名は章名から導出
- **Context**: research §4.1 の A/B/C。
- **Alternatives**: A 章と 1:1（章名）／B 現行名維持＋複数章連結／C 混在。
- **Selected Approach**: A。章と 1:1（連結なし）で、出力名は `outName(chapter)` で章パスから導出する（`index.md` は `{直近の親ディレクトリ名}-index.md`）。旧名 `grammar-model.md`・`call-spec.md`・`runtime-api.md`・`shiori-handlers.md` は削除する。
- **Rationale**: 連結（B）は見出しレベル調整とアンカー衝突処理が要る。当初案は「現行名を明示表で維持」だったが、章名とスキル名のずれ（`call-jump`↔`call-spec`）が整理負債として残るため、設計ディスカッション #3 で章名に統一した。
- **Trade-offs**: `SKILL.md` のリンク全面張り替え。持ち出し先は `references/` を丸ごと置き換える必要がある（`SKILL.md` に明記）。

#### Decision: 抽出は「最初と最後の `---`」、本文の口調は機械検出で全面禁止
- **Context**: research §6「落とす範囲の規約」。
- **Selected Approach**: コードフェンス外の行全体 `---` を区切りとし、最初と最後の間を本文とする。締め以降（旧「権威的仕様」引用含む）は捨てる。本文の散文（コードフェンス・表の行・インラインコードを除く）に広い `VOICE_MARKERS` のいずれかがあれば生成失敗（設計ディスカッション #1。当初案の狭いナレーション語のみの検出から拡大）。
- **Rationale**: 既存全章が満たす規約で、新しい記法を導入しない。コラムの口調混入を規約だけでなく機械で止める。
- **Trade-offs**: 生成対象章では本文中コラムが書けない（`AUTHORING.md` を改訂）。部分一致のため普通文体と衝突する語がある（下記「設計再検証」参照）。

#### Decision: 章外リンクは公開マニュアル URL へ書き換え
- **Selected Approach**: 同一スキルの生成ファイル宛て → 兄弟ファイル名＋アンカー。非生成章・別スキル宛て → `https://ekicyou.github.io/pasta/{章}.html#anchor`。画像等の相対リンクはエラー。
- **Rationale**: 5.4 の「参照切れを残さない」を満たしつつ、読者が辿れる導線を残す。絶対 URL は持ち出し先でも切れない。
- **Alternatives**: リンクを外してテキストだけ残す（導線が消える）。

#### Decision: リンク検証は drift-check を縮小改名して存続し、スキル自己完結検査を同居
- **Context**: research §4.2。
- **Selected Approach**: B（`link-check.mjs` へ改名、ドリフト・未マップ・TOML パーサを削除）。スキル 2 つの相対リンクの脱出・欠落と、`doc/spec`・`GRAMMAR.md`・`book/src` の語の混入を検出する機能を追加。
- **Rationale**: 既存のリンク検証とテストをそのまま流用でき、「リンクの健全性」という単一責務にまとまる。C（verify-content へ統合）は 429 行のファイルをさらに肥大させる。

#### Decision: 鮮度チェックは `manual.yml` の `paths` 拡張＋先頭ステップ
- **Context**: research §4.3。
- **Selected Approach**: A＋C。`paths` に 2 スキルを追加し、Setup Node 直後に `gen-skill-refs.mjs --check`。完了ゲートも同一コマンド。
- **Rationale**: ワークフローを増やさず、失敗時は `needs: build` で公開が止まる。`build.yml`（Windows 2 アーキ行列）への Node 追加はコストに見合わない。

#### Decision: 吸収の網羅は台帳で証明
- **Selected Approach**: `absorption-ledger.md` に吸収元の全見出しを列挙し、収録先 or 除外理由と実装照合位置を記録。全行充足を完了条件とする。
- **Rationale**: 10.6（黙って破棄しない）と 1.6/3.3（実装に一致）を同時に機械的に近い形で確認できる唯一の手段。research §5 の「見出し・キーワード対照表」案を具体化。

### Synthesis

- **Generalization**: 「文法・Lua API・pasta.toml・起動シーケンス」は同一の問題（章→スキルファイルの写像）であり、1 つの対応表と 1 つの生成関数で扱う。下流 `pasta-runtime-internals-doc` も対応表への行追加だけで再利用できる（インターフェースのみ汎用、実装は 21 行の定数）。
- **Build vs. Adopt**: mdBook の Markdown 出力プラグイン（mdbook-markdown 等）や remark 系ライブラリは、新規エコシステム依存（5.8 違反）かつ必要機能（区切り抽出・リンク書き換え）が数十行で済むため不採用。リンク抽出は既存 `drift-check.mjs` の実装を再利用する。
- **Simplification**: 対応表は設定ファイル化しない（消費者は生成器と verify-content のみ）。生成ヘッダの検出は 1 行目の固定文字列のみ。スキル内アンカーは 2 スキルに限って検証する（設計ディスカッション #2。book 内アンカーは検証しない）。`verify-drift-gate.mjs` 相当のゲート文面自己検査は再実装しない。

### Risks & Mitigations

- 規範内容の欠落 — 吸収台帳の全行充足を完了条件にする。
- 食い違い訂正による作業量増・「仕様変更」と誤解される — 台帳に実装照合位置を残し、挙動不変であることを示す。
- 単一 PR の大きさ — 移行フェーズ単位でコミットを分け、PR 説明に台帳を添える。
- 本文コラム禁止による執筆上の制約 — `voice-in-body` 検出と `AUTHORING.md` の規約化で早期に気づけるようにする。

---

## 設計再検証（整合性パス・2026-10-01）

設計ディスカッション（全 11 件）の決定を前提に、design.md 全体をリポジトリの実物と突き合わせた。見つかった問題と修正を記録する。

### 決定の波及漏れ（リポジトリの証拠つき）

| # | 見つかった問題 | 証拠 | design.md の修正 |
|---|----------------|------|------------------|
| 1 | `lua/modules/` への分割で `verify-content.mjs` の B が失敗し、D がモジュール章を検査しなくなる | B・D は固定ディレクトリを非再帰で読む（`book/tools/verify-content.mjs:170`・`:241`）。`@pasta_search` 等 5 モジュールは `lua/modules.md` にしか登場しない | VerifyContent に「B・D の一覧へ `lua/modules` を追加」を明記。Migration P2 に配置 |
| 2 | フェンス判定が単純な交互カウントだと、4 連バッククォート内の Lua ブロックが散文扱いになる | `book/src/grammar/block-structure.md:94-103`・`call-jump.md:79-107`・`actor-dictionary.md:67-76` | `maskFences`（CommonMark 準拠）を `link-check.mjs` に定義し生成器と共用。単体テストを追加 |
| 3 | `skill-anchor` を見出しだけで判定すると、明示アンカーへの既存リンクが誤検出になる | `authoring-patterns.md` の `<a id="s6-N">`（L8 ほか 11 箇所）と `pasta-ghost-authoring/SKILL.md:65,123,124,168,349,350` | アンカー集合を「見出し slug ∪ `<a id>`／`<a name>`」に変更 |
| 4 | `headingSlug` の規則が「英数字・日本語」と曖昧 | 実見出し `## [package] 予約注記`（`pasta-toml.md:161`）、`### 予約グローバル変数（pasta_ で始まる名前）`（`grammar/variables.md:164`） | Unicode 文字・数字＋`_`・`-`・空白を残す 5 手順として定義し、実例 3 つを記載 |
| 5 | 吸収元に既に切れたアンカーがある | `pasta-toml.md:51,409` の `#package予約注記`（正しい slug は `package-予約注記`） | ContentMigration に「移設時に直す」を追加（`skill-anchor` が検出する） |
| 6 | 広い `VOICE_MARKERS` が普通文体と衝突する | `くてよ` が「書かなくてよい」に一致（`pasta-toml.md:21`。吸収対象）。`ですの`・`ますの` は「ですので」「ますので」に一致。現行の生成対象章の本文には一致なし（全 12 章を走査） | 前提 A1（集合は変えず言い換える。エラーは行番号つき）として明記し、開発者確認事項とする |
| 7 | 旧名ファイルの削除し忘れを検出する手段が無い | 旧名 4 ファイルは生成ヘッダを持たないため孤立検出に掛からない | LinkCheck に `skill-unlisted`（`references/*.md` が `SKILL.md` からリンクされていること）を追加。6.1 の区分漏れも同時に検出 |
| 8 | Out of Boundary が決定 #6 と矛盾 | 旧記述「入門章・デバッグ章は参照されるのみ」「手書き 3 ファイルは外部参照除去以外触らない」 | 「誤記訂正・リンク張り替えは行う」へ修正。`first-ghost.md` の成果物ブロックは `tutorial-check` 対象のため不変と明記 |
| 9 | 分割で切れる book 内リンクが File Structure Plan に無い | `book/src/lua/index.md:43`・`reference/startup.md:114`・`SUMMARY.md:28`、および `lua/modules.md:18,82` の `patterns.md` | `lua/index.md` を変更対象へ追加。`startup.md` の変更内容を「リンク張り替えのみ」に訂正（本文に口調は無い） |
| 10 | book 内アンカーは未検証のため、見出し変更で黙って切れる | `getting-started/first-ghost.md:272,322`・`lua/patterns.md:117` → `grammar/variables.md` の 3 見出し | 維持すべき見出しとして明記 |
| 11 | マニュアルの既存「将来変更あり」節が要件 1.3 と衝突 | `grammar/call-jump.md:109`（フィルター・構文未受理）、`words.md:96`（`＠＄`・パーサ未実装）、`block-structure.md:112`、`actor-dictionary.md:63`、`grammar/index.md:80` | 未実装機能の節は削除して brief へ、受理されるが未処理の構文は現行挙動のみ記述。前提 A3 |
| 12 | スキル内のリポジトリ内パス記述を検出できない（6.5） | `pasta-toml.md:29`・`testing-lint.md:247` の `crates/…` | `FORBIDDEN_SKILL_TOKENS` に `crates/` を追加。マニュアルの pasta.toml 章にはリポジトリ内パスを書かない |
| 13 | `workflow.md` のスキル更新手順が「references を直接更新」のまま | `.kiro/steering/workflow.md:62-79` | CompletionGate に手順の書き換えを追加（9.5） |
| 14 | 移行順序の依存逆転 | 生成器が `link-check.mjs` を import するのに、旧案は生成器（P3）→ link-check（P5）の順。章末引用の削除は現行 `A-link`（`verify-content.mjs:122`）と同時でないと赤になる | P1〜P8 を組み直し、各フェーズの終了条件を記載 |
| 15 | マニュアル章からスキル手書きファイルへはリンクできない | `runtime-api.md:114,709`・`shiori-handlers.md:441` が `testing-lint.md`・`internal-modules.md` を指す | 台帳で「除外（スキル内ナビゲーション）」とする規則を追加 |

### 旧記述の残骸

- `MapEntry` から消えた `out` を参照する記述（リンク書き換え規則・出力先・Data Models）を `outName(chapter)` に統一。
- `outName` の重複を「`bad-structure` 相当の例外」とする曖昧な記述を、自己テストでの検査に変更。
- Traceability・Components 表に 1.8 を反映。Testing Strategy に 1.8・1.3/1.4・6.x・2.x の確認項目を追加。「新章 2 つ」を「新章 10 個」に訂正。
- `pasta-lua-coding/SKILL.md` の誤記は `function(req)`（L84）のみ。`RES.ok_with` は `lua/patterns.md:124` にだけある。
- 末尾の「設計ディスカッションの決定」を取り消し線なしの決定記録（表）に置換。本書の Decision 2 件（命名・口調検出）も決定後の内容に更新。

### 開発者判断が必要な前提（design.md「前提（要確認）」）

- A1: 口調マーカーと普通文体の衝突は言い換えで回避する（集合は変更しない）。
- A2: 旧公開 URL `lua/modules.html` のリダイレクトは設けない。
- A3: 「将来変更あり」表記は現行挙動の注記にのみ残す。
