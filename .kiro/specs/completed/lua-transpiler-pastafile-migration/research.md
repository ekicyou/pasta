# Research & Design Decisions

---
**Purpose**: lua-transpiler-pastafile-migration 設計のための調査結果と設計判断を記録
---

## Summary
- **Feature**: `lua-transpiler-pastafile-migration`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - pasta_runeに参照実装（Transpiler2）が存在し、設計パターンが確立済み
  - pasta_luaのTranspileContextはpasta_runeより簡素で、拡張が容易
  - PastaFileヘルパーメソッドの廃止は76マッチに影響（メカニカルな修正）

## Research Log

### 既存pasta_luaアーキテクチャ分析
- **Context**: 現在のpasta_luaトランスパイラーの構造を理解
- **Sources Consulted**: 
  - [transpiler.rs](crates/pasta_lua/src/transpiler.rs)
  - [context.rs](crates/pasta_lua/src/context.rs)
- **Findings**:
  - `LuaTranspiler::transpile()`: `&[ActorScope]` + `&[GlobalSceneScope]` を入力
  - `LuaTranspiler::transpile_with_globals()`: 上記 + `&[KeyWords]` を追加入力
  - `TranspileContext`: `SceneRegistry` + `WordDefRegistry` を保持
  - ファイル属性の累積機能なし（pasta_runeにはある）
- **Implications**: 
  - TranspileContextに `file_attrs: HashMap<String, AttrValue>` と `accumulate_file_attr()` を追加必要
  - 新メソッド `transpile_file(&PastaFile, ...)` を追加

### pasta_rune参照実装分析
- **Context**: pasta_runeのTranspiler2から設計パターンを抽出
- **Sources Consulted**:
  - [transpiler/mod.rs](crates/pasta_rune/src/transpiler/mod.rs)
  - [transpiler/context.rs](crates/pasta_rune/src/transpiler/context.rs)
- **Findings**:
  - 2パス構造: `transpile_pass1()` + `transpile_pass2()`
  - `TranspileContext2::accumulate_file_attr()`: ファイル属性の累積
  - `TranspileContext2::merge_attrs()`: ファイル属性とシーン属性のマージ
  - FileItem matchパターン: FileAttr → GlobalWord → GlobalSceneScope → ActorScope
- **Implications**: 
  - pasta_luaは1パスで完結（Lua言語設計による）
  - accumulate_file_attr()のシグネチャを流用
  - ActorScopeはfile_attr継承なし（明確化済み）

### PastaFileヘルパーメソッド影響分析
- **Context**: REQ-10（ヘルパーメソッド廃止）の影響範囲を調査
- **Sources Consulted**:
  - grep検索結果（file_attrs, words, global_scene_scopes, actor_scopes）
  - [pasta_core/src/parser/ast.rs](crates/pasta_core/src/parser/ast.rs)
  - [pasta_core/src/parser/mod.rs](crates/pasta_core/src/parser/mod.rs)
- **Findings**:
  - 定義箇所: ast.rs（PastaFile impl）
  - 使用箇所: 76マッチ以上
    - pasta_lua: 50マッチ
    - pasta_rune: 22マッチ（TranspileContext2.file_attrs()含む）
    - tests/: ワークスペースレベルテスト
  - ドキュメント・コメント内での言及も含む
- **Implications**:
  - メカニカルな修正（file.items イテレーションへの置換）
  - テスト側の修正が中心、プロダクションコードは限定的

### Lua言語 vs Rune言語の設計差異
- **Context**: なぜpasta_luaは1パス処理で完結するか
- **Findings**:
  - Rune: scene_selector（関数ポインタ）をpass2で生成する必要あり
  - Lua: 動的言語のため、関数参照をランタイムで解決可能
  - Lua: モジュールシステムが異なり、事前登録不要
- **Implications**:
  - `transpile_file()` の1メソッドで完結
  - pass2は実装しない

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| オプションA: 段階的移行 | フェーズ1（新API）→ フェーズ2（非推奨化）→ フェーズ3（廃止） | 段階的検証可能、リスク最小 | フェーズ数が多い | **推奨** |
| オプションB: 完全リプレイス | 既存メソッド削除、transpile_file()のみ | シンプルなAPI | Breaking Change、段階的検証不可 | 非推奨 |
| オプションC: ハイブリッド | フェーズ2・3を並行実施 | 工期短縮 | 複雑性増加 | 中程度リスク |

## Design Decisions

### Decision: 1パス処理の採用
- **Context**: pasta_runeは2パス、pasta_luaはどうするか
- **Alternatives Considered**:
  1. 2パス（pasta_rune互換）
  2. 1パス（Lua言語最適化）
- **Selected Approach**: 1パス処理
- **Rationale**: Lua言語設計により2段階処理が不要。開発者確認済み。
- **Trade-offs**: API名のみ一貫性維持（transpile_file）、内部実装は異なる
- **Follow-up**: なし

### Decision: TranspileContext拡張（新規フィールド追加）
- **Context**: ファイル属性累積機能をどこに実装するか
- **Alternatives Considered**:
  1. TranspileContextに直接追加
  2. 新しいコンテキスト構造体を作成
- **Selected Approach**: 既存TranspileContextに追加
- **Rationale**: 既存パターンに従い、変更を最小化
- **Trade-offs**: TranspileContextの責務が増加
- **Follow-up**: file_attrs()メソッドは追加しない（害悪）、個別アクセサのみ

### Decision: アクター処理（属性非依存）
- **Context**: アクターはファイル属性を継承するか
- **Alternatives Considered**:
  1. ファイル属性を継承
  2. ファイル属性を継承しない（出現順処理のみ）
- **Selected Approach**: 属性非継承、出現順処理のみ
- **Rationale**: 開発者確認済み。アクターとシーンの設計意図が異なる。
- **Trade-offs**: なし
- **Follow-up**: なし

## Risks & Mitigations
- **フェーズ2でのデータ損失**: 非推奨メソッド呼び出し時に警告を記載、ドキュメント化
- **テスト修正漏れ**: grep検索で網羅的に検出、メカニカルな置換
- **回帰リスク**: 段階的検証により早期発見

## References
- [pasta_rune Transpiler2](crates/pasta_rune/src/transpiler/mod.rs) - 参照実装
- [pasta_core parser AST](crates/pasta_core/src/parser/ast.rs) - FileItem定義
- [gap-analysis.md](gap-analysis.md) - ギャップ分析レポート
