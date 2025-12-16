# Active Specifications

このドキュメントは、現在進行中のPasta仕様の一覧と状態をまとめています。

---

## 進行中仕様一覧（9件）

### 1. pasta-call-resolution-priority
**カテゴリ**: 制御フロー  
**フェーズ**: 不明  
**概要**: Call文解決時の優先順位（ローカル → グローバル → Rune関数）を定義・実装。

**課題**:
- ローカルラベルとグローバルラベルの同名衝突解決
- 明示的グローバル指定（`＠＊関数名`）の実装

**関連**: `pasta-local-rune-calls`

---

### 2. pasta-conversation-inline-multi-stage-resolution
**カテゴリ**: 会話解決  
**フェーズ**: 不明  
**概要**: インライン会話内での多段階解決機構（変数展開 → 関数呼び出し → ラベル参照）。

**課題**:
- ネストした変数展開
- 関数戻り値の再展開
- パフォーマンス最適化

---

### 3. pasta-dialogue-continuation-syntax
**カテゴリ**: DSL文法  
**フェーズ**: 不明  
**概要**: 対話継続構文の実装。会話の自然な流れを記述するための文法拡張。

**課題**:
- 継続マーカー構文定義
- トランスパイル方法
- 既存文法との互換性

---

### 4. pasta-jump-function-calls
**カテゴリ**: 制御フロー  
**フェーズ**: 不明  
**概要**: Jump文（`－label`）からRune関数を直接呼び出す機能。

**課題**:
- Jump vs Call のセマンティクス統一
- 関数呼び出し後の継続処理

**関連**: `pasta-yield-propagation`, `pasta-local-rune-calls`

---

### 5. pasta-label-continuation
**カテゴリ**: 制御フロー  
**フェーズ**: 不明  
**概要**: ラベルの連鎖実行機構。複数ラベルを順次実行する仕組み。

**課題**:
- 継続チェーンの定義方法
- 循環参照検出
- デバッグ可能性

---

### 6. pasta-local-rune-calls
**カテゴリ**: Rune統合  
**フェーズ**: 不明  
**要件**: 定義済み  
**概要**: DSL内Runeブロックで定義されたローカル関数を、DSL会話行・Call行から呼び出す機能。

**要件**:
- ✅ `＠関数`、`＠関数（引数）`で呼び出し可能
- ✅ Call行から呼び出し可能
- ✅ 第一引数にキャラクター構造体（`ctx`）が渡される

**課題**:
- ユニットテスト実装
- スコープ解決（ローカル vs グローバル）
- ドキュメント更新

**関連**: `GRAMMAR.md` Runeコードブロックセクション

---

### 7. pasta-word-definition-dsl
**カテゴリ**: DSL文法  
**フェーズ**: design-generated（設計承認待ち）  
**要件**: 定義済み  
**設計**: 生成済み（未承認）  
**概要**: 単語定義DSLの実装。単語keyによる前方一致ランダム呼び出し機構。

**目標**:
- ラベルと同様に単語keyで前方一致検索
- シャッフルされた候補からランダム選択
- 会話のバリエーション強化

**課題**:
- 設計レビュー待ち（`/kiro-validate-design pasta-word-definition-dsl`推奨）
- 承認後のタスク分解・実装

**関連**: `product.md` 前方一致単語呼び出し目標

---

### 8. pasta-yield-propagation
**カテゴリ**: ランタイム（最優先）  
**フェーズ**: 不明  
**要件**: 定義済み  
**概要**: **最重要未完了仕様**。Call/Jump文で呼び出されたラベル関数のyieldイベント伝搬問題の解決。

**問題**:
RuneはネストしたGenerator関数のyieldを透過的に伝搬しない。現在のトランスパイル出力では、Call先のイベントが全て失われる。

**検証結果**:
```rune
pub fn a() {
    yield 11;
    yield 12;
    b();      // ← b()内のyield 21, 22は返らない
    yield 13;
}
fn b() {
    yield 21;
    yield 22;
}
// 実際の出力: 11, 12, 13 （21, 22が消失）
```

**解決方針**:
1. 呼び出し先関数をGenerator化
2. 手動yield伝搬: `for event in callee_gen { yield event; }`
3. Transpilerで適切なコード生成

**課題**:
- Transpiler修正
- Call文・Jump文の両方に対応
- パフォーマンス影響評価
- 既存テストの更新

