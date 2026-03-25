# 1. 文法モデルの基本原則

## 1.1 行指向文法

Pasta スクリプトは**行指向文法**です。行頭の数文字により行属性が確定します。

**例外**:
- Lua コードブロック：複数行にわたりコードブロックを形成する唯一の例外

## 1.2 ファイル構造（俯瞰）

```text
ファイル
├─ グローバルシーン (＊ 或いは *)
│  ├─ 単語定義行 (＠)
│  └─ Lua コードブロック
├─ グローバル単語定義 (＠)
└─ コメント行 (＃)
```

## 1.3 式（Expression）のサポート

Pasta DSL では**式（Expression）を記述できます**。式は変数代入、関数引数、条件式などで使用されます。

### 式の構文

| 要素     | pasta2.pest規則                                                                   | 説明                                     |
| -------- | --------------------------------------------------------------------------------- | ---------------------------------------- |
| 式       | `expr = { term ~ s ~ bin* }`                                                      | 項と二項演算の組み合わせ                 |
| 項       | `term = { paren_expr \| fn_call \| var_ref \| number_literal \| string_literal }` | 括弧式、関数呼び出し、変数参照、リテラル |
| 二項演算 | `bin = { bin_op ~ s ~ term ~ s }`                                                 | 演算子と右辺項                           |
| 演算子   | `bin_op = { add_op \| sub_op \| mul_op \| div_op \| modulo_op }`                  | 算術演算子                               |

### 対応演算子

| 種別 | 演算子（全角/半角） |
| ---- | ------------------- |
| 加算 | `+` / `＋`          |
| 減算 | `-` / `－`          |
| 乗算 | `*` / `＊` / `×`    |
| 除算 | `/` / `／` / `÷`    |
| 剰余 | `%` / `％`          |

### 使用例

```pasta
＄count＝10 + 5          # 算術式
＄result＝＄a * ＄b       # 変数を含む式
＠func（＄x + 1）         # 関数引数での式
＄nested＝（＄a + ＄b）* 2  # 括弧による優先順位制御
＄＝＠副作用関数（）       # 式文: 結果を代入せず式を評価のみ
```

### 式文（ExprStmt）

`＄＝expr` 形式で、代入を伴わない式の評価（式文）を記述できます。

| 要素 | pasta2.pest規則                         | 説明                           |
| ---- | --------------------------------------- | ------------------------------ |
| 式文 | `var_set_none = { var_marker ~ set }`   | 変数名を省略し結果を捨てる式文 |

`var_set` は以下の3形式があります：

```pest
var_set        =_{ var_set_global | var_set_local | var_set_none }
var_set_local  = { var_marker ~                 id ~ s ~ set }
var_set_global = { var_marker ~ global_marker ~ id ~ s ~ set }
var_set_none   = { var_marker ~                            set }
set            =_{ set_marker ~ s ~ ( expr | word_ref ) }
```

全バリアントが `＄`（`var_marker`）で始まり、LSP/TextMate で変数操作のシグナルとして一貫して認識できます。

### 複雑な演算

より複雑な演算や条件判定が必要な場合は、Lua ブロックで関数を定義することも可能です。

````pasta
```lua
function SCENE.calculate(act)
    local save, var = act:init_scene(SCENE)
    local result = 10 + 20 * 3
    return result
end
```
＄result＝＠calculate()
````

---

**関連章**:
- [Chapter 2: キーワード・マーカー定義](02-markers.md) - マーカーの詳細
- [Chapter 3: 行とブロック構造](03-block-structure.md) - ブロック構造の詳細
