# Requirements Document

## Project Description (Input)
Pasta DSL の関数呼び出し構文を2点改善する。

1. **`＠＊XX()` のグローバルスコープ修正**  
   現状 `FnScope::Global` でも `SCENE.XX(act)` と展開されており `$*` 変数（`save.XX`）との対称性が壊れている。  
   `＠＊XX()` を `GLOBAL.XX(act, ...)` に展開するよう変更する。

2. **`＄＝expr` 式文（ExprStmt）の新構文追加**  
   代入を伴わない関数呼び出し（副作用専用）を `＄＝expr` 行として記述できるようにする。  
   `＄XX＝＠fn()` が代入ありなのに対し、`＄＝＠fn()` は変数名を省略し結果を捨てる式文として展開する。  
   `var_set` 全バリアントが `＄`（`var_marker`）で始まる一貫性を確保し、LSP/TextMate での識別を容易にする。

## Introduction

Pasta DSL の `＠` 関数呼び出し構文に2つの改善を行う。
第一に、`＠＊XX()` のLuaコード生成先を `SCENE.XX` から `GLOBAL.XX` に変更し、`＄`/`＄＊` 変数スコープとの対称性を確保する。
第二に、代入を伴わない式文 `＄＝expr` を新構文として追加し、副作用専用の関数呼び出しを簡潔に記述できるようにする。

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

### Requirement 2: `＄＝expr` 式文（ExprStmt）構文の追加

**Objective:** DSL作者として、代入を伴わない関数呼び出しを `＄＝＠fn(...)` と書きたい。`＄変数名＝expr` から変数名を省略した自然な対称形として、副作用専用の呼び出しを簡潔に記述するため。`var_set` 全バリアントが `＄`（`var_marker`）で始まる一貫性を保ち、LSP/TextMate での変数スコープ識別を容易にする。

#### Acceptance Criteria

1. When `＄＝＠fn()` がローカルシーン内にインデント付きで記述された場合, the pasta_dsl parser shall `var_set_none` ASTノード（ExprStmt相当）を生成する
2. When `＄＝＠fn（x：10　y：20）` のように引数付きで記述された場合, the pasta_dsl parser shall 引数を含む `var_set_none` ASTノードを生成する
3. The pasta_dsl parser shall `＄＝` / `$=` の全角・半角混在を同等に受け入れる
4. When `var_set_none` がトランスパイルされた場合, the pasta_lua transpiler shall 式を評価する Lua コードを生成し、結果を変数に代入しない（式文として出力する）
5. When `＄＝＠＊fn()` が記述された場合, the pasta_lua transpiler shall `GLOBAL.fn(act)` を式文として生成する（Requirement 1 との組み合わせ）
6. The pasta_dsl parser shall `＄＝expr` 行を既存の `var_set_line` の一部として認識する（`local_scene_item` への変更不要）

#### 設計決定メモ: PEG文法リファクタリング案（議題2・追加議題クローズ済み）

`＄＝expr` は `var_set_none` として既存の `var_set_line` に統合する方針で確定。
全バリアントが `var_marker`（`＄`）で始まり、LSP/TextMate で `＄` を変数操作のシグナルとして一貫して認識できる。

```pest
# 確定案: var_set を3形式に拡張し、＄＝expr を var_set_none として統合
# 全バリアントが var_marker で始まる
var_set        =_{ var_set_global | var_set_local | var_set_none }
var_set_local  = { var_marker ~                 id ~ s ~ set }
var_set_global = { var_marker ~ global_marker ~ id ~ s ~ set }
var_set_none   = { var_marker ~                        set }
set            =_{ set_marker ~ s ~ ( expr | word_ref ) }
```

**議題2の結論**: `＄＝＠単語名`（`word_ref` を含む `var_set_none`）はパーサーレベルで禁止しない。
意味論は「式の結果を捨てる」であり、`word_ref` を含んでも無害（書く人はいない）。
`set` ルールを完全共用できシンプルさが保たれる（パターンB採用）。

**追加議題の結論**: `＝expr` ではなく `＄＝expr` を採用。`var_marker` を全バリアントの先頭に統一することで、
VSCode拡張のシンタックスハイライトと LSP 補完のトリガーの一貫性を確保する。

設計フェーズでは既存の `set` ルールの構造変更（`id` の位置移動：`set` 内 → 親ルール内）による
パーサー・AST の後方互換性への影響を精査すること。

### Requirement 3: 仕様ドキュメントの更新

**Objective:** 仕様策定者として、`doc/spec/` の関連章を更新したい。新構文と変更された動作が権威的仕様書に反映されるようにするため。

#### Acceptance Criteria

1. When 本機能の実装が完了した場合, the specification shall `doc/spec/09-variables.md` の関数呼び出し代入例に `＠＊` のグローバル展開先を明記する
2. When 本機能の実装が完了した場合, the specification shall `doc/spec/01-grammar-model.md` の式サポートセクションに `＄＝expr` 式文の構文と用途を追加する
3. The specification shall 新構文 `＄＝expr`（`var_set_none`）の PEG ルール名を文法モデルに記載する

### Requirement 4: 後方互換性の維持

**Objective:** 既存ゴースト作者として、既存の `.pasta` ファイルが変更なしで動作し続けることを期待する。

> **設計決定（議題1クローズ）**: Lua中間コードの後方互換性維持は不要。
> `pasta_lua` にはキャッシュバージョン管理機構（`.cache_version` / `CacheManager`）があり、
> `CARGO_PKG_VERSION` 変更時にキャッシュが全クリア・再トランスパイルされる。
> したがって R1-AC6 の `local GLOBAL` ヘッダー出力は常時出力（方針A）とし、
> 旧 R4-AC4「生成コードに一切の変化を生じさせない」は削除。

#### Acceptance Criteria

1. The pasta_dsl parser shall 既存の `＠XX()` ローカル関数呼び出し構文を変更なしでパースし続ける
2. The pasta_dsl parser shall 既存の `＄XX＝＠fn()` 変数代入構文を変更なしでパースし続ける
3. The pasta_dsl parser and pasta_lua transpiler shall 既存テスト（950+件）をすべてパスし続ける（スナップショットの期待値更新は許容）
