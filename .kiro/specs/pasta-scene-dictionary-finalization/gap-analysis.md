# Implementation Gap Analysis

## プロジェクト概要

本仕様は、`pasta-lua-cache-transpiler`で実装された`finalize_scene()`スタブを本実装に置き換え、Lua側で登録されたシーン・単語情報を収集してRust側の`SearchContext`を構築する機能を実装する。

**アーキテクチャ変更**：
- **Before（現状）**: トランスパイル時にRust側でSceneRegistry/WordDefRegistry構築 → SearchContextに渡す
- **After（本仕様）**: トランスパイル時はLuaコード出力のみ → Lua実行時にLua側レジストリに登録 → `finalize_scene()`でLua側から収集 → Rust側SearchContext構築

**設計決定（2026-01-22）**:
- ✅ **トランスパイラ責務削減**: SceneRegistry/WordDefRegistry構築をトランスパイラから完全削除
- ✅ **Lua側カウンタ管理**: シーン名カウンタ生成を`pasta.scene`モジュールに実装（Requirement 8）
- 🎯 **方針**: 積極的にLua実装に依存し、必要に応じてRustへリファクタリング

## 1. Current State Investigation

### 1.1 既存アーキテクチャとコンポーネント

#### 主要ファイル/モジュール

| ファイル/モジュール                                | 責務                                          | 現状                                                            |
| -------------------------------------------------- | --------------------------------------------- | --------------------------------------------------------------- |
| `crates/pasta_lua/scripts/pasta/init.lua`          | PastaモジュールエントリーポイントLua側API公開 | `finalize_scene()`スタブ実装済み（何もしない）                  |
| `crates/pasta_lua/scripts/pasta/scene.lua`         | Luaシーンレジストリ管理                       | `register()`, `get()`, `get_global_table()`実装済み             |
| `crates/pasta_lua/src/runtime/mod.rs`              | PastaLuaRuntime主要API                        | `from_loader_with_scene_dic()`で`scene_dic.lua`ロード実装済み   |
| `crates/pasta_lua/src/search/mod.rs`               | 検索モジュール登録                            | `register()`で`@pasta_search`登録実装済み                       |
| `crates/pasta_lua/src/search/context.rs`           | SearchContext実装                             | UserDataとして`search_scene()`, `search_word()`メソッド実装済み |
| `crates/pasta_lua/src/loader/cache.rs`             | キャッシュトランスパイラ                      | `generate_scene_dic()`で`finalize_scene()`呼び出し生成済み      |
| `crates/pasta_core/src/registry/scene_registry.rs` | SceneRegistry定義                             | シーン登録・管理機能実装済み                                    |
| `crates/pasta_core/src/registry/word_registry.rs`  | WordDefRegistry定義                           | 単語定義登録・管理機能実装済み                                  |

#### 既存パターン

1. **UserData パターン**: `SearchContext`がUserDataとして実装され、Luaメソッド(`search_scene`, `search_word`)を提供
2. **Module Registration パターン**: `register()`関数で`package.loaded`にモジュールを登録（`@pasta_search`, `@enc`）
3. **Function Binding パターン**: `lua.create_function()`でRust関数をLua関数にバインド（`enc.rs`の例）
4. **Runtime Factory パターン**: `from_loader_with_scene_dic()`でLoaderContextからランタイム構築

#### 統合サーフェス

- **Lua → Rust**: `require("pasta").finalize_scene()`でRust関数を呼び出す必要あり（未実装）
- **Rust → Lua**: Luaレジストリ（`pasta.scene`モジュール）からシーン情報を収集する必要あり（未実装）
- **Registry 変換**: Lua側データ → `SceneRegistry`/`WordDefRegistry` 形式への変換が必要（未実装）

### 1.2 既存規約

#### 命名規約

- Rustモジュール: スネークケース (`scene_registry`, `word_registry`)
- Luaモジュール: ドット区切り (`pasta.scene`, `pasta.ctx`)
- プリロードモジュール: `@`プレフィックス (`@pasta_search`, `@enc`)

#### 依存関係方向

