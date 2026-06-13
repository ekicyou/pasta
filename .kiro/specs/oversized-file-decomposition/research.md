# Gap Analysis: oversized-file-decomposition

> 純粋リファクタリング（振る舞い不変）仕様のため、本ギャップ分析は「不足機能の発見」ではなく
> **既存資産の検証・分割対象の確定・適用すべき既存パターンの特定・リスク隔離**を主眼とする。
> 全データは本ワークツリー（`claude/upbeat-newton-4b1cc6`）で 2026-06-14 時点に実測・検証済み。

## 1. Analysis Summary

- **本仕様にコードレベルのブロッカーは無い。** 主機構（インラインテストの兄弟ファイル外出し）の前提（`#[cfg(test)] #[path]` + `use super::*;` 規約・前例・マクロ生成テスト不在）は実コードで実証済み。
- **分割対象は 4 カテゴリ・約 20 ファイル**に確定。実測の 600 行超ファイルは 28 本だが、`tests/` の `*_test.rs` や既に `#[path]` 外出し済みのファイルを除いた是正対象は下表の通り。
- **brief（2026-06-13）からファイルが drift している。** `transport.rs` 844→1340、`debug/mod.rs` 1108→1288 等が直近の `debug-transport-hardening` コミットで増大。**実装開始時にインベントリを再スナップショットすること**（Research/Constraint 扱い）。
- **3 つの難易度層**に自然に分かれる：(A) 機械的・低リスクなテスト外出し、(B) 設計判断を伴う本番構造分割、(C) 順序依存の `handle_inbound` ループ解体（最高注意・隔離タスク）。
- **段階的検証が唯一の安全網。** 振る舞い不変の保証は「各ファイル/クレート単位で分割→`cargo build`/`cargo test` green」を反復する以外にない。`NoDefaultCurrentDirectoryInExePath` 無効化が `cargo` 実行の前提。

## 2. Current State Investigation（実測）

### 2.1 実測 600 行超ファイル一覧（target/worktree 除外, 2026-06-14）

| 行数 | ファイル | カテゴリ | brief 記載 |
|---:|---|---|---|
| 5539 | `pasta_lua/src/debug/wiring.rs` | C1+C4（テスト外出し＋ループ解体） | 5539（一致） |
| 3090 | `pasta_lua/src/debug/session.rs` | C1 | 3090（一致） |
| 1818 | `pasta_lua/src/debug/dap.rs` | C1+C3 | 1818（一致） |
| 1612 | `pasta_lua/tests/runtime/runtime_toggle_e2e_test.rs` | C2 | 1612（一致） |
| 1474 | `pasta_lua/src/debug/source_map.rs` | C1 | 1474（一致） |
| 1426 | `pasta_lua/src/debug/inspect.rs` | C1 | 1426（一致） |
| **1340** | `pasta_lua/src/debug/transport.rs` | C1 | **844 → drift +496** |
| **1288** | `pasta_lua/src/debug/mod.rs` | C1+C3 | **1108 → drift +180** |
| 1023 | `pasta_shiori/src/shiori_tests.rs` | C2（`#[path]` 済） | 1023（一致） |
| 996 | `pasta_lsp/src/analysis/visitors.rs` | C3（純本番・test 0%） | 996（一致） |
| 981 | `pasta_lua/src/code_gen/element_gen.rs` | C1 | 981（一致） |
| 961 | `pasta_dsl/tests/cue_cmd_test.rs` | C2 | 961（一致） |
| 817 | `pasta_shiori/tests/async_callback_integration_test.rs` | C2 | 817（一致） |
| 808 | `pasta_core/src/registry/scene_table_tests.rs` | C2（`#[path]` 済） | 808（一致） |
| 804 | `pasta_lua/tests/loader/config_test.rs` | C2 | 804（一致） |
| 798 | `pasta_lua/src/loader/config.rs` | C1 | 798（一致） |
| 766 | `pasta_lua/src/transpiler.rs` | C1 | 766（一致） |
| 759 | `pasta_lua/src/debug/hook.rs` | C1 | 759（一致） |
| **758** | `pasta_lua/tests/runtime/debug_integration_test.rs` | C2 | **brief 未記載（新規 drift）** |
| 739 | `pasta_shiori/tests/lua_request_test.rs` | C2 | 739（一致） |
| 718 | `pasta_lua/src/loader/mod.rs` | C3（純本番・test 0%） | 718（一致） |
| 665 | `pasta_lua/src/code_gen/scope_gen.rs` | C1 | 665（一致） |
| 654 | `pasta_lua/src/loader/discovery.rs` | C1 | 654（一致） |
| 648 | `pasta_shiori/src/windows.rs` | C1 | 648（一致） |
| 635 | `pasta_lua/tests/transpiler/record_wiring_test.rs` | C2 | 635（一致） |
| 635 | `pasta_lua/src/runtime/mod.rs` | C3（純本番・test 0%） | 635（一致） |
| 605 | `pasta_lua/tests/shiori/virtual_event_config_test.rs` | C2 | 605（一致） |
| 605 | `pasta_lua/src/loader/extract.rs` | C1 | 605（一致） |

