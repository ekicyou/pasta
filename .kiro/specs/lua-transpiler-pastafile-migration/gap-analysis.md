# ギャップ分析：lua-transpiler-pastafile-migration

## 分析概要

**機能**: pasta_luaトランスパイラーをPastaFile入力ベースの順序保持処理に移行し、pasta_runeのTranspiler2と同等のアーキテクチャを採用。

**主な課題**:
1. **現在のpasta_luaアーキテクチャ**: 分離された配列（actors, scenes）を入力 → FileItem出現順を無視
2. **要件との乖離**: REQ-2（FileItem出現順処理）が実装不可能
3. **API廃止の影響**: REQ-10でPastaFileヘルパーメソッド廃止 → テストケース大量修正が必要

**推奨アプローチ**: ハイブリッド手法（段階的移行）
- フェーズ1: PastaFile入力の新メソッド `transpile_file()` を追加
- フェーズ2: 既存メソッドを非推奨ラッパーに変更
- フェーズ3: ヘルパーメソッド廃止とテスト修正

---

## 1. 現在の状態分析

### 1.1 主要ファイル・モジュール構成

**pasta_lua構造**:
```
crates/pasta_lua/src/
├── lib.rs                    # クレートエントリーポイント
├── transpiler.rs             # LuaTranspiler（問題のモジュール）
├── context.rs                # TranspileContext
├── code_generator.rs         # Luaコード生成ロジック
├── config.rs                 # TranspilerConfig
└── error.rs                  # TranspileError
```

**現在のLuaTranspiler API**:
```rust
pub struct LuaTranspiler { config: TranspilerConfig }

impl LuaTranspiler {
    pub fn transpile<W: Write>(
        &self,
        actors: &[ActorScope],          // ← 分離配列
        scenes: &[GlobalSceneScope],    // ← 分離配列
        writer: &mut W,
    ) -> Result<TranspileContext, TranspileError>
    
    pub fn transpile_with_globals<W: Write>(
        &self,
        actors: &[ActorScope],
        scenes: &[GlobalSceneScope],
        global_words: &[KeyWords],      // ← ファイルレベル単語を別途指定
        writer: &mut W,
    ) -> Result<TranspileContext, TranspileError>
}
```

**参照実装（pasta_rune Transpiler2）**:
```rust
pub struct Transpiler2;

impl Transpiler2 {
    pub fn transpile_pass1<W: std::io::Write>(
        file: &PastaFile,              // ← 統一入力
        scene_registry: &mut SceneRegistry,
        word_registry: &mut WordDefRegistry,
        writer: &mut W,
    ) -> Result<(), TranspileError>
    
    pub fn transpile_pass2<W: std::io::Write>(
        scene_registry: &SceneRegistry,
        writer: &mut W,
    ) -> Result<(), TranspileError>
}
```

### 1.2 核心的な設計差異

| 項目 | pasta_lua現在 | pasta_rune Transpiler2 | 要件 |
|------|--------------|------------------------|------|
| **入力形式** | `&[ActorScope]` + `&[GlobalSceneScope]` | `&PastaFile` | REQ-1 |
| **処理順序** | 型別処理（アクター先、シーン後） | ファイル順序保持 | REQ-2 |
| **ファイルレベル属性** | 別メソッド呼び出しで指定 | FileItem::FileAttrとして順序保持 | REQ-3 |
| **グローバル単語** | 別メソッド呼び出しで指定 | FileItem::GlobalWordとして順序保持 | REQ-4 |
| **アクター処理** | 全アクターを先処理 | 出現順に個別処理 | REQ-6 |

### 1.3 テストの現状

**テスト利用状況**:
- `crates/pasta_lua/tests/transpiler_integration_test.rs` (610行)
  - 12個のテスト関数が `file.actor_scopes()` / `file.global_scene_scopes()` を使用
  - 各テストが分離配列を `transpile()` / `transpile_with_globals()` に渡す

