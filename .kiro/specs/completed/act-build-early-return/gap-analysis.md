# Gap Analysis: act-build-early-return

## 分析サマリー

- **スコープ**: ACT:build()とSHIORI_ACT:build()にnil早期リターンを導入し、撮影トークン0件時のパフォーマンス最適化と会話未作成検出を可能にする
- **主要課題**: 
  - 既存実装は空配列を返す前提で設計されており、nilリターンはBreaking Changeとなる
  - 呼び出し元（OnTalk/OnHourシーンハンドラ、yield()パターン）でのnil処理追加が必要
  - 既存テストケース（20件以上）が非nil前提で記述されている
- **推奨アプローチ**: Option A（既存関数拡張）- 最小変更でBreaking Changeの影響範囲を明確化し、段階的対応が可能

---

## 1. Current State Investigation

### 1.1 Key Files & Modules

| ファイル                                                             | 責務                       | 現在の実装                                                                |
| -------------------------------------------------------------------- | -------------------------- | ------------------------------------------------------------------------- |
| `crates/pasta_lua/scripts/pasta/act.lua`                             | ACT:build()実装            | L282-292: トークン取得→グループ化→統合→返却（空配列を含む）               |
| `crates/pasta_lua/scripts/pasta/shiori/act.lua`                      | SHIORI_ACT:build()実装     | L56-63: ACT.IMPL.build()呼び出し→BUILDER.build()変換→スクリプト文字列返却 |
| `crates/pasta_lua/scripts/pasta/shiori/event/init.lua`               | イベントハンドラ（メイン） | L40: `RES.ok(act:build())`パターン（nil非対応）                           |
| `crates/pasta_lua/scripts/pasta/shiori/event/virtual_dispatcher.lua` | OnTalk/OnHour発行          | シーン実行後に`act:build()`を呼び出し（nil非対応の可能性）                |
| `crates/pasta_lua/tests/lua_specs/act_test.lua`                      | ACT:build()テスト          | L410-442: 20件以上のテストケース（全て非nil前提）                         |
| `crates/pasta_lua/tests/lua_specs/shiori_act_test.lua`               | SHIORI_ACT:build()テスト   | L70-380: 22件のテストケース（全て非nil前提）                              |

### 1.2 Architecture Patterns

**既存のbuild()パターン:**
```lua
-- ACT_IMPL.build() (act.lua:282)
function ACT_IMPL.build(self)
    local tokens = self.token
    self.token = {}

    -- Phase 1: アクター切り替え境界でグループ化
    local grouped = group_by_actor(tokens)

    -- Phase 2: 連続talkを統合
    local merged = merge_consecutive_talks(grouped)

    return merged  -- 常にtable[]を返す（空配列含む）
end
```

**SHIORI_ACT_IMPL.build()パターン:**
```lua
-- SHIORI_ACT_IMPL.build() (shiori/act.lua:56)
function SHIORI_ACT_IMPL.build(self)
    -- 親のbuild()でトークン取得＆リセット
    local token = ACT.IMPL.build(self)
    -- sakura_builderで変換（新プロパティ名spot_newlinesを使用）
    local script = BUILDER.build(token, {
        spot_newlines = self._spot_newlines
    })
    return script  -- 常にstring型を返す
end
```

**呼び出し元パターン（イベントハンドラ）:**
```lua
-- init.lua:40 (ドキュメントコメント)
REG.OnBoot = function(act)
    act.sakura:talk("こんにちは")
    return RES.ok(act:build())  -- nil非対応
end
```

### 1.3 Naming & Coding Conventions

- **メソッド定義**: ドット構文 + 明示的self（`function IMPL.method(self, arg)`）
- **型アノテーション**: LuaLS形式（`--- @return table[]`, `--- @return string`）
- **エラーハンドリング**: pcallまたは明示的なnil検証
- **テスト**: BDDスタイル（lua_test、describe/test/expect）

### 1.4 Integration Surfaces

**データフロー:**
```
シーン関数 → act:talk/surface/wait (トークン蓄積)
          ↓
    ACT:build() (グループ化・統合)
          ↓
SHIORI_ACT:build() (さくらスクリプト変換)
          ↓
    RES.ok(script) (SHIORI/3.0応答)
```

**依存関係:**
- `ACT.IMPL.build()`: `group_by_actor()`, `merge_consecutive_talks()`に依存（act.lua内部関数）
- `SHIORI_ACT_IMPL.build()`: `ACT.IMPL.build()`, `BUILDER.build()`に依存
- イベントハンドラ: `SHIORI_ACT:build()`, `RES.ok()`に依存

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs

