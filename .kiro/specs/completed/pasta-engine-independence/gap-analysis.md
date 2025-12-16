# Gap Analysis: pasta-engine-independence

## 分析サマリー

- **現状**: グローバル`static PARSE_CACHE`と`Arc`による共有が存在し、要件に違反
- **主要ギャップ**: エンジンインスタンスの完全所有、純粋関数的パース/トランスパイル、テスト不在
- **実装アプローチ**: 既存アーキテクチャの構造的リファクタリング（Option B: 新規設計 + 段階移行）
- **複雑度**: M（3-7日） / リスク: Medium（既存パターンの大幅変更、後方互換性維持）
- **推奨**: 段階的移行でテストカバレッジを先に整備し、リグレッション防止

---

## 1. 現状調査（Current State Investigation）

### 1.1 既存アセット

**主要ファイル・モジュール**:
```
crates/pasta/src/
├── engine.rs              # PastaEngine本体（グローバルキャッシュ使用）
├── cache.rs               # ParseCache実装（Arc + RwLock）
├── parser/mod.rs          # parse_str() - 純粋関数
├── transpiler/mod.rs      # Transpiler::transpile() - 純粋関数
└── runtime/
    ├── labels.rs          # LabelTable（Box<dyn RandomSelector>）
    └── variables.rs       # VariableManager（独立）
```

**アーキテクチャパターン**:
- レイヤードアーキテクチャ: Parser → Transpiler → Rune Compiler → Runtime
- パース・トランスパイルは既に純粋関数的（副作用なし）
- グローバルキャッシュのみが共有状態

**テスト配置**:
- `crates/pasta/tests/` - 統合テスト（engine_integration_test.rs等）
- 現状：複数インスタンス独立性テストは**存在しない**

### 1.2 問題箇所の特定

#### 要件違反コード

**engine.rs (L17, L23-27)**:
```rust
use std::sync::{Arc, OnceLock};

static PARSE_CACHE: OnceLock<ParseCache> = OnceLock::new();

fn global_cache() -> &'static ParseCache {
    PARSE_CACHE.get_or_init(|| ParseCache::new())
}
```
❌ **違反**: `static` + `OnceLock`でグローバル共有キャッシュ  
❌ **違反**: 定数でない`static`変数の使用

**engine.rs (L61-67)**:
```rust
pub struct PastaEngine {
    unit: Arc<rune::Unit>,
    runtime: Arc<rune::runtime::RuntimeContext>,
    label_table: LabelTable,
}
```
❌ **違反**: `Arc`で共有（要件: 完全所有）  
✅ **適合**: `label_table`は所有

**cache.rs (L11-16, L23)**:
```rust
struct CacheEntry {
    ast: Arc<PastaFile>,
    rune_source: Arc<String>,
}

pub struct ParseCache {
    entries: Arc<RwLock<HashMap<u64, CacheEntry>>>,
}
```
❌ **違反**: 内部で`Arc`による共有  
⚠️ **注意**: グローバル使用前提の設計

#### 適合コード

✅ **parser/mod.rs**: `parse_str()`は純粋関数（副作用なし）  
✅ **transpiler/mod.rs**: `Transpiler::transpile()`は純粋関数  
✅ **runtime/labels.rs**: `LabelTable`は所有（`Box<dyn RandomSelector>`）  
✅ **runtime/variables.rs**: `VariableManager`は独立

### 1.3 統合インターフェース

**PastaEngine API**:
- `new(script: &str)` - コンストラクタ（パース・コンパイル実行）
- `execute_label(label_name: &str)` - ラベル実行
- `execute_label_with_filters()` - 属性フィルタ付き実行
- `on_event()` - イベントハンドリング

**依存関係**:
- Rune VM (`rune` crate) - 外部依存、内部でスレッドセーフ
- Parser/Transpiler - 純粋関数、既に依存なし

---

## 2. 要件実現性分析（Requirements Feasibility Analysis）

