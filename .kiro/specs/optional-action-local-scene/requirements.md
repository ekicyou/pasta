# Requirements Document

## Project Description (Input)
## アクション行がないローカルシーン分岐が書きたい
以下の行が成立するように、pest定義からluaトランスパイル、最終コード実行までの確認を行う。

```pasta
＊サンプル
　ぱすた：分岐するよ！
　　＞分岐

　　・分岐会話無し
　　・分岐会話アリ
　ぱすた：分岐したよ！
```

上記のようなコードをトランスパイルから実行まで行うインテグレーションテストを追加し、このシナリオが成立するように全体を修正せよ


具体的には以下のコード修正で行けるはず。

修正前：
```pest
local_start_scene_scope = {                     local_scene_item+ ~ code_scope* }
local_scene_scope       = { local_scene_start ~ local_scene_item+ ~ code_scope* }
```

修正後：
```pest
local_start_scene_scope = {                     local_scene_item* ~ code_scope* }
local_scene_scope       = { local_scene_start ~ local_scene_item* ~ code_scope* }
```

## Introduction

ローカルシーン分岐において、アクション行（会話行・変数操作行・コメント行等）を一切持たないシーンを許容する。
現状のPest文法定義では `local_scene_item` が1個以上必須（`+`）であるため、ローカルシーンの直後に次のローカルシーンが続く構文がパースエラーとなる。
この制約を緩和し、アクション行0個のローカルシーンを文法・トランスパイラ・ランタイムの全レイヤーで正しく処理できるようにする。

**スコープ判断**: 空のグローバルシーン（`＊シーン名` の直後に次の `＊` が続く）は文法上は同様の変更で許容可能だが、実運用では中身のないグローバルシーンの需要がないため、本仕様のスコープ外とする。

## Requirements

### Requirement 1: アクション行なしローカルシーンのパース許容

**Objective:** Pasta DSL作者として、アクション行を一切持たないローカルシーン分岐を記述したい。ローカルシーンの直後に次のローカルシーンが続く構文を文法エラーなく定義できるようにするため。

#### Acceptance Criteria
1. When ローカルシーン（`・分岐名`）の直後に別のローカルシーン（`・別の分岐名`）が続く場合, the pasta_dsl parser shall パースエラーを発生させずにASTを生成する
2. When アクション行を含むローカルシーンがパースされた場合, the pasta_dsl parser shall 従来通り正常にASTを生成する（リグレッションなし）
3. When ローカルシーン開始直後（`local_start_scene_scope`）にアクション行が0個の場合, the pasta_dsl parser shall 空の `local_scene_item` リストを持つASTノードを生成する

### Requirement 2: アクション行なしローカルシーンのLuaトランスパイル

**Objective:** Pasta DSL作者として、アクション行0個のローカルシーンがLuaコードに正しく変換されてほしい。トランスパイラがパニックやエラーなく空シーンを処理できるようにするため。

#### Acceptance Criteria
1. When アクション行0個のローカルシーンASTがトランスパイラに渡された場合, the pasta_lua transpiler shall 有効なLuaコードを生成する
2. When 同一グローバルシーン内にアクション行ありとアクション行なしのローカルシーンが混在する場合, the pasta_lua transpiler shall 両方を正しくLuaコードに変換する

### Requirement 3: アクション行なしローカルシーンの実行時動作

**Objective:** ゴースト作者として、アクション行なしのローカルシーンが実行時にエラーなく動作してほしい。Call文で分岐した先が空であっても、正常に呼び出し元に制御が戻るようにするため。

#### Acceptance Criteria
1. When Call文（`＞分岐`）でアクション行なしのローカルシーンが呼び出された場合, the pasta_lua runtime shall エラーなく実行し、呼び出し元に制御を返す

### Requirement 4: インテグレーションテスト

**Objective:** 開発者として、サンプルコード（ローカルシーン直後に次のローカルシーンが続く最も厳しいケース）が全レイヤーを通して正しく動作することを検証したい。パース→トランスパイル→実行の一連のフローが成立することを自動テストで保証するため。

#### Acceptance Criteria
1. The pasta_lua test suite shall 以下のサンプルコードをパース→トランスパイル→実行するインテグレーションテストを含む：ローカルシーン `・分岐会話無し` の直後に `・分岐会話アリ` が続き、アクション行が一切ない分岐が混在する構文（名前付き空ローカルシーン: Req1-AC1 検証）
2. The pasta_lua test suite shall `local_start_scene_scope` がアクション行0個で直接ローカルシーンに分かれるケースのインテグレーションテストを含む（空スタートスコープ: Req1-AC3 検証）
3. When `cargo test --all` が実行された場合, the test suite shall 新規テストを含む全テストがパスする（リグレッションなし）