- `pasta_lua` → `pasta_core` (コア型への依存)
- `runtime` → `search` (検索モジュール登録)
- `loader` → `runtime` (ランタイム構築)

#### テスト配置

- ユニットテスト: 各モジュール末尾の`#[cfg(test)] mod tests`
- 統合テスト: `crates/pasta_lua/tests/`

## 2. Requirements Feasibility Analysis

### 2.1 技術要件マッピング

#### Requirement 1: Lua側シーン情報収集

**必要な技術要素**:
- Lua関数からRust関数への呼び出しバインディング
- Luaテーブル（`pasta.scene`レジストリ）からのデータ読み取り
- Luaテーブル構造の走査（global_name → {local_name → function}）
- `SceneRegistry`への変換ロジック

**既存資産**:
- ✅ `SearchContext::new()`が`SceneRegistry`を受け取るインターフェース
- ✅ `pasta.scene`モジュールにシーン情報が格納されている
- ❌ Luaテーブル → `SceneRegistry`変換ロジック（**Missing**）

**制約**:
- Lua側シーン情報は`{global_name → {local_name → function}}`形式で格納
- `SceneRegistry`は`register_global()`/`register_local()`でビルドする設計
- トランスパイル時のカウンタ情報がLua側に存在しない（**Constraint**）

**複雑度**:
- Medium: Luaテーブル走査は標準APIで可能だが、SceneRegistry形式への変換ロジックが必要

#### Requirement 2: 単語辞書情報収集

**必要な技術要素**:
- Lua側単語レジストリ（`pasta.word`等）の新規設計・実装
- トランスパイラ出力コードがこのレジストリに登録するよう変更
- Luaテーブル（単語レジストリ）からのデータ読み取り
- `WordDefRegistry`への変換ロジック

**既存資産**:
- ✅ `WordDefRegistry`型定義済み（`pasta_core`）
- ✅ `WordTable`構築ロジック実装済み
- ❌ Lua側に単語辞書レジストリが存在しない（**Missing** - 新規実装必要）
- ❌ トランスパイラが単語定義をLua側レジストリに登録するコードを出力しない（**Missing** - トランスパイラ変更必要）

**制約**:
- 現在のトランスパイラはRust側`WordDefRegistry`に登録している（廃止予定）
- Lua側単語レジストリのデータ構造設計が必要（**Design Needed**）

**複雑度**:
- Medium: Lua側レジストリ新規実装、トランスパイラ変更、収集ロジック実装が必要

#### Requirement 3: SearchContext構築・登録

**必要な技術要素**:
- `SearchContext`の再構築メソッド
- `package.loaded["@pasta_search"]`の置換

**既存資産**:
- ✅ `SearchContext::new()`実装済み
- ✅ `register()`関数でモジュール登録パターン確立
- ❌ 既存`@pasta_search`の置換ロジック（**Missing**）

**制約**:
- `SearchContext`は既に`PastaLuaRuntime::with_config()`で初期構築されている
- `finalize_scene()`時点での再構築が必要（**Constraint**）

**複雑度**:
- Low: 既存パターンの再利用で実装可能

#### Requirement 4: Rust-Lua連携メカニズム

**必要な技術要素**:
- `lua.create_function()`によるRust関数バインディング
- Lua関数の置き換え（`PASTA.finalize_scene`スタブ → Rust関数）

**既存資産**:
- ✅ `enc.rs`で`lua.create_function()`使用例あり
- ✅ `package.loaded`へのモジュール登録パターン確立
- ❌ `pasta`モジュール（Luaスクリプト）へのRust関数注入方法（**Research Needed**）

**制約**:
- `pasta`モジュールはLuaスクリプト（`scripts/pasta/init.lua`）で定義
- Luaスクリプトで定義された関数をRust関数で置き換える必要（**Constraint**）

**複雑度**:
- Medium: Lua側で定義された関数をRust側で上書きする方法の調査が必要

#### Requirement 5: 初期化タイミング制御

**必要な技術要素**:
- `scene_dic.lua`ロード後の`finalize_scene()`呼び出し順序保証
- 複数回呼び出しへの対応

