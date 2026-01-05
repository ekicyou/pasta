# Gap Analysis Document

## Analysis Summary

本仕様は、pasta_luaトランスパイラーにおけるシーンアクター初期化処理の出力順序とAPI呼び出し形式を変更する。既存コードベースの分析結果、要件の実装には**最小限の変更**で対応可能である。

### 主な知見
- **変更範囲**: `code_generator.rs`の`generate_local_scene()`メソッド内の限定的な変更
- **影響度**: アクター初期化処理の出力順序変更のみ。既存テストスイートは修正が必要
- **実装難度**: 低 - 既存の`set_spot()`生成ロジックを移動・変更するのみ
- **リスク**: 低 - 独立した変更で既存機能に影響なし

---

## Current State Investigation

### 1. ファイル・モジュール構成

#### コアコンポーネント
- **[`pasta_lua/src/lib.rs`](c:\home\maz\git\pasta\crates\pasta_lua\src\lib.rs)**: クレートエントリーポイント
- **[`pasta_lua/src/transpiler.rs`](c:\home\maz\git\pasta\crates\pasta_lua\src\transpiler.rs)**: メイントランスパイラー
  - `LuaTranspiler::transpile()`: PastaFile → Lua コード変換
  - FileItem(ActorScope, GlobalSceneScope)を処理
- **[`pasta_lua/src/code_generator.rs`](c:\home\maz\git\pasta\crates\pasta_lua\src\code_generator.rs)**: Lua コード生成（869行）
  - `generate_actor()`: アクター定義ブロック生成
  - `generate_global_scene()`: グローバルシーン生成
  - `generate_local_scene()`: **ローカルシーン関数生成（変更対象）**
  - `generate_action_line()`, `generate_call_scene()`など、個別アクション生成
- **[`pasta_lua/src/context.rs`](c:\home\maz\git\pasta\crates\pasta_lua\src\context.rs)**: トランスパイルコンテキスト

#### パーサー（`pasta_core`依存）
- **[`pasta_core/src/parser/ast.rs`](c:\home\maz\git\pasta\crates\pasta_core\src\parser\ast.rs)**: AST定義
  - `SceneActorItem`: アクター情報（name, number, span）
  - `LocalSceneScope`: ローカルシーン（name, items, code_blocks等）

### 2. 現在の実装パターン（`generate_local_scene()` @ line 240-270）

```rust
pub fn generate_local_scene(
    &mut self,
    scene: &LocalSceneScope,
    counter: usize,
    actors: &[SceneActorItem],  // scene-level actors from GlobalSceneScope
) -> Result<(), TranspileError> {
    // ...
    self.writeln("local args = { ... }")?;
    self.writeln("local act, save, var = PASTA.create_session(SCENE, ctx)")?;
    
    // Generate set_spot calls for __start__ only (counter == 0)
    if counter == 0 && !actors.is_empty() {
        self.write_blank_line()?;
        for actor in actors {
            self.writeln(&format!("act.{}:set_spot({})", actor.name, actor.number))?;
        }
        self.write_blank_line()?;
    } else {
        self.write_blank_line()?;
    }
    
    self.generate_local_scene_items(&scene.items)?;
    // ...
}
```

**現状の特徴:**
1. `create_session()`の**後**で`set_spot()`を出力（Requirement 1でこれを逆順に）
2. `act.アクター名:set_spot(位置)`形式（Requirement 3で`act:set_spot("名前", 位置)`に変更）
3. `__start__`のみ（counter==0）でアクター設定を出力

### 3. 設計パターン・規約

| 項目                   | パターン                                                               |
| ---------------------- | ---------------------------------------------------------------------- |
| **メソッド分離**       | 各生成機能が独立メソッド（`generate_actor`, `generate_local_scene`等） |
| **インデント管理**     | `indent()`/`dedent()`とセット の writeln/write_indent                  |
| **エラーハンドリング** | `Result<(), TranspileError>`をメソッドのシグネチャに統一               |
| **命名規則**           | `generate_*()`, `writeln()`, `write_indent()`等（スネークケース）      |
| **Rustお作法**         | 所有権は`&self`でメソッド、変更可能な参照を受け取る                    |

### 4. テスト戦略

