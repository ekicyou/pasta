# 5. リテラル型

## 5.1 概要

変数、関数引数、属性値で使用可能な型。

## 5.2 型変換ルール

リテラル値は以下の優先順位で型変換される：

1. **bool型**: `true` / `false` → bool
2. **String型（引用符あり）**: `「...」`で囲まれている → String
3. **f64型**: 小数点が含まれる数値 → f64
4. **i64型**: 小数点が無い数値 → i64
5. **String型（その他）**: 上記以外 → String

**空白の扱い**:
- 引用符`「」`で囲まれている場合: 空白も文字列の一部
- 引用符なしの場合: 空白は区切り文字として認識され、文字列に含まれない

**例**:
```
true           → bool
false          → bool
「こんにちは」   → String (空白含む)
Hello          → String
3.14           → f64
42             → i64
hello world    → String "hello" と String "world" (2つの値)
「hello world」 → String "hello world" (1つの値)
```

---

**関連章**:
- [Chapter 2: キーワード・マーカー定義](02-markers.md) - リテラル・文字列の構文
- [Chapter 8: 属性](08-attributes.md) - 属性値の型解釈
- [Chapter 9: 変数・スコープ](09-variables.md) - 変数代入
