# Gap Analysis: pasta-word-definition-dsl

**Feature**: 単語定義DSLとランダム展開機能  
**Analysis Date**: 2025-12-20  
**Status**: Requirements Regenerated (awaiting approval)

---

## 分析サマリー

### スコープと目的
Pasta DSLに単語定義構文（`＠単語名：単語1　単語2`）を追加し、会話内から`＠単語名`でランダムに選択された単語を展開する機能を実装する。グローバル/ローカルスコープ、前方一致検索、シャッフルキャッシングを含む。

### 主要な発見
1. **パーサー層は完全実装済み**: `WordDef`パース、`PastaFile::global_words`/`LabelDef::local_words`への格納、エラーハンドリング、引用符エスケープまで実装・テスト済み
2. **トランスパイラ層は未実装（スタブのみ）**: 単語辞書の統合マージ、Rune静的変数への変換、Pass2での辞書生成が未実装
3. **ランタイム層は未実装（スタブのみ）**: `pasta_stdlib::word()`がスタブ実装、前方一致検索、シャッフルキャッシュ、フォールバック検索の実装なし
4. **既存技術が活用可能**: ラベル前方一致で使用中の`fast_radix_trie::RadixMap`と`RandomSelector`パターンを単語辞書にも流用可能

### 推奨アプローチ
**Option B (Create New Components)** - トランスパイラとランタイムに専用の単語辞書モジュールを追加し、ラベル機構のパターンを踏襲する。

### 主要リスク
- **中リスク**: 前方一致検索のパフォーマンス最適化（Trie構造の正しい使用、メモリ効率）
- **中リスク**: シャッフルキャッシュのスレッドセーフ性（将来の並行実行対応）
- **低リスク**: ラベル機構との一貫性維持（既存パターンの模倣で解決可能）

---

## 1. Current State Investigation

### 既存資産の調査

#### Parser Layer (`src/parser/`)
**実装済みコンポーネント**:
- ✅ `src/parser/ast.rs`:
  - `WordDef { name: String, values: Vec<String>, span: Span }`構造体定義
  - `PastaFile::global_words: Vec<WordDef>`フィールド
  - `LabelDef::local_words: Vec<WordDef>`フィールド
- ✅ `src/parser/mod.rs`:
  - `parse_word_def(pair: Pair<Rule>) -> Result<WordDef, PastaError>` (lines 72-100)
  - `parse_word_def_from_parts(pair: Pair<Rule>) -> Result<WordDef, PastaError>` (lines 103+)
  - グローバル定義のパース: `Rule::global_word_def` (lines 41-42, 51-52)
  - ローカル定義のパース: `Rule::word_def_content` (lines 152-153, 257-258)
  - 引用符エスケープ処理（`「「` → `「`）
  - 全角スペース/タブ区切り処理
  - 識別子検証（Rust識別子規則）
- ✅ `src/parser/pasta.pest`:
  - `global_word_def` rule (line 26)
  - `word_def_content` rule (line 48)
  - `at_marker`, `word_name`, `word_value_list`, `word_separator` patterns

**テストカバレッジ**:
- ✅ `tests/pasta_parser_line_types_test.rs::test_global_word_def` (lines 18-27)
- ✅ `tests/pasta_parser_line_types_test.rs::test_local_word_def` (lines 46-54)
- ✅ `tests/pasta_parser_spec_validation_test.rs::spec_ch10_1_global_word_definition` (lines 1281-1293)
- ✅ `tests/pasta_parser_spec_validation_test.rs::spec_ch10_2_local_word_definition` (lines 1297-1308)
- ✅ `tests/pasta_parser_golden_test.rs::golden_test_has_two_word_definitions` (line 220)

**アーキテクチャパターン**:
- メタデータパターン: `WordDef`は`Attribute`や`params`と同じパターンで、`Statement`ではなく宣言的データとして扱う
- エラーハンドリング: `Result<T, PastaError>`で統一、panic禁止
- Spanトラッキング: すべてのASTノードにソース位置情報を保持

