# Implementation Tasks: lua-transpiler-pastafile-migration

## Task Overview

**Feature**: pasta_luaトランスパイラーをPastaFile入力ベースに移行し、FileItem出現順処理を実装する
**Total Tasks**: 5 Major tasks
**Estimated Effort**: 6-9時間（フェーズ1: 3-4時間、フェーズ2: 3-5時間）
**Status**: ✅ 完了

## Task List

### MAJOR-1: TranspileContextの拡張 (P)
**Requirement IDs**: 3
**Status**: ✅ 完了

**Description**: 
`TranspileContext`にファイルレベル属性の累積機能を追加し、pasta_runeの`TranspileContext2`と同等の属性管理を実装する。ファイル属性のシャドーイング（後勝ち）をサポートし、シーン処理時のファイル属性とシーン属性のマージ機能を提供する。

**Deliverables**:
- [x] `file_attrs`フィールド（HashMap<String, AttrValue>）を追加
- [x] `accumulate_file_attr(&Attr)`メソッドを実装
- [x] `merge_attrs(&[Attr])`メソッドを実装
- [x] シャドーイング動作を検証するユニットテストを作成
- [x] マージ動作を検証するユニットテストを作成

**Test Coverage**:
- [x]* 属性累積の基本動作テスト
- [x]* シャドーイング（同じキーの上書き）テスト
- [x]* ファイル属性とシーン属性のマージテスト

---

### MAJOR-2: LuaTranspilerのPastaFile入力対応
**Requirement IDs**: 1, 2
**Status**: ✅ 完了

**Description**: 
既存の`transpile()`メソッドを削除し、`&PastaFile`を第一引数として受け取る新しいシグネチャに変更する。FileItem列挙を出現順に処理し、各種別（FileAttr、GlobalWord、GlobalSceneScope、ActorScope）を適切にハンドリングする。これはバグ修正であり、古いAPIは削除する。

**Deliverables**:
- [ ] `transpile(&PastaFile, ...)`の新シグネチャを実装
- [ ] `transpile_with_globals()`メソッドを削除
- [ ] `file.items`イテレーションによるFileItem出現順処理を実装
- [ ] FileItem種別ごとの処理ロジック（match文）を実装
- [ ] 各FileItem種別に対するユニットテストを作成

**Sub-tasks**:

#### MAJOR-2.1: FileAttr処理の実装 (P)
**Requirement IDs**: 3

**Description**: 
FileItem::FileAttrを検出し、`TranspileContext::accumulate_file_attr()`を呼び出して属性を累積する。コード生成は行わず、累積のみを実施する。

**Deliverables**:
- [x] FileItem::FileAttrのmatchアームを実装
- [x] accumulate_file_attr()の呼び出しを実装
- [x] FileAttr累積を検証するテストを作成

#### MAJOR-2.2: GlobalWord処理の実装 (P)
**Requirement IDs**: 4
**Status**: ✅ 完了

**Description**: 
FileItem::GlobalWordを検出し、`WordDefRegistry::register_global()`を呼び出してグローバル単語を登録する。登録順序がファイル内の出現順序と一致することを保証する。

**Deliverables**:
- [x] FileItem::GlobalWordのmatchアームを実装
- [x] WordDefRegistry::register_global()の呼び出しを実装
- [x] グローバル単語登録を検証するテストを作成

#### MAJOR-2.3: GlobalSceneScope処理の実装 (P)
**Requirement IDs**: 5
**Status**: ✅ 完了

**Description**: 
FileItem::GlobalSceneScopeを検出し、累積されたファイル属性とシーン属性をマージ後、シーンを登録しコード生成する。`TranspileContext::merge_attrs()`を呼び出し、結果を`LuaCodeGenerator::generate_global_scene()`に渡す。

**Deliverables**:
- [x] FileItem::GlobalSceneScopeのmatchアームを実装
- [x] merge_attrs(&scene.attrs)の呼び出しを実装
- [x] SceneRegistry::register_global()の呼び出しを実装
- [x] LuaCodeGenerator::generate_global_scene()へのマージ結果渡しを実装
- [x] シーン処理順序とファイル属性マージを検証するテストを作成

#### MAJOR-2.4: ActorScope処理の実装 (P)
**Requirement IDs**: 6
**Status**: ✅ 完了

**Description**: 
FileItem::ActorScopeを検出し、アクター定義を出現順に処理する。アクターはファイル属性を継承しないが、出現順処理を保証する必要がある。

**Deliverables**:
- [x] FileItem::ActorScopeのmatchアームを実装
- [x] LuaCodeGenerator::generate_actor()の呼び出しを実装
- [x] アクター処理順序を検証するテストを作成
- [x] アクターがファイル属性を継承しないことを検証するテストを作成

---

