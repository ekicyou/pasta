# Research Log: soul-document

## Summary

本リサーチでは、SOUL.mdで定義された「あるべき姿」を証明するRuntime E2Eテストの設計アプローチを調査しました。

### Key Findings

1. **既存テストパターンの発見**: `finalize_scene_test.rs`が既にRuntime E2E的なアプローチを持っているが、シーン辞書/単語辞書の「呼び出し側」動作が未検証
2. **統合ポイントの特定**: `PastaLuaRuntime`と`@pasta_search`モジュールが主要な統合対象
3. **ランダム性検証の課題**: 統計的検証にはサンプル数と許容誤差の設計が必要

### Chosen Approach

既存の`finalize_scene_test.rs`パターンを拡張し、以下の3層構成でRuntime E2Eテストを実装：

1. **Scene E2E Layer**: シーン辞書の前方一致→ランダム選択→実行の完全フロー
2. **Word E2E Layer**: 単語辞書のランダム選択→文字列置換の完全フロー
3. **Integration Layer**: Pastaスクリプト→トランスパイル→Lua実行→出力検証

---

## Research Log

### 2024-01-24: 既存テストアーキテクチャの調査

#### 調査対象
- `crates/pasta_lua/tests/finalize_scene_test.rs` (500行)
- `crates/pasta_lua/tests/search_module_test.rs` (327行)
- `crates/pasta_lua/tests/actor_word_dictionary_test.rs` (149行)

#### 発見事項

**finalize_scene_test.rs の構造**:
```rust
// E2E flow: Transpile → Execute → finalize_scene() → Search
fn create_runtime_with_finalize() -> mlua::Result<mlua::Lua>
fn transpile(source: &str) -> String
```

このパターンは「シーン辞書の登録」までは検証しているが、**実際のシーン呼び出し（Call文実行）とその結果**は未検証。

**search_module_test.rs の構造**:
```rust
// Registry layer test: TranspileContext → PastaLuaRuntime → search
fn create_test_context() -> TranspileContext
```

`search_scene`と`search_word`のAPI動作は検証済みだが、**Pastaスクリプトからの呼び出しフロー**は未検証。

#### ギャップ分析

| 検証済み | 未検証 |
|---------|--------|
| finalize_scene()によるレジストリ構築 | Call文実行後のシーン選択動作 |
| search_scene APIの応答 | 前方一致による複数候補からの選択分布 |
| search_word APIの応答 | 単語置換の実行時動作 |
| アクター単語のトランスパイル | アクター単語の実行時スコープ解決 |

### 2024-01-24: ランダム性検証アプローチの調査（更新）

#### 発見: キャッシュ機能の存在

単語辞書には**キャッシュ機能**が実装されていることが判明：
- 初回検索時に全要素をシャッフルして格納
- 同じ検索は順次キャッシュから消費（全要素消費後に再シャッフル）

この実装により、統計的検証は不要：
- N回実行しても必ず均等（N回 ÷ 要素数）になる
- テストすべきは「キャッシュ消費の正しさ」

#### 選定: キャッシュ消費テスト

```rust
// 3要素なら、3回で全要素が1回ずつ消費される
let mut results = HashSet::new();
for _ in 0..3 {
    results.insert(runtime.select_word("挨拶"));
}
assert_eq!(results.len(), 3, "3回で全要素が消費される");
```

**利点**:
- 決定的テスト（フレーキーなし）
- キャッシュ機能の正しさを直接検証
- シンプルで高速

### 2024-01-24: テストフィクスチャ設計の調査

#### 既存フィクスチャ
- `tests/fixtures/simple-test/` - 基本的なシーン定義
- `crates/pasta_lua/tests/fixtures/sample.pasta` - ローダーテスト用
- `tests/fixtures/comprehensive_control_flow.pasta` - 制御フロー検証

#### 新規フィクスチャ要件

**runtime_e2e_scene.pasta**:
```pasta
＊挨拶
  さくら：おはようございます

＊挨拶
  さくら：こんにちは

＊挨拶
  さくら：こんばんは

＊メイン
  ＞挨拶
```
→ 3つの「挨拶」シーンから1つがランダム選択されることを検証

