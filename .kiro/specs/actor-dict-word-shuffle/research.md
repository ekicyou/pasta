# リサーチ＆設計判断ログ

## サマリ
- **機能**: `actor-dict-word-shuffle`
- **ディスカバリ区分**: Extension（既存バグ修正）
- **主要所見**:
  - Rust 側 `WordTable` のシャッフル＆順次消費機構は完全に動作済み。修正は Lua 側のみで完結する
  - 修正箇所は `actor.lua` の `ACTOR_WORD_BUILDER_IMPL.entry()` 1箇所のみ（登録時バグ）
  - 検索順序（`PROXY_IMPL.word()` の 3 段フォールバック）は変更不要

## リサーチログ

### データ登録パスの検証

- **コンテキスト**: `entry()` が `actor[key]` と `self._word_builder:entry()` の両方に書き込む二重登録バグの影響範囲を確認
- **調査対象**: `actor.lua` L41-53, `word.lua`, `finalize.rs`, `word_registry.rs`, `word_table.rs`
- **所見**:
  - `self._word_builder:entry(...)` は `STORE.actor_words[actor_name][key]` にネスト配列として登録する（`word.lua` のビルダー）
  - `finalize_scene_impl()` が `STORE.actor_words` を走査し、`WordDefRegistry.register_actor()` 経由で Rust `WordTable` に投入する（`finalize.rs` L153-190）
  - アクターキー形式: `:__actor_{sanitized_name}__:{word_name}`（`word_registry.rs` L93）
  - `actor[key]` への書き込みはこのパイプラインとは無関係であり、Lua テーブルのみに留まる
- **含意**: `actor[key]` への書き込みを削除しても `WordTable` へのデータフローは一切影響しない

### 検索パスの検証

- **コンテキスト**: `PROXY_IMPL.word()` の 3 段フォールバックのうち、Level 1 / Level 2 の役割分担を確認
- **調査対象**: `actor.lua` L130-170, `word.lua` L135-157, `search/context.rs` L165-180
- **所見**:
  - **Level 1** (`actor[name]` 直接参照 → `WORD.resolve_value()`):
    - `resolve_value()` は `type(value)` で分岐: `function` → `value(act)` 呼び出し、`table` → `value[1]` 固定、`else` → `tostring`
    - 設計意図上、`actor[key]` には関数型エントリのみが入るべき（開発者確認済み）
    - `entry()` の二重登録バグにより、文字列テーブルが Level 1 で発見され短絡する（= バグ）
  - **Level 2** (`SEARCH:search_word(name, actor_scope)` → Rust `WordTable`):
    - `actor_scope = "__actor_" .. self.actor.name .. "__"` でスコープ指定
    - `SearchContext.search_word()` は `module_name` を受け取り、`WordTable.search_word()` に委譲
    - `WordTable.search_word()` はキャッシュベースの順次消費（シャッフル＆デッキ方式）を実装済み
  - **Level 3** (`self.act:word(name)` → シーン→グローバル): 正常動作
- **含意**: `entry()` の二重登録を除去すれば、文字列値は自然に Level 2 に到達しシャッフルが適用される。Level 1 は関数型エントリとして正しく機能し続ける

### `WORD.resolve_value()` のテーブル型分岐の影響

- **コンテキスト**: `entry()` 修正後に `resolve_value()` のテーブル型分岐が不要になるか
- **所見**:
  - 修正後、`actor[key]` には関数型のみが入るため、`resolve_value()` の `table` 分岐に到達するケースはアクター検索パスでは消滅する
  - ただし `resolve_value()` は `ACT_IMPL.word()` からも呼ばれており、グローバル/ローカル検索パスでの使用を想定した汎用関数である
  - テーブル型分岐はアクター以外のパスで引き続き使用される可能性がある
- **含意**: `resolve_value()` は変更不要。テーブル型分岐は汎用コードとして残す

### 既存テストカバレッジ

- **コンテキスト**: 修正に伴うリグレッションリスクの評価
- **所見**:
  | テストファイル | 検証対象 | シャッフル検証 |
  |---|---|---|
  | `actor_word_dictionary_test.rs` | トランスパイラ出力形式 | なし |
  | `actor_word_test.lua` | Lua モジュール API | なし |
  | `scene_test.rs` L261 | E2E: アクター単語スコープ解決 | なし（NOTE 付き） |
  | `word_table_test.rs` | Rust `WordTable` シャッフル | アクターキー形式なし |
