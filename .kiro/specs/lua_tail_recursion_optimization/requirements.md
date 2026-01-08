# Requirements Document

## Project Description (Input)
luaの末尾再帰を有効化するため、トランスパイラー出力を調整してほしい。関数末尾にcallが発生した場合に、「return 関数()」記法にすること。

## Introduction

Luaは末尾呼び出し最適化 (Tail Call Optimization, TCO) をサポートしています。関数の最後の文が `return func()` の形式である場合、Luaはスタックフレームを消費せずに呼び出しを行います。これにより、深い再帰やシーンチェーンでもスタックオーバーフローを回避でき、メモリ効率とパフォーマンスが向上します。

現在のcode_generator.rsでは、シーン関数の末尾で `act:call()` が生成されていますが、`return` が付いていないため、TCOが有効になりません。本機能では、関数末尾の呼び出しに `return` を自動付与することで、Luaの末尾再帰最適化を有効化します。

### 分析結果: 末尾最適化の対象

| 呼び出しパターン      | 現在の生成                                  | 末尾最適化対象 | 理由                                    |
| --------------------- | ------------------------------------------- | -------------- | --------------------------------------- |
| `act:call()`          | `act:call("モジュール", "ラベル", {}, ...)` | **対象**       | シーン遷移の主要パターン、戻り値不使用  |
| `act.xxx:talk()`      | `act.さくら:talk("...")`                    | 対象外         | 副作用目的、returnは意味がない          |
| `act.xxx:word()`      | `act.さくら:word("...")`                    | 対象外         | 副作用目的、returnは意味がない          |
| `SCENE.関数()` (式中) | `save.x = SCENE.関数(ctx, ...)`             | 対象外         | 戻り値が使用されている                  |
| `SCENE.関数()` (単独) | 現在は生成されない                          | 将来検討       | Pastaでは単独関数呼び出しはact:callのみ |

**結論**: 現時点では `act:call()` のみが末尾最適化の対象です。将来の拡張で単独関数呼び出しが追加された場合にも対応できる設計とします。

### 変更前後の比較

**Before:**
```lua
function SCENE.__start__(ctx, ...)
    local args = { ... }
    local act, save, var = PASTA.create_session(SCENE, ctx)

    act:call("メイン1", "グローバル単語呼び出し", {}, table.unpack(args))
    act:call("メイン1", "ローカル単語呼び出し", {}, table.unpack(args))
    act:call("メイン1", "引数付き呼び出し", {}, var.カウンタ, save.グローバル, table.unpack(args))
end
```

**After:**
```lua
function SCENE.__start__(ctx, ...)
    local args = { ... }
    local act, save, var = PASTA.create_session(SCENE, ctx)

    act:call("メイン1", "グローバル単語呼び出し", {}, table.unpack(args))
    act:call("メイン1", "ローカル単語呼び出し", {}, table.unpack(args))
    return act:call("メイン1", "引数付き呼び出し", {}, var.カウンタ, save.グローバル, table.unpack(args))
end
```

## Requirements

### Requirement 1: 末尾呼び出し検出
**Objective:** トランスパイラ開発者として、関数末尾に位置する呼び出しを正確に検出したい。これにより、TCO対象を特定できる。

#### Acceptance Criteria
1. When `generate_local_scene_items` が項目リストを処理する際、code_generatorはリストの最後の項目が `CallScene` であるかどうかを判定する
2. When 最後の項目が `CallScene` の場合、code_generatorは `is_tail_call` フラグを true に設定する
3. When 最後の項目が `CallScene` 以外（`ActionLine`, `VarSet`, `ContinueAction`）の場合、code_generatorは `is_tail_call` フラグを false に設定する

### Requirement 2: return文の条件付き生成
**Objective:** トランスパイラ開発者として、末尾呼び出しに対してのみ `return` を付与したい。これにより、LuaのTCOが有効化される。

#### Acceptance Criteria
1. When `is_tail_call` が true で `CallScene` を生成する場合、code_generatorは `return act:call(...)` 形式で出力する
2. When `is_tail_call` が false で `CallScene` を生成する場合、code_generatorは従来通り `act:call(...)` 形式で出力する
3. The code_generator shall 既存の `generate_call_scene` メソッドのシグネチャを拡張し、`is_tail_call: bool` パラメータを追加する

### Requirement 3: 既存テストの互換性維持
**Objective:** トランスパイラ開発者として、既存の動作を破壊しないことを保証したい。これにより、リグレッションを防止できる。

#### Acceptance Criteria
1. The code_generator shall 中間位置の `act:call()` について従来と同一の出力を維持する
2. When 既存テストを実行した場合、code_generatorは末尾以外の呼び出しで `return` を生成しない
3. The code_generator shall 末尾位置のみに `return` を追加することで、既存の動作ログ・出力順序を変更しない

### Requirement 4: 新規テストケース追加
**Objective:** トランスパイラ開発者として、末尾再帰最適化の正確な動作を検証したい。これにより、機能の正当性を保証できる。

#### Acceptance Criteria
1. When 単一の `act:call()` のみを含むシーン関数を生成した場合、テストは `return act:call(...)` 形式を検証する
2. When 複数の `act:call()` を含むシーン関数を生成した場合、テストは最後の呼び出しのみに `return` が付与されていることを検証する
3. When `act:call()` の後に `ActionLine` が続く場合、テストは `return` が生成されないことを検証する
4. When シーン関数に `act:call()` が含まれない場合、テストは `return` が生成されないことを検証する

### Requirement 5: 将来の拡張性維持（最小限の設計）
**Objective:** トランスパイラ開発者として、現時点では `CallScene` の末尾最適化に特化しつつ、将来的に新規パターンが追加される場合に容易に対応できる設計としたい。

#### Acceptance Criteria
1. The code_generator shall 末尾判定メソッドを「呼び出し判定」という汎用的な名前で実装し、コメントで将来の拡張意図を明記する
2. The code_generator shall 末尾判定ロジック内で `matches!(item, LocalSceneItem::CallScene(_))` パターンマッチを使用し、将来新規パターン追加時は `matches!` マクロに条件追加するだけで対応可能な構造とする
3. Where 将来 `LocalSceneItem::FnCall` など新規バリアントが追加される場合、末尾判定メソッドの `matches!` 条件を拡張するだけで対応可能とする
