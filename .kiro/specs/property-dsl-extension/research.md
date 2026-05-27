# ギャップ分析: property-dsl-extension

## 1. 要件→既存アセットマッピング

| 要件                                          | 関連アセット                                                                                                           | ギャップ                                                                                                 |
| --------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| R1: ＄％マーカー + プロパティ名文法           | `grammar.pest` L28 `dollar`, L26 `modulo`, L47 `actor_marker`                                                          | **Missing**: `property_marker`ルール、`property_id`ルール（`id`ルールとは文字クラスが異なる）            |
| R1: プロパティ名 `[a-zA-Z][_().a-zA-Z0-9]*`   | `grammar.pest` L21 `id` = `XID_START ~ XID_CONTINUE*`                                                                  | **Constraint**: 既存`id`ルールはUnicode全域を許容するが、プロパティ名はASCIIベースの独自文字クラスが必要 |
| R2: SET `＄％prop＝value`                     | `grammar.pest` L87-89 `var_set`, `parse_elements.rs` L110 `parse_var_set()`, `element_gen.rs` L18 `generate_var_set()` | **Missing**: `var_set_property`ルール、`VarScope::Property`ディスパッチ                                  |
| R3: GET代入 `＄var＝＄％prop`                 | `grammar.pest` L90 `set = set_marker ~ (expr \| word_ref)` → `term` → `var_ref`                                        | **Missing**: `var_ref_property`ルール。式のtermとしてプロパティ参照を認識する必要あり                    |
| R4: GETインライン `さくら：＄％prop テキスト` | `grammar.pest` L163 `action` → `var_ref`, `element_gen.rs` L168 `generate_action()` VarRef分岐                         | **Missing**: `VarScope::Property`用のコード生成 + バッチ最適化（プリフェッチ）                           |
| R5: 既存構文互換                              | `grammar.pest` 既存ルール全体                                                                                          | **Low Risk**: `＄％`はPEG順序で既存`var_ref_local`より先にマッチさせれば衝突なし（`％`はXID_START外）    |
| R6: 構文エラー                                | `pasta_dsl/src/error.rs`, Pest自動エラー                                                                               | **Missing**: プロパティ名不正時のカスタムエラーメッセージ                                                |

## 2. 現状調査

### 2.1 パーサー層 (pasta_dsl)

#### Pest文法 (`crates/pasta_dsl/src/parser/grammar.pest`)

**マーカー定義** (L26-57):
```pest
modulo = _{ "%" | "％" }       # L26
dollar = _{ "＄" | "$" }       # L28
actor_marker = _{ modulo }     # L57
var_marker   = _{ dollar }     # L54
```

**変数参照** (L79-84):
```pest
var_ref        =_{ var_ref_global | var_ref_local }
var_id         = { id | digit_id }
var_ref_local  = { var_marker ~ var_id ~ s }
var_ref_global = { var_marker ~ global_marker ~ id ~ s }
```

**変数代入** (L87-91):
```pest
var_set        =_{ var_set_global | var_set_local | var_set_none }
var_set_local  = { var_marker ~ id ~ s ~ set }
var_set_global = { var_marker ~ global_marker ~ id ~ s ~ set }
var_set_none   = { var_marker ~ set }
set            =_{ set_marker ~ s ~ ( expr | word_ref ) }
```

**アクション** (L163-164):
```pest
action  =_{ at_escape | dollar_escape | sakura_escape | fn_call | word_ref | var_ref | sakura_script | talk }
actions = { action+ }
```

**式のterm** (L70-76):
```pest
term = _{
    paren_expr | fn_call | var_ref | number_literal | string_literal
}
```

**衝突分析**: `＄％` で始まる入力に対し、既存ルールの挙動:
- `dollar_escape` = `dollar{2}` → `＄＄`のみ。`＄％`は不一致。
- `var_ref_local` = `var_marker ~ var_id` → `var_id = id | digit_id`。`id`は`XID_START`で始まる必要があり、`％` (U+FF05, カテゴリSo) はXID_START外。**衝突なし**。
- `var_set_local` = `var_marker ~ id ~ s ~ set` → 同上、`id`が`％`にマッチしない。**衝突なし**。
- `actor_marker` = `modulo` → 行頭`％`のみ使用（アクター定義行）。`＄％`は`var_set_line`/`action`文脈でのみ出現。**衝突なし**。

#### AST定義 (`crates/pasta_dsl/src/parser/ast/action.rs`)

