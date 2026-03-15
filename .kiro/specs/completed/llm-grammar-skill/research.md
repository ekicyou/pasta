# Research & Design Decisions: llm-grammar-skill

## Summary
- **Feature**: `llm-grammar-skill`
- **Discovery Scope**: Simple Addition（純粋なドキュメント成果物、コード変更なし）
- **Key Findings**:
  - VS Code Copilot Skill のトリガーは YAML Frontmatter の `description` フィールド内 USE FOR / DO NOT USE FOR キーワードで制御される
  - 既存ソース（`steering/grammar.md` 約200行 + サンプルゴースト約160行）から、300〜500行の自己完結SKILL.mdに収まると推定
  - `.agents/` ディレクトリはgit追跡対象とし、別リポジトリへのコピーを容易にする

## Research Log

### VS Code Copilot Skill ファイル形式
- **Context**: Req 1（スキルファイル構造）の実装仕様を確定するため、既存スキルの実例を調査
- **Sources Consulted**: ユーザーのグローバル設定 `~/.agents/skills/azure-resource-lookup/SKILL.md` 等
- **Findings**:
  - YAML Frontmatter は `name`, `description`, `license`, `metadata`（author, version）の4キー構成
  - `description` フィールドに USE FOR / DO NOT USE FOR キーワードリストを直接埋め込む（長文1行）
  - 本文は Markdown 形式で、When to Use / Quick Reference / Workflow 等のセクション構成
  - 1ファイル完結が標準パターン（補助ファイルのコンテキスト注入は不確実）
- **Implications**: Option A（単一ファイル）が標準パターンに最も合致。description のトリガーフレーズ設計が呼び出し精度に直結

### 情報ソースの棚卸しと密度設計
- **Context**: Req 2（文法リファレンス）の情報量を、LLMコンテキストウィンドウに最適なサイズに圧縮する戦略
- **Sources Consulted**: `steering/grammar.md`、`doc/spec/01-12`、サンプルゴースト `dic/*.pasta`
- **Findings**:
  - `steering/grammar.md`（約200行）: マーカー一覧、ドメイン概念、基本パターン、IR出力、さくらスクリプト → スキルの骨格として転用可能
  - `doc/spec/`（12章）: 権威的仕様だが実装者向け詳細が多い → コード生成に必要な構文ルールのみ抽出
  - サンプルゴースト `dic/*.pasta`（4ファイル、約160行）: 実証済みパターン → そのままパターン集に活用
  - IR出力（ScriptEvent）やPest文法定義は LLM のコード生成には不要 → 除外
- **Implications**: 以下の情報階層で圧縮する:
  1. マーカー一覧表（クイックリファレンス）
  2. 構文ルール（シーン・アクション行・単語・変数・Call・Lua・コメント・属性）
  3. アクター辞書・さくらスクリプト
  4. プロジェクト構造概要
  5. イベントマッピング
  6. 辞書制作パターン（実例ベース）

### SHIORI イベントディスパッチの作者向け簡略化
- **Context**: Req 5（イベントマッピング）をゴースト作者視点でどこまで説明するか
- **Sources Consulted**: `pasta_lua/scripts/pasta/shiori/entry.lua`, `virtual_dispatcher.lua`, `scene.lua`
- **Findings**:
  - ゴースト作者に必要な知識: 「シーン名 = イベント名にすれば自動的に呼ばれる」という1ルール
  - 仮想イベント（OnTalk, OnHour）は OnSecondChange 経由だが、作者には「`＊OnTalk` と書けばランダムトークになる」で十分
  - 4段階フォールバック検索の内部実装詳細は不要
- **Implications**: イベントマッピングは実用テーブル（自然言語の意図→シーン名）として設計し、内部ディスパッチ機構は1〜2行の概要に留める

### .gitignore と配布戦略
- **Context**: `.agents/` ディレクトリの git 追跡方針
- **Findings**:
  - 現在 `.gitignore` に `.agents/` は含まれていない
  - スキルは「別リポジトリにコピー」して使う前提のため、pastaリポジトリでも git 追跡対象とすべき
  - これにより、スキルの変更履歴管理とリリースが可能になる
- **Implications**: `.gitignore` への追加は不要。通常のファイルとして追跡

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 単一SKILL.md | 全情報を1ファイルに統合 | 標準パターン合致、コンテキスト注入効率最高 | ファイルサイズ（推定300-500行） | **採用** |
| B: SKILL.md + 補助ファイル | メタ+概要と詳細を分割 | ファイル管理容易 | 補助ファイルの自動注入不確実 | 不採用 |
| C: 外部参照方式 | 既存ドキュメントへの参照 | 重複最小 | 自己完結制約（1.5）に違反 | 除外済み |

## Design Decisions

### Decision: SKILL.md のセクション構成
- **Context**: 6つの要件を単一ファイルにどう配置するかの情報アーキテクチャ設計
- **Alternatives Considered**:
  1. 要件順（Req 1→6）に並べる — 論理的だがコード生成ワークフローと不一致
  2. 参照頻度順に並べる（マーカー表→パターン集→イベント→構造） — LLMの参照効率が高い
- **Selected Approach**: 参照頻度順。LLM がコード生成時に最も頻繁に参照する情報を先頭に配置
- **Rationale**: スキルが LLM コンテキストに注入される際、先頭の情報ほど参照されやすい。マーカー表と構文ルールが最高頻度
- **Trade-offs**: 要件との対応は非線形になるが、トレーサビリティ表で補完可能

### Decision: トリガーフレーズ言語方針
- **Context**: USE FOR / DO NOT USE FOR に日本語・英語どちらを使うか
- **Selected Approach**: 日英併記。日本語ゴースト開発者の自然言語入力（日本語）と、英語キーワード（Pasta DSL, ghost, SHIORI等）の両方でトリガーされるようにする
- **Rationale**: 開発者は「トークを作って」と日本語で指示するが、技術用語は英語（Pasta DSL, .pasta）のため、両方をカバー

### Decision: Luaコードブロック説明の深さ
- **Context**: Req 2.9 — サンプルゴーストにはLuaブロック未使用だが、機能としては存在
- **Selected Approach**: 最小限（記述方法と基本的な制約のみ、3〜5行程度）
- **Rationale**: 辞書制作サポートが目的であり、Pasta DSL構文のみで十分なケースが大半。Luaブロックは高度ユースケース向けのエスケープハッチとして言及に留める

## Risks & Mitigations
- **Risk**: SKILL.md が500行を超え、LLM コンテキストウィンドウの効率が低下する → **Mitigation**: 実装時にパターン例を必要最小限に絞り、冗長な説明を表形式で圧縮する
- **Risk**: トリガーフレーズが広すぎて無関係なコンテキストで呼び出される → **Mitigation**: DO NOT USE FOR に明確な除外キーワードを設定（pasta料理、Rustクレート開発等）
- **Risk**: `doc/spec/` の更新とスキルの乖離 → **Mitigation**: Req 6.4 により `doc/spec/` 優先の修正ルールを明文化済み。文法変更頻度は低い（Phase 0完了済み）

## References
- VS Code Copilot Skill の実例: `~/.agents/skills/azure-resource-lookup/SKILL.md`
- `steering/grammar.md` — AI向け完全参照（スキル骨格のソース）
- `doc/spec/01-12` — 権威的仕様書（正確性の保証元）
- `pasta_sample_ghost/dist-src/ghost/master/dic/` — 実証済みパターンのソース
- Gap analysis: `.kiro/specs/llm-grammar-skill/gap-analysis.md`
