# 設計書: manual-ssot-authority

## Overview

**Purpose**: 本仕様は、pasta の利用者向け情報（Pasta DSL 文法・公開 Lua API・`pasta.toml`・起動シーケンス）の権威を mdBook マニュアル（`book/src/`）の 1 箇所へ集約し、スキル `references/` の利用者向け規範ファイルをマニュアルから機械生成する。旧権威（`doc/spec/`・`GRAMMAR.md` 本文・drift-check 機構）は撤去する。

**Users**: ゴースト作者は公開マニュアルだけで規範情報へ到達できる。スキル利用者（AI エージェント／他リポジトリのゴースト開発者）は、マニュアルと同一内容の自己完結したスキルを持ち出せる。メンテナは「マニュアルを直して 1 コマンドで再生成する」だけで両者を同期でき、乖離は CI が検出する。

**Impact**: 文書の権威の向きを「doc/spec → book（drift-check で追従）／スキル手書き → book（起草元）」から「book → スキル（生成）」へ反転する。ランタイムの挙動・文法・API・設定解釈は一切変更しない（10.5）。

### Goals

- 利用者向け規範記述がマニュアルにのみ手書きで存在する（1.1–1.7, 3.1–3.4, 4.1–4.2, 10.2）。
- スキルの規範ファイルがマニュアルから決定論的に生成され、鮮度が CI と完了ゲートで保証される（5.1–5.9, 7.1–7.8）。
- 旧権威とその参照が現行文書から消え、リンク切れ検出は存続する（2.1–2.4, 8.1–8.5, 9.1–9.8）。
- 上記をすべて 1 ブランチ・1 PR で同時に統合する（10.1）。

### Non-Goals

- ランタイム内部設計の解説、`internal-modules` の権威移動（`pasta-runtime-internals-doc`）。
- 文法・API・設定の挙動変更、マニュアルのデザイン・シンタックスハイライト変更。
- `.kiro/specs/completed/` の書き換え、既に持ち出されたスキルのコピーの更新、`pasta-check` スキル。
- 生成機構の汎用化（任意スキル・任意章を設定ファイルで宣言する仕組み等）。対応表は本仕様の 14 章に限定した定数とする。

## Boundary Commitments

### This Spec Owns

- **マニュアル内容**: `book/src/grammar/*`・`book/src/lua/modules.md`・新章 `book/src/lua/shiori-events.md`・新章 `book/src/reference/pasta-toml.md`・`book/src/reference/startup.md`・`book/src/introduction.md`・`book/src/reference/external-links.md`・`book/src/SUMMARY.md` の規範的記述（移し替えと拡充）。
- **生成機構**: マニュアル章 → スキル `references/` の対応表・抽出規則・リンク書き換え規則・生成物ヘッダ・鮮度判定（`book/tools/gen-skill-refs.mjs`）。
- **リンク検証**: book 内リンク切れ検出とスキル自己完結検査（`book/tools/link-check.mjs`）。
- **スキル構成**: `pasta-ghost-authoring`・`pasta-lua-coding` の `references/` のファイル構成、`SKILL.md` の生成／手書き区分表示、手書きファイルからの規範的事実の除去。
- **撤去**: `doc/spec/`、`GRAMMAR.md` 本文、`book/manual-sources.toml`、`book/tools/drift-check.mjs`・`drift-check-test.mjs`・`verify-drift-gate.mjs`。
- **CI・ゲート**: `.github/workflows/manual.yml` の起動条件と検証ステップ、`workflow.md` DoD の Manual Sync Gate、`kiro-complete` スキルの同ゲート記述。
- **参照修正**: steering・`README.md`・`SOUL.md`・`OPTIMIZATION.md`・`book/AUTHORING.md`・`book/CONTENT-REVIEW.md`・進行中 spec・ソースコメント。
- **将来仕様の行き先**: `doc/spec/` ch08・ch12 の未実装項目の brief 起票とロードマップ記載。
- **既定値整合テストの読み先**: `crates/pasta_lua/tests/loader/config_defaults_test.rs` の参照パス。

### Out of Boundary

- ランタイム（`crates/**/src`）の挙動。本仕様が変更してよい crates 配下は、テストの読み先パスとコメントのみ。
- `internal-modules.md`・`coding-conventions.md`・`testing-lint.md` の内容（区分表示と外部参照除去以外は触らない）。
- マニュアルのデバッグ章・入門章の内容（リンク先として参照されるのみ）。
- `pasta-check` スキル、`.kiro/specs/completed/`、他リポジトリへ持ち出し済みのスキル。
- 新しい公開パイプライン（Pages 公開・bigram 索引・構文ハイライトの各ステップは不変）。

### Allowed Dependencies

- Node.js 20（CI と同一）の標準モジュール（`node:fs`・`node:path`・`node:url`）のみ。新規 npm 依存は追加しない（5.8）。
- 既存 `book/tools/` の関数の import（`link-check.mjs` の `extractLinks` 等）。依存方向は「`gen-skill-refs.mjs` → `link-check.mjs`」「`verify-content.mjs` → `gen-skill-refs.mjs`（対応表と `VOICE_MARKERS` の読み取りのみ）」に限る。逆方向の import は禁止。
- mdBook 0.5.3 の既存ビルド・公開パイプライン。
- 生成物ヘッダが案内する公開 URL `https://ekicyou.github.io/pasta/`（`book.toml` の `site-url` と整合）。

### Revalidation Triggers

- **対応表の変更**（生成対象章の追加・削除・出力ファイル名変更）: 両 `SKILL.md` のリンクと区分表、`verify-content.mjs` の SUMMARY 到達検査、`pasta-runtime-internals-doc` の生成方式の再確認が必要。
- **章構造規約の変更**（導入／締めの区切りを `---` 以外にする等）: 生成器の抽出規則、`book/AUTHORING.md`、全章の再生成。
- **公開 URL の変更**（`book.toml` の `site-url`・Pages の公開先）: 生成物ヘッダと章外リンク書き換えの基底 URL。
- **`pasta.toml` リファレンス章の表形式変更**: `config_defaults_test.rs` の行一致規則。
- **スキルのディレクトリ名・配置変更**: 生成器の出力先、`manual.yml` の `paths`、Manual Sync Gate の発火条件。
- 下流 `pasta-runtime-internals-doc` が `internal-modules` を生成化するときは、本仕様の対応表へ行を追加する形で再利用する（生成器の構造変更は不要）。

## Architecture

### Existing Architecture Analysis

- マニュアルの全章（文法・Lua・リファレンス）は「H1 → キャラ口調の導入 → `---` → 規範的本文 → `---` → キャラ口調の締め」という構造を持ち、最初と最後の `---` 行が導入・締めの境界になっている（`lua/modules.md` は本文内にも `---` を 7 本持つが、最初と最後の位置規約は保たれている）。文法章は締めの後に `> **権威的仕様**: … doc/spec/…` 引用を持つ。
- `book/tools/` は依存なしの `.mjs` スクリプト＋同居 `*-test.mjs` の自己テストという規約を持ち、CI は `*-test.mjs` を `find` で一括実行する。改行の LF 正規化（`/\r\n?/g`）は `drift-check.mjs`・`tutorial-check.mjs` に既存実装がある。
- `drift-check.mjs` はドリフト検出（doc/spec ハッシュ）とリンク切れ検出（book 内相対 `.md`・自リポ GitHub URL）を同居させている。
- スキルを読むコードは `config_defaults_test.rs` の 1 箇所のみ（`pasta-toml.md` の「キー名と `` `値` `` が同一行にある」ことを行単位で検査）。
- `manual.yml` は `book/**` と自身の変更時のみ起動し、build ジョブが失敗すると deploy ジョブは走らない（`needs: build`）。

### Architecture Pattern & Boundary Map

採用パターン: **単一ソース＋静的生成（Generate & Verify）**。マニュアル章を唯一の入力とし、生成器が純関数として出力を決め、同じ関数を「書き出し」と「照合（`--check`）」の 2 モードで使う。照合はローカル・CI・完了ゲートで同一コマンドを使う。

```mermaid
graph LR
    Maintainer --> BookSrc
    BookSrc --> GenSkillRefs
    GenSkillRefs --> GhostAuthoringRefs
    GenSkillRefs --> LuaCodingRefs
    BookSrc --> LinkCheck
    GhostAuthoringRefs --> LinkCheck
    LuaCodingRefs --> LinkCheck
    BookSrc --> VerifyContent
    BookSrc --> ConfigDefaultsTest
    BookSrc --> MdBookBuild
    ManualCI --> GenSkillRefs
    ManualCI --> LinkCheck
    ManualCI --> VerifyContent
    ManualCI --> MdBookBuild
    MdBookBuild --> PagesDeploy
    CompletionGate --> GenSkillRefs
    CompletionGate --> LinkCheck
```