**影響範囲** (REQ-10関連):
```
file.actor_scopes()       → 14マッチ
file.global_scene_scopes() → 24マッチ
file.file_attrs()         → 6マッチ
file.words()              → 6マッチ
```
総計: 50マッチ (6ファイル)

---

## 2. 要件の実現可能性分析

### REQ-1: PastaFile入力インターフェース
**技術的課題**: なし
- **理由**: pasta_runeで既に実装されている同等パターンをpasta_luaにコピーするだけ
- **既存資産**: TranspileContextは既にFileItem処理可能（内部に`accumulate_file_attr()`メソッドなし）
- **ギャップ**: TranspileContext内に`accumulate_file_attr()`メソッドを追加する必要がある

**実現方法**: **オプションA: 新メソッド追加** → `transpile_file(&PastaFile, ...)`

### REQ-2: FileItem出現順処理
**技術的課題**: 中程度
- **理由**: pasta_luaのTranspileContextはFileItem列挙の知識がない
- **既存資産**: `for item in &file.items { match item { ... } }` パターンはpasta_runeに存在
- **ギャップ**: pasta_luaにFileItem処理ロジックを追加する必要がある

**実現方法**: pasta_rune transpiler/mod.rs L82-105を参考に適用

```rust
for item in &file.items {
    match item {
        FileItem::FileAttr(attr) => {
            context.accumulate_file_attr(attr);
        }
        FileItem::GlobalWord(word) => {
            word_registry.register_global(...);
        }
        FileItem::GlobalSceneScope(scene) => {
            // scene処理
        }
        FileItem::ActorScope(actor) => {
            // actor処理（現在未実装）
        }
    }
}
```

### REQ-3 ～ REQ-6: ファイルレベル処理・出現順追跡
**技術的課題**: 低い
- **理由**: context が既にレジストリを保有しているため、ロジックの追加だけで実現可能
- **既存資産**: 
  - `context.register_global_scene()` 
  - `context.register_local_scene()`
  - `context.register_global_words()`
  - `context.register_local_words()`
  - SceneRegistry / WordDefRegistry は既に完全実装

**新規実装**:
- `context.accumulate_file_attr()` - FileAttr累積（pasta_runeを参考に）
- ファイル属性キャッシュをcontextに保持する仕組み

### REQ-7: API一貫性
**技術的課題**: なし
- **理由**: メソッドシグネチャをpasta_runeに合わせるだけ
- **参考**: `transpile_pass1(&PastaFile, &mut SceneRegistry, &mut WordDefRegistry, &mut W)` など

### REQ-8: 後方互換性
**技術的課題**: 低い
- **既存メソッド**: `transpile()` / `transpile_with_globals()` を非推奨ラッパーに変更
- **実装方法**:
  ```rust
  #[deprecated(since = "0.X.X", note = "use `transpile_file` instead")]
  pub fn transpile<W: Write>(
      &self,
      actors: &[ActorScope],
      scenes: &[GlobalSceneScope],
      writer: &mut W,
  ) -> Result<TranspileContext, TranspileError> {
      // 内部で transpile_file() を呼び出す
      // ただし file.items構築が必要
  }
  ```
- **制約**: 非推奨メソッド内でPastaFileを再構築する必要あり（順序情報は失われる）

### REQ-9: テストカバレッジ
**技術的課題**: 中程度
- **理由**: 新しいFileItem処理パターンのテストが必要
- **必要なテスト**:
  1. FileAttr単体テスト
  2. GlobalWord単体テスト
  3. FileItem出現順が処理順に反映されることの検証テスト
  4. ファイル属性累積が後続処理に影響することの検証
- **参考**: `tests/pasta_transpiler2_unit_test.rs` に同等テストあり

### REQ-10: PastaFileヘルパーメソッドの廃止【本仕様の核心】
**技術的課題**: 高い（スコープが大きい）
- **根本問題の解決**: ヘルパーメソッドの存在自体が出現順無視の実装を誘発
  - "あれば使ってしまう" 原則：便利なAPIは必ず使用される
  - 型別フィルタリングによる順序情報の喪失
  - 構造的に間違った実装を防ぐにはAPI廃止が唯一の解決策
