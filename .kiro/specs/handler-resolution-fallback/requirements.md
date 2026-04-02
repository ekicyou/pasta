# Requirements Document

## Project Description (Input)

ハンドラー解決フォールバックの統一。`ACT_IMPL` と `PROXY_IMPL` の3種の入口（シーン解決・ワード取得・expr関数呼び出し）に対し、共通の `find_handler()` 関数でフォールバック検索ロジックを統一する。モード（`"scene"` / `"word"` / `"expr"`）に応じた判定を1系統に集約し、新規の `expr_fn` メソッドおよびトランスパイラ変更を含む。

---

## Introduction

### 背景

現在の pasta Lua ランタイムでは、ハンドラー検索ロジックが `ACT_IMPL.find_scene()`（5段階）、`ACT_IMPL.word()`（4段階）、`PROXY_IMPL.word()`（3段階+委譲）でそれぞれ独自のフォールバック順序を持ち、経路が分散している。新たに「expr関数呼び出し」（`expr_fn`）を追加するにあたり、既存のハンドラー検索を統一的なインターフェースで再構築する。

### スコープ

- **対象レイヤー**: pasta_lua（Lua ランタイムスクリプト + Rust トランスパイラ）
- **対象ファイル（Lua側）**: `pasta_scripts/pasta/act.lua`, `pasta_scripts/pasta/actor.lua`
- **対象ファイル（Rust側）**: `src/code_gen/element_gen.rs`（`Expr::FnCall` 生成部分）

---

## Requirements

### Requirement 1: 共通ハンドラー検索関数 `find_handler`

**Objective:** ランタイム開発者として、シーン解決・ワード取得・expr関数呼び出しの3経路を統一的なインターフェースで検索したい。フォールバック順序の一貫性と保守性を確保するために。

#### Acceptance Criteria

1. The pasta runtime shall provide `ACT_IMPL.find_handler(act, mode, key)` が `mode` = `"scene"` / `"word"` / `"expr"` を受け取り、統一的なフォールバック検索を実行する
2. The pasta runtime shall provide `PROXY_IMPL.find_handler(proxy, mode, key)` が、まずアクターレベルの検索を行い、マッチしなければ `act:find_act_handler()` に委譲する
3. The pasta runtime shall provide コア検索関数 `PROXY_IMPL.find_actor_handler(proxy, mode, key)` がアクタースコープのみを検索する
4. The pasta runtime shall provide コア検索関数 `ACT_IMPL.find_act_handler(act, mode, key)` がローカルシーン → グローバルシーンの順で検索する

### Requirement 2: フォールバック戦略の定義

**Objective:** ランタイム開発者として、key に対するハンドラー解決の優先順位を明確に定義したい。検索結果の予測可能性を担保するために。

#### Acceptance Criteria

1. When `PROXY_IMPL.find_handler()` がアクタープロキシ経由で呼び出された場合, the pasta runtime shall 最初にアクターレベル検索（`find_actor_handler`）を実行し、マッチすれば即座に確定する
2. While `mode` が `"word"` であるとき, the pasta runtime shall アクターレベル検索において、`proxy.actor.XX` の完全一致を最優先で検索する（アクター解決はアクション行のword解決にのみ影響するため、word モード限定）
3. While `mode` が `"word"` であるとき, the pasta runtime shall アクターレベル検索において、アクター単語辞書（前方一致）も検索対象に含める
4. The pasta runtime shall `scene.XX` の完全一致をすべてのモードで最優先で検索する
5. The pasta runtime shall `GLOBAL.XX` の完全一致をすべてのモードで `scene.XX` の次に検索する（完全一致をまとめて先に解決することで、GLOBAL定義が辞書よりも常に優先されるという直感的な挙動をゴースト開発者に保証するため）
6. While `mode` が `"word"` であるとき, the pasta runtime shall ローカル単語辞書（前方一致）を検索する
7. While `mode` が `"scene"` または `"expr"` であるとき, the pasta runtime shall ローカルシーン辞書（前方一致）を検索する
8. While `mode` が `"word"` であるとき, the pasta runtime shall グローバル単語辞書（前方一致）を検索する
9. While `mode` が `"scene"` または `"expr"` であるとき, the pasta runtime shall グローバルシーン辞書（前方一致）を検索する
10. The pasta runtime shall すべての辞書検索でマッチしなかった場合、`act.XX`（`find_act_handler` の自身メソッド）の完全一致を `function` 型に限り、すべてのモードで検索する（`SHIORI_ACT_IMPL.transfer_req_to_var` 等のサブクラスメソッドへのフォールバックを保証するため）
11. If いずれの検索でもマッチしない場合, the pasta runtime shall `nil` を返却する