**Architecture Integration**:
- 選定理由: 生成物をリポジトリにコミットする方式は、スキルを「ディレクトリごとコピーするだけ」で持ち出せる要件（5.3）と両立する唯一の形である。照合モードを同じ生成関数で行うため、生成と検査の規則が二重化しない。
- 境界: 生成器は「章 → 生成ファイル」の写像だけを所有し、手書きファイル・`SKILL.md` には書き込まない。自己完結性の検査はリンク検証器が所有し、生成器は所有しない。
- 既存パターンの維持: 依存なし `.mjs`＋同居テスト、LF 正規化、`REPO_ROOT` 自己解決、CI の自己テスト一括実行、mdBook の章文体構造。
- 新規コンポーネントの理由: `gen-skill-refs.mjs` は生成と鮮度判定（5.x, 7.x）に必須。`link-check.mjs` は `drift-check.mjs` からリンク検証部だけを残した縮小版であり、新規ではなく改名・縮小である（8.4）。
- Steering 適合: 静的サイト維持・追加エコシステム依存なし（tech.md）、LF 正規化、`book/tools/` 配置（structure.md）。

### Dependency Direction

`book/src`（入力データ） → `link-check.mjs`（純関数ライブラリ＋CLI） → `gen-skill-refs.mjs`（対応表＋生成＋照合） → `verify-content.mjs`（対応表を読むだけ） → CI／完了ゲート（コマンド実行のみ）。各ファイルは左側のみを import する。スキル側ファイルはどのツールからも import されない（読み書きされるデータである）。

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| CLI / ツール | Node.js 20（素の ESM `.mjs`） | 生成・照合・リンク検証・コンテンツ検証 | 新規 npm 依存なし。`book/package.json` は変更しない |
| ドキュメント | mdBook 0.5.3 | マニュアルのビルド・公開 | 変更なし |
| CI | GitHub Actions `manual.yml`（ubuntu） | 鮮度チェック・リンク検証の実行、公開の阻止 | 起動 `paths` に 2 スキルを追加 |
| テスト | Rust `cargo test`（`build.yml`・Windows） | `pasta.toml` 既定値整合 | 読み先パスのみ変更 |

## File Structure Plan

### Directory Structure

```
book/
├── AUTHORING.md                  # [改] 権威＝マニュアル、生成対象章の執筆規約（本文に口調を入れない）、再生成手順
├── CONTENT-REVIEW.md             # [改] 冒頭に「doc/spec 廃止以前の歴史的レビュー記録」注記（OPEN QUESTION 6）
├── manual-sources.toml           # [削除]
├── src/
│   ├── SUMMARY.md                # [改] lua/shiori-events.md・reference/pasta-toml.md を目次へ追加
│   ├── introduction.md           # [改] 「doc/spec が権威」記述を「本マニュアルが権威」へ
│   ├── grammar/*.md (10 章)      # [改] doc/spec・GRAMMAR.md・スキル手書きの規範的内容を吸収。章末「権威的仕様」引用を削除
│   ├── lua/modules.md            # [改] 旧スキル runtime-api 相当へ拡充（→ スキル modules.md）
│   ├── lua/shiori-events.md      # [新] SHIORI イベントとハンドラ（REG/RES/イベント一覧/フォールバック/仮想ディスパッチャ）
│   ├── lua/patterns.md           # [改] 生成対象外。REG ハンドラ署名・RES の誤記を実装に合わせて訂正し、詳細は shiori-events へ誘導
│   ├── reference/pasta-toml.md   # [新] pasta.toml リファレンス（分類表・テンプレート・予約注記・各セクション詳細）
│   ├── reference/startup.md      # [改] 必要に応じ本文の口調コラムを導入/締めへ移すのみ
│   └── reference/external-links.md # [改] doc/spec へのリンク群を削除
└── tools/
    ├── gen-skill-refs.mjs        # [新] 対応表・抽出・リンク書き換え・ヘッダ付与・書き出し／--check 照合
    ├── gen-skill-refs-test.mjs   # [新] 生成器の自己テスト（CI の *-test.mjs 一括実行で走る）
    ├── link-check.mjs            # [改名] drift-check.mjs からリンク検証部のみ残し、スキル自己完結検査を追加
    ├── link-check-test.mjs       # [改名] drift-check-test.mjs からリンク検証ケースを残し、スキル検査ケースを追加
    ├── drift-check.mjs / drift-check-test.mjs / verify-drift-gate.mjs  # [削除]（前 2 者は改名で消滅）
    ├── verify-content.mjs        # [改] A 節の doc/spec リンク・manual-sources 検査を撤去、doc/spec 参照禁止・目次到達検査を追加
    ├── verify-scripts-test.mjs   # [改] verify-drift-gate ブロックを削除
    └── tutorial-check*.mjs       # [改] drift-check へのコメント言及のみ修正

.claude/skills/
├── pasta-ghost-authoring/
│   ├── SKILL.md                  # [改] references 区分表・生成ファイル優先の明示・早見表の非規範注記・リンク更新
│   └── references/
│       ├── grammar-index.md      # [生成・改名] ← grammar/index.md（旧 grammar-model.md は削除）
│       ├── markers.md            # [生成・新] ← grammar/markers.md
│       ├── block-structure.md    # [生成・新] ← grammar/block-structure.md
│       ├── call-jump.md          # [生成・改名] ← grammar/call-jump.md（旧 call-spec.md は削除）
│       ├── literals.md           # [生成・新] ← grammar/literals.md
│       ├── action-line.md        # [生成] ← grammar/action-line.md
│       ├── sakura-script.md      # [生成] ← grammar/sakura-script.md
│       ├── variables.md          # [生成] ← grammar/variables.md
│       ├── words.md              # [生成] ← grammar/words.md
│       ├── actor-dictionary.md   # [生成] ← grammar/actor-dictionary.md
│       ├── pasta-toml.md         # [生成] ← reference/pasta-toml.md
│       └── authoring-patterns.md # [手書き・改] 規範的事実を削り、生成ファイルへの参照に置換
└── pasta-lua-coding/
    ├── SKILL.md                  # [改] 区分表・internal-modules 暫定注記・book/ への相対リンク除去
    └── references/
        ├── modules.md            # [生成・改名] ← lua/modules.md（旧 runtime-api.md は削除）
        ├── shiori-events.md      # [生成・改名] ← lua/shiori-events.md（旧 shiori-handlers.md は削除）
        ├── startup.md            # [生成・新] ← reference/startup.md
        ├── internal-modules.md   # [手書き・暫定] 外部参照があれば除去のみ
        ├── coding-conventions.md # [手書き] 外部参照があれば除去のみ
        └── testing-lint.md       # [手書き] 外部参照があれば除去のみ

.kiro/specs/
├── manual-ssot-authority/absorption-ledger.md # [新] 吸収台帳（吸収元見出し → 収録先 or 除外理由）
├── scene-attribute-semantics/brief.md         # [新] 属性セマンティクス・ファイルレベル属性・属性フィルター
└── dynamic-word-reference/brief.md            # [新] 動的単語参照 ＠＄
```

### Modified Files

- `.github/workflows/manual.yml` — `paths` に `.claude/skills/pasta-ghost-authoring/**`・`.claude/skills/pasta-lua-coding/**` を追加。Setup Node 直後に「Skill references freshness」（`node book/tools/gen-skill-refs.mjs --check`）を追加。「Drift / broken-link check」を「Link check」（`node book/tools/link-check.mjs`）へ置換。「Verify drift gate」ステップを削除。ヘッダコメントのパイプライン説明を更新。
- `crates/pasta_lua/tests/loader/config_defaults_test.rs` — `config_reference_doc_matches_ssot` の読み先を `book/src/reference/pasta-toml.md` へ変更し、doc コメントの「pasta-toml.md（スキル）」記述を更新。照合規則（キー名と `` `値` `` の同一行）と失敗メッセージ（キー名を含む）は維持。
- `crates/pasta_lua/tests/runtime/syntax_test.rs` — `GRAMMAR.md` を指すコメントをマニュアル章へ付け替え。
- `crates/pasta_lua/README.md` — スキルの `pasta-toml.md` へのリンク（L144 付近）をマニュアル章 `book/src/reference/pasta-toml.md` へ付け替え（権威の向きの統一・9.3）。`startup.md` へのリンクは維持。
- `GRAMMAR.md` — 本文を全削除し、移設の告知と公開マニュアル URL（`https://ekicyou.github.io/pasta/grammar/index.html`）のみを置く（2.2, 2.3）。
- `doc/spec/` — ディレクトリごと削除（2.1）。
- `.kiro/steering/grammar.md` — 非規範要約へ縮小（後述「Steering: grammar.md」）。
- `.kiro/steering/workflow.md` — DoD「6. Manual Sync Gate」を再定義、最終タスクのドキュメント整合チェックリスト・更新チェックリスト・保守責任・保守ルールから `doc/spec/`・`GRAMMAR.md` を除き「マニュアル章＋スキル再生成」を追加、`.agents/skills` を `.claude/skills` へ修正。
- `.kiro/steering/tech.md`・`structure.md`・`product.md`・`roadmap.md` — drift-check／doc/spec／GRAMMAR.md 記述を生成方式とマニュアル権威へ置換。`roadmap.md` に将来仕様のキー情報を追記。
- `.claude/skills/kiro-complete/SKILL.md` — ステップ 4 と完了チェックリストの Manual Sync Gate 記述を新ゲートへ置換（判定本体は workflow.md を正とする構造は維持）。
- `README.md`・`SOUL.md`・`OPTIMIZATION.md` — doc/spec・GRAMMAR.md の行を削除またはマニュアルへ付け替え。`SOUL.md` の衝突ルールを「マニュアル（`book/src/`）を優先し、README・steering・スキル手書きを修正する」へ変更。`.agents/skills` を `.claude/skills` へ修正。
- `.kiro/specs/review-improvement-loop/{brief,matrix,tasks,design,requirements,research}.md` — 今後の実行指示として読まれる箇所のみ参照修正（OPEN QUESTION 7）。

