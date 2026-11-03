# pasta デバッグガイド（移設のご案内）

このファイルにあったデバッグの説明は、pasta 利用者マニュアルの **デバッグ章** に移設しました。
デバッグの有効化・開発用のイベント（シーンの再生・グローバル変数の書き出し）・辞書の検査の
**最新かつ正式な情報源はマニュアル**です。内容の二重管理を避けるため、ここでは
個別の手順や設定値を再掲しません。最新情報は必ずマニュアルを参照してください。

## マニュアルへの導線

- 公開サイト: <https://ekicyou.github.io/pasta/> のデバッグ章
- リポジトリ内ソース: `book/src/debug/`
  - `book/src/debug/index.md` — 概要・有効化（章の各ページへの入口）
- 開発用のイベントの引数と応答: `book/src/lua/shiori-events.md`（`OnPastaScene`・`OnPastaSceneAt`・`OnPastaDumpVariables`）
- `[debug]` の書き方: `book/src/reference/pasta-toml.md`
