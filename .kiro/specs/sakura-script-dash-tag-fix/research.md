# Research & Design Decisions: sakura-script-dash-tag-fix

## Summary
- **Feature**: `sakura-script-dash-tag-fix`
- **Discovery Scope**: Extension（既存システムの文字クラス拡張）
- **Key Findings**:
  - ukadoc公式タグリスト分析により、不足文字は5文字（`-+*?&`）で確定
  - 4箇所の変更のみで完結し、パイプライン下流はすべてパススルー（変更不要）
  - 文法衝突リスクなし（`local_marker`、`sakura_escape` との競合を検証済み）

## Research Log

### ukadoc公式タグリストの文字クラス分析
- **Context**: 当初スコープは `\-`（ハイフンのみ）だったが、他にも未対応の記号タグが存在する可能性があった
- **Sources Consulted**: https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html
- **Findings**:
  - 現在の文字クラス `[!_a-zA-Z0-9]` でカバーできない5文字を特定
  - `-` (U+002D): `\-`（ゴースト終了）
  - `+` (U+002B): `\+`（ランダム交代）、`\_+`（順次交代）
  - `*` (U+002A): `\*`（選択タイムアウト無効）
  - `?` (U+003F): `\_?`（タグ表示モード）
  - `&` (U+0026): `\&[ID]`（エンティティ参照）
- **Implications**: 文字クラスを `[!\-+*?&_a-zA-Z0-9]` に拡張することで、ukadoc記載の全タグを網羅可能

### Pest PEG文法における記号文字の扱い
- **Context**: Pest文法での `sakura_id` ルール拡張方法の確認
- **Sources Consulted**: `grammar.pest` L171、Pest公式ドキュメント
- **Findings**:
  - Pest では文字クラスは `('a'..'z')` 範囲または `"x"` リテラルの ordered choice で表現
  - 各記号文字は `"-"`, `"+"`, `"*"`, `"?"`, `"&"` として個別に追加
  - `"-"` はPest文字列リテラルなので、正規表現の範囲指定問題は発生しない
- **Implications**: 既存パターンに完全準拠した拡張が可能

### 正規表現における特殊文字の文字クラス内挙動
- **Context**: `tokenizer.rs` の `SAKURA_TAG_PATTERN` regex更新時の安全性確認
- **Sources Consulted**: Rust `regex` クレートドキュメント
- **Findings**:
  - 文字クラス `[]` 内では `*`, `+`, `?` はリテラルとして扱われる（エスケープ不要）
  - `-` は文字クラスの**末尾に配置**すればリテラルとして扱われる
  - `&` は正規表現の特殊文字ではない（どこでもリテラル）
  - 推奨パターン: `[0-9a-zA-Z_!+*?&-]`（`-` を末尾に配置）
- **Implications**: regex修正は安全かつ単純

### `local_marker` との衝突分析
- **Context**: `-` を `sakura_id` に追加した場合、`local_marker = _{ "・" | "-" }` と衝突しないか
- **Sources Consulted**: `grammar.pest`（全ルール構造分析）
- **Findings**:
  - `local_marker` は行レベルルール `local_scene_line` 内でのみ使用
  - `sakura_id` はアクション行内の `sakura_marker`（`\`）直後でのみ評価
  - 構文レベルが完全に分離しており、同一入力位置で競合することは不可能
- **Implications**: 衝突リスクなし

### `sakura_escape` との衝突分析
- **Context**: `\\` と `\-` 等の区別が正しく行われるか
- **Sources Consulted**: `grammar.pest`（`action` ルールの ordered choice 構造）
- **Findings**:
  - `action` の ordered choice で `sakura_escape`（`\\`）は `sakura_script` よりも前に試行される
  - `\\` は常にエスケープとして処理され、`\-` 等は `sakura_script` として処理される
- **Implications**: ordered choice により自動的に正しく分離

### 既存テストパターンの調査
- **Context**: テストケースの配置と記述パターンの確認
- **Sources Consulted**: `crates/pasta_core/tests/`, `crates/pasta_lua/tests/`, `tokenizer.rs` 内テスト
- **Findings**:
  - `pasta_core` テスト: `crates/pasta_core/tests/<feature>_test.rs` 形式（例: `span_byte_offset_test.rs`）
  - `pasta_lua` テスト: `crates/pasta_lua/tests/<feature>_test.rs` 形式（例: `sakura_script_integration_test.rs`）
  - `tokenizer.rs` 内: `#[cfg(test)] mod tests` で単体テスト（12テスト関数）
  - パーサーテストは `parse_str()` APIを使用、`PastaFile` → `FileItem::GlobalSceneScope` → `Action` を検証
  - トークナイザーテストは `Tokenizer::new()` + `tokenizer.tokenize()` で `TokenKind` と `text` を検証
