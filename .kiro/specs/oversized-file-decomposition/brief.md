# Brief: oversized-file-decomposition

## Problem
エージェント開発中に「`wiring.rs` は巨大（267KB）ですわね。要点だけ狙い撃ちで掴みますわ」という挙動が発生した。巨大ファイルは、AI・人間の双方にとって全体把握のコストを跳ね上げ、編集・レビュー・差分理解を阻害する技術的負債のサインである。開発者（および支援AI）が、ファイル全体を俯瞰できず部分的な「狙い撃ち」読解を強いられている。

## Current State
リポジトリ全体を俯瞰した結果、巨大ファイル問題の主因は **本番ロジックの設計崩壊ではなく、`#[cfg(test)]` のインラインテストモジュールが `src/` 内に大量同居していること** と判明した。

リポジトリ規約（`.kiro/steering/structure.md` 268–277行）は既に「private フィールドアクセスが必要な `src/` 内テストは `#[cfg(test)] #[path = "<name>_tests.rs"] mod tests;` で兄弟ファイルへ外出しする」と定めており、`pasta_core/src/registry/scene_table_tests.rs` と `pasta_shiori/src/shiori_tests.rs` が前例として存在する。`debug/` モジュール等はこの規約から取り残されている。

### 600行超ファイルの俯瞰（2026-06-13 時点・target/worktree 除外）

**カテゴリ1: 本番ファイル × インラインテスト同居（主目標 = テスト外出し）**
| ファイル | 総行数 | テスト比率 | 本番ロジック概算 |
|---|---:|---:|---:|
| `pasta_lua/src/debug/wiring.rs` | 5539 | ~97% | ~144 |
| `pasta_lua/src/debug/session.rs` | 3090 | ~90% | ~300 |
| `pasta_lua/src/debug/dap.rs` | 1818 | ~57% | ~773 |
| `pasta_lua/src/debug/source_map.rs` | 1474 | ~60% | ~580 |
| `pasta_lua/src/debug/inspect.rs` | 1426 | ~71% | ~402 |
| `pasta_lua/src/debug/mod.rs` | 1108 | ~37% | ~693 |
| `pasta_lua/src/code_gen/element_gen.rs` | 981 | ~42% | ~568 |
| `pasta_lua/src/debug/transport.rs` | 844 | ~65% | ~288 |
| `pasta_lua/src/loader/config.rs` | 798 | ~21% | ~623 |
| `pasta_lua/src/transpiler.rs` | 766 | ~66% | ~254 |
| `pasta_lua/src/debug/hook.rs` | 759 | ~70% | ~222 |
| `pasta_lua/src/code_gen/scope_gen.rs` | 665 | ~44% | ~367 |
| `pasta_lua/src/loader/discovery.rs` | 654 | ~78% | ~143 |
| `pasta_shiori/src/windows.rs` | 648 | ~57% | ~275 |
| `pasta_lua/src/loader/extract.rs` | 605 | ~62% | ~229 |
| `pasta_lua/src/debug/breakpoints.rs` | 591 | ~64% | ~212 |
| `pasta_lua/src/debug/types.rs` | 567 | ~45% | ~307 |

**カテゴリ2: 既に外出し済みだが巨大なテストファイル（クラスタ分割候補）**
- `pasta_lua/tests/runtime/runtime_toggle_e2e_test.rs` (1612)
- `pasta_shiori/src/shiori_tests.rs` (1023, `#[path]` 済み)
- `pasta_dsl/tests/cue_cmd_test.rs` (961)
- `pasta_shiori/tests/async_callback_integration_test.rs` (817)
- `pasta_core/src/registry/scene_table_tests.rs` (808, `#[path]` 済み)
- `pasta_lua/tests/loader/config_test.rs` (804)
- `pasta_shiori/tests/lua_request_test.rs` (739)
- `pasta_lua/tests/transpiler/record_wiring_test.rs` (635)
- `pasta_lua/tests/shiori/virtual_event_config_test.rs` (605)

