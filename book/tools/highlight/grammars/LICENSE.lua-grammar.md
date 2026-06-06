# vendor lua TextMate 文法のライセンスと出典

本ディレクトリの `lua.tmLanguage.json` は、入れ子 ```lua ブロックの二段トークナイズ着色専用に
**読み取り専用で vendor**（同梱）した第三者の TextMate 文法です。pasta ハイライトの SSOT
（`editors/vscode/syntaxes/pasta.tmLanguage.json`、scopeName `source.pasta`）とは**別言語の独立文法**であり、
pasta の第2文法ではありません（要件 5.1 に抵触しません）。

- **ファイル**: `lua.tmLanguage.json`
- **scopeName**: `source.lua`
- **形式**: JSON（vscode-textmate の `parseRawGrammar(content, '*.json')` でロード可能）
- **SPDX ライセンス識別子**: `MIT`
- **著作権者**: Microsoft Corporation（2015 - present）

## 出典

- **取得元 URL**: https://raw.githubusercontent.com/microsoft/vscode/main/extensions/lua/syntaxes/lua.tmLanguage.json
- **リポジトリ**: https://github.com/microsoft/vscode （`extensions/lua/syntaxes/lua.tmLanguage.json`）
- **リポジトリ LICENSE（一次情報・MIT）**: https://github.com/microsoft/vscode/blob/main/LICENSE.txt
- **取得日**: 2026-06-05（UTC）
- **上流の由来**: 当該ファイルは VS Code により
  https://github.com/sumneko/lua.tmbundle/blob/master/Syntaxes/Lua.plist から変換されたもので、
  ファイル先頭の `information_for_contributors` / `version` フィールド
  （`https://github.com/sumneko/lua.tmbundle/commit/b295d83bf0e91b5d3a69eb097f9ed351623b95be`）に由来が明記されています。
  この由来ヘッダは改変せず保持しています。

## 改変方針

- 本文法は**読み取り専用 vendor** であり、以後改変しません（由来ヘッダ含む）。
- **公開成果物には含めません**（build-time のみ使用）。`grammars/` は `book/book` 出力へコピーされず、
  後続の HtmlHighlighter は `book/book/**/*.html` のみを書き換えます。

## MIT License（原文・Microsoft Corporation, microsoft/vscode）

```
MIT License

Copyright (c) 2015 - present Microsoft Corporation

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
