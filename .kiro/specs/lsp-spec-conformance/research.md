# リサーチ & 設計判断ログ: lsp-spec-conformance

## サマリー
- **機能**: `lsp-spec-conformance`
- **ディスカバリースコープ**: Extension（既存システムの拡張）
- **主要な発見**:
  1. テキストスキャン方式（パターン B）が `visit_var_set` で確立済み — そのまま踏襲可能
  2. `CueCommandNode` の `command` / `args` フィールドに個別 Span がないため、行テキストからのカーソル走査が必須
  3. 既存トークンタイプ 15 個（インデックス 0-14）は末尾追加で後方互換性を完全保証

## リサーチログ

### 既存ビジターパターンの分析
- **背景**: キューコマンド行の細粒度トークン化に最適なアプローチを決定するため
- **調査対象**: `visitors.rs` 内の全ビジターメソッド
- **発見**:
  - **パターン A（Span ベース）**: `visit_attr`, `visit_keywords`, `visit_code_block`, `visit_call_scene` — AST ノードの Span を 1 トークンとして出力
  - **パターン B（テキストスキャン）**: `visit_var_set`, `tokenize_var_set_text`, `tokenize_expr_text` — ソーステキストをカーソル走査して複数トークン生成
  - `visit_action_line` はハイブリッド — アクター名・コロンを個別スキャンし、actions は Span ベース
- **含意**: キューコマンド行は複数のサブトークン（マーカー・コマンド名・スコープ・引数）を持つためパターン B が必須。`visit_var_set` の 60 行程度のテキストスキャンコードが直接的な前例となる

### AST Span 保持状況の確認
- **背景**: パターン A（Span ベース）で対応可能か判断するため
- **調査対象**: `pasta_dsl/src/parser/ast/cue.rs` の全型定義
- **発見**:
  - `CueCommandNode.span` → 行全体（pad 〜 or_comment_eol）の Span を保持 ✅
  - `CueCommandNode.command` → `String` のみ、Span なし ❌
  - `ScopedName.span` → `cue_cmd_scope` の `@name` / `@actor:name` 全体の Span ✅
  - `CueArgToken` → 値のみ、Span なし ❌
- **含意**: マーカーとスコープは Span から直接オフセット計算が可能だが、コマンド名と引数はソーステキストスキャンが必要。テキストスキャン方式で統一するのが最もシンプル

### TextMate 文法パターンの挿入位置
- **背景**: 既存パターンとの優先度衝突を回避するため
- **調査対象**: `pasta.tmLanguage.json` の patterns 配列と各 repository エントリ
- **発見**:
  - patterns 配列の順序: comment → lua-code-block → global-scene → local-scene → attribute → word → variable → call → actor → action-line
  - `action-line` パターン: `^(\\s+)(\\S+?)\\s*([：:])(.*)$` — インデント必須 + `\\S+?` + コロン必須
  - `!` / `！` で始まるキューコマンド行はインデント付きだが、コロンが「ない」ため `action-line` にはマッチしない
  - `call`（`＞` / `>`）の後、`actor`（`％` / `%`）の前に配置すればマーカー文字の重複なし
- **含意**: `call` と `actor` の間に `cue-command` パターンを挿入すれば衝突なし。ただし `action-line` の `\\S+?` は `!` にもマッチする可能性があるが、`action-line` はコロンが後続する必要があるため、キューコマンド行（コロンなし）とは区別される

### VSCode package.json セマンティックトークン設定
- **背景**: 新規トークンタイプの追加方法と既存設定との整合性確認
- **調査対象**: `editors/vscode/package.json` の `semanticTokenTypes` / `semanticTokenScopes`
- **発見**:
  - 既存カスタムトークンタイプ: `scene`, `word`, `call`, `actor`, `actorName`, `talk`, `codeBlock`, `sakuraScript`, `escape`, `number`（10 種）
  - 各カスタムタイプに `superType` が設定されている（テーマフォールバック用）
  - `semanticTokenScopes` で言語固有の TextMate スコープマッピングが定義済み
- **含意**: 新規タイプ `cueMarker` と `cueCommand` を `semanticTokenTypes` 末尾に追加し、`semanticTokenScopes` にマッピングを追加するのみ

### Oniguruma の全角スペース対応
- **背景**: R3.1 で「`\s` が U+3000 全角スペースを含まない」と注記されている
- **調査対象**: Oniguruma 正規表現エンジンの `\s` メタ文字仕様
- **発見**:
  - Oniguruma の `\s` は POSIX 準拠で `[ \t\n\r\f\v]` のみ — U+3000（全角スペース）を含まない
  - 既存の `comment` パターンでは `[\\s\u3000]*` を使用して全角スペースに対応済み
- **含意**: キューコマンドパターンのインデント部も `[\\s\u3000]*` を使用する必要がある

## アーキテクチャパターン評価

| オプション                  | 説明                                      | 強み                           | リスク/制限                        | 備考     |
| --------------------------- | ----------------------------------------- | ------------------------------ | ---------------------------------- | -------- |
| A: 既存コンポーネント拡張   | `visitors.rs` に `visit_cue_command` 追加 | 一貫性が高い、変更ファイル最小 | visitors.rs がさらに膨張（≈830行） | **推奨** |
| B: 新コンポーネント分離     | `analysis/cue_visitors.rs` 新設           | visitors.rs の膨張を抑制       | 結合増、80行では分離メリット薄い   | 不採用   |
| C: Span 追加 + ビジター拡張 | `pasta_dsl` に Span 追加                  | LSP ビジターがシンプル         | スコープ外、破壊的変更             | 不採用   |

## 設計判断

