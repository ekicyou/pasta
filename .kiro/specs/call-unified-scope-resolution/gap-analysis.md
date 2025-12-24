# Implementation Gap Analysis

**Feature**: `call-unified-scope-resolution`  
**Analysis Date**: 2025-12-24 (Updated)  
**Language**: ja

> **注意**: パーサー・トランスパイラー層を刷新したため、2025-12-21版から大幅に変更

---

## 1. Current State Investigation

### 1.1 Key Assets and Architecture (2025-12-24 Updated)

#### Existing Components

**Runtime Layer** ([src/runtime/](src/runtime)):
- **[scene.rs](src/runtime/scene.rs)**: `SceneTable` - シーン解決とランダム選択を管理
  - `resolve_scene_id(search_key, filters)`: 前方一致検索＋属性フィルタリング＋キャッシュベース選択
  - `prefix_index`: `RadixMap<Vec<SceneId>>` で `fn_name` をキーにシーンIDを前方一致検索
  - **制約**: スコープ区別なし、`module_name`引数なし

- **[words.rs](src/runtime/words.rs)**: `WordTable` - 単語検索と選択を管理
  - `collect_word_candidates(module_name, key)`: **2段階検索＋マージ**実装済み
    - ステップ1: ローカル検索 `:module_name:key` で前方一致
    - ステップ2: グローバル検索 `key` で前方一致（`:` で始まるキーを除外）
    - ステップ3: 両方の結果をマージして返す
  - `search_word()`: マージ候補をシャッフル＋キャッシュ
  - **🔑 参照実装として活用可能**

**Parser Layer** ([src/parser/](src/parser)) - **刷新済み**:
- **[grammar.pest](src/parser/grammar.pest)**: Pest PEG文法
  - `call_scene = { call_marker ~ id ~ s ~ args? }` - シンプルな構文
  - **✅ `JumpTarget` 列挙型は削除済み** - グローバルマーカー（`＊`）は非サポート
- **[ast.rs](src/parser/ast.rs)**: AST定義
  - `CallScene { target: String, args: Option<Args>, span }` - `target` は単純な文字列
- **[mod.rs](src/parser/mod.rs)**: パーサー実装
  - `parse_call_scene()`: `target` を直接パース、スコープ区別なし

**Transpiler Layer** ([src/transpiler/](src/transpiler)) - **刷新済み**:
- **[code_generator.rs](src/transpiler/code_generator.rs)**: Runeコード生成
  - `generate_call_scene()` (L186-191): 
    ```rust
    fn generate_call_scene(&mut self, call_scene: &CallScene) -> Result<(), TranspileError> {
        self.writeln(&format!(
            "for a in pasta::call(ctx, \"{}\") {{ yield a; }}",
            call_scene.target
        ))?;
        Ok(())
    }
    ```
  - **制約**: `module_name` を渡していない、引数も未使用
- **[context.rs](src/transpiler/context.rs)**: トランスパイルコンテキスト
  - `current_module()` / `set_current_module()`: **既に実装済み**
  - 単語登録（`word_registry.register_local()`）で使用中
- **[mod.rs](src/transpiler/mod.rs)**: Pass 1/2 制御
  - Pass 2: `scene_selector()` と `pasta::call()` を生成
  - `pasta::call(ctx, scene, filters, args)` 形式で生成（L145-155）

**Registry Layer** ([src/registry/](src/registry)):
- **[scene_registry.rs](src/registry/scene_registry.rs)**: シーン登録
  - グローバル: `{sanitized_name}_{counter}::__start__` (例: `会話_1::__start__`)
  - ローカル: `{parent}_{parent_counter}::{local_name}_{local_counter}` (例: `会話_1::選択肢_1`)
  - **✅ `parent: Option<String>` でスコープ区別可能**

**Standard Library** ([src/stdlib/mod.rs](src/stdlib/mod.rs)):
- `select_scene_to_id(scene, filters)`: `SceneTable::resolve_scene_id()` を呼び出し
  - **制約**: `module_name` 引数なし
- `word(module_name, key, filters)`: **✅ `module_name` 引数あり** - 参照パターン

### 1.2 Conventions and Patterns (Updated)

#### Naming Conventions
- シーン関数名: `{sanitized_name}_{counter}::__start__` (グローバル)
- ローカルシーン: `{parent}_{parent_counter}::{local_name}_{local_counter}`
- 検索キー: **現在は `fn_name` をそのまま使用**、2段階検索未実装