#### 既存テストスイート（`pasta_lua/tests/`）
- **[`transpiler_integration_test.rs`](c:\home\maz\git\pasta\crates\pasta_lua\tests\transpiler_integration_test.rs)** (958行)
  - Line 849-890: `test_set_spot_multiple_actors()` - 複数アクターのset_spot出力確認
  - Line 891-910: `test_set_spot_single_actor()` - 単一アクターのset_spot出力確認
  - Line 912-930: `test_set_spot_empty_actors()` - アクター未定義時、set_spot出力なし確認
  - Line 932+: `test_set_spot_with_explicit_number()` - 明示的な位置番号処理確認

**テスト修正が必要な箇所:**
- `test_set_spot_multiple_actors()`: `act.さくら:set_spot()` → `act:set_spot("さくら", ...)` の検証に変更
- `test_set_spot_single_actor()`: 同上
- `test_set_spot_with_explicit_number()`: 同上
- `create_session()` と `set_spot()` の順序をテストで検証

#### フィクスチャー（参考出力）
- [`fixtures/sample.generated.lua`](c:\home\maz\git\pasta\crates\pasta_lua\tests\fixtures\sample.generated.lua):
  - Line 23-26: 現在の出力（`create_session()`後に`set_spot()`）
  - 新形式では、**set_spot前に**`create_session()`移動と API呼び出し形式変更が必要

### 5. 依存関係・制約

