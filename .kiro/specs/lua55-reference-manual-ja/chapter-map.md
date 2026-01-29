# 章分割ファイルマップ

> 生成日: 2026-01-29  
> 目的: 翻訳タスクで使用するファイル対応表

## サマリー

| 項目 | EN | JA | 備考 |
|------|----|----|------|
| 主要章ファイル | 9 | 9 | 1-9章 |
| サブ分割ファイル | 22 | 22 | 3章(4), 4章(7), 6章(11) |
| **合計** | **31** | **31** | |

---

## 主要章ファイル一覧

| 章 | EN ファイル | サイズ | JA ファイル | サイズ | 対応 |
|---|------------|-------|------------|-------|------|
| 1 | `en/01-introduction.html` | 2.3KB | `ja/01-introduction.html` | 3.2KB | ✅ |
| 2 | `en/02-basic-concepts.html` | 40.1KB | `ja/02-basic-concepts.html` | 47.5KB | ✅ |
| 3 | `en/03-language.html` | 52KB | `ja/03-language.html` | 59.9KB | ⚠️ サブ分割 |
| 4 | `en/04-c-api.html` | 114.8KB | `ja/04-c-api.html` | 131.7KB | ⚠️ サブ分割 |
| 5 | `en/05-auxiliary-library.html` | 37.7KB | `ja/05-auxiliary-library.html` | 42.4KB | ✅ |
| 6 | `en/06-standard-libraries.html` | 108.1KB | `ja/06-standard-libraries.html` | 127.6KB | ⚠️ サブ分割 |
| 7 | `en/07-standalone.html` | 6.4KB | `ja/07-standalone.html` | 7.3KB | ✅ |
| 8 | `en/08-incompatibilities.html` | 3.8KB | `ja/08-incompatibilities.html` | 7.6KB | ⚠️ 5.5固有 |
| 9 | `en/09-complete-syntax.html` | 3.8KB | `ja/09-complete-syntax.html` | 3.2KB | ✅ |

---

## 3章サブ分割: The Language / 言語

| EN ファイル | サイズ | JA ファイル | サイズ | 対応 |
|------------|-------|------------|-------|------|
| `en/language/01-lexical-conventions.html` | 7KB | `ja/language/01-section-3-1.html` | 8.2KB | ✅ §3.1 |
| `en/language/02-variables.html` | 1.3KB | `ja/language/02-section-3-2.html` | 1.8KB | ✅ §3.2 |
| `en/language/03-statements.html` | 16.1KB | `ja/language/03-section-3-3.html` | 17KB | ✅ §3.3 |
| `en/language/04-expressions.html` | 27.3KB | `ja/language/04-section-3-4.html` | 30.3KB | ✅ §3.4 |
| （なし） | - | `ja/language/05-section-3-5.html` | 2.2KB | ⚠️ 5.4のみ |

**注意**: 5.4には§3.5「可視性ルール」が存在しますが、5.5には対応する節がありません（§2.2に統合された可能性）。

---

## 4章サブ分割: The Application Program Interface / C API

| EN ファイル | サイズ | JA ファイル | サイズ | 対応 |
|------------|-------|------------|-------|------|
| `en/c-api/01-the-stack.html` | 6.3KB | `ja/c-api/01-section-4-1.html` | 8.1KB | ✅ §4.1 |
| `en/c-api/02-c-closures.html` | 1.1KB | `ja/c-api/02-section-4-2.html` | 1.4KB | ✅ §4.2 |
| `en/c-api/03-registry.html` | 1.7KB | `ja/c-api/03-section-4-3.html` | 2.2KB | ✅ §4.3 |
| `en/c-api/04-error-handling-in-c.html` | 3.5KB | `ja/c-api/04-section-4-4.html` | 4.2KB | ✅ §4.4 |
| `en/c-api/05-handling-yields-in-c.html` | 5.8KB | `ja/c-api/05-section-4-5.html` | 6.8KB | ✅ §4.5 |
| `en/c-api/06-functions-and-types.html` | 78.3KB | `ja/c-api/06-section-4-6.html` | 87.5KB | ✅ §4.6 |
| `en/c-api/07-the-debug-interface.html` | 17.3KB | `ja/c-api/07-section-4-7.html` | 20.3KB | ✅ §4.7 |

**注意**: 4章は全節完全対応。§4.6「Functions and Types」は約78KB（最大）ですが、API関数リストなのでさらなる分割は不要。

---

## 6章サブ分割: The Standard Libraries / 標準ライブラリ

