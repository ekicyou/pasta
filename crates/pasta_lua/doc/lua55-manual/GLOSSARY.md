[← 目次](./README.md)

---

# Lua 5.5 リファレンスマニュアル用語対応表 (GLOSSARY)

> バージョン: v0 (初版)
> 作成日: 2026-01-29
> 更新日: 2026-01-29
> 目的: 翻訳時の用語統一を保証するための対応表

## 使用方法

翻訳時は以下の用語を参照し、同一概念に対して一貫した訳語を使用すること。

---

## 基本型 (Basic Types)

| English | 日本語 | 備考 |
|---------|--------|------|
| nil | nil | 原文維持（型名） |
| boolean | boolean | 原文維持（型名） |
| number | number | 原文維持（型名） |
| string | string | 原文維持（型名） |
| function | function | 原文維持（型名） |
| userdata | userdata | 原文維持（型名） |
| thread | thread | 原文維持（型名） |
| table | テーブル | 説明文中では日本語 |
| integer | 整数 / integer | 文脈による |
| float | 浮動小数点 / float | 文脈による |
| full userdata | フルuserdata | |
| light userdata | ライトuserdata | |

---

## 値とオブジェクト (Values and Objects)

| English | 日本語 | 備考 |
|---------|--------|------|
| value | 値 | |
| object | オブジェクト | |
| first-class value | 第一級の値 | |
| false values | 偽の値 | nil と false のこと |
| dynamically typed | 動的型付け | |
| immutable | 不変 | |
| 8-bit clean | 8ビットクリーン | |
| encoding-agnostic | エンコーディングに依存しない | |
| reference | 参照 | |

---

## メタテーブルとメタメソッド (Metatables and Metamethods)

| English | 日本語 | 備考 |
|---------|--------|------|
| metatable | メタテーブル | |
| metamethod | メタメソッド | |
| metavalue | メタ値 | |
| raw access | 生のアクセス | |
| raw equality | 生の等価性 | |
| rawget | rawget | 原文維持（関数名） |
| rawset | rawset | 原文維持（関数名） |
| callable value | 呼び出し可能な値 | |

---

## 環境 (Environments)

| English | 日本語 | 備考 |
|---------|--------|------|
| environment | 環境 | |
| global environment | グローバル環境 | |
| global variable | グローバル変数 | |
| free name | 自由名 | 宣言にバインドされていない名前 |
| upvalue | アップバリュー | |
| chunk | チャンク | |
| scope | スコープ | 5.5で重要概念 |
| visibility rules | 可視性ルール | |

---

## ガベージコレクション (Garbage Collection)

| English | 日本語 | 備考 |
|---------|--------|------|
| garbage collection | ガベージコレクション | |
| garbage collector | ガベージコレクタ | |
| dead object | デッドオブジェクト | |
| finalizer | ファイナライザ | |
| mark for finalization | ファイナライズ対象としてマーク | |
| resurrection | 復活 | |
| incremental mode | インクリメンタルモード | |
| generational mode | ジェネレーショナルモード | |
| minor collection | マイナーコレクション | |
| major collection | メジャーコレクション | |
| mark-and-sweep | マークアンドスイープ | |
| weak table | ウィークテーブル | |
| weak reference | ウィーク参照 | |
| weak keys | ウィークキー | |
| weak values | ウィーク値 | |
| ephemeron table | エフェメロンテーブル | |
| pause | 一時停止 | GCパラメータ |
| step multiplier | ステップ乗数 | GCパラメータ |
| step size | ステップサイズ | GCパラメータ |

---

## エラー処理 (Error Handling)

| English | 日本語 | 備考 |
|---------|--------|------|
| error | エラー | |
| raise an error | エラーを発生させる | |
| catch an error | エラーをキャッチする | |
| error object | エラーオブジェクト | |
| error message | エラーメッセージ | |
| protected call | 保護された呼び出し | |
| protected mode | 保護モード | |
| message handler | メッセージハンドラ | |
| stack traceback | スタックトレースバック | |
| warning | 警告 | |

---

## コルーチン (Coroutines)

| English | 日本語 | 備考 |
|---------|--------|------|
| coroutine | コルーチン | |
| thread | スレッド | コルーチンのハンドル |
| cooperative multithreading | 協調的マルチスレッディング | |
| resume | 再開 / resume | 動詞は「再開する」 |
| yield | yield / 中断 | 動詞は「yieldする」 |
| main function | メイン関数 | コルーチンの |
| dead coroutine | デッドコルーチン | |