- **廃止対象**:
  - `PastaFile::file_attrs()`
  - `PastaFile::words()`
  - `PastaFile::global_scene_scopes()`
  - `PastaFile::actor_scopes()`
- **コンパイルエラー発生箇所**: 70マッチ以上（9ファイル）
  - **pasta_lua**: 
    - `tests/parser2_integration_test.rs` (22マッチ)
    - `crates/pasta_lua/tests/transpiler_integration_test.rs` (28マッチ)
  - **pasta_rune**:
    - `crates/pasta_rune/tests/parser2_integration_test.rs` (14マッチ)
    - `crates/pasta_rune/tests/pasta_transpiler2_unit_test.rs` (2マッチ)
    - `crates/pasta_rune/src/transpiler/context.rs` (6マッチ - `ctx.file_attrs()`メソッド）
  - **共通**:
    - `tests/parser2_integration_test.rs` (パスタワークスペースレベル)
- **修正スコープ**: 
  - テストコード: `file.items`への書き換え（メカニカル）
  - TranspileContext: `ctx.file_attrs()`メソッドは残す（内部実装）
  - pasta_coreのAST定義からヘルパーメソッド削除

**実現難度**: M（メカニカルな修正だが量が多い）
**重要性**: 🔴 CRITICAL - 本仕様の存在理由

**注意**: この要件は本仕様の「核心」であり、除外不可。

---

## 3. 実装アプローチの選択肢

### オプションA: 段階的移行（推奨）

**方針**: 既存コードを最小限の変更で両立させながら、段階的にpasta_rune型に統合

**フェーズ1: 新API追加**
- `transpile_file(&PastaFile, &mut SceneRegistry, &mut WordDefRegistry, &mut W)` メソッドを追加
- `TranspileContext::accumulate_file_attr()` メソッドを追加
- pasta_runeの2パス戦略を模倣するか、1パスで完結するか選択

**フェーズ2: 既存メソッド非推奨化**
- `transpile()` / `transpile_with_globals()` に `#[deprecated]` を付与
- 内部でファイル再構築を行い `transpile_file()` を呼び出す（データ損失の可能性あり）

**フェーズ3: テスト修正**
- ファイル側のヘルパーメソッド廃止
- テストをすべて `file.items` 直接アクセスに変更

**メリット**:
- ✅ 既存テストを段階的に修正可能
- ✅ 逐次的な検証が容易（各フェーズ終了時にコンパイル・テスト実行）
- ✅ 後方互換性を保ちながら移行
- ✅ 他の依存箇所への波及を制御できる

**デメリット**:
- ❌ フェーズが多い（3段階）
- ❌ フェーズ2で非推奨メソッドがファイル再構築を行うため、出現順情報が失われる
- ❌ コードの複雑さが一時的に増加（非推奨メソッド + 新メソッルト並存）

**複雑度**: M（中程度）
**リスク**: Low（pasta_runeが参照実装として存在）

---

### オプションB: 完全リプレイス

**方針**: 既存メソッドを削除し、`transpile_file()` のみを提供。既存テストはすべて再実装。

**実装内容**:
1. `transpile()` / `transpile_with_globals()` を削除
2. `transpile_file(&PastaFile, ...)` のみを実装
3. すべてのテストを `file.items` ベースに再実装
4. ヘルパーメソッドもPastaFile側で廃止

**メリット**:
- ✅ シンプルなAPI（1メソッドのみ）
- ✅ 出現順処理が強制される（設計意図が明確）
- ✅ コード重複がない

**デメリット**:
- ❌ 既存テスト/呼び出し元がすべて破壊される（Breaking Change）
- ❌ 一度にすべてを修正する必要あり（リスク高い）
- ❌ 非推奨メソッドを使用している他のプロジェクトが即座に使用不可に

**複雑度**: L（大きい）
**リスク**: High（Breaking Changeが大きい、段階的検証が難しい）

---

### オプションC: ハイブリッド（段階的リプレイス）

**方針**: オプションAの改良版。フェーズ2・3を併行実施で短縮

**実装内容**:
1. `transpile_file(&PastaFile, ...)` を新規実装
2. `transpile()` / `transpile_with_globals()` に `#[deprecated]` を付与し、内部で `transpile_file()` を呼び出すようにする
3. **同時に**、テスト側でヘルパーメソッド廃止とファイル側API修正を実施
4. フェーズ2と3を同時実行

**メリット**:
- ✅ オプションAより工期が短い（フェーズ2・3並行）
- ✅ 後方互換性を保持（非推奨メソッドは動作）
- ✅ テストは一度の修正で完了

**デメリット**:
- ❌ フェーズ2・3の並行実施により、一時的に複雑性が高まる
- ❌ コンパイルエラーが多数発生し、修正に集中が必要

**複雑度**: M（中程度）
**リスク**: Medium（段階的検証が必要）

---

## 4. 要件別のギャップマップ

| 要件ID | タイトル | 現状 | ギャップ | 難度 | 対応策 |
|--------|---------|------|---------|------|-------|
| REQ-1 | PastaFile入力IF | transpile(&[...]) | 新メソッド追加 | S | オプションA: transpile_file() 追加 |
| REQ-2 | FileItem出現順 | 型別処理 | FileItem iterate 実装 | M | オプションA: match文追加 |
| REQ-3 | ファイルレベル属性 | transpile_with_globals() で指定 | accumulate_file_attr() 実装 | S | TranspileContextに追加 |
| REQ-4 | グローバル単語登録 | context.register_global_words() 呼び出し | FileItem::GlobalWord match arm | S | match文に追加 |
| REQ-5 | シーン処理順序 | 現在の実装で対応可能 | ファイル属性キャッシュ | M | context に file_attrs フィールド追加 |
| REQ-6 | アクター処理順序 | ActorScope処理ロジックなし | FileItem::ActorScope match arm | S | match文に追加 |
| REQ-7 | API一貫性 | API異なる | メソッドシグネチャ統一 | S | 参考: pasta_rune Transpiler2 |
| REQ-8 | 後方互換性 | 既存メソッド存在 | #[deprecated] 付与 | S | 非推奨ラッパー実装 |
| REQ-9 | テストカバレッジ | FileItem単体テストなし | 新テスト群追加 | M | pasta_transpiler2_unit_test 参考 |
| REQ-10 | ヘルパメソッド廃止 | file_attrs() など存在 | 廃止 + テスト修正 | M | 50マッチを file.items に置換 |

---

## 5. 実装複雑度と リスク評価

### オプションA: 段階的移行（推奨）

**工期見積**:
- フェーズ1 (新API): 1～2日
  - `transpile_file()` メソッド実装: 4時間
  - `accumulate_file_attr()` 追加: 2時間
  - FileItem match 実装: 2時間
  - 新テスト群: 4時間
  
- フェーズ2 (非推奨化): 1日
  - 既存メソッドにラッパー実装: 3時間
  - 互換性テスト: 2時間
  
- フェーズ3 (ヘルパ廃止): 1～2日
  - テスト修正 (50マッチ): 4時間
  - コンパイルエラー対応: 2時間
  - 回帰テスト: 2時間

**総工期**: 3～6日（作業の並行性による）

**リスク レベル**: **Low**
- **理由**:
  - pasta_runeに参照実装が存在
  - 段階的検証により問題の早期発見が可能
  - 後方互換性により既存テストの修正が随時可能

**リスク項目**:
1. **フェーズ2でのデータ損失** (リスク: Medium)
   - 既存メソッド呼び出し時に PastaFile を再構築 → 出現順情報が失われる
   - 完全な後方互換性が保証できない可能性
   - **対策**: 非推奨メソッド使用時に警告を記載

2. **テスト修正漏れ** (リスク: Low)
   - grep検索で50マッチ → メカニカルな修正
   - スクリプトで一括置換可能

---

### オプションB: 完全リプレイス（非推奨）

**工期見積**: 2～4日

**リスク レベル**: **High**
- **既存プロジェクト破壊**: Breaking Change
- **テスト修正**: 一度に大量修正 → デバッグが困難
- **段階的検証不可**: すべてが一度に変わるため、問題特定が難しい

---

### オプションC: ハイブリッド

**工期見積**: 2～3日

**リスク レベル**: **Medium**
- **フェーズ2・3並行**: 複雑性増加
- **段階的検証限定**: フェーズ1のみ検証可能

---

## 6. 設計フェーズへの提言

### 推奨アプローチ: **オプションA（段階的移行）**

**実装順序**:

1. **フェーズ1: 新API実装（リスク最小化）**
   - `transpile_file()` を新規実装
   - FileItem イテレーション処理を実装
   - `TranspileContext::accumulate_file_attr()` を追加
   - **検証**: 新テスト群が green になることを確認
   - **メリット**: 既存メソッドはそのまま → 回帰リスク 0

2. **フェーズ2: 既存メソッド非推奨化**
   - `transpile()` / `transpile_with_globals()` に `#[deprecated]` を付与
   - 内部実装を`transpile_file()`呼び出しに変更
   - **検証**: 既存テストが変わらずgreen
   - **注意**: データ損失の可能性を ドキュメント化

3. **フェーズ3: ヘルパーメソッド廃止**
   - PastaFile側から4メソッドを削除
   - テストを `file.items` に書き直し
   - **検証**: すべてのテストがgreen

### 設計ドキュメントで明確にすべき事項

1. **TranspileContext の拡張**
   - ファイル属性の累積状態管理（新フィールド）
   - accumulate_file_attr() の仕様

2. **FileItem イテレーション処理**
   - 各FileItem種別の処理順序
   - アクター処理の実装方針（TODO?）

3. **非推奨メソッドの取扱**
   - フェーズ2でのPastaFile再構築の実装詳細
   - データ損失の可能性と利用者への通知方法

4. **テスト修正戦略**
   - 50マッチの修正スコープ
   - 修正手順（スクリプト化の検討）

### 研究が必要な項目

1. **アクター処理の詳細仕様**
   - FileItem::ActorScope をどのように処理するか？
   - 現在のpasta_luaではアクターを先行処理しているが、出現順に変更した場合の影響は？
   - **アクション**: 設計フェーズで要件5.1と合意する

2. **ファイル属性の影響範囲**
   - ファイル属性がシーン/アクター処理にどのように影響するのか？
   - 累積された属性をコード生成時にどのように活用するのか？
   - **アクション**: pasta_rune Transpiler2 のコード生成部を詳しく確認

3. **2パス vs 1パス**
   - pasta_luaが2パス構造に対応するか？（pasta_runeはpas1と pass2に分かれている）
   - Luaコード生成には pass2 は不要か？
   - **アクション**: 設計フェーズで決定

---

## 7. 結論

### 現状から要件への移行可能性

✅ **全要件実現可能** (pasta_runeの参照実装が存在)

| 要件 | 実現性 | 理由 |
|------|--------|------|
| REQ-1～6 | 高 | pasta_runeで既に実装済み |
| REQ-7～9 | 高 | パターン既知、メカニカルな実装 |
| REQ-10 | 中 | スコープ大きい（50マッチ）が難度低い |

### 推奨実装アプローチ

**オプションA（段階的移行）**を採用すること。理由：
- ✅ リスク最小（Low）
- ✅ 段階的検証可能
- ✅ 既存テストの漸進的修正が可能
- ✅ 後方互換性を保ちながら移行

### 設計フェーズでの課題

1. TranspileContext 拡張仕様の明確化
2. FileItem処理順序とアクター処理の詳細設計
3. ファイル属性累積の実装詳細
4. テスト修正戦略の具体化

---

**次ステップ**: 要件を承認後、`/kiro-spec-design lua-transpiler-pastafile-migration` で設計ドキュメントを生成してください。
