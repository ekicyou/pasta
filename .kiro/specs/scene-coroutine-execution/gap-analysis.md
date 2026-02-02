# Gap Analysis: scene-coroutine-execution

## 1. Current State Investigation

### 1.1 Domain-Related Assets

| アセット | パス | 状態 | 備考 |
|----------|------|------|------|
| COモジュール | `scripts/pasta/co.lua` | ✅ 存在 | `CO.safe_wrap()` 既に実装済み |
| STOREモジュール | `scripts/pasta/store.lua` | ✅ 存在 | `co_handler` フィールド未定義 |
| EVENTモジュール | `scripts/pasta/shiori/event/init.lua` | ✅ 存在 | コルーチン対応なし |
| ShioriActクラス | `scripts/pasta/shiori/act.lua` | ✅ 存在 | `act:yield()` 既に実装済み |
| VirtualDispatcher | `scripts/pasta/shiori/event/virtual_dispatcher.lua` | ✅ 存在 | シーン直接実行中 |
| RESモジュール | `scripts/pasta/shiori/res.lua` | ✅ 存在 | `RES.ok()`, `RES.no_content()` 利用可能 |
| SCENEモジュール | `scripts/pasta/scene.lua` | ✅ 存在 | `SCENE.search()` 関数あり |
| second_change.lua | `scripts/pasta/shiori/event/second_change.lua` | ✅ 存在 | dispatcher結果の処理要改修 |

### 1.2 Existing Implementations (Key Discovery)

#### CO.safe_wrap() - 既存実装（完全適合）

```lua
-- scripts/pasta/co.lua より
function CO.safe_wrap(func)
    local co = coroutine.create(func)
    return function(...)
        if coroutine.status(co) == "dead" then
            return nil, "dead"
        end
        local results = { coroutine.resume(co, ...) }
        local ok = results[1]
        if not ok then
            return nil, results[2]  -- エラー
        else
            local status = coroutine.status(co)
            table.remove(results, 1)
            if status == "suspended" then
                return "yield", table.unpack(results)
            else
                return "return", table.unpack(results)
            end
        end
    end
end
```

**分析**: 要件1の `CO.safe_wrap()` は**既に完全実装済み**。戻り値は `(status, value)` 形式で、要件と完全一致。

#### act:yield() - 既存実装

```lua
-- scripts/pasta/shiori/act.lua:186 より
function SHIORI_ACT_IMPL.yield(self)
    local script = self:build()
    coroutine.yield(script)
    return self
end
```

**分析**: 要件6の `act:yield()` は**既に実装済み**。`build()` を呼び出してからyieldし、リセット済みselfを返す。

### 1.3 Conventions & Patterns

| 項目 | 規約 |
|------|------|
| モジュールテーブル命名 | UPPER_CASE（例: `EVENT`, `STORE`, `RES`） |
| 循環参照回避 | STOREは他モジュールをrequireしない |
| テストパターン | lua_testフレームワーク使用、BDDスタイル |
| エラーハンドリング | 上位のxpcallでキャッチ |

### 1.4 Integration Surfaces

- **EVENT.fire** ← **REG[req.id]** ハンドラテーブル
- **second_change.lua** → **dispatcher.dispatch(act)**
- **dispatcher** → **SCENE.search()** → **scene_fn(act)**

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs from Requirements

| 要件 | 技術的必要性 | 既存資産 | ギャップ |
|------|-------------|----------|---------|
| Req1: COモジュール | `CO.safe_wrap()` | ✅ 完全実装済み | なし |
| Req2: EVENT.fire改良 | コルーチン処理ロジック | ❌ 未対応 | **新規実装必要** |
| Req3: ハンドラ変換 | `CO.safe_wrap()` でラップ | ❌ 未対応 | **改修必要** |
| Req4: dispatch()改良 | co_handler返却 | ❌ 直接実行中 | **改修必要** |
| Req5: チェイントーク | `STORE.co_handler` 管理 | ❌ フィールドなし | **追加必要** |
| Req6: act:yield() | `coroutine.yield()` 呼び出し | ✅ 実装済み | なし |
| Req7: STORE拡張 | `co_handler` フィールド | ❌ 未定義 | **追加必要** |
| Req8: E2Eテスト | テストケース | ❌ 未作成 | **新規作成必要** |

### 2.2 Gap Summary

| ステータス | 項目数 | 詳細 |
|-----------|--------|------|
| ✅ 既存で充足 | 2 | CO.safe_wrap(), act:yield() |
| 🔧 改修必要 | 4 | EVENT.fire, dispatch(), check_hour, check_talk |
| ➕ 新規追加 | 2 | STORE.co_handler, E2Eテスト |

