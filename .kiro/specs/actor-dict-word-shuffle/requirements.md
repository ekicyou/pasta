# Requirements Document

## Project Description (Input)

バグレポート: アクター辞書（`％`ブロック）の複数値定義でシャッフルが機能しない。

`％アクター名` ブロック内の `＠単語：値1,値2,値3` 形式で複数値を定義しても、参照時に常に最初のエントリ（index 0）のみが返される。グローバル/ローカル単語で適用されるシャッフル＆順次消費方式がアクタースコープ単語には適用されていない。

### 根本原因（調査済み）

`pasta_scripts/pasta/actor.lua` の `ACTOR_WORD_BUILDER_IMPL.entry()` に二重登録バグがある。文字列値を Rust `WordTable`（`self._word_builder:entry(...)`）と `actor[key]` テーブルの**両方**に書き込んでいる。

`PROXY_IMPL.word()` の 3 段フォールバック検索において、Level 1 が `actor[name]` を直接参照するため、`entry()` が誤って書き込んだ文字列テーブルが Level 1 で発見される。その結果 `WORD.resolve_value()` が `value[1]`（常に先頭）を返し、シャッフルを実行する Level 2 に到達しない。

| 検索レベル | 経路 | 本来の用途 | 現状 |
|---|---|---|---|
| Level 1 | `actor[name]` 直接参照 → `WORD.resolve_value()` | **関数型エントリ専用**（例: `actor["笑顔"] = function(act) ... end`） | `entry()` の誤った文字列書き込みにより短絡発生 ❌ |
| Level 2 | `SEARCH:search_word(name, actor_scope)` → Rust `WordTable` | **文字列値の選択**（シャッフル＆順次消費） | Level 1 短絡のため未到達 ❌ |
| Level 3 | `act:word(name)` | シーン→グローバルフォールバック | 正常 ✅ |

---

## Requirements

### Requirement 1: アクタースコープ単語のシャッフル適用

**Objective:** ゴースト辞書制作者として、アクター辞書（`％`ブロック）で定義した複数値の単語が、グローバル/ローカル単語と同等のシャッフル＆順次消費方式で選択されるようにしたい。これにより、アクターごとの表情バリエーションが意図通りに機能するようになる

#### Acceptance Criteria

1. When アクター辞書で複数値を持つ単語（例: `＠笑顔：\s[A],\s[B],\s[C]`）が参照される, the pasta runtime shall シャッフル＆順次消費方式で候補値を1つ選択する
2. When 全候補値を一巡消費した後に同じ単語が再度参照される, the pasta runtime shall 候補リストを再シャッフルして新たな消費サイクルを開始する
3. While 未消費の候補値が残っている, the pasta runtime shall 同一候補値を再選択しない（デッキ方式）
4. When アクター辞書の単語が1つの値のみで定義されている, the pasta runtime shall その値を常に返す（既存動作との後方互換性を維持する）

### Requirement 2: `entry()` の不正二重登録の除去

**Objective:** pasta.dll 開発者として、`entry()` が登録した文字列値が `actor[key]` に二重書き込みされないようにしたい。`actor[key]` は関数型エントリ専用として設計されており、文字列値は Rust `WordTable` のみで管理すべきである。これにより Level 1 の短絡評価が解消され、R1 のシャッフル機構に到達できるようになる

#### Acceptance Criteria

1. When `entry()` が文字列値を登録する, the pasta runtime shall `self._word_builder:entry(...)` （Rust `WordTable`）にのみ登録し、`actor[key]` には一切書き込まない
2. When `actor[key]` に関数型エントリが登録されている（例: `actor["笑顔"] = function(act) return act:get_expression() end`）, the pasta runtime shall Level 1 でその関数を `act` を引数として呼び出し、結果を返す（既存の関数型エントリ動作を破壊しない）

### Requirement 3: リグレッション防止テスト

**Objective:** pasta.dll 開発者として、アクタースコープ単語のシャッフルが意図通りに動作することを自動テストで保証したい。これにより、将来の変更でシャッフルが再び壊れることを防ぐ

#### Acceptance Criteria

1. The pasta runtime shall アクタースコープ単語の複数値シャッフルを検証する自動テストを備える
2. The pasta runtime shall アクタースコープ→グローバルへのフォールバック検索を検証する自動テストを備える
3. The pasta runtime shall 単一値アクタースコープ単語の後方互換動作を検証する自動テストを備える

### Requirement 4: 仕様ドキュメント整合性

**Objective:** ゴースト辞書制作者として、アクター辞書の単語選択方式が仕様ドキュメントに明記されていてほしい。これにより、期待動作を正確に理解できる

#### Acceptance Criteria

1. The doc/spec/11-actor-dictionary.md shall アクタースコープ単語が複数値定義時にシャッフル＆順次消費方式で選択されることを明記する（「アクター単語はシャッフルされる」が仕様上の契約である）
2. The doc/spec/11-actor-dictionary.md shall 単語選択のシャッフル動作が [4.1.4 スコープ解決アルゴリズム](04-call-spec.md#414-スコープ解決アルゴリズム) と同一であることへの相互参照を含む