## System Flows

### 生成と照合

```mermaid
sequenceDiagram
    participant M as Maintainer
    participant G as GenSkillRefs
    participant B as BookSrc
    participant S as SkillRefs
    M->>G: run write mode
    loop each map entry
        G->>B: read chapter and normalize LF
        G->>G: split by first and last rule line
        G->>G: validate body and rewrite links
        G->>S: write header plus title plus body
    end
    M->>G: run check mode
    G->>B: render all entries in memory
    G->>S: read files and normalize LF
    G-->>M: stale or orphan list with fix command and exit 1
```

- 構造違反（区切り不足・本文への口調混入・解決不能リンク）は書き出しモードでも照合モードでも同じ例外で失敗し、ファイルを 1 つも書き換えない（全エントリをメモリ上で生成し終えてから書き出す）。

### CI と公開

```mermaid
graph TB
    Trigger[push or PR touching book or two skills] --> Freshness
    Freshness -->|fail| Stop
    Freshness --> Build[mdbook build and postprocess]
    Build --> LinkCheck
    LinkCheck -->|fail| Stop
    LinkCheck --> OtherVerify[tutorial verify static search content selftests]
    OtherVerify -->|fail| Stop
    OtherVerify --> Upload[upload Pages artifact on main]
    Upload --> Deploy
    Stop[job fails and deploy is skipped]
```

- deploy ジョブは `needs: build` のため、鮮度チェック失敗時は公開されない（7.6）。

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1.1 | doc/spec 実装済み章の規範を文法章へ欠落なく収録 | ContentMigration, AbsorptionLedger | 吸収台帳 | — |
| 1.2 | GRAMMAR.md・スキル手書き固有の挙動事実を収録 | ContentMigration, AbsorptionLedger | 吸収先対応表 | — |
| 1.3 | 現行挙動のみ収録・属性は現行挙動のみ | ContentMigration, FutureSpecRouting | ch08/ch12 仕分け表 | — |
| 1.4 | 将来仕様を brief／ロードマップへ | FutureSpecRouting | 仕分け表 | — |
| 1.5 | マニュアル外を権威として案内しない | ContentMigration, VerifyContent | `A-nospec` 検査 | — |
| 1.6 | 実装と矛盾しない | ContentMigration, AbsorptionLedger | 台帳の実装照合列 | — |
| 1.7 | 章構成規約の維持 | ContentMigration, VerifyContent, GenSkillRefs | 章構造契約 | — |
| 2.1 | doc/spec 削除（スタブなし） | Retirement | — | — |
| 2.2 | GRAMMAR.md は案内のみ | Retirement | — | — |
| 2.3 | 行き先 URL を示す | Retirement | — | — |
| 2.4 | doc/spec・GRAMMAR.md 依存の検証なし | Retirement, VerifyContent, LinkCheck | — | — |
| 3.1 | runtime-api 相当を減らさず収録 | ContentMigration, AbsorptionLedger | `lua/modules.md` | — |
| 3.2 | shiori-handlers 相当を減らさず収録 | ContentMigration, AbsorptionLedger | `lua/shiori-events.md` | — |
| 3.3 | 食い違い時は実装に一致 | ContentMigration, AbsorptionLedger | 台帳の実装照合列 | — |
| 3.4 | internal-modules は収録しない | SkillLayout | 区分表 | — |
| 4.1 | pasta.toml 章で全内容を収録 | ContentMigration | `reference/pasta-toml.md` | — |
| 4.2 | 目次から到達可能 | ContentMigration, VerifyContent | `A-summary` 検査 | — |
| 4.3 | 既定値整合テストがマニュアルを検証 | ConfigDefaultsTest | 読み先パス | — |
| 4.4 | 食い違い時に失敗しキー報告 | ConfigDefaultsTest | 既存失敗メッセージ | — |
| 4.5 | スキルを読み先にしない | ConfigDefaultsTest | 読み先パス | — |
| 5.1 | 4 系統のみ生成 | GenSkillRefs | `GENERATION_MAP` | 生成と照合 |
| 5.2 | 導入・締めを含めない | GenSkillRefs | `extractBody` | 生成と照合 |
| 5.3 | スキル外参照を含まない | GenSkillRefs, LinkCheck | `rewriteLinks`, `checkSkillSelfContained` | 生成と照合 |
| 5.4 | 章間参照を解決可能にする | GenSkillRefs | `rewriteLinks` | 生成と照合 |
| 5.5 | 環境非依存の同一出力 | GenSkillRefs | `renderEntry` | 生成と照合 |
| 5.6 | 生成物ヘッダ | GenSkillRefs | `renderHeader` | 生成と照合 |
| 5.7 | 1 操作で全再生成 | GenSkillRefs | CLI 書き出しモード | 生成と照合 |
| 5.8 | 既存ツールチェーン内 | GenSkillRefs | Node 標準のみ | — |
| 5.9 | 章欠落・構造違反で失敗し報告 | GenSkillRefs | `GenError` | 生成と照合 |
| 6.1 | ファイル単位の区分明示 | SkillLayout | 区分表 | — |
| 6.2 | 手書きは作例・手順・規約のみ | SkillLayout, AbsorptionLedger | 手書きファイル規約 | — |
| 6.3 | internal-modules 暫定明示 | SkillLayout | 区分表 | — |
| 6.4 | SKILL.md リンクが実在ファイルのみ | SkillLayout, LinkCheck | `checkSkillSelfContained` | CI と公開 |
| 6.5 | スキル全体が自己完結 | SkillLayout, LinkCheck | `checkSkillSelfContained` | CI と公開 |
| 6.6 | 生成ファイルを正とする明示 | SkillLayout | 区分表の前文 | — |
| 6.7 | 早見表は非規範の要約 | SkillLayout | 早見表注記 | — |
| 7.1 | 未再生成で失敗 | GenSkillRefs, ManualCI | `--check` | CI と公開 |
| 7.2 | 生成物手編集で失敗 | GenSkillRefs, ManualCI | `--check` | CI と公開 |
| 7.3 | 不一致ファイルと解消手順を報告 | GenSkillRefs | `CheckReport` | 生成と照合 |
| 7.4 | 改行差を不一致にしない | GenSkillRefs | LF 正規化 | 生成と照合 |
| 7.5 | 3 種の変更で CI 実行 | ManualCI | `paths` | CI と公開 |
| 7.6 | 失敗中は公開しない | ManualCI | `needs: build` | CI と公開 |
| 7.7 | ローカル再現 | GenSkillRefs | 同一 CLI | 生成と照合 |
| 7.8 | 完了ゲートに含める | CompletionGate | Manual Sync Gate | — |
| 8.1 | ハッシュ表・ドリフト検出の撤去 | Retirement, LinkCheck | — | — |
| 8.2 | 公開がドリフト検出に非依存 | ManualCI | ステップ構成 | CI と公開 |
| 8.3 | 権威リンク・ハッシュ登録を要求しない | VerifyContent | A 節 | — |
| 8.4 | リンク切れ検出の存続 | LinkCheck | `detectBrokenLinks` | CI と公開 |
| 8.5 | 完了ゲートがドリフト検出を記述しない | CompletionGate | — | — |
| 9.1 | doc/spec を現存として参照しない | ReferenceRepair | 参照修正一覧 | — |
| 9.2 | GRAMMAR.md を参照先として案内しない | ReferenceRepair | 参照修正一覧 | — |
| 9.3 | 権威＝マニュアル・衝突時マニュアル優先 | ReferenceRepair | SOUL.md 衝突ルール | — |
| 9.4 | AUTHORING.md の権威反転 | ReferenceRepair | AUTHORING.md 第 4 節 | — |
| 9.5 | 保守手順にマニュアル＋再生成 | ReferenceRepair | workflow.md チェックリスト | — |
| 9.6 | completed を書き換えない | ReferenceRepair | 対象除外 | — |
| 9.7 | マニュアル内の doc/spec リンク除去 | ContentMigration, VerifyContent | `A-nospec` 検査 | — |
| 9.8 | steering/grammar.md の縮小 | ReferenceRepair | 残す節の一覧 | — |
| 10.1 | 一括統合 | Migration Strategy | 単一 PR | — |
| 10.2 | 手書きの写しを持たない | Retirement, SkillLayout | — | — |
| 10.3 | 既存テスト全成功 | Testing Strategy | — | — |
| 10.4 | 静的サイトのビルド・公開 | ManualCI | — | CI と公開 |
| 10.5 | ランタイム挙動不変 | Boundary Commitments | crates 変更はテストパスとコメントのみ | — |
| 10.6 | 収録不能内容の扱いを決定 | AbsorptionLedger | 除外理由列 | — |

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies (P0/P1) | Contracts |
|-----------|--------------|--------|--------------|--------------------------|-----------|
| GenSkillRefs | ツール | 章→スキル生成と鮮度照合 | 5.1–5.9, 7.1–7.4, 7.7 | link-check `extractLinks` (P1) | Service, Batch |
| LinkCheck | ツール | book 内リンク切れ検出＋スキル自己完結検査 | 5.3, 6.4, 6.5, 8.4 | — | Service, Batch |
| VerifyContent | ツール | コンテンツ受入検査の更新 | 1.5, 1.7, 4.2, 8.3, 9.7 | GENERATION_MAP (P1) | Batch |
| ManualCI | CI | 鮮度・リンク検証の実行と公開阻止 | 7.5, 7.6, 8.2, 10.4 | GenSkillRefs (P0), LinkCheck (P0) | Batch |
| CompletionGate | プロセス文書 | 完了ゲートの再定義 | 7.8, 8.5 | GenSkillRefs (P0), LinkCheck (P0) | — |
| ConfigDefaultsTest | テスト | pasta.toml 既定値整合の読み先付け替え | 4.3–4.5 | reference/pasta-toml.md (P0) | — |
| ContentMigration | 文書 | 規範内容のマニュアルへの移し替え | 1.1–1.3, 1.5–1.7, 3.1–3.3, 4.1, 4.2, 9.7 | AbsorptionLedger (P0) | — |
| AbsorptionLedger | 文書 | 吸収元見出しの収録先・除外理由の台帳 | 1.1, 1.2, 1.6, 3.1–3.3, 6.2, 10.6 | — | State |
| FutureSpecRouting | 文書 | ch08/ch12 未実装項目の brief・ロードマップ化 | 1.3, 1.4 | — | — |
| SkillLayout | スキル | 区分表示と手書きファイル整理 | 3.4, 6.1–6.7, 10.2 | GenSkillRefs (P0) | — |
| Retirement | 文書／ツール | 旧権威の撤去 | 2.1–2.4, 8.1, 10.2 | — | — |
| ReferenceRepair | 文書 | 現行文書の参照修正 | 9.1–9.6, 9.8 | — | — |