**既存資産**:
- ✅ `load_scene_dic()`で`scene_dic.lua`実行済み
- ✅ `scene_dic.lua`末尾で`finalize_scene()`呼び出し生成済み
- ✅ 自然な実行順序保証

**制約**:
- なし

**複雑度**:
- Low: 既存フローで自動的に保証される

#### Requirement 6: エラーハンドリング

**必要な技術要素**:
- Rust側エラー → Luaエラーへの伝播
- `tracing`によるログ出力

**既存資産**:
- ✅ `SearchError`型定義済み
- ✅ `SearchContext::new()`でエラーハンドリング実装
- ✅ `tracing::debug!`, `tracing::warn!`使用例多数

**制約**:
- なし

**複雑度**:
- Low: 既存パターンの踏襲

#### Requirement 7: 将来拡張への備え

**必要な技術要素**:
- 拡張ポイントの設計（関数分離、ジェネリクス等）

**既存資産**:
- ✅ `SearchContext`は既にユーザーメソッド追加可能な構造
- ✅ モジュール登録パターンは拡張可能

**制約**:
- なし

**複雑度**:
- Low: 設計原則の遵守のみ

### 2.2 ギャップ・未解決事項

| カテゴリ            | 項目                                        | ステータス                               | 影響度 |
| ------------------- | ------------------------------------------- | ---------------------------------------- | ------ |
| **Missing**         | Luaテーブル → SceneRegistry変換ロジック     | 新規実装必要                             | High   |
| **Missing**         | `pasta`モジュール関数のRust置換メカニズム   | 実装方法調査必要                         | High   |
| **Missing**         | 既存`@pasta_search`置換ロジック             | 新規実装必要                             | Medium |
| **Missing**         | `pasta.word`モジュール実装                  | 新規実装必要 (Req9)                      | High   |
| **Missing**         | トランスパイラ単語定義Lua出力               | 新規実装必要 (Req2)                      | High   |
| ~~**Constraint**~~  | ~~トランスパイル時カウンタ情報のLua側欠如~~ | ✅ **解決済み** (Req8: Lua側カウンタ管理) | N/A    |
| ~~**Research Needed**~~ | ~~WordDefRegistry再収集 vs 再利用~~   | ✅ **決定済み** (Req2/9: Lua出力＋ビルダーAPI) | N/A |
| **Constraint**      | SceneRegistry再構築 vs 初期構築済み         | 設計判断必要                             | Medium |

### 2.3 複雑度シグナル

- **アルゴリズムロジック**: Luaテーブル構造走査、SceneRegistry構築
- **外部統合**: Lua ↔ Rust関数バインディング、Luaテーブルアクセス
- **ワークフロー**: 初期化タイミング制御（既存フローで解決済み）

## 3. Implementation Approach Options

### Option A: Extend Existing Components（既存拡張アプローチ）

#### 拡張対象ファイル

1. **`crates/pasta_lua/src/runtime/mod.rs`**
   - 新規メソッド追加: `finalize_scene_impl(&self) -> LuaResult<()>`
   - Lua側`pasta.scene`レジストリにアクセスしてSceneRegistry再構築
   - SearchContext再構築・登録

2. **`crates/pasta_lua/scripts/pasta/init.lua`**
   - `finalize_scene()`スタブをRust関数呼び出しに置換
   - または、Rust側で`package.loaded["pasta"]`のテーブルを取得して関数を上書き

#### 互換性評価

- ✅ 既存`PastaLuaRuntime`のインターフェースを拡張（破壊的変更なし）
- ✅ Luaスクリプト側は変更不要（Rust側で関数置換）
- ❌ `from_loader_with_scene_dic()`の初期SearchContext構築が無駄になる可能性

#### 複雑度・保守性

- 認知負荷: Medium（`PastaLuaRuntime`に新メソッド追加）
- 単一責任原則: 維持（ランタイム初期化の一部として妥当）
- ファイルサイズ: 現在481行 → 550行程度（許容範囲）

#### トレードオフ

- ✅ 最小限の新規ファイル（既存構造を活用）
- ✅ 既存パターン踏襲（学習曲線低）
- ❌ 初期SearchContext構築の二重化（設計要検討）
- ❌ `runtime/mod.rs`の責務増加（軽微）

