[← 前へ: 8 – 非互換性](08-incompatibilities.md) | [目次](./README.md)

---

<!--
  原文: https://www.lua.org/manual/5.5/manual.html#9
  参考: https://lua.dokyumento.jp/manual/5.4/manual.html#9
  翻訳日: 2026-01-29
  レビュー: AI Claude Opus 4.5
  用語対照: GLOSSARY.md参照
-->

# 9 – Luaの完全な構文

---

ここに、拡張BNFによるLuaの完全な構文を示します。拡張BNFの通例通り、`{A}`は0個以上のA、`[A]`はオプションのAを意味します。（演算子の優先順位については§3.4.8を、終端記号Name、Numeral、およびLiteralStringの説明については§3.1を参照してください。）

## 構文定義

```bnf
chunk ::= block

block ::= {stat} [retstat]

stat ::=  ';' | 
         varlist '=' explist | 
         functioncall | 
         label | 
         break | 
         goto Name | 
         do block end | 
         while exp do block end | 
         repeat block until exp | 
         if exp then block {elseif exp then block} [else block] end | 
         for Name '=' exp ',' exp [',' exp] do block end | 
         for namelist in explist do block end | 
         function funcname funcbody | 
         local function Name funcbody | 
         global function Name funcbody | 
         local attnamelist ['=' explist] | 
         global attnamelist | 
         global [attrib] '*' 

attnamelist ::=  [attrib] Name [attrib] {',' Name [attrib]}

attrib ::= '<' Name '>'

retstat ::= return [explist] [';']

label ::= '::' Name '::'

funcname ::= Name {'.' Name} [':' Name]

varlist ::= var {',' var}

var ::=  Name | prefixexp '[' exp ']' | prefixexp '.' Name 

namelist ::= Name {',' Name}

explist ::= exp {',' exp}

exp ::=  nil | false | true | Numeral | LiteralString | '...' | functiondef | 
         prefixexp | tableconstructor | exp binop exp | unop exp 

prefixexp ::= var | functioncall | '(' exp ')'

functioncall ::=  prefixexp args | prefixexp ':' Name args 

args ::=  '(' [explist] ')' | tableconstructor | LiteralString 

functiondef ::= function funcbody

funcbody ::= '(' [parlist] ')' block end

parlist ::= namelist [',' varargparam] | varargparam

varargparam ::= '...' [Name]

tableconstructor ::= '{' [fieldlist] '}'

fieldlist ::= field {fieldsep field} [fieldsep]

field ::= '[' exp ']' '=' exp | Name '=' exp | exp

fieldsep ::= ',' | ';'

binop ::=  '+' | '-' | '*' | '/' | '//' | '^' | '%' | 
         '&' | '~' | '|' | '>>' | '<<' | '..' | 
         '<' | '<=' | '>' | '>=' | '==' | '~=' | 
         and | or

unop ::= '-' | not | '#' | '~'
```

---

## Lua 5.5 での構文追加

### global 宣言関連

Lua 5.5では、`global`キーワードに関連する新しい構文が追加されました：

```bnf
stat ::= ... | 
         global function Name funcbody |    -- global関数定義
         global attnamelist |               -- global変数宣言
         global [attrib] '*'                -- 集合global宣言
```

#### global function 構文

グローバル関数を明示的に宣言します：

```lua
global function myFunc()
    -- 関数本体
end
```

これは以下と同等です：

```lua
global myFunc
function myFunc()
    -- 関数本体
end
```

#### global 変数宣言

グローバル変数を明示的に宣言します：

```lua
global x, y, z           -- グローバル変数の宣言
global<const> PI = 3.14  -- 定数属性付きグローバル変数
```

#### 集合 global 宣言

スコープ内のすべての自由名をグローバルとして扱います：

```lua
global *                 -- すべての自由名をグローバルとして扱う
global<const> *          -- すべての自由名を定数グローバルとして扱う
```

### varargparam の変更

可変長引数パラメータに名前を付けられるようになりました：

```bnf
varargparam ::= '...' [Name]
```

例：
```lua
function f(... args)
    -- argsはvararg tableとして利用可能
    for i, v in ipairs(args) do
        print(v)
    end
end
```

---

## 構文要素の説明

### ステートメント (stat)

| 構文 | 説明 |
|------|------|
| `;` | 空文 |
| `varlist = explist` | 代入 |
| `functioncall` | 関数呼び出し |
| `label` | ラベル |
| `break` | ループ脱出 |
| `goto Name` | ジャンプ |
| `do block end` | ブロック |
| `while exp do block end` | whileループ |
| `repeat block until exp` | repeatループ |
| `if exp then block ... end` | 条件分岐 |
| `for Name = exp, exp [, exp] do block end` | 数値forループ |
| `for namelist in explist do block end` | ジェネリックforループ |
| `function funcname funcbody` | 関数定義 |
| `local function Name funcbody` | ローカル関数定義 |
| `global function Name funcbody` | **[5.5新規]** グローバル関数定義 |
| `local attnamelist [= explist]` | ローカル変数宣言 |
| `global attnamelist` | **[5.5新規]** グローバル変数宣言 |
| `global [attrib] *` | **[5.5新規]** 集合グローバル宣言 |

### 属性 (attrib)

変数に付与できる属性：

| 属性 | 意味 |
|------|------|
| `<const>` | 定数（初期化後に変更不可） |
| `<close>` | to-be-closed変数（スコープ終了時に`__close`が呼ばれる） |

### 二項演算子 (binop)

| カテゴリ | 演算子 |
|----------|--------|
| 算術 | `+` `-` `*` `/` `//` `^` `%` |
| ビット | `&` `~` `|` `>>` `<<` |
| 連結 | `..` |
| 比較 | `<` `<=` `>` `>=` `==` `~=` |
| 論理 | `and` `or` |

### 単項演算子 (unop)

| 演算子 | 意味 |
|--------|------|
| `-` | 符号反転 |
| `not` | 論理否定 |
| `#` | 長さ |
| `~` | ビット否定 |

---

## 予約語一覧

Lua 5.5の予約語：

```
and       break     do        else      elseif    end
false     for       function  global    goto      if
in        local     nil       not       or        repeat
return    then      true      until     while
```

> **Lua 5.5 変更点**: `global`が予約語として追加されました。

---

## 終端記号

| 終端記号 | 説明 | 参照 |
|----------|------|------|
| `Name` | 識別子 | §3.1 |
| `Numeral` | 数値リテラル | §3.1 |
| `LiteralString` | 文字列リテラル | §3.1 |

---

> **Last update**: Mon Dec 15 21:02:05 UTC 2025
> **Revised for**: Lua 5.5.0
