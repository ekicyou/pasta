# Research & Design Decisions

## Summary
- **Feature**: `coroutine-resume-loop`
- **Discovery Scope**: Extension（既存システム拡張）
- **Key Findings**:
  1. `set_co_scene`パターンが確立済み - 同様の責務分離パターンで`resume_until_valid`を実装可能
  2. `coroutine.status()`の"suspended"/"dead"判定がループ終了条件に直接利用可能
  3. 初回resumeのみ`act`引数が必要、2回目以降は引数不要

## Research Log

### Luaコルーチン状態遷移
- **Context**: ループ終了条件の正確な判定
- **Sources Consulted**: Lua 5.5 Reference Manual, 既存コード調査
- **Findings**:
  - `coroutine.status(co)`: "running" | "suspended" | "dead" | "normal"
  - resume成功後、コルーチンがyieldした場合は"suspended"
  - resume成功後、関数が終了した場合は"dead"
  - resume失敗（エラー）の場合も"dead"になり、`ok=false`
- **Implications**: ループ条件は `ok and value == nil and status == "suspended"` で判定

### 既存`set_co_scene`パターン分析
- **Context**: 新規関数の設計パターン参照
- **Sources Consulted**: `pasta/shiori/event/init.lua` L83-100
- **Findings**:
  - ローカル関数として内部に閉じ込め
  - 引数検証→状態チェック→操作の順序
  - LuaDocアノテーション付き
- **Implications**: 同様のパターンで`resume_until_valid`を実装

### 初回resume引数の扱い
- **Context**: ハンドラ・シーン関数への`act`渡し方法
- **Sources Consulted**: 既存`EVENT.fire`実装、`SCENE.co_exec`の`wrapped_fn`
- **Findings**:
  - 現状: `coroutine.resume(result, act)` で初回のみ`act`を渡す
  - `wrapped_fn(act, ...)` が最初のresume引数を受け取る
  - 2回目以降のresumeは引数なし（yieldからの再開）
- **Implications**: `resume_until_valid(co, act)`シグネチャで初回引数を受け取り、ループ内2回目以降は引数なしでresume

## Architecture Pattern Evaluation

| Option                  | Description                         | Strengths            | Risks / Limitations        | Notes    |
| ----------------------- | ----------------------------------- | -------------------- | -------------------------- | -------- |
| A: インライン展開       | EVENT.fire内にwhileループを直接記述 | 変更箇所最小         | 関数肥大化、テスト粒度粗い | 不採用   |
| **B: ローカル関数分離** | `resume_until_valid`を新規作成      | 責務分離、テスト容易 | 関数追加1件                | **採用** |
| C: モジュール分離       | 別ファイルにユーティリティ化        | 再利用可能           | 過剰設計                   | 不採用   |

## Design Decisions

### Decision: ループ関数のシグネチャ
- **Context**: 初回resume引数と戻り値の設計
- **Alternatives Considered**:
  1. `resume_until_valid(co)` + 事前にresume済み
  2. `resume_until_valid(co, ...)` + 可変長引数で初回渡し
- **Selected Approach**: `resume_until_valid(co, ...)` - 可変長引数で初回resume引数を受け取る
- **Rationale**: 
  - 呼び出し元で初回resumeを分離すると責務が分散
  - 可変長引数により将来の拡張性を確保
- **Trade-offs**: 可変長引数のオーバーヘッドは無視できるレベル
- **Follow-up**: なし

### Decision: エラー時のコルーチンclose責務
- **Context**: resume中にエラーが発生した場合の処理
- **Alternatives Considered**:
  1. `resume_until_valid`内でclose
  2. 呼び出し元（EVENT.fire）でclose
- **Selected Approach**: 呼び出し元でclose（既存パターン維持）
- **Rationale**: 
  - 既存の`set_co_scene`がclose処理を担当
  - `resume_until_valid`は純粋にループ処理のみに集中
- **Trade-offs**: エラー時は`ok=false`を返し、呼び出し元がset_co_sceneを呼ぶ
- **Follow-up**: なし

## Risks & Mitigations
- **無限ループリスク** → 設計方針としてループ上限は設けない（`wrapped_fn`が必ず終了を保証）
- **後方互換性** → 既存のテストケースで検証可能

## References
- [Lua 5.5 Reference - Coroutines](https://www.lua.org/manual/5.5/manual.html#2.6)
- 既存実装: `crates/pasta_lua/scripts/pasta/shiori/event/init.lua`
- 既存テスト: `crates/pasta_lua/tests/lua_specs/event_coroutine_test.lua`