#### Transpiler Layer (`src/transpiler/`)
**実装状況**:
- ❌ **未実装（スタブのみ）**: `src/transpiler/mod.rs` lines 828, 834で空`Vec`を初期化するのみ
  ```rust
  global_words: vec![],  // line 828
  local_words: vec![],   // line 834
  ```
- ❌ 単語辞書の統合マージ機能なし
- ❌ `HashMap<String, Vec<String>>` (グローバル)、`HashMap<(String, String), Vec<String>>` (ローカル)への変換なし
- ❌ Rune静的変数への変換なし（`GLOBAL_WORD_DICT`, `LOCAL_WORD_DICT`相当）
- ❌ Pass2での辞書生成なし（`transpile_pass2`に単語辞書生成ロジックなし）

**既存パターン（流用可能）**:
- ✅ `LabelRegistry` (lines 1-150): ラベル登録・ID管理・重複処理のパターン
- ✅ Pass1/Pass2分離アーキテクチャ（Pass1でAST収集、Pass2で統合コード生成）
- ✅ モジュール生成パターン（`mod __pasta_trans2__`）

#### Runtime Layer (`src/runtime/`, `src/stdlib/`)
**実装状況**:
- ❌ **スタブ実装のみ**: `src/stdlib/mod.rs::word_expansion()` (lines 139-153)
  ```rust
  fn word_expansion(_ctx: Value, word: String, _args: Vec<Value>) -> VmResult<Value> {
      // P0: Just return the word name as text
      // P1 will implement:
      // - Word dictionary lookup
      // - Prefix matching
      // - Random selection with cache
      Ok(ScriptEvent::Talk { content: vec![ContentPart::Text(word)], ... }.into())
  }
  ```
- ❌ 単語辞書検索機能なし
- ❌ 前方一致検索なし
- ❌ シャッフルキャッシュなし
- ❌ フォールバック検索（Rune変数 → 単語辞書 → 前方一致ラベル）の実装なし

**既存パターン（流用可能）**:
- ✅ `src/runtime/labels.rs::LabelTable` (lines 1-284): 前方一致検索の実装
  - `RadixMap<Vec<LabelId>>`による前方一致インデックス (line 65)
  - `select_label()` (lines 142-210): フィルタリング＋ランダム選択＋キャッシュ
  - `CachedSelection`構造体 (lines 50-54): シャッフル済み候補のシーケンシャル消費
- ✅ `src/runtime/random.rs::RandomSelector` (trait): ランダム選択の抽象化
- ✅ `fast_radix_trie::RadixMap` (Cargo.toml line 22): 既存依存関係

#### Dependencies
**既存依存関係**:
- ✅ `fast_radix_trie = "1.1.0"` (Cargo.toml line 22): 前方一致検索に使用中
- ✅ `rand = "0.9"` (Cargo.toml line 19): ランダム選択に使用中
- ✅ `thiserror = "2"` (Cargo.toml line 14): エラー型定義に使用中

**新規依存関係**: なし（既存依存で要件を満たせる）

### 規約とパターン

#### ファイル命名規則 (`.kiro/steering/structure.md`)
- モジュール: `mod.rs`または`<feature>.rs`
- テスト: `tests/<feature>_test.rs` (アンダースコア区切り、単数形)
- Fixture: `tests/fixtures/<scenario>.pasta`

#### アーキテクチャ原則 (`.kiro/steering/tech.md`)
- **レイヤー分離**: Parser → Transpiler → Runtime（上位レイヤーのみに依存）
- **2パストランスパイル**: Pass1でAST収集・ラベル登録、Pass2で統合コード生成
- **Yield型実行モデル**: すべての出力は`yield ScriptEvent`
- **エラーハンドリング**: `Result<T, PastaError>`、panic禁止

#### コーディング規約
- **識別子**: Rust側スネークケース（`variable_name`）、DSL側Unicode識別子
- **エラーコンテキスト**: ファイル名、行番号、カラム位置を含める
- **テスト戦略**: ユニットテスト（レイヤーごと）、統合テスト（`tests/`）、Fixture駆動

