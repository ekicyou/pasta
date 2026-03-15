# ギャップ分析: word-multi-key

## 1. 現状調査

### 1.1 関連コンポーネントと構造

| コンポーネント | ファイル | 責務 |
|---|---|---|
| PEG文法 | `crates/pasta_dsl/src/parser/grammar.pest` | `key_words` ルール定義 |
| AST型 | `crates/pasta_dsl/src/parser/ast/action.rs` | `KeyWords` 構造体 |
| パーサー | `crates/pasta_dsl/src/parser/parse_elements.rs` | `parse_key_words()` 関数 |
| ファイルスコープ | `crates/pasta_dsl/src/parser/mod.rs` | `file_word_line` 処理 |
| シーンスコープ | `crates/pasta_dsl/src/parser/parse_scene.rs` | `global_scene_word_line` 処理 |
| LSP | `crates/pasta_lsp/src/analysis/visitors.rs` | `visit_keywords()` セマンティックトークン |

### 1.2 現在の `key_words` 文法ルール

```pest
key_words = { id ~ s ~ kv_marker ~ s ~ words }
```

- `id` = 単一の識別子（`XID_START ~ XID_CONTINUE*`）
- `kv_marker` = コロン（`：` / `:`）
- `words` = カンマ区切りの値リスト

**ポイント**: コロン左側は `id`（単一識別子）のみ。複数キーを受け付ける文法が存在しない。

### 1.3 現在の `KeyWords` AST構造体

```rust
pub struct KeyWords {
    pub name: String,       // 単一キー名
    pub words: Vec<String>, // 値リスト
    pub span: Span,         // ソース位置
}
```

### 1.4 下流コンシューマ（pasta_lua — 本仕様スコープ内）

`KeyWords.name` を参照する箇所（7箇所）:

| ファイル | 行 | 用途 |
|---|---|---|
| `context.rs:78` | `register_global(&kw.name, ...)` | グローバル単語登録 |
| `context.rs:86` | `register_local(module, &kw.name, ...)` | ローカル単語登録 |
| `transpiler.rs:83` | `register_global(&word.name, ...)` | トランスパイラPass1登録 |
| `transpiler.rs:126` | `register_actor(&actor.name, &word_def.name, ...)` | アクター単語登録 |
| `element_gen.rs:389` | `literalize(&word.name)` | Luaコード生成（グローバル） |
| `element_gen.rs:415` | `literalize(&word.name)` | Luaコード生成（ローカル） |
| `scope_gen.rs:60` | `word_def.name` | Luaコード生成（アクター） |

**LSP**（`visitors.rs:134`）: `visit_keywords` は `word.span` のみ使用。`name` フィールド未参照。

### 1.5 テスト資産

- `pasta_dsl/tests/actor_code_block_test.rs`: アクター辞書内の単語定義テスト（`words[0].name` 参照）
- `pasta_dsl/tests/ast_test.rs`: シーンスコープ内 `words` ベクタの存在確認
- `pasta_lua/tests/transpiler/comparison_test.rs`: `KeyWords` AST直接構築テスト
- `pasta_core/tests/word_table_test.rs`: WordTable検索テスト（DSL非依存）

### 1.6 規約・パターン

- **カンマ区切り**: 既存の `comma_sep = { s ~ comma ~ s }` / `comma = { "、" | "，" | "," }` を値リストでもアクターリストでも共用
- **アクターリスト**: `actors = { actors_item ~ ( comma_sep ~ actors_item )* ~ comma_sep? }` が複数要素カンマ区切りの先行例
- **PEGルール命名**: `key_*` プレフィクスでキーバリューペア系、`*_line` サフィクスで行文法

---

## 2. 要件ごとのフィージビリティ分析

### Req 1: 文法拡張

