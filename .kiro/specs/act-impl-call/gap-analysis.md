# Gap Analysis: act-impl-call

## 1. 現状調査

### 1.1 関連ファイル・モジュール構成

| ファイル                                                                                                           | 責務                                                           | 変更必要性                       |
| ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------- | -------------------------------- |
| [crates/pasta_lua/scripts/pasta/act.lua](crates/pasta_lua/scripts/pasta/act.lua)                                   | `ACT_IMPL.call`実装（現在はシンプルな`SCENE.get`呼び出しのみ） | ✅ **主要変更対象**               |
| [crates/pasta_lua/scripts/pasta/scene.lua](crates/pasta_lua/scripts/pasta/scene.lua)                               | `SCENE.search(name, global_scene_name)` - 2引数シグネチャ      | ⚠️ 第3引数`attrs`追加が必要       |
| [crates/pasta_lua/scripts/pasta/global.lua](crates/pasta_lua/scripts/pasta/global.lua)                             | ユーザー定義グローバル関数テーブル                             | 変更不要                         |
| [crates/pasta_shiori/tests/support/scripts/pasta/act.lua](crates/pasta_shiori/tests/support/scripts/pasta/act.lua) | pasta_shiori用の互換実装                                       | ⚠️ 同期が必要                     |
| [crates/pasta_lua/src/code_generator.rs](crates/pasta_lua/src/code_generator.rs)                                   | トランスパイラ出力生成                                         | 変更不要（現在の出力形式で互換） |

### 1.2 現在の実装

**act.lua の現在の `ACT_IMPL.call`** (L120-126):
```lua
function ACT_IMPL.call(self, search_result, opts, ...)
    local global_name, local_name = search_result[1], search_result[2]
    local scene_func = SCENE.get(global_name, local_name)
    if scene_func then
        scene_func(self, ...)
    end
end
```

**問題点**:
- `search_result` は `{global_name, local_name}` 形式を期待
- トランスパイラ出力は `act:call(SCENE.__global_name__, "ラベル名", {}, ...)` 形式
- **シグネチャ不一致**: 現在の実装は `(self, search_result, opts, ...)` だが、要件は `(self, global_scene_name, key, attrs, ...)`

### 1.3 トランスパイラ出力形式

`code_generator.rs` (L445-448):
```rust
let call_stmt = format!(
    "act:call(SCENE.__global_name__, \"{}\", {{}}, {})",
    target, args_str
);
```

**生成例**:
```lua
act:call(SCENE.__global_name__, "グローバル単語呼び出し", {}, table.unpack(args))
```

**第1引数**: `SCENE.__global_name__` (string) - グローバルシーン名
**第2引数**: `"ラベル名"` (string) - 検索キー
**第3引数**: `{}` (table) - 属性（現在は空テーブル）
**第4引数以降**: 可変長引数

### 1.4 既存パターン参照

**actor.lua の `ACTOR_PROXY_IMPL.word` (L180-220)** は類似の多段検索を実装:
1. Level 1-2: アクター辞書
2. Level 3-4: シーン（完全一致→前方一致）
3. Level 5-6: グローバル（完全一致→前方一致）

この実装パターンを `ACT_IMPL.call` に適用可能。

---

## 2. 要件対現状ギャップ分析

| 要件                                                               | 現状                               | ギャップ                   | ステータス            |
| ------------------------------------------------------------------ | ---------------------------------- | -------------------------- | --------------------- |
| **Req 1**: シグネチャ `(self, global_scene_name, key, attrs, ...)` | `(self, search_result, opts, ...)` | シグネチャ変更必要         | ⚠️ **Breaking Change** |
| **Req 2-1**: Level 1 `self.current_scene[key]`                     | 未実装                             | 新規追加                   | Missing               |
| **Req 2-2**: Level 2 `SCENE.search(key, global_scene_name, attrs)` | `SCENE.search`は2引数              | 第3引数追加必要            | Missing               |
| **Req 2-3**: Level 3 `require("pasta.global")[key]`                | 未実装                             | 新規追加                   | Missing               |
| **Req 2-4**: Level 4 `SCENE.search(key, nil, attrs)`               | 未実装                             | 新規追加                   | Missing               |
| **Req 3**: ハンドラー実行 `handler(act, ...)`                      | `scene_func(self, ...)`            | 実装済み（変数名変更のみ） | ✅ 互換                |
| **Req 4**: attrs渡し                                               | `SCENE.search`が2引数              | 第3引数サポート必要        | Missing               |
| **Req 5**: 後方互換性                                              | 現在の出力と新シグネチャ互換       | 互換性あり                 | ✅ 互換                |
| **Req 6**: ログ拡張ポイント                                        | 未実装                             | TODOコメント追加           | Missing               |

### 重要な発見

