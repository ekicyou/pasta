# Research & Design Decisions

## Summary
- **Feature**: `pasta-lua-skill`
- **Discovery Scope**: Simple Addition（単一SKILL.mdファイルの新規作成）
- **Key Findings**:
  - 情報ソース合計 ~1900行（lua-coding.md ~700行 + LUA_API.md ~1200行）を 400〜600行に圧縮する必要がある
  - 姉妹スキル `pasta-ghost-authoring` (~378行)が構造テンプレートとして確立済み
  - Gap分析で Option A（単一SKILL.md）が推奨済み

## Research Log

### 姉妹スキルの構造分析
- **Context**: 本スキルの構造テンプレートとして姉妹スキルを分析
- **Sources Consulted**: `.agents/skills/pasta-ghost-authoring/SKILL.md`
- **Findings**:
  - YAML Frontmatter: `name`, `description`（USE FOR / DO NOT USE FOR）, `metadata`（author, version）
  - セクション構成: §1 Purpose → §2 Quick Reference → §3 DSL Syntax (3.1-3.9) → §4 Structure → §5 Events → §6 Patterns
  - §2 Quick Reference が凝縮された一覧表で、LLMの即座の参照に有効
  - §3 が最大セクション（構文ルール）で、サブセクション9個
  - §6 Patterns がコード例集で、実践的な使用パターンを提示
- **Implications**: 本スキルも同じ §1-§N 構成を踏襲。Quick Reference を設けてAPI一覧を凝縮提示

### 情報圧縮の方針分析
- **Context**: ~1900行 → 400〜600行への圧縮方針
- **Sources Consulted**: lua-coding.md, LUA_API.md, gap-analysis.md
- **Findings**:
  - lua-coding.md の §1-§3（命名・モジュール構造・クラス設計）: 必須。コード生成品質に直結
  - lua-coding.md の §4-§5（EmmyLua・エラーハンドリング）: 要約レベルで十分。LLMはEmmyLua構文を既知
  - lua-coding.md の §6（pasta固有パターン）: 必須。STORE/ACT/WORD等のパターンはスキル中核
  - lua-coding.md の §7（テスト・Lint）: 必須。Req 6 に直結
  - lua-coding.md の §8（チェックリスト）: 除外。設計ドキュメント向けであり、LLMコンテキストには不適
  - LUA_API.md §1（カタログ）: Quick Reference に統合
  - LUA_API.md §2-§6（各モジュールAPI）: シグネチャ+パラメータ表+最小例に圧縮。冗長な説明・エッジケース除外
  - LUA_API.md §7（finalize_scene）: 呼び出しタイミングと役割のみ。処理フローは除外
  - LUA_API.md §8（mlua-stdlib）: 存在とrequireパスの一覧のみ。詳細APIは外部ドキュメント参照
  - LUA_API.md §9（SHIORIイベント）: REG登録パターン+RESモジュール+主要イベントのreference表。フロー図は除外
- **Implications**: 各セクションの圧縮率を明確にし、設計文書で上限行数を指定

### DSL vs Lua の判断基準分析
- **Context**: Req 1 AC7 — Luaでの実装が適切なケースの判断基準
- **Sources Consulted**: requirements.md, gap-analysis.md
- **Findings**:
  - DSLが得意: シーン定義、基本的な単語定義（少数エントリ）、変数操作、アクター設定
  - Luaが得意: 大量データ投入（ループ/外部ファイル読み込み）、条件分岐ロジック、カスタム計算、外部データ変換
  - 判断基準: 「DSLの宣言的記法では表現が冗長/不可能な場合にLuaを使う」
- **Implications**: §1 Purpose 内に判断基準を簡潔に記載

### scripts/ フォルダの配置規約分析
- **Context**: Req 1 AC8 — 独自Luaスクリプトの配置先
- **Sources Consulted**: pasta_sample_ghost dist-src 構造, lua-coding.md
- **Findings**:
  - `scripts/` フォルダ: ゴーストディレクトリ直下に配置
  - `main.lua`: ユーザースクリプトのエントリーポイント（自動実行）
  - パターン: `scripts/` 配下に `.lua` ファイルを配置し、`main.lua` から `require` する
  - pasta.toml の `[loader]` セクションで読み込みパターン設定可能
