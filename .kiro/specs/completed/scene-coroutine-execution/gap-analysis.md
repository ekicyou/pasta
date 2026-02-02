# ギャップ分析: scene-coroutine-execution

**分析日**: 2026-02-02
**ステータス**: 再分析完了（要件更新後）

## 1. 分析サマリー

### 主要発見事項

1. **act:yield()は既に実装済み** - SHIORI_ACT_IMPL.yield()がcoroutine.yield()を呼び出す実装が存在
2. **STOREにco_sceneフィールドなし** - 追加が必要
3. **EVENT.fireはhandlerを直接呼び出しているのみ** - thread判定・resume処理の追加が必要
4. **virtual_dispatcherはシーンを直接実行している** - threadを返す形式に変更が必要
5. **EVENT.no_entryもシーンを直接実行している** - threadを返す形式に変更が必要
6. **CO.safe_wrap()は使用しない** - coroutine.create()直接管理でcoroutine.close()を保証

### 実装複雑度

- **Effort**: M（3〜7日）- 既存パターンの拡張だが、複数モジュールにまたがる変更
- **Risk**: Medium - コルーチン管理は新パターンだが、Luaの標準APIを使用

---

## 2. 要件-アセットマップ

| 要件 | 関連アセット | ギャップ |
|------|-------------|---------|
| R1: コルーチン直接管理 | (新規パターン) | **New** - coroutine.create/resume/close使用パターンを導入 |
| R2: EVENT.fire拡張 | `pasta/shiori/event/init.lua` | **Missing** - thread判定、resume、状態保存ロジック |
| R3: ハンドラ戻り値 | virtual_dispatcher, EVENT.no_entry | **Change** - 実行からthread返却に変更 |
| R4: virtual_dispatcher改良 | `pasta/shiori/event/virtual_dispatcher.lua` | **Change** - execute_scene()をthread生成に変更 |
| R5: チェイントーク継続 | check_talk | **Missing** - STORE.co_scene確認ロジック |
| R6: act:yield() | `pasta/shiori/act.lua` L184-188 | **Exists** - ✅ 既存実装で対応可 |
| R7: STOREモジュール | `pasta/store.lua` | **Missing** - co_sceneフィールド、reset()のclose処理 |
| R8: テスト | (新規) | **New** - 統合テスト作成が必要 |

---

## 3. 既存コード分析

### 3.1 EVENT.fire (init.lua L102-112)

**現状**:
```lua
function EVENT.fire(req)
    local act = create_act(req)
    local handler = REG[req.id] or EVENT.no_entry
    return handler(act)  -- ← 直接呼び出し、戻り値をそのまま返す
end
```

**変更必要箇所**:
- handler(act)の戻り値がthreadかstring/nilか判定
- threadの場合: coroutine.resume(result, act)を実行
- resume後のstatus確認（suspended→STORE.co_scene保存、dead→クリア）
- エラー処理: coroutine.close()でリソース解放

### 3.2 EVENT.no_entry (init.lua L82-98)

**現状**:
```lua
function EVENT.no_entry(act)
    local SCENE = require("pasta.scene")
    local scene_result = SCENE.search(act.req.id, nil, nil)
    if scene_result then
        scene_result()  -- ← 直接実行
    end
    return RES.no_content()
end
```

**変更必要箇所**:
- scene_resultが見つかった場合: coroutine.create(scene_result)でthreadを返す
- 見つからない場合: nilを返す（EVENT.fireがno_content処理）

### 3.3 virtual_dispatcher execute_scene (L73-82)

**現状**:
```lua
local function execute_scene(event_name, act)
    if scene_executor then
        return scene_executor(event_name, act)
    end
    local SCENE = require("pasta.scene")
    local scene_fn = SCENE.search(event_name, nil, nil)
    if not scene_fn then return nil end
    return scene_fn(act)  -- ← 直接実行
end
```

**変更必要箇所**:
- 関数名を`create_scene_thread`などに変更検討
- scene_fnが見つかった場合: coroutine.create()でthreadを返す

### 3.4 check_talk (L126-159)

**現状**:
```lua
function M.check_talk(act)
    -- 時刻判定ロジック...
    local result = execute_scene("OnTalk", act)
    next_talk_time = calculate_next_talk_time(current_unix)
    return result and "fired" or nil
end
```

**変更必要箇所**（R5: チェイントーク継続）:
- 最初にSTORE.co_sceneを確認
- co_sceneがsuspendedなら、そのthreadを返す（新規シーン検索スキップ）
- co_sceneがnilなら、新規シーン検索してthread生成