### 統合ポイント

#### データモデル
- **AST構造**: `PastaFile::global_words`、`LabelDef::local_words`（実装済み）
- **Runtime表現**: `HashMap<String, Vec<String>>`（グローバル）、`HashMap<(String, String), Vec<String>>`（ローカル）
- **キャッシュキー**: `(search_key, sorted_attributes)` (ラベルと同様)

#### API統合点
- **Parser出力**: `WordDef`がAST内に格納済み → Transpilerが消費
- **Transpiler出力**: Rune静的変数として辞書を生成 → Runtimeが参照
- **Runtime呼び出し**: `pasta_stdlib::word(ctx, name, args)` → フォールバック検索実行

#### 制約
- **call/jump文からの単語辞書非アクセス**: `LabelTable`検索時に単語辞書を除外（設計で明確化）
- **宣言部配置制約**: ローカル単語定義は属性・変数定義と同じブロック（パーサーで検証済み）

---

## 2. Requirements Feasibility Analysis

### 技術的要件の分類

#### Requirement 1-3: Parser Requirements (✅ 実装済み)
- **データモデル**: `WordDef`構造体、`global_words`/`local_words`フィールド
- **パース機能**: Pest文法、引用符エスケープ、識別子検証
- **エラーハンドリング**: 構文エラー収集、`Result::Err`返却
- **Status**: **完全実装済み**、既存テストでカバー済み

#### Requirement 4-5: Runtime Word Expansion (❌ 未実装)
**必要な機能**:
- **単語辞書検索**: グローバル/ローカルスコープでの名前解決
- **前方一致検索**: Trieによる効率的な前方一致（`＠場所` → `場所`, `場所＿日本`, `場所＿外国`）
- **シャッフルキャッシュ**: 同じ検索キーで呼び出されたときに順次異なる単語を返す
- **フォールバック検索**: Rune変数/関数 → 単語辞書 → 前方一致ラベル

**Gap**:
- ❌ `pasta_stdlib::word()`がスタブ実装（単語名をそのまま返すのみ）
- ❌ 単語辞書データ構造なし（グローバル/ローカル）
- ❌ 前方一致インデックスなし
- ❌ シャッフルキャッシュなし
- ❌ フォールバック検索ロジックなし

**既存パターンからの流用可能性**:
- ✅ `LabelTable::select_label()`: 前方一致検索＋フィルタリング＋キャッシュのパターン
- ✅ `RadixMap<Vec<LabelId>>`: 前方一致インデックス構造
- ✅ `CachedSelection`: シャッフル済み候補のシーケンシャル消費

#### Requirement 6: AST & Transpiler (🔶 部分実装)
**必要な機能**:
- **AST構造**: `PastaFile::global_words`、`LabelDef::local_words` (✅ 実装済み)
- **トランスパイラ統合マージ**: 同名単語定義を`Vec::extend`でマージ
- **Rune静的変数生成**: `GLOBAL_WORD_DICT`, `LOCAL_WORD_DICT`相当
- **Pass2統合**: 辞書データをRune hashmap literalとして生成

**Gap**:
- ❌ トランスパイラに単語辞書処理なし（空vecスタブのみ）
- ❌ Rune静的変数生成なし
- ❌ Pass2に辞書生成ロジックなし

**実装戦略**:
- `LabelRegistry`パターンを踏襲し、`WordDefRegistry`を作成
- Pass1で`WordDef`を収集・マージ
- Pass2で`pub static GLOBAL_WORD_DICT: HashMap<String, Vec<String>>`等を生成

#### Requirement 7: Call/Jump Separation (✅ 設計済み、実装はTrivial)
- **設計**: call/jump処理は`LabelTable`のみ使用、単語辞書は参照しない
- **実装**: 既存の`pasta::call()`がラベル検索のみ実行（単語辞書アクセスなし）
- **Status**: **設計OK**、実装不要（既存動作維持）