### Requirement 3: モード別ポストプロセス（ハンドラー取得後の処理）

**Objective:** ランタイム開発者として、ハンドラー取得後の処理をモードごとに明確に定義したい。呼び出し側が型判定を気にせず統一的に利用できるようにするために。

#### Acceptance Criteria

##### ワードモード (`"word"`)
1. If ハンドラー `h` が `nil` の場合, the pasta runtime shall key が見つからないエラーログを出力し、何もせず return する
2. When ハンドラー `h` が `function` 型の場合, the pasta runtime shall `h(proxy or act)` を呼び出して戻り値を返す
3. When ハンドラー `h` がその他の型の場合, the pasta runtime shall `tostring(h)` を返す

##### シーンモード (`"scene"`)
4. When ハンドラー `h` が `function` 型の場合, the pasta runtime shall コルーチン化して実行する
5. If ハンドラー `h` が `function` 型でない場合, the pasta runtime shall key が見つからないエラーログを出力し、何もせず return する

##### Exprモード (`"expr"`)
6. When ハンドラー `h` が `function` 型の場合, the pasta runtime shall `h(proxy or act, ...)` を呼び出して戻り値を返す（可変引数を引き渡す）
7. If ハンドラー `h` が `function` 型でない場合, the pasta runtime shall key が見つからないエラーログを出力し、何もせず return する

### Requirement 4: `expr_fn` メソッドの新設

**Objective:** ゴースト制作者として、Pasta DSL 内からローカル関数を引数付きで呼び出したい。式の中で動的な計算やカスタムロジックを利用するために。

#### Acceptance Criteria

1. The pasta runtime shall `ACT_IMPL.expr_fn(act, key, ...)` メソッドを提供し、`find_handler(act, "expr", key)` でハンドラーを検索した上でポストプロセスを実行する
2. The pasta runtime shall `PROXY_IMPL.expr_fn(proxy, key, ...)` メソッドを提供し、`find_handler(proxy, "expr", key)` でハンドラーを検索した上でポストプロセスを実行する
3. When `expr_fn` 経由でハンドラーが見つかった場合, the pasta runtime shall 呼び出し元から渡された可変引数をそのままハンドラーに伝搬する

### Requirement 5: トランスパイラのローカル関数呼び出し変更

**Objective:** ゴースト制作者として、DSL のローカル関数呼び出し構文がランタイムの統一ハンドラー解決を経由して実行されるようにしたい。定義場所を意識せず関数を呼び出せるようにするために。

#### Acceptance Criteria

1. When アクター修飾付きローカル関数呼び出し（`さくら：＠XX（...）`）がトランスパイルされる場合, the pasta transpiler shall `proxy:expr_fn("XX", ...)` の形式で Lua コードを生成する
2. When 変数代入式でのローカル関数呼び出し（`$=＠XX（...）`）がトランスパイルされる場合, the pasta transpiler shall `act:expr_fn("XX", ...)` の形式で Lua コードを生成する
3. The pasta transpiler shall 引数リストを正しくリテラル化・エスケープして `expr_fn` の引数として渡す

### Requirement 6: 既存 `word()` / `find_scene()` のリファクタリング

**Objective:** ランタイム開発者として、既存の `ACT_IMPL.word()` / `PROXY_IMPL.word()` / `ACT_IMPL.find_scene()` を新しい `find_handler()` を用いた実装に移行したい。コードの重複を排除し、フォールバック順序の一貫性を保証するために。

#### Acceptance Criteria

1. The pasta runtime shall `ACT_IMPL.word()` を `find_handler(act, "word", key)` + ワードモードポストプロセスで再実装する
2. The pasta runtime shall `PROXY_IMPL.word()` を `find_handler(proxy, "word", key)` + ワードモードポストプロセスで再実装する
3. The pasta runtime shall `ACT_IMPL.find_scene()` を `find_handler(act, "scene", key)` + シーンモードポストプロセスで再実装する
4. The pasta runtime shall 既存の全テスト（950+件）がリファクタリング後も通過する

### Requirement 7: エラーハンドリングとログ出力

**Objective:** ランタイム開発者として、ハンドラーが見つからなかった場合の診断情報を得たい。辞書定義の不足やタイポを早期発見するために。

#### Acceptance Criteria

1. If ハンドラーが見つからなかった場合, the pasta runtime shall key・mode・検索経路（proxy or act）を含むエラーログを `@pasta_log` 経由で出力する
2. The pasta runtime shall エラーログ出力後、例外を投げずに `nil` を返却する（ゴーストの動作を中断しない）