**runtime_e2e_word.pasta**:
```pasta
＠挨拶：おはよう、こんにちは、こんばんは

＊メイン
  さくら：＠挨拶
```
→ 単語辞書からのランダム選択と文字列置換を検証

**runtime_e2e_actor_word.pasta**:
```pasta
％さくら
  ＠表情：笑顔、驚き、照れ

＊メイン
  さくら：＠表情　今日もいい天気だね
```
→ アクタースコープ単語の解決を検証

---

## Architecture Pattern Evaluation

### Pattern: Test Pyramid Extension

**概要**: 既存のUnit/Integration層を維持しつつ、E2E層を追加

```
         ╱╲
        ╱E2E╲          ← 新規追加（Runtime E2E）
       ╱──────╲
      ╱Integration╲    ← 既存（finalize_scene_test等）
     ╱──────────────╲
    ╱     Unit       ╲  ← 既存（parser, registry等）
   ╱──────────────────╲
```

**評価**:
- ✅ 既存テスト構造を破壊しない
- ✅ 段階的な検証が可能
- ✅ デバッグ容易性を維持
- ⚠️ E2E層は実行時間が長くなる可能性

### Pattern: Golden Test (Snapshot)

**概要**: 期待出力をファイルに保存し、実行結果と比較

```rust
#[test]
fn test_golden_output() {
    let output = execute_pasta("runtime_e2e_scene.pasta");
    insta::assert_snapshot!(output);
}
```

**評価**:
- ✅ リグレッション検知が容易
- ✅ 期待値の可視化
- ⚠️ ランダム出力には不向き（シード固定必須）
- ⚠️ `insta` crate導入が必要

### 選定: Test Pyramid Extension + 部分的Golden Test

- E2E層を新規追加
- 決定的出力部分にはGolden Test適用
- ランダム部分は統計的検証

---

## Design Decisions

### DD-1: テストファイル配置

**決定**: `crates/pasta_lua/tests/runtime_e2e_test.rs` に新規テストを配置

**理由**:
- 既存のテストファイル命名規則に準拠
- `pasta_lua`クレートがランタイム実行を担当
- 統合テストとして`tests/`ディレクトリに配置

### DD-2: TestHelper配置

**決定**: `tests/common/e2e_helpers.rs`に共通ヘルパーを配置し、既存テストもリファクタリング

**理由**:
- コードの重複排除による保守性向上
- `finalize_scene_test.rs`（500行）のリファクタリング価値
- 新規テスト追加と同時にリファクタリングする方が効率的

**影響**:
- `finalize_scene_test.rs`: ヘルパー関数を共通モジュールに移動
- `runtime_e2e_test.rs`: 共通ヘルパーを使用

**決定**: `crates/pasta_lua/tests/fixtures/e2e/` に新規フィクスチャを配置

**理由**:
- 既存フィクスチャと分離
- E2E専用であることを明示
- 将来の拡張に対応可能

### DD-3: ランダム性テスト戦略

**決定**: キャッシュ消費テスト（常時実行）

**理由**:
- 単語辞書のキャッシュ機能により、統計的検証は不要
- キャッシュ消費の正しさ（全要素が1回ずつ消費される）を検証
- 決定的テストのためフレーキーテストのリスクなし

### DD-5: pasta_shiori失敗テストの扱い

**決定**: Runtime E2Eテストとは別タスクとして扱う

**理由**:
- pasta_shiori失敗の原因は別問題（SHIORI.DLLライフサイクル）
- Runtime E2Eはpasta_lua層のテストに集中
- Phase 0完了のブロッカーとして別途対応

---

## Risks and Mitigations

### Risk 1: フレーキーテスト

**リスク**: ~~ランダム性テストがCI上で不安定になる~~ **解決済み**

**対策**: キャッシュ消費テストは決定的なため、フレーキーテストのリスクなし

### Risk 2: テスト実行時間増大

**リスク**: E2Eテストが`cargo test`全体の実行時間を増大させる

**軽減策**:
- キャッシュ消費テストはシンプル（3～4回の呼び出しのみ）
- フィクスチャのサイズを最小限に
- 並列実行を活用

### Risk 3: Lua環境依存

**リスク**: Lua VMの初期化やモジュールロードで環境差異が発生

**軽減策**:
- `create_runtime_with_finalize()`パターンを再利用
- 環境変数による設定を排除
- 相対パスを統一

