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

### REQ-3: ファイルレベル属性処理
**ID**: REQ-3  
**Title**: ファイルレベル属性処理  
**Statement**: When LuaTranspiler encounters a FileItem::FileAttr during iteration, the LuaTranspiler shall accumulate the attribute and apply it according to pasta_rune's file attribute handling rules.  
**Acceptance Criteria**:
- [ ] FileAttrがTranspileContext内で累積される
- [ ] 累積された属性が後続のシーン/アクター生成に影響する
- [ ] pasta_runeのaccumulate_file_attr()と同等の動作をする

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

### REQ-6: アクター処理順序
**ID**: REQ-6  
**Title**: アクター処理順序  
**Statement**: When LuaTranspiler encounters a FileItem::ActorScope during iteration, the LuaTranspiler shall process the actor definition with awareness of previously accumulated file attributes.  
**Acceptance Criteria**:
- [ ] アクター処理時に累積されたファイル属性が利用可能
- [ ] アクター内の単語定義がアクタースコープで正しく処理される

## Non-Functional Requirements

### REQ-7: API一貫性
**ID**: REQ-7  
**Title**: API一貫性  
**Statement**: The LuaTranspiler shall provide an API that is consistent with pasta_rune's Transpiler2, using the same method naming conventions and parameter patterns where applicable.  
**Acceptance Criteria**:
- [ ] メソッド名がpasta_runeと一致（transpile_file または transpile_pass1）
- [ ] パラメータ順序がpasta_runeと一致
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

### REQ-10: PastaFileヘルパーメソッドの廃止
**ID**: REQ-10  
**Title**: PastaFileヘルパーメソッドの廃止  
**Statement**: The PastaFile helper methods `file_attrs()`, `words()`, `global_scene_scopes()`, and `actor_scopes()` shall be removed from the PastaFile implementation, forcing all transpilers and consumers to iterate directly over `file.items`.  
**Rationale**: These methods encourage type-filtered access patterns that ignore FileItem出現順, which is critical for correct transpilation. Direct `file.items` iteration ensures order-sensitive processing is always performed.  
**Acceptance Criteria**:
- [ ] PastaFile から以下のメソッドが削除される：
  - `file_attrs()`
  - `words()`
  - `global_scene_scopes()`
  - `actor_scopes()`
- [ ] 削除されたメソッドを使用していたテストがすべて `file.items` を使用するように修正される
- [ ] 削除されたメソッドを使用していた他のコード（transpiler含む）がすべて修正される
- [ ] コンパイルエラーが生じないことを確認

## Out of Scope

- LuaCodeGenerator内部の変更（必要最小限を除く）
- Lua出力フォーマットの変更
- pasta_luaランタイムの変更
- パフォーマンス最適化（本移行の範囲外）