#### Requirement 8: Error Handling (🔶 部分実装)
- **パーサーエラー**: ✅ 実装済み（構文エラー、空定義、識別子検証）
- **ランタイムエラー**: ❌ 未実装（単語未発見時のログ、空文字列返却）

**Gap**:
- ❌ `pasta_stdlib::word()`にエラーハンドリングなし（スタブなので未実装）
- ❌ 単語未発見時のログ出力なし

#### Requirement 9: Documentation (❌ 未実装)
**必要なドキュメント**:
- `GRAMMAR.md`に「単語定義」セクション追加
- グローバル/ローカルスコープ説明
- フォールバック検索順序の明記
- 前方一致検索の例示
- サンプルスクリプト3件以上

**Gap**: すべて未実装（機能実装後に作成）

#### Requirement 10: Test Coverage (🔶 部分実装)
- **パーサーテスト**: ✅ 実装済み（`pasta_parser_line_types_test.rs`等）
- **トランスパイラテスト**: ❌ 未実装（単語辞書マージテストなし）
- **ランタイムテスト**: ❌ 未実装（単語展開、前方一致、キャッシュのテストなし）

### ギャップと制約の特定

#### Missing Capabilities
1. **トランスパイラ層**: 単語辞書統合マージ、Rune静的変数生成
2. **ランタイム層**: 単語辞書検索、前方一致インデックス、シャッフルキャッシュ、フォールバック検索
3. **テスト**: トランスパイラとランタイムの統合テスト
4. **ドキュメント**: GRAMMAR.md更新、サンプルスクリプト

#### Unknowns (Research Needed)
- **なし**: 既存のラベル前方一致実装がパターンとして存在し、技術的不確実性は低い

#### Constraints
- **既存アーキテクチャ**: 2パストランスパイルアーキテクチャを維持（Pass1で収集、Pass2で生成）
- **依存関係**: 既存の`fast_radix_trie`を使用（新規依存追加なし）
- **エラーハンドリング**: panic禁止、`Result<T, PastaError>`で統一
- **パフォーマンス**: 前方一致検索はO(log N)、キャッシュでシーケンシャルアクセスはO(1)

### 複雑性シグナル

#### Simple CRUD (該当なし)

#### Algorithmic Logic (🔶 中程度)
- **前方一致検索**: `RadixMap`による効率的な実装（既存パターンあり）
- **シャッフルキャッシュ**: `CachedSelection`パターンの流用（既存実装あり）
- **辞書マージ**: 同名単語定義の`Vec::extend`（単純なアルゴリズム）

#### Workflows (該当なし)

#### External Integrations (該当なし)

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components ❌ 非推奨

**対象ファイル**:
- `src/transpiler/mod.rs`: Pass1/Pass2に単語辞書処理を追加
- `src/stdlib/mod.rs`: `word_expansion()`にロジックを実装
- `src/runtime/labels.rs`: `LabelTable`に単語辞書を統合

**互換性評価**:
- ❌ `LabelTable`に単語辞書を統合すると責務が不明確（ラベルと単語の混在）
- ❌ `transpile_pass2()`が肥大化（ラベルセレクター＋単語辞書生成）
- ❌ 単一責任原則違反: `LabelTable`は「ラベル管理」、単語辞書は別責務

**複雑性と保守性**:
- ❌ `LabelTable`のコード行数が増加（現在284行、単語辞書追加で400行超）
- ❌ 将来の単語辞書拡張（例: タグ付き、重み付き選択）でラベル機構に影響

**Trade-offs**:
- ✅ ファイル数が増えない
- ❌ 既存コンポーネントが肥大化
- ❌ ラベルと単語の責務が混在
- ❌ テストが複雑化（ラベル機構と単語機構の分離が困難）

**結論**: **非推奨** - 単一責任原則違反、保守性低下

---

### Option B: Create New Components ✅ 推奨

