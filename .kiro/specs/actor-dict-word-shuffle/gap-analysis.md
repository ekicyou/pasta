# ギャップ分析: actor-dict-word-shuffle

## 1. 既存資産マップ

### 要件→既存資産の対応表

| 要件 | 既存資産 | ギャップ |
|---|---|---|
| **R1: シャッフル適用** | `WordTable::search_word()` (word_table.rs) にシャッフル＆キャッシュ実装済み | **バグ**: `PROXY_IMPL.word()` (actor.lua L143-147) が Level 1 短絡評価で `WordTable` をバイパス |
| **R2: コードパス統一** | Rust側: `WordDefRegistry::register_actor()` + `WordTable` 完備 / Lua側: `SEARCH:search_word(name, actor_scope)` 呼び出しコードあり（actor.lua L150-156） | **バグ**: Level 1 の `actor[name]` テーブル参照が完全一致時に常に先に返す。Level 2 に到達しない |
| **R3: テスト** | `actor_word_dictionary_test.rs`（トランスパイラ出力形式）、`actor_word_test.lua`（Luaモジュール単体）、`scene_test.rs`（E2E、NOTE付きで不完全） | **Missing**: シャッフル動作を検証するランタイムテストなし |
| **R4: 仕様ドキュメント** | `doc/spec/11-actor-dictionary.md`（アクター辞書仕様）、`doc/spec/10-words.md`（単語定義仕様） | **Missing**: 11章にシャッフル動作の記述なし |

---

## 2. 根本原因の詳細分析

### バグの発生メカニズム

**データ登録時**（2重登録）:

```
ACTOR:create_word("笑顔"):entry("\s[A]", "\s[B]", "\s[C]")
```

1. **word.lua 辞書への登録**（`WORD_BUILDER_IMPL.entry`）:
   - `STORE.actor_words["むらさき"]["笑顔"]` = `{ {"\s[A]", "\s[B]", "\s[C]"} }`（ネストされた配列）
   - この構造は `finalize_scene()` 後に Rust 側の `WordTable` に反映される

2. **ACTOR プロパティへの登録**（`ACTOR_WORD_BUILDER_IMPL.entry`）:
   - `actor["笑顔"]` = `{"\s[A]", "\s[B]", "\s[C]"}`（フラットな配列）
   - **このデータは Lua 側のみに存在し、Rust 側 `WordTable` とは無関係**

**参照時**:

```lua
-- PROXY_IMPL.word (actor.lua L143-147)
local actor_value = self.actor[name]        -- actor["笑顔"] = {"\s[A]", "\s[B]", "\s[C]"}
if actor_value ~= nil then
    return WORD.resolve_value(actor_value)   -- テーブル型 → value[1] = "\s[A]" (常に固定)
end
-- ↑ ここで return してしまい、Level 2 以降に到達しない
```

**`WORD.resolve_value()` の問題**（word.lua L145-157）:

```lua
function WORD.resolve_value(value, act)
    ...
    elseif type(value) == "table" then
        if #value > 0 then
            return value[1]  -- ← 常に index 1（Lua 1-indexed）を返す
        end
    ...
end
```

### 影響範囲の詳細

| レイヤー | ファイル | 影響 |
|---|---|---|
| Lua runtime | `pasta_scripts/pasta/actor.lua` L143-147 | Level 1 短絡評価（バグの直接原因） |
| Lua runtime | `pasta_scripts/pasta/word.lua` L145-157 | `resolve_value()` がテーブル型で `value[1]` 固定返却 |
| Lua runtime | `pasta_scripts/pasta/actor.lua` L41-53 | `ACTOR_WORD_BUILDER_IMPL.entry()` が2重登録 |
| Rust runtime | `search/context.rs` L145-157 | `search_word()` は正常動作（到達しないだけ） |
| Rust registry | `word_table.rs` L181-243 | `search_word()` シャッフル＆キャッシュは正常実装済み |
| Rust registry | `word_registry.rs` L85-97 | `register_actor()` のキー形式 `:__actor_xxx__:word` は正常 |

---

## 3. 実装アプローチ検討

### Option A: Level 1 短絡評価の削除（Lua 側修正のみ）

**概要**: `PROXY_IMPL.word()` の Level 1 完全一致を削除し、常に Level 2（Rust `SEARCH:search_word()`）を優先する

**変更箇所**:
- `pasta_scripts/pasta/actor.lua` の `PROXY_IMPL.word()` メソッド
  - Level 1 の `actor[name]` 直接参照を削除またはスキップ
  - Level 2 の `SEARCH:search_word(name, actor_scope)` を最初の検索パスにする
  - Level 2 で見つからない場合のみ Level 1（関数型のみ）→ Level 3（`act:word()`）にフォールバック

**トレードオフ**:
- ✅ Rust 側変更なし。Lua ファイル1つのみの修正
- ✅ 既存の `WordTable` シャッフル機構をそのまま活用
- ✅ グローバル/ローカル単語と完全に同一のシャッフルパス
- ❌ `actor[name]` の関数型エントリ（Luaコードブロック定義）の互換性に注意が必要
- ❌ Level 1 完全一致の高速パスが失われる（性能影響は軽微）

