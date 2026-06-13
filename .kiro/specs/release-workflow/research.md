# Research & Design Decisions: release-workflow

## Summary
- **Feature**: `release-workflow`
- **Discovery Scope**: Extension（既存ツール群の組み合わせによるオペレーション仕様）
- **Key Findings**:
  - `cargo publish` の認証は環境変数で有効であり、前提条件チェックは不要（`gh auth status` のみ確認すれば十分）
  - ルート `Cargo.toml` の5箇所のみでバージョン管理が完結する構造が確認済み
  - 既存リリース `v0.1.2` が完全な参照モデルとして利用可能（タイトル形式、アセット構成）

## Research Log

### cargo publish 認証トークン
- **Context**: gap-analysis.md で「未確認」としていた認証トークンの有無を実際に確認
- **Sources Consulted**: ローカルファイルシステム確認（`~/.cargo/credentials.toml`, `~/.cargo/credentials`）、環境変数
- **Findings**:
  - ファイルベースの credentials は存在しないが、環境変数 `CARGO_REGISTRY_TOKEN` による認証が有効
  - 過去のリリースで `cargo publish` が正常に動作していることを確認済み
  - cargo は環境変数とファイルの両方をサポートしている
- **Implications**: cargo publish の認証チェックは不要。環境変数による認証が既に有効であり、Phase 0 での前提条件確認は `gh auth status` のみで十分

### Cargo.toml バージョン更新箇所
- **Context**: gap-analysis.md で確認済みだが、設計のための正確な行番号を再確認
- **Sources Consulted**: `Cargo.toml` 直接読み取り
- **Findings**:
  - Line 9: `version = "0.1.2"` — `[workspace.package]` セクション
  - Line 47: `pasta_core = { path = "crates/pasta_core", version = "0.1.2" }`
  - Line 48: `pasta_lua = { path = "crates/pasta_lua", version = "0.1.2" }`
  - Line 49: `pasta_shiori = { path = "crates/pasta_shiori", version = "0.1.2" }`
  - 個別クレートの `Cargo.toml` は `version.workspace = true` で継承（更新不要）
- **Implications**: `replace_string_in_file` で旧バージョン文字列を新バージョンに4回置換すれば完了

### 既存リリース構造（v0.1.2）
- **Context**: GitHub Release 作成時のコマンドとパラメータの参照モデル
- **Sources Consulted**: gap-analysis.md の記録、RELEASE.md のテンプレート
- **Findings**:
  - タイトル形式: `pasta vX.Y.Z`
  - アセット: `pasta.dll` (2.59 MiB), `hello-pasta.nar` (1.29 MiB)
  - DLL パス: `target/i686-pc-windows-msvc/release/pasta.dll`
  - NAR パス: `release/hello-pasta.nar`（release.ps1 が WorkspaceRoot の release/hello-pasta.nar に出力）
- **Implications**: `gh release create` のコマンド構築時にこれらのパスとタイトル形式を使用

### チェンジログ生成パターン
- **Context**: 議題1で決定済み — `git log` + LLM 手動整形方式
- **Sources Consulted**: Conventional Commits 仕様、`git log` 出力のサンプル
- **Findings**:
  - プロジェクトのコミットメッセージは `type(scope): summary` 形式に従っている
  - 分類カテゴリ: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`
  - グループ見出し: `### ✨ Features`, `### 🐛 Bug Fixes`, `### 📝 Documentation` 等
  - `chore(spec):` や `docs(spec):` のような仕様管理コミットはリリースノートから除外が望ましい
- **Implications**: LLM がコミット履歴を読み取り、ユーザー向けに有意義なエントリのみを整形する

### release.ps1 実行フロー
- **Context**: gap-analysis.md の分析に基づく実行手順の確認
- **Sources Consulted**: gap-analysis.md（387行スクリプト、8ステップ構成）
- **Findings**:
  - 実行ディレクトリ: `crates/pasta_sample_ghost/`
  - 実行コマンド: `PowerShell -ExecutionPolicy Bypass -File release.ps1`
  - 出力: `hello-pasta.nar` + `target/i686-pc-windows-msvc/release/pasta.dll`
  - 前提: `i686-pc-windows-msvc` ターゲットがインストール済み（✅確認済み）