### ツール層

#### GenSkillRefs（`book/tools/gen-skill-refs.mjs`）

| Field | Detail |
|-------|--------|
| Intent | マニュアル章から規範本文を抽出してスキル `references/` を生成し、既存ファイルとの一致を照合する |
| Requirements | 5.1–5.9, 7.1–7.4, 7.7 |

**Responsibilities & Constraints**
- 対応表 `GENERATION_MAP`（モジュール内定数・順序固定）だけを入力の定義とする。設定ファイルは設けない。
- 生成ファイルのみを書き込む。手書きファイル・`SKILL.md` は読みも書きもしない（孤立生成物の検出時のみ `references/*.md` の先頭行を読む）。
- 全エントリをメモリ上で生成し終えてから書き出す（途中失敗で一部だけ更新された状態を作らない）。
- 出力は LF・末尾改行 1 つ・UTF-8（BOM なし）。時刻・環境値を出力に含めない（5.5）。

**Dependencies**
- Outbound: `link-check.mjs` の `extractLinks` 相当のリンク抽出規則（P1。抽出規則の二重化を避けるため import して再利用する）
- External: Node.js 標準 `node:fs`・`node:path`・`node:url`（P0）

**Contracts**: Service [x] / Batch [x]

##### Service Interface

```typescript
type SkillName = 'pasta-ghost-authoring' | 'pasta-lua-coding';

interface MapEntry {
  readonly chapter: string;   // book/src からの相対パス（例: 'grammar/markers.md'）
  readonly skill: SkillName;
}

// references/ 内の出力ファイル名は章パスから導出する（対応表に持たない）。
// 章のファイル名そのまま。ただし index.md は `{親ディレクトリ}-index.md`（例: grammar/index.md → grammar-index.md）。
function outName(chapter: string): string;

type GenError =
  | { kind: 'missing-chapter'; chapter: string }
  | { kind: 'bad-structure'; chapter: string; detail: string }      // 区切り行 2 本未満・H1 なし
  | { kind: 'voice-in-body'; chapter: string; markers: string[] }   // 本文散文に口調マーカー
  | { kind: 'unresolvable-link'; chapter: string; target: string }; // 章外の相対非 .md リンク等

interface CheckReport {
  readonly stale: string[];    // 期待内容と不一致・欠落の生成ファイル（リポジトリ相対）
  readonly orphans: string[];  // 生成ヘッダを持つが対応表に無いファイル
  readonly fixCommand: 'node book/tools/gen-skill-refs.mjs';
}

declare const GENERATION_MAP: readonly MapEntry[];
declare const MANUAL_BASE_URL: 'https://ekicyou.github.io/pasta/';

function extractBody(chapterText: string, chapter: string): { title: string; body: string }; // throws GenError
function rewriteLinks(body: string, entry: MapEntry): string;                                // throws GenError
function renderEntry(entry: MapEntry, repoRoot: string): string;                             // throws GenError
function generateAll(repoRoot: string): Map<string, string>;   // 出力相対パス → 内容
function checkAll(repoRoot: string): CheckReport;
```

- **対応表（確定値）**:

| chapter | skill | out（`outName` の結果） |
|---------|-------|-----|
| grammar/index.md | pasta-ghost-authoring | grammar-index.md |
| grammar/markers.md | pasta-ghost-authoring | markers.md |
| grammar/block-structure.md | pasta-ghost-authoring | block-structure.md |
| grammar/call-jump.md | pasta-ghost-authoring | call-jump.md |
| grammar/literals.md | pasta-ghost-authoring | literals.md |
| grammar/action-line.md | pasta-ghost-authoring | action-line.md |
| grammar/sakura-script.md | pasta-ghost-authoring | sakura-script.md |
| grammar/variables.md | pasta-ghost-authoring | variables.md |
| grammar/words.md | pasta-ghost-authoring | words.md |
| grammar/actor-dictionary.md | pasta-ghost-authoring | actor-dictionary.md |
| reference/pasta-toml.md | pasta-ghost-authoring | pasta-toml.md |
| lua/modules.md | pasta-lua-coding | modules.md |
| lua/shiori-events.md | pasta-lua-coding | shiori-events.md |
| reference/startup.md | pasta-lua-coding | startup.md |

  命名規則: 全生成ファイルを章名に揃える（設計ディスカッション #3。マニュアル章とスキルファイルの対応を名前だけで辿れるようにし、今後の整理負債を残さない）。出力名は `outName` で章パスから機械的に導出し、対応表に別名を持たせない。旧名ファイル（`grammar-model.md`・`call-spec.md`・`runtime-api.md`・`shiori-handlers.md`）は生成ヘッダを持たない手書きファイルのため孤立検出に掛からない。切替（Migration P4）で明示的に削除する。`(skill, outName(chapter))` は全体で一意でなければならず、`generateAll` は重複を `bad-structure` 相当の例外で拒否する。デバッグ・入門・`lua/basics`・`lua/patterns`・`lua/dsl-vs-lua`・`lua/index`・`reference/external-links` は生成しない（5.1）。

- **抽出規則（`extractBody`）**: 入力を LF 正規化し、コードフェンス（```` ``` ```` / `~~~`）外で「行全体が `---`」の行を区切り行とみなす。区切り行が 2 本未満、または先頭行が `# ` で始まる H1 でなければ `bad-structure`。H1 行をタイトルとし、最初の区切り行の次行から最後の区切り行の前行までを本文とする（本文内の `---` は保持）。本文の前後の空行は除去する。最後の区切り行以降（締め・旧「権威的仕様」引用）はすべて捨てる。本文の散文部（コードフェンス・表の行（`|` 始まり）・インラインコードを除去した残り）に口調マーカー（`verify-content.mjs` の広い集合 `VOICE_MARKERS`: `ですわ`・`ますの`・`おほほ`・`わたくし`・`くてよ` 等）のいずれかが現れたら `voice-in-body`。`VOICE_MARKERS` は `gen-skill-refs.mjs` へ移して export し、`verify-content.mjs` はそれを import する（依存方向を守り、集合を一本化する）。生成対象章の本文では口調を全面禁止し、コラム・励ましは導入か締めへ置く（設計ディスカッション #1 で確定）。作例の台詞に口調を含めたい場合はコードフェンス内に置く。
- **リンク書き換え規則（`rewriteLinks`）**: コードフェンス外のインラインリンク `[text](target)` のみを対象とする。
  1. `http(s)://`・`mailto:` 等の絶対 URL、および `#anchor` のみのリンク → そのまま。
  2. 相対 `.md` リンク（アンカー付き可）→ 章のディレクトリ基準で `book/src` 内パスへ解決し、`GENERATION_MAP` に同一スキル宛てのエントリがあれば `{out}{#anchor}`（同じ `references/` 内の兄弟ファイル）。それ以外（非生成章・別スキル宛て）は `{MANUAL_BASE_URL}{chapterPath の .md を .html に置換}{#anchor}`。
  3. 上記以外の相対リンク（画像・非 `.md`）および `book/src` 外へ出る相対リンク → `unresolvable-link`。
