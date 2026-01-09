# Research & Design Decisions

---
**Feature**: `pasta_lua_transpiler`
**Discovery Scope**: Extension（既存システムへの修正）
**Key Findings**:
- 6つの行レベル変更のみで Act-first アーキテクチャ対応完了
- 既存の StringLiteralizer パターンを活用可能
- 親仕様の API 設計との完全な整合性を確認
---

## Summary

本仕様は `pasta_lua_design_refactor` 親仕様で定義された Act-first アーキテクチャに準拠するため、`code_generator.rs` の Lua 出力パターンを修正する。

### Discovery 結果

| 項目 | 分析結果 |
|-----|---------|
| 変更タイプ | Extension（既存コード修正） |
| 変更箇所 | 6つの行レベル修正（L253, L268, L270, L278, L509, L510） |
| 新規依存 | なし（既存依存のみ使用） |
| API変更 | 出力形式のみ（code_generator.rs の公開 API 変更なし） |
| テスト影響 | transpiler_integration_test.rs の期待値更新必要 |

---

## Research Log

### 親仕様 API 設計確認

**Context**: 親仕様 `pasta_lua_design_refactor` の API 設計を確認し、code_generator.rs が生成すべき Lua コードパターンを特定

**Sources Consulted**: 
- [pasta_lua_design_refactor/design.md](.kiro/specs/completed/pasta_lua_design_refactor/design.md) L50-700
- [pasta_lua_design_refactor/requirements.md](.kiro/specs/completed/pasta_lua_design_refactor/requirements.md)

**Findings**:

1. **シーン関数シグネチャ**:
   - 設計: `function SCENE.__start__(act, ...)`
   - act は `pasta.act` オブジェクトへの参照
   - ctx は co_action 内部で管理され、シーン関数には渡されない

2. **init_scene パターン**:
   - 設計: `local save, var = act:init_scene(SCENE)`
   - act が既に第1引数で渡されるため、create_session は不要
   - init_scene は current_scene を設定し、save/var 参照を返す

3. **スポット管理**:
   - 設計: `act:clear_spot()`, `act:set_spot("name", idx)`
   - act メソッドとして実装（ctx 引数不要）

4. **アクタープロキシ**:
   - 設計: `act.アクター:talk()`, `act.アクター:word()`
   - __index メタメソッドで動的にプロキシ生成
   - word() は検索のみ、結果を talk() で包む: `act.アクター:talk(act.アクター:word("名前"))`

5. **さくらスクリプト**:
   - 設計: `act:sakura_script(text)`
   - アクター非依存、専用メソッド

**Implications**: 
- code_generator.rs の 6 箇所を修正することで完全準拠可能
- 既存の StringLiteralizer は引き続き使用
- 構造的なリファクタリング不要

### 現行実装分析

**Context**: code_generator.rs の現在の実装構造を確認

**Sources Consulted**:
- [code_generator.rs](crates/pasta_lua/src/code_generator.rs) L230-600

**Findings**:

1. **generate_local_scene() (L237-290)**:
   - L253: シグネチャ出力 `function SCENE.{}(ctx, ...)`
   - L268: `PASTA.clear_spot(ctx)` 出力
   - L270: `PASTA.set_spot(ctx, "{}", {})` 出力
   - L278: `local act, save, var = PASTA.create_session(SCENE, ctx)` 出力

2. **generate_action() (L453-540)**:
   - L506: `act.{}:talk({})` 形式（既に正しい）
   - L509: SakuraScript を `act.{}:talk({})` で出力（要変更）
   - L510 付近: word() 出力は talk() でラップ必要

3. **StringLiteralizer 使用**:
   - L507: talk() の文字列引数で使用（OK）
   - SakuraScript でも使用（OK）
   - word() の引数でも使用必要（確認事項）

**Implications**:
- 6 つの変更箇所は明確に特定済み
- 各変更は独立しており、段階的に実装可能

### StringLiteralizer 統一ルール確認

**Context**: 文字列リテラル処理の一貫性を確認

