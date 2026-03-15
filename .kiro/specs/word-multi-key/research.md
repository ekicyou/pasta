# Research & Design Decisions: word-multi-key

## Summary
- **Feature**: `word-multi-key`
- **Discovery Scope**: Extension（既存PEG文法・AST・トランスパイラの拡張）
- **Key Findings**:
  - `actors` ルールが複数要素カンマ区切りの先行パターンとして存在し、`key_list` ルールの設計に直接転用可能
  - Option C（`names: Vec<String>` + `name()` ヘルパー）が一貫性・移行容易性・pasta_lua対応の全バランスで最適
  - `pasta_lsp` の `visit_keywords` は `word.span` のみ使用しており変更不要だが、将来のセマンティックハイライト拡張でキーごとのSpanが有用

## Research Log

### PEGルール設計: コロン左側のキーリスト
- **Context**: `key_words` ルールでコロン左側に複数キーを受け付ける文法をどう定義するか
- **Sources Consulted**: `grammar.pest` 内の `actors` ルール、`words` ルール、`comma_sep` 定義
- **Findings**:
  - `actors = { actors_item ~ ( comma_sep ~ actors_item )* ~ comma_sep? }` が複数要素カンマ区切りの先行例
  - `comma_sep = { s ~ comma ~ s }` / `comma = { "、" | "，" | "," }` で全角・半角両対応済み
  - コロン（`kv_marker`）が明確な境界となるため、キー区切りカンマと値区切りカンマの曖昧性は発生しない
  - 末尾カンマ（trailing comma）はキーリストでは不要（`actors` は末尾許容だが、キーリストで `＠key1、：values` は意味をなさない。PEG `id` が空文字を拒否するため自動排除される）
- **Implications**: `key_list = { id ~ ( comma_sep ~ id )* }` で十分。末尾カンマ非対応。

### PEGルール命名
- **Context**: 新規ルール名の選定
- **Sources Consulted**: `grammar.pest` 全体の命名規約調査
- **Findings**:
  - キーバリューペア系: `key_literal`, `key_expr`, `key_words`, `key_attr` → `key_` プレフィクス
  - リスト系: `words`, `actors`, `attrs` → 複数形名詞
  - `key_list` は `key_` プレフィクスに準拠しつつリスト性を示す
  - `word_keys` は `word_` プレフィクス（値側の命名空間）と衝突する可能性
  - `multi_key` は既存命名パターンに合わない
- **Implications**: `key_list` を採用。`key_` プレフィクス系列に一貫。

### AST設計: Option A / B / C の最終評価
- **Context**: `KeyWords` 構造体のフィールド設計
- **Sources Consulted**: gap-analysis.md、既存AST型パターン、pasta_lua側の参照箇所7箇所
- **Findings**:

  | 評価軸 | Option A (`names`) | Option B (`aliases`) | Option C (`names` + `name()`) |
  |---|---|---|---|
  | 内部一貫性 | ◎ 全キー等価 | △ 非対称 | ◎ 全キー等価 |
  | pasta_lua移行コスト | × 7箇所+テスト変更 | ◎ 既存コード不変 | ○ 7箇所の機械的変更 |
  | イテレーション自然さ | ◎ `names.iter()` | × `once(&name).chain(aliases)` | ◎ `names.iter()` |
  | 後方互換性 | × フィールド名変更 | ◎ フィールド追加のみ | ○ `name` → `name()` |
  | セマンティック正確性 | ◎ 全キーが対等 | △ 最初のキーが特別 | ◎ 全キーが対等 |
  | コンパイルエラー検出 | ◎ 未対応箇所が即座にエラー | × 対応漏れが検出できない | ◎ フィールド→メソッド変更でエラー |

