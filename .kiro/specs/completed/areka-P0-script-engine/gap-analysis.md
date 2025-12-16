# Gap Analysis Report

| 項目 | 内容 |
|------|------|
| **Document Title** | areka スクリプトエンジン ギャップ分析 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Spec ID** | areka-P0-script-engine |
| **Phase** | Requirements Draft |

---

## Executive Summary

本分析は、Pastaスクリプトエンジンの要件と既存コードベースのギャップを評価する。主要な発見事項：

### 主要発見

1. **統合ポイント確認済み**: `TypewriterToken` enumが明示的に"areka-P0-script-engine と共有"とコメントされており、IR層の設計意図が明確
2. **Runeエンジン未実装**: 要件で定義されたRuneスクリプト実行エンジンがコードベースに存在しない（**最大のギャップ**）
3. **ECSアーキテクチャ活用可能**: bevy_ecsによる既存の設計パターンを活用してスクリプトエンジンを統合可能
4. **Typewriter側準備完了**: IR受信側（Typewriter）は既に実装済み、スクリプトエンジンはIR生成に専念できる

### 推奨アプローチ

**オプションB: 新規実装 + 外部クレート統合**を推奨（実装工数: L、リスク: 中）

- Runeスクリプトエンジンを外部クレートとして統合（`rune` crate）
- Pasta DSLパーサーを新規実装
- 既存のTypewriterToken IRとの統合は最小限の修正で実現可能

---

## Requirements Coverage

### 1. 要件とコードベース資産のマッピング

| 要件ID | 要件概要 | 既存資産 | カバレッジ | ギャップ |
|--------|---------|---------|-----------|---------|
| **Req-1** | 対話記述DSLの解釈・実行 | なし | 0% | DSLパーサー全体が未実装 |
| **Req-2** | 中間表現（IR）出力 | `TypewriterToken` enum | 80% | トークン種別追加が必要（サーフェス切り替え、発言者切り替え） |
| **Req-3** | さくらスクリプト互換出力 | なし | 0% | さくらスクリプトコマンドパーサーが未実装 |
| **Req-4** | 変数管理 | なし | 0% | グローバル/ローカル変数システムが未実装 |
| **Req-5** | 制御構文 | なし | 0% | 条件分岐・ループ・演算子が未実装 |
| **Req-6** | 複数キャラクター会話制御 | なし | 0% | 発言者管理システムが未実装 |
| **Req-7** | イベントハンドリング | `Messages<T>` (bevy_ecs) | 40% | イベント名とハンドラのマッピングが未実装 |

#### 詳細マッピング

##### Req-2: 中間表現（IR）出力 - 詳細

**既存資産**: `crates/wintf/src/ecs/widget/text/typewriter_ir.rs`

```rust
/// TypewriterToken - areka-P0-script-engine と共有
pub enum TypewriterToken {
    Text(String),       // ✅ 実装済み: Req-2.2 (テキストトークン)
    Wait(f64),          // ✅ 実装済み: Req-2.3 (ウェイトトークン)
    FireEvent {         // 🔶 部分実装: Req-2.4, 2.5
        target: Entity,
        event: TypewriterEventKind,
    },
}

pub enum TypewriterEventKind {
    // ⚠️ ギャップ: サーフェス切り替え、発言者切り替えの定義が不足
    // 現状は汎用FireEventのみ
}
```

**ギャップ**:
- ❌ サーフェス切り替え専用トークン（Req-2.4）
- ❌ 発言者切り替え専用トークン（Req-2.5）

**提案**:
```rust
pub enum TypewriterToken {
    Text(String),
    Wait(f64),
    FireEvent { target: Entity, event: TypewriterEventKind },
    // 追加提案
    ChangeSurface(u32),           // サーフェスID
    ChangeSpeaker(String),        // 発言者名（キャラクターID）
}
```

##### Req-7: イベントハンドリング - 詳細

**既存資産**: `bevy_ecs::message::Messages<T>`（メッセージパッシングシステム）

**既存の使用パターン**:
```rust
// crates/wintf/src/ecs/world.rs
pub fn send_message<T: 'static + Send + Sync>(&mut self, message: T) {
    self.get_resource_mut::<Messages<T>>().unwrap().send(message);
}
```

**ギャップ**:
- イベント名（文字列）とハンドラ（Runeスクリプト関数）のマッピングシステムが未実装
- ダブルクリック等のUI操作イベントの定義が未実装

