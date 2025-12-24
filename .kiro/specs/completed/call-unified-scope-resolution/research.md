# Research & Design Decisions

---
**Feature**: `call-unified-scope-resolution`  
**Discovery Scope**: Extension（既存システム拡張）  
**Last Updated**: 2025-12-24  
**Key Findings** (Updated):
1. ✅ WordTable の 2段階検索＋マージ実装が完全なリファレンスとして存在
2. ✅ `JumpTarget` 列挙型は削除済み - `CallScene.target` は単純な `String` 型
3. ✅ `TranspileContext2.current_module()` は既に実装済み（単語登録で使用中）
4. ✅ Transpiler層は `code_generator.rs` に刷新済み（`generate_call_scene()` を修正対象）
---

## Summary

本機能は Call 文（＞シーン）のスコープ解決を単語検索（＠単語）と統一する拡張です。

**2025-12-24 更新**: パーサー・トランスパイラー刷新により実装スコープが縮小：
- **Parser層**: 変更不要（`JumpTarget` 削除済み、`＊` 構文は非サポート）
- **Transpiler層**: `code_generator.rs` の `generate_call_scene()` のみ修正
- **Runtime層**: `SceneTable.find_scene_merged()` 追加
- **Stdlib層**: `select_scene_to_id()` シグネチャ変更

## Research Log

### WordTable の 2段階検索パターン

- **Context**: 単語検索のスコープ解決実装を確認し、シーン検索への流用可能性を調査
- **Sources Consulted**: 
  - [src/runtime/words.rs](src/runtime/words.rs) L106-L150 `collect_word_candidates()`
- **Findings**:
  - 検索キー形式: ローカル = `:module_name:key`、グローバル = `key`
  - Step 1: ローカル検索 `iter_prefix(":module:key")`
  - Step 2: グローバル検索 `iter_prefix("key")` ただし `:` で始まるキーを除外
  - Step 3: 両方の entry_ids をマージして word リストを構築
- **Implications**: 
  - 同一パターンを SceneTable に適用可能
  - `:` プレフィックスによるローカル/グローバル区別が確立済み

### Transpiler (CodeGenerator) の Call 文処理 ✅ 2025-12-24 更新

- **Context**: パーサー刷新後の Call 文処理を確認
- **Sources Consulted**: 
  - [src/transpiler/code_generator.rs](src/transpiler/code_generator.rs) L186-191 `generate_call_scene()`
- **Findings**:
  - 現在の生成コード: `pasta::call(ctx, "{target}")`
  - `JumpTarget` 列挙型は削除済み、`CallScene.target` は単純な `String`
  - `self.context.current_module()` は利用可能（`generate_word()` で使用実績あり）
- **Implications**: 
  - `module_name` を第2引数として追加: `pasta::call(ctx, "{target}", "{module_name}")`
  - シンプルな1箇所の変更で対応可能

### TranspileContext2 の current_module ✅ 2025-12-24 確認

- **Context**: current_module がどのように設定・使用されているか確認
- **Sources Consulted**: 
  - [src/transpiler/context.rs](src/transpiler/context.rs) `current_module()` / `set_current_module()`
- **Findings**:
  - `current_module: String` フィールドが既存
  - 単語登録（`word_registry.register_local()`）で使用中
  - `generate_word()` で `self.context.current_module()` として参照
- **Implications**: 
  - **追加実装不要** - `generate_call_scene()` でも同じ方法で使用可能

## Architecture Pattern Evaluation

| Option             | Description                                                                | Strengths                     | Risks / Limitations         | Notes    |
| ------------------ | -------------------------------------------------------------------------- | ----------------------------- | --------------------------- | -------- |
| A: Extend existing | SceneTable に `find_scene_merged()` 追加、transpiler/stdlib シグネチャ変更 | 最小変更、word パターン再利用 | 4ファイル変更が連鎖         | **採用** |
| B: New component   | ScopeResolver 新規作成                                                     | 責務分離                      | 過剰設計、word との非対称性 | 不採用   |

## Design Decisions

### Decision: 検索キーのスコープ区別方式

- **Context**: ローカルシーンとグローバルシーンを前方一致検索時に区別する方法
- **Alternatives Considered**:
  1. Option A: prefix_index 登録時に `:parent:local` 形式に変換（WordTable 完全統一）
  2. Option B: 検索時に `parent::prefix` で動的構築（fn_name 形式維持）
  3. Option C: fn_name 自体を `:` 区切りに変更（大規模変更）
- **Selected Approach**: **Option A**（prefix_index 登録時にキー変換）
- **Decision Date**: 2025-12-21
- **Rationale**: 
  - WordTable と完全統一により、実装パターン流用が最も容易
  - `collect_word_candidates()` のロジックをそのまま適用可能
  - テストケースも word 実装を参考にできる
- **Trade-offs**: 
  - ✅ WordTable との完全対称性
  - ✅ 実装工数削減（既存パターン流用）
  - 🟡 fn_name と検索キーが異なる（SceneInfo に両方を保持）
  - 🟡 prefix_index 構築時のキー変換コスト（初回のみ、許容範囲）
- **Follow-up**: SceneTable::from_scene_registry() でキー変換実装、テストで `:module:prefix` 検索を検証

### Decision: キャッシュキーへの module_name 追加

- **Context**: 同一 search_key でも異なる module から呼び出すと候補が異なる
- **Alternatives Considered**:
  1. キャッシュキーに module_name を追加
  2. キャッシュを廃止
- **Selected Approach**: Option 1（キャッシュキー拡張）
- **Rationale**: 
  - 既存の WordCacheKey と同様の設計
  - パフォーマンス維持
- **Trade-offs**: 
  - キャッシュエントリ数が増加（module × key × filters）
  - メモリ使用量微増

### Decision: stdlib 関数シグネチャ変更

- **Context**: `select_scene_to_id` に module_name 引数を追加
- **Alternatives Considered**:
  1. 必須引数として追加（破壊的変更）
  2. オプション引数として追加（後方互換）
- **Selected Approach**: Option 1（必須引数）
- **Rationale**: 
  - 内部 API であり外部公開していない
  - 生成コードのみが呼び出し元（Pass 2 で生成）
  - 後方互換性の維持不要
- **Trade-offs**: 
  - 全ての Call 文生成コードを変更必要
- **Follow-up**: 既存テストの生成コード期待値を更新

## Risks & Mitigations

- **Risk 1**: ローカルシーンの検索キー形式不整合
  - Mitigation: word と同じ `:module:prefix` パターンを厳密に踏襲
  - Validation: 単体テストで `:` プレフィックス検索を検証

- **Risk 2**: キャッシュ汚染（異なる module で同一キャッシュ使用）
  - Mitigation: SceneCacheKey に module_name を追加
  - Validation: テストで異なる module からの呼び出しを検証

- **Risk 3**: 既存テストの挙動変化
  - Mitigation: fixtures 調査済み（`＞＊` 未使用）
  - Validation: `cargo test --all` で全テスト成功確認

## References

- [src/runtime/words.rs](src/runtime/words.rs) - WordTable 2段階検索実装
- [src/runtime/scene.rs](src/runtime/scene.rs) - SceneTable 現在の実装
- [src/transpiler/mod.rs](src/transpiler/mod.rs) - Transpiler Call 文処理
- [src/stdlib/mod.rs](src/stdlib/mod.rs) - select_scene_to_id 関数
- [SPECIFICATION.md](SPECIFICATION.md) Section 4 - Call 詳細仕様
- [SPECIFICATION.md](SPECIFICATION.md) Section 10.3 - 単語参照スコープ解決ルール