### MAJOR-3: LuaCodeGeneratorのシグネチャ拡張 (P)
**Requirement IDs**: 3, 5
**Status**: ✅ 完了

**Description**: 
`LuaCodeGenerator::generate_global_scene()`メソッドにファイル属性引数を追加する。将来の拡張性のため引数に含めるが、現時点では未使用パラメータとして扱う。

**Deliverables**:
- [x] `generate_global_scene()`のシグネチャに`file_attrs: &HashMap<String, AttrValue>`を追加
- [x] 既存の呼び出し箇所を新シグネチャに適合させる（MAJOR-2.3で実施）
- [x] シグネチャ変更がコンパイル可能であることを確認

**Test Coverage**:
- [x]* 新シグネチャでの呼び出しが成功することを既存テストで検証

---

### MAJOR-4: 統合テストの作成
**Requirement IDs**: 2, 3, 5, 6, 8
**Status**: ✅ 完了

**Description**: 
FileItem出現順処理、ファイル属性のシャドーイング、アクターのファイル属性非継承を検証する統合テストを作成する。pasta_runeのテストパターンを参考に、同等のカバレッジを確保する。

**Deliverables**:
- [x] FileItem出現順序を検証する統合テストを作成
- [x] ファイル属性シャドーイング（順序依存性）を検証する統合テストを作成
- [x] アクターがファイル属性を継承しないことを検証する統合テストを作成
- [x] グローバル単語登録順序を検証するテストを作成

**Test Coverage**:
- [x]* FileItem出現順序の検証
- [x]* ファイル属性シャドーイング動作の検証
- [x]* アクターの属性非継承の検証
- [x]* pasta_runeとの一貫性検証

---

### MAJOR-5: PastaFileヘルパーメソッドの廃止
**Requirement IDs**: 9
**Status**: ✅ 完了

**Description**: 
pasta_coreのPastaFileから型別フィルタリングメソッド（`file_attrs()`, `words()`, `global_scene_scopes()`, `actor_scopes()`）を削除し、すべての使用箇所を`file.items`イテレーションに書き換える。

**Deliverables**:
- [x] PastaFileから4メソッドを削除（file_attrs, words, global_scene_scopes, actor_scopes）
- [x] pasta_luaの全使用箇所を`file.items`イテレーションに書き換え
- [x] pasta_runeの全使用箇所を`file.items`イテレーションまたは内部フィールドアクセスに書き換え
- [x] tests/ディレクトリの全使用箇所を書き換え
- [x] `cargo check --all`が成功することを確認
- [x] `cargo test --all`が成功することを確認（リグレッション0件）

**Note**: TranspileContext2.file_attrs()はTranspileContext内で累積された属性を返すgetterであり、PastaFileのヘルパーとは性質が異なるため維持。pasta_luaでも同様のパターンを採用（MAJOR-1で実装）。

**Test Coverage**:
- [x]* 既存テストがすべてパスすることを確認
- [x]* コンパイルエラーが0件であることを確認

## Dependencies & Sequencing

### Sequential Dependencies
- **MAJOR-1** → **MAJOR-2**: TranspileContext拡張が完了してから、LuaTranspilerで利用
- **MAJOR-2** → **MAJOR-4**: FileItem処理実装が完了してから、統合テストを実施
- **MAJOR-1, MAJOR-2, MAJOR-3, MAJOR-4** → **MAJOR-5**: フェーズ1完了後、ヘルパーメソッド廃止（フェーズ2）

### Parallelization Opportunities
- **MAJOR-1 (P)**, **MAJOR-3 (P)**: TranspileContextとLuaCodeGeneratorのシグネチャ拡張は並行可能
- **MAJOR-2サブタスク (P)**: FileAttr、GlobalWord、GlobalSceneScope、ActorScopeの処理実装は並行可能（ただしMAJOR-1完了後）

## Verification Checklist

### Phase 1 Completion Criteria
- [x] TranspileContextに`file_attrs`, `accumulate_file_attr()`, `merge_attrs()`が実装されている
- [x] `transpile(&PastaFile, ...)`が実装され、既存メソッドが削除されている
- [x] FileItem出現順処理がすべての種別で動作している
- [x] LuaCodeGeneratorが新シグネチャでコンパイル可能
- [x] すべてのユニットテストがパスしている
- [x] すべての統合テストがパスしている

### Phase 2 Completion Criteria
- [x] PastaFileから4メソッドが削除されている
- [x] すべての使用箇所が`file.items`イテレーションに書き換えられている
- [x] `cargo check --all`が成功している
- [x] `cargo test --all`が成功している（リグレッション0件）

### Final Acceptance
- [x] 全9要件がテストでカバーされている
- [x] pasta_runeとの一貫性が保たれている
- [x] 既存機能のリグレッションがゼロである
- [x] コンパイルエラーが存在しない
