# pasta デバッグガイド（移設のご案内）

このファイルにあったデバッグの説明は、pasta 利用者マニュアルの **デバッグ章** に移設しました。
デバッグの使い方・有効化・VSCode 接続・`.pasta` ソースレベル操作・構造的制約と緩和策の
**最新かつ正式な情報源はマニュアルのデバッグ章**です。内容の二重管理を避けるため、ここでは
個別の手順や設定値を再掲しません。最新情報は必ずマニュアルを参照してください。

## デバッグ章への導線

- 公開サイト: <https://ekicyou.github.io/pasta/> のデバッグ章
- リポジトリ内ソース: `book/src/debug/`
  - `book/src/debug/index.md` — 概要・有効化・ウォークスルー
  - `book/src/debug/vscode-setup.md` — VSCode 拡張の導入・`launch.json`・attach 手順
  - `book/src/debug/source-level.md` — `.pasta` ソースレベルの操作（ブレークポイント・ステップ・変数 inspect・提示モード切替）
  - `book/src/debug/constraints.md` — 構造的制約（ブレーク中はホスト応答が止まる）と緩和策
