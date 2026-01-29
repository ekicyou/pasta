# Implementation Plan

## Overview
Lua 5.5リファレンスマニュアル日本語版の第6章「標準ライブラリ」を要約版から完全版に再作成する。原文HTML（約3,900行、11サブセクション）のすべての関数・パラメータ・説明を日本語に翻訳し、既存ファイルを置換する。

**タスク分割戦略**: モジュール規模に応じて1モジュール単位または関数グループ（5-10関数）単位で分割。大規模モジュール（§6.2, §6.5, §6.8, §6.9）は関数グループ単位で翻訳し、最終的に統合する。

---

## Tasks

### Phase 1: 小規模モジュール翻訳（1モジュール = 1タスク）

- [ ] 1. (P) §6.1 Cコードでのライブラリロード翻訳
  - `chapters/en/standard-libraries/01-loading-the-libraries-in-c-code.html`を読み込み、完全翻訳
  - `luaL_openlibs`と`luaL_openselectedlibs`の2関数を個別に完全説明
  - Lua 5.5新規セクションであることを明示
  - `luaL_openselectedlibs`がLua 5.5新規関数であることを注記
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.7, 2.1, 2.2, 3.1, 5.1, 5.2, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4_

- [ ] 2. (P) §6.3 コルーチン操作翻訳
  - `chapters/en/standard-libraries/03-coroutine-manipulation.html`を読み込み、完全翻訳
  - `coroutine.*`全8関数を個別に完全説明（create, resume, running, status, wrap, yield, isyieldable, close）
  - 各関数のパラメータ・戻り値・動作・例外を詳細記述
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 2.1, 2.2, 3.1, 3.2, 3.3, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4_

- [ ] 3. (P) §6.4 モジュール翻訳
  - `chapters/en/standard-libraries/04-modules.html`を読み込み、完全翻訳
  - `require`関数と`package.*`全関数・変数（10項目）を個別に完全説明
  - パッケージローディングメカニズムの詳細を完全翻訳
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 2.1, 2.2, 3.1, 3.2, 3.3, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4_

- [ ] 4. (P) §6.6 UTF-8サポート翻訳
  - `chapters/en/standard-libraries/06-utf-8-support.html`を読み込み、完全翻訳
  - `utf8.*`全6関数を個別に完全説明（char, charpattern, codes, codepoint, len, offset）
  - UTF-8エンコーディング処理の詳細を完全翻訳
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 2.1, 2.2, 3.1, 3.2, 3.3, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4_

- [ ] 5. (P) §6.7 テーブル操作翻訳
  - `chapters/en/standard-libraries/07-table-manipulation.html`を読み込み、完全翻訳
  - `table.*`全7関数を個別に完全説明（concat, insert, move, pack, remove, sort, unpack）
  - 各関数のパラメータ・戻り値・動作を詳細記述
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 2.1, 2.2, 3.1, 3.2, 3.3, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4_

- [ ] 6. (P) §6.10 オペレーティングシステム機能翻訳
  - `chapters/en/standard-libraries/10-operating-system-facilities.html`を読み込み、完全翻訳
  - `os.*`全11関数を個別に完全説明（clock, date, difftime, execute, exit, getenv, remove, rename, setlocale, time, tmpname）
  - 各関数のパラメータ・戻り値・動作・例外を詳細記述
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 2.1, 2.2, 3.1, 3.2, 3.3, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4_

- [ ] 7. (P) §6.11 デバッグライブラリ翻訳
  - `chapters/en/standard-libraries/11-the-debug-library.html`を読み込み、完全翻訳
  - `debug.*`全16関数を個別に完全説明（debug, gethook, getinfo, getlocal, getmetatable, getregistry, getupvalue, getuservalue, sethook, setlocal, setmetatable, setupvalue, setuservalue, traceback, upvalueid, upvaluejoin）
  - デバッグ機能の詳細と注意事項を完全翻訳
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 2.1, 2.2, 3.1, 3.2, 3.3, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4_

---

### Phase 2: §6.2 基本関数翻訳（関数グループ単位）

- [ ] 8. §6.2 基本関数グループ1翻訳
- [ ] 8.1 (P) assert, collectgarbage, dofile, error, _G
  - `chapters/en/standard-libraries/02-basic-functions.html`から対象5関数を読み込み、完全翻訳
  - `collectgarbage`は全オプション（collect, stop, restart, count, step, setpause, setstepmul, incremental, generational, param）を完全解説
  - Lua 5.5の`collectgarbage`変更点（"param"オプション新規）を明示
  - 各関数のパラメータ・戻り値・動作・例外を詳細記述
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 3.1, 3.2, 3.3, 5.3, 6.2, 6.3, 7.2, 7.3_