| 要件ID | 技術的必要事項                                 | 現状の実装                  | Gap                       |
| ------ | ---------------------------------------------- | --------------------------- | ------------------------- |
| R1     | ACT:build()でトークン0件判定                   | `#self.token`でカウント可能 | ✅ 実装可能                |
| R1     | nilリターン前にself.tokenリセット              | `self.token = {}`実装済み   | ✅ 既存ロジック流用可      |
| R2     | ACT.IMPL.build()のnil検証                      | 現状は非nil前提             | ❌ nil検証追加必要         |
| R2     | BUILDER.build()スキップ                        | 条件分岐で可能              | ✅ 実装可能                |
| R3     | 型アノテーション更新                           | `@return table[]            | nil`, `@return string     | nil` | ✅ LuaLS対応済み |
| NFR1   | group_by_actor/merge_consecutive_talksスキップ | 早期リターンで自動スキップ  | ✅ 実装可能                |
| NFR2   | 呼び出し元のnil処理                            | 未実装                      | ❌ Breaking Change対応必要 |

### 2.2 Missing Capabilities

**呼び出し元のnil処理（Breaking Change）:**
- `init.lua:40`: `RES.ok(act:build())`がnilを受け取った場合の挙動未定義
- `virtual_dispatcher.lua`: OnTalk/OnHourシーン実行後の`act:build()`がnilを返す可能性
- 既存テスト: 全て非nil前提で記述されており、nil時の挙動テストが存在しない

**Research Needed:**
- `RES.ok(nil)`の挙動（RES.no_content()等に置き換えるべきか）
- OnTalk/OnHourシーン実行時にトークン0件となる実際のケースの頻度
- BUILDER.build()の内部実装（空配列渡し時のコスト見積もり）

### 2.3 Constraints from Existing Architecture

**制約1: Breaking Change**
- ACT:build()の戻り値型変更（`table[]` → `table[]|nil`）は既存コードに影響する
- 既存のイベントハンドラやテストケースが非nil前提で実装されている
- 段階的移行戦略が必要（後方互換性維持 vs クリーンな変更）

**制約2: テストカバレッジ**
- 既存テスト22件（shiori_act_test.lua）+ 20件（act_test.lua）が全て更新対象
- 新規テストケース（トークン0件時のnil検証）の追加が必要
- 統合テスト（OnTalk/OnHour発動時のnil処理）も追加必要

**制約3: Lua型システム**
- LuaLSの型アノテーションのみで、実行時型チェックはなし
- nilリターンの明示的な検証コード（`== nil`）が必須
- `if not value`パターンはfalseとnilを区別できないため使用不可

### 2.4 Complexity Signals

- **Simple logic**: トークン数カウント（`#self.token == 0`）は単純な条件分岐
- **Integration complexity**: 呼び出し元が多数（イベントハンドラ、テスト）あり、全て確認・修正が必要
- **Test impact**: 既存テスト40件以上の更新が必要、新規テスト10件以上追加推奨

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components ✅ 推奨

**対象ファイル:**
1. `crates/pasta_lua/scripts/pasta/act.lua` - ACT_IMPL.build()
2. `crates/pasta_lua/scripts/pasta/shiori/act.lua` - SHIORI_ACT_IMPL.build()
3. `crates/pasta_lua/scripts/pasta/shiori/event/init.lua` - イベントハンドラ（nil処理追加）
4. `crates/pasta_lua/scripts/pasta/shiori/event/virtual_dispatcher.lua` - OnTalk/OnHourハンドラ（nil処理確認）
5. `crates/pasta_lua/tests/lua_specs/act_test.lua` - 新規テスト追加
6. `crates/pasta_lua/tests/lua_specs/shiori_act_test.lua` - 新規テスト追加

**変更内容（ACT_IMPL.build）:**
```lua
function ACT_IMPL.build(self)
    local tokens = self.token
    self.token = {}

    -- 早期リターン: トークン0件時
    if #tokens == 0 then
        return nil
    end

    -- Phase 1: アクター切り替え境界でグループ化
    local grouped = group_by_actor(tokens)

    -- Phase 2: 連続talkを統合
    local merged = merge_consecutive_talks(grouped)

    return merged
end
```

**変更内容（SHIORI_ACT_IMPL.build）:**
```lua
function SHIORI_ACT_IMPL.build(self)
    -- 親のbuild()でトークン取得＆リセット
    local token = ACT.IMPL.build(self)
    
    -- 早期リターン: tokenがnilの場合
    if token == nil then
        return nil
    end
    
    -- sakura_builderで変換
    local script = BUILDER.build(token, {
        spot_newlines = self._spot_newlines
    })
    return script
end
```