### 3.5 STORE (store.lua)

**現状**:
- co_sceneフィールドなし
- reset()にclose処理なし

**変更必要箇所**:
- `STORE.co_scene = nil` フィールド追加
- reset()内でSTORE.co_sceneがsuspendedならcoroutine.close()してからnil

### 3.6 act:yield() (shiori/act.lua L184-188)

**現状**:
```lua
function SHIORI_ACT_IMPL.yield(self)
    local script = self:build()
    coroutine.yield(script)
    return self
end
```

**ステータス**: ✅ **既存実装で対応可能**
- build()でさくらスクリプトを生成
- coroutine.yield()でスクリプト文字列をyield
- 再開後にself（リセット済み）を返す

---

## 4. 実装アプローチ評価

### Option A: 既存コンポーネント拡張（推奨）

**変更対象**:
1. `pasta/shiori/event/init.lua` - EVENT.fire, EVENT.no_entry
2. `pasta/shiori/event/virtual_dispatcher.lua` - execute_scene → create_scene_thread
3. `pasta/store.lua` - co_sceneフィールド、reset()

**Trade-offs**:
- ✅ 最小限の新規ファイル
- ✅ 既存アーキテクチャを維持
- ✅ 後方互換性をEVENT.fireで一元管理
- ❌ 複数ファイルにまたがる変更

**推奨度**: ⭐⭐⭐⭐⭐

### Option B: 新規コルーチンマネージャ作成

**新規作成**:
- `pasta/shiori/coroutine_manager.lua` - コルーチン管理専用モジュール

**Trade-offs**:
- ✅ コルーチン管理ロジックを一箇所に集約
- ❌ 過剰な抽象化（現段階では不要）
- ❌ 既存コードとの統合ポイントが増える

**推奨度**: ⭐⭐

---

## 5. 実装順序（推奨）

1. **STORE拡張** - co_sceneフィールド追加、reset()にclose処理
2. **EVENT.fire拡張** - thread判定、resume、状態保存
3. **EVENT.no_entry変更** - thread返却
4. **virtual_dispatcher変更** - thread返却、check_talkにチェイントーク継続
5. **統合テスト** - E2Eテスト作成

---

## 6. 潜在的課題（設計フェーズで検討）

### 6.1 actオブジェクトのスコープ

**課題**: check_talkでSTORE.co_sceneを返す場合、前回のactと今回のactが異なる可能性

**要検討**: 
- コルーチン再開時に新しいactをresume引数として渡す設計が必要
- シーン関数側でact = coroutine.yield(script)のパターンで更新されたactを受け取る

**現状のact:yield()実装**:
```lua
function SHIORI_ACT_IMPL.yield(self)
    local script = self:build()
    coroutine.yield(script)  -- ← 戻り値を無視している
    return self  -- ← 古いselfを返す
end
```

**潜在的問題**: 再開時にactが更新されない可能性あり → 設計フェーズで検討

### 6.2 check_hour/check_talkの戻り値統一

**現状**: 
- check_hour: `"fired"` or `nil`
- check_talk: `"fired"` or `nil`

**変更後**: 両方ともthread or nilを返す

**影響**: dispatch()の戻り値処理も変更が必要

### 6.3 シーン実行と時刻更新のタイミング

**現状**: check_talkは実行後にnext_talk_timeを更新

**変更後**: threadを返すだけなので、時刻更新タイミングを検討
- 選択肢A: thread返却時に更新（現状踏襲）
- 選択肢B: resume完了後に更新（EVENT.fire側で制御）

---

## 7. ファイル変更マップ

```
scripts/pasta/
├── co.lua                          # ⚠️ 使用しない（coroutine.create直接管理）
├── store.lua                       # 🔧 co_scene フィールド追加、reset()改修
└── shiori/
    ├── act.lua                     # ✅ 変更不要（yield()実装済み）
    ├── res.lua                     # ✅ 変更不要
    └── event/
        ├── init.lua                # 🔧 EVENT.fire, EVENT.no_entry 改修
        ├── register.lua            # ✅ 変更不要
        ├── second_change.lua       # 🔧 dispatcher結果処理改修（必要に応じて）
        └── virtual_dispatcher.lua  # 🔧 thread返却、check_talk改修

tests/lua_specs/
└── coroutine_chain_spec.lua        # ➕ 新規作成（E2Eテスト）
```

---

## 8. 次のステップ

1. 上記の潜在的課題について開発者と議論
2. `/kiro-spec-design scene-coroutine-execution` で設計ドキュメント生成
3. 設計レビュー後に実装開始
