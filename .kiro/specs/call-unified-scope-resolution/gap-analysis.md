# Implementation Gap Analysis

**Feature**: `call-unified-scope-resolution`  
**Analysis Date**: 2025-12-21  
**Language**: ja

---

## 1. Current State Investigation

### 1.1 Key Assets and Architecture

#### Existing Components

**Runtime Layer** ([src/runtime/](src/runtime)):
- **[scene.rs](src/runtime/scene.rs)**: `SceneTable` - シーン解決とランダム選択を管理
  - `resolve_scene_id()`: 前方一致検索＋属性フィルタリング＋キャッシュベース選択
  - `prefix_index`: `RadixMap<Vec<SceneId>>` でシーンIDを前方一致検索
  - **制約**: スコープ区別なし（グローバル・ローカル混在検索）、`module_name`引数なし

- **[words.rs](src/runtime/words.rs)**: `WordTable` - 単語検索と選択を管理
  - `collect_word_candidates(module_name, key)`: **2段階検索＋マージ**実装済み
    - ステップ1: ローカル検索 `:module_name:key` で前方一致
    - ステップ2: グローバル検索 `key` で前方一致（`:` で始まるキーを除外）
    - ステップ3: 両方の結果をマージして返す
  - `search_word()`: マージ候補をシャッフル＋キャッシュ

**Transpiler Layer** ([src/transpiler/](src/transpiler)):
- **[scene_registry.rs](src/transpiler/scene_registry.rs)**: シーン登録と一意ID管理
  - グローバル: `{name}_{counter}::__start__` 形式で登録
  - ローカル: `{parent}_{counter}::{name}_{counter}` 形式で登録
  - **重要**: 完全修飾名（fully qualified name）で登録、スコープ情報はキー内に埋め込み

- **[mod.rs](src/transpiler/mod.rs)**: AST→Runeコード変換
  - `Statement::Call` の処理（line 398, 701）:
    - `JumpTarget::Local` / `Global` / `LongJump` / `Dynamic` を `search_key` に変換
    - 生成コード: `crate::pasta::call(ctx, "{search_key}", #{}, [args])`
    - **制約**: 現在のグローバルシーン名（module_name）をRune引数に渡していない

**Standard Library** ([src/stdlib/mod.rs](src/stdlib/mod.rs)):
- `select_scene_to_id()`: `SceneTable::resolve_scene_id()` を呼び出し
  - 引数: `scene: String`, `filters: Value`
  - **制約**: `module_name` 引数なし、スコープコンテキスト未渡し

### 1.2 Conventions and Patterns

#### Naming Conventions
- シーン関数名: `{sanitized_name}_{counter}::__start__` (グローバル)
- ローカルシーン: `{parent}_{parent_counter}::{local_name}_{local_counter}`
- 検索キー: 完全修飾名で格納、前方一致で検索

#### Data Flow Pattern
1. **Transpiler Pass 1**: シーンを `SceneRegistry` に登録、完全修飾名を生成
2. **Transpiler Pass 2**: Call文を `crate::pasta::call(ctx, search_key, ...)` に変換
3. **Runtime**: `SceneTable::resolve_scene_id(search_key, filters)` で前方一致検索

#### Testing Approach
- 統合テスト: `tests/pasta_word_definition_e2e_test.rs` で単語のローカル＋グローバルマージ検証済み
- **欠落**: Call文のスコープ統合検索を検証するテストなし

### 1.3 Integration Points

- **AST定義**: `JumpTarget` 列挙型（`Local`, `Global`, `LongJump`, `Dynamic`）
- **SceneInfo**: `parent: Option<String>` でローカル/グローバル区別
- **現在のグローバルコンテキスト**: トランスパイル時に `current_module` を `WordDefRegistry` に渡すが、Call文には未適用

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs (EARS要件から抽出)

