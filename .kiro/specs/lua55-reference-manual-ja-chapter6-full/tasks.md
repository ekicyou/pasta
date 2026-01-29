# Implementation Plan

## Overview

Lua 5.5リファレンスマニュアル日本語版の第6章「標準ライブラリ」を完全版で再作成。**超細粒度ファイルベース翻訳**により、各タスクで確実にファイル出力。

**タスク数**: 35タスク（準備1 + 翻訳29 + 統合3 + 置換2）

---

## Tasks

### Phase 1: 準備

- [x] 1. translatedディレクトリ作成
  - `.kiro/specs/lua55-reference-manual-ja-chapter6-full/translated/` 作成
  - _成果物: translated/ フォルダ_

---

### Phase 2: 超細粒度翻訳（29タスク・並列可能）

#### §6.1 Cコードでのライブラリロード（1タスク）

- [x] 2. §6.1 Cコードでのライブラリロード翻訳
  - 入力: `01-loading-the-libraries-in-c-code.html`
  - 関数: luaL_openlibs, luaL_openselectedlibs + ライブラリ定数テーブル
  - _成果物: `translated/01-loading-libraries.md`_

---

#### §6.2 基本関数（5タスク）

- [x] 3. §6.2-A 基本関数 assert-error
  - 入力: `02-basic-functions.html` 該当部分
  - 関数: assert, collectgarbage, dofile, error
  - collectgarbageの全オプション完全解説必須
  - _成果物: `translated/02-basic-A.md`_

- [x] 4. §6.2-B 基本関数 _G-loadfile
  - 入力: `02-basic-functions.html` 該当部分
  - 関数: _G, getmetatable, ipairs, load, loadfile
  - _成果物: `translated/02-basic-B.md`_

- [x] 5. §6.2-C 基本関数 next-rawset
  - 入力: `02-basic-functions.html` 該当部分
  - 関数: next, pairs, pcall, print, rawequal, rawget, rawlen, rawset
  - _成果物: `translated/02-basic-C.md`_

- [x] 6. §6.2-D 基本関数 select-type
  - 入力: `02-basic-functions.html` 該当部分
  - 関数: select, setmetatable, tonumber, tostring, type
  - _成果物: `translated/02-basic-D.md`_

- [x] 7. §6.2-E 基本関数 _VERSION-xpcall
  - 入力: `02-basic-functions.html` 該当部分
  - 関数: _VERSION, warn, xpcall
  - _成果物: `translated/02-basic-E.md`_

---

#### §6.3 コルーチン操作（1タスク）

- [x] 8. §6.3 コルーチン操作翻訳
  - 入力: `03-coroutine-manipulation.html`
  - 関数: coroutine.close, coroutine.create, coroutine.isyieldable, coroutine.resume, coroutine.running, coroutine.status, coroutine.wrap, coroutine.yield
  - _成果物: `translated/03-coroutine.md`_

---

#### §6.4 モジュール（2タスク）

- [x] 9. §6.4-A モジュール 前半
  - 入力: `04-modules.html` 該当部分
  - 関数: require, package.config, package.cpath, package.loaded, package.loadlib
  - requireのローダーメカニズム詳細必須
  - _成果物: `translated/04-modules-A.md`_

- [x] 10. §6.4-B モジュール 後半
  - 入力: `04-modules.html` 該当部分
  - 関数: package.path, package.preload, package.searchers, package.searchpath
  - _成果物: `translated/04-modules-B.md`_

---

#### §6.5 文字列操作（4タスク）

- [x] 11. §6.5-A 文字列 byte-find
  - 入力: `05-string-manipulation.html` 該当部分
  - 関数: string.byte, string.char, string.dump, string.find
  - _成果物: `translated/05-string-A.md`_

- [x] 12. §6.5-B 文字列 format-len
  - 入力: `05-string-manipulation.html` 該当部分
  - 関数: string.format, string.gmatch, string.gsub, string.len
  - string.formatの全書式指定子完全解説必須
  - _成果物: `translated/05-string-B.md`_

- [x] 13. §6.5-C 文字列 lower-rep
  - 入力: `05-string-manipulation.html` 該当部分
  - 関数: string.lower, string.match, string.pack, string.packsize, string.rep
  - _成果物: `translated/05-string-C.md`_

