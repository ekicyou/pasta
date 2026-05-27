# 9. 変数・スコープ

## 9.1 変数型

### グローバル変数

グローバル変数は永続的に有効な変数です。

| 構文 | pasta2.pest規則                                                     | 説明             |
| ---- | -------------------------------------------------------------------- | ---------------- |
| 参照 | `var_ref_global = { var_marker ~ global_marker ~ id ~ s }`          | `＄＊変数名`     |
| 代入 | `var_set_global = { var_marker ~ global_marker ~ id ~ s ~ set }`    | `＄＊変数名＝値` |

```pasta
宣言: ＄＊var_name＝value
参照: ＄＊var_name
```

**スコープ**: 永続的

### ローカル変数

ローカル変数は一連のシーンが終わるまで有効な変数です。

| 構文 | pasta2.pest規則                                  | 説明           |
| ---- | ------------------------------------------------ | -------------- |
| 参照 | `var_ref_local = { var_marker ~ id ~ s }`        | `＄変数名`     |
| 代入 | `var_set_local = { var_marker ~ id ~ s ~ set }`  | `＄変数名＝値` |

```pasta
宣言: ＄var_name＝value
参照: ＄var_name
```

**スコープ**: 一連のシーンが終わるまで

### プロパティ変数

プロパティ変数はSSP共有プロパティの読み書きを行う変数です。

| 構文 | pasta2.pest規則                                                              | 説明                  |
| ---- | ----------------------------------------------------------------------------- | --------------------- |
| 参照 | `var_ref_property = { var_marker ~ property_marker ~ property_id ~ s }`       | `＄％prop.path`       |
| 代入 | `var_set_property = { var_marker ~ property_marker ~ property_id ~ s ~ set }` | `＄％prop.path＝値`  |

```pasta
SET: ＄％sakura.name＝Alice       → act:set_property("sakura.name", "Alice")
GET: ＄name＝＄％sakura.name      → var.name = act:get_property("sakura.name")
インライン: さくら：＄％sakura.name  → act.さくら:talk(tostring(act:get_property("sakura.name")))
```

**スコープ**: SSP共有プロパティ（ベースウェア管理）

**プロパティIDの識別子規則**: `property_id` は通常の `id` と異なり、ドット（`.`）を含むことができる。これにより `sakura.name` や `baseware.version` などのSSPプロパティ名をそのまま記述できる。

### 使用例

```pasta
＄ローカル変数＝10       # ローカル変数代入（一連のシーンが終わるまで有効）
＄＊グローバル変数＝100  # グローバル変数代入（永続的）
Alice：値は ＄ローカル変数 です
Bob：値は ＄＊グローバル変数 です
```

## 9.2 変数代入の制約

変数代入では**式を使用できます**（[1.3](01-grammar-model.md#13-式expressionのサポート)参照）。代入可能な値は以下です：

### 許可される値の型

| 値の種類                 | 構文例                           | 説明                   |
| ------------------------ | -------------------------------- | ---------------------- |
| リテラル値               | `＄score＝100`                   | 数値、文字列リテラル   |
| 単語参照                       | `＄value＝＠word_name`           | 登録済み単語の参照                       |
| 変数参照                       | `＄new_var＝＄old_var`           | 他の変数の値をコピー                     |
| 式                             | `＄result＝＄a + ＄b * 2`        | 算術式                                   |
| ローカル関数呼び出し           | `＄result＝＠calculate()`        | `SCENE.calculate(act)` の戻り値          |
| グローバル関数呼び出し         | `＄result＝＠＊calculate()`      | `GLOBAL.calculate(act)` の戻り値         |
| 関数呼び出し（引数付き）       | `＄sum＝＠add（x：10　y：20）`   | 名前付き引数で呼び出し                   |

### 関数スコープの展開先

| DSL 構文         | Lua 展開先          | 説明                           |
| ---------------- | ------------------- | ------------------------------ |
| `＠func()`       | `SCENE.func(act)`   | ローカルスコープ（シーン内）   |
| `＠＊func()`     | `GLOBAL.func(act)`  | グローバルスコープ（永続領域） |

`＠＊` のグローバルスコープは変数 `＄＊` → `save.` との対称構造です。

### 式文（ExprStmt）

代入を伴わない関数呼び出し（副作用専用）を `＄＝expr` で記述できます。

| 構文 | pasta2.pest規則                     | 説明               |
| ---- | ----------------------------------- | ------------------ |
| 式文 | `var_set_none = { var_marker ~ set }` | `＄＝＠func()`     |

```pasta
＄＝＠副作用関数（）           # ローカル: SCENE.副作用関数(act)
＄＝＠＊グローバル副作用（）   # グローバル: GLOBAL.グローバル副作用(act)
```

`＄＝expr` は `var_set` の第3バリアントで、変数名を省略して式の結果を捨てます。
全ての `var_set` バリアント（`var_set_local`、`var_set_global`、`var_set_none`）が
`＄`（`var_marker`）で始まる一貫性を保ちます。

### 代替方法：Lua 関数を使用

````pasta
```lua
function add(ctx, x, y)
    return x + y
end

function is_greater(ctx, x, threshold)
    return x > threshold
end
```

＄result ： ＠add（x：10　y：20）
＄flag ： ＠is_greater（x：＠score　threshold：5）
````

---

**関連章**:
- [Chapter 1: 文法モデルの基本原則](01-grammar-model.md) - 式のサポート
- [Chapter 2: キーワード・マーカー定義](02-markers.md) - 変数マーカー
- [Chapter 5: リテラル型](05-literals.md) - 値の型変換
- [Chapter 6: アクション行](06-action-line.md) - 変数参照
