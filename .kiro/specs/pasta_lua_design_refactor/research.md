# Research & Design Decisions

## Summary
- **Feature**: `pasta_lua_design_refactor`
- **Discovery Scope**: Extension（既存システムのリファクタリング）
- **Key Findings**:
  1. 現在のLua実装は責務分担が不明確で、ctx-first設計になっている
  2. Rust側code_generator.rsは既にact-first パターン（`act.アクター:talk()`, `act.アクター:word()`）を生成している
  3. co_actionコルーチンがactの生存期間を定義し、act:call()連鎖でactが継続される

## Research Log

### 既存Luaモジュール構造の分析
- **Context**: pasta_luaのLua側設計を明確化するため、現在のモジュール構成を調査
- **Sources Consulted**: 
  - `crates/pasta_lua/scripts/pasta/init.lua`
  - `crates/pasta_lua/scripts/pasta/ctx.lua`
  - `crates/pasta_lua/scripts/pasta/act.lua`
  - `crates/pasta_lua/scripts/pasta/actor.lua`
- **Findings**:
  - `init.lua`: 11行、未完成（create_session空関数）
  - `ctx.lua`: 68行、co_action/start_action/yield/end_action実装済み
  - `act.lua`: 53行、トークン蓄積ロジック実装済み
  - `actor.lua`: 71行、アクター管理、talk/wordメソッド未完成
  - `areka/init.lua`, `shiori/init.lua`: 空ファイル
- **Implications**: 基本構造は存在するが、設計原則の明確化とAPI整合性が必要

### Rust側code_generator.rsの分析
- **Context**: Lua APIとRust生成コードの整合性確認
- **Sources Consulted**: `crates/pasta_lua/src/code_generator.rs` (910行)
- **Findings**:
  - `PASTA.create_actor("名前")` パターン生成済み（105-116行）
  - `PASTA.create_scene("モジュール名")` パターン生成済み（142-181行）
  - `act.アクター:talk()`, `act.アクター:word()` パターン生成済み（471行）
  - `PASTA.create_session(SCENE, ctx)` パターン生成済み（270行）
  - `PASTA.set_spot(ctx, "name", number)`, `PASTA.clear_spot(ctx)` パターン生成済み（260-268行）
- **Implications**: 
  - Rust側は既にact-firstパターンを生成
  - create_scene はグローバル名のみ（ローカル名対応が必要、別仕様）
  - set_spot/clear_spotはctxを渡す形式→actメソッドに変更が必要（別仕様）

### co_actionコルーチンとactライフサイクル
- **Context**: actの生存期間とvar/save変数スコープの確定
- **Sources Consulted**: `crates/pasta_lua/scripts/pasta/ctx.lua` (12-21行)
- **Findings**:
  ```lua
  function IMPL.co_action(self, scene, ...)
      return coroutine.create(function()
          local act = self:start_action()
          scene(self, table.unpack(args))
          if #act.token > 0 then
              act:end_action()
          end
      end)  -- コルーチン終了 = actの生存期間の終わり
  end
  ```
  - actはco_actionコルーチン内で生成・消滅
  - シーン関数間でact:call()により継続
  - 全シーン連鎖が終了するまでactは同一インスタンス
- **Implications**: 
  - var = act.var（アクション期間中のみ有効）
  - save = ctx.save（永続、コルーチン終了後も残る）

### シーンレジストリ設計
- **Context**: Rust側前方一致検索とLua側シーン取得の連携
- **Sources Consulted**: 設計議論ログ（conversation summary）
- **Findings**:
  - Rustは前方一致検索で (global_name, local_name) タプルを返す
  - Luaはscene.luaで階層構造 `{ [global_name] = { [local_name] = scene_func } }` を管理
  - act:call(search_result, opts, ...) でシーン関数を取得・実行
- **Implications**: scene.luaモジュールの新規設計が必要

### アクタープロキシ設計
- **Context**: `act.アクター:method()` パターンの実現方法
- **Sources Consulted**: 設計議論ログ（conversation summary）
- **Findings**:
  - actの`__index`メタメソッドでアクタープロキシを生成
  - プロキシはactへの逆参照を保持
  - プロキシ経由のtalk/wordメソッドがactにトークンを蓄積
- **Implications**: actor.luaにプロキシ生成ロジックが必要

## Architecture Pattern Evaluation

| Option               | Description                                        | Strengths                    | Risks / Limitations                    | Notes              |
| -------------------- | -------------------------------------------------- | ---------------------------- | -------------------------------------- | ------------------ |
| Act-first (Selected) | シーン関数がactを第1引数で受け取り、actがctxを内包 | 映画撮影比喩と一致、責務明確 | 既存Rust生成コードとの互換性調整が必要 | steering原則に準拠 |
| CTX-first (Current)  | シーン関数がctxを第1引数で受け取る                 | 現状維持                     | 責務が曖昧、設計意図と乖離             | 廃止予定           |

## Design Decisions

### Decision: Act-first アーキテクチャの採用
- **Context**: シーン関数の第1引数をctxからactに変更
- **Alternatives Considered**:
  1. CTX-first維持 — 変更なし
  2. Act-first — actが第1引数、ctxは内包
