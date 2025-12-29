# Requirements

## Project Description

pasta_luaトランスパイラーがPastaFileを入力として受け取り、pasta_runeと同じアーキテクチャで処理するように移行する。現在はactorsとscenesを別々の配列として受け取っているが、FileItemを出現順に処理する統一されたAPIに変更する。

## Actors

- **LuaTranspiler**: pasta_luaモジュールのメインエントリーポイント。PastaFileをLuaコードに変換する
- **LuaCodeGenerator**: Luaコードの実際の生成を担当するコンポーネント
- **TranspileContext**: シーンレジストリ・単語レジストリを保持するコンテキスト
- **PastaFile**: パーサーが生成するAST。FileItem要素のリストを含む
- **FileItem**: PastaFile内の個々の要素（FileAttr, GlobalWord, GlobalSceneScope, ActorScope）

## Functional Requirements

### REQ-1: PastaFile入力インターフェース
**ID**: REQ-1  
**Title**: PastaFile入力インターフェース  
**Statement**: When LuaTranspiler processes a Pasta source file, the LuaTranspiler shall accept a `&PastaFile` reference as the primary input parameter instead of separate `&[ActorScope]` and `&[GlobalSceneScope]` arrays.  
**Acceptance Criteria**:
- [ ] LuaTranspilerに`transpile_file(&PastaFile, ...)` メソッドが存在する
- [ ] メソッドのシグネチャがpasta_runeのTranspiler2と一致している
- [ ] 既存のtranspile()メソッドは後方互換性のため残すが、非推奨マークを付ける

### REQ-2: FileItem出現順処理
**ID**: REQ-2  
**Title**: FileItem出現順処理  
**Statement**: When LuaTranspiler receives a PastaFile, the LuaTranspiler shall iterate over `file.items` and process each FileItem element in document order (the order they appear in the source file).  
**Acceptance Criteria**:
- [ ] FileItem::FileAttrがファイル属性として累積される
- [ ] FileItem::GlobalWordがグローバル単語として登録される
- [ ] FileItem::GlobalSceneScopeがシーンとして処理される
- [ ] FileItem::ActorScopeがアクターとして処理される
- [ ] 上記すべてが出現順に処理される

### REQ-3: ファイルレベル属性処理【順序依存・シャドーイング】
**ID**: REQ-3  
**Title**: ファイルレベル属性処理  
**Priority**: 🔴 HIGH - 順序依存の処理ロジック  
**Statement**: When LuaTranspiler encounters a FileItem::FileAttr during iteration, the LuaTranspiler shall accumulate the attribute and apply it according to pasta_rune's file attribute handling rules, respecting the shadowing semantics where later attributes override earlier ones.  
**Shadowing Semantics**:
- FileAttrは**直後のグローバルシーン**が参照する
- 同じキーの属性が再出現すると**上書き**される（シャドーイング）
- グローバルシーン出現時の属性状態が**そのシーンに継承**される
- 例:
  ```pasta
  &author:A
  ＊シーン1  ← author=A を継承
  &author:B  ← A を上書き
  ＊シーン2  ← author=B を継承
  ```
- **順序が処理結果に直接影響**するため、HashMap列挙は使用不可

**Acceptance Criteria**:
- [ ] FileAttrがTranspileContext内で累積される（順序保持）
- [ ] 同じキーの属性が再出現した場合、新しい値で上書きされる
- [ ] 累積された属性が後続のシーン/アクター生成時に正しい値で利用可能
- [ ] pasta_runeのaccumulate_file_attr()と同等の動作をする
- [ ] 属性の適用順序がファイル内の出現順序と一致することをテストで検証

### REQ-4: グローバル単語登録
**ID**: REQ-4  
**Title**: グローバル単語登録  
**Statement**: When LuaTranspiler encounters a FileItem::GlobalWord during iteration, the LuaTranspiler shall register the word definition in the WordDefRegistry as a global word, following pasta_rune's global word registration rules.  
**Acceptance Criteria**:
- [ ] GlobalWordがWordDefRegistryにグローバルスコープで登録される
- [ ] 登録順序がファイル内の出現順序と一致する
- [ ] pasta_runeのword_registry.register_global()と同等の動作をする

### REQ-5: シーン処理順序
**ID**: REQ-5  
**Title**: シーン処理順序  
**Statement**: When LuaTranspiler encounters a FileItem::GlobalSceneScope during iteration, the LuaTranspiler shall process the scene with awareness of previously accumulated file attributes and registered global words.  
**Acceptance Criteria**:
- [ ] シーン処理時に累積されたファイル属性が利用可能
- [ ] シーン処理時に登録済みグローバル単語が利用可能
- [ ] 現在のシーンより前に定義されたグローバル単語のみが参照可能

### REQ-6: アクター処理順序【属性非依存】
**ID**: REQ-6  
**Title**: アクター処理順序  
**Priority**: ℹ️ INFO - アクターは属性の影響を受けない  
**Statement**: When LuaTranspiler encounters a FileItem::ActorScope during iteration, the LuaTranspiler shall process the actor definition in document order, but actors shall NOT inherit file attributes (unlike GlobalSceneScopes).  
**Design Rationale**:
- アクターは**ファイル属性の影響を受けない**（file_attrのシャドーイングはアクターに継承されない）
- ただし、**出現順に処理**される（FileItem列挙順序を保持）
- アクター内の単語定義はアクタースコープで処理される

