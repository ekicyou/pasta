# Implementation Gap Analysis

## 分析サマリー

- **スコープ**: `code_generator.rs`内の3箇所における文字列置換レベルの修正
- **既存パターン**: 明確な修正対象コードが特定済み（L533, L595, L663）
- **テスト影響**: `sample.generated.lua`と`.expected.lua`の再生成が必要
- **推奨アプローチ**: Option A（既存コンポーネント拡張）- 単純な文字列置換で完結
- **リスク**: Low - 影響範囲が明確で、既存テストによる検証が可能

---

## 1. Current State Investigation

### 1.1 Domain-Related Assets

**対象ファイル:**

| ファイル                                                                                | 役割               | 行数    | 問題箇所                    |
| --------------------------------------------------------------------------------------- | ------------------ | ------- | --------------------------- |
| [`code_generator.rs`](../../../crates/pasta_lua/src/code_generator.rs)                  | Luaコード生成      | 1,003行 | L533, L595, L663            |
| [`sample.generated.lua`](../../../crates/pasta_lua/tests/fixtures/sample.generated.lua) | テストフィクスチャ | 119行   | L87（再生成により自動修正） |
| [`sample.expected.lua`](../../../crates/pasta_lua/tests/fixtures/sample.expected.lua)   | 期待値リファレンス | 119行   | L87（手動更新が必要）       |

**問題の具体的内容:**

```rust
// ❌ 誤った実装（3箇所）
"{}{}(ctx{})"  // L533: Action::FnCall内
"{}{}(ctx{})"  // L595: Expr::FnCall (generate_expr)
"{}{}(ctx{})"  // L663: Expr::FnCall (generate_expr_to_buffer)

// ✅ 正しい実装
"{}{}(act{})"  // actオブジェクトを第1引数として渡す
```

**興味深い矛盾:**

- [`sample.generated.lua`](../../../crates/pasta_lua/tests/fixtures/sample.generated.lua) L103では`function SCENE.関数(act, value, ...)`と正しく定義されている
- しかし呼び出し側L87では`SCENE.関数(ctx, 2 + 1)`と誤って生成されている
- これは**関数定義**（手書きLua埋め込み）と**関数呼び出し**（トランスパイラ生成）の不整合を示している

### 1.2 Architecture Patterns & Conventions

**コード生成パターン:**

```rust
// LuaCodeGenerator構造
pub struct LuaCodeGenerator<'a, W: Write> {
    writer: &'a mut W,          // 出力先
    indent_level: usize,        // インデント管理
    line_ending: LineEnding,    // 改行スタイル
}

// 主要メソッド（19個）
- write_header()                  // PASTAモジュールrequire生成
- generate_actor()                // アクター定義生成
- generate_global_scene()         // グローバルシーン生成
- generate_local_scene()          // ローカルシーン生成
- generate_action()               // アクション行生成 ★問題箇所1
- generate_expr()                 // 式評価（直接出力） ★問題箇所2
- generate_expr_to_buffer()       // 式評価（バッファ出力） ★問題箇所3
- generate_args_string()          // 引数文字列生成
```

**テスト戦略:**

| テストファイル                   | テスト種別       | カバレッジ対象             |
| -------------------------------- | ---------------- | -------------------------- |
| `transpiler_integration_test.rs` | 統合テスト       | 全体的なトランスパイル動作 |
| `transpiler_snapshot_test.rs`    | スナップショット | instaによる出力比較        |
| `actor_word_dictionary_test.rs`  | ユニット         | アクター辞書機能           |
| `finalize_scene_test.rs`         | 統合             | シーン初期化               |

**関連テストケース:**

- `test_transpile_sample_pasta_to_lua_comparison()` - `sample.pasta` → `sample.expected.lua`比較
- このテストでは`sample.generated.lua`を生成し、Git追跡対象として保存
- 現在は`sample.expected.lua`との不一致により失敗している可能性が高い

### 1.3 Integration Surfaces

**上流依存:**

- `pasta_core::parser` - AST定義（`Action::FnCall`, `Expr::FnCall`）
- `pasta_core::registry` - シーンレジストリ（使用されていないが将来的に活用予定）

**下流依存:**

- Lua VM実行時: `act`オブジェクトが`PASTA.create_session()`により初期化される
- `ACT_IMPL`メソッド: `act:call()`, `act:word()`等がLuaランタイムで提供

**データフロー:**