**新規作成ファイル**:
1. `src/transpiler/word_registry.rs`: 単語辞書の収集・マージ・ID管理
   - `WordDefRegistry` struct
   - `register_global()`, `register_local()`: 単語定義の登録とマージ
   - `iter()`: Pass2でのRune変数生成用イテレーター
2. `src/runtime/words.rs`: 単語辞書の検索・キャッシング
   - `WordTable` struct: `HashMap<String, Vec<String>>`（グローバル）、`HashMap<(String, String), Vec<String>>`（ローカル）
   - `RadixMap<Vec<WordId>>`による前方一致インデックス
   - `CachedWordSelection`: シャッフル済み単語のシーケンシャル消費
   - `search_word(name, local_scope)`: 前方一致検索＋キャッシュ
3. `src/runtime/resolver.rs` (Optional): フォールバック検索の統合
   - `resolve_reference(ctx, name)`: Rune変数 → `WordTable` → `LabelTable`の順序で検索
   - または`pasta_stdlib::word()`内でフォールバック実装（軽量な場合）

**統合ポイント**:
- **Parser → Transpiler**: `PastaFile::global_words`、`LabelDef::local_words`を`WordDefRegistry`が消費
- **Transpiler → Runtime**: Pass2で生成されたRune静的変数を`WordTable`が読み込み（Rune VMから取得）
- **Runtime呼び出し**: `pasta_stdlib::word()` → `WordTable::search_word()` → 前方一致＋キャッシュ

**責務境界**:
- **WordDefRegistry**: 単語定義の収集・マージ・重複検出
- **WordTable**: 単語検索・前方一致・キャッシング
- **LabelTable**: ラベル検索・前方一致・キャッシング（既存責務維持）
- **pasta_stdlib::word()**: フォールバック検索の調整、ScriptEvent生成

**Trade-offs**:
- ✅ 単一責任原則を維持
- ✅ ラベル機構との明確な分離
- ✅ テストの独立性（単語辞書テストとラベルテストが分離）
- ✅ 将来の拡張性（タグ付き、重み付き選択等）
- ❌ ファイル数が増加（2-3ファイル追加）
- ❌ モジュール間の依存関係管理が必要

**結論**: **推奨** - クリーンな責務分離、既存パターンとの一貫性、テスト容易性

---

### Option C: Hybrid Approach 🔶 検討可能

**戦略**:
- **Phase 1 (Minimal Viable)**: Option Bのサブセット実装
  - `WordDefRegistry`のみ作成（トランスパイラ層）
  - `pasta_stdlib::word()`内にインラインで単語検索実装（Trie使わず線形検索）
  - シャッフルキャッシュなし（毎回ランダム選択）
- **Phase 2 (Full Implementation)**: Option Bの完全実装
  - `WordTable` + `RadixMap`による前方一致検索
  - `CachedWordSelection`によるシャッフルキャッシング
  - フォールバック検索の統合

**組み合わせ戦略**:
- **Phase 1**: 単語辞書の基本動作を確認（トランスパイル → 単純な検索 → 展開）
- **Phase 2**: パフォーマンス最適化とキャッシング追加

**段階的実装**:
1. **Pass1実装**: `WordDefRegistry`で単語定義を収集・マージ
2. **Pass2実装**: Rune静的変数として辞書を生成
3. **Runtime基本実装**: 線形検索による単語展開（キャッシュなし）
4. **Runtime最適化**: `RadixMap`＋シャッフルキャッシング

**Trade-offs**:
- ✅ 早期の動作確認が可能（Phase 1で基本機能検証）
- ✅ リスク分散（前方一致・キャッシュの複雑性を後回し）
- ❌ Phase 1のコードをPhase 2で大幅修正（リファクタリングコスト）
- ❌ Phase 1時点でパフォーマンス要件（前方一致）を満たさない

**結論**: **検討可能** - リスク低減を優先する場合は有効だが、要件は明確なのでOption Bの一括実装を推奨

---

## 4. Effort & Risk Assessment

### Effort Estimation

**Option B (Create New Components)**: **M (3-7 days)**

