# Gap Analysis: req-var-expansion

## 概要

`act.req` テーブルの内容を `act.var` に展開してDSLからアクセスしやすくする機能のギャップ分析。
全パイプライン（PEG文法 → AST → トランスパイラ → Lua ランタイム）を調査し、実証テストにより検証済み。

---

## 1. 現状調査

### 1.1 既存コンポーネントの全体像

| コンポーネント | ファイル | 状態 |
|---|---|---|
| PEG文法 `var_id`/`digit_id` | `crates/pasta_core/src/parser/pasta.pest` | ✅ 全角数字対応済み |
| パーサー `digit_id → VarScope::Args` | `crates/pasta_core/src/parser/mod.rs` L826-835 | ✅ 動作確認済み |
| AST `VarScope::Args(u8)` | `crates/pasta_core/src/parser/ast.rs` L719-727 | ✅ 定義済み |
| コード生成 `Args → args[N+1]` | `crates/pasta_lua/src/code_generator.rs` L519,583,651 | ✅ 動作中 |
| `ACT` ベースクラス | `crates/pasta_lua/scripts/pasta/act.lua` | ✅ `var={}` 初期化 |
| `SHIORI_ACT` 派生クラス | `crates/pasta_lua/scripts/pasta/shiori/act.lua` | ✅ `req` 保持 |
| `transfer_date_to_var()` | `crates/pasta_lua/scripts/pasta/shiori/act.lua` | ✅ テンプレートパターン |
| `act.req.reference[N]` | SHIORI リクエストパーサー | ✅ 0-indexed配列 |

### 1.2 パイプラインフロー（現在）

```
＄０ (Pasta DSL)
  ↓ PEG: dollar ~ var_id → digit_id
  ↓ Parser: digit_id → normalize_number_str("０")→"0" → VarScope::Args(0)
  ↓ CodeGen: VarScope::Args(0) → format!("args[{}]", 0+1) → "args[1]"
  ↓ Lua出力: act.さくら:talk(tostring(args[1]))
```

### 1.3 `args` と `var` の関係

| 属性 | `args` | `var` |
|------|--------|-------|
| 定義場所 | `local args = { ... }` (関数ローカル) | `self.var = {}` (ACT.new) |
| データ源 | Call/Jump引数 | DSL `＄X＝Y` / transfer系関数 |
| ライフサイクル | 関数呼び出しごと | actインスタンス全体 |
| Luaアクセス | `args[N]` | `var.キー` or `var["キー"]` |

---

## 2. 🚨 重大な設計衝突の発見

### 2.1 `＄０` の二重の意味

**現在の動作**: `＄０` → `args[1]` （Call/Jump引数参照）
**要件の期待**: `＄０` → `var["０"]` （`act.req.reference[0]` の値）

`＄０` は既に **シーン引数参照** として使われており、**Requirement 4** が期待する `var["０"]` アクセスとは異なるパスを通ります。

### 2.2 実証テスト結果（パーサー）

`crates/pasta_core/tests/digit_id_var_test.rs` を作成し4テスト全パス:

| テスト | 結果 | 確認内容 |
|---|---|---|
| `test_fullwidth_digit_0_parsed_as_args_0` | ✅ | `＄０` → `VarScope::Args(0)`, name="０" |
| `test_fullwidth_digit_1_to_9_parsed_as_args` | ✅ | `＄１`〜`＄９` → `Args(1)`〜`Args(9)` |
| `test_halfwidth_digit_0_also_parsed_as_args` | ✅ | `$0` → `Args(0)` (回帰テスト) |
| `test_multidigit_fullwidth_parsed_as_args` | ✅ | `＄１０` → `Args(10)`, name="１０" |

### 2.3 Lua側の検証

- `var["０"]` はLuaテーブルキーとして**完全に有効**（文字列キーは任意の文字列が使える）
- 既存の `var["時１２"]`, `var["年"]` 等が問題なく動作している（`transfer_date_to_var` テスト47件パス）
- **問題はLua側ではなく、DSLの `＄０` が `var["０"]` を読まないこと**

---

## 3. 実装アプローチの選択肢

### Option A: `transfer_req_to_var()` のみ（DSL変更なし）

**概要**: `transfer_req_to_var()` で `var["r0"]`〜`var["r9"]` と `var.event` 等に展開。DSLの `＄０` は引数参照のまま残す。Luaコードブロック内では `var["０"]` で直接アクセス可能。

**変更箇所**:
- `crates/pasta_lua/scripts/pasta/shiori/act.lua` に `transfer_req_to_var()` メソッド追加のみ

