# ダウンロード

| やりたいこと | ファイル |
| ------------ | -------- |
| [まずゴーストを動かしてみる](#サンプルゴースト) | [`hello-pasta.nar`](https://github.com/ekicyou/pasta/releases/latest/download/hello-pasta.nar) |
| [自分のゴーストの `pasta.dll` を差し替える](#shiori-の本体) | [`pasta.dll.zip`](https://github.com/ekicyou/pasta/releases/latest/download/pasta.dll.zip) |
| [VS Code を使わずに書く](#vs-code-を使わずに書くアプリ) | [`pasta-recipe.x64.zip`](https://github.com/ekicyou/pasta/releases/latest/download/pasta-recipe.x64.zip)・[`pasta-recipe.arm64.zip`](https://github.com/ekicyou/pasta/releases/latest/download/pasta-recipe.arm64.zip) |
| [VS Code で書く](#vs-code-の拡張) | [Marketplace の拡張](https://marketplace.visualstudio.com/items?itemName=ekicyou.pasta-vscode)（VSIX は手動で入れるとき。[最新のリリース](https://github.com/ekicyou/pasta/releases/latest)の Assets にある） |
| [辞書を検査する・.nar を作る](#検査の-cli) | [`pasta_check.zip`](https://github.com/ekicyou/pasta/releases/latest/download/pasta_check.zip) |
| [AI に書かせる](#ai-向けのスキル) | [`pasta-skills.zip`](https://github.com/ekicyou/pasta/releases/latest/download/pasta-skills.zip) |

ファイル名のリンクは、最新版を直接ダウンロードする。過去の版は[過去の版](#過去の版)を見る。

## サンプルゴースト

`hello-pasta.nar` は、`pasta.dll` とシェルを含む完成版のゴースト一式である。SSP のウィンドウへドロップして入れる。自分のゴーストへ写す元にもなる。
同梱の `pasta.dll` の版は、マニュアルの対象バージョンと同じである。
詳しくは[前提環境と準備](getting-started/prerequisites.md)と[hello-pasta から写す](getting-started/setup.md#hello-pasta-から写す)を見る。

## SHIORI の本体

`pasta.dll.zip` は、`pasta.dll` とライセンス表示を含む。自分のゴーストの `ghost/master/` にある `pasta.dll` を、これで差し替える。
マニュアルの対象バージョンと同じ版を使うこと。
置き方は[hello-pasta から写す](getting-started/setup.md#hello-pasta-から写す)、対象バージョンは[このマニュアルが対象とするバージョン](introduction.md#このマニュアルが対象とするバージョン)を見る。

## VS Code を使わずに書くアプリ

pasta-recipe の zip は Windows の種類ごとに 2 つある。自分の Windows（x64 か arm64）に合うほうを選ぶ。
使っている pasta と同じ版を使うこと。
種類の見分け方と置き方は[入手と置き方](debug/recipe.md#入手と置き方)を見る。

## VS Code の拡張

通常は Marketplace から入れる。手動で入れるときは、最新のリリースのページの Assets にある `pasta-vscode-<版>.vsix` を使う。
使っている pasta と同じ版を選ぶこと。
詳しくは[VS Code の準備](debug/vscode-setup.md)を見る。

## 検査の CLI

`pasta_check.zip` は、辞書の検査と `.nar` の作成を行う CLI である。
使っている pasta と同じ版を使うこと。
使い方は[使い方](debug/validate.md#使い方)を見る。

## AI 向けのスキル

Claude Code などの AI のエージェントにゴーストを書かせるときは、pasta のスキルを読ませる。
スキルは `pasta-skills.zip` にまとめて配布しており、中身は次の 3 つである。

| スキル | 役目 |
| ------ | ---- |
| `pasta-ghost-authoring` | Pasta DSL の文法と辞書の書き方 |
| `pasta-lua-coding` | ランタイムの Lua API と書き方の決まり |
| `pasta-check` | `pasta_check` による辞書の検査とリリースビルド |

`pasta-skills.zip` は、冒頭の表の固定リンクか、[過去の版](#過去の版)の節のリリースページから入手する。
入手した zip は、ゴーストの開発フォルダの `.claude/skills/` に展開する。展開すると、次の木ができる。

```text
.claude/skills/
├── pasta-ghost-authoring/SKILL.md
├── pasta-lua-coding/SKILL.md
├── pasta-check/SKILL.md
└── LICENSE
```

スキルは、使っている pasta と同じ版の `pasta-skills.zip` を使うこと。pasta の版を上げるときは、
3 つのフォルダを消してから新しい版の zip を展開する。上書きで展開すると、新しい版で消えたファイルが古いまま残るためである。

## 過去の版

固定リンクは常に最新版を落とす。過去の版は、[リリースページ](https://github.com/ekicyou/pasta/releases)の各版の Assets から落とす。
