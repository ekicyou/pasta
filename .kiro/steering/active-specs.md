# Active Specifications

このドキュメントは、現在進行中のPasta仕様の一覧と状態をまとめています。

---

## 未着手仕様一覧（9件）= Phase 0・1修正仕様

**重要**: これらは新機能ではなく、`areka-P0-script-engine`の根本的な設計欠陥を修正するための仕様群。

### 1. pasta-call-resolution-priority 🟡 P1
**カテゴリ**: Phase 0・1修正（スコープ解決の明確化）  
**位置付け**: `areka-P0-script-engine`のCall文実装にスコープ解決が欠落していた問題の修正  
**概要**: Call文解決時の優先順位（ローカル → グローバル → Rune関数）を定義・実装。

**修正内容**:
- ローカルラベルとグローバルラベルの同名衝突解決ルール確立
- 明示的グローバル指定（`＠＊関数名`）の実装
- スコープ解決アルゴリズムの明確化

**関連**: `pasta-local-rune-calls`  
**Phase 0完了条件**: ✅ 必須（P1）

---

### 2. pasta-conversation-inline-multi-stage-resolution ⚪ P3
**カテゴリ**: Phase 1以降（高度機能）  
**位置付け**: 初期実装の欠落機能（基盤確立後に追加）  
**概要**: インライン会話内での多段階解決機構（変数展開 → 関数呼び出し → ラベル参照）。

**Phase 0完了条件**: ❌ 不要（P3・Phase 1以降）

---

### 3. pasta-dialogue-continuation-syntax ⚪ P3
**カテゴリ**: Phase 1以降（文法拡張）  
**位置付け**: 初期実装の欠落機能（基盤確立後に追加）  
**概要**: 対話継続構文の実装。会話の自然な流れを記述するための文法拡張。

**Phase 0完了条件**: ❌ 不要（P3・Phase 1以降）

---

### 4. pasta-jump-function-calls 🟢 P2
**カテゴリ**: Phase 1以降（制御フロー拡張）  
**位置付け**: Jump文機能拡張（基盤確立後に追加）  
**概要**: Jump文（`－label`）からRune関数を直接呼び出す機能。

**関連**: `pasta-yield-propagation`（依存）, `pasta-local-rune-calls`  
**Phase 0完了条件**: ❌ 不要（P2・Phase 1以降）

---

### 5. pasta-label-continuation 🟢 P2
**カテゴリ**: Phase 1以降（制御フロー拡張）  
**位置付け**: ラベル連鎖機構（基盤確立後に追加）  
**概要**: ラベルの連鎖実行機構。複数ラベルを順次実行する仕組み。

**Phase 0完了条件**: ❌ 不要（P2・Phase 1以降）

---

### 6. pasta-local-rune-calls 🔴 P0
**カテゴリ**: Phase 0修正（Runeブロック統合の完成）  
**位置付け**: `areka-P0-script-engine`で部分実装だったRuneブロック機能の完全実装  
**フェーズ**: 要件定義済み  
**要件**: 定義済み  
**概要**: DSL内Runeブロックで定義されたローカル関数を、DSL会話行・Call行から呼び出す機能の完全実装。

**実装要件**:
- ✅ `＠関数`、`＠関数（引数）`で呼び出し可能
- ✅ Call行から呼び出し可能
- ✅ 第一引数にキャラクター構造体（`ctx`）が渡される

**修正内容**:
- ユニットテスト実装
- スコープ解決（ローカル vs グローバル）の明確化
- ドキュメント更新（GRAMMAR.md）

**関連**: `GRAMMAR.md` Runeコードブロックセクション、`pasta-call-resolution-priority`  
**Phase 0完了条件**: ✅ 必須（P0）

---

### 7. pasta-word-definition-dsl 🟡 P1
**カテゴリ**: Phase 0・1修正（コア機能の実装）  
**位置付け**: `areka-P0-script-engine`で未実装だった単語ジャンプテーブル機能の追加  
**フェーズ**: design-generated（設計承認待ち）  
**要件**: 定義済み  
**設計**: 生成済み（未承認）  
**概要**: 単語定義DSLの実装。単語keyによる前方一致ランダム呼び出し機構。

**実装内容**:
- ラベルと同様に単語keyで前方一致検索
- シャッフルされた候補からランダム選択
- 会話のバリエーション強化（product.mdのコア目標）

**課題**:
- 設計レビュー待ち（`/kiro-validate-design pasta-word-definition-dsl`推奨）
- 承認後のタスク分解・実装

**関連**: `product.md` 前方一致単語呼び出し目標  
**Phase 0完了条件**: ✅ 必須（P1）

---

### 8. pasta-yield-propagation 🔴 P0 最優先
**カテゴリ**: Phase 0修正（最重大欠陥の修正）  
**位置付け**: `areka-P0-script-engine`のGenerator機構設計ミスによるCall/Jump文完全動作不全の修正  
**フェーズ**: 要件定義済み  
**要件**: 定義済み  
**概要**: **最重要修正仕様**。Call/Jump文で呼び出されたラベル関数のyieldイベント伝搬機構の実装。

