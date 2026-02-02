# リサーチログ: scene-coroutine-execution

**作成日**: 2026-02-02  
**スコープ**: Light Discovery（既存システム拡張）

## サマリー

本機能はPasta SHIORIランタイムの既存イベント処理システムを拡張し、コルーチンベースのシーン実行とチェイントーク継続機能を追加する。

### 主要発見事項

1. **既存実装の活用**: `act:yield()`は既に実装済み（`coroutine.yield(script)`を使用）
2. **Lua 5.4 コルーチンAPI**: `coroutine.close()`はLua 5.4で追加された機能であり、to-be-closedリソースの確実な解放を保証
3. **既存パターン踏襲**: EVENT.fireのhandler呼び出しパターン、STOREのフィールド管理パターンを踏襲

---

## リサーチログ

### Topic 1: Lua 5.4 コルーチンAPI

**調査範囲**: coroutine.create/resume/close/status の動作仕様

**発見事項**:
- `coroutine.create(f)` - 関数fを本体とする新しいコルーチン（thread型）を作成
- `coroutine.resume(co, ...)` - コルーチンの実行を開始/再開。最初の引数はメイン関数に渡され、以降の引数はyieldの戻り値として渡される
- `coroutine.status(co)` - 状態を返す: "running", "suspended", "normal", "dead"
- `coroutine.close(co)` - Lua 5.4追加。suspended状態のコルーチンをclose、to-be-closed変数を解放、戻り値は(true)または(false, errormsg)

**影響**:
- `coroutine.close()`の使用により、CO.safe_wrap()不要で厳密なリソース解放が可能
- エラー時もcoroutine.close()で確実にクリーンアップ

**出典**: Lua 5.4 Reference Manual, gap-analysis.md

### Topic 2: 既存EVENT.fire実装パターン

**調査範囲**: 現在のハンドラ呼び出しフロー

**現状コード** (`pasta/shiori/event/init.lua`):
```lua
function EVENT.fire(req)
    local act = create_act(req)
    local handler = REG[req.id] or EVENT.no_entry
    return handler(act)  -- 直接呼び出し
end
```

**拡張ポイント**:
1. handlerの戻り値を判定（thread/string/nil）
2. thread時: `coroutine.resume(result, act)`を実行
3. 状態管理: `set_co_scene()`ヘルパー関数でSTORE.co_sceneを更新

**影響**:
- 後方互換性はEVENT.fireで一元管理
- 新規ハンドラはthread返却必須

### Topic 3: virtual_dispatcher設計

**調査範囲**: check_talk/check_hour/dispatch関数の変更要件

**変更方針**:
1. `execute_scene()` → 実行せずthreadを返す形式に変更
2. `check_talk()` - STORE.co_scene確認ロジック追加（チェイントーク継続）
3. `check_hour()` - 変更なし（通常イベントとして毎回完結）

**チェイントーク継続フロー**:
```
OnTalk発火時:
1. STORE.co_sceneがsuspendedなら、そのthreadを返す
2. nilなら新規シーン検索してthread生成
```

### Topic 4: set_co_scene()ヘルパー関数設計

**調査範囲**: STORE.co_sceneの統一管理

**設計決定**:
```lua
local function set_co_scene(co)
    -- 1. 引数検証（suspended以外はclose）
    if co and coroutine.status(co) ~= "suspended" then
        coroutine.close(co)
        co = nil
    end

    -- 2. 同一オブジェクトチェック
    if STORE.co_scene == co then
        return
    end

    -- 3. 旧コルーチンをclose（存在すれば無条件）
    if STORE.co_scene then
        coroutine.close(STORE.co_scene)
    end

    -- 4. 上書き
    STORE.co_scene = co
end
```

**設計理由**:
- 引数検証を最初に: dead/running状態のコルーチンはすぐclose
- 同一オブジェクトチェック: 自身をcloseしない
- 無条件close: 旧コルーチンは状態に関わらずclose（suspended以外のケースは稀だが安全策）

---

## アーキテクチャパターン評価

### 選択パターン: 既存コンポーネント拡張（Option A）

**理由**:
- 最小限の新規ファイル
- 既存アーキテクチャを維持
- 後方互換性をEVENT.fireで一元管理
- STOREパターン踏襲

**棄却パターン**: 新規コルーチンマネージャ作成（Option B）
- 過剰な抽象化
- 統合ポイントが増加

---

## 設計決定記録

| DD | 決定 | 理由 |
|----|------|------|
| DD1 | OnTalkのみチェイントーク対応 | OnHourは時報、毎回完結が自然 |
| DD2 | 新規ハンドラはthread必須 | EVENT.fireで後方互換を一元管理 |
| DD3 | エラー時即座にクリア | 安全側、次回OnTalkは新規シーンから |
| DD4 | resume(co, act)でact渡す | 継続時も同じactを使用 |
| DD5 | RES.ok()で空チェック | 統一的な空文字列処理 |

---

## リスクと緩和策

| リスク | 影響度 | 緩和策 |
|--------|--------|--------|
| コルーチンリーク | High | set_co_scene()で統一管理、STORE.reset()でclose |
| 後方互換性破壊 | Medium | EVENT.fireでstring/nil互換を維持 |
| エラー時の状態不整合 | Medium | エラー時即座にclose & クリア |

---

## 次のステップ

1. design.md生成
2. 設計レビュー
3. 実装（STORE → EVENT.fire → virtual_dispatcher → RES.ok → テスト）