- **含意**: アクタースコープのシャッフル動作を検証するテストが不在。修正と同時に追加が必要

### `finalize_scene_impl()` のデータ収集

- **コンテキスト**: `entry()` 修正後もアクター単語データが Rust 側に正しく到達するか
- **調査対象**: `finalize.rs` L153-190
- **所見**:
  - `collect_words()` は `STORE.global_words`, `STORE.local_words`, `STORE.actor_words` の 3 テーブルを走査し `WordCollectionEntry` に変換
  - アクター単語は `STORE.actor_words[actor_name][key]` から読み取り。`actor[key]` テーブルは参照しない
  - `build_word_registry()` は `WordDefRegistry.register_actor()` でキー形式 `:__actor_xxx__:{word}` を生成
- **含意**: `entry()` 修正は `finalize_scene_impl()` のデータ収集パスに一切影響しない。`STORE.actor_words` への登録は `self._word_builder:entry()` で既に行われている

## アーキテクチャパターン評価

| オプション | 概要 | 強み | リスク | 備考 |
|---|---|---|---|---|
| Option A: Level 1 削除 | `PROXY_IMPL.word()` の Level 1 を削除し Level 2 優先 | Lua 1 ファイル修正 | 関数型エントリの互換性リスク | 検索順序の変更が必要 |
| Option B: `resolve_value()` シャッフル対応 | Level 1 維持、Lua 側独自シャッフル | 高速パス維持 | ロジック二重管理、キャッシュ同期困難 | 非推奨 |
| **Option C: 二重登録廃止**（採用） | `entry()` の `actor[key]` 書き込み削除 | 1 箇所修正、検索順序不変、SoT 一元化 | なし | **推奨・developer確認済み** |

## 設計判断

### 判断: Option C — `entry()` の `actor[key]` 書き込み削除

- **コンテキスト**: 二重登録バグの修正方法として 3 案を検討
- **選択**: Option C（二重登録廃止）
- **根拠**:
  - 修正箇所が `actor.lua` の `entry()` 関数内 1 箇所のみ
  - 検索フォールバック順序（Level 1 → 2 → 3）は**変更なし**。Level 1 は関数型エントリ用として正しく動作し続ける
  - データの Single Source of Truth を Rust `WordTable` に一元化
  - 開発者が設計意図（`actor[key]` は関数型専用）を明示的に確認済み
- **トレードオフ**: `actor[key]` のテーブル直接アクセスは不可になるが、仕様上このパターンは推奨されておらず影響なし
- **フォローアップ**: シャッフル動作を検証するテストの追加

### 判断: テスト戦略 — Lua ランタイム E2E テスト中心

- **コンテキスト**: R3（リグレッション防止テスト）の実装方針
- **選択**: Lua ランタイム E2E テスト + 既存 Rust `WordTable` テストの活用
- **根拠**:
  - バグは Lua 側（`actor.lua`）にのみ存在し、Rust 側（`WordTable`）は正常動作済み
  - `WordTable` のシャッフル＆キャッシュは `word_table_test.rs` で既にカバー
  - E2E テストで「Pasta DSL → transpile → Lua 実行 → finalize → search → シャッフル結果」のパイプライン全体を検証すべき
  - `set_word_selector()` API でモックセレクタを注入でき、決定論的テストが可能
- **フォローアップ**: `scene_test.rs` の既存 NOTE を E2E テストに昇格
  
## リスク＆緩和策

- **リスク: 関数型エントリの既存利用** — 現行コードベースで `actor[key]` に関数型を登録するパターンは 11.5 節（将来拡張）の予約のみ。現在のテスト・辞書ファイルに使用例なし。リスク: なし
- **リスク: パフォーマンス** — Level 2（Rust FFI）は Level 1（Lua テーブル参照）より遅いが、単語参照頻度ではネグリジブル（1 回のトーク構築で数回程度）。リスク: なし
- **リスク: 仕様ドキュメント整合性** — `doc/spec/11-actor-dictionary.md` にシャッフル動作の記述がない。R4 で修正予定。リスク: 低（ドキュメント追記のみ）

## 参照

- [doc/spec/04-call-spec.md §4.1.4](../../doc/spec/04-call-spec.md) — スコープ解決アルゴリズム（キャッシュベースの順次消費）
- [doc/spec/11-actor-dictionary.md](../../doc/spec/11-actor-dictionary.md) — アクター辞書仕様
- [gap-analysis.md](./gap-analysis.md) — バグの発生メカニズム詳細と実装オプション比較
