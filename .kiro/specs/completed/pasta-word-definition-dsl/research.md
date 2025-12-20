# Research & Design Decisions: pasta-word-definition-dsl

---
**Purpose**: 単語定義機能の技術設計に先立つ調査結果とアーキテクチャ決定の記録

---

## Summary
- **Feature**: `pasta-word-definition-dsl`
- **Discovery Scope**: Extension（既存LabelRegistry/LabelTableパターンの拡張）
- **Key Findings**:
  1. 既存の`LabelRegistry`と`LabelTable`のパターンを完全に踏襲可能
  2. `fast_radix_trie`の`iter_prefix()`でローカル/グローバル2段階検索を実現
  3. `CachedSelection`パターン（シャッフル＋Pop）がそのまま適用可能
  4. トランスパイラのモジュール名取得は既存の`parent_counter`パターンで対応

---

## Research Log

### Topic 1: 既存LabelRegistryパターンの分析

- **Context**: `WordDefRegistry`の設計にあたり、類似構造の`LabelRegistry`を分析
- **Sources Consulted**: 
  - [label_registry.rs](../../src/transpiler/label_registry.rs)
  - [labels.rs](../../src/runtime/labels.rs)
- **Findings**:
  - `LabelRegistry`はPass1でラベルを収集し、IDを採番
  - `register_global(name, attributes) -> (id, counter)` パターン
  - `register_local(name, parent_name, parent_counter, attributes)` パターン
  - `sanitize_name()`でRune識別子に変換
  - `HashMap<i64, LabelInfo>`でID→情報マッピング
- **Implications**: 
  - `WordDefRegistry`も同様のパターンで実装可能
  - ただし単語定義は`fn_path`不要、代わりに`values: Vec<String>`が必要
  - エントリ単位（マージなし）で保持する設計に合致

### Topic 2: 前方一致インデックス（RadixMap）の使用パターン