- [ ] 9. §6.2 基本関数グループ2翻訳
- [ ] 9.1 (P) getmetatable, ipairs, load, loadfile, next
  - `chapters/en/standard-libraries/02-basic-functions.html`から対象5関数を読み込み、完全翻訳
  - `load`と`loadfile`のチャンクローディングメカニズムを詳細解説
  - 各関数のパラメータ・戻り値・動作・例外を詳細記述
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 3.1, 3.2, 3.3, 6.2, 6.3, 7.2, 7.3_

- [ ] 10. §6.2 基本関数グループ3翻訳
- [ ] 10.1 (P) pairs, pcall, print, rawequal, rawget
  - `chapters/en/standard-libraries/02-basic-functions.html`から対象5関数を読み込み、完全翻訳
  - `pcall`のエラーハンドリング動作を詳細解説
  - 各関数のパラメータ・戻り値・動作・例外を詳細記述
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 3.1, 3.2, 3.3, 6.2, 6.3, 7.2, 7.3_

- [ ] 11. §6.2 基本関数グループ4翻訳
- [ ] 11.1 (P) rawlen, rawset, select, setmetatable, tonumber
  - `chapters/en/standard-libraries/02-basic-functions.html`から対象5関数を読み込み、完全翻訳
  - `setmetatable`のメタテーブル設定動作を詳細解説
  - 各関数のパラメータ・戻り値・動作・例外を詳細記述
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 3.1, 3.2, 3.3, 6.2, 6.3, 7.2, 7.3_

- [ ] 12. §6.2 基本関数グループ5翻訳
- [ ] 12.1 (P) tostring, type, _VERSION, warn, xpcall
  - `chapters/en/standard-libraries/02-basic-functions.html`から対象5関数を読み込み、完全翻訳
  - `xpcall`のエラーハンドリング動作を詳細解説
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 2.2, 3.1, 3.2, 3.3, 6.2, 6.3, 7.2, 7.3_

---

### Phase 3: §6.5 文字列操作翻訳（機能単位）

- [ ] 13. §6.5 文字列関数グループ翻訳
- [ ] 13.1 (P) byte, char, dump, find, format, gmatch, gsub, len, lower, match, packsize, rep, reverse, sub, unpack, upper
  - `chapters/en/standard-libraries/05-string-manipulation.html`から文字列操作関数16個を読み込み、完全翻訳
  - 各関数のパラメータ・戻り値・動作・例外を詳細記述
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 3.1, 3.2, 3.3, 6.2, 6.3, 7.2, 7.3_