```
Pasta DSL ─(parse)→ AST ─(transpile)→ Lua Code ─(execute)→ Lua VM
                           │
                           └─ Action::FnCall → "SCENE.関数(ctx, ...)" ❌
                           └─ Expr::FnCall   → "SCENE.関数(ctx, ...)" ❌
                                                              ↓ 修正必要
                                              "SCENE.関数(act, ...)" ✅
```

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs Mapping

| Requirement                             | 技術的実装要件               | 既存資産             | ギャップ           |
| --------------------------------------- | ---------------------------- | -------------------- | ------------------ |
| Req 1: Action::FnCall引数修正           | L533の`ctx`→`act`置換        | ✅ 明確なコード位置   | 🟢 なし             |
| Req 2: Expr::FnCall引数修正（直接）     | L595の`ctx`→`act`置換        | ✅ 明確なコード位置   | 🟢 なし             |
| Req 3: Expr::FnCall引数修正（バッファ） | L663の`ctx`→`act`置換        | ✅ 明確なコード位置   | 🟢 なし             |
| Req 4: テストフィクスチャ更新           | `sample.generated.lua`再生成 | ✅ テストハーネス存在 | 🟡 手動確認必要     |
| Req 5: ドキュメント整合性               | 影響ドキュメント確認         | ✅ ステアリング存在   | 🟢 なし（確認のみ） |

### 2.2 Gaps & Constraints

**Missing Capabilities:**
- **なし** - すべての必要な機能とテストインフラが既に存在

**Unknowns (Research Needed):**
- **なし** - 修正箇所と影響範囲が完全に明確

**Constraints:**
- ✅ **後方互換性**: 関数シグネチャは変更なし（引数名のみ変更）
- ✅ **既存テスト**: 全テストが`act`引数を期待している
- ⚠️ **手動更新**: `sample.expected.lua`はGit追跡対象のため手動更新が必要

### 2.3 Complexity Signals

| 要素             | 複雑度     | 理由                     |
| ---------------- | ---------- | ------------------------ |
| ビジネスロジック | 🟢 単純     | 文字列置換のみ           |
| データモデル     | 🟢 変更なし | AST構造に影響なし        |
| ワークフロー     | 🟢 単純     | コード生成→テスト→再生成 |
| 外部統合         | 🟢 なし     | 内部のみの変更           |

---

## 3. Implementation Approach Options

### Option A: 既存コンポーネント拡張（Extend Existing） ✅ **推奨**

#### 修正対象ファイル

**`code_generator.rs`の3箇所を修正:**

1. **L533 - Action::FnCall内** (トークアクション内の関数呼び出し)
   ```rust
   // 修正前
   "act.{}:talk(tostring({}{}(ctx{})))"
   
   // 修正後
   "act.{}:talk(tostring({}{}(act{})))"
   ```

2. **L595 - Expr::FnCall (generate_expr)** (式評価・直接出力)
   ```rust
   // 修正前
   "{}{}(ctx{})"
   
   // 修正後
   "{}{}(act{})"
   ```

3. **L663 - Expr::FnCall (generate_expr_to_buffer)** (式評価・バッファ出力)
   ```rust
   // 修正前
   "{}{}(ctx{})"
   
   // 修正後
   "{}{}(act{})"
   ```

#### 互換性評価

- ✅ **既存インターフェース尊重**: `LuaCodeGenerator`の公開APIは変更なし
- ✅ **破壊的変更なし**: 関数シグネチャは`act`を第1引数として受け取る設計
- ✅ **テストカバレッジ**: 既存の統合テストが変更を検証

#### 複雑性と保守性

- ✅ **認知負荷**: 非常に低い（3箇所の文字列置換のみ）
- ✅ **単一責任**: コード生成ロジックの修正に留まる
- ✅ **ファイルサイズ**: 1,003行→変更なし（文字列のみ変更）

#### Trade-offs

**メリット:**
- ✅ 最小限の変更（3箇所のみ）
- ✅ 既存パターンを維持
- ✅ 即座に検証可能（既存テスト活用）
- ✅ リグレッションリスク最小

**デメリット:**
- ❌ 特になし（この修正は明確で安全）

---

### Option B: 新規コンポーネント作成（Create New）

**評価**: ❌ **不適切**

**理由:**
- 単純な文字列置換で解決する問題に対して過剰
- 新規ファイル作成は複雑性を不必要に増加させる
- コード生成ロジックの責任境界は既に明確

---

### Option C: ハイブリッドアプローチ（Hybrid）

**評価**: ❌ **不要**