**内訳**:
- **Transpiler Layer** (1-2 days):
  - `WordDefRegistry`作成（`LabelRegistry`のパターン流用）
  - Pass1での単語定義収集（既存`global_words`/`local_words`を走査）
  - Pass2でのRune静的変数生成（hashmapリテラル出力）
- **Runtime Layer** (2-3 days):
  - `WordTable`作成（`LabelTable`のパターン流用）
  - `RadixMap`による前方一致インデックス構築
  - `CachedWordSelection`実装（`LabelTable::CachedSelection`と同様）
  - `search_word()`実装（前方一致＋キャッシュ）
- **stdlib Integration** (0.5-1 day):
  - `pasta_stdlib::word()`のスタブ置き換え
  - フォールバック検索の実装（Rune変数 → 単語辞書 → ラベル）
- **Testing** (1-2 days):
  - トランスパイラテスト（単語辞書マージ、Rune出力検証）
  - ランタイムテスト（前方一致、キャッシュ、フォールバック）
  - 統合テスト（E2E: パース → トランスパイル → 実行）

**根拠**:
- 既存パターン（`LabelRegistry`, `LabelTable`）の流用により実装コストを削減
- `RadixMap`は既存依存関係（ラベル前方一致で使用中）
- `RandomSelector`抽象化により単体テストが容易

### Risk Assessment: **中リスク (Medium)**

**High Risk要素**: なし

**Medium Risk要素**:
1. **前方一致検索のパフォーマンス**: 
   - **リスク**: `RadixMap`の誤用によるメモリ肥大化またはO(N)線形検索への退化
   - **軽減策**: `LabelTable`の既存実装をパターンとして流用、ベンチマークテストで検証
2. **シャッフルキャッシュのスレッドセーフ性**:
   - **リスク**: 将来の並行実行（複数会話の同時実行）でキャッシュ競合
   - **軽減策**: P0では単一スレッド前提、P1で`Mutex`または`Arc<RwLock>`によるスレッドセーフ化
3. **フォールバック検索の順序**:
   - **リスク**: Rune変数と単語名の衝突時の挙動が不明確
   - **軽減策**: 要件定義で順序を明確化（Rune変数 → 単語辞書 → ラベル）、テストで検証

**Low Risk要素**:
1. **既存パターンの流用**: `LabelRegistry`, `LabelTable`, `CachedSelection`のパターンが明確
2. **依存関係**: 新規依存なし（`fast_radix_trie`, `rand`は既存）
3. **エラーハンドリング**: `Result<T, PastaError>`パターンが確立済み

---

## 5. Recommendations for Design Phase

### 推奨アプローチ
**Option B: Create New Components** - 新規モジュール作成によるクリーンな実装

### 主要な設計決定
1. **モジュール構成**:
   - `src/transpiler/word_registry.rs`: 単語定義の収集・マージ
   - `src/runtime/words.rs`: 単語辞書の検索・キャッシング
   - `src/stdlib/mod.rs::word_expansion()`: スタブ置き換え（フォールバック検索実装）

2. **データ構造**:
   - **Transpiler**: `WordDefRegistry { global: HashMap<String, Vec<String>>, local: HashMap<(String, String), Vec<String>> }`
   - **Runtime**: `WordTable { global_dict: HashMap<..>, local_dict: HashMap<..>, prefix_index: RadixMap<Vec<WordId>>, cache: HashMap<CacheKey, CachedWordSelection> }`

3. **前方一致検索**:
   - `RadixMap<Vec<WordId>>`を使用（`LabelTable`と同様）
   - 検索キー正規化: 全角英数字 → 半角変換（optional、要設計判断）

4. **シャッフルキャッシング**:
   - `CachedWordSelection { candidates: Vec<String>, next_index: usize, history: Vec<String> }`
   - キャッシュキー: `(search_key, local_scope_option)`
   - キャッシュ枯渇時の再シャッフル

