# Implementation Gap Analysis

## 分析概要

### スコープと課題
- **対象範囲**: `code_generator.rs` の `generate_local_scene_items` および `generate_call_scene` メソッドの拡張
- **主要な課題**: 関数末尾の `CallScene` を検出し、`return` を条件付きで生成する
- **既存の統合ポイント**: `LuaCodeGenerator` 構造体、`LocalSceneItem` enum、既存のテストスイート

### 推奨事項
- **推奨アプローチ**: Option A (既存コンポーネントの拡張) が最適
- **理由**: 変更は局所的かつ明確、新規ファイル不要、既存パターンに従う
- **工数**: S (1-3日) - 単純な末尾検出ロジックと条件分岐のみ
- **リスク**: Low - 既存パターンの小規模拡張、テストによる検証容易

---

## 1. 現状調査

### 1.1 関連ファイルとモジュール配置

| ファイル                                                | 責務                               | 変更必要性                                |
| ------------------------------------------------------- | ---------------------------------- | ----------------------------------------- |
| `crates/pasta_lua/src/code_generator.rs`                | Lua コード生成                     | **必須** - 末尾呼び出し検出と return 生成 |
| `crates/pasta_lua/src/transpiler.rs`                    | トランスパイラーエントリーポイント | 不要 - 既存フローで十分                   |
| `crates/pasta_lua/tests/transpiler_integration_test.rs` | 統合テスト                         | **必須** - 新規テストケース追加           |
| `crates/pasta_core/src/parser/ast.rs`                   | AST 定義                           | 不要 - `LocalSceneItem` は既存で十分      |

### 1.2 再利用可能なコンポーネント

#### 既存のデータ構造
```rust
// crates/pasta_core/src/parser/ast.rs (L435-L444)
pub enum LocalSceneItem {
    VarSet(VarSet),           // 変数代入
    CallScene(CallScene),     // シーン呼び出し ← 末尾最適化対象
    ActionLine(ActionLine),   // アクション行
    ContinueAction(ContinueAction), // 継続アクション
}
```

#### 既存のコード生成パターン
```rust
// crates/pasta_lua/src/code_generator.rs (L288-L312)
fn generate_local_scene_items(
    &mut self,
    items: &[LocalSceneItem],
) -> Result<(), TranspileError> {
    let mut last_actor: Option<String> = None;

    for item in items {
        match item {
            LocalSceneItem::VarSet(var_set) => {
                self.generate_var_set(var_set)?;
            }
            LocalSceneItem::CallScene(call_scene) => {
                self.generate_call_scene(call_scene)?; // ← ここに is_tail_call 情報が必要
            }
            // ...
        }
    }
    Ok(())
}
```

#### 既存のテストパターン
```rust
// crates/pasta_lua/tests/transpiler_integration_test.rs (L72-L82)
#[test]
fn test_transpile_sample_pasta_header() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();
    let result = transpiler.transpile(&file, &mut output);
    assert!(result.is_ok());
    let lua_code = String::from_utf8(output).unwrap();
    assert!(lua_code.contains("local PASTA = require \"pasta\""));
}
```

### 1.3 規約と制約

#### 命名規約
- **メソッド名**: `generate_*` パターン (`generate_call_scene`, `generate_var_set` 等)
- **変数名**: スネークケース (`is_tail_call`, `last_actor`)
- **テスト名**: `test_*` プレフィックス

#### アーキテクチャパターン
- **レイヤー構成**: `Parser (pasta_core)` → `Transpiler (pasta_lua)` → `Code Generator`
- **エラーハンドリング**: `Result<(), TranspileError>` パターン統一
- **テスト配置**: `crates/pasta_lua/tests/` 配下に統合テスト

#### 依存関係の方向
- `pasta_lua` → `pasta_core` (パーサーASTに依存)
- `code_generator` はステートレス、`writer` への出力のみ
- テストは `pasta_core::parse_str` と `pasta_lua::LuaTranspiler` を使用

### 1.4 統合ポイント