**影響度**: 🔴 HIGH - Call/Jump文が正常動作しない

**関連**: `pasta-jump-function-calls`, `pasta-declarative-control-flow`

---

### 9. ukagaka-desktop-mascot
**カテゴリ**: メタ仕様  
**フェーズ**: completed（子仕様管理中）  
**完了日**: 2025-11-29  
**概要**: 伺か互換デスクトップマスコット実現のための統合メタ仕様。32の子仕様を管理。

**子仕様総数**: 32件
- **Kiro開発プロセス**: 1件（`kiro-P0-roadmap-management`）
- **wintf UI基盤**: 7件（P0: 5件、P1: 2件）
- **areka コア**: 7件（P0: 4件、P1: 3件）
- **areka リファレンス**: 4件（P0: 4件）
- **areka 高度機能**: 6件（P1-P2）
- **areka 将来機能**: 4件（P3）
- **areka IDE統合**: 3件（P1-P3）

**子仕様状態**:
- ✅ 完了: `areka-P0-script-engine`（Pasta統合基盤）
- 🔄 進行中/未着手: 31件

**主要子仕様**:
- `areka-P0-package-manager`: ゴーストパッケージ管理
- `areka-P0-persistence`: 状態永続化
- `areka-P0-mcp-server`: MCP Server統合
- `wintf-P0-image-widget`: 画像ウィジェット
- `wintf-P0-event-system`: イベントシステム
- `wintf-P0-typewriter`: タイプライター効果
- `wintf-P0-balloon-system`: バルーン（吹き出し）

**ロードマップ**:
- Phase 0: プロセス支援仕様作成（完了）
- Phase 1: 31子仕様requirements.md作成（完了）
- Phase 2-6: 子仕様実装・統合・リリース（進行中）

---

## 優先順位マトリクス

### P0（最優先）
1. **pasta-yield-propagation** 🔴 - Call/Jump文が動作しない
2. **pasta-local-rune-calls** - Runeブロック統合

### P1（高優先度）
3. **pasta-word-definition-dsl** - コア機能（前方一致単語呼び出し）
4. **pasta-call-resolution-priority** - スコープ解決の明確化

### P2（中優先度）
5. **pasta-label-continuation** - 会話チェーン機能
6. **pasta-jump-function-calls** - Jump文拡張

### P3（低優先度）
7. **pasta-conversation-inline-multi-stage-resolution** - 高度な会話解決
8. **pasta-dialogue-continuation-syntax** - 文法拡張
9. **ukagaka-desktop-mascot** - 長期メタ仕様（継続監視）

---

## 推奨アクション

### 即座に着手すべき仕様
1. **pasta-yield-propagation**
   - `/kiro-spec-status pasta-yield-propagation` で状態確認
   - 要件が定義済みなら `/kiro-spec-design pasta-yield-propagation` で設計生成
   - 設計承認後 `/kiro-spec-tasks pasta-yield-propagation` でタスク分解
   - `/kiro-spec-impl pasta-yield-propagation` で実装

2. **pasta-local-rune-calls**
   - 要件定義済み、テスト実装とドキュメント更新

### レビュー待ち
3. **pasta-word-definition-dsl**
   - `/kiro-validate-design pasta-word-definition-dsl` で設計レビュー
   - 承認後タスク分解・実装

---

## 依存関係グラフ

```
pasta-yield-propagation (P0)
  ├── pasta-declarative-control-flow (完了)
  └── pasta-jump-function-calls (P2)

pasta-local-rune-calls (P0)
  └── pasta-call-resolution-priority (P1)

pasta-word-definition-dsl (P1)
  └── pasta-label-resolution-runtime (完了)

ukagaka-desktop-mascot (メタ)
  ├── areka-P0-script-engine (完了)
  └── 31 子仕様 (進行中)
```

---

## 次のステップ

1. **pasta-yield-propagation**を最優先で完了させる（Call/Jump文修正）
2. **pasta-local-rune-calls**のテスト実装を完了させる
3. **pasta-word-definition-dsl**の設計レビューと実装
4. 残りの制御フロー仕様を順次完了
5. **ukagaka-desktop-mascot**子仕様の段階的実装

進行中仕様の状態は `/kiro-spec-status {feature-name}` で随時確認してください。
