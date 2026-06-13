# Brief: pasta-vscode-lua-highlight

## Problem
pasta ゴースト作者が `.pasta` ファイルを VS Code で編集する際、Lua コードブロック（```` ```lua ... ``` ````）の中身が Lua として色付けされず、単色（`codeBlock`）で塗りつぶされている。複雑なロジックを Lua で記述する作者にとって、キーワード・文字列・関数・コメントの視認性が無く、可読性と編集効率が低い。

## Current State
- TextMate 文法 [editors/vscode/syntaxes/pasta.tmLanguage.json](editors/vscode/syntaxes/pasta.tmLanguage.json#L113) の `lua-code-block` は、ブロックに `meta.embedded.block.lua` というスコープ名を付与するのみで、実際の Lua 文法（`source.lua`）を注入していない。
- pasta_lsp（WASM）由来のセマンティックトークンが [crates/pasta_lsp/src/analysis/visitors.rs](crates/pasta_lsp/src/analysis/visitors.rs#L109) の `visit_code_block` で Lua ブロック**本文全域**を単一 `codeBlock` トークンとして出力（[add_token_from_span](crates/pasta_lsp/src/analysis/visitors.rs#L844) が複数行スパンを全行カバー）。
- セマンティックトークンは TextMate より優先されるため、仮に `source.lua` を注入しても本文全域の `codeBlock` トークンに上書きされて埋め込み Lua ハイライトは見えない。
- VS Code には Lua 文法 `source.lua` が組み込みで常時同梱されている。

## Desired Outcome
- `.pasta` の Lua コードブロック内部が、VS Code 組み込みの Lua 文法で自動的にシンタックスハイライトされる（ユーザー操作不要の埋め込み言語モード）。
- フェンスマーカー（```` ``` ````, `lua`）は引き続き pasta 側のスコープで識別可能。
- pasta 固有のハイライト（シーン・アクター・単語・変数等、Lua ブロック外）は無回帰。

## Approach
**TextMate 埋め込み言語注入 ＋ セマンティックトークン範囲調整**（採用済み）。

1. **TextMate 層**: `lua-code-block` の content に `{ "include": "source.lua" }` を注入し、ブロック本文を Lua 文法で色付け。
2. **セマンティックトークン層**: pasta_lsp の `visit_code_block` が出す `codeBlock` トークンを、本文全域ではなく**フェンスマーカーのみ**（または本文では非出力）に絞り、埋め込み Lua の TextMate ハイライトが見えるようにする。

理由: VS Code 標準の embedded languages 手法は確立されており、`source.lua` は常時利用可能。LSP 調整は Lua ハイライトを可視化するための必須前提（ユーザー承認済み 2026-06-12）。

## Scope
- **In**:
  - TextMate 文法への `source.lua` 注入（自動埋め込み Lua ハイライト）
  - pasta_lsp `visit_code_block` の `codeBlock` トークン範囲調整（フェンスのみ等）
  - 回帰テスト（Lua ブロック内ハイライト確認・Lua ブロック外の pasta ハイライト無回帰・セマンティックトークン無回帰）
- **Out**:
  - ユーザー操作でハイライトを切り替えるトグルコマンド/ボタン（今回は自動切替のみ。手動トグルは将来仕様）
  - インライン Lua（`＠func()` 関数呼び出し）への Lua 文法注入（本仕様は複数行 Lua ブロックに限定）
  - Lua ブロック内の診断・補完・LSP 機能拡張（ハイライトのみ）
  - book/ マニュアル側のコードブロックハイライト（pasta-manual-syntax-highlight が別途担当）

## Boundary Candidates
- TextMate 文法注入（拡張のシンタックス定義の責務）
- セマンティックトークン生成ロジック（pasta_lsp 解析層の責務）
- 両者の協調（どちらが Lua ブロック本文を塗るかの責任分界）

## Out of Boundary
- 手動ハイライトモードトグル（`pasta.debug.toggleSourcePresentation` 類似の UX）— 本仕様は所有しない
- Lua ブロック内のコード補完・定義ジャンプ・診断
- pasta DSL 文法そのものの変更

## Upstream / Downstream
- **Upstream**: pasta-vscode-extension（完了・TextMate 文法とセマンティックトークン基盤を構築）、pasta-language-server（完了・LSP/WASM 解析基盤）
- **Downstream**: 将来の「手動ハイライトモードトグル」仕様、Lua ブロック内 LSP 機能拡張

## Existing Spec Touchpoints
- **Extends**: pasta-vscode-extension（completed・TextMate 文法）、pasta_lsp の解析層（completed 群）
- **Adjacent**: pasta-manual-syntax-highlight（mdBook 側の *.pasta ハイライト・読者層と出力先が異なる別境界）、pasta-debug-lua-view-toggle（デバッグ中の提示モード切替・ハイライトではなくソース提示）

## Constraints
- VS Code エンジン `^1.85.0`（[editors/vscode/package.json](editors/vscode/package.json#L25)）の embedded languages 仕様に準拠
- `source.lua` は VS Code 組み込み文法に依存（追加 grammar 同梱は不要）
- セマンティックトークンと TextMate の優先順位（セマンティック優先）を前提に設計
- Lua ブロック外の pasta 固有ハイライトおよび既存セマンティックトークンは外部観測挙動として保存