※ brief のカテゴリ1（`breakpoints.rs` 591 / `types.rs` 567）は現在 600 行未満だが `#[cfg(test)]` 同居のため、規約準拠の観点で**任意の外出し候補**として残す。

### 2.2 規約・前例の検証（実コード）

- **`structure.md` 268–277 行 `src/` 内テスト配置方針**が SSOT。private/`pub(crate)` フィールド直アクセスが必要なテストは `#[cfg(test)] #[path = "<name>_tests.rs"] mod tests;` + `use super::*;` で兄弟ファイルへ。
- **実装前例（実在確認済み）**：
  - `pasta_core/src/registry/scene_table.rs:417-418` … `#[cfg(test)] #[path = "scene_table_tests.rs"]`
  - `pasta_shiori/src/shiori.rs:330-331` … `#[cfg(test)] #[path = "shiori_tests.rs"]`
- **`debug/` は規約未準拠**：`debug/*.rs` 全 10 本がインライン `#[cfg(test)]` を保持、`#[path]` 適用は 0 本。これが巨大ファイルの主因。

### 2.3 `wiring.rs` テストモジュール構造（実測・11 クラスタ確定）

`^mod NAME {`（トップレベル）で 11 個の独立テストモジュールを確認。`1 既存 mod → 1 兄弟ファイル`で機械的に対応可能：

| # | mod 名 | 開始行 |
|---:|---|---:|
| 1 | `tests` | 617 |
| 2 | `source_map_wiring_tests` | 1311 |
| 3 | `resolver_attach_tests` | 1369 |
| 4 | `attach_source_presentation_tests` | 1509 |
| 5 | `bp_translator_tests` | 1670 |
| 6 | `pasta_bp_e2e` | 1969 |
| 7 | `pasta_step_e2e` | 2481 |
| 8 | `pasta_mode_edge_e2e` | 3369 |
| 9 | `pasta_break_coalesce_e2e` | 3990 |
| 10 | `source_presentation_toggle_tests` | 4716 |
| 11 | `bridge_lifecycle_tests` | 5080 |

※ 12 個目の `#[cfg(test)]`（145 行）は本番コード内のインライン属性であり、テストモジュールではない。
※ 行番号は drift するため、実装時に再取得すること（命名は安定）。

### 2.4 `handle_inbound` / `run_socket_bridge` 制御フロー（実コード確認）