```rust
// L260-267
pub enum VarScope {
    Local,       // $var
    Global,      // $*var
    Args(u8),    // $0, $1, ...
}
// derive: Debug, Clone, Copy, PartialEq, Eq
```

`VarScope`は`Action::VarRef`、`Expr::VarRef`、`VarSet`の3箇所で使用。`Property`バリアント追加は全箇所に波及する。

#### AST構築

- **変数参照パース** (`parse_action.rs` L80-128): `Rule::var_ref_local`と`Rule::var_ref_global`をmatch。`Rule::var_ref_property`を追加する必要あり。
- **変数代入パース** (`parse_elements.rs` L110-190): `pair.as_rule()`で`VarScope`を決定。`Rule::var_set_property => VarScope::Property`を追加。
- **シーンアイテムディスパッチ** (`parse_scene.rs` L190, L243): `Rule::var_set_local | Rule::var_set_global | Rule::var_set_none`のmatchパターンに`Rule::var_set_property`を追加（2箇所）。

#### 部分パーサー (`partial.rs` L54)

```rust
'＄' | '$' => Some(Rule::var_set_line),
```

`＄％`も`＄`で始まるため、`var_set_line`ルール推論は変更不要。

### 2.2 トランスパイラ層 (pasta_lua)

#### コード生成 (`crates/pasta_lua/src/code_gen/element_gen.rs`)

**`generate_var_set()`** (L18-69):
```rust
let var_path = match var_set.scope {
    VarScope::Local => format!("var.{}", name),
    VarScope::Global => format!("save.{}", name),
    VarScope::Args(_) => { /* error */ }
};
// → var_path = expr  形式で出力
```

SETの場合: `＄％prop＝value` → `act:set_property("prop", value)` を生成する必要あり。既存の`var_path = expr`パターンとは根本的に異なるコード生成が必要。

GET代入の場合: `＄var＝＄％prop` → 右辺の`＄％prop`は式のterm（`Expr::VarRef { scope: Property }`）として解析される。`generate_expr()`で`act:get_property("prop")`を生成する必要があるが、**`get_property()`はコルーチンyieldを伴う非同期関数**。式中でyieldはできない。

**`generate_action()`** (L168-235):
```rust
Action::VarRef { name, scope, .. } => {
    let var_path = match scope {
        VarScope::Local => format!("var.{}", name),
        VarScope::Global => format!("save.{}", name),
        VarScope::Args(index) => format!("args[{}]", index + 1),
    };
    self.writeln(&format!("act.{}:talk(tostring({}))", actor, var_path))?;
}
```

インラインGETの場合: アクション行内の`＄％prop`は`act:get_property("prop")`を呼んで結果をtalkする必要がある。**get_propertyはyieldするため、個別にtalkを分割するか、プリフェッチが必要**。

**`generate_action_line()`** (L131-146):
```rust
pub(super) fn generate_action_line(&mut self, action_line: &ActionLine, last_actor: &mut Option<String>) -> Result<(), TranspileError> {
    let actor = &action_line.actor;
    *last_actor = Some(actor.clone());
    for action in &action_line.actions {
        self.generate_action(action, actor)?;
    }
    Ok(())
}
```

アクションは**逐次処理**される。バッチ最適化（R4: 複数プロパティ参照の一括プリフェッチ）には、ここにプリプロセスパスを追加する必要がある。

**`generate_expr()`** (L244-310):
```rust
Expr::VarRef { name, scope } => {
    let var_path = match scope {
        VarScope::Local => format!("var.{}", name),
        VarScope::Global => format!("save.{}", name),
        VarScope::Args(index) => format!("args[{}]", index + 1),
    };
    write!(self.writer, "{}", var_path)?;
}
```

`generate_expr_to_buffer()` (L321-395) にも同じパターンあり。式中のプロパティ参照はスコープ外（R3は`＄var＝＄％prop`のみ、式中の`＄％a ＋ ＄％b`は対象外）。

### 2.3 プロパティAPI (pasta_lua)

**`set_property()`** (`pasta_scripts/pasta/shiori/act.lua`):
```lua
function SHIORI_ACT_IMPL.set_property(self, name, value)
    -- name: string (必須), value: any (tostring変換)
    -- escape_tag_arg()でエスケープ
    -- \![set,property,name,value] タグを発行
    -- 同期的、yield不要
end
```