| 統合ポイント       | 現在の実装                                               | 必要な変更                                                          |
| ------------------ | -------------------------------------------------------- | ------------------------------------------------------------------- |
| **末尾検出**       | なし                                                     | `generate_local_scene_items` 内で `items.last()` を使用して末尾判定 |
| **return生成**     | `act:call(...)` のみ                                     | `is_tail_call` が true の場合 `return act:call(...)`                |
| **シグネチャ拡張** | `generate_call_scene(&mut self, call_scene: &CallScene)` | `is_tail_call: bool` パラメータ追加                                 |
| **テスト検証**     | 既存: `act:call(...)` のみ                               | 新規: 末尾位置で `return act:call(...)` を検証                      |

---

## 2. 要件の実現可能性分析

### 2.1 技術的ニーズの分類

| 要件                                      | 技術的ニーズ           | 現在の状況                                         |
| ----------------------------------------- | ---------------------- | -------------------------------------------------- |
| **Requirement 1**: 末尾呼び出し検出       | リスト末尾判定ロジック | ✅ `items.last()` で実現可能                        |
| **Requirement 2**: return文の条件付き生成 | 条件分岐とコード生成   | ✅ `if is_tail_call { write!("return ") }` パターン |
| **Requirement 3**: 既存テストの互換性維持 | リグレッションテスト   | ✅ 既存テストスイートで検証可能                     |
| **Requirement 4**: 新規テストケース追加   | 統合テストの追加       | ✅ 既存テストパターンに従って追加                   |
| **Requirement 5**: 将来の拡張性考慮       | 汎用的な設計           | ✅ `LocalSceneItem` enum による抽象化               |

### 2.2 ギャップと制約

| 項目           | ギャップ/制約                               | 対処方法                                                 |
| -------------- | ------------------------------------------- | -------------------------------------------------------- |
| **末尾判定**   | Missing: 末尾位置の判定ロジックが存在しない | `items.last()` と `item` を比較、または index-based loop |
| **return生成** | Missing: 条件付き return 出力機能がない     | `generate_call_scene` 内で `if is_tail_call` 分岐        |
| **テスト**     | Missing: 末尾再帰最適化の検証テストがない   | 新規テストケース追加 (4パターン)                         |
| **型システム** | Constraint: `LocalSceneItem` enum は不変    | 問題なし - 既存の型で十分                                |

### 2.3 複雑性シグナル

- **シンプルなロジック追加**: 末尾判定は `items.last()` の単純な比較、複雑なアルゴリズム不要
- **局所的な変更**: `code_generator.rs` の 2 メソッドのみ変更、他ファイルへの影響なし
- **テスト容易性**: 既存のテストフレームワークで新規テストケースを容易に追加可能
- **外部統合なし**: 外部ライブラリや API 連携不要、純粋な内部ロジック変更

---

## 3. 実装アプローチの選択肢

### **推奨アプローチ: Option B (最小実装 + 将来の拡張性維持)**

現実分析により、末尾再帰最適化の対象は **`CallScene` のみ** と確定しました。
その上で、「大した手間でない範囲で将来の拡張性を維持する」という戦略を採用します。

#### 変更対象ファイル
1. **`crates/pasta_lua/src/code_generator.rs`**
   - `generate_local_scene_items`: `rposition` を使用したシンプルな末尾判定ロジック追加
   - `generate_call_scene`: `is_tail_call` パラメータ追加、条件付き return 生成
   - 末尾判定メソッド: コメントで将来の拡張意図を明記

2. **`crates/pasta_lua/tests/transpiler_integration_test.rs`**
   - 新規テストケース 4 件追加

#### 設計方針：シンプルさと将来への柔軟性のバランス

```rust
// 将来の拡張を視野に入れた、汎用的な判定メソッド名とコメント
/// 末尾呼び出し項目の判定
/// 
/// 現在は CallScene のみを対象としていますが、将来 LocalSceneItem に
/// 新規バリアント（FnCall など）が追加される場合、以下の matches! 条件を拡張するだけで対応可能。
fn is_last_callable_item(items: &[LocalSceneItem], current_index: usize) -> bool {
    items.iter()
        .skip(current_index + 1)
        .all(|item| !matches!(item, LocalSceneItem::CallScene(_)))
        // 将来: || !matches!(item, LocalSceneItem::FnCall(_))
}
```