| 項目 | 状態 | 備考 |
|---|---|---|
| PEGルール `key_words` 変更 | **要変更** | `id` → `key_list` に置換 |
| `key_list` 新規ルール | **Missing** | `id ~ ( comma_sep ~ id )*` のパターン |
| `comma_sep` 再利用 | ✅ 既存利用可 | 値リストと同一セパレータ |
| キーとコロン間の空白 | ✅ `s` で処理済み | |

**文法上の構造的課題**: コロン左側でのカンマがキー区切りなのか値の一部なのかの曖昧性は **発生しない**。コロン（`kv_marker`）が明確な境界となるため、パーサーは左側をキーリスト、右側を値リストと確定的に区別できる。

### Req 2: AST拡張

| 項目 | 状態 | 備考 |
|---|---|---|
| `KeyWords.name` 既存フィールド | **要変更判断** | 下記オプション参照 |
| 新フィールド追加 | **Missing** | エイリアス情報の表現 |
| `Span` 保持 | ✅ 既存 | 行全体のSpanは保持可 |
| キーごとのSpan | **Unknown** | 個別キーの位置情報が必要か要検討 |

### Req 3: 出現コンテキスト対応

| 項目 | 状態 | 備考 |
|---|---|---|
| ファイルレベル（`file_word_line`） | ✅ 文法変更で自動対応 | `key_words` ルール共有のため |
| シーンスコープ（`global_scene_word_line`） | ✅ 同上 | |
| アクター辞書（`actor_scope_item`） | ✅ 同上 | |

**key_words ルールは3箇所すべてで共有されている**ため、文法変更が自動的に全コンテキストに波及する。パーサーコード（`parse_key_words()`）も単一関数であるため、1箇所の変更で全対応可能。

### Req 4: エラーハンドリング

| 項目 | 状態 | 備考 |
|---|---|---|
| コロンなし形式 | ✅ Constraint | PEGが `kv_marker` を要求するためマッチしない |
| 空キー（`＠、key2：values`） | **要検討** | PEG `id` ルールが空文字を許容しないためマッチ失敗する |

### Req 4: pasta_lua トランスパイル・レジストリ登録対応

| 項目 | 状態 | 備考 |
|---|---|---|
| キーリスト列挙可能な構造 | **Missing** | 現状 `name: String` のみ |
| 値リストの共有保持 | ✅ 既存 | `words: Vec<String>` は共有前提 |
| WordDefRegistry変更不要 | ✅ 確認済 | 各キーで `register_*` を呼べば動作 |
| transpiler.rs 登録ループ | **要変更** | 全キーに対して `register_*` を呼ぶようイテレーション |
| element_gen.rs コード生成 | **要変更** | 全キーに対して `create_word(key)` を出力 |
| scope_gen.rs アクター生成 | **要変更** | 全キーに対して `create_word(key)` を出力 |

---

## 3. 実装アプローチオプション

### Option A: `name` フィールドを `names: Vec<String>` に変更

**概要**: `KeyWords.name: String` を `KeyWords.names: Vec<String>` に完全置換。

**変更範囲**:
- `grammar.pest`: `key_words = { key_list ~ s ~ kv_marker ~ s ~ words }` + `key_list = { id ~ ( comma_sep ~ id )* }`
- `ast/action.rs`: `name: String` → `names: Vec<String>`
- `parse_elements.rs`: `parse_key_words()` — `Rule::key_list` 内の `Rule::id` を全収集
- **pasta_lua側**（スコープ外）: `kw.name` → `kw.names` 参照の7箇所を変更

**トレードオフ**:
- ✅ 最もシンプルで一貫性の高い設計
- ✅ 単一キーの場合 `names.len() == 1` で意味的に等価
- ✅ `pasta_lua` 側でのイテレーション処理が自然
- ❌ **`pasta_lua`側で`kw.name`参照7箇所 + テスト多数のコンパイルエラーが発生**（APIブレーキングチェンジ）
- ❌ 単一キー時に `names[0]` アクセスが必要（やや冗長）

### Option B: `aliases: Vec<String>` フィールドを追加

