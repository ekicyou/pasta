# Requirements Document

## Project Description (Input)

### ACT_IMPL.callの本格実装

`act:call(SCENE.__global_name__, "グローバル単語呼び出し", {}, table.unpack(args))`などと、トランスパイルされる`ACT_IMPL.call()`メソッドについて、本格的な実装に置き換える。

#### 基本

+ 入力引数
  + self(=act)
  + global_scene_name
  + key
  + attrs 属性テーブル
  + ... 可変長引数（handlerにそのまま渡す）
+ 処理
  + 検索処理により、関数ハンドラー`handler`を取得する。
  + handler == nilだったときは何もしない。（将来的にログを出す）
  + `return handler(act, ...)`として呼び出す。

#### 検索処理

以下の優先順位で最初に見つけた有効な関数を結果とする。

1. `self.current_scene[key]`
2. `SCENE.search(key, global_scene_name, attrs)`
3. `require("pasta.global")[key]`
4. `SCENE.search(key, nil, attrs)`

#### 備考
入力引数`attrs`は現在は使用していないが、将来的に`SCENE.search(key, global_scene_name, attrs)`などとしてsearchの第三引数として使われる。

---

## Introduction

本仕様は、Pastaトランスパイラが生成する`ACT_IMPL.call()`メソッドを、優先順位付き4段階検索を持つ本格的な実装に置き換えることを目的とする。現在の実装は`SCENE.get()`による単純な検索のみであり、シーンローカル単語・グローバルシーン検索・`pasta.global`モジュール・フォールバック検索を統合した多段解決が必要である。

## Requirements

### Requirement 1: ACT_IMPL.call シグネチャ定義

**Objective:** As a トランスパイラ出力コード, I want 統一されたcallメソッドシグネチャ, so that 一貫した呼び出しパターンでシーン/単語を解決できる。

#### Acceptance Criteria

1. The ACT_IMPL.call shall accept the following parameters: `self` (Act object), `global_scene_name` (string), `key` (string), `attrs` (table), and `...` (variadic arguments).
2. When `global_scene_name` is nil, the ACT_IMPL.call shall perform global search without scope restriction.
3. When `key` is provided, the ACT_IMPL.call shall use it as the search key for handler lookup.
4. The ACT_IMPL.call shall pass `attrs` parameter to future search operations (currently unused, reserved for extensibility).

### Requirement 2: 優先順位付き4段階検索

**Objective:** As a シーン実行者, I want 優先順位に従った段階的な関数検索, so that ローカル定義が優先されつつグローバルへのフォールバックが機能する。

#### Acceptance Criteria

1. When `key` is searched, the ACT_IMPL.call shall first check `self.current_scene[key]` (Level 1: シーンローカル)。
2. If Level 1 returns nil, the ACT_IMPL.call shall call `SCENE.search(key, global_scene_name, attrs)` (Level 2: グローバルシーン名スコープ検索)。
3. If Level 2 returns nil, the ACT_IMPL.call shall check `require("pasta.global")[key]` (Level 3: グローバル関数モジュール)。
4. If Level 3 returns nil, the ACT_IMPL.call shall call `SCENE.search(key, nil, attrs)` (Level 4: スコープなし全体検索)。
5. The ACT_IMPL.call shall return the first non-nil handler found in the priority order.

### Requirement 3: ハンドラー実行

**Objective:** As a シーン実行者, I want 見つかったハンドラーを正しく呼び出す, so that シーン処理が継続される。

#### Acceptance Criteria

1. When a valid handler is found, the ACT_IMPL.call shall invoke it as `handler(act, ...)` passing the Act object and variadic arguments.
2. The ACT_IMPL.call shall return the result of the handler invocation.
3. When no handler is found (all levels return nil), the ACT_IMPL.call shall return nil without error.
4. While handler is nil, the ACT_IMPL.call shall not invoke any function (silent no-op for robustness).

### Requirement 4: SCENE.searchへの属性渡し

**Objective:** As a 将来の拡張機能, I want attrs引数がSCENE.searchに渡される構造, so that 将来的な属性ベースフィルタリングが可能になる。

#### Acceptance Criteria

1. The ACT_IMPL.call shall pass `attrs` parameter to `SCENE.search()` as the third argument when calling Level 2 search.
2. The ACT_IMPL.call shall pass `attrs` parameter to `SCENE.search()` as the third argument when calling Level 4 search.
3. Where `attrs` is nil or empty table, the ACT_IMPL.call shall pass it unchanged to SCENE.search (no special handling required).

### Requirement 5: 既存コードとの互換性

**Objective:** As a 既存のトランスパイル済みコード, I want 新しいcall実装が既存の呼び出しパターンと互換, so that リグレッションなく移行できる。

#### Acceptance Criteria

1. The ACT_IMPL.call shall maintain backward compatibility with existing transpiler output patterns.
2. When called with the current transpiler output format, the ACT_IMPL.call shall function correctly.
3. The ACT_IMPL.call shall not break existing scene function invocation workflows.

### Requirement 6: ログ出力（将来対応）

**Objective:** As a 開発者, I want ハンドラー未発見時のログ出力の拡張ポイント, so that 将来的なデバッグ機能を追加できる。

#### Acceptance Criteria

1. Where handler is not found, the ACT_IMPL.call shall have a designated location for future logging implementation.
2. The ACT_IMPL.call shall include a TODO comment indicating where logging should be added.
