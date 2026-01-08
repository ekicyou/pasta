# Requirements Document

## Introduction

pasta_luaは、Pasta DSLをLua言語にトランスパイルするバックエンドです。Pasta DSLは「シーンを記述するDSL」であり、トランスパイル結果は「シーン関数」の列挙となります。

### 設計の基本思想：映画撮影の比喩

- **シーン関数** = 台本（撮影対象のシーン記述）
- **アクションオブジェクト（act）** = 「アクション！」の号令で実行を開始する実行コンテキスト
- **環境（ctx）** = アクションオブジェクト内部に保持される環境情報（変数、アクター等）

### シーン関数とactの関係性

シーン関数は `act` に「実際の行動」を与える役割を持ちます：

```lua
function SCENE.__start__(act, ...)
    -- シーン関数はactに行動を与える
    act.set_spot("さくら", 0)              -- 誰が参加するか、立ち位置はどこか
    act.さくら:talk("こんにちは")          -- 会話内容
    act.さくら:word("笑顔")                -- 表情変化
end
```

`act` は与えられた情報をトークンとして収集し、最終的に環境（ctx）に反映します：
- トークン蓄積：各API呼び出し（talk、word等）でトークン配列に追加
- CTX反映：yield/end_action時にトークンを出力し、必要に応じてctxの状態を更新

従って、シーン関数は `act`（アクションオブジェクト）を受け取り、`act` の中に `ctx`（環境）を内包させます。

### 本仕様の性質：設計整理ドキュメント

**重要**: この仕様は「ソースコードの変更を伴わない設計整理」です。

- **要件フェーズ**: 設計原則と責務の明確化
- **設計フェーズ**: モジュール構造、API仕様、データフローの詳細設計
- **実装フェーズ**: **設計ドキュメントの作成**（コード変更は別仕様で実施）

本仕様の成果物は、pasta_lua Lua側の完全な設計ドキュメントであり、将来の実装仕様の基礎資料となります。

### 現状の問題点

- **責務の曖昧性**: `pasta/init.lua`, `ctx.lua`, `act.lua`, `actor.lua`の役割分担が不明確
- **設計の不整合**: 現在のトランスパイルは `ctx` を渡す設計だが、`act` を渡す設計に変更が必要
- **未完成のサブモジュール**: `pasta/areka/init.lua`, `pasta/shiori/init.lua`が空ファイル
- **APIの不整合**: Rust側コード生成が期待するLua APIと実装が乖離

本仕様では、pasta_luaのLua側設計を整理し、「シーン関数は act を受け取る」という明確な原則に基づいた設計を定義します。

## Project Description (Input)
pasta_luaのlua側設計を手伝ってほしい。迷走しているので調整が必要。

## Requirements

### Requirement 1: モジュール構造の明確化
**Objective:** As a pasta_lua開発者, I want 各Luaモジュールの責務が明確に定義されている, so that 保守性と拡張性が向上する

#### Acceptance Criteria
1. The 設計ドキュメント shall 5つのコアモジュールの責務を定義する: `pasta.init`, `pasta.ctx`, `pasta.act`, `pasta.actor`, `pasta.scene`
2. The 設計ドキュメント shall `require "pasta"` が返すべき公開APIテーブル構造を定義する
3. The 設計ドキュメント shall `create_actor`, `create_scene`, `create_session`, `clear_spot`, `set_spot` 関数の仕様を記載する
4. The 設計ドキュメント shall scene.lua がシーンテーブル（グローバルシーン名とローカルシーン名の階層構造）を管理する設計を定義する
5. The 設計ドキュメント shall グローバル状態汚染を防ぐ実装方針を記載する

### Requirement 2: CTX（環境コンテキスト）の設計
**Objective:** As a シーン実行者, I want 環境コンテキストが変数スコープとアクター管理を提供する, so that シーン間でデータを共有できる

#### Acceptance Criteria
1. The 設計ドキュメント shall CTXオブジェクトが保持すべき `var` テーブルの仕様を定義する（ローカルセッション変数）
2. The 設計ドキュメント shall CTXオブジェクトが保持すべき `save` テーブルの仕様を定義する（永続変数）
3. The 設計ドキュメント shall CTXオブジェクトが保持すべき `actors` テーブルの仕様を定義する
4. The 設計ドキュメント shall `CTX.new(save, actors)` 初期化API仕様を記載する
5. The 設計ドキュメント shall CTXがActオブジェクトに内包される設計を定義する
6. The 設計ドキュメント shall CTXがスポット管理情報を保持する仕様を記載する

### Requirement 3: Act（アクション）の設計
**Objective:** As a シーン記述者, I want アクションを通じてトークン列を構築できる, so that 発話やスクリプト出力を生成できる

