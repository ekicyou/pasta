# Brief: pasta-runtime-internals-doc

> **ステータス**: 未着手（discovery 完了・2026-10-01）。`manual-ssot-authority` 完了後に `/kiro-start pasta-runtime-internals-doc` で開始する。

## Problem

pasta ランタイムの内部設計（トランスパイル・コルーチン・シーン検索・ローダ・SHIORI 非同期／アクター基盤・デバッグ）を通して理解できる文書が存在しない。コントリビュータ（および実装を担う AI エージェント）は、完了済み spec の design.md 群やコードを横断して読み解くしかない。

既存の内部向け記述は散在し、一部は陳腐化している:
- `OPTIMIZATION.md`（145 行・最終更新 2026-01-25「Phase 0 完了」のまま）
- 各クレート `README.md`（`pasta_lua` 573 行など。crates.io の顔と内部解説が混在）
- `pasta-lua-coding` スキル `references/internal-modules.md`（794 行・`STORE` パターン等）
- `book/src/reference/startup.md`（利用者向けだが起動シーケンスに踏み込む）

由来: pasta-user-manual の設計ディスカッションで「ランタイム内部設計は利用者マニュアル外・将来仕様」と決定（R5 は API 使用法に限定）。以後 pasta-manual-debugging・lua-require-robustness 等でも申し送り先として参照され続けている。

## Current State

- mdBook マニュアル（`book/`）は利用者向け（入門・文法・Lua・デバッグ・リファレンス）のみ。
- `manual-ssot-authority`（上流）が「mdBook を権威とし、スキル `references/` の利用者向け部分を mdBook から生成する」方式と生成機構を確立する。`internal-modules` の権威移動は本 spec へ申し送られている。

## Desired Outcome

- 同じ mdBook の末尾に**「内部設計（コントリビュータ向け）」パート**があり、ランタイム内部設計を一貫して解説している。
- 散在していた内部向け記述がこのパートへ集約され、権威が一本化されている。
- 内部設計章が実装と乖離し続けない運用（完了ゲート＋定期レビュー）が組み込まれている。

## Approach

**同一 mdBook に内部設計パートを追加（案 A）。**
- 議題 1（manual-ssot-authority）で整う基盤（ビルド・検索・ハイライト・Pages・スキル生成・CI）をそのまま再利用する。
- 利用者章と相互リンクで接続し二重記述を避ける（例: 起動シーケンスは `reference/startup.md` を利用者向け権威とし、内部章は実装詳細のみ）。
- `SUMMARY.md` の最終パートに置き、見出しで「コントリビュータ向け」と明示して読者を分ける。

**既存文書の扱い**:

| 文書 | 扱い |
|---|---|
| `OPTIMIZATION.md` | 吸収して廃止（吸収時に現行実装と照合し最新化） |
| スキル `internal-modules.md` | mdBook 内部章を権威にし、スキルへ生成（manual-ssot-authority の生成機構に乗せる） |
| 各クレート `README.md` | 存続（crates.io の顔）。詳細な内部解説部分のみ mdBook へ移しリンク化 |
| steering `tech.md` / `structure.md` | 対象外（AI プロジェクトメモリ。必要ならリンクのみ） |
| `SOUL.md` / `TEST_COVERAGE.md` | 対象外 |

**鮮度維持（a＋b 併用）**:
- a: `kiro-complete` の DoD に「本 spec が内部設計章の対象領域に触れたなら該当章を更新したか」の確認項目を追加（日常の追従）。
- b: `review-improvement-loop` の次元⑦（ドキュメント/依存整合）に内部設計章と実装の照合を含める（定期総点検）。

却下: 別 mdBook（ビルド/CI 二重化・横断リンク弱化）、非公開 `doc/internals/`（doc/spec 廃止の流れに逆行）、鮮度維持の仕組みなし（OPTIMIZATION.md の二の舞）。

## Scope

- **In**:
  - mdBook「内部設計」パートの新設と執筆。対象題材（網羅範囲の確定は要件フェーズ）:
    - トランスパイルパイプライン（パース → 2 パストランスパイル → Lua コード生成・最適化）
    - シーン/単語レジストリとシーン検索
    - ランタイム実行モデル（yield-resume コルーチン・`co_scene`・ACT/STORE）
    - ローダ自己展開・モジュール解決（lua-require-robustness 後の起動シーケンス）
    - SHIORI 層（FFI・非同期トーク・アクターランタイム・CH marshaling・presentation event stream）
    - デバッグ基盤（DAP バックエンド・ソースマップ）とシーンキック
  - `OPTIMIZATION.md` の吸収・廃止
  - `internal-modules` の mdBook 権威化とスキル生成への移行
  - クレート README の内部解説部分の移設とリンク化
  - `kiro-complete` DoD への追従確認項目追加、`review-improvement-loop` 次元⑦への照合追加
- **Out**:
  - 利用者向け章の改訂（manual-ssot-authority・既存章の領域）
  - コード・挙動の変更（記述のみ）
  - steering の再編

## Boundary Candidates

- 内部設計章の執筆（題材ごとに独立して書ける）
- 既存内部文書の集約（OPTIMIZATION.md・internal-modules・クレート README）
- 鮮度維持の運用組み込み（kiro-complete・review-improvement-loop）

## Out of Boundary

- mdBook 権威化・生成機構そのもの（manual-ssot-authority が提供）
- 利用者向けの API 使用法（既存 `lua/` 章・manual-ssot-authority で拡充）

## Upstream / Downstream

- **Upstream**: `manual-ssot-authority`（mdBook 権威方式と生成機構・`internal-modules` の申し送り）、`pasta-user-manual`
- **Downstream**: 以後の全 spec（完了ゲートで内部設計章の追従義務を負う）

## Existing Spec Touchpoints

- **Extends**: `review-improvement-loop`（次元⑦に照合項目を追加）
- **Adjacent**: `manual-ssot-authority`、完了済みの各実装 spec（記述対象・読み取り専用）

## Constraints

- mdBook 基盤（静的 HTML+JS）と manual-ssot-authority の生成機構に従う。追加エコシステム依存なし。
- 記述は現行実装に整合させる（設計当時の design.md ではなくコードを正とする）。
- crates.io に表示されるクレート README は単体で読める状態を保つ。