### 2.1 技術的ニーズと実現可能性

| 要件 | 技術的ニーズ | 現状 | ギャップ |
|------|------------|------|---------|
| Req 1.1 | 全データ所有（Arc排除） | `Arc<Unit>`, `Arc<RuntimeContext>` | **Gap**: `Arc`を直接所有に変更 |
| Req 1.5 | `static`変数ゼロ | `static PARSE_CACHE` | **Gap**: グローバルキャッシュ削除 |
| Req 2.1 | インスタンス内キャッシュ | グローバルキャッシュ | **Gap**: フィールドとして再設計 |
| Req 2.4 | 純粋関数的実装 | パース/トランスパイルは純粋 | ✅ **適合**: 既に純粋関数 |
| Req 3.4 | `Send`トレイト実装 | 未確認 | **Research Needed**: Rune型のSend実装確認 |
| Req 4-6 | テストスイート | 存在しない | **Gap**: テスト新規作成 |

### 2.2 主要ギャップ

#### Gap 1: グローバルキャッシュのインスタンス化
**現状**: `static PARSE_CACHE: OnceLock<ParseCache>` - 全エンジンで共有  
**必要**: `PastaEngine`のフィールドとして`cache: ParseCache`を保持  
**変更**: 
- グローバル変数削除
- `PastaEngine`構造体に`cache`フィールド追加
- `new()`でキャッシュ初期化
**制約**: なし（キャッシュ機構は維持、共有を止めるだけ）  
**複雑度**: 低（構造変更のみ、ロジック変更なし）

#### Gap 2: Arc排除とデータ所有
**現状**: `unit: Arc<rune::Unit>`, `runtime: Arc<rune::runtime::RuntimeContext>`  
**必要**: `unit: rune::Unit`, `runtime: rune::runtime::RuntimeContext`（所有）  
**制約**: Rune型がClone可能か、所有コストは？  
**Research Needed**: `rune::Unit`と`rune::runtime::RuntimeContext`のClone/所有可能性

#### Gap 3: インスタンス独立性テスト
**現状**: テストなし  
**必要**: 7つのテストケースカテゴリ（Req 4-6）  
**複雑度**: 中（新規テストファイル作成）

#### Gap 4: 並行実行テスト
**現状**: テストなし  
**必要**: マルチスレッドテスト、Miri対応  
**複雑度**: 中（スレッド制御とアサーション）

### 2.3 制約と未確認事項

**制約**:
- Runeライブラリの内部実装に依存（Arc使用の必要性）
- パフォーマンス: グローバルキャッシュ削除による再パース増加
- 後方互換性: 既存テストの動作保証

**Research Needed**:
1. ~~`rune::Unit`は`Clone`可能か？所有コストは?~~ → **調査完了**
2. ~~`rune::runtime::RuntimeContext`は`Clone`可能か?~~ → **調査完了**
3. ~~RuneのVMはスレッドセーフか（Send実装）？~~ → **調査完了**

**重要な設計方針**:
- ❌ キャッシュを削除するわけではない
- ✅ **キャッシュをエンジンインスタンスのフィールドとして保持**
- グローバル`static PARSE_CACHE` → `PastaEngine`の`cache: ParseCache`フィールドに移動

### 2.4 Rune依存性調査結果

#### 結論: Arc使用は**Rune APIの必須要件**

Rune 0.14のドキュメント調査により、以下が判明：

**公式ドキュメントの説明**:
> "Compiling a Unit and a RuntimeContext are expensive operations compared to the cost of calling a function."  
> "Once you have a Unit and a RuntimeContext they are thread safe and can be used by multiple threads simultaneously through Arc<Unit> and Arc<RuntimeContext>. **Constructing a Vm with these through Vm::new is a very cheap operation.**"