- **Implications**: 
  - Option Bはpasta_luaスコープ内の場合、「対応漏れが検出できない」リスクがある（`kw.name`が引き続きコンパイル通るため、新キーの登録漏れに気づけない）
  - Option Cは`kw.name` → `kw.name()`の変更でコンパイルエラーが発生し、対応すべき箇所を強制的に洗い出せる
  - Option Aは最もシンプルだが、`names[0]`アクセスが冗長で`name()`ヘルパーが結局必要になる

### キーごとのSpan保持
- **Context**: 各キーに個別のSpanを持たせるか
- **Sources Consulted**: `pasta_lsp/src/analysis/visitors.rs`、`Span`型定義
- **Findings**:
  - 現在の `visit_keywords` は行全体の `word.span` でセマンティックトークンを生成
  - キーごとのSpanがあれば、将来LSPで各キー名をホバーした際に個別のハイライトが可能
  - ただし現時点ではLSPはキー名を個別に認識しておらず、行全体を `WORD` トークンとして扱っている
  - PEGの `id` ルールは各マッチに対して `Pair::as_span()` を提供するため、パーサー側でのSpan取得は容易
- **Implications**: キーごとのSpanは将来のLSP拡張に有用だが、本仕様では必須ではない。`names` を `Vec<(String, Span)>` にすると過剰設計。代わりに行全体の `span` を維持し、将来の拡張ポイントとして記録する。

### KeyWords の pub re-export 状況
- **Context**: API境界への影響調査
- **Sources Consulted**: `pasta_dsl/src/parser/ast/mod.rs`, `pasta_dsl/src/lib.rs`
- **Findings**:
  - `ast/mod.rs` で `pub use action::*` により `KeyWords` が re-export
  - `pasta_lua` は `use pasta_dsl::parser::KeyWords` でインポート
  - フィールド直接アクセス（`kw.name`, `kw.words`）が7箇所で使用されている
  - `KeyWords` のフィールドは `pub` であり、構造体リテラル構築がテストコード1箇所で使用（`comparison_test.rs`）
- **Implications**: フィールド変更は外部クレート（pasta_lua, pasta_lsp）のコンパイルに即座に影響する。Option Cの場合、`name` フィールド消失によりコンパイルエラーが発生し、対応箇所が明示される。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: `names: Vec<String>` | フィールド完全置換 | 最もシンプル、全キー等価 | `names[0]`アクセスが冗長 | `name()`ヘルパー追加で実質Option C |
| B: `aliases: Vec<String>` | フィールド追加 | 後方互換性最大 | 非対称性、対応漏れリスク | pasta_luaスコープ内では不適切 |
| C: `names` + `name()` | ハイブリッド | 一貫性+移行容易性 | メソッド/フィールドの混在 | **推奨**: バランス最良 |

## Design Decisions

### Decision: AST設計 — Option C（`names: Vec<String>` + `name()` ヘルパー）を採用

- **Context**: `KeyWords` 構造体で複数キーをどう表現するか。3つのオプションが要件定義フェーズで検討済み。
- **Alternatives Considered**:
  1. Option A — `name: String` → `names: Vec<String>` に完全置換
  2. Option B — `aliases: Vec<String>` フィールドを追加
  3. Option C — `names: Vec<String>` + `pub fn name() -> &str` ヘルパー
- **Selected Approach**: Option C
- **Rationale**:
  - 全キーが意味的に対等な `Vec<String>` 表現（Option A/Cの利点）
  - `name()` ヘルパーにより、単一キー参照時の利便性維持
  - `kw.name` → `kw.name()` のコンパイルエラーで、pasta_lua側の対応漏れを強制検出
  - `names.iter()` による自然なイテレーションで、pasta_luaの複数キー登録が直感的
  - Option Bの「対応漏れが検出できない」リスクを回避
- **Trade-offs**: メソッド呼び出しとフィールドアクセスの混在（`kw.name()` vs `kw.words`）。ただしRustの慣習として許容範囲。
- **Follow-up**: テストコード（`comparison_test.rs`）の構造体リテラル構築を `names: vec![...]` に更新する必要あり。