| 項目                     | 詳細                                                                                                                                         |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------- |
| **`SceneActorItem`**     | [`pasta_core/src/parser/ast.rs` line 314](c:\home\maz\git\pasta\crates\pasta_core\src\parser\ast.rs#L314) で定義。name, numberフィールド保持 |
| **`LocalSceneScope`**    | [`GlobalSceneScope`の`local_scenes: Vec<LocalSceneScope>`](c:\home\maz\git\pasta\crates\pasta_core\src\parser\ast.rs)として参照              |
| **呼び出し元**           | `generate_global_scene()` (line 184)で各local_sceneに対し`generate_local_scene()`呼び出し                                                    |
| **インターフェース変化** | なし - `generate_local_scene()`のシグネチャ変更不要（内部実装のみ）                                                                          |

---

## Requirements Feasibility Analysis

### 要件から技術ニーズへのマッピング

| 要件                            | 技術的ニーズ                                                | 現状                                       | ギャップ                    |
| ------------------------------- | ----------------------------------------------------------- | ------------------------------------------ | --------------------------- |
| **Req 1**: 出力位置変更         | `set_spot()`を`create_session()`の前に配置                  | 後に配置                                   | **Missing**                 |
| **Req 2**: `clear_spot()`追加   | 新API呼び出し`act:clear_spot()`を先頭に追加                 | 実装なし                                   | **Missing**                 |
| **Req 3**: `set_spot()`形式変更 | `act:set_spot("名前", 位置)`形式に変更                      | `act.名前:set_spot(位置)`形式              | **Missing**                 |
| **Req 4**: コード構造の整合性   | 出力順序: args → clear_spot → set_spot × N → create_session | 現在: args → create_session → set_spot × N | **Missing**                 |
| **Req 5**: 互換性               | アクター辞書、会話アクション、Call形式に影響なし            | 独立実装                                   | **OK - No Change Required** |

### 複雑度信号

| 信号                 | 評価                                                     |
| -------------------- | -------------------------------------------------------- |
| **新API導入**        | `act:clear_spot()`呼び出し追加（単純）                   |
| **既存ロジック変更** | `set_spot()`生成コードの移動・形式変更（中程度）         |
| **外部依存**         | `pasta_core::parser`の`SceneActorItem`構造に依存（安定） |
| **テスト影響**       | 既存テストの更新が必須（7-10テストケース）               |
| **リグレッション**   | 低 - アクター初期化処理のみ変更                          |

---

## Implementation Approach Options

### Option A: 既存`generate_local_scene()`メソッド内で出力順序と形式を変更

#### 変更対象ファイル
- **[`pasta_lua/src/code_generator.rs`](c:\home\maz\git\pasta\crates\pasta_lua\src\code_generator.rs)** line 240-270
  - `generate_local_scene()`メソッドの`set_spot()`生成ロジック

#### 実装概要

```rust
pub fn generate_local_scene(
    &mut self,
    scene: &LocalSceneScope,
    counter: usize,
    actors: &[SceneActorItem],
) -> Result<(), TranspileError> {
    // ... fn_name etc ...
    self.writeln(&format!("function SCENE.{}(ctx, ...)", fn_name))?;
    self.indent();

    // (新) Session初期化の前にアクター設定を出力
    self.writeln("local args = { ... }")?;
    
    // (新) __start__のみ、create_session前にアクター初期化
    if counter == 0 && !actors.is_empty() {
        self.write_blank_line()?;
        self.writeln("act:clear_spot()")?;  // (新) clear_spot()追加
        for actor in actors {
            self.writeln(&format!(
                "act:set_spot(\"{}\", {})",  // (変更) 形式変更
                actor.name,
                actor.number
            ))?;
        }
    }
    
    // (変更位置) create_session をアクター設定後に移動
    self.writeln("local act, save, var = PASTA.create_session(SCENE, ctx)")?;
    self.write_blank_line()?;
    
    // ... rest of function ...
}
```

**利点:**
- ✅ 最小限の変更（1メソッド内のみ）
- ✅ インターフェース変化なし（呼び出し元コード不変）
- ✅ 既存パターンに従う（設計規約との整合性高い）
- ✅ 実装が直線的で理解しやすい

**欠点:**
- ❌ テストスイートの更新が必須（7-10テスト）
- ❌ 固定出力ロジックが複雑化する可能性（if/else分岐が増える）

**複雑度:** 低
**リスク:** 低

---

### Option B: アクター初期化処理を独立メソッドに抽出

#### 変更対象ファイル
- **[`pasta_lua/src/code_generator.rs`](c:\home\maz\git\pasta\crates\pasta_lua\src\code_generator.rs)**
  - 新メソッド: `generate_actor_initialization()`追加
  - `generate_local_scene()`から呼び出し

#### 実装概要

```rust
/// アクター初期化処理を生成（Req 2, 3対応）
fn generate_actor_initialization(
    &mut self,
    actors: &[SceneActorItem],
) -> Result<(), TranspileError> {
    if actors.is_empty() {
        return Ok(());
    }
    
    self.write_blank_line()?;
    self.writeln("act:clear_spot()")?;
    for actor in actors {
        self.writeln(&format!(
            "act:set_spot(\"{}\", {})",
            actor.name,
            actor.number
        ))?;
    }
    Ok(())
}

pub fn generate_local_scene(
    &mut self,
    scene: &LocalSceneScope,
    counter: usize,
    actors: &[SceneActorItem],
) -> Result<(), TranspileError> {
    // ...
    self.writeln("local args = { ... }")?;
    
    if counter == 0 {
        self.generate_actor_initialization(actors)?;
    }
    
    self.writeln("local act, save, var = PASTA.create_session(SCENE, ctx)")?;
    // ...
}
```

**利点:**
- ✅ 責務分離が明確（初期化ロジック独立）
- ✅ テストがしやすい（メソッドを単体でテスト可）
- ✅ 将来の拡張に対応しやすい

**欠点:**
- ❌ メソッドが1つ増える（ファイルが複雑化）
- ❌ 追加のインディレクションで可読性低下の可能性

**複雑度:** 中程度
**リスク:** 低-中

---

### Option C: ハイブリッド（段階的実装）

#### 段階1（Phase 0）
- Option A で出力順序を変更、形式を修正
- 既存テスト修正

#### 段階2（Phase 1、オプション）
- Option B でリファクタリング
- テスト追加で初期化ロジックをカバー

**利点:**
- ✅ リスク軽減（段階的）
- ✅ フィードバック反映可能

**欠点:**
- ❌ 複数フェーズに分割→オーバーヘッド増加

**複雑度:** 中程度
**リスク:** 低

---

## Implementation Complexity & Risk Assessment

| 項目                   | 評価            | 理由                                                       |
| ---------------------- | --------------- | ---------------------------------------------------------- |
| **実装難度**           | **S (1-3日)**   | 既存コード移動・形式変更のみ。新しいパターン不要           |
| **リスク**             | **Low**         | アクター初期化処理は他機能から独立。リグレッション範囲限定 |
| **テスト修正**         | **M (3-7時間)** | 7-10テストケースの検証文字列更新 + 順序確認テスト追加      |
| **フィクスチャー更新** | **S**           | `sample.generated.lua` の参考出力更新のみ                  |

---

## Requirement-to-Asset Map

| 要件                            | 対象コンポーネント                                                    | 現状                      | 必要なアクション                   |
| ------------------------------- | --------------------------------------------------------------------- | ------------------------- | ---------------------------------- |
| **Req 1: 出力位置**             | `code_generator.rs` `generate_local_scene()` L254-261                 | `create_session()` **後** | 移動: アクター初期化を**前**に     |
| **Req 2: `clear_spot()`追加**   | `code_generator.rs` `generate_local_scene()`                          | 実装なし                  | 新規追加: `act:clear_spot()`       |
| **Req 3: `set_spot()`形式変更** | `code_generator.rs` L261 `act.{}:set_spot()`                          | プロパティアクセス形式    | 変更: `act:set_spot("名前", 位置)` |
| **Req 4: コード構造整合**       | `code_generator.rs` 全体フロー                                        | 現在の順序                | 統合テスト追加: 順序検証           |
| **Req 5: 互換性**               | `generate_actor()`, `generate_action_line()`, `generate_call_scene()` | 独立実装                  | 修正なし（確認のみ）               |

**Constraints & Unknowns:**
- ❓ `PASTA.create_session()` / `act:clear_spot()` ランタイムAPI が `pasta_lua`モジュール側で実装済みか確認
  - **Note:** Luaランタイム実装は `pasta_lua/src/stdlib/` スコープ外（本仕様範囲は Rustトランスパイラーのみ）
  - **Assumption**: Luaランタイムが既に`act:clear_spot()` / `act:set_spot(name, pos)` をサポートしていると仮定
  - **Research Needed**: ランタイム側の実装確認（設計フェーズ）

---

## Recommendations for Design Phase

### 推奨: **Option A** を採用

**理由:**
1. **実装が直線的** - `generate_local_scene()`内の出力順序・形式のみ変更
2. **リスク最小** - インターフェース変化なし、アクター初期化処理は独立
3. **テスト修正が明確** - 既存テストの検証文字列更新のみ
4. **設計規約に従う** - 既存パターン（各生成機能の分離）を維持

### 設計フェーズで実施するべき項目

1. **テスト戦略の詳細化**
   - 順序検証テスト追加（args → clear_spot → set_spot × N → create_session）
   - アクター未定義時の clear_spot 未出力確認
   - 単一/複数アクターの位置番号正確性確認

2. **Luaランタイム側の確認**
   - `act:clear_spot()` / `act:set_spot(name, pos)` API の実装確認
   - API シグネチャ（パラメータ型、戻り値）確認
   - 既存`act.name:set_spot(pos)`との互換性検討

3. **フィクスチャー・統合テスト更新**
   - `sample.generated.lua` の参考出力更新
   - 新形式での生成コード例提示

4. **ドキュメント更新**
   - `code_generator.rs` の`generate_local_scene()`ドキュメントコメント更新
   - Requirement 1-4 への対応状況をコメント記載

### Research Items（Design相談）

| 項目                                                                      | 理由                                       | 対応フェーズ         |
| ------------------------------------------------------------------------- | ------------------------------------------ | -------------------- |
| Luaランタイムの`act:clear_spot()` / `act:set_spot(name, pos)` API実装確認 | Transpiler出力が依存                       | Design               |
| テスト追加の具体的な仕様（順序検証方法）                                  | テスト戦略決定に必要                       | Design               |
| フィクスチャー生成の自動化有無                                            | `sample.generated.lua` 更新の手動/自動判定 | Design（オプション） |

---

## Output Checklist

- ✅ **Requirement-to-Asset Map**: 要件から変更対象コンポーネントへのマッピング完了
- ✅ **Options A/B/C**: 3つの実装アプローチを提示（Option A推奨）
- ✅ **Effort & Risk**: S（1-3日）/ Low リスク で評価
- ✅ **Gap Analysis**: 現状と要件の差分を明確化
- ✅ **Design Recommendations**: 次フェーズのアクション・Research項目を列挙