- **Implications**: ステップ4で `Push-Location` + `release.ps1` 実行 + `Pop-Location` の流れ

## 並行作業性の分析（cc-sdd 3.0 書き直しで追加）

### Context
旧設計は全工程を単一の Sequential Pipeline として直列実行していた。本書き直しでは、各処理が要求する共有リソースを実コードから確認し、安全に並行化できる箇所と偽の依存関係を特定した。

### Sources Consulted
- `crates/pasta_sample_ghost/release.ps1`（Step 1: `cargo build --release --target i686-pc-windows-msvc -p pasta_shiori`）
- `editors/vscode/package.json`（`prepackage` = `build:wasm` → `compile`、`build:wasm` = `powershell -File scripts/build-wasm.ps1`）
- `Cargo.toml`（workspace 構造、6箇所のバージョン参照）
- `cargo publish` の挙動（既定で検証ビルド + クリーンワークツリー要求）

### Findings — 共有リソースモデル
| リソース | 種別 | 保持する処理 |
| --- | --- | --- |
| R1: cargo ターゲットロック | 排他 | `cargo build/test/run/publish`、VSCode `build:wasm` |
| R2: git ワークツリー＋index | 排他 | ファイル生成、`git add/commit/restore/tag`、`release.ps1` |
| R3: ネットワーク | 非排他 | `cargo publish` upload/index待機、`vsce publish`、`gh release create` |

### Findings — 偽の依存関係
- 旧設計は「crates.io 公開（Phase 3）→ ゴーストビルド（Phase 5）」と直列化していたが、`release.ps1` は **ローカルソースから** pasta.dll をビルドしており crates.io 公開済みクレートに依存しない。よってゴーストビルドは crates.io 公開に**非依存**であり、バージョン更新コミットにのみ依存する。

### Implications — スケジューリング決定
- R1・R2 を共有する全ローカルビルドは真の並行ができないため、1つの直列ステージ（Stage A）に集約し、ワークツリーをクリーン化してから crates.io 公開を開始する。
- crates.io 公開（Track X）・Marketplace 公開（Track Y）・チェンジログ生成（Track Z）は R2 を変更せず互いに独立するため Stage B として**並行実行可能**。特に非クリティカルな `vsce publish`（R3 のみ）を Track X の長いインデックス待機に重ねることで wall-clock を短縮する。
- 不可逆な crates.io 公開（Track X）の成功を Stage C（タグ・プッシュ）の前提とし、安全順序を保証する。
- VSCode は `build:wasm`（R1）を要するため**ビルドは Stage A**、Marketplace への **upload（R3 のみ）は Stage B Track Y** に分離する。

### Decision: Resource-Aware Staged Concurrency（採用）
- **Selected Approach**: Stage A（ローカル直列）→ Stage B（並行3トラック X∥Y∥Z）→ Stage C（タグ・プッシュ）→ Stage D（GitHub Release）
- **Rationale**: 排他リソース制約を尊重しつつ、独立したネットワークトラックを並行化して所要時間を短縮し、非クリティカル失敗を隔離する。
- **Trade-offs**: 並行トラックの完了個別検証が必要（Req 8.7）。逐次環境ではインターリーブ近似となる。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 完全インタラクティブ | LLM が各ステップを逐次実行 | 柔軟なエラー対応、結果確認、チェンジログ整形が自然 | セッション切断リスク、実行時間 | **採用** — 仕様の趣旨に最適 |
| B: ラッパースクリプト | 全工程を自動化スクリプト化 | 再現性、実行速度 | 対話要素の処理困難、保守コスト | 不採用 — 仕様の趣旨と矛盾 |
| C: ハイブリッド | 部分スクリプト化 | チェンジログ品質安定 | 境界管理の煩雑さ | 不採用 — 不要な複雑性 |

## Design Decisions

### Decision: 完全インタラクティブ実行（Option A）
- **Context**: 本仕様は LLM が繰り返しリリース作業を実行するオペレーション仕様
- **Alternatives Considered**:
  1. Option A — LLM が各ステップをターミナルで逐次実行
  2. Option B — ラッパースクリプト作成
  3. Option C — 部分スクリプト化