### 2.3 Complexity Signals

- **アルゴリズムロジック**: コルーチン状態管理（中程度）
- **インテグレーション**: EVENT → dispatcher → SCENE チェーン改修（中程度）
- **テスト**: 既存BDDフレームワーク活用可能（低）

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components (推奨)

**概要**: 既存ファイルを最小限改修し、コルーチン対応を追加

**変更対象ファイル**:

| ファイル | 変更内容 | 影響度 |
|----------|----------|--------|
| `store.lua` | `STORE.co_handler = nil` 追加、`reset()` 修正 | 低 |
| `event/init.lua` | `EVENT.fire` のコルーチン処理ロジック追加 | 中 |
| `virtual_dispatcher.lua` | `execute_scene` → co_handler返却、check_talk改修 | 中 |
| `second_change.lua` | dispatcher結果処理の改修 | 低 |

**Trade-offs**:
- ✅ 既存パターンを維持、学習コスト低
- ✅ ファイル増加なし
- ✅ 既存テストとの互換性維持しやすい
- ❌ EVENT.fireの責務が増加（コルーチン管理）

### Option B: Create New Components

**概要**: コルーチン管理専用モジュールを新規作成

**新規ファイル**:
- `scripts/pasta/shiori/coroutine_manager.lua` - コルーチン状態管理

**Trade-offs**:
- ✅ 責務分離が明確
- ✅ テスト容易性向上
- ❌ ファイル増加
- ❌ 既存フローとの統合ポイント増加
- ❌ 循環参照リスク（STOREとの関係）

### Option C: Hybrid Approach

**概要**: 状態管理はSTORE拡張、処理ロジックは既存ファイル改修

**方針**:
1. `STORE.co_handler` を追加（状態管理はSTORE一元化）
2. `EVENT.fire` にコルーチン処理ロジック追加
3. `virtual_dispatcher` はハンドラ返却に改修
4. `second_change` はEVENT.fireに処理委譲

これは実質 **Option A** と同等だが、STORE一元化を明示的に設計原則とする。

---

## 4. Implementation Complexity & Risk

### Effort Estimate: **M (3-7 days)**

**根拠**:
- 既存モジュール（CO, act:yield）が再利用可能
- 改修対象は4-5ファイル、各ファイル中程度の変更
- テスト作成に1-2日

### Risk Level: **Medium**

**根拠**:
- **既知技術**: Lua coroutineは既にCOモジュールで使用実績あり
- **統合複雑性**: EVENT → dispatcher → SCENE のチェーンを改修
- **リグレッションリスク**: 既存イベント処理への影響（テストでカバー可能）

**リスク軽減策**:
- 既存テスト（virtual_dispatcher_spec.lua）の拡張
- 段階的実装（STORE → dispatcher → EVENT.fire）

---

## 5. Recommendations for Design Phase

### Preferred Approach: **Option A（Extend Existing Components）**

**理由**:
1. CO.safe_wrap() / act:yield() が既に完全実装済み（50%のコード資産活用）
2. 既存のモジュール構造・循環参照回避パターンを維持
3. 変更範囲が明確で、リグレッションテストが容易

### Key Design Decisions

1. **EVENT.fire の責務拡張**: コルーチン状態管理をEVENT.fireに集約
2. **dispatcher の役割変更**: シーン実行 → ハンドラ取得・返却
3. **second_change の簡素化**: dispatcher結果をそのままEVENT.fireに渡す

### Research Items to Carry Forward

| 項目 | 詳細 | 優先度 |
|------|------|--------|
| OnHour継続可否 | OnHourもチェイントーク対象か？ | 中（設計時確認） |
| エラー時のco_handler処理 | エラー発生時にco_handlerをクリアするか？ | 高 |
| REGハンドラのco_handler対応 | REG[req.id]ハンドラもco_handler化するか？ | 高（設計時決定） |

---

## 6. Appendix: File Modification Map

```
scripts/pasta/
├── co.lua                          # ✅ 変更不要（既存実装で充足）
├── store.lua                       # 🔧 co_handler フィールド追加
└── shiori/
    ├── act.lua                     # ✅ 変更不要（yield()実装済み）
    ├── res.lua                     # ✅ 変更不要
    └── event/
        ├── init.lua                # 🔧 EVENT.fire コルーチン対応
        ├── register.lua            # ✅ 変更不要
        ├── second_change.lua       # 🔧 dispatcher結果処理改修
        └── virtual_dispatcher.lua  # 🔧 co_handler返却、check_talk改修

tests/lua_specs/
└── coroutine_chain_spec.lua        # ➕ 新規作成（E2Eテスト）
```