- [ ] 14. §6.5.1 パターンマッチング構文翻訳
- [ ] 14.1 (P) パターンマッチング詳細解説
  - 文字クラス（%a, %c, %d, %g, %l, %p, %s, %u, %w, %x, %z）の全リストと説明
  - マジック文字（( ) . % + - * ? [ ^ $）の完全説明
  - 繰り返し演算子（*, +, -, ?）の動作詳細
  - キャプチャ（()）の使用方法と例
  - アンカー（^, $）の説明
  - 文字セット（[set], [^set]）の説明
  - パターンマッチングの実例を含む
  - _Requirements: 1.1, 1.2, 1.3, 1.7, 2.1, 4.1, 6.2, 6.3, 7.2, 7.3_

- [ ] 15. §6.5 string.format 書式指定子翻訳
- [ ] 15.1 (P) format書式指定子完全解説
  - `string.format`の全書式指定子（%c, %d, %e, %E, %f, %g, %G, %i, %o, %s, %u, %x, %X, %%）を完全解説
  - 幅・精度・フラグ指定の詳細
  - 各書式の動作と例を含む
  - _Requirements: 1.1, 1.2, 1.3, 1.7, 2.1, 4.2, 6.2, 6.3, 7.2, 7.3_

- [ ] 16. §6.5 string.pack/unpack 書式翻訳
- [ ] 16.1 (P) pack/unpack書式文字列完全解説
  - `string.pack`と`string.unpack`の全書式文字（b, B, h, H, l, L, j, J, T, i[n], I[n], f, d, n, c[n], z, s[n], x, X[n], <, >, =, !）を完全解説
  - エンディアン・アライメント・サイズ指定の詳細
  - 各書式の動作と例を含む
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.7, 2.1, 2.2, 4.3, 6.2, 6.3, 7.2, 7.3_

---

### Phase 4: §6.8 数学関数翻訳（関数グループ単位）

- [ ] 17. §6.8 数学関数グループ1翻訳
- [ ] 17.1 (P) abs, acos, asin, atan, ceil, cos, deg, exp, floor, fmod
  - `chapters/en/standard-libraries/08-mathematical-functions.html`から対象10関数を読み込み、完全翻訳
  - 各関数のパラメータ・戻り値・動作を詳細記述
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 3.1, 3.2, 3.3, 6.2, 6.3, 7.2, 7.3_

- [ ] 18. §6.8 数学関数グループ2翻訳
- [ ] 18.1 (P) log, max, maxinteger, min, mininteger, modf, pi, rad, random, randomseed
  - `chapters/en/standard-libraries/08-mathematical-functions.html`から対象10関数/定数を読み込み、完全翻訳
  - `random`と`randomseed`の乱数生成メカニズムを詳細解説
  - 各関数のパラメータ・戻り値・動作を詳細記述
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 3.1, 3.2, 3.3, 6.2, 6.3, 7.2, 7.3_

- [ ] 19. §6.8 数学関数グループ3翻訳
- [ ] 19.1 (P) sin, sqrt, tan, tointeger, type, ult, 残りの関数・定数
  - `chapters/en/standard-libraries/08-mathematical-functions.html`から残り関数/定数を読み込み、完全翻訳
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 2.2, 3.1, 3.2, 3.3, 6.2, 6.3, 7.2, 7.3_

---

### Phase 5: §6.9 入出力機能翻訳（機能単位）

- [ ] 20. §6.9 io.*関数翻訳
- [ ] 20.1 (P) close, flush, input, lines, open, output, popen, read, tmpfile, type, write
  - `chapters/en/standard-libraries/09-input-and-output-facilities.html`から`io.*`関数を読み込み、完全翻訳
  - ファイルハンドル取得・ストリーム操作の詳細を完全記述
  - 各関数のパラメータ・戻り値・動作・例外を詳細記述
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 3.1, 3.2, 3.3, 6.2, 6.3, 7.2, 7.3_

- [ ] 21. §6.9 ファイルハンドルメソッド翻訳
- [ ] 21.1 (P) file:close, file:flush, file:lines, file:read, file:seek, file:setvbuf, file:write
  - `chapters/en/standard-libraries/09-input-and-output-facilities.html`からファイルメソッドを読み込み、完全翻訳
  - ファイルハンドルメソッドの動作詳細を完全記述
  - `file:seek`のシークモード（set, cur, end）を完全解説
  - `file:setvbuf`のバッファモード（no, full, line）を完全解説
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.9, 1.10, 2.1, 3.1, 3.2, 3.3, 6.2, 6.3, 7.2, 7.3_

- [ ] 22. §6.9 入出力補足説明翻訳
- [ ] 22.1 (P) 入出力モデル・エラー処理の補足
  - 入出力モデル（標準入力・標準出力・標準エラー）の説明
  - エラー処理とファイルハンドルクローズの詳細
  - セクション冒頭説明文を完全翻訳
  - _Requirements: 1.1, 1.2, 1.3, 1.7, 2.1, 2.2, 2.3, 3.1, 6.2, 6.3, 7.2, 7.3_

---

### Phase 6: 統合・品質チェック

- [ ] 23. 最終統合と品質チェック
  - Phase 1-5で翻訳した全モジュールを統合し、単一ファイル`06-standard-libraries.md`を作成
  - ファイルヘッダー（Source, Translation, Glossary参照）を追加
  - ナビゲーションリンク（前章・次章・目次）を設定
  - 章導入文を完全翻訳
  - Lua 5.5変更点まとめ（セクション番号対応表含む）を追加
  - GLOSSARY.md用語の一貫性を確認
  - 全11モジュール（§6.1-§6.11）の翻訳完了を確認
  - 各モジュールの関数が原文と同数であることを確認
  - §6.5.1パターンマッチング詳細解説の存在を確認
  - §6.5 format書式指定子の完全解説の存在を確認
  - §6.5 pack/unpack書式の完全解説の存在を確認
  - 見出しレベル統一（H1/H2/H3）を確認
  - コードブロック設定を確認
  - 既存ファイル`crates/pasta_lua/doc/lua55-manual/06-standard-libraries.md`を完全版で置換
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 3.1, 3.4, 4.1, 4.2, 4.3, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.4, 9.3, 9.4, 9.5, 9.6_

---

## 完了基準（DoD）

すべて同時に満たすこと：

1. **Spec Gate**: 全フェーズ承認済み
2. **Doc Gate**: 仕様差分を反映
3. **Steering Gate**: 既存ステアリングと整合
4. **Soul Gate**: [SOUL.md](../../../SOUL.md) との整合性確認
5. **Translation Gate**: 11モジュールすべての完全翻訳完了、要約・省略表現ゼロ