**互換性リスク**: **低**
- `actor[name]` に `function` 型が入るケースは将来拡張（11.5節）で予約されているのみ
- 現行では関数型エントリは使用されていない

### Option B: `WORD.resolve_value()` のシャッフル対応

**概要**: Level 1 パスを維持したまま、`WORD.resolve_value()` でテーブル型のシャッフル選択を実装

**変更箇所**:
- `pasta_scripts/pasta/word.lua` の `WORD.resolve_value()` メソッド
  - テーブル型の場合にランダム選択 + 順次消費ロジックを追加
  - Lua 側に独自のシャッフルキャッシュを実装

**トレードオフ**:
- ✅ Level 1 の高速パスを維持
- ❌ Rust 側と Lua 側でシャッフルロジックが二重管理になる
- ❌ データ構造の不一致: `actor[name]` はフラットな配列（1つのエントリの値を展開）、`STORE.actor_words` はネストされた配列（複数エントリ対応）
- ❌ Lua 側でシャッフルキャッシュの実装が必要（Rust側と同期しない）
- ❌ セッション跨ぎのキャッシュ管理が複雑

### Option C: 2重登録の廃止 + Level 2 優先（推奨）

**概要**: `ACTOR_WORD_BUILDER_IMPL.entry()` での `actor[name]` への重複登録を廃止し、単語データを Rust 側 `WordTable` のみに一元管理。`PROXY_IMPL.word()` は Level 2（SEARCH API）を最初に試行し、関数型エントリのみ Level 1 で処理。

**変更箇所**:
1. `pasta_scripts/pasta/actor.lua`:
   - `ACTOR_WORD_BUILDER_IMPL.entry()`: `actor[name]` への値追加を削除（`word_builder:entry()` のみ残す）
   - `PROXY_IMPL.word()`: Level 2 を最初に試行。結果なしの場合のみ Level 1（関数型チェック）→ Level 3
2. テストの追加

**トレードオフ**:
- ✅ データの一元管理（Single Source of Truth: Rust `WordTable`）
- ✅ シャッフルロジックの二重管理を完全排除
- ✅ 既存の `WordTable` キャッシュ機構がそのまま有効
- ✅ 将来拡張（Luaコードブロック定義、動的単語参照）と干渉しない
- ❌ `actor[name]` で値を直接保持しなくなるため、`actor.通常` のようなLuaアクセスが不可になる
  - ただし仕様上このパターンは推奨されておらず、影響なし

---

## 4. 既存テストカバレッジ

| テストファイル | 検証対象 | シャッフル検証 |
|---|---|---|
| `actor_word_dictionary_test.rs` | トランスパイラ出力形式（`ACTOR:create_word` API） | なし |
| `actor_word_test.lua` | Lua モジュール API（`create_actor`, `get_actor_words`） | なし |
| `scene_test.rs` (L248-300) | E2E: アクター単語スコープ解決 | なし（NOTE: Actor-scoped search is not yet implemented） |
| `word_table_test.rs` | Rust `WordTable` シャッフル＆キャッシュ | グローバル/ローカルのみ。アクターキー形式のテストなし |

### 不足しているテスト

1. **Rust 単体テスト**: `word_table_test.rs` に `:__actor_xxx__:word` キー形式の `search_word` テスト
2. **Lua ランタイムテスト**: `PROXY_IMPL.word()` が複数回呼び出しで異なる値を返すことの検証
3. **E2E テスト**: アクター辞書の複数値定義が実際にシャッフルされることの検証

---

## 5. 実装複雑度・リスク評価

| 項目 | 評価 | 根拠 |
|---|---|---|
| **工数** | **S**（1〜3日） | Lua ファイル1〜2箇所の修正 + テスト追加。Rust 側変更なし |
| **リスク** | **Low** | 既存パターンの活用、既知の技術スタック、明確なスコープ |

### リスク詳細

- **後方互換性**: `actor[name]` のテーブル直接アクセスを外部から利用するパターンは仕様上存在しない
- **性能**: Level 2（Rust FFI 呼び出し）は Level 1（Lua テーブル参照）より遅いが、単語参照の頻度ではネグリジブル
- **将来拡張との整合**: 11.5節の Lua コードブロック拡張は `function` 型エントリとして Level 1 で処理可能。Option C はこれと干渉しない

---

## 6. 設計フェーズへの推奨

### 推奨アプローチ: Option C（2重登録の廃止 + Level 2 優先）

**理由**:
- データの一元管理原則に最も適合
- シャッフルロジックの二重管理を排除
- 既存の検証済み `WordTable` シャッフル機構を完全活用
- 変更量が最小で、リグレッションリスクが低い

### 設計フェーズで決定すべき事項

1. **Level 1 の関数型エントリの保持**: `actor[name]` に関数型（将来拡張 11.5節）のみ保持するか、完全に廃止するか
2. **テスト戦略**: Rust 単体テスト vs Lua ランタイムテスト vs E2E のバランス
3. **`WORD.resolve_value()` の扱い**: Level 1 を完全に削除する場合、この関数のテーブル型分岐は不要になるか