- **Implications**: 新規テストは既存パターンに準拠して追加

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 文字クラス拡張 | 既存4箇所の文字クラスに5文字を追加 | 最小変更、既存パターン準拠、リグレッションリスク極低 | なし | 推奨・唯一の合理的選択肢 |

新コンポーネントの作成、アーキテクチャ変更、ハイブリッドアプローチはいずれも不要。

## Design Decisions

### Decision: 5文字一括追加（`-+*?&`）
- **Context**: 当初は `\-`（ハイフンのみ）がスコープだったが、ukadoc分析で追加4文字が判明
- **Alternatives Considered**:
  1. `-` のみ追加（当初スコープ）
  2. 5文字すべて追加（ukadoc網羅）
  3. より広い許容文字クラス（全ASCII印字可能文字等）
- **Selected Approach**: Option 2 — 5文字すべて追加
- **Rationale**: ukadocを権威的情報源とし、そこに存在しない記号は対応不要。5文字で全タグを網羅できることを確認済み
- **Trade-offs**: 変更量はOption 1より若干増えるが、将来の追加修正が不要になる
- **Follow-up**: テストケースで5文字すべてのタグを検証

### Decision: テスト追加箇所
- **Context**: テストケースの配置場所と網羅範囲
- **Alternatives Considered**:
  1. `tokenizer.rs` 内の `#[cfg(test)] mod tests` に追加
  2. `crates/pasta_core/tests/` に新規テストファイル作成
  3. 両方
- **Selected Approach**: Option 3 — 両方に追加
- **Rationale**: パーサーレイヤー（pasta_core）とランタイムレイヤー（pasta_lua）は独立したクレートであり、それぞれのレイヤーで検証すべき
- **Trade-offs**: テストファイルが増えるが、レイヤー分離の原則に合致
- **Follow-up**: 設計書のテスト戦略セクションで具体的なテストケースを定義

## Risks & Mitigations
- **Risk 1**: regex文字クラス内の `-` 配置ミスによる範囲指定バグ → **Mitigation**: `-` を文字クラス末尾に配置する規約を設計書に明記
- **Risk 2**: `\--`, `\++` 等の複数記号連続マッチ → **Mitigation**: Pastaは意味を解釈しないため問題なし。ukadocにもそのようなタグは存在しない
- **Risk 3**: 4箇所の同期漏れ → **Mitigation**: Req 4で一貫性テストを定義、ドキュメントに変更箇所リストを明記

## References
- [ukadoc さくらスクリプトリスト](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html) — 権威的タグリスト
- [Pest Book](https://pest.rs/book/) — PEG文法記述リファレンス
- [Rust regex crate docs](https://docs.rs/regex/latest/regex/) — 正規表現構文
- [doc/spec/07-sakura-script.md](../../doc/spec/07-sakura-script.md) — Pasta言語仕様書（さくらスクリプト章）
- [GRAMMAR.md](../../GRAMMAR.md) — Pasta DSL文法リファレンス（人間向け）