**提案**:
```rust
// 新規: イベント名とスクリプトラベルのマッピング
pub struct ScriptEventRegistry {
    handlers: HashMap<String, String>, // イベント名 → ラベル名
}

// 使用例
registry.register("OnDoubleClick", "*挨拶");
registry.register("OnMouseDown", "*クリック反応");
```

---

## Implementation Options

### オプションA: 最小実装（拡張アプローチ）

**概要**: 既存のTypewriterToken IRを最小限拡張し、シンプルなDSLパーサーのみ実装

**実装内容**:
1. Pastaパーサー実装（パーサージェネレータ選択は設計時）
2. 変数管理とランダム選択のみサポート（Rune統合なし）
3. TypewriterTokenに必要な列挙子を追加
4. bevy_ecsのメッセージングでイベントハンドリング

**メリット**:
- ✅ 実装スコープが明確（工数S-M）
- ✅ 既存コードへの影響が最小限
- ✅ 早期に動作デモが可能

**デメリット**:
- ❌ 要件Req-4（変数）、Req-5（制御構文）が制限される
- ❌ Runeスクリプト統合がないため拡張性が低い
- ❌ 複雑なロジック（演算、関数）が実装できない

**工数**: S-M（2-4週間）  
**リスク**: 低  
**推奨度**: ⭐️⭐️（MVP検証には有効だが、長期的には再実装が必要）

---

### オプションB: 新規実装 + 外部クレート統合（推奨）

**概要**: Runeスクリプトエンジン（`rune` crate）を統合し、PastaパーサーからRuneへトランスコンパイル

**実装内容**:
1. `rune` crateをCargo.tomlに追加（https://crates.io/crates/rune）
2. Pastaパーサー実装（パーサージェネレータ選択は設計時）
3. Pasta → Rune トランスコンパイラ実装
4. Rune VMとコードベースの統合レイヤー
5. TypewriterTokenへの出力インターフェース
6. イベントハンドリングシステム（bevy_ecs Messages）

**詳細設計**:

#### アーキテクチャ

```
┌─────────────────────────────────────────────────────────┐
│ Pasta DSL (./dic/**/*.pasta)                           │
│ ＊挨拶　さくら：こんにちは＠W（300）元気？              │
└──────────────────┬──────────────────────────────────────┘
                   │ parse (nom/pest)
                   ↓
┌─────────────────────────────────────────────────────────┐
│ Pasta AST (Abstract Syntax Tree)                       │
│ Label("挨拶", [Text("さくら：こんにちは"), ...])       │
└──────────────────┬──────────────────────────────────────┘
                   │ transpile
                   ↓
┌─────────────────────────────────────────────────────────┐
│ Rune Script (generated)                                │
│ pub fn 挨拶() { emit_text("こんにちは"); wait(0.3); }  │
└──────────────────┬──────────────────────────────────────┘
                   │ compile & execute
                   ↓
┌─────────────────────────────────────────────────────────┐
│ Rune VM (rune crate)                                   │
│ + IR出力関数 (emit_text, wait, fire_event)             │
└──────────────────┬──────────────────────────────────────┘
                   │ accumulate IR
                   ↓
┌─────────────────────────────────────────────────────────┐
│ Vec<TypewriterToken>                                   │
│ [Text("こんにちは"), Wait(0.3), Text("元気？")]        │
└──────────────────┬──────────────────────────────────────┘
                   │ send to ECS
                   ↓
┌─────────────────────────────────────────────────────────┐
│ Typewriter System (wintf-P0-typewriter)                │
│ タイピングアニメーション実行                             │
└─────────────────────────────────────────────────────────┘
```

#### Rune統合インターフェース

```rust
// Rune VMから呼び出される関数（Rustで実装）
#[rune::function]
fn emit_text(text: String) {
    IR_BUFFER.lock().unwrap().push(TypewriterToken::Text(text));
}

#[rune::function]
fn wait(seconds: f64) {
    IR_BUFFER.lock().unwrap().push(TypewriterToken::Wait(seconds));
}

#[rune::function]
fn fire_event(target: String, event: String) {
    // Entity解決とイベント発火
}
```

#### Pastaトランスコンパイル例

**入力（Pasta DSL）**:
```
＊挨拶
　　＠時間帯：朝
　さくら：おはよう！＠W（500）元気？
```

**出力（Rune Script）**:
```rune
pub fn 挨拶_1() {
    emit_text("さくら：おはよう！");
    wait(0.5);
    emit_text("元気？");
}

// 属性メタデータ
pub fn 挨拶_1_attrs() {
    {"時間帯": "朝"}
}
```