### Option B: Create New Components（新規コンポーネント作成）

#### 新規作成ファイル

1. **`crates/pasta_lua/src/runtime/finalize.rs`**
   - `finalize_scene_impl()`関数を独立実装
   - Luaレジストリ収集・SceneRegistry構築ロジック
   - SearchContext再構築・登録ロジック

2. **`crates/pasta_lua/src/runtime/lua_registry.rs`**
   - Luaテーブル → SceneRegistry変換ユーティリティ
   - `collect_scenes_from_lua()`, `build_scene_registry()`等

#### 統合ポイント

- `runtime/mod.rs`から`finalize::register_finalizer()`を呼び出し
- `PastaLuaRuntime::from_loader_with_scene_dic()`でファイナライザー登録

#### 責務境界

- `finalize.rs`: finalize_scene実装全般
- `lua_registry.rs`: Lua ↔ Rustデータ変換ユーティリティ
- `runtime/mod.rs`: ランタイム統合・オーケストレーション

#### トレードオフ

- ✅ 明確な責務分離（finalize専用モジュール）
- ✅ テスト容易性向上（独立モジュールとしてテスト可能）
- ✅ 将来拡張時の変更局所化
- ❌ ファイル数増加（ナビゲーション複雑化）
- ❌ 小規模機能に対して過剰な構造化の可能性

### Option C: Hybrid Approach（ハイブリッドアプローチ）

#### 戦略

**Phase 1: Minimal Viable Implementation（最小実装）**
- `runtime/mod.rs`に`finalize_scene_impl()`を直接実装
- Luaレジストリ収集・変換ロジックをメソッド内に記述
- 既存SearchContext初期化を遅延化（`finalize_scene()`まで待機）

**Phase 2: Refactoring（リファクタリング）**
- 実装が安定したら`runtime/finalize.rs`に抽出
- Lua変換ロジックを`lua_registry.rs`に分離
- テストカバレッジ拡充

#### 実装フロー

1. `PastaLuaRuntime::with_config()`で初期SearchContext構築をスキップ（Option型で管理）
2. `finalize_scene_impl()`でLuaレジストリ収集
3. SceneRegistry/WordDefRegistry構築
4. SearchContext構築・登録
5. 後続Phase 2でモジュール分離

#### リスク軽減

- ✅ 段階的ロールアウト（Phase 1で動作確認後にリファクタリング）
- ✅ 初期実装の簡潔性（オーバーエンジニアリング回避）
- ❌ Phase 2への移行コストが不明確

#### トレードオフ

- ✅ バランスの取れたアプローチ（実装速度と保守性の両立）
- ✅ 段階的改善が可能
- ❌ Phase間の計画・調整コスト
- ❌ Phase 1のコードが技術負債化するリスク

## 4. Research Items（設計フェーズでの調査事項）

| 項目                                      | 理由                                                                                 | 優先度 |
| ----------------------------------------- | ------------------------------------------------------------------------------------ | ------ |
| **Luaスクリプト関数のRust置換メカニズム** | `pasta`モジュール（Luaスクリプト）の`finalize_scene()`をRust関数で置き換える実装方法 | High   |
| **SceneRegistry再構築 vs 遅延初期化**     | 初期構築済みSearchContextを破棄して再構築するか、遅延初期化するか                    | High   |
| **トランスパイルカウンタ情報の復元**      | Lua側にカウンタ情報がない場合の`SceneRegistry`構築戦略                               | Medium |
| **WordDefRegistry再利用可否**             | トランスパイル時のWordDefRegistryをそのまま使用できるか、再収集が必要か              | Low    |

## 5. Implementation Complexity & Risk

### 工数見積もり

**Size: M (3–7 days)**

- Lua ↔ Rust連携メカニズム調査・実装: 1.5日
- Luaレジストリ収集・SceneRegistry変換: 1.5日
- SearchContext再構築・登録: 0.5日
- テスト実装（ユニット+統合）: 2日
- ドキュメント・レビュー: 1日