#### Requirement 1: スコープ統合検索
- **必要機能**: `SceneTable` に `find_scene_merged(module_name, prefix)` メソッド追加
- **データモデル**: 現状の `RadixMap` ベース前方一致検索を再利用
- **業務ロジック**: 単語検索と同じ2段階検索＋マージロジック（[words.rs](src/runtime/words.rs#L100-L150) から流用可能）

#### Requirement 2: グローバルプレフィックス廃止
- **必要機能**: `JumpTarget::Global` を非推奨化、パーサーは引き続き受理（後方互換性）
- **データモデル**: `JumpTarget` 列挙型の変更不要、トランスパイル時の処理統一のみ

#### Requirement 3: ランタイム解決の一貫性
- **API変更**: `select_scene_to_id(scene, module_name, filters)` に第2引数追加
- **Rune生成コード**: `crate::pasta::call(ctx, scene, "module_name", ...)` に現在モジュール名を渡す

#### Requirement 4: 既存テスト互換性
- **テスト更新**: `＞＊シーン` 構文を使用するテストケースの洗い出し（現在0件、fixtureにも不使用）
- **回帰防止**: 新規テストケース追加（ローカル＋グローバル候補マージ検証）

#### Requirement 5: SPECIFICATION.md更新
- **ドキュメント**: Section 4 (Call詳細仕様) の全面改定

### 2.2 Identified Gaps and Constraints

#### Missing Capabilities
1. **SceneTable に統合スコープ検索ロジックなし**: 現在は `resolve_scene_id(search_key, filters)` のみ、`module_name` 引数なし
2. **Transpiler が現在グローバルコンテキストを Call 引数に渡していない**: Pass 2 で `context.current_module` を使用していない
3. **stdlib の `select_scene_to_id` が module_name 受け取っていない**: Rune関数シグネチャ変更必要

#### Unknowns / Research Needed
- **RadixMap のキー設計**: ローカルシーンの検索キーを `:module:name` 形式にするか、別途マッピングテーブルを持つか
  - **調査済み（word実装参照）**: `:module:name` 形式で格納・検索する方式が既存実装で動作確認済み

#### Constraints from Existing Architecture
- **完全修飾名の変更不可**: `SceneRegistry` が生成する `fn_name` は他のコード（Pass 2のID→関数マッピング）に依存、変更リスク大
- **後方互換性維持**: `＞＊シーン` 構文を引き続きサポート必要、パーサー・トランスパイラーで両方を同等処理

### 2.3 Complexity Signals

- **アルゴリズム**: 2段階検索＋マージは既存 `words.rs` で実装済み、流用可能（低複雑度）
- **統合**: Transpiler→Runtime の引数追加が必要、複数箇所の変更が連鎖（中複雑度）
- **テスト**: 既存テスト群の挙動変化は最小限、新規テスト追加のみ（低複雑度）

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components ✅ **推奨**

#### Rationale
- 単語検索と同じスコープ解決パターンを再利用し、実装の一貫性を保つ
- `SceneTable` と `WordTable` の対称性が向上、保守性が高まる

#### Files to Extend

1. **[src/runtime/scene.rs](src/runtime/scene.rs)** (200-300行)
   - `find_scene_merged(module_name: &str, prefix: &str)` メソッド追加
   - 2段階検索ロジック: `collect_word_candidates` の実装パターンを流用
   - 既存の `resolve_scene_id` は内部で `find_scene_merged` を呼ぶようリファクタリング

2. **[src/transpiler/mod.rs](src/transpiler/mod.rs)** (948行、400-450行目周辺)
   - `Statement::Call` の処理部分（line 398, 701）:
     - `context.current_module()` を Call 引数リストに追加
     - 生成コード: `crate::pasta::call(ctx, "{search_key}", "{module_name}", #{}, [args])`
   - `transpile_pass2` で各シーン処理前に `context.set_current_module(module_name)` を呼び出し

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
     - パターン1（`＊シーン`）削除または非推奨化
     - パターン2を「ローカル＋グローバル統合検索」に更新
     - Section 10.3（単語参照）と同じスコープ解決ルール適用を明記

#### Compatibility Assessment
- **既存インターフェース**: `SceneTable::resolve_scene_id` は内部実装変更のみ、呼び出し側の変更不要
- **後方互換性**: `JumpTarget::Global` は引き続きパース可能、トランスパイル時に `Local` と同等処理
- **テスト影響**: 既存テストは `＞＊` 構文未使用、影響なし（fixtures調査済み）

#### Complexity and Maintainability
- **追加機能の範囲**: 2段階検索ロジックは100行程度（word実装参照）、中規模追加
- **単一責任原則**: `SceneTable` の責務は「シーン検索」で変わらず、スコープマージは自然な拡張
- **ファイルサイズ**: `scene.rs` は現在284行、+100行で384行（許容範囲）

#### Trade-offs
- ✅ 最小限のファイル変更（4ファイル）
- ✅ 既存の word 検索パターン再利用で実装工数削減
- ✅ 後方互換性維持容易
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

## 4. Implementation Complexity & Risk

### Effort Estimate
**M (3-7 days)**

- 実装: 2-3日
  - `SceneTable::find_scene_merged` 実装: 0.5日（word実装参照）
  - Transpiler の Call 文処理修正: 0.5日
  - stdlib 関数シグネチャ変更: 0.5日
  - SPECIFICATION.md 更新: 0.5-1日
- テスト: 1-2日
  - 新規テストケース作成（ローカル＋グローバルマージ検証）: 1日
  - 既存テスト回帰確認: 0.5日
  - E2Eテスト追加: 0.5日
- ドキュメント: 0.5日
  - SPECIFICATION.md Section 4 全面改定

### Risk Assessment
**Medium**

#### Risks
1. **RadixMap 検索キー設計の不整合**
   - **リスク**: ローカルシーンの検索キー形式が word と異なる場合、検索失敗
   - **軽減策**: word実装（`:module:name`）を踏襲、同じキー形式を使用
   - **確率**: 低（既存実装で動作確認済み）

2. **既存テストの挙動変化**
   - **リスク**: グローバル候補追加により、ローカル期待のテストが失敗
   - **軽減策**: fixtures調査済み（`＞＊` 未使用）、影響範囲は限定的
   - **確率**: 低

3. **Rune関数シグネチャ変更の影響**
   - **リスク**: `select_scene_to_id` の引数追加により、既存Runeコードがコンパイルエラー
   - **軽減策**: Pass 2 で生成されるコードのみが呼び出し元、外部依存なし
   - **確率**: なし（内部API）

#### Known Perf/Security Paths
- **パフォーマンス**: RadixMap の前方一致検索は O(key長)、候補数増加の影響は最小限
- **セキュリティ**: スコープ解決ロジック変更のみ、新たな脆弱性導入なし

---

## 5. Recommendations for Design Phase

### 5.1 Preferred Approach
**Option A: Extend Existing Components**

#### Key Decisions
1. **2段階検索キー形式**: `:module:name` を採用（word実装と統一）
2. **後方互換性**: `＞＊シーン` を非推奨扱いだが引き続きサポート、警告なし
3. **優先順位**: 完全ランダムマージ（ローカル優先fallbackは採用しない、単語検索と同じ挙動）

### 5.2 Research Items

#### 1. Call文のスコープコンテキスト引き回し方法の詳細設計
- **内容**: `TranspileContext` に `current_module` を保持し、Pass 2 の各シーン処理前に設定
- **調査方法**: [transpiler/mod.rs](src/transpiler/mod.rs) の `transpile_pass2` 実装を精読
- **期待結果**: `context.set_current_module(module_name)` 呼び出し位置の特定

#### 2. SceneTable の検索キー登録方法の確認
- **内容**: `SceneRegistry` から `SceneTable` に変換する際、ローカルシーンのキーを `:parent:local` 形式で登録する必要があるか確認
- **調査方法**: [scene_registry.rs](src/transpiler/scene_registry.rs) の `SceneInfo::fn_name` 生成ロジックを確認
- **期待結果**: 現状は `parent_1::local_1` 形式、`:` プレフィックス追加の必要性を判断
- **現状**: `fn_name` は変更せず、`prefix_index` への登録時にローカルシーンのみ `:` プレフィックスを付ける方式を検討

#### 3. 既存の SceneTable テストケースの拡張方針
- **内容**: [scene.rs のテスト](src/runtime/scene.rs#L240-) でローカル＋グローバルマージを検証するテストケース追加
- **調査方法**: [words.rs のテスト](src/runtime/words.rs#L200-) の `test_collect_word_candidates_merge` を参考
- **期待結果**: `test_find_scene_merged_local_and_global` テストケース設計

---

## 6. Summary

### Analysis Summary
- **スコープ**: SceneTable、Transpiler、stdlib の3レイヤーにまたがる統合検索ロジック追加
- **主要課題**: 現在のグローバルコンテキスト（module_name）をTranspiler→Runtime に引き渡す仕組みの実装
- **推奨実装**: Option A（既存コンポーネント拡張）、単語検索パターンの再利用で工数削減・一貫性向上

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