**Trade-offs**:
- ✅ 既存動作に一切影響なし（破壊的変更ゼロ）
- ✅ 実装最小限（Luaファイル1つに1メソッド追加）
- ✅ `＄r0`〜`＄r9` でDSLからもアクセス可能（`r0`はXID_START開始なので`VarScope::Local`）
- ❌ `＄０` でのアクセスは不可（`args[1]`に行く）
- ❌ Requirement 4 を直接満たせない

### Option B: `＄０` のセマンティクス変更（args → var）

**概要**: コード生成で `VarScope::Args(N)` → `var["０"]` に変更。`init_scene` 内で `args` を `var` に自動展開。

**変更箇所**:
- `code_generator.rs`: `VarScope::Args` の出力を `args[N+1]` → `var["全角数字"]` に変更
- `act.lua`: `init_scene` で `args` → `var["０"]`〜`var["９"]` に自動転記
- テストフィクスチャ: `sample.lua` 等の期待値更新

**Trade-offs**:
- ✅ `＄０` がReq Referenceにも引数にもシームレスに使える
- ✅ Requirement 4 を完全に満たす
- ❌ **既存のargs参照パターンが変わる**（破壊的変更の可能性）
- ❌ `args` と `var` の二重管理が必要（Call/Jump引数も`var`に書く）
- ❌ 変更範囲が大きい（Rust+Lua+テスト）

### Option C: ハイブリッド（推奨）

**概要**: `transfer_req_to_var()` で `var["０"]`〜`var["９"]` と `var["r0"]`〜`var["r9"]` の両方に展開。`＄０` の既存セマンティクス（args参照）は維持。DSLからのアクセスは `＄r0`（ASCII識別子）を推奨。

**変更箇所**:
- `crates/pasta_lua/scripts/pasta/shiori/act.lua`: `transfer_req_to_var()` 追加
- Requirement 4 の修正: `＄０` ではなく `＄r0` をDSL推奨アクセスとする
- テスト追加

**Trade-offs**:
- ✅ 既存動作完全互換
- ✅ `var["０"]` はLuaコードブロック内で使用可能
- ✅ `＄r0`〜`＄r9` でDSL構文からも参照可能
- ✅ 実装シンプル
- ⚠️ Requirement 4 の文言修正が必要（`＄０` → `＄r0` に変更）

---

## 4. 要件とのギャップマッピング

| 要件 | 状態 | ギャップ |
|------|------|----------|
| Req 1: Reference展開 | 🟡 Gap小 | `transfer_req_to_var()` を新規作成するだけ。テンプレート（`transfer_date_to_var`）が完備 |
| Req 2: イベント識別情報展開 | 🟡 Gap小 | 同上。`act.req.id`, `act.req.base_id` が既にLuaテーブルに存在 |
| Req 3: 明示的呼び出しパターン | ✅ Gap無し | `transfer_date_to_var` と完全に同じパターン |
| Req 4: `＄ｒ０`でのDSLアクセス | ✅ Gap解消 | `＄ｒ０` は `id` ルール → `VarScope::Local` → `var.ｒ０` に解決。要件を `＄ｒ０` ベースに修正済み |
| Req 5: 既存機能との整合性 | ✅ Gap無し | キー衝突なし確認済み（date展開は漢字/英語キー、req展開は数字/rNキー） |

---

## 5. 実装複雑度とリスク

| 項目 | 評価 | 理由 |
|------|------|------|
| 工数 | **S（1-3日）** | Luaファイル1つにメソッド1つ追加 + テスト。既存パターン踏襲 |
| リスク | **低** (Option A/C) / **中** (Option B) | Option A/C は既存コードに影響なし。Option B は破壊的変更あり |

---

## 6. 設計フェーズへの推奨事項

### 推奨アプローチ: **Option C（ハイブリッド）** → ✅ 採用決定

> **決定事項（2026-02-07）**: `＄ｒ０`〜`＄ｒ９`（全角`ｒ` + 全角数字）を採用。`ｒ` は XID_START であるため通常の `id` ルールでパースされ `VarScope::Local` → `var.ｒ０` に解決される。文法変更不要。requirements.md は更新済み。

1. ~~**Requirement 4 を修正**~~ → ✅ 完了。`＄０` → `＄ｒ０`〜`＄ｒ９` に変更済み
2. **`transfer_req_to_var()` の実装**: `transfer_date_to_var()` と同一パターン
3. **展開キーの設計**:
   - 全角キー: `var["ｒ０"]`〜`var["ｒ９"]` （DSL `＄ｒ０` 用、推奨）
   - ASCIIキー: `var.r0`〜`var.r9` （DSL `＄r0` 用、代替）

---

## 付録: 実証テストファイル

- `crates/pasta_core/tests/digit_id_var_test.rs` — パーサーの全角数字→Args変換テスト（4テスト全パス）
