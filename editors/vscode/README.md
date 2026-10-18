# Pasta DSL

Pasta DSL（`*.pasta` ファイル）のシンタックスハイライトと診断情報を提供する VSCode 拡張です。

![Syntax Highlighting](https://raw.githubusercontent.com/ekicyou/pasta/main/editors/vscode/img/screenshot-syntax-highlight.png)

## 機能

- **TextMate 文法ハイライト** — 全角/半角マーカー両対応の構文ハイライト
- **セマンティックトークン** — pasta_lsp (WASM) による17種のセマンティックトークン提供
- **診断情報** — パースエラーの Problems パネル表示
- **ゴースト全体の診断** — `ghost/master` 配下の `.pasta`・`ghost/master/pasta.toml` を保存したとき、または `ghost/master` 配下の `.pasta`・`.lua` を作成・削除・改名したときに、そのゴースト全体を `pasta_check validate` と同じ規則で検査し（`ghost`・`master`・`pasta.toml` の名前と拡張子は大文字・小文字を区別しない。`Ghost/Master` も対象）、誤りを Error・注意を Warning として該当ファイル・行に表示（source は `pasta validate`。続けて保存したときはまとめて 1 回検査。保存前の入力中と起動直後は検査しない。ディスクに保存された内容を読む。WASM ロード失敗時は動作しない）。報告の意味と直し方は [マニュアル](https://ekicyou.github.io/pasta/debug/validate.html#エディタの診断) を参照
- **フォールバック** — WASM ロード失敗時は TextMate 文法のみで動作
- **デバッガ接続** — 実行中の Pasta デバッグバックエンドへの attach（DAP）と、`.pasta` ⇔ `.lua` の提示モード切替（コマンドパレット / デバッグツールバー / ステータスバー）

## Pasta DSL とは

Pasta は「伺か」のようなデスクトップマスコットなどを実現するための対話スクリプト言語です。日本語フレンドリーな全角マーカーと宣言的フロー制御が特徴です。

詳しくは [GitHub リポジトリ](https://github.com/ekicyou/pasta) をご覧ください。

## 対応環境

- VSCode ^1.85.0

## セマンティックトークン一覧

| # | タイプ       | Pasta 構文要素                  |
|---|-------------|--------------------------------|
| 0 | comment     | コメント行 (`＃` / `#`)        |
| 1 | namespace   | グローバルシーン (`＊` / `*`)  |
| 2 | scene       | ローカルシーン (`・` / `-`)    |
| 3 | decorator   | 属性定義 (`＆` / `&`)         |
| 4 | word        | 単語定義 (`＠` / `@`)         |
| 5 | variable    | 変数参照 (`＄` / `$`)         |
| 6 | call        | Call文 (`＞` / `>`)           |
| 7 | actor       | アクター定義 (`％` / `%`)     |
| 8 | actorName   | アクション行のアクター名       |
| 9 | codeBlock   | Lua コードブロック             |
| 10| talk        | Talk テキスト                  |
| 11| sakuraScript| さくらスクリプトタグ           |
| 12| escape      | エスケープシーケンス           |
| 13| operator    | コロン区切り (`：` / `:`)・代入 (`＝` / `=`)・括弧・二項演算子 |
| 14| number      | 数値リテラル                   |
| 15| cueMarker   | キューコマンドマーカー (`！` / `!`) |
| 16| cueCommand  | キューコマンド名               |

## 開発

- `npm run package` で VSIX を作るには、手元に `pwsh`（PowerShell 7）・`wasm-pack`・`cargo-about` が要ります（WASM はリリースビルドになります）。

## ライセンス

MIT