**`get_property()`** (`pasta_scripts/pasta/shiori/act.lua`):
```lua
function SHIORI_ACT_IMPL.get_property(self, name_or_names, timeout, timeout_message)
    -- name_or_names: string | string[] (必須)
    -- 複数名を受け取り、\![get,property,event_id,name1,name2,...] を発行
    -- coroutine.yield()でSSPからの応答を待機
    -- 戻り値: table.unpack(out, 1, n) — 複数値を返す
    -- コルーチン外では error()
end
```

**重要**: `get_property()`は**複数プロパティ名を一括取得可能**。これがバッチ最適化の基盤となる。

### 2.4 テストパターン

既存パターン (`crates/pasta_dsl/tests/digit_id_var_test.rs`):
```rust
fn find_var_refs_in_scene(scene: &GlobalSceneScope) -> Vec<(String, VarScope)> {
    // ActionLine/ContinueAction内のAction::VarRefを収集
}

#[test]
fn test_fullwidth_digit_0_parsed_as_args_0() {
    let input = "＊テスト\n　さくら：＄０\n";
    let result = parse_str(input, "test.pasta");
    // assert VarScope::Args(0)
}
```

トランスパイラテストは `crates/pasta_lua/tests/` 配下にLuaスナップショットテスト形式。

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張（推奨）

既存の`VarScope`・`VarRef`・`VarSet`機構を拡張し、`Property`バリアントを追加。

**変更対象ファイル**:

| ファイル              | 変更内容                                                                                           | 行数目安        |
| --------------------- | -------------------------------------------------------------------------------------------------- | --------------- |
| `grammar.pest`        | `property_marker`, `property_id`, `var_ref_property`, `var_set_property` 追加 + 既存ルール順序更新 | +10行, ~3行変更 |
| `ast/action.rs`       | `VarScope::Property` 追加                                                                          | +2行            |
| `parse_action.rs`     | `Rule::var_ref_property` match arm                                                                 | +12行           |
| `parse_elements.rs`   | `Rule::var_set_property` ディスパッチ                                                              | +1行            |
| `parse_scene.rs`      | `Rule::var_set_property` をmatchパターンに追加 (2箇所)                                             | +2行            |
| `element_gen.rs`      | SET/GET/インライン用コード生成 + バッチ最適化                                                      | +40-80行        |
| テストファイル (新規) | パーサー + トランスパイラテスト                                                                    | +100-150行      |

**トレードオフ**:
- ✅ 既存パターンに完全準拠（`VarScope`拡張は`Args`追加時の前例あり）
- ✅ matchの網羅性チェックで漏れを自動検出
- ✅ `partial.rs`変更不要（`＄`始まりのため既存推論が有効）
- ❌ `element_gen.rs`のバッチ最適化が`generate_action_line()`の構造変更を要求

### Option B: 新規ASTノード（PropertyRef / PropertySet）

`Action::PropertyRef`と`PropertySet`を`VarRef`/`VarSet`とは独立した新型として定義。

**トレードオフ**:
- ✅ プロパティ固有のセマンティクスを型で表現（name, path区分等）
- ❌ ASTノード増加、既存のmatch網羅性チェックが活かせない
- ❌ `Expr::VarRef`との二重管理が必要
- ❌ コード量が大幅増加

### Option C: ハイブリッド

パーサー層はOption A（VarScope拡張）、トランスパイラ層のバッチ最適化のみ新規ヘルパー関数追加。

**トレードオフ**:
- ✅ パーサーは最小変更、トランスパイラのみ新ロジック
- ✅ バッチ最適化を独立関数として分離可能
- ❌ 若干の設計判断が必要（プリフェッチ関数の配置）

## 4. 技術的課題と設計判断ポイント

### 4.1 プロパティ名ID文法 (Research Needed)

要件R1は`[a-zA-Z][_().a-zA-Z0-9]*`を要求。既存`id`ルール（Unicode XID_START + XID_CONTINUE）とは異なる専用ルールが必要。

**設計判断**:
- 全角`（）．`も許容するか？（DSLは全角/半角両対応が基本方針）
- `property_id`を独立ルールにするか、既存`id`ルールに制約を追加するか？

### 4.2 SET (`＄％prop＝value`) のコード生成パターン

既存の`generate_var_set()`は`var_path = expr`形式を出力するが、SETは`act:set_property("name", expr)`形式。

**選択肢**:
- A: `generate_var_set()`内でProperty分岐し、異なるコード生成パスを通す
- B: `generate_var_set()`の前段で`VarScope::Property`を検出し、別関数にディスパッチ