- **Implications**: §1 に scripts/ の位置づけを簡潔に説明

## Design Decisions

### Decision: 単一 SKILL.md ファイル構成
- **Context**: Gap分析 Option A vs Option B
- **Alternatives Considered**:
  1. Option A — 単一 SKILL.md に全情報集約（400〜600行）
  2. Option B — SKILL.md + API_REFERENCE.md + PATTERNS.md 分割
- **Selected Approach**: Option A（単一ファイル）
- **Rationale**: 姉妹スキルとの構造的一貫性、コピー運用の容易さ、VS Code Copilot Skill機構が SKILL.md のみを自動ロードする制約
- **Trade-offs**: 行数が多くなるとコンテキストウィンドウ圧迫の可能性あるが、400〜600行は許容範囲
- **Follow-up**: 実装後に行数が600行を超える場合は再検討

### Decision: セクション構成（7セクション構成）
- **Context**: ~1900行の情報ソースをどう体系化するか
- **Alternatives Considered**:
  1. 姉妹スキル準拠の6セクション構成
  2. 要件ベースの7セクション構成（要件1件=1セクション）
  3. 機能レイヤーベースの7セクション構成
- **Selected Approach**: 機能レイヤーベースの7セクション構成
- **Rationale**: 要件は6つだが、Req 1（スキル構造）は §1 Purpose に対応し、残りの5要件がそれぞれ独立セクションに対応。Quick Reference を §2 として挟むことで7セクション。情報の論理的な依存関係（規約→API→モジュール→ハンドラ→テスト）に沿った配置
- **Trade-offs**: 姉妹スキルの6セクションより1つ多いが、カバー範囲が広いため妥当

### Decision: 情報圧縮の原則
- **Context**: Gap① — 情報圧縮制約
- **Alternatives Considered**:
  1. API全文転記（忠実性重視）
  2. シグネチャ+パラメータ表のみ（極限圧縮）
  3. シグネチャ+パラメータ表+最小使用例（バランス型）
- **Selected Approach**: シグネチャ+パラメータ表+最小使用例（バランス型）
- **Rationale**: LLMがコードを生成するには、シグネチャだけでなく使用パターンの例示が必要。ただし複数の例や詳細なエッジケース説明は不要
- **Trade-offs**: やや情報量が多くなるが、コード生成精度の向上が期待できる
- **Follow-up**: 各APIモジュールにつき1つの使用例（3〜5行）に制限

### Decision: EmmyLua / エラーハンドリングの記載レベル
- **Context**: lua-coding.md §4-§5 の扱い
- **Selected Approach**: §3 Coding Conventions 内にルール要約（箇条書き）のみ。コード例は省略
- **Rationale**: LLMはEmmyLua構文を既知。`@module`, `@class`, `@param`, `@return` の使用ルールを列挙すれば十分。エラーハンドリングもガードクローズ・pcall・nilチェックのパターン名を列挙すれば、LLMは正しいコードを生成可能

## Risks & Mitigations
- **行数超過リスク**: 目標400-600行を超える可能性 → 各セクションに上限行数を設定し、実装時に監視
- **情報陳腐化リスク**: LUA_API.md / lua-coding.md の更新にスキルが追従できない → Introduction に情報ソース明記の原則を記載し、更新追従の手がかりを残す

## References
- `.kiro/steering/lua-coding.md` — Luaコーディング規約（~700行）
- `crates/pasta_lua/LUA_API.md` — Lua APIリファレンス（~1200行）
- `.agents/skills/pasta-ghost-authoring/SKILL.md` — 姉妹スキル構造テンプレート（~378行）
- `.kiro/specs/pasta-lua-skill/gap-analysis.md` — ギャップ分析結果
