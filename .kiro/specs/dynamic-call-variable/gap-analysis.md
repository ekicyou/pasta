# ギャップ分析: dynamic-call-variable

## 分析対象

**仕様書参照**: doc/spec/04-call-spec.md §4.1 パターン2（動的ターゲット）

```text
call_target ::= call_marker ~ (id | expr)
例: ＞＄target_label          (変数参照 = expr の var_ref)
   >＄＊dynamic_choice   (グローバル変数 = expr の var_ref_global)
   >（＠func（））          (括弧式 = expr の paren_expr + fn_call)
   >＄a ＋ ＄b           (二項演算 = expr の Binary)
```

**技術的前提**: `id`（XID_START / `_` 先頭）と `expr`（`＄`/`＠`/`（`/数字/`「`/`"` 先頭）の先頭文字集合は完全に素集合→ PEG の `(id | expr)` は曖昧性なし

## 1. 現状調査

### 修正対象ファイル一覧

| レイヤー | ファイル | 現状 |
|---------|--------|------|
| PEG 文法 | `crates/pasta_dsl/src/parser/grammar.pest` | `call_scene = { call_marker ~ id ~ s ~ args? }` — **id 固定** |
| AST 型 | `crates/pasta_dsl/src/parser/ast/action.rs` | `CallScene.target: String` — **文字列固定** |
| パーサー | `crates/pasta_dsl/src/parser/parse_action.rs` | `parse_call_scene()` — **`Rule::id` のみハンドル** |
| コード生成 | `crates/pasta_lua/src/code_gen/element_gen.rs` | `generate_call_scene()` — **ハードコード文字列リテラル** |
| Lua ランタイム | `crates/pasta_lua/pasta_scripts/pasta/act.lua` | `ACT_IMPL.call(self, global, key, ...)` — key=nil 時の早期リターンガード追加（小規模変更） |

### 既存パターンの活用

**`Expr` AST 型**（`ast/action.rs`）―完全実装済み:
```rust
pub enum Expr {
    Integer(i64), Float(f64), String(String), BlankString,
    VarRef { name: String, scope: VarScope },
    FnCall { name: String, args: Vec<Expr> },
    Paren(Box<Expr>),
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
}
```

**`generate_expr()` 関数**（`element_gen.rs`）―完全実装済み:
- 全 `Expr` バリアントに対応した Lua 式を生成する
- 動的コールのターゲット生成は `tostring(generate_expr(expr))` の形でそのまま再利用可能

**`try_parse_expr()` / `parse_expr_from_parts()` 関数**（`parse_action.rs`）―完全実装済み:
- Pest パースソースランを受け取り `Expr` AST を構築する
- この関数を `parse_call_scene()` で再利用することで新規ロジックが不要

**`ACT_IMPL.call` の既存シグネチャ**:
```lua
function ACT_IMPL.call(self, global_scene_name, key, attrs, ...)
```
- `key` パラメータは既に任意の文字列値を受け付ける
- **Lua ランタイム側の変更は不要**

## 2. 要件と既存資産の対照表（Requirement-to-Asset Map）

| 要件 | 既存資産 | ギャップ |
|------|---------|---------|
| R1: `＞＄変数名` のパース | `var_ref` ルールは存在するが `call_scene` が `id` 固定 | **Missing**: `call_scene` に `var_ref` 代替を追加 |
| R1: 半角 `>$var` 対応 | `call_marker =_{ gt }` / `var_marker =_{ dollar }` で全角半角両対応済み | ✅ ギャップなし（PEG ルール結合で自動的に対応）|
| R1: グローバル変数 `＞＄＊var` | `var_ref_global` ルール存在 | **Missing**: `call_scene` への統合 |
| R1: 静的コールとの区別 | `CallScene.target: String` | **Missing**: 列挙型 `CallTarget` 導入 |
| R2: Lua コード生成 | `generate_call_scene()` 内で `target` を文字列リテラルとして出力 | **Missing**: 動的ターゲット用分岐 |
| R2: 前方一致セマンティクス | `act:call()` → `find_scene()` で既に前方一致検索実行 | ✅ ギャップなし |
| R2: フィルター対応 | 静的コールのフィルターは将来予約（現在無視） | ✅ ギャップなし（同等の扱い）|
| R3: ランタイム実行 | `ACT_IMPL.call()` は任意文字列 key を受け付ける | ✅ ギャップなし |
| R3: 候補不在時の挙動 | `find_scene()` が `nil` 返却 → ログ出力後 `nil` 返却 | ✅ ギャップなし |
| R3: 未定義変数(nil)時 | `ACT_IMPL.call` に key=nil ガードなし。`tostring(nil)`=`"nil"` で検索される | **Missing**: `ACT_IMPL.call` 先頭で key==nil チェック、警告ログ、早期 return nil |
| R4: 既存互換性 | 950+ テスト全パス | テスト回帰チェックのみ |

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張（**推奨**）

