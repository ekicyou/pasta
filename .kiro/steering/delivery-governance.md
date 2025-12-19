# Delivery Governance（MVP禁止・リグレッション責任の明確化）

<meta>
purpose: LLM が持ち込む「MVP」思考の排除と、要件遵守・テスト無欠の完成基準を明確化
scope: 設計/実装/レビュー/CI の意思決定を統制
updated_at: 2025-12-19
</meta>

---

## 目的
- LLM/実装者が **要件を勝手に矮小化（MVP化）** して「ここまでできたから完成」と判断することを**禁止**する。
- 仕様駆動（Kiro 3-phase: Requirements → Design → Tasks → Implementation）に沿った**完全性**を完成基準とする。
- 既存テストへ**リグレッションが発生した場合は、原因修正の責任が実装者にある**ことを明示し、同一 PR 内で解決する。

---

## 適用範囲
- この steering は **単一ドメイン: Delivery Governance**（完成基準・品質ゲート・回帰責任）に限定する。
- 軽い参照は `.kiro/specs/`（仕様） と `.kiro/steering/`（他の記憶）にのみ許容。

---

## 基本原則
1. **MVP 禁止**: LLM/実装者は「MVP/部分実装/スキャフォールドのみ」を完成と**表明しない**。
2. **要件完全性**: 現行仕様のスコープを **減らさない**。変更は `/kiro-spec-requirements` で合意を得る。
3. **DoD（Definition of Done）**は以下を**同時**に満たす：
   - 仕様の各フェーズが承認済み（Requirements/Design/Tasks/Implementation）。
   - 既存テスト（`tests/`）とサンプル実行（`examples/`）が**全て緑**。
   - ドキュメント/ガイドが更新され、 steering と整合。
4. **回帰（リグレッション）ゼロ宣言**: 既存の合格テストを**壊さない**。壊した場合は **実装者が修正**してからマージ。
5. **パターン優先**: 具体的な運用パターンを示し、全面列挙は避ける（Steering Principles 準拠）。

---

## 完成基準（DoD Gate パターン）
**狙い**: 「十分に動くから完成」の主観を排除し、客観条件でゲートを通過。

- **Spec Gate**: `/kiro-spec-status {feature}` が Requirements/Design/Tasks/Implementation 全て承認済。
- **Test Gate**: `cargo test --workspace` が成功。`tests/` 配下の統合・E2E・パーサ・トランスパイラ・ランタイム系が**無欠**。
- **Doc Gate**: 仕様差分を反映（必要なら `README.md`/`GRAMMAR.md`/関連 docs）。
- **Steering Gate**: 既存 steering と不整合がない（命名/構造/責務分離）。
- **ChangeLog Gate**: 非互換変更があれば、影響範囲と移行手順を明記。

> Note: いずれか 1 つでも未達なら「未完成」。到達前の「MVP 完了宣言」は禁止。

---

## 要件維持（Scope Discipline パターン）
**狙い**: 設計・実装中に要件を暗黙縮小しない。

- **変更提案の経路**: スコープ変更は `/kiro-spec-requirements` で提案→承認→設計反映。
- **勝手な縮小禁止**: 実装フェーズで「今回はここまで」や「将来対応」へ移す判断は不可。
- **タスクの完全性**: `/kiro-spec-tasks` のタスクを**全部**満たす。未着手なら**未完成**扱い。

---

## 回帰時の責任（Regression-First Fix パターン）
**狙い**: 壊した人が直す。品質コストを分散せず即時に吸収。

- **同一PRで修正**: 既存テストが落ちた PR は**直してから**マージ。暫定 skip/ignore は原則不可。
- **原因特定**: 失敗テストの**最小再現**を特定し、根本原因を修正（表層対処禁止）。
- **テスト更新の正当性**: 挙動変更が仕様上正当なら、テストを**先に**更新し、変更理由を明記。
- **責任の明示**: レビューで「誰が壊したか」を問い、**実装者が修正完了**を宣言する。

---

## 変更管理（Change Management パターン）
**狙い**: スコープ外変更や互換破壊を可視化。

- **PR 説明の必須項目**:
  - 目的と仕様リンク（`.kiro/specs/...`）
  - 影響範囲（パーサ/IR/ランタイム/トランスパイラ/stdlib）
  - 既存テストへの影響と**対処**（追加/修正/維持）
  - 完成基準の充足宣言（Spec/Test/Doc/Steering/ChangeLog）
- **コミット規律**: コミットはタスク粒度で分割し、メッセージに対象仕様/テスト名を含める。

---

## LLM 行動制約（Anti-MVP 宣言）
**禁止語彙**（完成の言い換えに使用しない）:
- 「MVP」「部分実装」「スキャフォールドのみ」「簡易対応」「とりあえず動く」

**推奨表現**:
- 「仕様フェーズ承認済み」「全テスト合格」「DoD Gate 通過」「追加タスク待ち（未完成）」

**振る舞い**:
- 段階的 delivery を行う場合は **WIP** と明示し、完成を宣言しない。
- 追加仕様が判明したら `/kiro-spec-requirements` へ戻し、設計/タスクを再合意。

---

## 具体例（Bad → Good）
**Bad**:
> 「MVP としてパーサは 2.1/2.3 の一部のみ対応。全体は後続。」

**Good**:
> 「2.x の必須文法（空白/インデント/ラベル/＠/＄/Call/属性/コメント）を**全て**パース。`tests/` の既存/新設ケースが**全緑**。仕様の未決項目は `/kiro-spec-requirements` に戻して合意待ち（未完成を宣言しない）。」

---

## 運用コマンド（参考）
- 進捗確認: `/kiro-spec-status {feature}`
- フェーズ: `/kiro-spec-init`, `/kiro-spec-requirements`, `/kiro-spec-design`, `/kiro-spec-tasks`, `/kiro-spec-impl`
- 追加検証: `/kiro-validate-gap`, `/kiro-validate-design`, `/kiro-validate-impl`
- テスト実行（例）:
  ```bash
  cargo test --workspace
  cargo test -p pasta -- --nocapture
  ```

---

## まとめ（運用パターン）
- **完成は DoD Gate 通過でのみ宣言**（主観不可）。
- **要件は仕様で合意し変更は仕様から**（暗黙縮小不可）。
- **回帰は実装者が同一 PR で修正**（後回し不可）。
- **Steering と整合しパターンに従う**（全面列挙ではなく原則重視）。

> このファイルは Delivery Governance のプロジェクト記憶であり、仕様や設計を置き換えるものではない。パターンに従えば、新しいコードは既存品質を保ったまま拡張される。