**設計原則**: actはシーン関数から「行動」を受け取り、トークンとして蓄積し、最終的にctxに反映する。

#### Acceptance Criteria
1. The 設計ドキュメント shall Actオブジェクトが内部にctxを保持する設計を定義する
2. The 設計ドキュメント shall `act.ctx` プロパティを通じた環境アクセス仕様を記載する
3. The 設計ドキュメント shall `act.var`, `act.save` プロパティアクセス仕様を記載する（ctx.var, ctx.saveへの委譲）
4. The 設計ドキュメント shall Actオブジェクトの `token` 配列蓄積メカニズムを定義する
5. The 設計ドキュメント shall Actの `__index` メタメソッドによるアクタープロキシ設計を記載する（`act.アクター名` でアクタープロキシを返す、プロキシはactへの参照を保持する）
6. The 設計ドキュメント shall `act:talk(actor, text)` API仕様を記載する（talkトークンの蓄積）
7. The 設計ドキュメント shall `act:sakura_script(text)` API仕様を記載する（さくらスクリプトトークンの蓄積）
7. The 設計ドキュメント shall `act:yield()` API仕様とトークン出力動作を記載する（蓄積トークンの出力とコルーチン一時停止）
8. The 設計ドキュメント shall `act:end_action()` API仕様と終了処理を記載する（最終トークンの出力とアクション終了）
9. The 設計ドキュメント shall `act:call(module, label, opts, ...)` シーン呼び出しAPI仕様を記載する
10. The 設計ドキュメント shall `act:word(name)` 単語解決API仕様を記載する
11. The 設計ドキュメント shall `act.set_spot(name, number)` スポット設定API仕様を記載する（ctx内部のスポット情報を更新）
12. The 設計ドキュメント shall `act.clear_spot()` スポットクリアAPI仕様を記載する（ctx内部の全スポット情報をリセット）
13. The 設計ドキュメント shall Actがシーン関数の第1引数である設計原則を明記する
14. The 設計ドキュメント shall トークン蓄積からCTX反映までのデータフローを定義する

### Requirement 4: Actor（アクター）の設計
**Objective:** As a キャラクター管理者, I want アクターがスポット位置と属性を持つ, so that 複数キャラクターの会話を管理できる

#### Acceptance Criteria
1. The 設計ドキュメント shall Actorオブジェクトの `name` プロパティ仕様を定義する
2. The 設計ドキュメント shall Actorオブジェクトの `spot` プロパティ仕様を定義する（0以上の整数）
3. The 設計ドキュメント shall `PASTA.create_actor(name)` のキャッシュ機構を定義する
4. The 設計ドキュメント shall 動的属性代入の仕様を記載する（例: `ACTOR.通常 = [=[\s[0]]=]`）
5. The 設計ドキュメント shall Actorの `talk(text)` メソッド仕様を記載する（actプロキシから呼ばれる、トークンをactに蓄積）
6. The 設計ドキュメント shall Actorの `word(name)` メソッド仕様を記載する（actプロキシから呼ばれる、単語をトークン化してactに蓄積）
7. The 設計ドキュメント shall Actorメソッドが呼び出し元actへの参照を持つ設計を定義する（逆参照によるトークン蓄積）

### Requirement 5: Scene（シーン）の設計
**Objective:** As a トランスパイル出力消費者, I want シーンがモジュール単位で管理される, so that Call/Jumpによるシーン遷移が可能

**設計原則**: シーン関数はactに「実際の行動」（会話、表情、スポット設定等）を与える台本である。シーンレジストリはグローバルシーン名とローカルシーン名の階層構造を持ち、Rust側の前方一致検索結果から対応するシーン関数を取得する。

#### Acceptance Criteria
1. The 設計ドキュメント shall `PASTA.create_scene(global_name, local_name, scene_func)` API仕様を記載する（グローバルシーン名とローカルシーン名でシーン関数を登録）
2. The 設計ドキュメント shall シーンテーブル構造 `{ [global_name] = { [local_name] = scene_func, ... }, ... }` を定義する
3. The 設計ドキュメント shall エントリーポイント `__start__` ローカルシーン関数の役割を定義する（モジュールロード時の自動実行ポイント）
4. The 設計ドキュメント shall `__name_N__` パターンの命名規則を定義する（ローカルシーン関数の番号付けパターン）
5. The 設計ドキュメント shall Rust側の前方一致検索がグローバル名・ローカル名の組を返す仕様と、Lua側がそれを用いてシーン関数を取得する方法を定義する
6. The 設計ドキュメント shall シーン関数シグネチャ `(act, ...)` を定義する（第1引数はActオブジェクト）
7. The 設計ドキュメント shall シーン関数内でactに行動を与えるパターンを例示する（`act.アクター:talk("テキスト")`, `act.set_spot()` 等、アクタープロキシ経由のメソッド呼び出し）
8. The 設計ドキュメント shall `act.var`, `act.save` を通じた変数アクセス方法を記載する
9. The 設計ドキュメント shall act.__indexメタメソッドによるアクタープロキシ取得パターンを例示する
10. The 設計ドキュメント shall シーン関数が「行動の記述」に専念し、トークン管理はactに委譲する設計を明記する

