# Requirements Document

## Project Description (Input)
Pasta DSL の関数呼び出し構文を2点改善する。

1. **`＠＊XX()` のグローバルスコープ修正**  
   現状 `FnScope::Global` でも `SCENE.XX(act)` と展開されており `$*` 変数（`save.XX`）との対称性が壊れている。  
   `＠＊XX()` を `GLOBAL.XX(act, ...)` に展開するよう変更する。

2. **`＝expr` 式文（ExprStmt）の新構文追加**  
   代入を伴わない関数呼び出し（副作用専用）を `＝expr` 行として記述できるようにする。  
   `＄XX＝＠fn()` が代入ありなのに対し、`＝＠fn()` は結果を捨てる式文として展開する。  
   これにより `：発話省略` と同様に `＄変数名省略` のパターンとしてDSLの一貫性が保たれる。

## Introduction

Pasta DSL の `＠` 関数呼び出し構文に2つの改善を行う。
第一に、`＠＊XX()` のLuaコード生成先を `SCENE.XX` から `GLOBAL.XX` に変更し、`＄`/`＄＊` 変数スコープとの対称性を確保する。
第二に、代入を伴わない式文 `＝expr` を新構文として追加し、副作用専用の関数呼び出しを簡潔に記述できるようにする。

## Requirements

### Requirement 1: `＠＊XX()` グローバル関数呼び出しの展開先修正

**Objective:** DSL作者として、`＠＊XX()` が `GLOBAL.XX(act, ...)` に展開されるようにしたい。`＄＊XX` → `save.XX` と対称的なスコープモデルを実現するため。

#### Acceptance Criteria

1. When `＠＊XX()` がアクション行内で使用された場合, the pasta_lua transpiler shall `act.{actor}:talk(tostring(GLOBAL.XX(act)))` を生成する
2. When `＠＊XX（引数名：値）` が名前付き引数付きで使用された場合, the pasta_lua transpiler shall `GLOBAL.XX(act, 値)` を引数展開付きで生成する
3. When `＄YY＝＠＊XX()` が変数代入の右辺で使用された場合, the pasta_lua transpiler shall `var.YY = GLOBAL.XX(act)` を生成する
4. The pasta_dsl parser shall 既存の `FnScope::Global` ASTノードをそのまま維持する（文法変更なし）
5. When `＠XX()`（`＊`なし）が使用された場合, the pasta_lua transpiler shall `SCENE.XX(act)` を生成する（ドット記法。既存の Action::FnCall パスのコロン記法 `SCENE:` バグは本仕様に先立ち修正済み）
6. The pasta_lua transpiler shall ヘッダー部に `local GLOBAL = require "pasta.global"` を `local PASTA = require "pasta"` の次行に出力する

### Requirement 2: `＝expr` 式文（ExprStmt）構文の追加

**Objective:** DSL作者として、代入を伴わない関数呼び出しを `＝＠fn(...)` と書きたい。`＄変数名＝expr` から変数名を省略した自然な対称形として、副作用専用の呼び出しを簡潔に記述するため。

#### Acceptance Criteria

1. When `＝＠fn()` がローカルシーン内にインデント付きで記述された場合, the pasta_dsl parser shall `ExprStmt` ASTノードを生成する
2. When `＝＠fn（x：10　y：20）` のように引数付きで記述された場合, the pasta_dsl parser shall 引数を含む `ExprStmt` ASTノードを生成する
3. The pasta_dsl parser shall `＝` の全角（`＝`）と半角（`=`）を同等に受け入れる
4. When `ExprStmt` がトランスパイルされた場合, the pasta_lua transpiler shall 式を評価する Lua コードを生成し、結果を変数に代入しない（式文として出力する）
5. When `＝＠＊fn()` が記述された場合, the pasta_lua transpiler shall `GLOBAL.fn(act)` を式文として生成する（Requirement 1 との組み合わせ）
6. The pasta_dsl parser shall `＝expr` 行を `local_scene_item` として認識する（`var_set_line`, `call_scene_line`, `action_line` と同じレベル）

#### 設計検討メモ: PEG文法リファクタリング案

`＝expr` を独立した `expr_stmt_line` として追加するのではなく、`var_set` 自体をリファクタリングして統合する案を設計フェーズで検討すること。

```pest
# 案: var_set を3形式に拡張し、＝expr を var_set_none として統合
var_set        =_{ var_set_global | var_set_local | var_set_none }
var_set_local  = { var_marker                 ~ id ~ s ~ set }
var_set_global = { var_marker ~ global_marker ~ id ~ s ~ set }
var_set_none   = { set }
set            =_{ set_marker ~ s ~ ( expr | word_ref ) }
```

この案では `＝expr` が `var_set_none` として既存の `var_set_line` に統合され、新規ルール `expr_stmt_line` の追加や `local_scene_item` への変更が不要になる可能性がある。設計フェーズで既存の `set` ルールとの構造変更（`id` の位置移動）による後方互換性への影響を精査すること。

### Requirement 3: 仕様ドキュメントの更新

**Objective:** 仕様策定者として、`doc/spec/` の関連章を更新したい。新構文と変更された動作が権威的仕様書に反映されるようにするため。

#### Acceptance Criteria

1. When 本機能の実装が完了した場合, the specification shall `doc/spec/09-variables.md` の関数呼び出し代入例に `＠＊` のグローバル展開先を明記する
2. When 本機能の実装が完了した場合, the specification shall `doc/spec/01-grammar-model.md` の式サポートセクションに `＝expr` 式文の構文と用途を追加する
3. The specification shall 新構文 `＝expr` の PEG ルール名を文法モデルに記載する（具体的なルール名は設計フェーズで確定）

### Requirement 4: 後方互換性の維持

**Objective:** 既存ゴースト作者として、既存の `.pasta` ファイルが変更なしで動作し続けることを期待する。

#### Acceptance Criteria

1. The pasta_dsl parser shall 既存の `＠XX()` ローカル関数呼び出し構文を変更なしでパースし続ける
2. The pasta_dsl parser shall 既存の `＄XX＝＠fn()` 変数代入構文を変更なしでパースし続ける
3. The pasta_lua transpiler shall `＠XX()` （ローカルスコープ）の生成コードを変更しない
4. While 既存の `.pasta` ファイルに `＠＊` を使用していない場合, the pasta_lua transpiler shall 生成コードに一切の変化を生じさせない
5. The pasta_dsl parser and pasta_lua transpiler shall 既存テスト（950+件）をすべてパスし続ける
