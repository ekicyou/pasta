# 要件定義書: lsp-spec-conformance

> **バージョン**: v1（2026-03-12）
> **関連完了仕様**: `pasta-cue-dsl-extension`（Cue コマンド DSL 拡張）
> **スコープ**: pasta_lsp クレートおよび VSCode 拡張のキューコマンド対応を中心とした仕様追従

## イントロダクション

`pasta-cue-dsl-extension` 仕様により、pasta_dsl にキューコマンド行（`!` / `！`）の構文解析と `CueCommandNode` AST が追加された。pasta_lsp（LSP サーバー）および VSCode 拡張は最低限の match arm 追加（OPERATOR トークン1つ）で対応しているが、以下の点で仕様への追従が不十分である：

1. **セマンティックトークンの粒度不足** — キューコマンド行全体を単一 OPERATOR トークンとして扱っており、マーカー(`!`)・コマンド名・スコープ(`@name`)・引数の個別トークン化が未実装
2. **専用トークンタイプの欠如** — キューコマンドに適切なトークンタイプが定義されていない
3. **TextMate 文法の未対応** — VSCode 拡張の `pasta.tmLanguage.json` にキューコマンド行のパターンが存在しない
4. **テストカバレッジのギャップ** — LSP 統合テストにキューコマンド関連テストが存在しない

### 実装完了の定義

1. キューコマンド行の各構成要素（マーカー・コマンド名・スコープ・引数）が個別のセマンティックトークンとして生成される
2. VSCode 拡張の TextMate 文法でキューコマンド行が基本的にハイライトされる
3. キューコマンド行のパースエラーが LSP Diagnostics として適切に報告される
4. 既存テストにリグレッションがない

### 責務境界

以下は本仕様のスコープ外とする：
- コマンド名の意味解釈・バリデーション（dola 側の責務）
- キューコマンドの補完候補提案（将来仕様）
- キューコマンドの Hover 情報提供（将来仕様）
- Go to Definition / Find References（将来仕様）

---

## 要件

### Requirement 1: キューコマンド行のセマンティックトークン細分化

**Objective:** As a VSCode 拡張利用者, I want キューコマンド行の各構成要素が個別にハイライトされること, so that コマンド名・スコープ・引数を視覚的に区別でき、スクリプトの可読性が向上する

#### Acceptance Criteria

1. When `!command` 形式のキューコマンド行が解析された場合, the pasta_lsp shall マーカー（`!` / `！`）と コマンド名をそれぞれ独立したセマンティックトークンとして生成する
2. When `!command@scope` 形式のキューコマンド行が解析された場合, the pasta_lsp shall マーカー・コマンド名・`@` 記号・スコープ識別子をそれぞれ独立したセマンティックトークンとして生成する
3. When `!command@scope(arg1, arg2)` 形式のキューコマンド行が解析された場合, the pasta_lsp shall マーカー・コマンド名・スコープ・括弧・各引数をそれぞれ独立したセマンティックトークンとして生成する
4. When 引数に文字列リテラル（`"..."` / `「...」`）が含まれる場合, the pasta_lsp shall 文字列リテラル用のトークンタイプで生成する
5. When 引数に数値リテラルが含まれる場合, the pasta_lsp shall 数値リテラル用のトークンタイプで生成する
6. When 引数に `@` 参照が含まれる場合, the pasta_lsp shall 単語参照用のトークンタイプで生成する
7. The pasta_lsp shall 全角マーカー（`！`）と半角マーカー（`!`）で同一のトークン生成結果を返す

### Requirement 2: キューコマンド専用トークンタイプの追加

**Objective:** As a テーマ作成者 / 拡張開発者, I want キューコマンドに専用のセマンティックトークンタイプが割り当てられていること, so that カスタムテーマでキューコマンドの配色を他の構文要素と独立して制御できる

#### Acceptance Criteria

1. The pasta_lsp shall キューコマンドのマーカー（`!` / `！`）に対して専用のトークンタイプ（`cueMarker` または既存の適切なタイプ）を割り当てる
2. The pasta_lsp shall キューコマンドのコマンド名に対して適切なトークンタイプを割り当てる（具体的なタイプは設計判断 D2 で決定）
3. The pasta_lsp shall `SemanticTokensLegend` にキューコマンド用トークンタイプを登録する
4. The pasta_lsp shall VSCode 拡張の `package.json` でキューコマンド用セマンティックトークンタイプのデフォルトカラーマッピングを宣言する（必要な場合）

