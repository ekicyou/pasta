# Gap Analysis: actor-spot-refactoring

## 1. Current State Investigation

### 影響ファイル一覧

| ファイル | 役割 | 変更種別 |
|----------|------|----------|
| `crates/pasta_lua/scripts/pasta/act.lua` | トークン生成の親クラス | 修正 |
| `crates/pasta_lua/scripts/pasta/shiori/sakura_builder.lua` | トークン→さくらスクリプト変換 | 修正 |
| `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/scripts/pasta/shiori/act.lua` | SHIORI専用ACT継承クラス | 修正 |
| `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/scripts/pasta/config.lua` | 設定ドキュメント例 | 修正（コメント） |
| `crates/pasta_lua/tests/lua_specs/act_test.lua` | ACTモジュールテスト | 修正 |
| `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua` | sakura_builderテスト | 修正 |

### 現行アーキテクチャパターン

```
[act.lua] talk() → [トークン配列] → [sakura_builder.lua] build() → さくらスクリプト文字列
                   ↑                    ↑
                   actor, spot_switch   spot_switch → \n[N]
                   ↓                    actor.spot → \p[N]
```

**トークン生成フロー（act.lua:76-83）**:
```lua
if self.now_actor ~= actor then
    table.insert(self.token, { type = "actor", actor = actor })
    local spot_id = actor.spot or 0
    if self._current_spot ~= nil and self._current_spot ~= spot_id then
        table.insert(self.token, { type = "spot_switch" })  -- ← リネーム対象
    end
    self._current_spot = spot_id
    self.now_actor = actor
end
```

**トークン処理フロー（sakura_builder.lua:60-65）**:
```lua
if t == "actor" then
    local spot_id = spot_to_id(token.actor.spot)
    table.insert(buffer, spot_to_tag(spot_id))
elseif t == "spot_switch" then  -- ← リネーム対象
    local percent = math.floor(spot_switch_newlines * 100)
    table.insert(buffer, string.format("\\n[%d]", percent))
```

### 命名規則・パターン

| 項目 | 現行 | 提案 |
|------|------|------|
| トークンタイプ | `"spot_switch"` | `"spot"` |
| 設定プロパティ | `spot_switch_newlines` | `spot_newlines` |
| ローカル変数 | `spot_switch_newlines` | `spot_newlines` |
| フィールド | `_spot_switch_newlines` | `_spot_newlines` |

## 2. Requirements Feasibility Analysis

### 技術要件マッピング

| 要件 | 技術ニーズ | ギャップ状態 |
|------|------------|--------------|
| Req 1: トークンタイプ名変更 | 文字列リテラル置換 | **対応可能** - 単純なリネーム |
| Req 2: spotトークン責務明確化 | 既存ロジック維持 | **対応可能** - 変更なし |
| Req 3: 設定プロパティ名一貫性 | 設定キー置換 | **対応可能** - 単純なリネーム |
| Req 4: テスト更新 | テスト文字列置換 | **対応可能** - 単純なリネーム |
| Req 5: actor/spot独立性 | 既存設計維持 | **対応可能** - 変更なし |

### 依存関係分析

```
pasta.act (親) ← pasta.shiori.act (子/サンプルゴースト)
      ↓                   ↓
   トークン生成       さくらスクリプト直接生成
      ↓
sakura_builder (トークン変換)
```

**注意点**: 
- `pasta_sample_ghost`内の`pasta.shiori.act`は独自実装を持ち、`spot_switch_newlines`設定を直接参照している
- 設定ファイル（pasta.toml）の`ghost.spot_switch_newlines`キーも変更が必要

### 制約事項

1. **後方互換性**: 設定プロパティ名変更は既存ゴーストに影響
   - オプション: 旧プロパティ名をフォールバックとしてサポート
   - 推奨: Out of Scopeとして明記済み、破壊的変更として許容

2. **サンプルゴースト連動**: `pasta_sample_ghost`内の実装も更新必須

## 3. Implementation Approach Options

### Option A: 単純リネーム（推奨）

**概要**: すべての`spot_switch`関連文字列を`spot`にリネーム

**変更箇所**:
1. `act.lua`: 1箇所（L82: `type = "spot_switch"` → `type = "spot"`）
2. `sakura_builder.lua`: 4箇所（トークンタイプ、設定、変数、ドキュメント）
3. `pasta.shiori.act.lua`（サンプル）: 3箇所（フィールド、設定読み込み、ドキュメント）
4. `act_test.lua`: 6箇所（テストケース名、期待値）
5. `sakura_builder_test.lua`: 10箇所（テストケース名、トークン、設定）
6. `config.lua`（サンプル）: 1箇所（ドキュメントコメント）

**Trade-offs**:
- ✅ 最小の変更量
- ✅ ロジック変更なし
- ✅ テスト修正のみで既存動作保証
- ❌ 後方互換性なし（意図通り）

### Option B: 後方互換フォールバック

**概要**: 新プロパティ名を優先しつつ旧プロパティ名もサポート

**追加変更**:
```lua
local spot_newlines = config.spot_newlines or config.spot_switch_newlines or 1.5
```

**Trade-offs**:
- ✅ 既存ゴーストの設定が継続動作
- ❌ コード複雑化
- ❌ 非推奨プロパティの永続化リスク

### Option C: ハイブリッド（段階的移行）

**概要**: 
1. Phase 1: トークンタイプのみリネーム
2. Phase 2: 設定プロパティリネーム（deprecation warning付き）
3. Phase 3: 旧プロパティ削除

**Trade-offs**:
- ✅ 移行期間を確保
- ❌ 複数リリースにまたがる
- ❌ 現状のプロジェクト規模に対してオーバーエンジニアリング

## 4. Implementation Complexity & Risk

### Effort: **S (1-2 days)**

**理由**:
- 全て文字列リテラルのリネーム
- ロジック変更なし
- 既存テストが変更の正確性を保証
- 影響範囲が明確で限定的

### Risk: **Low**

**理由**:
- 確立されたパターンの単純変更
- 既存テストスイートによるリグレッション検出
- ブレーキングチェンジは意図的かつスコープ内
- 外部依存なし

## 5. Recommendations for Design Phase

### 推奨アプローチ: **Option A（単純リネーム）**

**根拠**:
1. プロジェクトは開発初期段階（Phase 2）であり、後方互換性より明確性を優先
2. 影響を受けるのは`pasta_sample_ghost`のみで、これも同時更新可能
3. 変更量が少なく、リスクが低い

### 設計フェーズへの引き継ぎ事項

| カテゴリ | 項目 |
|----------|------|
| 決定事項 | Option Aを採用 |
| 確認不要 | 外部研究不要 |
| 実装順序 | 1. act.lua → 2. sakura_builder.lua → 3. サンプルゴースト → 4. テスト |
| 検証方法 | 既存テストスイート実行（`cargo test`） |

## 6. 変更影響サマリー

```
変更ファイル数: 6
変更箇所数: 約25箇所
テスト影響: テストコードのみ変更、ロジック変更なし
破壊的変更: 設定プロパティ名（意図的、Out of Scope明記）
```
