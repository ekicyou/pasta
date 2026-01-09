# Requirements Document

## Introduction

本仕様は `pasta_lua_design_refactor` 親仕様から派生するLua実装子仕様である。

親仕様で定義された5つのコアモジュール（init, ctx, act, actor, scene）と2つの拡張モジュール（areka, shiori）の実装要件を定義する。

### 親仕様との関係

- **親仕様**: `.kiro/specs/pasta_lua_design_refactor/`
- **参照セクション**:
  - pasta.init, pasta.ctx, pasta.act, pasta.actor, pasta.scene
  - Extension Modules (areka, shiori)
  - Token構造
  - System Flows

### 前提条件

- `pasta_lua_transpiler` 仕様の実装が完了し、新形式のLuaコードが生成されること
- Luaスケルトンコードが親仕様で作成されていること

## Requirements

### Requirement 1: pasta/init.lua の実装
**Objective:** As a Lua開発者, I want PASTAテーブルが公開APIを提供する, so that トランスパイラー出力が正しく動作する

**親仕様参照**: Requirement 1.1, 1.2, 1.3, 1.4, 1.5

#### Acceptance Criteria
1. The init.lua shall `require "pasta"` で PASTA テーブルを返す
2. The init.lua shall `PASTA.create_actor(name)` を実装する（ACTOR.get_or_create委譲）
3. The init.lua shall `PASTA.create_scene(global_name, local_name, scene_func)` を実装する
4. The init.lua shall グローバル状態汚染を防ぐ（ローカルテーブル内に閉じる）
5. The init.lua shall 依存モジュール（ctx, act, actor, scene）をrequireする

### Requirement 2: pasta/ctx.lua の実装
**Objective:** As a Lua開発者, I want CTXが環境管理と外部接続を提供する, so that コルーチン制御が可能

**親仕様参照**: Requirement 2.1, 2.2, 2.3, 2.4, 2.5

#### Acceptance Criteria
1. The ctx.lua shall CTXクラスをメタテーブルで実装する
2. The ctx.lua shall `CTX.new(save, actors)` コンストラクタを実装する
3. The ctx.lua shall save テーブル（永続変数）を保持する
4. The ctx.lua shall actors テーブル（登録アクター）を保持する
5. The ctx.lua shall `CTX:co_action(scene, ...)` でコルーチンを生成する
6. The ctx.lua shall `CTX:start_action()` で Act を生成する
7. The ctx.lua shall `CTX:yield(act)` でトークンを出力する
8. The ctx.lua shall `CTX:end_action(act)` で終了処理を行う

### Requirement 3: pasta/act.lua の実装
**Objective:** As a Lua開発者, I want Actがシーン撮影を記録する, so that トークン蓄積が可能

**親仕様参照**: Requirement 3.1-3.17

#### Acceptance Criteria
1. The act.lua shall ACTクラスをメタテーブルで実装する
2. The act.lua shall `ACT.new(ctx)` コンストラクタを実装する
3. The act.lua shall ctx への参照を保持する
4. The act.lua shall var テーブル（作業変数）を保持する
5. The act.lua shall token 配列を保持する
6. The act.lua shall now_actor（現在のアクター）を保持する
7. The act.lua shall current_scene（現在のSCENE）を保持する
8. The act.lua shall `__index` メタメソッドでアクタープロキシを動的生成する
9. The act.lua shall `ACT:init_scene(scene)` を実装する（save, var を返す）
10. The act.lua shall `ACT:talk(actor, text)` を実装する（トークン蓄積）
11. The act.lua shall `ACT:word(name)` を実装する（3レベル検索）
12. The act.lua shall `ACT:sakura_script(text)` を実装する
13. The act.lua shall `ACT:yield()` を実装する
14. The act.lua shall `ACT:end_action()` を実装する
15. The act.lua shall `ACT:call(search_result, opts, ...)` を実装する
16. The act.lua shall `ACT:set_spot(name, number)` を実装する
17. The act.lua shall `ACT:clear_spot()` を実装する

### Requirement 4: pasta/actor.lua の実装
**Objective:** As a Lua開発者, I want Actorがキャラクター情報を保持する, so that 複数アクター管理が可能

**親仕様参照**: Requirement 4.1-4.8

#### Acceptance Criteria
1. The actor.lua shall ACTORクラスをメタテーブルで実装する
2. The actor.lua shall actor_cache によるキャッシュ機構を実装する
3. The actor.lua shall `ACTOR.get_or_create(name)` を実装する
4. The actor.lua shall name プロパティを持つ
5. The actor.lua shall 動的属性（word定義）を保持できる
6. The actor.lua shall PROXYクラスをメタテーブルで実装する
7. The actor.lua shall `ACTOR.create_proxy(actor, act)` を実装する
8. The actor.lua shall `PROXY:talk(text)` を実装する（act逆参照）
9. The actor.lua shall `PROXY:word(name)` を実装する（4レベル検索）

### Requirement 5: pasta/scene.lua の実装
**Objective:** As a Lua開発者, I want Sceneがシーンレジストリを提供する, so that シーン遷移が可能

**親仕様参照**: Requirement 5.1-5.11

#### Acceptance Criteria
1. The scene.lua shall registry テーブル（階層構造）を実装する
2. The scene.lua shall `SCENE.register(global_name, local_name, scene_func)` を実装する
3. The scene.lua shall `SCENE.get(global_name, local_name)` を実装する
4. The scene.lua shall `SCENE.get_global_table(global_name)` を実装する
5. The scene.lua shall `SCENE.get_global_name(scene_table)` を実装する
6. The scene.lua shall `SCENE.get_start(global_name)` を実装する

### Requirement 6: トークン構造の実装
**Objective:** As a Lua開発者, I want トークン形式が統一されている, so that areka/shiori層で解釈できる

**親仕様参照**: Requirement 7.1-7.6

#### Acceptance Criteria
1. The act.lua shall talk トークン `{ type = "talk", text = string }` を生成する
2. The act.lua shall actor トークン `{ type = "actor", actor = Actor }` を生成する
3. The act.lua shall sakura_script トークン `{ type = "sakura_script", text = string }` を生成する
4. The act.lua shall yield トークン `{ type = "yield" }` を生成する
5. The act.lua shall end_action トークン `{ type = "end_action" }` を生成する
6. The ctx.lua shall yield戻り値 `{ type = "yield" | "end_action", token = [...] }` を生成する

### Requirement 7: 拡張モジュールの実装
**Objective:** As a Lua開発者, I want 拡張ポイントが定義されている, so that 将来の拡張が可能

**親仕様参照**: Requirement 9.1-9.4

#### Acceptance Criteria
1. The areka/init.lua shall AREKA モジュールのスタブを実装する
2. The shiori/init.lua shall SHIORI モジュールのスタブを実装する
3. The 拡張モジュール shall コアpastaモジュールの独立動作を阻害しない
4. The 拡張モジュール shall オプショナルである

### Requirement 8: テスト実装
**Objective:** As a Lua開発者, I want ユニットテストが存在する, so that 品質を保証できる

#### Acceptance Criteria
1. The テスト shall 各モジュールのユニットテストを実装する
2. The テスト shall トークン蓄積・出力フローを検証する
3. The テスト shall アクタープロキシ生成を検証する
4. The テスト shall シーン遷移を検証する

## Out of Scope

- Rust側検索モジュール（pasta_search_module 仕様）
- code_generator.rs の修正（pasta_lua_transpiler 仕様）
- areka/shiori の具体的機能実装