- **Selected Approach**: Act-first（Option 2）
- **Rationale**: 
  - 映画撮影比喩（台本=シーン関数、アクション！=act）と一致
  - Rust側code_generator.rsが既にact-firstパターンを生成
  - 責務分担が明確化される
- **Trade-offs**: 既存テストコードの更新が必要
- **Follow-up**: Rust側の変更は別仕様で実施

### Decision: save/var変数スコープの分離
- **Context**: 永続変数と作業変数の明確な分離
- **Alternatives Considered**:
  1. 両方ctxに配置
  2. save=ctx, var=act
- **Selected Approach**: save=ctx.save（永続）、var=act.var（作業用）
- **Rationale**: 
  - actのライフサイクル（co_action期間）と変数スコープを一致
  - 短記法アクセス（`local save, var = act.init_scene(SCENE)`）との整合性
- **Trade-offs**: ctx.varからact.varへの移行が必要
- **Follow-up**: act.init_scene()が両方の参照を返す

### Decision: シーンレジストリの階層構造
- **Context**: 複数シーンの管理とRust側検索結果との連携
- **Alternatives Considered**:
  1. フラット構造（global_name_local_name）
  2. 階層構造（`{ [global_name] = { [local_name] = func } }`）
- **Selected Approach**: 階層構造（Option 2）
- **Rationale**: 
  - Rust側前方一致検索の結果（global_name, local_name）と直接対応
  - グローバルシーン単位での検索が効率的
- **Trade-offs**: scene.lua の追加実装が必要
- **Follow-up**: create_scene(global_name, local_name, func) API設計

### Decision: アクタープロキシの__indexメタメソッド実装
- **Context**: `act.アクター:method()` パターンの実現
- **Alternatives Considered**:
  1. 静的プロパティ（事前登録）
  2. __indexメタメソッド（動的生成）
- **Selected Approach**: __indexメタメソッド（Option 2）
- **Rationale**: 
  - アクターは動的に追加される
  - プロキシはactへの逆参照を保持し、トークン蓄積を可能にする
- **Trade-offs**: メタメソッドのオーバーヘッド（無視可能）
- **Follow-up**: プロキシオブジェクトの詳細設計

### Decision: act.init_scene(SCENE) APIの採用
- **Context**: create_session廃止、シーン初期化APIの再設計
- **Alternatives Considered**:
  1. `PASTA.create_session(SCENE, ctx)` — 現行
  2. `act.init_scene(SCENE)` — actメソッド化
- **Selected Approach**: `act.init_scene(SCENE)`（Option 2）
- **Rationale**: 
  - code_generator.rsが `save.変数名`, `var.変数名` を頻繁に出力
  - 短記法 `local save, var = act.init_scene(SCENE)` で効率化
  - actがシーンへの参照を持つことで単語検索にグローバルシーン名を使用可能
- **Trade-offs**: PASTA公開APIからcreate_sessionを削除
- **Follow-up**: Rust側code_generator.rsの修正（別仕様）

### Decision: 単語検索のRust連携仕様
- **Context**: ActorProxy:word() の実装におけるLua→Rust連携の明確化
- **Alternatives Considered**:
  1. Lua側で全検索実装（Rust呼び出しなし）
  2. Lua前処理 + Rust検索の役割分担
- **Selected Approach**: Lua前処理 + mlua FFI経由でRust検索関数を呼び出し（Option 2）
- **Rationale**: 
  - アクターfield検索はLua側のみで完結（前処理）
  - グローバルシーン名検索・全体検索はRust側のWordTableを活用
  - 優先順位制御はLua側、検索ロジックはRust側と責務が明確
- **Trade-offs**: mlua FFIのセットアップが必要
- **Follow-up**: 
  - Rust側で `PASTA_RUNTIME.search_word(global_name|nil, name)` を公開
  - pasta_core の WordTable を使用（実装詳細は別仕様）
- **API形式**:
  ```lua
  -- グローバルシーン名スコープ検索
  local result = PASTA_RUNTIME.search_word("メインシーン", "挨拶")
  -- 全体スコープ検索
  local result = PASTA_RUNTIME.search_word(nil, "挨拶")
  ```
- **Evidence**: design.md の ActorProxy:word() に詳細コメント追記済み

## Risks & Mitigations
- **Risk 1**: Rust側code_generator.rsの修正が必要 → 別仕様で対応、本仕様は設計ドキュメントのみ
- **Risk 2**: 既存テストコードとの非互換 → 設計ドキュメント完成後、実装仕様で対応
- **Risk 3**: areka/shiori拡張モジュールが空 → 拡張ポイント設計のみ本仕様で定義

## References
- [pasta_lua/scripts/pasta/](../../crates/pasta_lua/scripts/pasta/) — 現在のLuaモジュール
- [pasta_lua/src/code_generator.rs](../../crates/pasta_lua/src/code_generator.rs) — Rust側コード生成
- [steering/product.md](../../.kiro/steering/product.md) — プロダクトビジョン
- [steering/tech.md](../../.kiro/steering/tech.md) — 技術スタック