### Requirement 3: TextMate 文法のキューコマンド行対応

**Objective:** As a VSCode 利用者, I want セマンティックトークンが無効な場合でもキューコマンド行が基本的にハイライトされること, so that TextMate 文法ベースのフォールバックハイライトでもスクリプトが読みやすい

#### Acceptance Criteria

1. The VSCode 拡張 shall `pasta.tmLanguage.json` にキューコマンド行パターンを追加し、`!` / `！` で始まる行を認識する
2. The VSCode 拡張 shall キューコマンドマーカー（`!` / `！`）に `keyword.other.marker.pasta` スコープを割り当てる
3. The VSCode 拡張 shall キューコマンド名に `entity.name.function.cue.pasta` スコープを割り当てる
4. The VSCode 拡張 shall キューコマンドの `@scope` 部分に参照用スコープを割り当てる
5. The VSCode 拡張 shall キューコマンドの括弧と引数を適切なスコープで装飾する

### Requirement 4: テストカバレッジの確保

**Objective:** As a 開発者, I want キューコマンド行の LSP サポートが十分なテストで裏付けられていること, so that 将来の変更でリグレッションが発生した場合に即座に検出できる

#### Acceptance Criteria

1. The pasta_lsp shall キューコマンド行（4 形式: `!id` / `!id@scope` / `!id(args)` / `!id@scope(args)`）のセマンティックトークン生成を検証する統合テストを持つ
2. The pasta_lsp shall 全角マーカー（`！`）のキューコマンド行が半角マーカー（`!`）と同一のトークンタイプを生成することを検証するテストを持つ
3. The pasta_lsp shall キューコマンド行を含むシーンと含まないシーンが混在するドキュメントの正しいトークン生成を検証するテストを持つ
4. The pasta_lsp shall キューコマンド行の構文エラー（例: 引数括弧の不一致）が Diagnostics として報告されることを検証するテストを持つ
5. While 既存テスト（79 テスト）がすべてパスしている場合, the pasta_lsp shall 新規テスト追加後もリグレッションなくパスする

### Requirement 5: 後方互換性の維持

**Objective:** As a 既存 VSCode 拡張利用者, I want キューコマンドを使用しない既存の pasta スクリプトのハイライトが変わらないこと, so that 拡張更新時に既存の作業環境が損なわれない

#### Acceptance Criteria

1. The pasta_lsp shall 既存の 15 トークンタイプ（インデックス 0-14）のインデックス値を変更しない
2. The pasta_lsp shall 新しいトークンタイプを TOKEN_TYPES 配列の末尾に追加する
3. The VSCode 拡張 shall 既存の TextMate 文法パターンの正規表現を変更しない
4. The VSCode 拡張 shall 新しいキューコマンドパターンを既存パターンとの優先度衝突が起きない位置に挿入する

---

## 設計判断事項（設計フェーズで解決）

| ID  | 項目                                   | 概要                                                             |
| --- | -------------------------------------- | ---------------------------------------------------------------- |
| D1  | キューコマンドマーカーのトークンタイプ | 新規 `cueMarker` 追加 vs 既存 `keyword` 相当の再利用             |
| D2  | コマンド名のトークンタイプ             | `function` 相当 vs `method` 相当 vs 新規 `cueCommand`            |
| D3  | visit_cue_command の実装方式           | Span ベーステキストスキャン vs CueCommandNode フィールド直接利用 |
| D4  | TextMate 文法の挿入位置                | `action-line` の前 vs 後、patterns 配列内の順序                  |
| D5  | ScopedName の actor:name 分割粒度      | `@actor:name` 形式で actor/name を別トークンにするか全体で1トークンか |

---

## 影響範囲

| コンポーネント                                  | 変更内容                                     | 影響度 |
| ----------------------------------------------- | -------------------------------------------- | ------ |
| `pasta_lsp/src/analysis/token_types.rs`         | トークンタイプ追加                           | 小     |
| `pasta_lsp/src/analysis/visitors.rs`            | `visit_cue_command` 細分化実装               | 中     |
| `editors/vscode/syntaxes/pasta.tmLanguage.json` | キューコマンドパターン追加                   | 小     |
| `editors/vscode/package.json`                   | セマンティックトークン設定追加（必要な場合） | 小     |
| `pasta_lsp/tests/`                              | 新規テストファイル追加                       | 中     |
| `pasta_lsp/README.md`                           | トークンタイプ表の更新                       | 小     |