**カテゴリ3: 純粋な本番肥大（真の構造分割が必要・テスト比率≒0）**
- `pasta_lsp/src/analysis/visitors.rs` (996, テスト0%・ASTビジター群)
- `pasta_lua/src/loader/mod.rs` (718, テスト0%)
- `pasta_lua/src/runtime/mod.rs` (635, テスト0%)
- `pasta_lua/src/debug/dap.rs` / `debug/mod.rs`（テスト外出し後の本番残余が ~773 / ~693）

### Viability（実現可能性）検証済み事項
- **テスト外出し（Part 1）= ブロッカーなし。** `#[cfg(test)] #[path]` + `use super::*;` 規約は実証済み。テストは `super::` 経由で private/`pub(crate)` 項目（例: `attach_pasta_resolver`）を参照しており、同一モジュールパスの兄弟ファイルへ移動しても保持される。マクロ生成テスト（`macro_rules!` / `rstest` / `paste`）はスコープ内に存在せず、抽出は機械的。
- **`wiring.rs` のテストは既に 11 個の独立 `mod NAME` ブロック**（617/1311/1369/1509/1670/1969/2481/3369/3990/4716/5080 行）に分割済み。クラスタ別分割は「1 既存モジュール → 1 兄弟ファイル」で自然に対応。
- **ループ解体（Part 2）= 高注意で実現可能。** `handle_inbound()` は free fn で `run_socket_bridge()` から1フレームごとに呼ばれる。`pasta/sourcePresentation` トグルの `apply→response→event→command` 順序はコード位置で担保。`setBreakpoints` 分岐は「VM 実行中に有効な唯一のコマンド・session 非転送」という不変条件を持ち、**原子的に保持**する。残り分岐（sourcePresentation / attach override / 即時応答 / 汎用 session 転送）は順序保証をドキュメント化した上でヘルパー関数化可能。

## Desired Outcome
- リポジトリ内の Rust ファイルが、AI・人間ともに「全体を俯瞰できる」サイズに収まる（巨大ファイルが「狙い撃ち読解」を強制しない）。
- `src/` 内のインラインテストは、リポジトリ規約 `#[cfg(test)] #[path]` に従って論理クラスタ別の兄弟テストファイルへ外出しされ、本番ロジックと分離されている。
- `debug/` の最も絡み合った制御フロー（`handle_inbound` の分岐）が、順序保証を保ったまま責務単位のヘルパーに解体され、可読性とテスト容易性が向上している。
- 純粋に肥大した本番ファイル（`visitors.rs`・`loader/mod.rs`・`runtime/mod.rs` 等）が責務単位のサブモジュールへ分割されている。
- 全変更を通じて `cargo test` / `cargo build` が green を維持し、振る舞いの回帰がない（純粋なリファクタリング）。

## Approach
**選択: リポジトリ全体スコープ × クラスタ別テストファイル分割 × 絡み合うループも解体（完全版）。**

1. **テスト外出しスイープ（主機構・高ROI）** — カテゴリ1の各本番ファイルについて、インライン `#[cfg(test)] mod ...` を論理クラスタ別の兄弟ファイル `<name>_<topic>_tests.rs` へ `#[cfg(test)] #[path] mod ...;` で分離。`wiring.rs` は既存の 11 モジュールを 11 兄弟ファイルへ。本番コードの振る舞いは不変。
2. **巨大テストファイルのクラスタ分割（カテゴリ2）** — 既に外出し済みでも巨大なテストファイルを、論理単位で複数ファイルへ再分割。
3. **本番構造分割（カテゴリ3）** — `visitors.rs`・`loader/mod.rs`・`runtime/mod.rs`、およびテスト外出し後も大きい `dap.rs`/`debug/mod.rs` を責務単位のサブモジュールへ分割。
4. **ループ解体（最難関）** — `debug/wiring.rs` の `handle_inbound()` を、`setBreakpoints` 分岐を原子的に保持しつつ、残り分岐を順序保証付きヘルパーへ抽出。