- `fn handle_inbound(...)`（free fn）@ `wiring.rs:280`、`pub(crate) fn run_socket_bridge(...)` @ `wiring.rs:231`。後者が 1 フレームごとに前者を呼ぶ。
- **分岐順序を実コードで確認（`apply → response → event → command`）**：
  1. `adapter.lock()`（poison 時は `return false`、bridge 内で panic しない不変条件）
  2. `if command == "pasta/sourcePresentation"` … トグル適用＋`attach_pasta_resolver` 再実行
  3. `if let Some(mode) = decoded.attach_source_mode`（`attach` 明示 mode・最高優先）… 適用＋resolver 再実行
  4. `if command == "attach"` … 完了 ack の後に `pasta/sourcePresentation` イベント emit
  5. `match decoded.command { Some(SetBreakpoints{..}) => …原子的…, Some(cmd) => 汎用 session 転送, None => … }`
- **`setBreakpoints` 分岐の不変条件**（コードコメントで明示）：「VM 実行中に有効な唯一のコマンド・session 非転送」。**原子的に保持し内部分解しない**（R4-1）。
- `run_socket_bridge` のループ多重化コアは書き換え対象外（R4-4）。

## 3. Requirement-to-Asset Map（要件→資産対応・ギャップタグ）

| 要件 | 主対象資産 | アプローチ | ギャップタグ |
|---|---|---|---|
| **R1** インラインテスト外出し | C1 全本番ファイル（`debug/*.rs`・`code_gen/*`・`loader/config.rs`・`transpiler.rs`・`windows.rs` 等） | 前例パターン適用（機械的） | **Constraint**: 規約準拠のみ。新規能力不要 |
| **R2** 巨大テストファイルのクラスタ分割 | C2（`runtime_toggle_e2e_test.rs` 1612・`cue_cmd_test.rs` 961・`shiori_tests.rs` 1023 等） | `tests/<category>/main.rs`+`mod` 規約 / `#[path]` 内クラスタ分割 | **Constraint**: 論理クラスタ境界の判断が必要 |
| **R3** 純粋本番の責務分割 | C3（`visitors.rs` 996・`loader/mod.rs` 718・`runtime/mod.rs` 635、外出し後の `dap.rs`/`debug/mod.rs` 残余） | ディレクトリモジュール化（既存 `code_gen/`・`loader/` 流儀） | **Unknown**: 責務境界＝設計判断（design フェーズへ繰越） |
| **R4** `handle_inbound` 解体 | `wiring.rs:280` free fn | 順序保証付きヘルパー抽出・`setBreakpoints` 原子保持 | **Constraint(High)**: 順序・不変条件の機械的保証 |
| **R5** 振る舞い不変・段階検証 | 全クレート | `cargo build`/`cargo test` 反復 green | **Constraint**: `NoDefaultCurrentDirectoryInExePath` 無効化が前提 |
| **R6** 規約準拠 | `structure.md` 規約 | 前例（`scene_table_tests.rs`/`shiori_tests.rs`）整合 | **Constraint**: 既存規約の適用のみ |

ギャップ凡例: **Missing**（新規構築要）／**Unknown**（design で要研究）／**Constraint**（既存制約への準拠）。本仕様に **Missing は 0**。純粋リファクタリングゆえ大半が Constraint、責務境界のみ Unknown。

## 4. Implementation Approach Options

本仕様は「拡張 vs 新規 vs 折衷」ではなく、**カテゴリ別の分割戦略**として A/B/C を提示する（全カテゴリを段階実行する前提）。

### Option A — テスト外出しスイープ優先（機械的・最大 ROI 先行）
**内容**: C1 のインラインテスト外出しを最優先で全消化 → C2 → C3 → C4 の順。
- ✅ 巨大ファイルの主因（テスト同居）を最小リスクで除去。前例があり機械的・レビュー容易
- ✅ 各ファイル独立 → 段階検証の粒度が細かく回帰隔離が容易
- ✅ C1 完了時点で `dap.rs`/`debug/mod.rs` の本番残余サイズが確定し、C3 の責務分割判断が正確になる
- ❌ 最難関の C4（ループ解体）が後半に集中、終盤リスクが残る
- **推奨**。C1→C3 の依存（残余サイズ確定）が自然で、リスク逓減型。

