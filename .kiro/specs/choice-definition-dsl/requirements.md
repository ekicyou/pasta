# Requirements Document

## Introduction
本仕様は、Pasta DSLに選択肢定義マーカー（`＠？`）を追加し、対話分岐メニューの宣言的定義と選択結果の自動ルーティングを実現する機能を定義する。さとり（里々）の「選択肢＝トーク名」による簡便さをパスタ流に再構成し、ゴースト作者がさくらスクリプトやLuaイベントハンドラを手書きすることなく対話分岐を記述できるようにする。

## Boundary Context
- **In scope**: `＠？`選択肢マーカーのDSL構文、さくらスクリプト（`\![*]\q[...]`）の自動生成、`OnChoiceSelectEx`イベントによるシーン自動ルーティング、`!select(timeout)`による選択タイムアウト指定、サンプルゴーストへの選択肢デモ追加
- **Out of scope**: 条件付き選択肢表示（`＠？target if 条件`）、入れ子選択肢、コロン形式（`＠？target：text`）、選択肢専用シーンマーカー（`＊？`）、LSP対応（補完・ジャンプ先候補表示）、選択肢のスタイリング（色・フォント等）、アクター非依存の選択肢DSL構文（`＠？` をトーク行外に配置する文法。Luaランタイムはハイブリッド対応するがDSL文法は現行スコープ外）
- **Adjacent expectations**: SSPが`\q[...]`および`\![*]`さくらスクリプトタグを処理すること。SSPがユーザーの選択肢クリック時に`OnChoiceSelectEx`イベントを送信すること。既存のCall文（`＞`）と同じスコープ解決ルール（前方一致検索＋シャッフル＆順次消費）がシーン検索に利用可能であること。

## Requirements

### Requirement 1: 選択肢マーカー構文
**Objective:** ゴースト作者として、選択肢をDSL内で宣言的に定義したい。さくらスクリプトを手書きする必要なく対話分岐を簡潔に記述できるように。

#### Acceptance Criteria
1. When ゴースト作者が`＠？target`（省略形）を記述する, Pasta shall `target`を選択肢IDかつ表示テキストとして認識する
2. When ゴースト作者が`＠？target「表示テキスト」`（括弧形）を記述する, Pasta shall `target`を選択肢ID、`「」`内の文字列を表示テキストとして認識する
3. The Pasta shall 選択肢マーカー`＠？`の全角形式（`＠？`）と半角形式（`@?`）を等価に扱う
4. The Pasta shall 選択肢マーカーをグローバルシーンおよびローカルシーン内の行として受け入れる
5. If 選択肢マーカーの後にtarget識別子が存在しない, Pasta shall パースエラーを報告する

### Requirement 2: さくらスクリプト自動生成
**Objective:** ゴースト作者として、選択肢マーカーから適切なさくらスクリプトタグが自動生成されてほしい。`\q[...]`や`\![*]`を手動で記述する必要をなくしたい。

#### Acceptance Criteria
1. When `＠？target`（省略形）が処理される, Pasta shall `\![*]\q[target,target]`を出力する
2. When `＠？target「表示テキスト」`（括弧形）が処理される, Pasta shall `\![*]\q[表示テキスト,target]`を出力する
3. When 複数の`＠？`行がシーン内に存在する, Pasta shall 各行に対して個別に`\![*]\q[...]`を出力する

### Requirement 3: 選択肢コールバック自動ルーティング
**Objective:** ゴースト作者として、選択肢がクリックされた時に選択IDと同名のシーンが自動実行されてほしい。`OnChoiceSelectEx`ハンドラを個別に書く必要をなくしたい。

#### Acceptance Criteria
1. When SSPから`OnChoiceSelectEx`イベントを受信する, Pasta shall Reference1（選択ID）をシーン名として前方一致検索し、マッチした通常シーン（`＊target`または`・target`）を自動実行する
2. When 前方一致検索で複数のシーンがマッチする, Pasta shall シャッフル＆順次消費方式で1つを選択して実行する
3. If 前方一致検索でマッチするシーンが存在しない, Pasta shall 自動ルーティングをスキップし、通常のイベントハンドラ処理に委ねる
4. The Pasta shall 最後に実行されたグローバルシーン名を記憶し、自動ルーティング時にそのスコープ内のローカルシーンも検索対象に含める（ローカル → グローバルの順で前方一致検索）
5. When ゴースト作者が`＊OnChoiceSelectEx`シーンを明示的に定義している, Pasta shall 自動ルーティングより明示的ハンドラを優先する

### Requirement 4: 選択肢タイムアウト
**Objective:** ゴースト作者として、選択肢の表示に制限時間を設けたい。一定時間内にユーザーが選択しなかった場合の動作を制御できるように。

#### Acceptance Criteria
1. When ゴースト作者が`!select(秒数)`を選択肢の後に記述する, Pasta shall 指定された秒数の選択タイムアウトを設定する
2. The Pasta shall `!select`の引数に正の数値を受け入れる
3. If `!select`に引数が指定されない, Pasta shall タイムアウトなし（無制限待機）として扱う