5. **フォールバック検索**:
   - Phase 1: Rune変数検索（`ctx.var.get(name)`）
   - Phase 2: 単語辞書検索（`WordTable::search_word(name, local_scope)`）
   - Phase 3: 前方一致ラベル検索（`LabelTable::search_label(name, filters)`）
   - 各フェーズで見つかった時点で処理終了

### 設計フェーズに持ち越す調査項目
1. **前方一致検索の正規化**:
   - **Question**: 全角英数字を半角に正規化するか？（例: `＠Ａｂｃ` → `＠Abc`）
   - **Research**: 既存のラベル前方一致の挙動を確認、ユーザビリティ観点で判断
   
2. **ローカルスコープの解決**:
   - **Question**: ローカル単語検索時のグローバルラベルコンテキストをどう取得するか？
   - **Options**:
     - A. `ctx.current_label: String`をRuntimeコンテキストに追加
     - B. `pasta_stdlib::word()`呼び出し時にラベル名を追加引数で渡す
   - **Research**: Runeのコンテキスト渡しのベストプラクティス調査

3. **キャッシュのライフサイクル**:
   - **Question**: シャッフルキャッシュをいつクリアするか？
   - **Options**:
     - A. スクリプト実行終了時（現在のラベルキャッシュと同様）
     - B. グローバルラベル実行終了時（ローカルスコープ終了でクリア）
     - C. 明示的なクリアAPI（`pasta_stdlib::reset_word_cache()`）
   - **Research**: ユーザーストーリー（同じ会話で繰り返しを避ける期間）を設計フェーズで詳細化

4. **エラーメッセージの日本語化**:
   - **Question**: 単語未発見エラーのメッセージフォーマット
   - **Example**: `"単語定義 @場所 が見つかりません（検索キー: 場所, スコープ: ローカル）"`
   - **Research**: 既存エラーメッセージのトーン・スタイルとの一貫性確認

---

## 6. Technical Debt & Future Considerations

### 技術的負債（既存）
- **トランスパイラのスタブ**: `global_words: vec![]`, `local_words: vec![]`が無視されている（lines 828, 834）
- **stdlibのスタブ**: `word_expansion()`が単語名をそのまま返す（P0実装として意図的）

### 将来的な拡張可能性
1. **タグ付き単語定義**: 
   - 構文例: `＠場所#日本：東京　大阪　京都`
   - 検索時にタグフィルタリング: `＠場所#日本` → `日本`タグのみ選択
2. **重み付きランダム選択**:
   - 構文例: `＠挨拶：こんにちは*3　やあ*1` (選択確率3:1)
   - `CachedWordSelection`の重複登録で実現可能
3. **単語辞書の永続化**:
   - セッション間で単語辞書を共有（JSON/TOML export/import）
   - `pasta_stdlib::save_word_dict()`, `load_word_dict()`
4. **動的単語追加**:
   - ランタイムでの単語定義追加: `pasta_stdlib::add_word("場所", ["ロンドン", "パリ"])`
   - `WordTable`をmutableに変更、キャッシュクリア

---

## 7. Conclusion

### 実装可能性
**✅ 実装可能** - パーサー層は完成済み、既存のラベル前方一致実装をパターンとして流用可能

### 推奨実装戦略
**Option B: Create New Components** - `WordDefRegistry`（トランスパイラ）、`WordTable`（ランタイム）を新規作成

### 見積もり
- **Effort**: M (3-7 days)
- **Risk**: Medium（前方一致検索のパフォーマンス、キャッシュのスレッドセーフ性）

### 次のステップ
1. **要件承認**: 再生成された`requirements.md`のレビュー・承認
2. **設計フェーズ**: 以下の調査項目を含む詳細設計
   - 前方一致検索の正規化戦略
   - ローカルスコープ解決のRune統合
   - キャッシュライフサイクルポリシー
   - エラーメッセージの日本語フォーマット
3. **タスク生成**: 設計完了後、実装タスクへ分解（Transpiler → Runtime → Testing → Documentation）

---

**Document Status**: ✅ Gap Analysis Complete  
**Next Phase**: Design Generation (`/kiro-spec-design pasta-word-definition-dsl`)