- [x] 14. §6.5-D 文字列 reverse-upper + パターン + Pack書式
  - 入力: `05-string-manipulation.html` 該当部分
  - 関数: string.reverse, string.sub, string.unpack, string.upper
  - §6.5.1 パターンマッチング完全解説（文字クラス、マジック文字、繰り返し、キャプチャ）
  - §6.5.2 Pack/Unpack書式文字列完全解説
  - _成果物: `translated/05-string-D.md`_

---

#### §6.6 UTF-8サポート（1タスク）

- [x] 15. §6.6 UTF-8サポート翻訳
  - 入力: `06-utf-8-support.html`
  - 関数: utf8.char, utf8.charpattern, utf8.codes, utf8.codepoint, utf8.len, utf8.offset
  - _成果物: `translated/06-utf8.md`_

---

#### §6.7 テーブル操作（1タスク）

- [x] 16. §6.7 テーブル操作翻訳
  - 入力: `07-table-manipulation.html`
  - 関数: table.concat, table.create, table.insert, table.move, table.pack, table.remove, table.sort, table.unpack
  - _成果物: `translated/07-table.md`_

---

#### §6.8 数学関数（5タスク）

- [x] 17. §6.8-A 数学 abs-cos
  - 入力: `08-mathematical-functions.html` 該当部分
  - 関数: math.abs, math.acos, math.asin, math.atan, math.ceil, math.cos
  - _成果物: `translated/08-math-A.md`_

- [x] 18. §6.8-B 数学 deg-huge
  - 入力: `08-mathematical-functions.html` 該当部分
  - 関数: math.deg, math.exp, math.floor, math.fmod, math.frexp, math.huge
  - _成果物: `translated/08-math-B.md`_

- [x] 19. §6.8-C 数学 ldexp-mininteger
  - 入力: `08-mathematical-functions.html` 該当部分
  - 関数: math.ldexp, math.log, math.max, math.maxinteger, math.min, math.mininteger
  - _成果物: `translated/08-math-C.md`_

- [x] 20. §6.8-D 数学 modf-randomseed
  - 入力: `08-mathematical-functions.html` 該当部分
  - 関数: math.modf, math.pi, math.rad, math.random, math.randomseed
  - _成果物: `translated/08-math-D.md`_

- [x] 21. §6.8-E 数学 sin-ult
  - 入力: `08-mathematical-functions.html` 該当部分
  - 関数: math.sin, math.sqrt, math.tan, math.tointeger, math.type, math.ult
  - _成果物: `translated/08-math-E.md`_

---

#### §6.9 入出力機能（4タスク）

- [x] 22. §6.9-A 入出力 io.close-io.lines
  - 入力: `09-input-and-output-facilities.html` 該当部分
  - 関数: io.close, io.flush, io.input, io.lines
  - 入出力モデルの説明（暗黙の入力/出力ファイル）を含む
  - _成果物: `translated/09-io-A.md`_

- [x] 23. §6.9-B 入出力 io.open-io.read
  - 入力: `09-input-and-output-facilities.html` 該当部分
  - 関数: io.open, io.output, io.popen, io.read
  - io.openのモード、io.readの書式完全解説
  - _成果物: `translated/09-io-B.md`_

- [x] 24. §6.9-C 入出力 io.stderr-io.write
  - 入力: `09-input-and-output-facilities.html` 該当部分
  - 関数: io.stderr, io.stdin, io.stdout, io.tmpfile, io.type, io.write
  - _成果物: `translated/09-io-C.md`_

- [x] 25. §6.9-D 入出力 file:メソッド
  - 入力: `09-input-and-output-facilities.html` 該当部分
  - 関数: file:close, file:flush, file:lines, file:read, file:seek, file:setvbuf, file:write
  - _成果物: `translated/09-io-D.md`_

---

#### §6.10 オペレーティングシステム機能（2タスク）

- [x] 26. §6.10-A OS clock-exit
  - 入力: `10-operating-system-facilities.html` 該当部分
  - 関数: os.clock, os.date, os.difftime, os.execute, os.exit
  - os.dateの書式指定子完全解説必須
  - _成果物: `translated/10-os-A.md`_

- [x] 27. §6.10-B OS getenv-tmpname
  - 入力: `10-operating-system-facilities.html` 該当部分
  - 関数: os.getenv, os.remove, os.rename, os.setlocale, os.time, os.tmpname
  - _成果物: `translated/10-os-B.md`_

---

#### §6.11 デバッグライブラリ（3タスク）