**呼び出し元対応例（init.lua）:**
```lua
REG.OnBoot = function(act)
    act.sakura:talk("こんにちは")
    local script = act:build()
    
    if script == nil then
        return RES.no_content()  -- 204 No Content
    end
    
    return RES.ok(script)
end
```

**互換性評価:**
- ✅ 型アノテーション更新により、LuaLSで型検証可能
- ❌ 既存の呼び出し元コードが非nil前提（nil処理追加必要）
- ✅ 既存テストは引き続きパス（トークンありケース）

**複雑性・保守性:**
- ✅ 既存のbuild()ロジックに3行追加のみ（最小変更）
- ✅ Single Responsibility Principle維持（早期リターンは責務に含まれる）
- ✅ ファイルサイズ影響なし（act.lua 376行、shiori/act.lua 134行）

**Trade-offs:**
- ✅ 最小変更で要件達成可能
- ✅ 既存パターンを踏襲（学習コスト低）
- ❌ Breaking Changeにより呼び出し元修正必須
- ❌ 既存テスト40件以上の影響確認必要

### Option B: Create New Components

**想定構成:**
1. `act_nil_safe.lua` - nil対応版ACT_IMPL.build()ラッパー
2. `shiori_act_nil_safe.lua` - nil対応版SHIORI_ACT_IMPL.build()ラッパー

**実装イメージ:**
```lua
-- act_nil_safe.lua
local ACT = require("pasta.act")

local function build_nil_safe(act)
    if #act.token == 0 then
        act.token = {}
        return nil
    end
    return ACT.IMPL.build(act)
end

return { build = build_nil_safe }
```

**Trade-offs:**
- ✅ 既存コードへの影響なし（後方互換性維持）
- ✅ 段階的移行可能（新規コードからnil_safe版使用）
- ❌ モジュール数増加（act.lua + act_nil_safe.lua）
- ❌ 2つのAPIが共存し、混乱の原因になる
- ❌ 最終的には既存API削除が必要（移行コスト2倍）

**推奨しない理由:**
- Breaking Changeを避けることはできず、最終的には移行必須
- 移行期間中にAPIが2系統存在し、保守性が低下
- 要件では「会話未作成をnil応答で検出可能にする」ことが目的であり、後方互換性維持の必要性が低い

### Option C: Hybrid Approach

**想定構成:**
1. Phase 1: ACT_IMPL.build()とSHIORI_ACT_IMPL.build()を拡張（Option A）
2. Phase 2: 呼び出し元を順次修正（イベントハンドラ → 統合テスト → 単体テスト）
3. Phase 3: テストケース追加・リファクタリング

**段階的実装:**
- Week 1: ACT/SHIORI_ACTの早期リターン実装 + 型アノテーション更新
- Week 2: イベントハンドラのnil処理追加（init.lua, virtual_dispatcher.lua）
- Week 3: 既存テストの影響確認 + 新規テストケース追加

**リスク軽減:**
- 段階的コミット（各フェーズで動作確認）
- cargo test --workspace実行によるリグレッション検出
- nil処理を追加したイベントハンドラから順次デプロイ

**Trade-offs:**
- ✅ Breaking Change影響を段階的に吸収
- ✅ 各フェーズで動作確認可能（ロールバック容易）
- ❌ 実装期間が長くなる（3週間想定）
- ❌ Phase 1完了後もPhase 2未完了状態が存在（不整合リスク）

**推奨しない理由:**
- Option Aと実質同じアプローチ（段階的コミットは標準的な開発プラクティス）
- 複雑なフェーズ管理が不要（変更ファイル数が少ない）
- 要件の緊急性により、一括実装が望ましい

---

## 4. Implementation Complexity & Risk

### Effort: M (3-7 days)

**内訳:**
- Day 1-2: ACT/SHIORI_ACTの早期リターン実装 + 型アノテーション更新
- Day 3-4: イベントハンドラのnil処理追加（init.lua, virtual_dispatcher.lua）
- Day 5-6: 既存テスト影響確認 + 新規テストケース追加（10件）
- Day 7: 統合テスト実行 + ドキュメント更新

**根拠:**
- 変更ファイル数: 6ファイル（実装2 + 呼び出し元2 + テスト2）
- 既存テスト影響: 40件以上の確認が必要だが、パターンは共通
- 新規テスト: 10件程度（トークン0件時のnil検証）
- 統合テスト: OnTalk/OnHour実行時のnil処理確認

### Risk: Medium