**メリット**:
- ✅ 要件を完全にカバー（Req-1～Req-7すべて）
- ✅ Runeエコシステムの恩恵（標準ライブラリ、デバッガ等）
- ✅ 拡張性が高い（MCP連携、複雑なロジック対応）
- ✅ トランスコンパイラは一度実装すれば安定

**デメリット**:
- ⚠️ 実装工数が大きい（L）
- ⚠️ Runeクレートの学習コスト
- ⚠️ トランスコンパイラのデバッグが複雑

**工数**: L（6-10週間）  
**リスク**: 中  
**推奨度**: ⭐️⭐️⭐️⭐️⭐️（要件を完全に満たし、長期的な拡張性を確保）

---

### オプションC: ハイブリッド実装

**概要**: オプションAで基本機能を実装し、Phase 2でRuneを段階的に統合

**実装内容**:
- Phase 1: オプションAの最小実装（変数・ランダム選択のみ）
- Phase 2: Rune統合（ローカル関数、複雑なロジック）

**メリット**:
- ✅ 段階的リリースが可能（MVP → 完全版）
- ✅ 初期リスクが低い

**デメリット**:
- ❌ 二度実装のコスト（Phase 1の設計がPhase 2で大きく変わる）
- ❌ トランスコンパイラの設計が複雑化

**工数**: S-M（Phase 1） + M-L（Phase 2） = L-XL  
**リスク**: 中  
**推奨度**: ⭐️⭐️⭐️（段階的リリースが重要な場合のみ）

---

## Effort & Risk Assessment

| オプション | 工数 | リスク | 実現可能性 | 要件カバレッジ |
|-----------|------|--------|-----------|---------------|
| **A: 最小実装** | S-M | 低 | 高 | 60%（基本機能のみ） |
| **B: 新規実装 + Rune統合（推奨）** | L | 中 | 高 | 100%（完全対応） |
| **C: ハイブリッド** | L-XL | 中 | 中 | 段階的に100% |

### 工数詳細（オプションB）

| タスク | 工数 | 備考 |
|--------|------|------|
| Rune統合準備（crate追加、学習） | S | 1週間 |
| Pastaパーサー実装（パーサー選択と実装） | M | 3-4週間（複雑な構文のため） |
| Pasta → Rune トランスコンパイラ | M | 3-4週間 |
| Rune VM統合レイヤー（IR出力関数） | S | 1-2週間 |
| TypewriterToken拡張（新トークン追加） | XS | 2-3日 |
| イベントハンドリングシステム | S | 1週間 |
| テストとデバッグ | M | 2-3週間 |
| **合計** | **L** | **6-10週間** |

### リスク評価

| リスク要因 | 影響度 | 発生確率 | 緩和策 |
|-----------|--------|---------|--------|
| Runeクレートの学習曲線 | 中 | 高 | 事前にRune公式ドキュメントと小規模PoC実装 |
| Pastaパーサーの複雑性 | 高 | 中 | 段階的実装（基本構文 → 高度な機能） |
| トランスコンパイラのバグ | 高 | 中 | 包括的なテストスイート構築 |
| IR層の拡張互換性 | 低 | 低 | TypewriterToken既に拡張可能な設計 |
| bevy_ecsとの統合 | 低 | 低 | 既存のメッセージングパターンを活用 |

---

## Dependencies & Integration Points

### 既存コンポーネントとの依存関係

```
┌─────────────────────────────────────────────┐
│ areka-P0-script-engine (新規実装)           │
│                                             │
│ ┌─────────────┐    ┌──────────────┐        │
│ │ Pasta Parser│ →  │ Rune VM      │        │
│ └─────────────┘    └──────┬───────┘        │
│                            │                │
│                            ↓ IR生成         │
│                   Vec<TypewriterToken>      │
└────────────────────────────┬────────────────┘
                             │
                             ↓ 送信（bevy_ecs Messages）
┌────────────────────────────┴────────────────┐
│ wintf-P0-typewriter (既存)                  │
│                                             │
│ fn process_tokens(&mut self,                │
│     tokens: Vec<TypewriterToken>) {         │
│     // タイピングアニメーション実行          │
│ }                                           │
└─────────────────────────────────────────────┘
```

### 統合ポイント詳細

#### 1. TypewriterToken IR（既存インターフェース）

**ファイル**: `crates/wintf/src/ecs/widget/text/typewriter_ir.rs`