**理由:**
- 問題の性質（3箇所の文字列置換）に対して過剰設計
- Option Aで完全に解決可能

---

## 4. 実装複雑性とリスク評価

### 4.1 Effort Estimation

**レベル: S (1-3日)**

**内訳:**
- コード修正: 10分（3箇所の文字列置換）
- テスト実行: 5分（`cargo test --package pasta_lua`）
- フィクスチャ再生成: 5分（自動生成）
- 手動確認・Git追跡ファイル更新: 20分
- ドキュメント確認: 10分

**合計見積もり: 約1時間**

**理由:**
- 既存パターンの単純な修正
- 依存関係なし
- テストインフラ完備
- 明確な検証手順

### 4.2 Risk Assessment

**レベル: Low**

**理由:**

| リスク要因         | 評価   | 根拠                           |
| ------------------ | ------ | ------------------------------ |
| 技術的不確実性     | 🟢 なし | 修正箇所と方法が完全に明確     |
| 統合複雑性         | 🟢 なし | コード生成内部のみの変更       |
| アーキテクチャ変更 | 🟢 なし | 既存設計を維持                 |
| パフォーマンス影響 | 🟢 なし | ランタイムロジック変更なし     |
| セキュリティ懸念   | 🟢 なし | 変数スコープの適正化むしろ改善 |

**潜在的リスク:**

1. **テストフィクスチャの不整合** (確率: 低、影響: 中)
   - 軽減策: 自動生成後に手動確認、Gitで差分レビュー

2. **見落とし箇所の存在** (確率: 極低、影響: 低)
   - 軽減策: `grep -r "ctx" code_generator.rs`で全文検索、テストで検出

---

## 5. 設計フェーズへの推奨事項

### 5.1 Preferred Approach

**Option A (既存コンポーネント拡張)を推奨**

**理由:**
- 問題の性質（単純な文字列置換）に最適
- リスク最小で効果最大
- 既存テストによる即座の検証が可能

### 5.2 Key Decisions for Design Phase

1. **修正の実施順序:**
   - Step 1: 3箇所の`ctx`→`act`置換
   - Step 2: `cargo test --package pasta_lua`実行
   - Step 3: `sample.generated.lua`確認
   - Step 4: `sample.expected.lua`を手動更新（必要に応じて）
   - Step 5: 全テスト合格確認

2. **テスト戦略:**
   - 既存の`test_transpile_sample_pasta_to_lua_comparison()`を活用
   - 新規テスト追加は不要（既存テストで十分）

3. **ドキュメント更新:**
   - [`SOUL.md`](../../../SOUL.md): 確認のみ（変更不要）
   - [`SPECIFICATION.md`](../../../SPECIFICATION.md): 確認のみ（関数呼び出し仕様に影響なし）
   - [`TEST_COVERAGE.md`](../../../TEST_COVERAGE.md): 既存マッピング確認のみ

### 5.3 Research Items to Carry Forward

**なし** - すべての情報が揃っており、追加調査は不要

---

## 6. Requirement-to-Asset Map

| Requirement                                 | 既存資産                   | ギャップ     | 実装アプローチ        |
| ------------------------------------------- | -------------------------- | ------------ | --------------------- |
| **Req 1**: Action::FnCall引数修正           | ✅ `code_generator.rs` L533 | 🟢 なし       | Option A - 文字列置換 |
| **Req 2**: Expr::FnCall引数修正（直接）     | ✅ `code_generator.rs` L595 | 🟢 なし       | Option A - 文字列置換 |
| **Req 3**: Expr::FnCall引数修正（バッファ） | ✅ `code_generator.rs` L663 | 🟢 なし       | Option A - 文字列置換 |
| **Req 4**: テストフィクスチャ更新           | ✅ テストハーネス           | 🟡 再生成必要 | 自動生成 + 手動確認   |
| **Req 5**: ドキュメント整合性               | ✅ ステアリング完備         | 🟢 なし       | 確認のみ              |

**凡例:**
- 🟢 ギャップなし（既存資産で対応可能）
- 🟡 軽微なギャップ（自動化または手動作業で対応）
- 🔴 重大なギャップ（新規実装必要）

---

## 結論

本仕様は**極めて低リスク・低複雑度**であり、既存コードベースの単純な修正で完結する。すべての必要なインフラ（テスト、フィクスチャ、ドキュメント）が既に整っており、設計フェーズでは実装手順の明確化のみが必要。

**次のステップ:** `/kiro-spec-design transpiler-ctx-instead-of-act`で技術設計を生成してくださいませ。
