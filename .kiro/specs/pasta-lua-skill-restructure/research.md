# Research & Design Decisions: pasta-lua-skill-restructure

## Summary
- **Feature**: `pasta-lua-skill-restructure`
- **Discovery Scope**: Extension（既存ドキュメント資産の再編成）
- **Key Findings**:
  1. 既存スキルはすべてSKILL.md単体構成。references/パターンは初の導入
  2. VS Code Copilot Skillsの仕様上、SKILL.mdのYAMLフロントマターのみ自動検出。references/内ファイルはSKILL.md内のread_file指示でロードされる
  3. 情報ソース（LUA_API.md 1160行 + lua-coding.md 695行 = 1855行）から5つのリファレンスファイルへの統合・リッチ化が必要

## Research Log

### VS Code Copilot Skills のファイルロード仕様
- **Context**: references/ディレクトリのファイルをAIエージェントがどのように認識・ロードするかの確認
- **Sources Consulted**: 既存スキル3件（karpathy-guidelines, pasta-ghost-authoring, pasta-lua-coding）のディレクトリ構成
- **Findings**:
  - SKILL.mdのYAMLフロントマター（`name`, `description`）のみが自動検出される
  - `description`内のUSE FOR / DO NOT USE FORキーワードでスキル起動を判定
  - SKILL.md本文内で「このファイルをread_fileで読め」と指示すれば、エージェントは追加ファイルをロード可能
  - references/パターンは前例なし。本仕様が初の導入事例
- **Implications**: SKILL.md内にReferencesインデックスセクションを設け、各ファイルのパス・概要・ロード指示を明記する必要がある

### 既存情報ソースの構造分析
- **Context**: references/ファイルへの内容マッピングを確定するため、ソースの構造を分析
- **Sources Consulted**: SKILL.md (588行), LUA_API.md (1160行), lua-coding.md (695行)
- **Findings**:
  - **LUA_API.md**: 9セクション構成（モジュールカタログ、@pasta_search、@pasta_persistence、@enc、@pasta_config、@pasta_sakura_script、finalize_scene、mlua-stdlib、SHIORIイベント）
  - **lua-coding.md**: 7セクション構成（命名規約、モジュール構造、クラス設計、EmmyLua型注釈、エラーハンドリング、内部モジュール、テスト）
  - **SKILL.md**: 上記2つの要約版（各セクションの要約率20-51%）
  - §1 Purpose / §2 Quick Reference は独自コンテンツ（ソースにない付加価値）
- **Implications**: 5リファレンスファイルへのマッピングが明確に定義可能

### リファレンスファイルへの内容マッピング
- **Context**: R3（5ファイル分割）のためのソース→ターゲットマッピング
- **Findings**:

| リファレンスファイル | SKILL.mdセクション | LUA_API.mdセクション | lua-coding.mdセクション | 想定行数 |
|---------------------|-------------------|---------------------|----------------------|---------|
| runtime-api.md | §4 (120行) | §2-§6, §8 (590行) | — | 300-400行 |
| internal-modules.md | §5 (115行) | §7 (91行) | §6 (164行) | 250-350行 |
| shiori-handlers.md | §6 (90行) | §9 (322行) | — | 250-350行 |
| coding-conventions.md | §3 (95行) | — | §1-§5 (374行) | 300-400行 |
| testing-lint.md | §7 (60行) | — | §7 (117行) | 120-180行 |

### 相互リファレンスリンク設計
- **Context**: R4-AC3（ファイル間クロスリファレンス）の実現方式
- **Findings**:
  - 相対パスリンク: `[ACTオブジェクト](internal-modules.md#act-オブジェクト)` 形式
  - スキルフォルダごとコピーする運用のため、相対パスで自己完結
  - Markdownアンカーは日本語見出しにも対応（GitHub Flavored Markdown準拠）
- **Implications**: ファイル内の見出しID（アンカー）を安定させる命名規則が必要

### レガシー文書の廃止手順
- **Context**: R6（旧権威ドキュメントの統合・廃止）の実行手順
- **Sources Consulted**: SOUL.md (line 24), GRAMMAR.md (line 753)
- **Findings**:
  - SOUL.md: `- [pasta_lua/LUA_API.md](crates/pasta_lua/LUA_API.md) - Lua APIリファレンス` → skillリファレンスへのリンクに変更
  - GRAMMAR.md: 同上
  - LUA_API.md: 完全削除（references/が権威文書を引き継ぐ）
  - lua-coding.md: 完全削除（スキルのUSE FORキーワードによりオンデマンド発見可能なため、steering常時ロードは不要）