📖 **参照**: [Multithreading - The Rune Programming Language](https://rune-rs.github.io/book/multithreading.html)

**設計理念**:
1. **コンパイルは高コスト** → Unit/RuntimeContextを再利用
2. **VM作成は低コスト** → スレッドごとにVm::new()で新規作成
3. **Arcでスレッド間共有** → 複数VMが同じUnit/Contextを参照

**Vm::new()のシグネチャ（必須）**:
```rust
pub const fn new(
    context: Arc<RuntimeContext>,
    unit: Arc<Unit>
) -> Self
```
- `Arc<Unit>`と`Arc<RuntimeContext>`は**API仕様**
- コンパイラはこのシグネチャ以外受け付けない

**型特性の分析**:

| 型 | TryClone | Send | Sync | Arc必須？ | 備考 |
|----|----------|------|------|---------|------|
| `Unit<S>` | ✅ | 条件付き¹ | 条件付き¹ | **Yes** (API) | ¹ S: Send/Sync時のみ |
| `RuntimeContext` | ✅ | ✅ | ✅ | **Yes** (API) | 無条件実装 |
| `Vm` | ✅ | ❌ | ❌ | N/A | 所有権移動は可 |
| `Arc<Unit>` | - | ✅² | ✅² | - | ² Arcが保証 |
| `Arc<RuntimeContext>` | - | ✅ | ✅ | - | 無条件 |

**重要な発見**:
1. `Unit<S>`のSend/Syncは**内部ストレージ`S`に依存**（条件付き自動実装）
2. `RuntimeContext`はSend/Sync**無条件実装**
3. **Arc包装によりスレッド安全性が保証される**（`Arc<T>: Send + Sync if T: Send + Sync`）
4. `Vm::new()`は`Arc`包装を要求 → 直接所有は不可
5. `Vm`自体は`!Send` / `!Sync` → スレッド間共有不可（所有権移動は可）
6. ドキュメント記載: "Multithreaded execution" と "Memory safe through reference counting"

**Arc排除の技術的不可能性**:
- Runeライブラリの設計思想: 参照カウント前提
- VMは軽量（"`Vm::new()` is cheap constant-time"）
- 各スレッドで独立VM作成 → `Arc`共有はライブラリ内部のみ

**Arc使用の技術的理由**:

1. **Rune APIの設計**:
   ```rust
   // Vm::newのシグネチャ（変更不可）
   pub const fn new(
       context: Arc<RuntimeContext>,
       unit: Arc<Unit>
   ) -> Self
   ```
   - **API自体がArcを要求** → 回避不可
   - Vm内部で`Arc::clone()`して参照を保持する設計

2. **現在のPastaEngine実装**:
   ```rust
   pub struct PastaEngine {
       unit: Arc<rune::Unit>,
       runtime: Arc<rune::runtime::RuntimeContext>,
       // ...
   }
   
   // execute_label()内で毎回
   let mut vm = Vm::new(self.runtime.clone(), self.unit.clone());
   ```
   - `Arc::clone()`は参照カウント+1のみ（軽量）
   - **シングルスレッドでもAPI制約でArc必須**

3. **Arcが必要な理由（推測）**:
   - Vm内部で`Unit`/`RuntimeContext`への参照を保持
   - Vmのライフタイムと独立して`Unit`/`Context`が存在
   - 複数VM間で同じコンパイル結果を共有する設計前提
   
4. **マルチスレッド対応（副次的）**:
   - `Arc`により自動的にスレッド安全
   - PastaEngineでは使用しないが、Rune側の設計思想

**結論**:
- ✅ 各`PastaEngine`は`Arc<Unit>`と`Arc<RuntimeContext>`を**所有**
- ✅ エンジンインスタンス間で`Arc`を**共有しない**設計は可能
- ❌ `Arc`ラッパー自体の排除は**不可能**（**Rune API制約**）
- ⚠️ **シングルスレッドでもArc必須** → Vm::newのシグネチャ要求

**PastaEngineへの影響**:
- 現在の実装: `execute_label()`で毎回`Vm::new(self.runtime.clone(), self.unit.clone())`
- `clone()`コストは軽量（参照カウント+1のみ）
- **問題はArc使用ではなく、グローバルキャッシュの共有**

**実装方針**:
```rust
pub struct PastaEngine {
    unit: Arc<rune::Unit>,
    runtime: Arc<rune::runtime::RuntimeContext>,
    label_table: LabelTable,
    cache: ParseCache,  // ← グローバルから移動、エンジンごとに独立
}
```

→ **要件1.1の正しい解釈**: "Arc禁止" → "Arcエンジン間共有禁止、各エンジンが独立所有・Rune API制約によるArc使用は許容"

---

## 3. 実装アプローチオプション

### Option A: 既存コンポーネント拡張
**適用不可** - 設計原則の根本的変更が必要

**理由**:
- グローバルキャッシュ削除は既存設計の前提を覆す
- `Arc`排除は構造体定義の変更を伴う
- 「拡張」ではなく「再設計」が必要

---

### Option B: 新規設計 + 段階的移行 ⭐ **推奨**

#### 戦略
1. **Phase 1**: テスト整備（現在の実装で動作確認）
2. **Phase 2**: 新PastaEngine実装（独立性保証）
3. **Phase 3**: 既存テスト移行と検証
4. **Phase 4**: 旧実装削除

#### 実装詳細

**Phase 1: テスト基盤構築**
- 新規ファイル: `crates/pasta/tests/engine_independence_test.rs`
- 新規ファイル: `crates/pasta/tests/concurrent_execution_test.rs`
- 内容: 現在の実装で「失敗するテスト」を作成（TDD）
- 期待: グローバルキャッシュ起因の干渉を可視化

**Phase 2: 新PastaEngine実装**
- 変更: `engine.rs` - `PastaEngine`構造体の再設計
  - `unit: rune::Unit` (所有)
  - `runtime: rune::runtime::RuntimeContext` (所有)
  - `cache: Option<(PastaFile, String)>` (インスタンス内キャッシュ、1スクリプト用)
  - `label_table: LabelTable` (既存)
- 削除: `static PARSE_CACHE`, `global_cache()`
- 変更: `new()`メソッド - グローバルキャッシュ参照削除
- 変更: `cache.rs` - エクスポート削除またはローカル化

**Phase 3: 統合と検証**
- 既存テスト（`engine_integration_test.rs`等）の実行と修正
- 新規テストの成功確認
- パフォーマンステスト（ベンチマーク追加）

**Phase 4: クリーンアップ**
- `cache.rs`の完全削除またはプライベート化
- ドキュメント更新（独立性保証を明記）

#### 統合ポイント
- Rune VMインターフェース: 変更なし（内部所有のみ）
- 公開API: `PastaEngine::new()`, `execute_label()`は維持
- 後方互換性: API署名は不変、動作は独立性強化

#### トレードオフ
✅ **利点**:
- 完全な独立性保証（要件100%達成）
- テストファースト開発でリグレッション防止
- 段階的移行で安全性確保

❌ **欠点**:
- パフォーマンス低下の可能性（キャッシュ効果減少）
- 実装期間が長い（テスト込みで5-7日）
- Rune型のClone実装に依存（調査必要）

---

### Option C: ハイブリッド（インスタンスキャッシュ保持）

#### 戦略
Option Bと同様だが、インスタンス内に「最後にパースしたスクリプト」のみキャッシュ

#### 詳細
- `PastaEngine`に`last_script: Option<(String, PastaFile, String)>`追加
- `new()`で前回と同じスクリプトならキャッシュヒット
- 異なるスクリプトなら再パース

#### トレードオフ
✅ **利点**: 同一エンジンでの再利用時にパフォーマンス維持  
❌ **欠点**: 実装複雑度がわずかに増加、メモリ使用量増加

---

## 4. 実装複雑度とリスク

### 複雑度: **M (Medium, 3-7日)**

**内訳**:
- Phase 1 (テスト整備): 1-2日
- Phase 2 (新実装): 2-3日
- Phase 3 (統合・検証): 1-2日
- Phase 4 (クリーンアップ): 0.5日

**根拠**:
- 構造変更は局所的（engine.rs, cache.rs）
- パース/トランスパイルは変更不要（既に純粋関数）
- テストは新規だが既存パターン踏襲可能

### リスク: **Medium**

**リスク要因**:
1. **Rune型の所有可能性**: `Arc`が必須の可能性 → 設計見直し必要
2. **パフォーマンス低下**: キャッシュレスでベンチマーク悪化 → 最適化検討
3. **後方互換性**: 既存テストが失敗 → API変更不要で緩和

**緩和策**:
1. Research Phaseでrune crateドキュメント調査、必要ならissue報告
2. ベンチマーク作成とパフォーマンスベースライン確立
3. テストファーストで既存動作保証

---

## 5. 要件-アセットマッピング

| 要件 | 必要アセット | 現状 | ギャップ | 実装箇所 |
|------|------------|------|---------|---------|
| Req 1.1 | Arc排除の完全所有 | `Arc<Unit>`, `Arc<RuntimeContext>` | **Gap** | `engine.rs:61-67` |
| Req 1.5 | static変数ゼロ | `static PARSE_CACHE` | **Gap** | `engine.rs:23` |
| Req 2.1 | インスタンスキャッシュ | グローバルキャッシュ | **Gap** | `engine.rs` 新フィールド |
| Req 2.4 | 純粋関数 | パース/トランスパイル | ✅ **適合** | - |
| Req 3.1-3.5 | Send実装 | 未確認 | **Research** | `engine.rs` trait実装 |
| Req 4 | 独立性テスト | なし | **Missing** | `tests/engine_independence_test.rs` |
| Req 5 | 並行実行テスト | なし | **Missing** | `tests/concurrent_execution_test.rs` |
| Req 6 | グローバル状態検証 | なし | **Missing** | 静的チェックスクリプト |
| Req 7 | CI統合 | 既存CI | **Extend** | `.github/workflows/` |

---

## 6. 設計フェーズへの推奨事項

### 推奨アプローチ: **Option B（新規設計 + 段階的移行）**

**理由**:
1. 要件を100%満たす唯一のアプローチ
2. テストファーストでリスク軽減
3. 段階的移行で後方互換性維持

### 優先調査項目（Research Items）

1. **Rune型の所有可能性調査**:
   - `rune::Unit`と`rune::runtime::RuntimeContext`のClone実装確認
   - Runeドキュメント: https://docs.rs/rune/
   - 必要ならrune crateのissue確認・報告

2. **パフォーマンスベンチマーク作成**:
   - 現状のグローバルキャッシュ効果測定
   - キャッシュレス実装との比較
   - 同一スクリプトの繰り返し実行シナリオ

3. **Sendトレイト実装確認**:
   - `PastaEngine`がSend可能か検証
   - Rune VMのスレッドセーフ性確認

### 重要な設計判断

1. **キャッシュ戦略**: Option B (キャッシュレス) vs Option C (インスタンスキャッシュ)
   - ベンチマーク結果に基づいて決定
   - 初期実装はOption B（シンプル）推奨

2. **Arc排除の実現可能性**:
   - Rune型がClone可能なら直接所有
   - Clone不可なら設計再検討（要件緩和の可能性）

3. **テスト駆動開発の徹底**:
   - Phase 1で「失敗するテスト」を先に作成
   - Phase 2で実装し、テストをパスさせる

---

## 7. 次のステップ

1. **Gap Analysisレビュー**: 本ドキュメントの確認と追加調査項目の特定
2. **Research実施**: Rune型の調査とベンチマーク作成
3. **設計フェーズ**: `/kiro-spec-design pasta-engine-independence`実行
   - Option B実装の詳細設計
   - テストケース仕様
   - マイグレーション計画

---

_Generated: 2025-12-10_