### Option B — クレート単位スライス
**内容**: `pasta_lua`（最大）→`pasta_shiori`→`pasta_lsp`→`pasta_core`→`pasta_dsl` の順に、各クレート内で C1〜C3 を完結。
- ✅ `cargo test -p <crate>` で検証スコープを限定でき高速
- ✅ クレート境界で PR/コミットを分割しやすい
- ❌ `debug/` の密結合（C4）が `pasta_lua` 内に埋もれ、隔離タスク化しにくい
- ❌ カテゴリ横断の規約一貫性レビューが分散

### Option C — 難易度層 3 分割（隔離実行）
**内容**: 層1=機械的テスト外出し（C1+C2）を一括 → 層2=本番構造分割（C3）→ 層3=`handle_inbound` 解体（C4）を完全隔離タスク化。
- ✅ C4 を順序依存の高注意タスクとして物理的に隔離（最大リスクの封じ込め）
- ✅ 層ごとにレビュー観点（機械的整合 / 設計判断 / 不変条件保証）を切り替えられる
- ❌ 層1 が巨大バッチになり、段階検証の粒度を別途定義する必要
- ❌ C3 の責務境界が C1 完了に依存するため、層境界に逆依存が残る

**折衷推奨**: Option A の順序（C1→C2→C3→C4）を骨格に、**C4 を Option C 同様の隔離タスク**として切り出す。C1 はファイル単位、C2/C3 はクラスタ/責務単位で段階検証。

## 5. Effort & Risk

| 層 | 対象 | Effort | Risk | 根拠 |
|---|---|---|---|---|
| C1 テスト外出し | ~15 本の本番ファイル | **L (1–2週)** | **Low** | 機械的・前例あり・ファイル独立。量が多いだけ |
| C2 テストファイル分割 | ~9 本の巨大テスト | **M (3–7日)** | **Low–Medium** | 論理クラスタ境界判断あり。振る舞いには無影響 |
| C3 本番責務分割 | `visitors.rs`/`loader,runtime/mod.rs` + 残余 | **M (3–7日)** | **Medium** | 責務境界＝設計判断。公開 API/可視性不変が制約 |
| C4 `handle_inbound` 解体 | `wiring.rs` free fn 1 本 | **M (3–7日)** | **High** | 順序保証・`setBreakpoints` 原子性・bridge 非 panic 不変条件 |
| 全体（段階検証込み） | 全クレート | **XL (2週+)** | **Medium** | 量×検証反復。各ステップ green 維持が律速 |

## 6. Research Needed（design フェーズへ繰越）