### 決定: D1 — キューコマンドマーカーのトークンタイプ
- **背景**: `!` / `！` マーカーにどのトークンタイプを割り当てるか
- **検討した選択肢**:
  1. 既存 `keyword`（= `NAMESPACE` 相当）を再利用
  2. 新規 `cueMarker` を追加
- **選択**: 新規 `cueMarker` を `TOKEN_TYPES` インデックス 15 に追加
- **理由**: テーマ作成者がキューコマンドマーカーを他のマーカー（シーン、属性等）と独立して配色制御できる。R2.1「専用のトークンタイプを割り当てる」の要件を直接的に満たす
- **トレードオフ**: トークンタイプ数が増加するが、LSP 仕様上の制限はない
- **フォローアップ**: `package.json` に `superType: "keyword"` を設定し、未対応テーマでのフォールバック表示を保証

### 決定: D2 — コマンド名のトークンタイプ
- **背景**: コマンド名（`emote`, `mark`, `yield` 等）にどのトークンタイプを割り当てるか
- **検討した選択肢**:
  1. 既存 `function`（= `WORD` インデックス 4）を再利用
  2. 新規 `cueCommand` を追加
- **選択**: 新規 `cueCommand` を `TOKEN_TYPES` インデックス 16 に追加
- **理由**: キューコマンド名は「単語参照」や「関数呼び出し」とは意味的に異なる（dola 側の演出指示）。独立したトークンタイプにより、テーマでの差別化が可能（R2.2）
- **トレードオフ**: `function` 再利用なら追加不要だが、将来のコマンド補完機能等での識別が困難になる
- **フォローアップ**: `package.json` に `superType: "function"` を設定

### 決定: D3 — visit_cue_command の実装方式
- **背景**: キューコマンド行の細粒度トークン生成にどの方式を採用するか
- **検討した選択肢**:
  1. Span ベーステキストスキャン（パターン B）
  2. CueCommandNode フィールド直接利用（パターン A 拡張）
- **選択**: Span ベーステキストスキャン（パターン B）
- **理由**: `command` と `args` に個別 Span がないため、パターン B が唯一の選択肢。`visit_var_set` / `tokenize_var_set_text` の前例に従い一貫性を維持
- **トレードオフ**: テキストスキャンは Span ベースより脆弱だが、既存のバリデーション済みパターンを踏襲するため信頼性は十分

### 決定: D4 — TextMate 文法の挿入位置
- **背景**: `pasta.tmLanguage.json` の patterns 配列内でのキューコマンドパターン位置
- **検討した選択肢**:
  1. `call` の後、`actor` の前
  2. `actor` の後、`action-line` の前
  3. `action-line` の後（最低優先度）
- **選択**: `call` の後、`actor` の前
- **理由**: マーカー文字 `!` / `！` は他のパターンのマーカー文字（`＞`/`>`/`％`/`%`）と重複しない。`action-line` の前に配置することで、インデント付きキューコマンド行が `action-line` の `\\S+?` パターンより先にマッチする
- **トレードオフ**: なし — 衝突の可能性はゼロ

### 決定: D5 — ScopedName の actor:name 分割粒度
- **背景**: `@actor:name` 形式を actor と name で別トークンにするか全体で 1 トークンにするか。また `@` 記号を OPERATOR として分離するか WORD に含めるか
- **選択**: `@` を含む全体で 1 WORD トークン（`ScopedName.span` をそのまま使用）
- **理由**: (1) `ActionLine` の `WordRef`（`@笑顔` を `@` 込みで 1 WORD として emit）と同一方針 ← 設計の一貫性を最優先、(2) `ScopedName.span` が `@` を含む全体をカバーしており追加走査不要、(3) `@actor:name` の actor/name 分離は意味解析レベル（dola 側）の責務であり LSP のハイライト粒度としては過剰
- **確定（2026-03-12）**: 開発者確認により案B（1トークン）に決定。`@` の OPERATOR 分離は行わない

### 決定: D6 — 括弧・カンマ記号のトークン化
- **背景**: `(` `)` `,` 等の区切り記号を LSP セマンティックトークンとして生成するか
- **検討した選択肢**:
  1. OPERATOR トークンとして生成（`visit_var_set` の括弧トークン化と同様）
  2. TextMate に委譲（セマンティックトークンは値部分のみ）
- **選択**: OPERATOR トークンとして生成
- **理由**: (1) `visit_var_set` / `tokenize_args_text` で括弧の OPERATOR トークン生成が確立済み、(2) セマンティックトークンが有効な場合、TextMate パターンは上書きされるため、括弧をスキップすると無色になる可能性がある、(3) 一貫性の観点で他のビジターと同じ方針が望ましい
- **トレードオフ**: テキストスキャンコードが若干複雑になるが、`tokenize_args_text` のパターンを部分的に再利用可能

## リスクと緩和策
- **リスク 1**: テキストスキャンでマーカー位置の検出が全角/半角で不一致 → 緩和: `visit_var_set` の全角/半角マーカー検出パターンを踏襲し、全角優先で検索
- **リスク 2**: visitors.rs の行数増加（≈830 行）→ 緩和: 既に guideline exception として許容済み。将来の分離は別仕様で検討
- **リスク 3**: TextMate パターンの Oniguruma 全角スペース非対応 → 緩和: `[\\s\u3000]*` パターンを使用（既存の `comment` パターンで実証済み）

## 参考資料
- [LSP Semantic Tokens Specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_semanticTokens) — SemanticTokensLegend, トークンエンコーディング仕様
- [TextMate Grammar Reference](https://macromates.com/manual/en/language_grammars) — スコープ命名規約
- [Oniguruma Regular Expressions](https://github.com/kkos/oniguruma/blob/master/doc/RE) — `\s` メタ文字の仕様