- **Selected Approach**: Option A — LLM が `run_in_terminal` と `replace_string_in_file` を組み合わせて実行
- **Rationale**: 仕様の趣旨（LLM による繰り返し実行）に最適。エラー時の柔軟な判断、チェンジログの知的な整形が可能
- **Trade-offs**: 実行時間は長いが、品質と柔軟性を優先
- **Follow-up**: セッション切断時の中間状態からの復旧手順を設計に含める



### Decision: チェンジログの仕様管理コミット除外
- **Context**: `docs(spec):` や `chore(spec):` のコミットはリリースノートに不要
- **Selected Approach**: LLM が Conventional Commits のプレフィックスとスコープを判定し、仕様管理（spec）スコープのコミットを除外
- **Rationale**: ユーザー向けリリースノートに内部仕様管理の変更は不要
- **Trade-offs**: LLM の判断に依存するが、コンテキスト理解力で十分対応可能

## Risks & Mitigations
- **Risk 1**: セッション切断時の中間状態 → 各ステップでコミットを行うため、`git log` で進捗を把握し再開可能
- **Risk 2**: crates.io インデックス更新遅延 → 10秒待機＋確認で対処。不足時は追加待機
- **Risk 3**: `gh` CLI 認証切れ → Phase 0 で `gh auth status` を確認し、未認証ならガイダンス提示