### Requirement 6: Spot管理（アクター表示位置）の設計
**Objective:** As a シーン演出者, I want アクターの表示位置を制御できる, so that 伺かの掛け合い表現が可能

**設計原則**: スポット管理はシーン関数がactに与える「行動」の一種であり、「誰がシーンに参加するか、立ち位置はどこか」を指定する。

#### Acceptance Criteria
1. The 設計ドキュメント shall `act.clear_spot()` API仕様を記載する（全スポット割り当てリセット、内部的にctxのスポット情報を更新）
2. The 設計ドキュメント shall `act.set_spot(name, number)` API仕様を記載する（アクター位置指定、内部的にctxのスポット情報を更新）
3. The 設計ドキュメント shall シーン関数内での呼び出しパターン `act.set_spot("さくら", 0)` を例示する
4. The 設計ドキュメント shall スポット割り当て情報のCTX内保存方法を定義する
5. The 設計ドキュメント shall `act.set_spot()` / `act.clear_spot()` が内部的にctx.spotsを更新する仕様を定義する
6. The 設計ドキュメント shall スポット情報がactのトークン出力に反映される仕組みを定義する（必要に応じてスポットトークンの生成）

### Requirement 7: トークン出力仕様
**Objective:** As a ランタイム実装者, I want トークン形式が統一されている, so that areka/shiori層で解釈できる

#### Acceptance Criteria
1. The 設計ドキュメント shall talkトークン構造 `{ type = "talk", actor = Actor, text = string }` を定義する
2. The 設計ドキュメント shall actorトークン構造 `{ type = "actor", actor = Actor }` を定義する
3. The 設計ドキュメント shall sakura_scriptトークン構造 `{ type = "sakura_script", text = string }` を定義する
4. The 設計ドキュメント shall yieldトークン構造 `{ type = "yield" }` を定義する
5. The 設計ドキュメント shall end_actionトークン構造 `{ type = "end_action" }` を定義する
6. The 設計ドキュメント shall コルーチンyield時の戻り値構造 `{ type = "yield" | "end_action", token = [token...] }` を定義する

### Requirement 8: Rust生成コードとの互換性
**Objective:** As a トランスパイラ開発者, I want Lua側APIがRust生成コードと一致する, so that トランスパイル結果が正しく動作する

**実装分担**: Rust側は前方一致検索でグローバル名・ローカル名を返し、Lua側はその結果からシーン関数を取得する。

#### Acceptance Criteria
1. The 設計ドキュメント shall `code_generator.rs` が将来生成すべき関数呼び出しパターンを定義する
2. The 設計ドキュメント shall `PASTA.create_actor("name")` API仕様を記載する
3. The 設計ドキュメント shall `PASTA.create_scene(global_name, local_name, scene_func)` API仕様を記載する（グローバル・ローカル名の階層管理）
4. The 設計ドキュメント shall シーン関数シグネチャ `function SCENE.__start__(act, ...)` を定義する
5. The 設計ドキュメント shall `act.var.変数名`, `act.save.変数名` アクセスパターンを定義する
6. The 設計ドキュメント shall `act:call(search_result, {}, ...)` API仕様を記載する（Rust側から返されたグローバル名・ローカル名から場景を取得、search_resultは`{global_name, local_name}`タプル）
7. The 設計ドキュメント shall `act:word("name")` API仕様を記載する
8. The 設計ドキュメント shall Rust側code_generator.rsの修正が別仕様で必要となることを明記する

### Requirement 9: 拡張ポイントの設計（areka/shiori）
**Objective:** As a 統合開発者, I want areka/shiori固有機能の拡張ポイントがある, so that プラットフォーム固有の実装を追加できる

#### Acceptance Criteria
1. The 設計ドキュメント shall `pasta.areka` モジュールの拡張ポイント設計を定義する
2. The 設計ドキュメント shall `pasta.shiori` モジュールの拡張ポイント設計を定義する
3. The 設計ドキュメント shall コアpastaモジュールの独立動作原則を記載する
4. The 設計ドキュメント shall 拡張モジュールのオプショナル性を明記する