- **ヘッダ（`renderHeader`）**: 生成ファイルの 1〜2 行目に固定する。リポジトリ内パスは含めない（5.3）。

```markdown
<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->
<!-- このファイルは pasta 利用者マニュアル「{H1 タイトル}」（{MANUAL_BASE_URL}{chapter を .html にしたもの}）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->

# {H1 タイトル}

{本文}
```

  1 行目の固定文字列 `<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->` を生成物の機械判定（孤立検出）に使う。
- Preconditions: `GENERATION_MAP` の各 `chapter` が `book/src` に存在する。
- Postconditions: 書き出しモード成功後、`checkAll` は `stale`・`orphans` ともに空を返す。
- Invariants: 同一入力に対し `generateAll` の結果はバイト単位で同一（入力の改行コードに依存しない）。

##### Batch / Job Contract

- Trigger: メンテナの手動実行（`node book/tools/gen-skill-refs.mjs`）、CI（`--check`）、完了ゲート（`--check`）。
- Input / validation: `book/src` の対応章。検証失敗は `GenError` を `chapter` 付きで標準エラーへ出し exit 1（5.9）。
- Output / destination: 書き出しモード → 各スキルの `references/{out}`（内容が同一なら書き換えない）。照合モード → 不一致時に `STALE {path}`／`ORPHAN {path}` の一覧と「再生成: `node book/tools/gen-skill-refs.mjs` を実行してコミット」「孤立ファイルは削除するか対応表へ追加」を出して exit 1、一致時 exit 0（7.3）。照合時は既存ファイルも LF 正規化して比較する（7.4）。
- Idempotency & recovery: 書き出しは冪等。失敗時は何も書かないため再実行で回復する。

**Implementation Notes**
- Integration: CLI 判定は既存ツールと同じ `import.meta.url` と `process.argv[1]` の比較で行い、関数は export してテストから使う。
- Validation: `gen-skill-refs-test.mjs` が抽出・書き換え・ヘッダ・決定性・照合をサンドボックスで検証する（Testing Strategy 参照）。
- Risks: 章執筆者が本文に口調コラムを入れると生成が失敗する。これは意図した失敗であり、`AUTHORING.md` に規約として明記する。

#### LinkCheck（`book/tools/link-check.mjs`）

| Field | Detail |
|-------|--------|
| Intent | book 内のリンク切れと、スキルのディレクトリ外参照を検出する |
| Requirements | 5.3, 6.4, 6.5, 8.4 |

**Responsibilities & Constraints**
- `drift-check.mjs` を改名し、`detectBrokenLinks`・`extractLinks`・`githubUrlToRepoPath`・`isWithinRoot`・`listMarkdownFiles` を現行のまま残す。`parseManualSources`・`detectDrift`・`detectUnmapped`・`sha256File`・`UNMAPPED_EXCLUDE`・`DRIFT_STRICT` は削除する。
- スキル自己完結検査を追加する。対象は `.claude/skills/pasta-ghost-authoring/**/*.md` と `.claude/skills/pasta-lua-coding/**/*.md`（手書き・生成の両方）。

**Contracts**: Service [x] / Batch [x]

##### Service Interface

```typescript
interface BrokenLink {
  file: string;   // リポジトリ相対
  target: string;
  kind: 'internal-md' | 'github-repo-path' | 'skill-escape' | 'skill-missing' | 'skill-anchor' | 'skill-forbidden-ref';
  detail: string;
}

declare const CHECKED_SKILLS: readonly ['pasta-ghost-authoring', 'pasta-lua-coding'];
declare const FORBIDDEN_SKILL_TOKENS: readonly ['doc/spec', 'GRAMMAR.md', 'book/src'];

function detectBrokenLinks(repoRoot: string): BrokenLink[];           // 既存（book/src 対象）
function checkSkillSelfContained(repoRoot: string): BrokenLink[];     // 新規
function headingSlug(heading: string): string;                       // 新規（GitHub 方式）
function runLinkCheck(repoRoot: string): { broken: BrokenLink[]; failed: boolean };
```