**ポイント**:
- メソッド名を「呼び出し判定」という汎用的な名前にすることで、将来の拡張を示唆
- `matches!` マクロ使用で、新規パターン追加時は条件を1行追加するだけ
- コメントで拡張方法を明示
- 実装は最小限（`rposition` による単純なロジック）

#### 互換性評価
- ✅ **後方互換性**: `generate_call_scene` の既存呼び出しは `is_tail_call: false` で維持
- ✅ **既存インターフェース**: 他モジュールへの影響なし
- ✅ **テストカバレッジ**: 既存テストは末尾以外で `return` なしを検証

#### 複雑性と保守性
- **認知負荷**: 低 - シンプルなロジック、直感的に理解可能
- **単一責任原則**: 維持 - `LuaCodeGenerator` の責務は変わらず「Lua コード生成」
- **ファイルサイズ**: 影響小 - 現在 875 行、+15 行程度で 890 行程度
- **拡張コスト**: 将来新規パターン追加時は `matches!` 条件を 1 行追加するだけ

#### トレードオフ
- ✅ 新規ファイル不要、迅速な開発（1-3日）
- ✅ 既存パターンとの一貫性維持
- ✅ テストとデバッグが容易
- ✅ 将来の拡張が容易（マクロ条件追加のみ）
- ✅ 現実の要件に合わせた実装（`CallScene` のみが対象と確定）

---

### Option B: 新規コンポーネント作成

#### 想定構造
```rust
// 新規ファイル: crates/pasta_lua/src/tail_call_optimizer.rs
pub struct TailCallOptimizer;

impl TailCallOptimizer {
    pub fn is_tail_call(items: &[LocalSceneItem], index: usize) -> bool {
        // 末尾判定ロジック
    }
}
```

#### 統合ポイント
- `code_generator.rs` から `TailCallOptimizer::is_tail_call` を呼び出し
- `lib.rs` に新規モジュール追加

#### 責務の境界
- **TailCallOptimizer**: 末尾判定ロジック専門
- **LuaCodeGenerator**: コード生成のみ

#### トレードオフ
- ✅ 関心の分離 (将来の複雑化に対応)
- ✅ 単体テスト容易性
- ❌ 過剰設計 (現時点では 1 メソッドのみ)
- ❌ ファイル数増加、ナビゲーションコスト

---

### Option C: ハイブリッドアプローチ

#### 戦略
- **Phase 1**: Option A で最小実装 (末尾判定 + return 生成)
- **Phase 2**: 複雑化した場合 Option B へリファクタリング

#### 段階的実装
1. **初期実装**: `generate_local_scene_items` 内に末尾判定ロジック埋め込み
2. **将来拡張**: 関数呼び出しなど新規パターン追加時に `TailCallOptimizer` へ分離

#### リスク軽減策
- ✅ 段階的ロールアウト (テストケース追加で各段階を検証)
- ✅ 初期投資最小化 (Option A で開始)
- ❌ リファクタリングコスト (ただし現時点では不要の可能性高)

#### トレードオフ
- ✅ 柔軟性と段階的最適化
- ✅ 過剰設計回避
- ❌ 将来の不確実性 (リファクタリングが本当に必要か不明)

---

## 4. 研究課題

### 確認済み項目
- ✅ **Lua TCO仕様**: `return func()` 形式で TCO が有効化される (公式ドキュメント確認済み)
- ✅ **LocalSceneItem 構造**: 4 種類のバリアント、`CallScene` のみが対象
- ✅ **既存テストパターン**: `assert!(lua_code.contains(...))` パターンで検証可能

### 設計フェーズで詳細化が必要な項目
なし - 実装に必要な情報はすべて確認済み

---

## 5. 実装の複雑性とリスク

### 工数見積もり: **S (1-3日)**