- [x] 28. §6.11-A デバッグ debug-getlocal
  - 入力: `11-the-debug-library.html` 該当部分
  - 関数: debug.debug, debug.gethook, debug.getinfo, debug.getlocal
  - debug.getinfoのwhat文字列完全解説必須
  - _成果物: `translated/11-debug-A.md`_

- [x] 29. §6.11-B デバッグ getmetatable-setlocal
  - 入力: `11-the-debug-library.html` 該当部分
  - 関数: debug.getmetatable, debug.getregistry, debug.getupvalue, debug.getuservalue, debug.sethook, debug.setlocal
  - _成果物: `translated/11-debug-B.md`_

- [x] 30. §6.11-C デバッグ setmetatable-upvaluejoin
  - 入力: `11-the-debug-library.html` 該当部分
  - 関数: debug.setmetatable, debug.setupvalue, debug.setuservalue, debug.traceback, debug.upvalueid, debug.upvaluejoin
  - デバッグライブラリ使用上の注意事項を含む
  - _成果物: `translated/11-debug-C.md`_

---

### Phase 3: 統合（3タスク）

- [x] 31. 章導入文翻訳
  - 入力: `06-standard-libraries.html` 冒頭部分
  - 章タイトル「6 – 標準ライブラリ」
  - 標準ライブラリ概要、ライブラリ一覧、**fail**表記説明
  - _成果物: `translated/00-chapter-intro.md`_

- [x] 32. 全ファイル統合
  - 入力: `translated/` 内の30ファイル（00-chapter-intro.md + 29翻訳ファイル）
  - ファイルヘッダー追加（Source, Translation, Glossary参照）
  - Lua 5.5変更点まとめを末尾に追加
  - **出力は別名**: `06-standard-libraries.new.md`
  - _成果物: `crates/pasta_lua/doc/lua55-manual/06-standard-libraries.new.md`_

- [x] 33. 統合ファイル検証
  - 検証項目:
    - ファイル存在確認: ✅
    - 1668行（完全版）: ✅
    - §6.1-§6.11 全11セクション存在確認: ✅
    - 要約・省略表現がないこと: ✅ (検出された「省略」は原文の正当な翻訳)
  - _成果物: 検証レポート（チャット報告）_

---

### Phase 4: 安全な置換（2タスク）

- [x] 34. 旧ファイル削除
  - `crates/pasta_lua/doc/lua55-manual/06-standard-libraries.md` を削除 (404行 → 削除済)
  - **前提条件**: タスク33の検証成功 ✅
  - _成果物: 旧ファイル削除完了_

- [x] 35. 新ファイルリネーム
  - `06-standard-libraries.new.md` → `06-standard-libraries.md`
  - 最終確認、git commit
  - _成果物: `06-standard-libraries.md`（完全版・1668行・110KB）_

---

## 並列実行ガイド

### Phase 2 並列グループ

以下のタスクは**依存関係なし**、並列実行可能：

| グループ | タスク |
|----------|--------|
| G1 | 2, 3, 4, 5, 6, 7 |
| G2 | 8, 9, 10, 11, 12, 13, 14 |
| G3 | 15, 16, 17, 18, 19, 20, 21 |
| G4 | 22, 23, 24, 25, 26, 27 |
| G5 | 28, 29, 30 |

### 依存関係チェーン

```
[1] → [2-30] → [31] → [32] → [33] → [34] → [35]
       ↑並列可能↑
```

---

## 完了基準（DoD）

1. **File Gate**: 30個の翻訳ファイルが`translated/`に存在
2. **Merge Gate**: 統合ファイルが2000行以上
3. **Section Gate**: §6.1-§6.11 全11セクション含む
4. **Replace Gate**: 最終ファイルが正しい場所に配置
5. **Translation Gate**: 要約・省略表現ゼロ

---

## タスク完了報告テンプレート

```
✅ タスク X 完了
- 出力: translated/XX-name.md
- 行数: XXX行
- 関数: func1, func2, func3
```

---

## 参照パス

### 英語原文
- `.kiro/specs/completed/lua55-reference-manual-ja/chapters/en/standard-libraries/` (11ファイル)
- `.kiro/specs/completed/lua55-reference-manual-ja/chapters/en/06-standard-libraries.html` (章導入文用)

### 中間出力先
- `.kiro/specs/lua55-reference-manual-ja-chapter6-full/translated/`

### 最終出力先
- `crates/pasta_lua/doc/lua55-manual/06-standard-libraries.md`