**概要**: 既存の `name: String` を維持し、追加キーを `aliases: Vec<String>` に格納。

**変更範囲**:
- `grammar.pest`: 同上
- `ast/action.rs`: `aliases: Vec<String>` フィールド追加
- `parse_elements.rs`: 最初の `id` を `name` に、残りを `aliases` に格納
- **pasta_lua側**: 既存の `kw.name` 参照はそのまま動作。`aliases` のイテレーション追加のみ

**トレードオフ**:
- ✅ **`pasta_lua`側の既存コードが壊れない**（`kw.name` が引き続き動作）
- ✅ 後方互換性が最大限保持される
- ✅ 段階的移行が可能（`pasta_lua`側は別仕様で対応）
- ❌ 「最初のキーが`name`でそれ以外が`aliases`」という非対称性
- ❌ 全キーをイテレートするときに `iter::once(&kw.name).chain(kw.aliases.iter())` が必要

### Option C: ハイブリッド — `names` + ヘルパーメソッド

**概要**: `names: Vec<String>` に変更しつつ、`name()` メソッドで最初のキーを返すヘルパーを提供。

**変更範囲**:
- `grammar.pest`: 同上
- `ast/action.rs`: `names: Vec<String>` + `pub fn name(&self) -> &str { &self.names[0] }`
- `parse_elements.rs`: 全キーを `names` に収集
- **pasta_lua側**: `kw.name` → `kw.name()` （メソッド呼び出しへの機械的変更）

**トレードオフ**:
- ✅ 一貫性の高い内部表現（`Vec<String>`）
- ✅ `name()` ヘルパーで単一キー時の使い勝手を維持
- ✅ `names.iter()` で全キーの自然なイテレーション
- ❌ `pasta_lua`側で`kw.name` → `kw.name()` の機械的変更が必要（コンパイルエラーは最小限）
- ❌ フィールドアクセスとメソッドアクセスの混在

---

## 4. 実装複雑度とリスク

**工数**: **S**（1–3日）
- 文法変更は既存パターン（`actors` ルール）のほぼコピー
- AST変更はフィールド追加/変更のみ
- パーサー変更は `parse_key_words()` の1関数内

**リスク**: **Low**
- 文法変更によるパース曖昧性なし（コロンが境界）
- 既存テストは単一キーケースをカバーしており、リグレッション検出可能
- `pasta_dsl` + `pasta_lua` の両クレートがスコープ内であり、AST変更の影響を一貫して対応可能

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

**スコープ拡大により、Option A / B / C いずれも実現可能。設計フェーズで最終決定する。**

各オプションの特徴：
- **Option A**（`names: Vec<String>`）: 最もシンプルで一貫性が高い。`pasta_lua`側7箇所の`kw.name`→`kw.names`変更が必要。
- **Option B**（`aliases` 追加）: 後方互換性が最大。`pasta_lua`側の既存コードはそのまま動作し、`aliases`イテレーション追加のみ。
- **Option C**（`names` + `name()` ヘルパー）: 一貫性と移行容易性のバランス。`kw.name`→`kw.name()`の機械的変更。

`pasta_lua`がスコープ内のため、いずれのオプションでもインクリメンタルに実装可能（AST変更→文法変更→pasta_lua対応の順）。

### 設計フェーズで決定すべき事項

1. **AST設計の最終決定**: Option A / B / C のいずれを採用するか
2. **PEGルール名の命名**: `key_list` vs `word_keys` vs `multi_key` 等
3. **キーごとのSpan**: 各キーに個別のSpanを持たせるか（LSPのホバー・補完に影響）
4. **空キーの扱い**: PEG `id` ルールが空文字をマッチしないため自動排除されるが、明示的エラーメッセージが必要か

### Research Needed

- `pasta_dsl` の `lib.rs` での `KeyWords` の pub re-export 状況（API境界への影響）
- `pasta_lsp` が `KeyWords.name` を今後使用する可能性（セマンティックハイライト拡張計画）