#### Data Flow Pattern (Updated)
1. **Parser**: `CallScene { target, args, span }` を生成
2. **Transpiler Pass 1**: シーンを `SceneRegistry` に登録
3. **Transpiler (CodeGenerator)**: `pasta::call(ctx, "{target}")` を生成
4. **Transpiler Pass 2**: `pasta::call()` → `scene_selector()` → 関数ポインタ解決
5. **Runtime**: `SceneTable::resolve_scene_id(search_key, filters)` で前方一致検索

#### Testing Approach
- 統合テスト: `tests/pasta_word_definition_e2e_test.rs` で単語の2段階検索を検証済み
- **欠落**: Call文のスコープ統合検索を検証するテストなし

### 1.3 Integration Points (Updated)

- **AST定義**: `CallScene.target: String` - 単純な文字列（`JumpTarget` 列挙型は削除済み）
- **SceneInfo**: `parent: Option<String>` でローカル/グローバル区別
- **TranspileContext2**: `current_module()` で現在のグローバルシーン名を取得可能（単語登録で使用中）
- **stdlib word関数**: `word(module_name, key, filters)` が参照パターン

---

## 2. Requirements Feasibility Analysis (Updated)

### 2.1 Technical Needs (EARS要件から抽出)

#### Requirement 1: スコープ統合検索
- **必要機能**: `SceneTable` に `find_scene_merged(module_name, prefix)` メソッド追加
- **データモデル**: 現状の `RadixMap` ベース前方一致検索を再利用
- **業務ロジック**: 単語検索と同じ2段階検索＋マージロジック（[words.rs](src/runtime/words.rs#L100-L150) から**そのままコピー可能**）

#### Requirement 2: グローバルプレフィックス廃止
- **✅ 既に達成済み**: パーサー刷新で `JumpTarget` 列挙型は削除、`＊` プレフィックスは非サポート
- **後方互換性**: 不要（新パーサーでは最初からサポートなし）

#### Requirement 3: ランタイム解決の一貫性
- **API変更**: `select_scene_to_id(scene, module_name, filters)` に第2引数追加
- **Rune生成コード**: `pasta::call(ctx, scene, "module_name")` に現在モジュール名を渡す
- **参照パターン**: `word(module_name, key, filters)` が既に同じパターンで実装済み

#### Requirement 4: 既存テスト互換性
- **テスト更新**: `＞＊シーン` 構文は新パーサーでは最初から非サポート、影響なし
- **回帰防止**: 新規テストケース追加（ローカル＋グローバル候補マージ検証）

#### Requirement 5: SPECIFICATION.md更新
- **ドキュメント**: Section 4 (Call詳細仕様) の全面改定
- **注意**: `＊` 構文の非推奨化は不要（既に削除済み）

### 2.2 Identified Gaps and Constraints (Updated)

#### Missing Capabilities
1. **SceneTable に統合スコープ検索ロジックなし**: 
   - 現在は `resolve_scene_id(search_key, filters)` のみ
   - `module_name` 引数なし
   - **必要**: `find_scene_merged(module_name, prefix)` メソッド追加

2. **Transpiler (CodeGenerator) が module_name を Call に渡していない**:
   - 現在: `pasta::call(ctx, "{target}")`
   - 必要: `pasta::call(ctx, "{target}", "{module_name}")`
   - **参照**: `word()` 関数は `module_name` を渡している

3. **stdlib の `select_scene_to_id` が module_name 受け取っていない**:
   - 現在: `select_scene_to_id(scene, filters)`
   - 必要: `select_scene_to_id(scene, module_name, filters)`

4. **SceneTable の prefix_index キー形式が WordTable と異なる**:
   - 現在: `fn_name` をそのまま使用 (例: `会話_1::選択肢_1`)
   - 必要: ローカルシーンは `:parent:local` 形式 (例: `:会話_1:選択肢_1`)
   - **注意**: `from_scene_registry()` でキー変換が必要

#### Unknowns / Research Needed
- ✅ **解決済み**: 2段階検索キー形式は `:module:name` を採用（word実装と統一）

#### Constraints from Existing Architecture
- **完全修飾名の変更不可**: `SceneInfo.fn_name` は他のコード（Pass 2のID→関数マッピング）に依存
- **対策**: `prefix_index` への登録時にのみキー変換を行う（`fn_name` 自体は変更しない）

### 2.3 Complexity Signals (Updated)

- **アルゴリズム**: 2段階検索＋マージは既存 `words.rs` で実装済み、**コピー可能**（低複雑度）
- **統合**: Transpiler→Runtime の引数追加が必要、複数箇所の変更が連鎖（中複雑度）
- **テスト**: 既存テスト群の挙動変化は最小限、新規テスト追加のみ（低複雑度）
- **✅ 簡素化**: `JumpTarget` 列挙型が削除されたため、パーサー・AST層の変更不要

---

## 3. Implementation Approach Options (Updated)

### Option A: Extend Existing Components ✅ **推奨**

#### Rationale
- 単語検索と同じスコープ解決パターンを再利用し、実装の一貫性を保つ
- `SceneTable` と `WordTable` の対称性が向上、保守性が高まる
- **✅ パーサー刷新により簡素化**: AST/Parser層の変更不要、Runtime/Stdlib層のみ修正

#### Files to Extend

1. **[src/runtime/scene.rs](src/runtime/scene.rs)** (200-300行)
   - `find_scene_merged(module_name: &str, prefix: &str)` メソッド追加
   - 2段階検索ロジック: `collect_word_candidates` の実装パターンを**そのままコピー**
   - 既存の `resolve_scene_id` は内部で `find_scene_merged` を呼ぶようリファクタリング
   - **キー形式変更**: `from_scene_registry()` でローカルシーンを `:parent:local` 形式で登録

2. **[src/transpiler/code_generator.rs](src/transpiler/code_generator.rs)** (~300行)
   - `generate_call_scene()` メソッド修正（L186-191）:
     - 現在: `pasta::call(ctx, "{target}")`
     - 変更後: `pasta::call(ctx, "{target}", "{module_name}")`
   - **参照**: `generate_word()` が `module_name` を渡す方法を踏襲

3. **[src/stdlib/mod.rs](src/stdlib/mod.rs)** (421行、80-110行目周辺)
   - `select_scene_to_id` 関数シグネチャ変更:
     ```rust
     fn select_scene_to_id(
         scene: String,
         module_name: String,  // 新規引数
         filters: rune::runtime::Value,
         scene_table: &Mutex<SceneTable>,
     ) -> Result<i64, String>
     ```
   - `SceneTable::find_scene_merged(module_name, scene)` 呼び出しに変更

4. **[SPECIFICATION.md](SPECIFICATION.md)** (1210行、591-650行目周辺)
   - Section 4 (Call詳細仕様) の全面改定:
     - `＊` プレフィックス構文の説明を削除（既にパーサーで非サポート）
     - 統合スコープ検索（ローカル＋グローバルマージ）を明記
     - Section 10.3（単語参照）と同じスコープ解決ルール適用を明記

#### Compatibility Assessment
- **既存インターフェース**: `SceneTable::resolve_scene_id` は内部実装変更のみ、呼び出し側の変更不要
- **✅ 後方互換性不要**: `JumpTarget` 列挙型は削除済み、`＊` 構文は新パーサーで非サポート
- **テスト影響**: 既存テストは `＞＊` 構文未使用、影響なし（fixtures調査済み）

#### Complexity and Maintainability
- **追加機能の範囲**: 2段階検索ロジックは100行程度（word実装参照）、中規模追加
- **単一責任原則**: `SceneTable` の責務は「シーン検索」で変わらず、スコープマージは自然な拡張
- **ファイルサイズ**: `scene.rs` は現在284行、+100行で384行（許容範囲）
- **✅ 簡素化**: パーサー/AST層の変更が不要になり、変更箇所が減少

#### Trade-offs
- ✅ 最小限のファイル変更（4ファイル）- パーサー変更なし
- ✅ 既存の word 検索パターン再利用で実装工数削減
- ✅ `JumpTarget` 削除により AST 層がシンプルに維持
- ❌ `scene.rs` の複雑度がやや増加（ただしwordと対称なので理解しやすい）

---

### Option B: Create New Components

#### Rationale（採用しない理由）
- Call解決は `SceneTable` の本質的責務、新規コンポーネント不要
- 単語検索と同じパターンなので、対称性のため同一ファイル内実装が望ましい

---

### Option C: Hybrid Approach

#### Rationale（採用しない理由）
- 本件は既存コンポーネント拡張のみで実現可能、段階的導入の必要性なし

---

## 4. Implementation Complexity & Risk (Updated)

### Effort Estimate
**S (1-3 days)** ← 旧見積もりから短縮

- 実装: 1-2日
  - `SceneTable::find_scene_merged` 実装: 0.5日（word実装からコピー）
  - `code_generator.rs` の Call 文処理修正: 0.25日
  - stdlib 関数シグネチャ変更: 0.25日
  - SPECIFICATION.md 更新: 0.5日
- テスト: 0.5-1日
  - 新規テストケース作成（ローカル＋グローバルマージ検証）: 0.5日
  - 既存テスト回帰確認: 自動（`cargo test`）
- ドキュメント: 0.5日
  - SPECIFICATION.md Section 4 改定

### ✅ 工数削減の理由
- **パーサー/AST層の変更不要**: `JumpTarget` 削除済み、`CallScene.target = String`
- **`current_module()` 既存**: Transpiler層で単語登録に使用中、Call文にも流用可能
- **参照実装あり**: `words.rs` の `collect_word_candidates()` をそのままコピー

### Risk Assessment
**Low** ← 旧評価（Medium）から軽減

#### Risks
1. **SceneTable 検索キー形式の不整合**
   - **リスク**: ローカルシーンの検索キー形式が word と異なる場合、検索失敗
   - **軽減策**: word実装（`:module:name`）を踏襲、同じキー形式を使用
   - **確率**: 低（既存実装で動作確認済み）

2. **既存テストの挙動変化**
   - **リスク**: グローバル候補追加により、ローカル期待のテストが失敗
   - **軽減策**: fixtures調査済み（`＞＊` 未使用）、影響範囲は限定的
   - **確率**: 低

3. ~~Rune関数シグネチャ変更の影響~~
   - **✅ 解消**: 内部APIのみ、外部依存なし

#### Known Perf/Security Paths
- **パフォーマンス**: RadixMap の前方一致検索は O(key長)、候補数増加の影響は最小限
- **セキュリティ**: スコープ解決ロジック変更のみ、新たな脆弱性導入なし

---

## 5. Recommendations for Design Phase (Updated)

### 5.1 Preferred Approach
**Option A: Extend Existing Components**

#### Key Decisions
1. **2段階検索キー形式**: `:module:name` を採用（word実装と統一）
2. **✅ 後方互換性不要**: `＊` 構文は新パーサーで非サポート、移行作業なし
3. **優先順位**: 完全ランダムマージ（ローカル優先fallbackは採用しない、単語検索と同じ挙動）

### 5.2 Research Items (Updated)

#### 1. ✅ Call文のスコープコンテキスト引き回し方法 **解決済み**
- **発見**: `TranspileContext2.current_module()` は既に実装済み
- **使用箇所**: 単語登録（`word_registry.register_local()`）で使用中
- **対応**: `generate_call_scene()` で同じ方法を使用

#### 2. SceneTable の検索キー登録方法の確認
- **内容**: `from_scene_registry()` でローカルシーンのキーを `:parent:local` 形式で登録する
- **現状の `fn_name`**: `parent_1::local_1` 形式（Rust関数名として有効）
- **対応**: `fn_name` は変更せず、`prefix_index` への登録時のみキー変換

#### 3. 既存の SceneTable テストケースの拡張方針
- **内容**: `scene.rs` のテストでローカル＋グローバルマージを検証
- **参照**: `words.rs` の `test_collect_word_candidates_merge` を参考
- **期待結果**: `test_find_scene_merged_local_and_global` テストケース設計

---

## 6. Summary (Updated: 2025-01-09)

### Analysis Summary
- **スコープ**: SceneTable、Transpiler (`code_generator.rs`)、stdlib の3レイヤーにまたがる統合検索ロジック追加
- **主要課題**: 現在のグローバルコンテキスト（module_name）をTranspiler→Runtime に引き渡す仕組みの実装
- **推奨実装**: Option A（既存コンポーネント拡張）、単語検索パターンの再利用で工数削減・一貫性向上
- **✅ 簡素化**: パーサー刷新により `JumpTarget` 削除済み、AST/Parser層の変更不要

### Key Changes from Previous Analysis
1. **`JumpTarget` 列挙型削除**: パーサー刷新で完全削除、後方互換性対応不要
2. **`current_module()` 既存**: Transpiler層で単語登録に使用中、追加実装不要
3. **工数見積もり短縮**: M (3-7日) → S (1-3日)
4. **リスク軽減**: Medium → Low

### Document Status
Gap分析完了。詳細設計フェーズに進む準備が整いました。

### Next Steps
```bash
/kiro-spec-design call-unified-scope-resolution
```

または自動承認で進む場合:
```bash
/kiro-spec-design call-unified-scope-resolution -y
```