**Acceptance Criteria**:
- [ ] アクターが出現順に処理される
- [ ] アクターはfile_attrの累積状態を**継承しない**
- [ ] アクター内の単語定義がアクタースコープで正しく処理される

## Non-Functional Requirements

### REQ-7: API一貫性【1パス処理】
**ID**: REQ-7  
**Title**: API一貫性  
**Priority**: ℹ️ INFO - Lua言語とRune言語の設計差異により2パス不要  
**Statement**: The LuaTranspiler shall provide an API that is consistent with pasta_rune's Transpiler2 in terms of input parameters and method naming conventions, but shall use a single-pass implementation due to Lua language design differences.  
**Design Rationale**:
- Rune言語: 2パス必要（pass1: 登録+生成、pass2: scene_selector）
- Lua言語: 1パスで完結（言語設計の違いにより2段階処理が不要）
- API名は `transpile_file()` を採用（pasta_runeのpass1相当だが、実装は1パス完結）

**Acceptance Criteria**:
- [ ] メソッド名は `transpile_file()` （pasta_runeと一致）
- [ ] パラメータは `&PastaFile` を第一引数として受け取る
- [ ] 1パス処理で完結する（pass2は実装しない）
- [ ] 戻り値の型がpasta_runeのパターンに準拠

### REQ-8: 後方互換性
**ID**: REQ-8  
**Title**: 後方互換性  
**Statement**: The LuaTranspiler shall maintain backward compatibility by keeping the existing `transpile()` and `transpile_with_globals()` methods as deprecated wrappers.  
**Acceptance Criteria**:
- [ ] 既存のtranspile()メソッドが引き続き動作する
- [ ] 既存のtranspile_with_globals()メソッドが引き続き動作する
- [ ] 非推奨メソッドに#[deprecated]属性が付与されている
- [ ] 既存のテストが変更なしでパスする

### REQ-9: テストカバレッジ
**ID**: REQ-9  
**Title**: テストカバレッジ  
**Statement**: The new transpile_file() method shall have comprehensive test coverage including unit tests for each FileItem type and integration tests for order-sensitive processing.  
**Acceptance Criteria**:
- [ ] 各FileItem種別に対するユニットテストが存在する
- [ ] FileItem出現順序を検証する統合テストが存在する
- [ ] pasta_runeの対応テストと同等のカバレッジがある

### REQ-10: PastaFileヘルパーメソッドの廃止【本仕様の核心】
**ID**: REQ-10  
**Title**: PastaFileヘルパーメソッドの廃止  
**Priority**: 🔴 CRITICAL - 本仕様の根本的な目的  
**Statement**: The PastaFile helper methods `file_attrs()`, `words()`, `global_scene_scopes()`, and `actor_scopes()` shall be removed from the PastaFile implementation, forcing all transpilers and consumers to iterate directly over `file.items`.  
**Rationale**: 
- **根本問題**: これらのメソッドが存在すること自体が、出現順を無視した実装を誘発する
- **設計原則**: "あれば使ってしまう" → 開発者（人間・LLM問わず）は便利なヘルパーがあれば使用する
- **結果**: 型別フィルタリングによりFileItem出現順が失われ、正しいトランスパイルが不可能になる
- **解決策**: API自体を廃止し、`file.items`の直接イテレーションを強制することで、構造的に出現順処理を保証する

**Scope Note**: 本要件はpasta_core（PastaFile定義）の変更を伴うため、pasta_lua・pasta_rune両方のトランスパイラーとテストに影響する。影響範囲：
- pasta_lua: 50マッチ（6ファイル）
- pasta_rune: 22マッチ（3ファイル）
- **合計: 70マッチ以上の修正が必要**

**Acceptance Criteria**:
- [ ] pasta_core の PastaFile から以下のメソッドが削除される：
  - `file_attrs()` - 型別フィルタリングによる順序喪失
  - `words()` - 型別フィルタリングによる順序喪失
  - `global_scene_scopes()` - 型別フィルタリングによる順序喪失
  - `actor_scopes()` - 型別フィルタリングによる順序喪失
- [ ] pasta_rune の TranspileContext2 から以下のメソッドが削除される：
  - `file_attrs()` - HashMapによる列挙は順序情報を喪失（害悪）
- [ ] pasta_lua の TranspileContext に同様のメソッドがあれば削除される
- [ ] pasta_lua のすべての使用箇所が `file.items` イテレーションに修正される
- [ ] pasta_rune のすべての使用箇所が `file.items` イテレーションまたは内部フィールドアクセスに修正される
- [ ] tests/ ディレクトリのすべての使用箇所が修正される
- [ ] `cargo check --all` が成功する
- [ ] `cargo test --all` が成功する

## Out of Scope

- LuaCodeGenerator内部の変更（必要最小限を除く）
- Lua出力フォーマットの変更
- pasta_luaランタイムの変更
- パフォーマンス最適化（本移行の範囲外）
