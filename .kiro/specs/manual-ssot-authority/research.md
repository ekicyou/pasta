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