- **Implications**: SOUL.md/GRAMMAR.mdのリンク先はスキルディレクトリのReferencesインデックスを指す

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| スキル単体維持 | SKILL.md内で全情報を保持 | 単純、前例あり | 588行→500行制約違反、リッチ化不可 | R1/R4未達 |
| references/分離（Extract & Link） | SKILL.mdを要約化、references/に詳細移動 | 工数小、500行クリア | references/が薄い | R4未達 |
| references/リッチ化分離（**採用**） | 要約化 + ソースからのリッチ化 | 全要件達成、自己完結 | 工数中（3-5日） | gap-analysis推奨 |

## Design Decisions

### Decision: リファレンスファイル命名規則
- **Context**: R3で定義された5ドメインに対するファイル名の確定
- **Alternatives Considered**:
  1. 番号プレフィックス（`01-runtime-api.md`）— 読み順を強制
  2. 番号なし英語ケバブケース（`runtime-api.md`）— アルファベット順で自然にソート
- **Selected Approach**: 番号なし英語ケバブケース
- **Rationale**: 参照時のタイプ量削減、新規ファイル追加時に番号振り直し不要
- **Trade-offs**: 明示的な読み順がないが、SKILL.md内のReferencesインデックスが順序を提供

### Decision: SKILL.md内のリファレンスロード指示方式
- **Context**: AIエージェントにreferences/ファイルのロードを促す方法
- **Alternatives Considered**:
  1. 各要約セクション末尾にread_file指示を埋め込む
  2. Referencesインデックスセクションに集約する
  3. 両方（セクション末尾リンク + インデックス）
- **Selected Approach**: 3. 両方
- **Rationale**: セクション末尾のリンクは「今すぐ詳細が必要」なケースに対応。インデックスは「全体を見渡して選びたい」ケースに対応。冗長だが、エージェントの自律的判断を支援する
- **Trade-offs**: 若干の行数増加（各セクションに1行追加 × 5セクション = 5行）

### Decision: 相互リファレンスのアンカー命名規則
- **Context**: ファイル間クロスリファレンスの安定性確保
- **Selected Approach**: 英語ケバブケースの明示的IDは使わず、Markdown見出しの自然なアンカーを使用
- **Rationale**: GFMの自動アンカー生成で十分。日本語見出しもアンカー化される（例: `#act-オブジェクト`）。明示的ID管理は保守コストが高い
- **Trade-offs**: 見出し変更時にリンク切れリスクあり。ただし同一スキル内の5ファイルなので影響範囲は限定的

### Decision: lua-coding.md の廃止方式
- **Context**: steering/lua-coding.md（695行）をreferences/統合後にどう扱うか
- **Alternatives Considered**:
  1. 3行のリダイレクトに置換 — steeringとして残り、他の場所を探す手間を排除
  2. 完全削除 — steeringの常時ロードコストを完全排除
- **Selected Approach**: 2. 完全削除
- **Rationale**: steeringファイルは全タスクで常時コンテキストにロードされる。3行のリダイレクトでもLua非関連タスクのコンテキストを浪費する。スキルのUSE FORキーワード（`pasta lua`, `Luaスクリプト`, `Lua API`等）によりエージェントはオンデマンドで`pasta-lua-coding`スキルを発見可能なため、steeringレベルの誘導は不要
- **Trade-offs**: 過去にlua-coding.mdを直接参照していたエージェントは見つけられなくなるが、SKILL.mdのUSE FORキーワードで代替される

### Decision: §3-§7要約の必須キーワード基準
- **Context**: SKILL.md §3-§7の3-5行要約に「何を含めるべきか」の判断基準
- **Selected Approach**: 各セクションの必須キーワード一覧をdesign.mdに明記
- **Rationale**: 「主要キーワードを含める」という抽象的な指示では実装時に判断がブレる。エージェントがロード判断に必要な最低限の情報を事前定義することで、実装品質のばらつきを防止
- **Trade-offs**: 設計が実装の詳細に踏み込む形になるが、ドキュメント再編成タスクでは実装詳細 ≈ コンテンツ判断のため妥当

## Risks & Mitigations
- **リスク1**: リファレンスファイル間のリンク切れ — 見出し変更時に発生しうる。**軽減策**: 実装完了時にリンク検証タスクを追加
- **リスク2**: SKILL.md要約が不十分でエージェントが適切なリファレンスを選択できない — **軽減策**: 各要約セクションにキーワード（API名、モジュール名）を含める
- **リスク3**: レガシー文書削除後に外部参照が残る — **軽減策**: R6で明示的にSOUL.md/GRAMMAR.mdのリンク更新を要件化済み

## References
- `.agents/skills/pasta-lua-coding/SKILL.md` — 現行スキル定義（588行）
- `crates/pasta_lua/LUA_API.md` — 現行権威APIリファレンス（1160行、削除予定）
- `.kiro/steering/lua-coding.md` — 現行コーディング規約（695行、削除予定）
- `.kiro/specs/pasta-lua-skill-restructure/requirements.md` — 要件定義書
- `.kiro/specs/pasta-lua-skill-restructure/gap-analysis.md` — ギャップ分析レポート