**現状**:
```rust
pub enum TypewriterToken {
    Text(String),
    Wait(f64),
    FireEvent { target: Entity, event: TypewriterEventKind },
}
```

**必要な拡張**:
```rust
pub enum TypewriterToken {
    Text(String),
    Wait(f64),
    FireEvent { target: Entity, event: TypewriterEventKind },
    // 追加
    ChangeSurface { character_id: String, surface_id: u32 },
    ChangeSpeaker(String),
}
```

**影響範囲**: 
- `typewriter_systems.rs`: トークン処理ロジックに新ケース追加
- `typewriter.rs`: コンポーネント状態管理の拡張
- 工数: XS（2-3日）

#### 2. bevy_ecs Messages（イベントシステム）

**既存の使用パターン**:
```rust
// crates/wintf/src/ecs/world.rs
world.send_message(ScriptExecutionRequest {
    label: "＊挨拶".to_string(),
    args: vec![],
});
```

**提案する新規メッセージ型**:
```rust
// 新規: スクリプト実行要求
pub struct ScriptExecutionRequest {
    pub label: String,
    pub args: Vec<RuneValue>,
    pub filters: HashMap<String, String>, // 属性フィルタ
}

// 新規: スクリプト実行完了通知
pub struct ScriptExecutionComplete {
    pub label: String,
    pub ir_tokens: Vec<TypewriterToken>,
}

// 新規: イベントハンドラ登録
pub struct RegisterScriptEvent {
    pub event_name: String,
    pub handler_label: String,
}
```

**統合方法**:
1. スクリプトエンジンはbevy_ecsの`System`として登録
2. `ScriptExecutionRequest`メッセージを購読
3. Rune VM実行 → IR生成 → `ScriptExecutionComplete`送信
4. TypewriterシステムがIRを受信して実行

#### 3. 新規クレート依存

**追加するCargo依存関係**:

```toml
# crates/wintf/Cargo.toml に追加
[dependencies]
rune = "0.15"           # Runeスクリプトエンジン
# パーサー実装は設計フェーズで選択
# 候補: nom (コンビネータ), pest (PEG), 手書き再帰下降, etc.
```

**パーサー選択は設計時判断**:
- 要件段階では実装の詳細を規定しない
- 設計フェーズで以下を考慮して選択:
  - パフォーマンス特性
  - エラーメッセージ品質
  - 保守性とデバッグ容易性
  - チームの習熟度
  - DSL構文の複雑さ

---

## Open Questions

### 設計上の未解決事項

**⚠️ 本セクションの議題は`design-decisions.md`で詳細に展開されました。**

各議題の詳細な設計判断、実装計画、コード例については以下を参照：