---

## 言語構文 (Language Syntax)

| English | 日本語 | 備考 |
|---------|--------|------|
| lexical conventions | 字句規則 | |
| token | トークン | |
| identifier | 識別子 | |
| keyword | キーワード | |
| reserved word | 予約語 | |
| literal | リテラル | |
| comment | コメント | |
| long bracket | 長い括弧 | |
| statement | ステートメント | |
| expression | 式 | |
| block | ブロック | |
| chunk | チャンク | |
| assignment | 割り当て | |
| multiple assignment | 多重代入 | |
| local variable | ローカル変数 | |
| to-be-closed variable | クローズされる変数 | 5.4以降 |
| constant variable | 定数ローカル変数 | 5.4以降 |

---

## 予約語 (Reserved Words) - 原文維持

以下の予約語は翻訳せず原文のまま使用：

```
and       break     do        else      elseif    end
false     for       function  global    goto      if
in        local     nil       not       or        repeat
return    then      true      until     while
```

※ `global` は Lua 5.5 で新規追加

---

## C API基本用語 (C API Basic Terms)

| English | 日本語 | 備考 |
|---------|--------|------|
| stack | スタック | |
| index | インデックス | |
| pseudo-index | 疑似インデックス | |
| valid index | 有効なインデックス | |
| acceptable index | 許容可能なインデックス | |
| registry | レジストリ | |
| C closure | Cクロージャ | |
| Lua state | Lua状態 | |
| API function | API関数 | |
| library function | ライブラリ関数 | |

---

## C API関数名 - 原文維持

すべての `lua_*` および `luaL_*` 関数名は原文維持：

例: `lua_pushstring`, `lua_gettable`, `luaL_checkinteger`

---

## 標準ライブラリ (Standard Libraries)

| English | 日本語 | 備考 |
|---------|--------|------|
| basic library | 基本ライブラリ | |
| coroutine library | コルーチンライブラリ | |
| package library | パッケージライブラリ | |
| string library | 文字列ライブラリ | |
| UTF-8 library | UTF-8ライブラリ | |
| table library | テーブルライブラリ | |
| math library | 数学ライブラリ | |
| I/O library | I/Oライブラリ | |
| OS library | OSライブラリ | |
| debug library | デバッグライブラリ | |
| pattern matching | パターンマッチング | |
| capture | キャプチャ | パターンの |
| format string | フォーマット文字列 | |
| file handle | ファイルハンドル | |
| iterator | イテレータ | |

---

## 標準ライブラリ関数名 - 原文維持

すべての標準ライブラリ関数名は原文維持：

例: `print`, `type`, `pairs`, `ipairs`, `next`, `select`, `tonumber`, `tostring`

例: `string.format`, `table.insert`, `math.floor`, `io.open`, `os.time`

例: `coroutine.create`, `coroutine.resume`, `coroutine.yield`

---

## その他の技術用語 (Other Technical Terms)

| English | 日本語 | 備考 |
|---------|--------|------|
| syntactic sugar | 糖衣構文 | |
| bytecode | バイトコード | |
| virtual machine | 仮想マシン | |
| host program | ホストプログラム | |
| embedding program | 埋め込みプログラム | |
| extension language | 拡張言語 | |
| stand-alone | スタンドアロン | |
| interpreter | インタープリター | |
| compiler | コンパイラ | |
| binary chunk | バイナリチャンク | |
| library | ライブラリ | |
| module | モジュール | |
| searcher | サーチャー | パッケージの |
| loader | ローダー | パッケージの |
| hook | フック | デバッグの |
| breakpoint | ブレークポイント | |

---

## 更新履歴

| バージョン | 日付 | 変更内容 |
|-----------|------|---------|
| v0 | 2026-01-29 | 初版作成（約80語） |

---

## 注意事項

1. **原文維持の原則**: API名・関数名・予約語・型名はすべて原文のまま使用
2. **一貫性**: 同一概念には常に同一の訳語を使用
3. **文脈依存**: 一部の用語は文脈により訳し分ける場合あり（備考欄参照）
4. **追加更新**: Phase 1翻訳中に新規用語を発見した場合は本表に追加
5. **5.5新機能**: `global`キーワード、スコープ関連の新概念に注意