変更ファイル数を最小化し、既存パターンに沿って4ファイルを外科的に拡張する。

**変更内容（4ファイル）**:

1. **grammar.pest** — `call_scene` ルール拡張
   ```pest
   # Before
   call_scene = { call_marker ~ id ~ s ~ args? }
   
   # After
   call_scene = { call_marker ~ (id | expr) ~ s ~ args? }
   ```
   - `id` と `expr` の先頭文字集合は完全に素集合 → PEG の選択で曖昧性なし
   - `＞＄変数`・`＞＄＊変数`・`＞（＠func（））` 等はすべて `expr` ブランチで解決される

2. **ast/action.rs** — `CallScene` 型拡張
   ```rust
   pub enum CallTarget {
       Static(String),
       Dynamic(Expr),   // 全 Expr バリアントを許容
   }
   
   pub struct CallScene {
       pub target: CallTarget,  // 旧: target: String
       pub args: Option<Args>,
       pub span: Span,
   }
   ```

3. **parse_action.rs** — `parse_call_scene()` 拡張
   - `id` ブランチ: 既存の文字列取得 → `CallTarget::Static(String)`
   - `expr` ブランチ: 既存の `try_parse_expr()` を再利用 → `CallTarget::Dynamic(Expr)`

4. **element_gen.rs** — `generate_call_scene()` 拡張
   ```rust
   // Dynamic(expr): tostring(をラップして generate_expr(expr) を出力
   // act:call(SCENE.__global_name__, tostring(<expr>), {}, ...)
   // Static(name): 既存そのまま
   // act:call(SCENE.__global_name__, "name", {}, ...)
   ```
   - `generate_expr()` は全 Expr バリアント実装済みのため、新規ロジックは `tostring()` ラップのみ

**トレードオフ**:
- ✅ 最小変更（5ファイル、各数行の変更）
- ✅ 既存の `Expr` AST・`generate_expr()`・`try_parse_expr()` を完全再利用（新規ロジックなし）
- ✅ Lua ランタイムは key=nil ガードの追加のみ（構造的変更なし）
- ✅ 末尾呼び出し最適化（TCO）は既存の `is_tail_call` フラグで自動対応
- ❌ `CallScene.target` 型変更により既存参照箇所の修正が必要（コンパイラが検出）

### Option B: 新コンポーネント（別 AST ノード）

`CallScene` は変更せず、`DynamicCallScene` を新規追加する。

**トレードオフ**:
- ✅ 既存の `CallScene` は一切変更なし
- ❌ コード重複（`generate_call_scene` と `generate_dynamic_call_scene` がほぼ同一）
- ❌ パーサー側で2つの別ルールを分岐管理が必要
- ❌ `local_scene_item` 等の受け入れリスト拡張が大量に必要

### Option C: ハイブリッド（文法だけ拡張、AST は文字列保持）

`CallScene.target` を `String` のまま維持し、動的ターゲットの場合は特殊プレフィックス（`$varname`）付き文字列として格納する。

**トレードオフ**:
- ✅ AST 型変更なし
- ❌ 型安全性の喪失（文字列パターンマッチに依存）
- ❌ グローバル変数 `$*var` の表現が煩雑
- ❌ Rust の型システムの恩恵を受けられない

## 4. 実装複雑度とリスク

**工数**: **S（1〜3日）**
- 既存パターンの直接的な拡張であり、新規アーキテクチャなし
- 変更ファイル数が5個と限定的
- Lua ランタイムは key=nil ガードの追加のみ（構造的変更なし）

**リスク**: **Low**
- 既存の `var_ref` 解析パターンが確立しており、その流用で完結
- `CallScene.target` 型変更の影響範囲はコンパイラが完全に検出
- 950+ テストによる回帰検証可能

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ
**Option A（既存コンポーネント拡張）** — 最小変更・最大型安全性

### 主要設計判断（設計フェーズで確定）
1. `CallTarget` 列挙型の正確な定義（`VarScope` の再利用方法）
2. `CallScene.target` 型変更に連動する全参照箇所の特定と修正戦略
3. スナップショットテストの追加パターン（静的/動的/グローバル変数/末尾呼び出し）

### リサーチ不要項目
- Lua ランタイムの構造的変更: 不要確認済み（key=nil ガードの追加のみ）
- フィルター対応: 静的コールと同等の将来予約扱い
- 半角/全角対応: PEG マーカー定義で自動対応済み