**理由**:
- 既存パターン踏襲部分が多いが、新規要素（Lua関数置換、テーブル走査）が含まれる
- トランスパイルカウンタ問題の代替設計が必要
- 統合テストでの動作確認が必須

### リスク評価

**Risk: Medium**

**リスク要因**:
- Luaスクリプト関数のRust置換方法が既存コードベースに前例なし
- SceneRegistryへの変換で、トランスパイル時カウンタ情報の欠如により代替ロジックが必要
- 初期SearchContext構築との二重化問題（設計判断が必要）

**リスク軽減策**:
- mlua公式ドキュメント・サンプルコード調査
- 既存`enc.rs`のバインディングパターン応用
- プロトタイプ実装で早期検証

**理由**:
- 既存の検索装置（SearchContext）は実装済みで、統合パターンも確立
- 主なリスクはLua ↔ Rust連携の詳細実装方法
- パフォーマンス・セキュリティ面でのリスクは低い（既存VM・サンドボックス利用）

## 6. 推奨アプローチ

### 推奨: **Option C (Hybrid Approach)** with Phase 1 focus

**理由**:
1. **段階的実装**: Phase 1で最小実装を動作させ、Phase 2で構造改善（リスク分散）
2. **オーバーエンジニアリング回避**: 小規模機能に対して過剰な分離を避ける
3. **既存パターン活用**: `runtime/mod.rs`拡張で既存構造を維持
4. **柔軟性**: 実装経験を踏まえてPhase 2の抽出判断が可能

### Phase 1実装ポイント

1. `PastaLuaRuntime::with_config()`でSearchContext初期構築を**条件付きスキップ**
   - `from_loader_with_scene_dic()`経由の場合のみスキップ
   - 既存テストとの互換性維持
2. `finalize_scene_impl()`を`runtime/mod.rs`に直接実装
   - Luaレジストリ（`pasta.scene`）からシーン情報収集
   - SceneRegistry構築（カウンタは連番で代替）
   - WordDefRegistry再利用
   - SearchContext構築・`@pasta_search`置換
3. Rust関数バインディング
   - `lua.create_function()`で`finalize_scene_impl`をラップ
   - `package.loaded["pasta"]`テーブルの`finalize_scene`フィールドを上書き

### 設計フェーズでの重要判断

1. **SceneRegistry構築戦略** ✅ **決定済み**
   - **方針**: Lua側でカウンタ情報を保持・管理（Requirement 8参照）
   - `pasta.scene`モジュールにカウンタロジックを実装
   - トランスパイラは番号なしの`PASTA.create_scene("メイン")`を生成
   - Lua実行時に動的に`"メイン1"`, `"メイン2"`等の番号付与
   - Rust側SceneRegistry廃止により、トランスパイル時のカウンタ管理を完全に削除

2. **初期SearchContext構築の扱い**
   - `from_loader_with_scene_dic()`フラグで初期構築をスキップ
   - または、ダミー（空）SearchContextを初期構築し、`finalize_scene()`で置換

3. **Lua関数置換の実装方法**
   - `package.loaded["pasta"]`テーブル取得 → `finalize_scene`フィールド上書き
   - `runtime/mod.rs`の初期化フローで実行

### 次フェーズ（Design Phase）での作業

- [ ] mlua APIによるLuaテーブル走査実装詳細
- [ ] SceneRegistry構築ロジック詳細設計（カウンタ代替案）
- [ ] `finalize_scene_impl()`のエラーハンドリング戦略
- [ ] ユニットテスト・統合テストシナリオ策定
- [ ] Phase 2リファクタリング判断基準明確化

## まとめ

本ギャップ分析により、以下が明確になりましたわ：

1. **主要ギャップ**: Luaテーブル→SceneRegistry変換、Lua関数Rust置換メカニズム
2. **推奨実装**: Hybrid Approach（Phase 1: 最小実装、Phase 2: リファクタリング）
3. **工数・リスク**: M規模（3-7日）、Mediumリスク（Lua連携の前例なし）
4. **設計判断事項**: SceneRegistryカウンタ代替策、初期SearchContext構築制御

次のステップとして、設計フェーズで上記Research Itemsを調査し、詳細設計ドキュメントを作成することを推奨いたしますわ。