**根拠:**
- **Breaking Change**: 呼び出し元修正が必須だが、影響範囲は明確（イベントハンドラ）
- **テストカバレッジ**: 既存テストが充実しており、リグレッション検出可能
- **パフォーマンスリスク**: 早期リターンによりパフォーマンス向上が期待される（リスクなし）
- **未知の呼び出し元**: 公開APIではないため、pastaプロジェクト内のみ影響（外部依存なし）

**リスク軽減策:**
1. 型アノテーション更新により、LuaLSで呼び出し元の型エラーを検出
2. cargo test --workspace実行で既存テストの全件パス確認
3. nil処理追加後のイベントハンドラで統合テスト実行
4. コミット前に全テストスイート実行（リグレッション防止）

---

## 5. Recommendations for Design Phase

### Preferred Approach: Option A（既存関数拡張）

**理由:**
1. 最小変更で要件達成可能（2ファイル、各3行追加）
2. Breaking Change影響範囲が明確（イベントハンドラのみ）
3. 既存パターンを踏襲し、学習コスト低
4. 段階的移行が不要（一括変更が望ましい）

### Key Design Decisions

**Decision 1: nil vs 空文字列**
- **採用**: nilリターン
- **理由**: 
  - 要件で明示的に「nil応答で検出可能にする」と指定
  - Lua慣例（値がない状態 = nil）に準拠
  - 空文字列は有効なさくらスクリプトとして扱われる可能性

**Decision 2: RES.ok(nil) vs RES.no_content()**
- **Research Needed**: RES.ok(nil)の挙動確認
- **推奨**: トークン0件時は`RES.no_content()`を返す
  - SHIORI/3.0仕様で204 No Contentは正常系
  - 会話未作成を明示的に表現

**Decision 3: 既存テストの対応**
- **採用**: 既存テストはそのまま（トークンありケースのみ）
- **追加**: 新規テストケース（トークン0件時のnil検証）
  - act_test.lua: `test("build() returns nil when tokens are empty")`
  - shiori_act_test.lua: `test("build() returns nil when ACT.IMPL.build() returns nil")`

### Research Items to Carry Forward

1. **RES.ok(nil)の挙動**:
   - SHIORI/3.0応答形式での扱い
   - nilが渡された場合のエラー発生有無

2. **OnTalk/OnHourシーン実行時のトークン0件頻度**:
   - 実際のゴースト運用でのケース分析
   - パフォーマンス最適化効果の見積もり

3. **BUILDER.build()の空配列処理コスト**:
   - 空配列渡し時のCPU/メモリコスト測定
   - 早期リターンによる削減効果の定量化

4. **呼び出し元の網羅的調査**:
   - act:build()を直接呼び出している箇所の全件洗い出し
   - 未発見の呼び出し元（トランスパイル出力等）の確認

---

## 6. Gap Analysis Process Compliance

### Checklist Status

- ✅ **Requirement-to-Asset Map**: 
  - R1: ACT_IMPL.build() (act.lua:282) - トークン数判定追加必要
  - R2: SHIORI_ACT_IMPL.build() (shiori/act.lua:56) - nil検証追加必要
  - R3: 呼び出し元（init.lua, virtual_dispatcher.lua） - nil処理追加必要
  - NFR2: 既存テスト（act_test.lua, shiori_act_test.lua） - 新規テスト追加必要

- ✅ **Options A/B/C with rationale**: 
  - Option A（推奨）: 最小変更、Breaking Change影響明確
  - Option B（非推奨）: API 2系統共存による保守性低下
  - Option C（非推奨）: Option Aと実質同じ、不要なフェーズ管理

- ✅ **Effort & Risk labels**: 
  - Effort: M (3-7 days) - 実装2日 + nil処理2日 + テスト3日
  - Risk: Medium - Breaking Changeだが影響範囲明確、テストカバレッジ充実

- ✅ **Recommendations**: 
  - Preferred: Option A（既存関数拡張）
  - Key Decisions: nilリターン採用、RES.no_content()使用、新規テスト追加
  - Research Items: RES.ok(nil)挙動、トークン0件頻度、BUILDER.build()コスト、呼び出し元網羅調査

---

## 7. Conclusion

**実装戦略:** Option A（既存関数拡張）により、ACT:build()とSHIORI_ACT:build()にnil早期リターンを導入する。Breaking Changeにより呼び出し元のnil処理追加が必須だが、影響範囲はイベントハンドラに限定され、既存テストカバレッジにより安全に実装可能。

**次のステップ:** 設計フェーズでRES.ok(nil)の挙動確認、呼び出し元の網羅的調査を実施し、詳細実装計画を策定する。

**推奨コマンド:** `/kiro-spec-design act-build-early-return` で設計フェーズに進む。