| 議題 | 決定内容 | 詳細リンク |
|------|---------|-----------|
| 1. ファイルローディング戦略 | ✅ ハイブリッド（起動時全ロード + ホットリロード） | [design-decisions.md#議題1](./design-decisions.md#議題1-ファイルローディング戦略) |
| 2. Rune VMライフサイクル | ✅ bevy_ecs Resource | [design-decisions.md#議題2](./design-decisions.md#議題2-rune-vmのライフサイクル管理) |
| 3. 変数の永続化 | ✅ メモリストレージ + JSON永続化 | [design-decisions.md#議題3](./design-decisions.md#議題3-変数の永続化) |
| 4. エラーハンドリング方針 | ✅ 多層（ログ + メッセージ + UI） | [design-decisions.md#議題4](./design-decisions.md#議題4-エラーハンドリング方針) |
| 5. さくらスクリプトエスケープ | ✅ Typewriter層での解釈 | [design-decisions.md#議題5](./design-decisions.md#議題5-さくらスクリプトのエスケープ処理) |
| 6. ランダムシード管理 | ✅ ハイブリッド（時刻 + 固定 + セーブ） | [design-decisions.md#議題6](./design-decisions.md#議題6-ランダム選択のシード管理) |

### サマリー

すべての設計判断が完了し、以下の方針が確定しました：

#### アーキテクチャ方針
- **クレート分離**: サブクレート`pasta`として独立実装
- **ECS統合**: bevy_ecs Resourceとして管理
- **責務分離**: Pasta層（会話フロー）/ Typewriter層（表示制御）

#### 実装優先度
1. **Phase 1 (P0/MVP)**: 起動時全ロード、bevy_ecs統合、メモリストレージ、基本エラーログ、Typewriter層エスケープ、システム時刻ランダム
2. **Phase 2 (P1)**: ホットリロード、JSON永続化、エラーメッセージ通知、固定シードデバッグ
3. **Phase 3 (P2)**: 開発者UI、RNG状態永続化

次のステップ: `/kiro-spec-design areka-P0-script-engine`で技術設計文書を作成

---

## Recommendations for Design Phase

### 優先度付きアクション項目

#### 高優先度（Phase 1: 設計フェーズ）

1. **✅ オプションB（新規実装 + Rune統合）を採用**
   - 理由: 要件を完全にカバーし、長期的な拡張性を確保
   - 次のステップ: Runeクレートの詳細調査とPoC実装

2. **✅ TypewriterToken IRの拡張設計**
   - 新トークン: `ChangeSurface`, `ChangeSpeaker`
   - 設計文書に詳細なトークン仕様を記載

3. **✅ Pastaパーサーの文法定義**
   - パーサー実装手法の選択（設計時判断）
   - 段階的実装計画（基本構文 → 高度な機能）

4. **✅ Rune統合アーキテクチャ設計**
   - IR出力関数（`emit_text`, `wait`, `fire_event`）の仕様
   - Rune VMのライフサイクル管理
   - 変数ストレージ設計

5. **✅ イベントハンドリングシステム設計**
   - `ScriptEventRegistry`の詳細仕様
   - UI操作イベント（クリック、ダブルクリック等）の定義

#### 中優先度（Phase 2: 実装フェーズ前半）

6. **🔶 エラーハンドリング戦略の決定**
   - ログ出力形式、エラーメッセージ設計
   - 開発者向けデバッグ情報の仕様

7. **🔶 ファイルローディング戦略の決定**
   - 起動時全ロード vs 遅延ロード
   - ホットリロード機能の設計（開発時のみ）

8. **🔶 変数永続化設計**
   - セーブデータフォーマット（JSON/TOML/独自形式）
   - areka-P0-package-managerとの連携

#### 低優先度（Phase 2: 実装フェーズ後半）

9. **⬜ さくらスクリプトエスケープの責務分担**
   - Typewriter層での解釈を推奨
   - 既存SHIORI.DLL形式との互換性検証

10. **⬜ ランダム選択シード管理の詳細仕様**
    - デバッグモード、再現性確保の仕組み

---

## Technical Debt & Future Considerations

### 既存コードベースの技術的負債

1. **TypewriterTokenの拡張性**
   - 現状: 3種類のトークン（Text, Wait, FireEvent）
   - 将来: さらに多くのトークン種別が必要になる可能性
   - 推奨: トークンのカテゴリ化、プラグイン的な拡張機構を検討

2. **IRのバージョニング**
   - 現状: バージョン情報なし
   - 将来: IR形式の変更時に互換性問題が発生する可能性
   - 推奨: IRにバージョンフィールドを追加、マイグレーション機構を設計

### 将来の拡張性考慮事項

1. **MCP連携（areka-P2-llm-integration）**
   - Runeスクリプトから外部MCPコマンドを呼び出す仕組み
   - `＠mcp_call（"command"　"args"）`のような構文

2. **デバッグツール（areka-P1-devtools）**
   - Runeデバッガ統合（ブレークポイント、変数ウォッチ）
   - Pastaスクリプトのプロファイリング（実行時間、メモリ使用量）

3. **SHIORI.DLL互換層**
   - 既存のSHIORI.DLLゴーストとの互換性
   - さくらスクリプト完全サポート

---

## Appendix A: 既存コードベース調査結果

### 調査対象ファイル

| ファイルパス | 関連性 | 調査結果 |
|-------------|--------|---------|
| `crates/wintf/src/ecs/widget/text/typewriter_ir.rs` | ⭐️⭐️⭐️⭐️⭐️ | **統合ポイント**: TypewriterToken定義、明示的に"areka-P0-script-engine と共有"とコメント |
| `crates/wintf/src/ecs/widget/text/typewriter_systems.rs` | ⭐️⭐️⭐️⭐️ | TypewriterToken処理ロジック、トークン追加時に修正が必要 |
| `crates/wintf/src/ecs/widget/text/typewriter.rs` | ⭐️⭐️⭐️⭐️ | Typewriterコンポーネント、IR受信側の実装 |
| `crates/wintf/src/ecs/world.rs` | ⭐️⭐️⭐️ | bevy_ecsメッセージング、イベントハンドリングの参考実装 |
| `crates/wintf/Cargo.toml` | ⭐️⭐️⭐️ | 既存依存関係、新規クレート追加の参考 |

### TypewriterToken使用箇所（20箇所）

```rust
// 主要な使用箇所
crates/wintf/src/ecs/widget/text/typewriter_ir.rs:15    // 定義
crates/wintf/src/ecs/widget/text/typewriter_systems.rs:42  // 処理ロジック
crates/wintf/src/ecs/widget/text/typewriter.rs:87       // コンポーネント統合
// ... 他17箇所
```

**影響範囲の評価**: TypewriterToken拡張時の影響は限定的（XS工数）

---

## Appendix B: Rune統合の技術的詳細

### Runeクレートの基本情報

| 項目 | 内容 |
|------|------|
| **クレート名** | `rune` |
| **最新バージョン** | 0.15.x |
| **ライセンス** | MIT / Apache-2.0 |
| **公式サイト** | https://rune-rs.github.io/ |
| **GitHub** | https://github.com/rune-rs/rune |

### Runeの特徴

1. **動的型付け**: スクリプトコンパイル時に型チェック不要
2. **async/await対応**: 非同期処理をネイティブサポート
3. **Rustとの相互運用**: Rust関数をRuneから呼び出し可能
4. **ホットリロード**: スクリプト再コンパイルが高速

### 統合サンプルコード

```rust
use rune::{Context, Vm, Unit};
use std::sync::Arc;

// Rust側で提供する関数
#[rune::function]
fn emit_text(text: String) {
    println!("Emit: {}", text);
}

fn main() -> rune::Result<()> {
    // コンテキスト作成（利用可能な関数を登録）
    let mut context = Context::with_default_modules()?;
    context.install(&rune::modules::default_module()?)?;
    context
        .function_meta(emit_text)?
        .build()?;

    // スクリプトのコンパイル
    let mut sources = rune::sources! {
        entry => {
            pub fn main() {
                emit_text("Hello from Rune!");
            }
        }
    };

    let unit = rune::prepare(&mut sources)
        .with_context(&context)
        .build()?;

    // VM実行
    let vm = Vm::new(Arc::new(context.runtime()?), Arc::new(unit));
    let output = vm.execute(["main"], ())?;

    Ok(())
}
```

### Pastaトランスコンパイル実装のヒント

```rust
// Pasta ASTノード
enum PastaNode {
    Label(String, Vec<PastaNode>),
    Text(String, String), // (speaker, text)
    Wait(f64),
    Call(String),
    Jump(String),
}

// Rune生成例
fn transpile_to_rune(nodes: Vec<PastaNode>) -> String {
    let mut rune_code = String::new();
    
    for node in nodes {
        match node {
            PastaNode::Label(name, body) => {
                rune_code.push_str(&format!("pub fn {}() {{\n", name));
                for child in body {
                    rune_code.push_str(&transpile_node(child));
                }
                rune_code.push_str("}\n");
            }
            _ => {}
        }
    }
    
    rune_code
}

fn transpile_node(node: PastaNode) -> String {
    match node {
        PastaNode::Text(speaker, text) => {
            format!("    emit_text(\"{}: {}\");\n", speaker, text)
        }
        PastaNode::Wait(seconds) => {
            format!("    wait({});\n", seconds)
        }
        _ => String::new(),
    }
}
```

---

## Conclusion

本ギャップ分析により、Pastaスクリプトエンジンの実装に必要な情報を網羅的に整理した。

### サマリー

- **統合ポイント明確**: TypewriterToken IRが既に設計されており、統合は容易
- **最大ギャップ**: Runeスクリプトエンジンの統合とPasta DSLパーサーの新規実装
- **推奨アプローチ**: オプションB（新規実装 + Rune統合）
- **実装工数**: L（6-10週間）
- **リスク**: 中（Rune学習曲線とトランスコンパイラの複雑性）

### 次のフェーズへの準備

設計フェーズでは以下を明確にする必要がある：

1. ✅ Runeクレートの詳細調査とPoC実装
2. ✅ Pastaパーサーの完全な文法定義（パーサー実装手法の選択を含む）
3. ✅ Pasta → Rune トランスコンパイラの設計
4. ✅ TypewriterToken IRの拡張仕様
5. ✅ イベントハンドリングシステムの詳細設計
6. ✅ エラーハンドリング戦略
7. ✅ ファイルローディングとホットリロードの仕様

これらの項目を設計文書で詳細化し、実装フェーズへの移行準備を整える。

---

**分析者**: GitHub Copilot  
**分析日**: 2025-11-29  
**レビュー状態**: 初稿完成、レビュー待ち