- **Context**: 単語検索における2段階前方一致の実現方法
- **Sources Consulted**:
  - [fast_radix_trie docs](https://docs.rs/fast_radix_trie/1.1.0/)
  - [labels.rs#L160](../../src/runtime/labels.rs) - 既存の`iter_prefix`使用例
- **Findings**:
  - `RadixMap::iter_prefix(query)` で前方一致する全エントリを取得
  - 返り値は`Iterator<Item = (&[u8], &V)>` - キーとValue参照のタプル
  - `Vec<usize>`をValueとすることで、同一プレフィックスに複数エントリIDを格納
  - O(M)検索（Mは検索キー長、エントリ数に非依存）
- **Implications**:
  - ローカル検索: `":module_name:key"`前方一致
  - グローバル検索: `"key"`前方一致
  - 両結果のエントリIDリストを結合してマージ処理

### Topic 3: CachedSelectionパターンの再利用

- **Context**: シャッフル＋順次消費パターンの適用
- **Sources Consulted**:
  - [labels.rs](../../src/runtime/labels.rs) - `CachedSelection`構造体
- **Findings**:
  - 構造: `{ candidates: Vec<LabelId>, next_index: usize, history: Vec<LabelId> }`
  - キャッシュキー: `CacheKey { search_key: String, filters: Vec<(String, String)> }`
  - 初回アクセス時にシャッフルして格納
  - `next_index`をインクリメントしてPopを模倣
  - 枯渇時は再シャッフル
- **Implications**:
  - 単語検索では`(module_name, key)`をキャッシュキーに
  - `filters`は将来の属性フィルタリング用（P1）
  - `history`は不要（単語選択履歴の追跡は要件にない）

### Topic 4: トランスパイラの単語参照コード生成

- **Context**: `SpeechPart::FuncCall`から`pasta_stdlib::word()`への変換
- **Sources Consulted**:
  - [mod.rs#L428-450](../../src/transpiler/mod.rs) - 現行実装
- **Findings**:
  - 現行: `yield Talk(pasta_stdlib::word(ctx, "word", [args]));`
  - 要件: `yield Talk(pasta_stdlib::word("module_name", "word", [filters]));`
  - `ctx`パラメータは廃止し、`module_name`を第1引数に
  - トランスパイル時に現在のグローバルラベル名（サニタイズ済み）を取得可能
  - グローバルスコープでの呼び出しは存在しない（`__start__`もローカル）
- **Implications**:
  - `transpile_speech_part_to_writer()`を修正
  - 呼び出し元から`module_name`を渡す必要あり
  - `TranspileContext`に`current_module`フィールド追加

### Topic 5: キー形式とID空間の分離

- **Context**: グローバル/ローカルキーの衝突回避
- **Sources Consulted**: 要件定義書
- **Findings**:
  - グローバルキー: `"単語名"` (例: `"挨拶"`)
  - ローカルキー: `":モジュール名:単語名"` (例: `":会話_1:挨拶"`)
  - コロン（`:`）で始まるかどうかで区別
  - 前方一致検索でも衝突しない設計
- **Implications**:
  - エントリIDはグローバル通し番号（`Vec<WordEntry>`のインデックス）
  - 検索結果のIDリストを単純結合可能（重複排除不要）
  - キー生成ロジック: `format!(":{}:{}", module_name, key)`

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| LabelRegistry/LabelTableパターン踏襲 | 既存のトランスパイラ→ランタイム分離パターンをそのまま適用 | 実績あり、一貫性、コードレビュー容易 | なし | **選択** |
| 単一構造体 | トランスパイラ・ランタイム兼用の単一WordTable | 実装量削減 | 責務混在、テスト困難 | 却下 |
| 遅延ビルド | ランタイム初回アクセス時にインデックス構築 | 初期化高速化 | 複雑性増加、初回検索遅延 | 却下 |

---

## Design Decisions

### Decision: WordDefRegistry/WordTable分離

- **Context**: 単語定義の収集と実行時検索を分離する必要性
- **Alternatives Considered**:
  1. 単一構造体で両方の責務を持つ
  2. トランスパイラ層(WordDefRegistry) + ランタイム層(WordTable)に分離
- **Selected Approach**: Option 2 - 2層分離
- **Rationale**: 
  - 既存LabelRegistry/LabelTableと一貫性
  - 責務の明確な分離（収集 vs 検索）
  - 将来の拡張性（属性フィルタリング等）
- **Trade-offs**: 
  - コード量増加（2つの構造体）
  - 変換処理が必要（Registry → Table）
- **Follow-up**: `from_word_def_registry()`変換メソッドの実装

### Decision: キー形式によるスコープ区別

- **Context**: ローカル/グローバル単語の区別方法
- **Alternatives Considered**:
  1. 別々のRadixMapで管理
  2. キー形式で区別（コロンプレフィックス）
  3. メタデータフラグで区別
- **Selected Approach**: Option 2 - キー形式による区別
- **Rationale**:
  - 単一RadixMapで効率的
  - 前方一致検索が自然に動作
  - 追加のデータ構造不要
- **Trade-offs**:
  - キー文字列が冗長（ローカルは`:module:key`形式）
- **Follow-up**: キー生成ヘルパー関数の実装

### Decision: CachedSelectionキー設計

- **Context**: キャッシュのキー設計（モジュール間分離）
- **Alternatives Considered**:
  1. `key`のみ（グローバルキャッシュ）
  2. `(module_name, key)`タプル（モジュール別キャッシュ）
  3. `search_key`（生成済みキー文字列）
- **Selected Approach**: Option 2 - `(module_name, key)`タプル
- **Rationale**:
  - モジュール間でキャッシュを分離（要件4.8準拠）
  - 検索キー生成とキャッシュキー生成を分離可能
  - デバッグ時に意味がわかりやすい
- **Trade-offs**:
  - HashMap keyのオーバーヘッド（タプル比較）
- **Follow-up**: なし

### Decision: 空文字列返却（マッチなし時）

- **Context**: 単語が見つからない場合の返却値
- **Alternatives Considered**:
  1. `Option<String>`で`None`返却
  2. 空文字列`""`返却
  3. エラー発生（panic）
- **Selected Approach**: Option 2 - 空文字列返却
- **Rationale**:
  - 要件3.6準拠「空文字列として処理を継続」
  - Rune側での処理が単純（nullチェック不要）
  - Graceful degradation
- **Trade-offs**:
  - エラー検出が困難（ログで対応）
- **Follow-up**: エラーログ出力の実装

### Decision: fast_radix_trie継続使用

- **Context**: 前方一致インデックスのデータ構造選択
- **Alternatives Considered**:
  1. `trie-rs` - LOUDSベースの効率的なTrie
  2. `fast_radix_trie` - 既存で使用中のRadixMap
  3. 自作実装
- **Selected Approach**: Option 2 - fast_radix_trie継続使用
- **Rationale**:
  - 既にLabelTableで使用実績あり
  - `iter_prefix()`で前方一致検索を直接サポート
  - 新規依存追加不要
  - コードパターンの一貫性
- **Trade-offs**:
  - trie-rsより機能が限定的（LOUDSなし）
  - 十分な機能があるため問題なし
- **Follow-up**: なし（既存依存）

---

## Risks & Mitigations

- **Risk 1: パフォーマンス（大量エントリ）** — RadixMapのO(M)検索により緩和、ベンチマーク追加で検証
- **Risk 2: メモリ使用量** — エントリ数に比例するが、実用上問題なし（数千エントリ想定）
- **Risk 3: キャッシュ肥大化** — 同一(module, key)ペアでのみキャッシュ生成、問題になれば LRU 導入
- **Risk 4: APIシグネチャ変更** — 現行`word(ctx, key, args)`から`word(module, key, filters)`への変更、stdlib修正が必要

---

## References

- [fast_radix_trie crate documentation](https://docs.rs/fast_radix_trie/1.1.0/fast_radix_trie/) — RadixMap API
- [pasta-label-resolution-runtime design](../completed/pasta-label-resolution-runtime/design.md) — LabelTable設計
- [SPECIFICATION.md](../../../SPECIFICATION.md) — Pasta DSL仕様