**理由**: 巨大ファイルの主因（テスト同居）を既存規約どおりに是正することで最大の可読性改善を低リスクで得つつ、真に肥大した本番ファイルとデバッグ制御フローの絡み合いにも踏み込むことで、妥協のない完全な是正を達成する。

## Scope
- **In**:
  - リポジトリ全 Rust ファイルの巨大ファイル是正（テスト外出し・テストファイル分割・本番構造分割）
  - `debug/wiring.rs` の `handle_inbound` ループ解体（順序保証を保つ範囲）
  - `structure.md` の `#[cfg(test)] #[path]` 規約への全面準拠
  - 純粋リファクタリング（振る舞い不変・既存テストが green を維持）
- **Out**:
  - 機能追加・バグ修正・振る舞い変更
  - `setBreakpoints` 分岐の分解（不変条件保護のため原子的に保持）
  - `run_socket_bridge` のループ多重化コア自体の書き換え
  - TypeScript（vscode 拡張）のテストファイル分割（Rust リファクタリングに集中。必要なら別途）
  - 公開 API の可視性変更を伴う `tests/` への完全外部化（カプセル化維持のため不採用）

## Boundary Candidates
- テスト外出し（機械的・低リスク） vs 本番構造分割（設計判断を伴う）
- 各クレート単位（pasta_lua / pasta_shiori / pasta_lsp / pasta_core / pasta_dsl）の独立した分割作業
- `debug/` モジュール内のファイル群（相互に密結合・まとめて扱う単位）
- ループ解体（`handle_inbound`）は順序依存のため独立した高注意タスクとして隔離

## Out of Boundary
- 振る舞いの変更・最適化（純粋リファクタリングに限定）
- `setBreakpoints` 分岐の内部分解
- 公開 API シグネチャや可視性の変更（テスト外出しは同一モジュールパス維持で可視性変更不要）
- 新規テストケースの追加（既存テストの移動・分割のみ）

## Upstream / Downstream
- **Upstream**: `.kiro/steering/structure.md`（`#[cfg(test)] #[path]` 規約・命名規則）、完了済み `ai-friendly-file-split` 仕様（同方針の先行作業）、`pasta_core/scene_table_tests.rs`・`pasta_shiori/shiori_tests.rs`（実装パターンの前例）
- **Downstream**: 今後の `debug/` 機能拡張・DAP 関連仕様が、解体後の見通しの良いコードベース上で進む。AI 支援開発全般の俯瞰コスト低減。

## Existing Spec Touchpoints
- **Extends**: なし（新規仕様。ただし完了済み `ai-friendly-file-split` の方針を全リポジトリへ拡張する位置づけ）
- **Adjacent**: `debug/` を扱う完了済み DAP 関連仕様群、`review-improvement-loop`（レビュー観点で隣接）

## Constraints
- **純粋リファクタリング**: 各分割の前後で `cargo test`（全クレート）と `cargo build` が green を維持すること。振る舞いの回帰禁止。
- **規約準拠**: `structure.md` の `#[cfg(test)] #[path = "<name>_tests.rs"] mod tests;` + `use super::*;` パターンに従う。private アクセスが不要な公開 API テストは従来どおり `tests/` 配置の判断基準を尊重。
- **環境**: LuaJIT ビルドのため `cargo` 実行前に `NoDefaultCurrentDirectoryInExePath` 環境変数の無効化が必要（既知の落とし穴）。
- **ループ解体の順序保証**: `apply→response→event→command` の順序はコード位置で担保されているため、ヘルパー抽出時は順序をドキュメント化し、テストで順序を検証すること。
- **段階的検証**: ファイル/クレート単位で分割→ビルド→テストを繰り返し、各ステップで green を確認しながら進める（一括変更でのデグレ回避）。