**Findings**:
- 現行: StringLiteralizer::literalize() を talk(), SakuraScript で使用
- 要件: すべての文字列リテラル出力で統一使用（例外なし）
- word() 引数も StringLiteralizer 経由で処理する必要あり

**Implications**:
- word() ラッピング実装時に StringLiteralizer を適用

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: 行レベル修正 | 6つの変更を個別に実施 | 段階的検証可能、問題特定容易 | 中間状態でテスト失敗 | **採用** |
| Option B: 統合修正 | すべての変更を一括実施 | 単一コミット | リグレッション特定困難 | 非推奨 |

**選択理由**: Option A（行レベル修正）
- 各変更が独立しており、相互依存性が低い
- 段階的なテスト検証が可能
- 問題発生時の原因特定が容易

---

## Design Decisions

### Decision: シーン関数シグネチャの変更方式

**Context**: ctx → act への第1引数変更

**Alternatives Considered**:
1. 文字列置換のみ
2. 新しいパラメータ構造の導入

**Selected Approach**: 文字列置換のみ（Option 1）

**Rationale**: 
- 既存の関数シグネチャ生成ロジックは維持
- 出力文字列の `ctx` を `act` に変更するだけで対応可能

**Trade-offs**: 
- ✅ 最小限の変更
- ✅ 既存テストの更新が容易
- ❌ なし

### Decision: init_scene の戻り値変更

**Context**: create_session の 3 値戻りから init_scene の 2 値戻りへ

**Alternatives Considered**:
1. 戻り値パターンを直接変更
2. 互換レイヤーを追加

**Selected Approach**: 戻り値パターンを直接変更（Option 1）

**Rationale**:
- act は既にシーン関数の第1引数として渡される
- save/var のみを init_scene から取得する設計に準拠

**Trade-offs**:
- ✅ 親仕様との完全な整合性
- ✅ シンプルな実装
- ❌ 既存テストの期待値更新が必要

### Decision: word() の talk() ラッピング実装

**Context**: word() 呼び出しを talk() メソッドで包む必要

**Alternatives Considered**:
1. generate_action() 内で word() 検出時にラッピング
2. 新しい Action バリアントを追加

**Selected Approach**: generate_action() 内でラッピング（Option 1）

**Rationale**:
- 親仕様 design.md L891 の例: `act.さくら:talk(act.さくら:word("笑顔"))`
- 既存の Action::Word 処理を修正するのみ

**Trade-offs**:
- ✅ 最小限のコード変更
- ✅ 既存パターンとの一貫性
- ❌ なし

### Decision: さくらスクリプト専用メソッド

**Context**: アクター経由の talk() から act:sakura_script() への変更

**Alternatives Considered**:
1. Action::SakuraScript のマッチング分岐を変更
2. 新しいアクション処理パスを追加

**Selected Approach**: マッチング分岐を変更（Option 1）

**Rationale**:
- 既存の Action::SakuraScript 処理を直接修正
- `act.{}:talk()` を `act:sakura_script()` に置換

**Trade-offs**:
- ✅ 単純な文字列パターン変更
- ✅ StringLiteralizer は引き続き使用
- ❌ なし

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| 変更漏れ | 中 | grep で全対象行を事前検索、実装完了後に再検証 |
| テスト失敗の連鎖 | 低 | 各変更後に `cargo test --lib` 実行 |
| 出力形式不正 | 低 | 親仕様の例と照合、Lua 構文検証 |
| StringLiteralizer 適用漏れ | 低 | word() 引数処理時に明示的に literalize() 呼び出し |

---

## References

- [pasta_lua_design_refactor/design.md](.kiro/specs/completed/pasta_lua_design_refactor/design.md) — 親仕様の Lua API 設計
- [code_generator.rs](crates/pasta_lua/src/code_generator.rs) — 変更対象ファイル
- [transpiler_integration_test.rs](crates/pasta_lua/tests/transpiler_integration_test.rs) — テスト更新対象
- [gap_analysis.md](.kiro/specs/pasta_lua_transpiler/gap_analysis.md) — 6つの変更の詳細分析