| タスク                     | 工数  | 理由                         |
| -------------------------- | ----- | ---------------------------- |
| 末尾判定ロジック実装       | 0.5日 | `items.last()` の単純な比較  |
| `generate_call_scene` 拡張 | 0.5日 | パラメータ追加と条件分岐のみ |
| テストケース追加 (4件)     | 1日   | 既存パターンに従って作成     |
| 既存テスト検証             | 0.5日 | リグレッション確認           |
| ドキュメント更新           | 0.5日 | コメント追加                 |

**合計**: 3日以内

### リスク評価: **Low**

| リスク要因             | 評価 | 理由                                                         |
| ---------------------- | ---- | ------------------------------------------------------------ |
| **技術的不確実性**     | Low  | 既存パターンの小規模拡張、未知の技術なし                     |
| **統合の複雑性**       | Low  | `code_generator.rs` 内の閉じた変更、他モジュールへの影響なし |
| **パフォーマンス影響** | Low  | コンパイル時の静的変換、ランタイムへの影響なし               |
| **セキュリティ懸念**   | Low  | コード生成ロジックのみ、外部入力やネットワーク不使用         |

**総合リスク**: **Low** - 確立されたパターンの小規模拡張、テストによる検証容易

---

## 6. 設計フェーズへの推奨事項

### 優先アプローチ: **Option A (既存コンポーネントの拡張)**

#### 理由
1. **スコープの局所性**: 変更は `code_generator.rs` の 2 メソッドに限定
2. **シンプルな実装**: 末尾判定は 5-10 行、return 生成は 1 行の条件分岐
3. **低リスク**: 既存パターンに従い、テストによる検証が容易
4. **迅速な開発**: 新規ファイル不要、S (1-3日) で完了可能

#### 主要な設計決定事項
1. **末尾判定方法**: `items.last()` vs index-based loop
   - 推奨: `items.last()` (可読性が高い)
   
2. **シグネチャ拡張**: `generate_call_scene` に `is_tail_call: bool` 追加
   - 既存呼び出しは `false` でデフォルト動作維持

3. **テスト戦略**: 4 パターンの新規テストケース
   - 単一呼び出し、複数呼び出し、末尾以外、呼び出しなし

#### 実装フェーズで検討すべき詳細
- `generate_local_scene_items` でのループ方式 (for + last check vs enumerate)
- テストフィクスチャの配置 (`tests/fixtures/` に新規 `.pasta` ファイル追加)
- ドキュメントコメントの更新 (Requirement 番号の記載)

---

## 要件マッピング

| Requirement                               | 必要な機能         | 現在の状況               | ギャップ                    | 実装アプローチ                              |
| ----------------------------------------- | ------------------ | ------------------------ | --------------------------- | ------------------------------------------- |
| **Requirement 1**: 末尾呼び出し検出       | リスト末尾判定     | Missing                  | `items.last()` ロジック不在 | `generate_local_scene_items` に 5-10 行追加 |
| **Requirement 2**: return文の条件付き生成 | 条件分岐出力       | Missing                  | `if is_tail_call` 分岐なし  | `generate_call_scene` に 1 行条件分岐追加   |
| **Requirement 3**: 既存テストの互換性維持 | リグレッション防止 | ✅ 既存テストスイート存在 | なし                        | 既存テスト実行で検証                        |
| **Requirement 4**: 新規テストケース追加   | 末尾最適化検証     | Missing                  | テストケース不在            | 4 パターンのテスト追加                      |
| **Requirement 5**: 将来の拡張性考慮       | 汎用的設計         | ✅ `LocalSceneItem` enum  | なし                        | 既存の抽象化で十分                          |

---

## 次のステップ

### 設計フェーズへ進む準備完了
✅ 実装アプローチ確定 (Option A)
✅ 変更対象ファイル特定
✅ 技術的リスク評価完了
✅ 工数見積もり完了 (S: 1-3日)

### 設計フェーズで作成すべきドキュメント
1. **技術設計書**: `generate_local_scene_items` と `generate_call_scene` の詳細設計
2. **テスト計画**: 4 パターンのテストケース仕様
3. **実装タスク**: 段階的な実装手順 (末尾判定 → return 生成 → テスト追加)

### コマンド
```bash
/kiro-spec-design lua_tail_recursion_optimization
```