| EN ファイル | サイズ | JA ファイル | サイズ | 対応 |
|------------|-------|------------|-------|------|
| `en/standard-libraries/01-loading-the-libraries-in-c-code.html` | 2.8KB | （なし） | - | ⚠️ 5.5のみ |
| `en/standard-libraries/02-basic-functions.html` | 16.3KB | `ja/standard-libraries/01-section-6-1.html` | 19.3KB | ✅ 5.5:§6.2 ↔ 5.4:§6.1 |
| `en/standard-libraries/03-coroutine-manipulation.html` | 4KB | `ja/standard-libraries/02-section-6-2.html` | 4.7KB | ✅ 5.5:§6.3 ↔ 5.4:§6.2 |
| `en/standard-libraries/04-modules.html` | 12.5KB | `ja/standard-libraries/03-section-6-3.html` | 16KB | ✅ 5.5:§6.4 ↔ 5.4:§6.3 |
| `en/standard-libraries/05-string-manipulation.html` | 25.4KB | `ja/standard-libraries/04-section-6-4.html` | 30.5KB | ✅ 5.5:§6.5 ↔ 5.4:§6.4 |
| `en/standard-libraries/06-utf-8-support.html` | 4.3KB | `ja/standard-libraries/05-section-6-5.html` | 5KB | ✅ 5.5:§6.6 ↔ 5.4:§6.5 |
| `en/standard-libraries/07-table-manipulation.html` | 4.9KB | `ja/standard-libraries/06-section-6-6.html` | 5.1KB | ✅ 5.5:§6.7 ↔ 5.4:§6.6 |
| `en/standard-libraries/08-mathematical-functions.html` | 7.7KB | `ja/standard-libraries/07-section-6-7.html` | 8KB | ✅ 5.5:§6.8 ↔ 5.4:§6.7 |
| `en/standard-libraries/09-input-and-output-facilities.html` | 10.8KB | `ja/standard-libraries/08-section-6-8.html` | 13.3KB | ✅ 5.5:§6.9 ↔ 5.4:§6.8 |
| `en/standard-libraries/10-operating-system-facilities.html` | 8.1KB | `ja/standard-libraries/09-section-6-9.html` | 9.9KB | ✅ 5.5:§6.10 ↔ 5.4:§6.9 |
| `en/standard-libraries/11-the-debug-library.html` | 10.6KB | `ja/standard-libraries/10-section-6-10.html` | 12.8KB | ✅ 5.5:§6.11 ↔ 5.4:§6.10 |

**注意**: 
- 5.5では§6.1「Loading the Libraries in C code」が新設（5.4に対応なし → 独自翻訳）
- 以降の節番号は5.5が+1ずれ（5.5:§6.x ↔ 5.4:§6.(x-1)）
- 翻訳時は節タイトルでマッチングし、対応するJAファイルを参照すること

---

## 翻訳時ファイル対応早見表

### 単純対応（参考JA版あり）

| 5.5 EN節 | 翻訳元 | 参考JA版 |
|---------|--------|---------|
| 1章 | `en/01-introduction.html` | `ja/01-introduction.html` |
| 2章 | `en/02-basic-concepts.html` | `ja/02-basic-concepts.html` |
| §3.1-3.4 | `en/language/0X-*.html` | `ja/language/0X-section-3-X.html` |
| §4.1-4.7 | `en/c-api/0X-*.html` | `ja/c-api/0X-section-4-X.html` |
| 5章 | `en/05-auxiliary-library.html` | `ja/05-auxiliary-library.html` |
| §6.2-6.11 | `en/standard-libraries/0X-*.html` | `ja/standard-libraries/(X-1)-*.html` |
| 7章 | `en/07-standalone.html` | `ja/07-standalone.html` |
| 9章 | `en/09-complete-syntax.html` | `ja/09-complete-syntax.html` |

### 独自翻訳（参考JA版なし）

| 5.5 EN節 | 翻訳元 | 備考 |
|---------|--------|------|
| §2.2 | `en/02-basic-concepts.html`内 | `global`キーワード追加、5.4参考は部分的 |
| §6.1 | `en/standard-libraries/01-loading-the-libraries-in-c-code.html` | 5.5新設、完全独自翻訳 |
| 8章 | `en/08-incompatibilities.html` | 5.5固有内容（5.4→5.5の変更点） |

---

## サイズ分類

### 小章（<10KB）- 全文+参考全文+GLOSSARY
- 1章 (2.3KB)
- §3.2 (1.3KB)
- §4.2-4.4 (1.1-3.5KB)
- §6.1, 6.3, 6.6-6.7 (2.8-4.9KB)
- 7章 (6.4KB)
- 8章 (3.8KB)
- 9章 (3.8KB)

### 中章（10-40KB）- 全文+用語抽出リスト+GLOSSARY
- 2章 (40.1KB)
- §3.1, 3.3, 3.4 (7-27.3KB)
- §4.1, 4.5, 4.7 (5.8-17.3KB)
- 5章 (37.7KB)
- §6.2, 6.4-6.5, 6.8-6.11 (10.6-25.4KB)

### 大章（>40KB）- 節単位分割
- 3章全体 (52KB) → サブ分割済み
- 4章全体 (114.8KB) → サブ分割済み
- 6章全体 (108.1KB) → サブ分割済み

---

## 次のステップ

1. **タスク3.1**: GLOSSARY.md初版作成（5.4日本語版から50-80語抽出）
2. **タスク3.2**: トークン数見積もり実施
3. **タスク4.x**: 章別AI翻訳開始（1章から順次）