**シグネチャ変更は破壊的変更ではない**:
- トランスパイラ出力 `act:call(SCENE.__global_name__, "key", {}, ...)` は新シグネチャ `(self, global_scene_name, key, attrs, ...)` と完全一致
- 現在の実装が間違っている（`search_result`配列を期待しているが、実際は個別引数で渡される）

---

## 3. 実装アプローチオプション

### Option A: 既存ファイル拡張（推奨）

**act.lua の `ACT_IMPL.call` を直接置き換え**

| 項目                 | 内容                                         |
| -------------------- | -------------------------------------------- |
| **変更ファイル**     | `act.lua` (1ファイル)                        |
| **変更行数**         | 約30行（関数全体置換 + requireパターン追加） |
| **SCENE.search変更** | 第3引数`attrs`を追加（現在は無視、将来対応） |

**実装イメージ**:
```lua
function ACT_IMPL.call(self, global_scene_name, key, attrs, ...)
    -- Level 1: シーンローカル
    local handler = self.current_scene and self.current_scene[key]
    
    -- Level 2: グローバルシーン名スコープ検索
    if not handler then
        local result = SCENE.search(key, global_scene_name, attrs)
        handler = result and result.func
    end
    
    -- Level 3: グローバル関数モジュール
    if not handler then
        local GLOBAL = require("pasta.global")
        handler = GLOBAL[key]
    end
    
    -- Level 4: スコープなし全体検索
    if not handler then
        local result = SCENE.search(key, nil, attrs)
        handler = result and result.func
    end
    
    -- ハンドラー実行
    if handler then
        return handler(self, ...)
    end
    -- TODO: ログ出力（将来対応）
    return nil
end
```

**Trade-offs**:
- ✅ 最小限の変更
- ✅ 既存パターン（actor.lua）との一貫性
- ✅ テスト容易
- ❌ SCENE.searchのシグネチャ変更が必要

### Option B: 検索ヘルパー関数分離

**検索ロジックを別関数として分離**

| 項目             | 内容                                                         |
| ---------------- | ------------------------------------------------------------ |
| **新規関数**     | `ACT_IMPL.find_handler(self, global_scene_name, key, attrs)` |
| **変更ファイル** | `act.lua` (1ファイル)                                        |
| **テスト容易性** | 検索ロジックを独立テスト可能                                 |

**Trade-offs**:
- ✅ 単一責任原則
- ✅ 検索ロジックの独立テスト
- ❌ 追加の関数呼び出しオーバーヘッド（微小）

### Option C: ハイブリッド（段階的実装）

**Phase 1**: シグネチャ変更のみ（現在の検索ロジック維持）
**Phase 2**: 4段階検索の完全実装

**Trade-offs**:
- ✅ リスク分散
- ❌ 中間状態が存在
- ❌ 2回のレビューサイクル必要

---

## 4. 追加調査事項

### SCENE.searchの第3引数対応

**現状**: `SCENE.search(name, global_scene_name)` - 2引数

**必要変更**:
```lua
function SCENE.search(name, global_scene_name, attrs)  -- 第3引数追加
    -- attrs は将来対応のため現在は無視
    -- ...既存ロジック...
end
```

**影響範囲**:
- scene.lua: シグネチャ変更のみ
- 呼び出し元: 2引数呼び出しは互換（Luaは余剰引数を無視）

### pasta_shiori互換実装

**ファイル**: `crates/pasta_shiori/tests/support/scripts/pasta/act.lua`

この互換実装も同期が必要。テスト用サポートファイルのため、優先度は低いが、テスト失敗を避けるため同時更新推奨。

---

## 5. 複雑度・リスク評価

| 項目       | 評価          | 理由                                                       |
| ---------- | ------------- | ---------------------------------------------------------- |
| **工数**   | **S (1-3日)** | 既存パターン（actor.lua）の流用可能、単一ファイル変更      |
| **リスク** | **Low**       | 既存テスト基盤あり、パターン確立済み、シグネチャ変更は互換 |

---

## 6. 推奨事項

### 推奨アプローチ: **Option A（既存ファイル拡張）**

**理由**:
1. 変更範囲が最小限（act.lua + scene.luaシグネチャ）
2. actor.luaの多段検索パターンを流用可能
3. トランスパイラ出力との互換性が確保されている
4. 既存テスト（scene_search_test.rs）を活用可能

### 設計フェーズでの検討事項

1. **SCENE.search第3引数の仕様確定**: 将来の属性フィルタリングの具体的なユースケース
2. **ログ出力の設計**: tracingクレートとの統合方法
3. **pasta_shiori互換実装の同期戦略**: 同時更新 or 別タスク化
4. **テスト戦略**: ユニットテスト（Lua単体）と統合テスト（Rust経由）の両方

### Research Needed

- [ ] `attrs`パラメータの将来的なフィルタリング仕様（設計フェーズで検討）