### 4.3 GET代入 (`＄var＝＄％prop`) の非同期処理

`get_property()`はコルーチンyieldを伴う。`＄var＝＄％prop`の右辺に`Expr::VarRef { scope: Property }`が出現した場合:

**生成パターン案**:
```lua
-- ＄var＝＄％currentghost.name
local __prop_1 = act:get_property("currentghost.name")
var.name = __prop_1
```

`generate_var_set()`でProperty式を検出し、プリフェッチコードを先行出力する必要あり。

### 4.4 ~~インラインGET バッチ最適化~~ （スコープ外に決定）

トークンバッファ保全改修（4.6）により、`get_property()`を逐次呼び出してもテキスト分断は発生しない。バッチ最適化（複数プロパティの一括取得によるyield回数削減）は**パフォーマンス最適化**に過ぎず、正確性には影響しない。

**決定**: 本specではバッチ最適化を実装しない。トランスパイラは各`＄％`参照に対して個別に`act:get_property()`を生成する。後方互換を壊さず後付け可能なため、将来の最適化specで対応する。

**生成コード例（逐次方式）**:
```lua
-- さくら：名前は＄％currentghost.name、作者は＄％currentghost.craftman
act.さくら:talk("名前は")
local __prop_1 = act:get_property("currentghost.name")
act.さくら:talk(tostring(__prop_1))
act.さくら:talk("、作者は")
local __prop_2 = act:get_property("currentghost.craftman")
act.さくら:talk(tostring(__prop_2))
```

### 4.5 式中のプロパティ参照 (スコープ外だが考慮必要)

`＄var＝＄％a ＋ ＄％b` は要件R3のスコープ外（R3は`＄var＝＄％prop`のみ）。`generate_expr()`内で`VarScope::Property`が出現した場合のエラー処理は設計フェーズで決定。

### 4.6 `get_property()`のトークンバッファ保全（解決済み）

`get_property()`は内部で`self:build()`を呼び、**蓄積済みトークンをSSPに送信**してからyieldする。これはアクション行の途中でGETすると、GETより前のtalkトークンがその時点で配信される（「トークン汚染」）ことを意味する。

**決定**: `get_property()`内部でトークンバッファを退避・復元する方式を採用。

```lua
function SHIORI_ACT_IMPL.get_property(self, name_or_names, ...)
    -- 1. 既存トークンを退避
    local saved_tokens = self.token
    -- 2. 空のトークンバッファを作成し、getタグのみ登録
    self.token = {}
    table.insert(self.token, { type = "raw_script", text = tag })
    -- 3. getタグのみでyield
    local refs, reason = coroutine.yield(self:build())
    -- 4. 退避トークンを復元
    self.token = saved_tokens
    -- ...
end
```

**効果**: トランスパイラは呼び出し位置を意識せず素直に`act:get_property()`を生成できる。バッチ最適化はR4-AC4の「一度に配信」保証に引き続き有効（yield回数削減）。

## 5. 複雑度とリスク

**工数**: **M (3-7日)**
- パーサー拡張: S（既存パターンの機械的複製）
- SET トランスパイル: S（同期的、シンプル）
- GET代入 トランスパイル: S-M（トークン保全改修により素直に呼べる）
- GETインライン: S（トークン保全により逐次呼び出しで十分、バッチ最適化不要）
- get_propertyトークン保全: S（Lua内部改修のみ）
- テスト: S-M（パーサー + トランスパイラ + トークン保全）

**リスク**: **Medium**
- パーサー拡張は前例豊富（`Args`、cue-dsl-extension等）で低リスク
- バッチ最適化は既存`generate_action_line()`の構造変更を伴うが、影響範囲は限定的
- `get_property()`のyield/resumeセマンティクスは既存APIで確立済み
- プロパティ名文法の全角/半角対応は設計判断が必要（Medium）

## 6. 設計フェーズへの推奨事項

### 推奨アプローチ
**Option A（既存コンポーネント拡張）** + バッチ最適化ヘルパー

### 設計フェーズでの決定事項
1. `property_id` Pestルールの全角/半角対応方針
2. SET `generate_var_set()`内の分岐 vs 別関数ディスパッチ
3. GET代入のコード生成パターン（ローカル変数命名規則）
4. 式中`VarScope::Property`のエラーメッセージ（TranspileError拡張）