- `checkSkillSelfContained` の規則: (a) 相対リンクはスキルディレクトリ内に解決され（`skill-escape`）、かつ実在すること（`skill-missing`）。(a') `*.md#anchor`（同一ファイル内の `#anchor` を含む）は、リンク先ファイルのコードフェンス外の見出し（`#`〜`######`）を `headingSlug` で変換した集合にアンカーが含まれること（`skill-anchor`）。`headingSlug` は GitHub 方式（小文字化、英数字・日本語・`-`・`_`・空白以外を除去、空白を `-` へ、同名見出しは `-1`・`-2` を付番）。対象は 2 スキル内のリンクに限り、book 内のアンカーは検証しない（設計ディスカッション #2）。(b) ファイル本文（HTML コメントを含む全文）に `FORBIDDEN_SKILL_TOKENS` のいずれかが含まれないこと（`skill-forbidden-ref`。旧 `<!-- source: doc/spec/... -->` の残存も捕捉する）。絶対 URL は許可する。
- Batch: `node book/tools/link-check.mjs`。違反があれば分類表示して exit 1、無ければ exit 0。

**Implementation Notes**
- Integration: `gen-skill-refs.mjs` はリンク抽出をここから import する（逆方向の import はしない）。
- Validation: `link-check-test.mjs` は旧 `drift-check-test.mjs` のリンク検証系ケース（相対 `.md`・GitHub URL・トラバーサル・決定性）を残し、スキル検査（脱出・欠落・禁止語・絶対 URL 許可）のケースを追加する。ドリフト・未マップ・TOML パーサのケースは削除する。

#### VerifyContent（`book/tools/verify-content.mjs` の改修）

- **Intent**: コンテンツ受入検査から旧権威前提を除き、新しい不変条件を加える。Requirements: 1.5, 1.7, 4.2, 8.3, 9.7。
- 削除: `A-link:*`（doc/spec 権威リンク必須）、`A-toml`・`A-toml-src`（manual-sources 整合）。
- 追加: `A-nospec`（`book/src/**/*.md` に文字列 `doc/spec` が無い）、`A-summary:*`（`GENERATION_MAP` の全 `chapter` が `book/src/SUMMARY.md` からリンクされている。対応表は `gen-skill-refs.mjs` から import）。
- 維持: `A-exist`・`A-body`・B〜G。D（ボイス）は既存のディレクトリ走査により新章 `lua/shiori-events.md`・`reference/pasta-toml.md` にも自動適用され、導入・締めの口調の存在を要求する（1.7）。
- ヘッダコメントの検証範囲説明を更新する。`verify-scripts-test.mjs` の件数閾値（>= 50）は削除後も満たす（現行約 86 件から 12 件減・数件増）。

### CI・プロセス層

#### ManualCI（`.github/workflows/manual.yml`）

- **Intent**: 鮮度・リンク検証を公開前ゲートとして実行する。Requirements: 7.5, 7.6, 8.2, 10.4。
- 起動 `paths`（push・pull_request 共通）: `book/**`・`.github/workflows/manual.yml`・`.claude/skills/pasta-ghost-authoring/**`・`.claude/skills/pasta-lua-coding/**`。生成器は `book/tools/` にあるため「生成機構のみの変更」も `book/**` で捕捉される（7.5）。
- ステップ: Setup Node の直後（`npm ci` より前）に `node book/tools/gen-skill-refs.mjs --check` を置き、依存インストールやビルドの前に速く失敗させる。旧「Drift / broken-link check」位置に `node book/tools/link-check.mjs`。「Verify drift gate」削除。自己テスト一括実行は不変（新テストは自動で拾われる）。
- 公開阻止: すべて `build` ジョブ内のステップであり、失敗時は `deploy`（`needs: build`）が走らない（7.6）。
- 不採用: `build.yml`（Windows・cargo、2 アーキの行列）への Node ステップ追加は、全 PR で走る利点に比べセットアップ重複のコストが大きい。スキル手編集は `paths` 追加で捕捉できるため不要と判断した。

#### CompletionGate（`workflow.md` DoD・`kiro-complete/SKILL.md`）

- **Intent**: 完了ゲートを鮮度チェック＋リンク検証へ置き換える。Requirements: 7.8, 8.5。
- ゲート名は「Manual Sync Gate（条件付き）」を維持し、意味を「マニュアルとスキル生成物の同期」に再定義する。
- 発火条件: 当該 spec の変更が `book/`・`.claude/skills/pasta-ghost-authoring/`・`.claude/skills/pasta-lua-coding/` のいずれかに触れる場合。
- 判定: `node book/tools/gen-skill-refs.mjs --check` と `node book/tools/link-check.mjs` がともに exit 0。非ゼロなら完了を中断する。
- 解消フロー: マニュアル章を正として修正 → `node book/tools/gen-skill-refs.mjs` で再生成 → コミット → ゲート再実行。
- スキップ: いずれにも触れない spec。`doc/spec`・ハッシュ・ドリフトの語は記述から除く（8.5）。
- 判定本体は `workflow.md` を正とし、`kiro-complete/SKILL.md` は発火とコマンドのみを持つ既存構造を維持する。

#### ConfigDefaultsTest（`config_defaults_test.rs`）

- **Intent**: `pasta.toml` 既定値の SSOT 照合の読み先をマニュアル章へ移す。Requirements: 4.3–4.5。
- 変更は `repo_root().join(".claude/skills/pasta-ghost-authoring/references/pasta-toml.md")` を `repo_root().join("book/src/reference/pasta-toml.md")` へ置き換えることと doc コメントの更新のみ。照合規則・失敗メッセージ（キー名と期待値を含む）は不変であり、4.4 を既存のまま満たす。
- 契約（マニュアル側の制約）: `reference/pasta-toml.md` は、検査対象の各キー（`talk_interval_min` 等）について「キー名と `` `既定値` `` が同じ行に並ぶ」表行を本文に持つ。`AUTHORING.md` にこの制約を明記する。

### 文書層

#### ContentMigration と AbsorptionLedger

- **Intent**: 規範内容をマニュアルへ欠落なく移し、その網羅を台帳で証明する。Requirements: 1.1–1.3, 1.5–1.7, 3.1–3.3, 4.1, 4.2, 6.2, 9.7, 10.6。
- **吸収台帳**（`.kiro/specs/manual-ssot-authority/absorption-ledger.md`・spec 成果物）: 各吸収元の見出し（`doc/spec` ch01–07・09–11 の節、`GRAMMAR.md` の節、スキル手書き文法 7 ファイル・`runtime-api.md`・`shiori-handlers.md`・`pasta-toml.md` の節、`authoring-patterns.md` の挙動事実）を 1 行ずつ列挙し、列「収録先（章#節）／既存収録済み／除外（理由）」「実装照合（照合したソース位置 or 不要）」を持つ。全行が埋まることを完了条件とする（10.6）。食い違いは実装を正として記述し、照合位置を記録する（1.6, 3.3）。
- **章ごとの収録先（確定）**:

| 吸収元 | 収録先 |
|--------|--------|
| doc/spec ch01 文法モデル | grammar/index.md |
| doc/spec ch02 マーカー（全角半角正規化・`＠＠`/`@@` 非正規化の例外を含む） | grammar/markers.md |
| doc/spec ch03 ブロック構造、ch08 の属性行構文・配置ルール（「構文は受理されるが処理に反映されない」現行挙動として）、コメント・空行・エンコーディング（ch12 §12.14・§12.17・§12.19）、Lua ブロック規則（ch02 §2.6） | grammar/block-structure.md |
| doc/spec ch04 Call 仕様（ターゲット形式・スコープ例・候補選択の現行挙動 §12.11） | grammar/call-jump.md |
| doc/spec ch05 リテラル、単語値・引数値・属性値の型解釈（ch12 §12.6・§12.10・§12.12） | grammar/literals.md |
| doc/spec ch06 アクション行、行継続（ch12 §12.13・`：` 必須） | grammar/action-line.md |
| doc/spec ch07 さくらスクリプト、括弧内エスケープ（ch12 §12.16）、主要タグ早見（スキル sakura-script） | grammar/sakura-script.md |
| doc/spec ch09 変数、Lua 予約語の制約（ch12 §12.15）、日時変数（authoring-patterns §6.4）、`＄＊` の保存先・DSL→Lua 対応表（スキル variables） | grammar/variables.md |
| doc/spec ch10 単語、シャッフル＆順次消費（authoring-patterns §6.6）、複数キー（§6.10） | grammar/words.md |
| doc/spec ch11 アクター辞書、3 段フォールバック・バルーン連携（スキル actor-dictionary） | grammar/actor-dictionary.md |
| キューコマンド行（doc/spec ch02 §2.11・`GRAMMAR.md`）、選択肢行（ch02 §2.12・`GRAMMAR.md`・authoring-patterns §6.11 の構文部） | grammar/block-structure.md（行の種類として節を新設） |
| チェイントーク `＞チェイントーク`/`＞yield`（ch12 §12.2 を置換・`GRAMMAR.md`・authoring-patterns §6.7）、`＞ゴースト終了（ms）`（§6.2）、`＞` に任意式を置く呼び出しと nil ガード（スキル call-spec） | grammar/call-jump.md（「特殊な呼び出し」節を新設） |
| スキル runtime-api.md 全節（6 モジュール＋mlua-stdlib） | lua/modules.md |
| スキル shiori-handlers.md 全節（REG・RES・イベント一覧・シーン関数フォールバック・仮想ディスパッチャ）、時報の 4 段フォールバック（authoring-patterns §6.4）、選択肢の `OnChoiceSelectEx` ルーティング（§6.11 の挙動部） | lua/shiori-events.md（新章） |
| スキル pasta-toml.md 全節、`pasta_patterns` の自動読み込み（authoring-patterns §6.8） | reference/pasta-toml.md（新章） |

- 規則: 現行実装で確認できない記述は規範として収録せず、FutureSpecRouting へ回す（1.3）。
- **既知の食い違い（実装が正）**: 設計時調査で、吸収元（doc/spec・book・スキル）の記述が実装と食い違う箇所が見つかった。移し替えは食い違いを訂正したうえで行う（1.6, 3.3）。これは文書の訂正であり、挙動の変更ではない（10.5）。台帳には照合したソース位置を記録する。

| 領域 | 文書の記述 | 実装（正） |
|------|-----------|-----------|
| 行継続 | インデントだけで継続 | 継続行は `：` 始まり（`grammar.pest` `continue_action_line`）。先行アクターなしはトランスパイルエラー |
| 継続内の空行 | 1 改行になる | 何も出力しない（糖衣構文は未実装） |
| ローカル／グローバル候補 | 統合して選択 | ローカル優先。ローカル候補が 1 つでもあればローカルのみ、無ければグローバル（統合しない） |
| Call 検索 | 4 段 | 5 段（シーン完全一致→ローカル前方一致→act メソッド→GLOBAL 完全一致→グローバル前方一致） |
| Call フィルター `＞シーン＆k＝v` | 予約・無視 | 構文として受理されない（将来仕様へ） |
| 引数 | 名前付きのみ・空白区切り・未実装 | 読点／カンマ区切り、位置引数可、Lua へ渡される |
| 文字列エスケープ | `\n` `\\` `\"` | エスケープ規則なし。囲みの多重化（`「「…」」` 等）で引用符を含める。空文字列可 |
| 単語値 | 空白区切り | 読点／カンマ区切り。空白は値に含まれ、末尾カンマ可 |
| 属性行の配置 | ローカルシーン直後にも可 | 行としてはグローバルシーン初期部のみ。ローカルシーンは宣言行への付記のみ。ファイルレベル属性は解析・統合されるが利用されない |
| Lua ブロックのフェンス | ```` ``` ```` / ```` ```lua ```` のみ | 3 個以上のバッククォート＋任意識別子 |
| キューコマンド | Lua 生成時は常にスキップ | `!select` は `act:choice_timeout` を生成。他はスキップ |
| 単語定義とさくらスクリプト | 単語定義行で使えない | 単語値にさくらスクリプトを書ける（アクター辞書が依存） |
| REG ハンドラ | `function(req)` | `function(act)`。リクエストは `act.req.*` |
| RES | `RES.ok_with(headers)` | 存在しない。`RES.ok(value, dic)` ほか `no_content`・`not_enough`・`advice`・`bad_request`・`err`・`warn`・`build`・`env` |
| シーン関数フォールバック | 204 を返す・pcall で保護 | `SCENE.co_exec` で実行し `RES.ok(script)`（200）。保護は `entry.lua` の xpcall |
| OnSecondChange・コールバック | ハンドラ上書き例・`CALLBACK.resume_pending` | 既定ハンドラが `CALLBACK.sweep` と仮想ディスパッチャを駆動（上書きで OnTalk/OnHour が止まる注意）。`resume_pending` は存在しない |
| さくらスクリプト変換のウェイト | `actor.talk` サブテーブル、既定 50/100/75、加算 | アクター表直下の `script_wait_normal/period/comma/strong/leader`、既定 50/1000/500/500/200、挿入値は `値 - 50`、連続句読点は最大値 |
| pasta.toml の既定値の出典 | `src/loader/config.rs` | `src/loader/config/mod.rs`・`sections.rs` |

  `book/src/lua/patterns.md`（生成対象外）と `pasta-lua-coding/SKILL.md` の早見表にも `function(req)`・`RES.ok_with` があるため同時に訂正する。実装照合で上表と異なる結果が出た場合は実装を正とし、台帳に記録する。
- 吸収元のどこにも書かれていない実装上の構文（例: 単独 `＊` 行による直前グローバルシーンの継続、`％a＝0、b` の番号付け、`＄０` のシーン引数参照、末尾 `#` コメント）は本仕様の収録義務の対象外とする（1.1・1.2 は吸収元の内容が対象）。台帳の付録に「未記載の実装事実」として列挙し、扱いは設計ディスカッションで決める（OPEN QUESTION 9）。各章の導入・締めは Claudia 口調、本文・表・コード・構文定義は普通文体（1.7・AUTHORING.md 準拠）。生成対象章は本文に口調コラムを置かない。
- 章末の `> **権威的仕様**` 引用、`grammar/index.md` の doc/spec 案内、`introduction.md` の権威記述、`external-links.md` の doc/spec リンク群を削除する（1.5, 9.7）。
- 新章は `SUMMARY.md` へ追加する: 「Lua API / コーディング」配下に `[SHIORI イベントとハンドラ](lua/shiori-events.md)`、「リファレンス」配下に `[pasta.toml リファレンス](reference/pasta-toml.md)`（4.2）。

#### FutureSpecRouting

- **Intent**: ch08・ch12 の未実装・将来項目を具体度で仕分ける。Requirements: 1.3, 1.4。
- 仕分け規則: (M) 現行実装で確認できる事実 → マニュアルへ収録（上表）。(B) 構文が既に定義され範囲が明確な未実装機能 → brief 起票＋ロードマップにキー情報。(R) 方針未定・DSL 範囲外の留保 → ロードマップにキー情報のみ。
- 仕分け結果（実装照合で (M) が成立しない項目は (R) へ倒す）:

| 項目 | 区分 | 行き先 |
|------|------|--------|
| ch08 属性のセマンティクス、§8.3＋§12.18 ファイルレベル属性の継承（解析・統合は実装済みで未利用）、§12.5 Call 属性フィルター（構文未受理） | B | `.kiro/specs/scene-attribute-semantics/brief.md`（現行挙動＝受理されるが処理に反映されない、はマニュアルへ） |
| §12.7 動的単語参照 `＠＄`（文法定義はあるがパーサ未実装） | B | `.kiro/specs/dynamic-word-reference/brief.md`（OPEN QUESTION 5） |
| §12.4 シーンのパラメータ | R | roadmap（対応予定なし・変数で代替） |
| ch11 §11.5 アクタースコープ内のコードブロック（予約） | R | roadmap |
| §12.8 Call の戻り値と変数代入 | R | roadmap（DSL 非定義・ランタイム設計の領域） |
| §12.9 ローカル変数のスコープ詳細（`ctx.local`／`ctx.global`） | 除外 | 実装で `var`／`save` に置換済み。現行挙動は variables.md。台帳に除外理由を記録 |
| §12.2 チェイントーク DSL 非採用 | M（置換） | `＞チェイントーク` による現行の実現方法を call-jump.md へ |
| §12.11 前方一致時の候補選択 | M | call-jump.md（ローカル優先・シャッフル消費） |
| §12.6・§12.10・§12.12 値の型解釈 | M | literals.md |
| §12.15 識別子と Lua 予約語 | M | variables.md |
| §12.13・§12.14・§12.16・§12.17・§12.19・§12.20 | M | 上の収録先表 |

- ロードマップ記載形式: `roadmap.md` の Phase 2（DSL 統合）配下に「将来仕様（doc/spec 廃止時の申し送り）」小節を設け、1 項目 1 行で「名称 — 要旨 — brief 参照 or（brief なし）」のみを書く（1.4）。

#### SkillLayout

- **Intent**: スキルの生成／手書き区分を明示し、手書きを作例・手順・規約に限定する。Requirements: 3.4, 6.1–6.7, 10.2。
- 各 `SKILL.md` に「references 一覧」表を置く。列: ファイル／区分（`生成（マニュアルから）` or `手書き（スキルが権威）`／`手書き（暫定）`）／生成元マニュアル章（公開 URL）／用途。表の前文に「生成ファイルは編集しない。生成ファイルと手書きファイルが同じ事実を扱う場合は生成ファイルの記述を正とする」と明記する（6.1, 6.6）。
- `pasta-lua-coding/SKILL.md`: `internal-modules.md` を「手書き（暫定）— 将来 `pasta-runtime-internals-doc` でマニュアル権威＋生成へ移行予定」と明記（3.4, 6.3）。`../../../book/src/reference/startup.md` へのリンクを `references/startup.md` へ置換（6.5）。
- `pasta-ghost-authoring/SKILL.md`: マーカー表等の早見表は手書きで保持し、直前に「参照先を選ぶための非規範の要約。食い違う場合は生成ファイルが正」と注記する（6.7）。旧 `grammar-model.md` への 11 件のリンクは、内容の移動先（`grammar-index.md`・（`markers.md`・`block-structure.md`・`literals.md` 等）へ張り替える。アンカーは生成ファイルの見出しに合わせる（6.4）。
- 手書き `authoring-patterns.md`: 時報変数・シャッフル消費・チェイントーク等の挙動説明を削り、作例と「詳細は `variables.md` 等を参照」の参照に置き換える（6.2）。残すのは作例・ファイル分割指針・自然言語→シーン変換指針等の手順のみ。
- 手書き Lua 3 ファイル: スキル外参照があれば除去する以外は変更しない。ただし旧名（`runtime-api.md`・`shiori-handlers.md`）へのリンクは新名（`modules.md`・`shiori-events.md`）へ張り替える。`testing-lint.md` の `runtime-api.md#set_scene_selector--set_word_selector` は `modules.md#set_scene_selector--set_word_selector` とし、`lua/modules.md` へ移すセレクタ節の見出しを同じアンカーになる形（`### set_scene_selector(...) / set_word_selector(...)`）で保つ。`internal-modules.md` の旧名リンク、他の手書き→生成リンクも、生成後のファイル名・見出しに合わせて張り替える（`skill-missing`・`skill-anchor` で機械確認）。
- 改名に伴う持ち出し先の注意: 両 `SKILL.md` の references 区分表の前文に「持ち出し先を更新するときは `references/` を丸ごと置き換える（旧名ファイルを残さない）」と明記する。
- `pasta-lua-coding/SKILL.md` の早見表（`pasta.*` 表）にある `function(req)` を実装どおり `function(act)` に訂正する（早見表は非規範だが誤りを残さない）。
- 自己完結は LinkCheck が機械検証する（6.4, 6.5）。

#### Retirement と ReferenceRepair

- 撤去（2.1–2.4, 8.1）: `doc/spec/`、`book/manual-sources.toml`、`book/tools/verify-drift-gate.mjs`、`drift-check*.mjs`（改名で消滅）、`GRAMMAR.md` 本文。
- `GRAMMAR.md` の最終形（全文）は「Pasta DSL 文法リファレンスは利用者マニュアルへ移りました」の告知 1 文と `https://ekicyou.github.io/pasta/grammar/index.html` へのリンクのみ（2.2, 2.3）。
- `book/AUTHORING.md`（9.4）: 第 4 節「流用／リンク方針」を「権威と生成」へ書き換える。内容: マニュアルが利用者向け情報の唯一の権威、スキル規範ファイルは生成物で編集元はマニュアル、生成対象章の一覧は `gen-skill-refs.mjs` の対応表が正、生成対象章の規約（導入と締めの間の本文に口調・コラムを置かない、最初と最後の `---` 以外の位置規約、章外への相対リンクは公開 URL に書き換わる、画像・非 `.md` 相対リンク禁止、`pasta-toml.md` の同一行表形式）、再生成コマンド。サンプル A の「権威的仕様」引用と第 5 節の doc/spec チェック項目を削除し、「生成対象章を変更したら再生成してコミット」を追加。流用元一覧表から doc/spec・GRAMMAR.md・スキル（起草元）を削除。
- `.kiro/steering/grammar.md`（9.8）: 残す節＝「このドキュメントの役割」（非規範の要約であり権威はマニュアル `book/src/grammar/`、食い違い時はマニュアルが正、と書き換え）・「マーカー一覧」（早見表）・「よくある間違いパターン」・「IR 出力（ScriptEvent）」（開発者向け）。削る節＝「権威的仕様書」・「ドメイン概念」全小節・「基本パターン」・「Lua ブロック」・「さくらスクリプト」（いずれも規範的事実でありマニュアルが持つ）。
- `workflow.md`（9.5）: 更新チェックリストの「DSL 文法変更」「Lua API 変更」「公開 API 変更」行と最終タスクのドキュメント整合チェックリストを「`book/src/` の該当章を更新し `node book/tools/gen-skill-refs.mjs` で再生成」へ。保守責任表から `GRAMMAR.md`・`doc/spec/` を削除し `book/src/`（利用者向け仕様・権威）を追加。保守ルール 1・3 をマニュアル起点へ。
- 参照修正の網羅確認: 完了前に `git grep -n -E "doc/spec|GRAMMAR\.md|drift-check|verify-drift-gate|manual-sources" -- ':!.kiro/specs/completed' ':!.kiro/specs/manual-ssot-authority'` を実行し、残る行がすべて「廃止済みであることの説明」「歴史的記録として除外したファイル」のいずれかであることを確認する（9.1, 9.2, 9.6）。

## Data Models

### Domain Model

- **章（Chapter）**: `book/src` 相対パスで識別。構造不変条件＝H1 1 行・区切り行 2 本以上・本文散文に口調マーカーなし（生成対象章のみ）。
- **対応表エントリ（MapEntry）**: `chapter` と `(skill, out)` は 1 対 1。`(skill, out)` は全体で一意。
- **生成ファイル（GeneratedRef）**: 1 行目の固定ヘッダで識別。内容は `renderEntry(entry)` の値と LF 正規化後に一致しなければならない。
- **手書きファイル（HandwrittenRef）**: 固定ヘッダを持たない。生成器の書き込み対象外。

## Error Handling

### Error Strategy

- 生成器は Fail Fast。最初の `GenError` で終了し、`chapter` と理由を 1 行で出す。部分書き出しはしない。
- 照合の不一致は全件を列挙してから exit 1（修正を 1 回で済ませるため）。
- リンク検証は全件を分類表示してから exit 1。
- 予期しない例外（読み取り不能等）は exit 2 とスタックを出す（既存 `drift-check.mjs` の慣行を踏襲）。

### Error Categories and Responses

| 状況 | 検出 | 出力 | 解消 |
|------|------|------|------|
| 対応章が無い | GenSkillRefs `missing-chapter` | 章パス | 章を作るか対応表を直す |
| 区切り不足・H1 なし | GenSkillRefs `bad-structure` | 章パス＋理由 | 章構造を規約に合わせる |
| 本文に口調語 | GenSkillRefs `voice-in-body` | 章パス＋語 | コラムを導入／締めへ移す |
| 解決不能リンク | GenSkillRefs `unresolvable-link` | 章パス＋リンク | `.md` 章リンクか絶対 URL に直す |
| 生成物が古い・手編集 | `--check` `STALE` | ファイル＋再生成コマンド | 再生成してコミット |
| 孤立生成物 | `--check` `ORPHAN` | ファイル | 削除するか対応表へ追加 |
| book 内リンク切れ | LinkCheck `internal-md`/`github-repo-path` | ファイル＋リンク | リンクを直す |
| スキル外参照 | LinkCheck `skill-*` | ファイル＋リンク／語 | スキル内参照か絶対 URL に直す |
| pasta.toml 既定値不一致 | `config_defaults_test` | キー名＋期待値 | マニュアル表を実装に合わせる |

## Testing Strategy

### Unit Tests（`gen-skill-refs-test.mjs`・`link-check-test.mjs`）

- `extractBody`: 導入・締め・締め後の引用が出力に含まれず、本文内の `---` は保持される（5.2）。区切り 1 本・H1 なしで `bad-structure`、本文散文に `わたくし` や文末 `ですわ` で `voice-in-body`、コードフェンス内・表の行・インラインコード内の `ですわ` や `---` は無視される（5.2, 5.9）。
- `rewriteLinks`: 同一スキル宛て → 兄弟ファイル名＋アンカー、非生成章・別スキル宛て → 公開 URL（`.html`＋アンカー）、絶対 URL・`#anchor` は不変、画像相対リンクで `unresolvable-link`、コードフェンス内は不変（5.3, 5.4）。
- `outName`: `grammar/markers.md` → `markers.md`、`grammar/index.md` → `grammar-index.md`、`(skill, outName)` 重複で例外。
- 決定性: 同一章の LF 版と CRLF 版から生成した結果がバイト一致し、ヘッダの 2 行が固定文字列である（5.5, 5.6）。
- `checkAll`（tmp サンドボックス）: 章だけ変更 → `stale`、生成物だけ手編集 → `stale`、生成物を CRLF 化しただけ → 一致、ヘッダ付きの対応表外ファイル → `orphans`（7.1, 7.2, 7.4）。
- `checkSkillSelfContained`: `../` で脱出するリンク、実在しない `references/x.md`、存在しない見出しへの `x.md#anchor`、`doc/spec` を含む HTML コメントを検出し、`https://` リンクと実在見出しへのアンカーは許可する（5.3, 6.4, 6.5）。`headingSlug`: 英字見出し・日本語見出し・記号入り見出し（`set_scene_selector(...) / set_word_selector(...)` → `set_scene_selector--set_word_selector`）・同名見出しの付番。旧リンク切れケース（相対 `.md`・GitHub URL・トラバーサル）は非回帰（8.4）。

### Integration Tests（実リポジトリ）

- `node book/tools/gen-skill-refs.mjs --check` が exit 0（全 14 生成ファイルが最新・孤立なし）。
- `node book/tools/link-check.mjs` が exit 0（book 内リンク切れなし・2 スキルが自己完結）。
- `node book/tools/verify-content.mjs` が exit 0（`A-nospec`・`A-summary:*` を含む）と `verify-scripts-test.mjs` の非回帰（8.3, 9.7, 4.2）。
- `cargo test -p pasta_lua --test loader`（`config_defaults_test` を含むターゲット）がマニュアル章を読んで成功し、表の値を 1 つ変えると該当キー名付きで失敗することを実装時に一度確認する（4.3, 4.4）。
- `cargo test --all` 全成功（10.3）。

### E2E（CI パイプライン）

- `manual.yml` の PR 実行で、鮮度チェック→ビルド→リンク検証→既存検証→自己テストがすべて緑（10.4, 8.2）。
- スキル配下だけを変更したコミットで `manual.yml` が起動する（`paths` の確認・7.5）。
- `mdbook build book` の成果物に新章 2 つが含まれ、目次から到達できる（4.2・`verify-static.mjs` の SUMMARY リンク健全性）。

### 内容の網羅確認（人手＋台帳）

- 吸収台帳の全行が「収録先」または「除外理由」で埋まり、実装照合列が空でない（1.1, 1.2, 1.6, 3.1–3.3, 10.6）。
- 参照修正の網羅 grep（ReferenceRepair 参照）の残存行がすべて許容理由付き（9.1, 9.2）。

## Migration Strategy

同一ブランチ上で以下の順に進め、最後に単一 PR で統合する（10.1）。途中の状態は main に出さない。

```mermaid
graph LR
    P1[content migration and ledger] --> P2[new chapters and SUMMARY]
    P2 --> P3[generator and tests]
    P3 --> P4[skill switch and SKILL md]
    P4 --> P5[link check and verify content and CI]
    P5 --> P6[retire doc spec GRAMMAR drift]
    P6 --> P7[reference repair and briefs and roadmap]
    P7 --> P8[full verification]
```

- P1 で吸収元を読みながら台帳を作るため、P6 で吸収元を削除する前に網羅が確定している。
- P4 の時点で旧手書きスキルファイルは生成物で上書きされ、旧名 4 ファイル（`grammar-model.md`・`call-spec.md`・`runtime-api.md`・`shiori-handlers.md`）は `git rm` で削除する。旧ファイルにしか無い内容は P1 の台帳で移設済みであることを前提とする。
- ロールバック: 全変更が 1 PR のため、マージ前なら PR を閉じる、マージ後なら squash コミットを revert する。

## Open Questions / Risks

- リスク: 移設時の規範内容の欠落 → 吸収台帳の全行充足を完了条件にする。
- リスク: 単一 PR が大きくレビュー負荷が高い → Migration の P1〜P8 単位でコミットを分け、PR 説明に台帳へのリンクを置く。
- リスク: マニュアルの見出し変更でスキル内アンカーリンクが黙って切れる → LinkCheck の `skill-anchor` で検出する（スキル内は GitHub 方式 slug で判定。mdBook の slug との差は公開 URL 側の問題であり本検査の対象外）。
- リスク: 既知の食い違い（約 20 件）の訂正で作業量が増え、「移し替えのみ」の印象と衝突する → 訂正は「実装に合わせた記述修正」であり挙動不変（10.5）であることを台帳の実装照合列で示す。
- リスク: 生成対象章の本文に口調コラムを置けなくなる（現行 `AUTHORING.md` はコラムを許容） → 生成器が `voice-in-body` で検出し、規約を `AUTHORING.md` へ明記する。

### 設計ディスカッションへ持ち越す論点（前提を置いて起草済み）

1. ~~スキル生成ファイルの命名~~ → 解決済み（#3）: 全生成ファイルを章名に揃える（`outName` で導出、`index.md` は `{dir}-index.md`）。旧名 4 ファイルは削除。
2. 生成対象外・別スキル宛ての章間リンクの扱い（前提: 公開マニュアル URL へ書き換え）。
3. 鮮度チェックの CI 配置（前提: `manual.yml` の `paths` 拡張＋先頭ステップ）。
4. SHIORI 章の構成（前提: `lua/shiori-events.md` 1 章、`lua/modules.md` は 1 章のまま約 700 行へ拡充）。
5. 動的単語参照 `＠＄` を brief 化するか（前提: brief 起票）。
6. `book/CONTENT-REVIEW.md` の扱い（前提: 歴史的記録の注記を付けて残す）。
7. `review-improvement-loop` の修正範囲（前提: 今後の指示として読まれる箇所のみ。完了済みセル記録と `reports/` は歴史的記録として残す）。
8. 既知の食い違いの訂正範囲（前提: 生成対象外の `lua/patterns.md` と `SKILL.md` 早見表も訂正）。
9. 吸収元に無い実装事実の扱い（前提: 収録義務の対象外。台帳付録に列挙）。
10. ~~生成対象章の本文から口調を一切排除する規約~~ → 解決済み（#1）: 全面禁止。検出は広い `VOICE_MARKERS` をコードフェンス・表・インラインコード除去後の散文に適用。
11. ~~生成ファイル内アンカーへのリンク検証~~ → 解決済み（#2）: 2 スキル内のアンカー付きリンクを `skill-anchor` で検証。book 内は対象外。