**根本的問題**:
`areka-P0-script-engine`のトランスパイル設計が、RuneのGenerator機構を正しく理解していなかった。ネストしたGenerator関数のyieldを透過的に伝搬する機構が全く実装されていない。

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

**修正方針**:
1. 呼び出し先関数をGenerator化
2. 手動yield伝搬: `for event in callee_gen { yield event; }`パターンの実装
3. Transpilerで正しいyield伝搬コードを生成

**修正内容**:
- Transpiler修正（Call/Jump文の両方）
- Generator呼び出しコードの生成
- yield伝搬ループの挿入
- 既存テストの更新

**影響度**: 🔴 CRITICAL - Call/Jump文が一切動作しない

**関連**: `pasta-jump-function-calls`, `pasta-declarative-control-flow`（過去の不完全実装）  
**Phase 0完了条件**: ✅ 必須（P0・最優先）

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

## 優先順位マトリクス（Phase 0・1修正仕様）

### Phase 0完了条件（P0・P1: 4仕様）

#### 🔴 P0（最優先・根本的欠陥修正）
1. **pasta-yield-propagation** - Call/Jump文完全動作不全の修正
2. **pasta-local-rune-calls** - Runeブロック統合の完成

#### 🟡 P1（高優先度・コア機能実装）
3. **pasta-word-definition-dsl** - 単語ジャンプテーブル（コア機能欠落の補完）
4. **pasta-call-resolution-priority** - スコープ解決の明確化

---

### Phase 1以降（P2・P3: 5仕様）

#### 🟢 P2（中優先度・機能拡張）
5. **pasta-label-continuation** - ラベル連鎖機構
6. **pasta-jump-function-calls** - Jump文拡張

#### ⚪ P3（低優先度・高度機能）
7. **pasta-conversation-inline-multi-stage-resolution** - 多段解決
8. **pasta-dialogue-continuation-syntax** - 継続構文

#### 📋 メタ仕様
9. **ukagaka-desktop-mascot** - 32子仕様管理（Phase 0完了後）

---

## ⚠️ 現状認識: Phase 0（未着手仕様9件による根本修正中）

### プロジェクト構造の理解

- **初期実装**: `areka-P0-script-engine`（設計欠陥あり）
- **完了11件**: 小規模改修パッチ（根本解決せず）
- **未着手9件**: **Phase 0・1修正仕様**（これらが真の修正）

### 推奨アクション

**重要**: 小規模改修では解決できない。**未着手仕様9件を順次実装**することが根本解決。

### Phase 0完了への道筋（P0・P1: 4仕様実装）

#### 1. pasta-yield-propagation（🔴 最優先）
- `/kiro-spec-status pasta-yield-propagation` で状態確認
- 要件定義済み → `/kiro-spec-design pasta-yield-propagation` で設計生成
- 設計承認 → `/kiro-spec-tasks pasta-yield-propagation` でタスク分解
- `/kiro-spec-impl pasta-yield-propagation` で実装
- `/kiro-validate-impl pasta-yield-propagation` で検証

#### 2. pasta-local-rune-calls（🔴 P0）
- 要件定義済み → 同様のプロセスで実装

#### 3. pasta-word-definition-dsl（🟡 P1）
- 設計生成済み → `/kiro-validate-design pasta-word-definition-dsl` でレビュー
- 承認後、タスク分解・実装

#### 4. pasta-call-resolution-priority（🟡 P1）
- 未着手 → 要件定義から開始

### Phase 0完了条件

P0・P1の4仕様が全て完了し、以下を満たすこと：

- ✅ Call/Jump文の完全動作（yield伝搬含む）
- ✅ Runeブロック統合の完成
- ✅ 単語ジャンプテーブルの実装
- ✅ スコープ解決の明確化
- ✅ 全既存テストのパス
- ✅ 各仕様の人間承認完了

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

### Phase 0（現在: P0・P1の4仕様実装）

1. 🔴 **pasta-yield-propagation** - 設計 → タスク分解 → 実装 → 検証
2. 🔴 **pasta-local-rune-calls** - 同上
3. 🟡 **pasta-word-definition-dsl** - 設計レビュー → タスク分解 → 実装 → 検証
4. 🟡 **pasta-call-resolution-priority** - 要件定義 → 設計 → 実装 → 検証

### Phase 1以降（Phase 0完了後: P2・P3の5仕様）

5. 🟢 **pasta-label-continuation** - Phase 0完了後に着手
6. 🟢 **pasta-jump-function-calls** - Phase 0完了後に着手
7. ⚪ **pasta-conversation-inline-multi-stage-resolution** - Phase 1以降
8. ⚪ **pasta-dialogue-continuation-syntax** - Phase 1以降
9. 📋 **ukagaka-desktop-mascot** - Phase 0完了後に再開

### 現在の最優先タスク

1. `/kiro-spec-status pasta-yield-propagation` で状態確認
2. `/kiro-spec-design pasta-yield-propagation` で設計生成
3. 人間による設計承認
4. `/kiro-spec-tasks pasta-yield-propagation` でタスク分解
5. `/kiro-spec-impl pasta-yield-propagation` で実装

各仕様の状態は `/kiro-spec-status {feature-name}` で随時確認してください。