## References
- [Conventional Commits](https://www.conventionalcommits.org/) — コミットメッセージ分類基準
- [cargo publish](https://doc.rust-lang.org/cargo/commands/cargo-publish.html) — crates.io 公開コマンド
- [gh release create](https://cli.github.com/manual/gh_release_create) — GitHub Release 作成コマンド
- gap-analysis.md — 既存アセットとギャップの詳細分析

---

# ギャップ分析（Req 10 追加: ワークツリー実行・PR ベース main 統合）

> 2026-06-14 追記。本セクションは新設 **Requirement 10**（ワークツリー実行と PR マージコミット方式での main 統合）に限定したギャップ分析である。旧仕様（v0.1.2 基準）の Req 1–9 のギャップは上記既存セクションおよび `gap-analysis.md` を参照。

## 1. 現状調査（Req 10 関連アセット）

| 要件領域 | 既存アセット | 状態 |
|----------|-------------|------|
| ワークツリー供給 | Claude Code ハーネスが非デフォルトブランチを供給（kiro-complete と同一前提） | ✅ 既に成立 |
| PR 作成・マージ機構 | `kiro-complete` SKILL.md「PR 可否判定 / PR 作成→マージ→ブランチ削除 / 中断条件 / エラー回避」 | ✅ **ほぼ完全な参照実装**（`--squash` 版） |
| PR 可否ゲート | 非デフォルトブランチ＋`{remote}`あり＋`gh` 認証（kiro-complete） | ✅ そのまま流用可 |
| ブランチ削除 | `gh pr merge --delete-branch`（repo `deleteBranchOnMerge: false` のため API 削除に依存） | ✅ 流用可 |
| 失敗時セマンティクス | 「ブランチを残し中断」「ローカル削除警告は非致命」（kiro-complete） | ✅ 流用可 |
| push 許可 | `.claude/settings.json` L4 `Bash(git push origin main:*)`（カーブアウト用直 push 許可） | ⚠️ PR 化で**不要化**、別途タグ push 許可が**未整備** |
| ステアリング | `workflow.md` §3（PR squash）＋ L113 リリースカーブアウト（直 push 容認・settings 許可保持） | ⚠️ Req 10 と**整合せず要改訂** |
| 現行設計/タスク | `design.md` L130/227/493・`tasks.md` L128 が `git push origin main --tags`（直 push 前提） | ⚠️ Stage C を**全面書き換え対象** |

## 2. Requirement-to-Asset Map（Req 10）

| AC | 必要機能 | 既存アセット | ギャップ |
|----|---------|-------------|---------|
| 10.1 ワークツリー上で動作 | 現在ブランチ上で実行 | ハーネス供給（kiro-complete 同様） | **なし**（前提が既に成立） |
| 10.2 コミットを作業ブランチに保持 | `git commit`（Stage A） | 既存 | **なし** |
| 10.3 PR マージコミット統合・SHA 保持 | `gh pr create` + `gh pr merge --merge` | kiro-complete の PR 機構（`--squash`） | **Constraint**: `--merge` へ変更／**repo の merge-commit 許可が要確認** |
| 10.4 squash-PR・直 push 禁止 | フロー選択・経路撤去 | settings.json 直 push 許可 | **Constraint**: 直 push 経路撤去、許可エントリ見直し |
| 10.5 タグ到達性・タグ push | タグ ref の push | `git tag -a` / `git push`（既存手順） | **Missing**: settings.json に**タグ push 許可なし**（`git push origin main:*` はタグ ref を被覆しない） |
| 10.6 / 10.7 公開前のマージ可能性検証 | `gh pr view --json mergeable` or `git fetch`+dry-run | なし | **Missing**: 不可逆 crates.io 公開前の**マージ可能性プローブ**が未設計 |
| 10.8 失敗時は非破壊で中断 | ブランチ非削除・中断 | kiro-complete の中断セマンティクス | **なし**（パターン流用） |

## 3. 重大な落とし穴（Critical Findings）

### ⚠️ 落とし穴1: repo が merge-commit を許可しているか未確認（最優先）
`gh pr merge --merge`（マージコミット方式）は repo 設定 `mergeCommitAllowed: true` を要する。本 repo は spec 完了で `--squash` を常用しており、**merge-commit が有効かは不明**。無効の場合 Req 10 の中核が成立しない。

> **【確認結果 2026-06-14・設計フェーズ】** `gh repo view --json mergeCommitAllowed,squashMergeAllowed,rebaseMergeAllowed,deleteBranchOnMerge` 実行: `mergeCommitAllowed=false`, `squashMergeAllowed=true`, `rebaseMergeAllowed=false`, `deleteBranchOnMerge=true`（`ekicyou/pasta`, default=`main`）。**→ merge-commit は現状無効。`gh pr merge --merge` は今のままでは失敗する。** Req 10 成立には一回限りで `gh repo edit --enable-merge-commit` を実施（squash は spec 完了フローで使用中のため**併存維持**＝両方有効）。なお `deleteBranchOnMerge=true`（旧 kiro-gitflow 記録の `false` から変化済み）のため `--delete-branch` は冗長だが無害。**Research Needed #1 解決済み**。

### ⚠️ 落とし穴2: 不可逆な crates.io 公開とマージ失敗の順序リスク
現行設計は Stage B（crates.io 公開＝不可逆）→ Stage C（タグ・統合）。PR マージはこれより後段になるため、「公開済みだが PR マージ失敗（コンフリクト等）」の窓が生じる。Req 10 AC6/7 はこれを「**不可逆公開の前に**マージ可能性を検証」で緩和する設計意図だが、**検証手段とタイミングが未設計**。→ **Research Needed #2**（PR 早期作成＋`mergeable` ポーリング vs `git fetch origin {default} && git merge-tree`/dry-run マージ）。残余リスク（検証〜マージ間の main 移動）は既存 Req 3.4「既公開クレートは残し開発者指示待ち」の許容範囲内。
>
> **【議題1 決定 2026-06-14】**: **Option 2「統合先・公開後」を採用**。安全順序を「main 統合（タグ・PR マージ）→ crates.io 公開 → GitHub Release」へ反転し、不可逆な公開を可逆な統合の後段に置く。これにより「公開済みだが統合不能」の窓が消滅（統合が先・ゲートになる）。統合成功後に公開が失敗した場合は main は既に正しいリリース状態であり、公開リトライ／中断で回復（Req 8.5・10 AC8）。要件側は Req 8.5・8.2・6.1・3.1・10 AC2/6–8 を更新済み。設計フェーズは Stage 順序を「Stage A 準備・ビルド → Stage B 統合（tag+PR merge）→ Stage C 公開（crates.io ∥ Marketplace）→ Stage D GitHub Release」へ再構成すること。

### ⚠️ 落とし穴3: settings.json／steering がカーブアウト（直 push）前提のまま
`.claude/settings.json` の `Bash(git push origin main:*)` と `workflow.md` L113 カーブアウト（DD5, kiro-gitflow-worktree-pr 由来）は**リリース直 push を許容する設計**で、Req 10（PR ベース・直 push 禁止）と矛盾する。Req 10 では (a) **タグ push 許可**（例 `Bash(git push origin v*:*)` 等）を追加し、(b) 直 push 許可とカーブアウトを**タグ公開限定に縮退 or 撤去**する必要がある。
>
> **【議題2 決定 2026-06-14】**: これら周辺設定変更（settings.json 許可・workflow.md カーブアウト改訂・repo merge-commit 有効化）は **release-workflow の繰り返しタスクには含めない一回限りのセットアップ**として扱う。`spawn_task`（チップス）等での別セッション委譲はワークツリー隔離のため不可（別セッションは独自ワークツリーで起動し本ブランチの未コミット状態を継承できない）。よって**本セッション内で、設計確定後（タスク分解の前後）にエージェントが手動で実施**する。Steering Gate でも整合確認。

## 4. 実装アプローチ評価

### Option A: 既存 Stage C を PR 化＋kiro-complete パターン流用（推奨）
Stage C を「タグ作成 → `gh pr create` → `gh pr merge --merge --delete-branch` → タグ push」へ置換し、PR 可否ゲート・中断セマンティクス・ローカル削除警告の非致命扱いを kiro-complete から流用。Stage B 直前にマージ可能性プローブ（AC6/7）を追加。settings.json／workflow.md を更新。
- ✅ 検証済み PR パターンの再利用で設計・実装コスト最小／挙動の一貫性
- ✅ コード新規作成ゼロ（オペレーション仕様の性質を維持）
- ❌ Stage 順序にマージ可能性プローブを挿入する調整が必要

### Option B: リリース専用 PR ヘルパー部品を新設
- ❌ 対話的 LLM 実行の趣旨に反し、新部品は過剰。不採用

### Option C: ハイブリッド（流用＋専用プローブ）
Option A に対しマージ可能性プローブのみ独立サブステップ化。実質 A の一形態。プローブ手段が確定するまでの暫定整理として有効。

## 5. 複雑度・リスク評価

- **Effort: S（1–3日）** — オペレーション仕様の手順差し替え。PR 機構は kiro-complete から流用、新規コードなし。主作業は design/tasks の Stage C 改訂＋settings.json＋workflow.md 更新。
- **Risk: Medium** — 単独では Low だが、(1) repo の merge-commit 許可状態が未確認（中核を左右）、(2) 不可逆公開×PR マージの順序リスク、の2点が Medium 要因。落とし穴1の確認で High/Low が確定する。

## 6. 設計フェーズへの推奨事項

1. **推奨アプローチ**: Option A。Stage C を PR マージコミット方式へ置換し kiro-complete パターンを流用。
2. **着手前に Research Needed #1（merge-commit 許可）を解消**してから設計確定すること。
3. Stage B 直前に **マージ可能性プローブ**（Req 10 AC6/7）を新ステップとして配置し、不可逆公開前ゲートとする。
4. **タグはタグ ref push**（`git push origin vX.Y.Z`）とし、`--merge` 後に main から到達可能化。Stage D（Release 作成）はマージ後に実行。
5. **settings.json／workflow.md の整合更新**を設計の File Structure Plan ＋ Steering Gate に明記（タグ push 許可追加、直 push 許可・カーブアウトの縮退）。

### Research Needed（設計フェーズで調査）
1. **【最優先】** repo の merge-commit 許可状態（`gh repo view --json mergeCommitAllowed,squashMergeAllowed,deleteBranchOnMerge`）。無効なら有効化要否を判断。
2. （議題1で方針確定）マージ可能性は **PR マージ実行そのものがゲート**となる（統合先・公開後 = Option 2 採用）。事前の読み取り専用プローブは安全ゲートとしては不要化。残る検討は「ビルド前に早期 fast-fail させるための任意の事前チェックを置くか」のみ（任意・最適化）。
3. 将来の GitHub ブランチ保護／タグ保護と本フローの相互作用（main 保護はタグ push を妨げないが、必須ステータスチェック有効化時は `gh pr merge` 即時マージがブロックされ得る）。
4. settings.json 許可エントリの最終形（タグ push 許可の具体パターン、`git push origin main` 撤去可否）。
5. タグ push と PR マージの実行順序（タグ ref を merge 前に push するか後にするか）と、Release 作成（Stage D）のマージ後実行の確定。
6. リリース PR の title/body 生成方針（`--merge` のマージコミットメッセージは自動付与のため、PR 本文の供給方法を決める。kiro-complete の squash メッセージ生成方針（`merge-base..HEAD` 履歴要約）を流用するか）。

## Design Synthesis（Req 10 設計フェーズ 2026-06-14）

### Build vs Adopt
- **Adopt**: PR 統合の制御ロジック（PR 可否判定・中断セマンティクス・`--delete-branch` ローカル削除警告の非致命扱い・メッセージ生成方針）は `kiro-complete` SKILL.md に検証済み実装がある。**新規構築せず流用**し、`--squash` を `--merge` に置換するのみ。リリース固有差分（マージコミット方式・タグ ref push・統合をゲートとする安全順序）だけを上乗せする。

### Simplification
- 議題1の Option 2（統合先・公開後）採用により、当初検討した「不可逆公開前の読み取り専用マージ可能性プローブ」は**不要化**。**PR マージ実行そのものが安全ゲート**となる（統合が先・失敗したら公開しない）。これにより別手段のプローブ設計（Research Needed #2）を削減し、Stage 構成を単純化。
- 早期 fast-fail のための任意事前チェックは設計に含めない（過剰）。Phase 0 で merge-commit 許可と非デフォルトブランチを確認するのみ。

### Generalization
- Stage モデルを「準備 → **統合** → 公開 → Release」の 4 段に一般化。統合フェーズ（Stage B）を独立の安全ゲートとして切り出し、将来 main 直 push 禁止（ブランチ保護）が有効化されても同一フローで成立する構造とした。

### Decision: 4-Stage Resource-Aware Staged Concurrency（改訂版・採用）
- **Selected**: Stage A（ローカル直列）→ Stage B（統合 = tag + PR merge --merge、安全ゲート）→ Stage C（公開 crates.io ∥ Marketplace）→ Stage D（GitHub Release）
- **Rationale**: 不可逆な crates.io 公開を可逆な main 統合の後段に置き、「公開済みだが統合不能」を排除。PR マージコミット方式で SHA・タグの参照整合性を保ち、直 push を廃して将来のブランチ保護に前方互換。
- **Trade-offs**: main にマージコミットが 1 つ増える（squash の単一コミットより履歴は冗長）。repo の merge-commit 有効化（一回限りセットアップ）が前提。

### Decision: ブランチ現在性は「ビルド前の自動非破壊マージ」（設計議題1 2026-06-14）
- **Context**: `gh pr merge --merge` は main 分岐時に 3-way マージとなり、(a) ローカル HEAD と統合後 main の乖離、(b) ローカル HEAD から実行する `cargo publish` の公開内容が main と不一致、という二重リスクがある（レビュー Critical Issue 1・2）。
- **Selected**: main 先行を検出したら **Phase 1（ビルド前）で `git merge origin/{default}` により非破壊で取り込む**（自動更新）。`reset`/`rebase` は使わず steering の危険 git 操作禁止に準拠。コンフリクト時は `git merge --abort` で中止・報告。Stage B Phase 6 で**最終 ff 再検証**し、Phase 1 後に main が再先行した稀ケースはリビルドループ回避のため中止・再実行誘導。
- **Rationale**: ビルド前取り込みにより成果物（crates/ghost/VSIX）が更新後ツリーを反映し、公開内容＝main＝タグの整合が保証される（Req 10.9）。取り込みを Stage B に置くと成果物が陳腐化し再ビルドが必要になるため前倒し。
- **Trade-offs**: main 先行時にマージコミットが作業ブランチに増える（rebase の線形性より保守的だが安全）。再ビルドコストは「先行検出時のみ」に限定。
- **Impact**: design.md Phase 1 step3・Phase 6 step0・Track X 前提・Req 10.9 を追加。settings.json 一回限りセットアップに `git fetch`/`git merge` 許可を追加。

## References（追加）
- `.claude/skills/kiro-complete/SKILL.md` — PR 可否判定・PR 作成/マージ・中断条件・エラー回避（流用元の参照実装）
- `.kiro/steering/workflow.md` L83–113 — リモート同期（PR squash）＋リリースタグ公開カーブアウト
- `.claude/settings.json` — `git push origin main` 許可（カーブアウト用、要見直し）
- `.kiro/specs/completed/kiro-gitflow-worktree-pr/` — PR 化の設計判断（DD5 カーブアウト、`deleteBranchOnMerge: false` 等）