1. **C3 責務境界の確定**（Unknown）: `visitors.rs`（AST ビジター群）・`loader/mod.rs`・`runtime/mod.rs` を、既存ディレクトリモジュール流儀（`code_gen/`・`loader/`）に沿ってどの責務軸で分割するか。公開 API（`mod.rs` の `pub use` re-export）を不変に保つサブモジュール構成案を design で確定。
2. **C2 クラスタ境界の定義**（Constraint）: `runtime_toggle_e2e_test.rs`(1612)・`cue_cmd_test.rs`(961) 等を、`tests/<category>/main.rs`+`mod` 規約のどの粒度で再分割するか。10 本超クレートのサブディレクトリ化方針（structure.md 258–266）との整合。
3. **C4 順序保証のドキュメント化＋特性化テスト先行**（Constraint, High）: 抽出後ヘルパーの呼び出し順 `apply→response→event→command` をコードコメントで担保しつつ、**解体着手前に当該順序と `setBreakpoints` 原子性を固定する最小限の特性化テスト（characterization test）を先行整備**する（議題4で決定・新規テスト禁止の明示的例外）。既存テストでカバー済みなら流用、不足分のみ追加。解体は「1 ヘルパー抽出 = 1 検証 = 1 コミット」の独立 green・revert 可能な小ステップで実施。`setBreakpoints` 原子境界の正確な行範囲を実装時に再取得。
4. **インベントリ再スナップショット**（運用 Constraint）: brief→現状の drift（`transport.rs`/`debug/mod.rs` 増大・`debug_integration_test.rs` 新規）を踏まえ、**実装着手時に 600 行超リストと `wiring.rs` mod 行番号を再取得**してから着手する手順を tasks に明記。
5. **段階検証の粒度とコミット境界**（Constraint）: 「1 ファイル＝1 検証＝1 コミット」を基本とするか、クレート単位でまとめるか。`NoDefaultCurrentDirectoryInExePath` 無効化を各 `cargo` 実行の前段に組み込む運用も明記。
6. **debug テスト外出し時のポートガード保全**（Constraint, Low — 設計調査で de-risk 済）: 設計フェーズの実コード調査により、`debug/*.rs` の src インラインテストモジュールには `#[ctor]` も env 操作（`set_var`/`remove_var`）も**存在しない**ことが判明。debug テストは OS 割当ポート0（loopback）で bind し `PASTA_DEBUG*` を読むのみのため env 非依存で、src→兄弟ファイル移動は純機械的（保全すべきガード無し）。固定ポート中和の `#[ctor]` は **runtime テストハーネス側**（`tests/` 配下）に存在するため、C2 で `tests/runtime/runtime_toggle_e2e_test.rs`・`debug_integration_test.rs` を分割する際は、当該ハーネスの `#[ctor]` がテストバイナリに引き続きリンクされること（共有 `common`/support モジュール経由）のみ確認する。

## 8. Synthesis Decisions（design 合成結果）

- **一般化**: 4 カテゴリは各々が単一の反復操作。C1=「1 インライン `#[cfg(test)] mod` → 1 兄弟 `#[path]` ファイル＋`use super::*;`」、C3=「単一 `impl Type` を子モジュール群へ分割（祖先 private 参照は可視性変更不要）」、C2=「論理クラスタ → 複数テストファイル＋`main.rs` の `mod` 登録 / `#[path]` 多重宣言」、C4=「分岐 → 順序固定ヘルパー」。各カテゴリ内は機械的反復。
- **Build vs Adopt**: 新規構築・新規依存ゼロ。Rust ネイティブのモジュール／`#[path]`／split-`impl` と既存前例（`scene_table.rs`・`shiori.rs`）を adopt するのみ。
- **簡素化（最小設計）**: テスト外出しは**単一兄弟ファイルを既定**とし、単一移動後も 600 行超かつ自然なクラスタを持つ大型ファイル（`session_tests`・`dap_tests`・`source_map_tests` 等）に限り `#[path]` 多重宣言で複数兄弟へ分割する。投機的な抽象化・先回り分割はしない。
- **可視性変更の最小化**: C3 で必要な可視性変更は `loader::ProcessStats` の `private→pub(super)` 1 箇所のみ（never re-export・外部影響なし）。それ以外は Rust の「子モジュールは祖先の private 項目を参照可」規則に依拠し、フィールド／メソッド可視性を一切広げない。

## 7. Recommendations for Design Phase

- **推奨アプローチ**: Option A 順序（C1→C2→C3→C4）＋ C4 隔離。リスク逓減型で、C1 完了が C3 の責務分割判断を正確にする自然な依存を活かす。
- **主要な設計判断**: (1) C3 各ファイルの責務軸とサブモジュール構成（公開 API 不変）、(2) C4 ヘルパー抽出の関数分割境界と順序保証の表現方式、(3) C2 クラスタ分割の粒度。
- **繰越研究項目**: 上記 §6 の 1–5。特に C3 責務境界（Unknown）と C4 順序保証（High Risk）を design で重点設計。
- **不変条件チェックリスト**（全層共通）: 公開 API シグネチャ・可視性不変／テスト集合（名前・件数・アサーション）の移動のみ／各ステップ `cargo build`+`cargo test` green／`setBreakpoints` 原子性・bridge 非 panic。