### Decision: PEGルール名 — `key_list` を採用

- **Context**: 新規PEGルールの命名
- **Selected Approach**: `key_list`
- **Rationale**: `key_` プレフィクス系列（`key_literal`, `key_expr`, `key_words`）に一貫。リスト性を明示。
- **Trade-offs**: なし。既存命名規約に完全準拠。

### Decision: キーごとのSpan — 本仕様では保持しない

- **Context**: 各キーに個別のSpanを持たせるか
- **Selected Approach**: 保持しない。行全体の `span: Span` を維持。
- **Rationale**: 現在の `pasta_lsp` はキー名を個別に認識しておらず、即座の利用先がない。`names` を `Vec<(String, Span)>` にするとAST構造が複雑化し、pasta_lua側のイテレーションも煩雑になる。
- **Trade-offs**: 将来のLSPセマンティックハイライト拡張時に再検討が必要。ただしその時点で `names` → `Vec<(String, Span)>` への移行は容易（同一仕様パターンの変更で対応可能）。

## Risks & Mitigations
- **Risk 1**: `kw.name` → `kw.name()` の変更によるpasta_lua側のコンパイルエラー多発 — **Mitigation**: コンパイルエラーは意図的（対応箇所の洗い出し）。7箇所の機械的変更で対応完了。
- **Risk 2**: テストコードの構造体リテラル構築（`KeyWords { name: ... }`）のコンパイルエラー — **Mitigation**: `names: vec![...]` への機械的変更。影響箇所は `parse_elements.rs` 1箇所（コンストラクタ）+ `comparison_test.rs` 2箇所 + `transpiler.rs` テスト 5箇所 + `context.rs` テスト 2箇所 = 計10箇所。`actor_code_block_test.rs` の1箇所は `.name` フィールドアクセスであり `.name()` メソッド呼び出しへの変更が必要。
- **Risk 3**: 将来の仕様拡張（動的単語参照 `＠＄変数`）との整合性 — **Mitigation**: 動的単語参照はキー名の解決方法の問題であり、`KeyWords` AST構造とは独立。影響なし。

### Decision: `register_*_words()` ヘルパー関数の削除

- **Context**: 設計レビューで transpiler.rs の単語登録パスに非対称性を発見。GlobalWord と ActorScope は `context.word_registry` を直接呼び出しているのに対し、LocalWord のみ `context.register_local_words()` ヘルパーを経由していた。`register_global_words()` はプロダクションコードで未使用（テストのみ）。
- **Alternatives Considered**:
  1. 設計図のみ修正（コード変更なし）
  2. ヘルパーを追加して全パスをヘルパー経由に統一
  3. ヘルパーを削除して全パスを `word_registry` 直接呼び出しに統一
- **Selected Approach**: 3（ヘルパー削除）
- **Rationale**:
  - ヘルパー関数は引数加工を一切行わない純粋なパススルーであり、抽象化の価値がない
  - 全3パス（GlobalWord, LocalWord, ActorScope）を対称な直接呼び出しに統一することでAPI整合性が向上
  - word-multi-key 実装時に `names.iter()` ループを各パスに直接記述する方が、既存コードパターンと一貫する
- **Trade-offs**: transpiler.rs 内のコードが数行増えるが、間接呼び出しの廃止により見通しが向上
- **Follow-up**: word-multi-key タスクに「ヘルパー削除 + LocalWord パスの直接呼び出し化 + テスト修正」を含める

## References
- [doc/spec/10-words.md](../../../doc/spec/10-words.md) — 単語定義・参照の仕様書
- [doc/spec/11-actor-dictionary.md](../../../doc/spec/11-actor-dictionary.md) — アクター辞書定義の仕様書
- [gap-analysis.md](./gap-analysis.md) — 要件定義フェーズで作成したギャップ分析
