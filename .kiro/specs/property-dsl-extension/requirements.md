# Requirements Document

## Introduction
本仕様は、Pasta DSLに `＄％`（プロパティスコープ修飾子）を導入し、SSPプロパティシステムへの読み書きをDSL内から直接記述可能にする。既存の変数スコープ修飾パターン（`＄` = ローカル、`＄＊` = グローバル）の自然な拡張として、`＄％` を「共有プロパティスコープ」に割り当てる。プロパティ書き込み（SET）は同期的に処理され、読み取り（GET）はフレームワーク内部の非同期通信で透過的に処理される。

## Boundary Context
- **In scope**: `＄％prop.path＝value` による書き込み、`＄var＝＄％prop.path` による読み取り、アクション行内のインライン展開 `＄％prop.path`、プロパティ名の文字クラス定義、パーサー文法拡張、コンパイラのコード生成、`get_property()` のトークンバッファ保全改修（yield前後のトークン退避・復元）
- **Out of scope**: 式中でのプロパティ参照（`＄var＝＄％a ＋ ＄％b`）、新規Lua APIの追加、プロパティ値の型変換（文字列として返す）、`%property[name]` 環境変数展開、LSP対応（補完・バリデーション等）、プロパティ名の存在検証（SSP側の責任）
- **Adjacent expectations**: 既存の `act:set_property()` および `act:get_property()` Lua APIが正常に動作すること。既存のyield/resume非同期通信基盤（`shiori-async-talk` spec）がプロパティGETのコールバック処理を正しくルーティングすること

## Requirements

### Requirement 1: プロパティスコープ修飾子と名前の文法
**Objective:** ゴースト作者として、`＄％` マーカーの後にSSPプロパティ名（`system.name`、`currentghost.balloon.scope(0).validwidth.initial` 等）をエスケープや引用符なしで直接記述したい。これにより、プロパティアクセスが変数参照と同じ直感性で書けるようになる。

#### Acceptance Criteria
1. The pasta parser shall `＄％` を変数スコープ修飾子として認識し、後続をプロパティ名として解析する
2. The pasta parser shall プロパティ名として、ASCII英字 `[a-zA-Z]` で始まり、後続文字に `[_().a-zA-Z0-9]` を許容する文字列を認識する
3. When プロパティ名にドット `.` を含む場合（例: `system.name`）, the pasta parser shall ドットをプロパティ名の一部として認識する
4. When プロパティ名に括弧と数字を含む場合（例: `scope(0)`）, the pasta parser shall 括弧と数字をプロパティ名の一部として認識する
5. When プロパティ名の直後に許容文字クラス外の文字（空白、改行、全角文字、演算子等）が出現した場合, the pasta parser shall その位置でプロパティ名を終端する
6. The pasta parser shall `＄％` の全角形式と半角形式 `$%` を同等に扱う

### Requirement 2: プロパティ書き込み（SET）
**Objective:** ゴースト作者として、`＄％prop.path＝value` でSSPプロパティを書き込みたい。Luaブロックを使わずにDSL内でプロパティ設定を簡潔に記述でき、辞書の可読性が向上する。

#### Acceptance Criteria
1. When ゴースト作者が `＄％prop.path＝value` を記述した場合, the pasta framework shall 実行時にSSPプロパティ `prop.path` を指定値に設定する
2. When SET文の値にリテラル（数値、文字列）を指定した場合, the pasta framework shall リテラル値をプロパティ値として設定する
3. When SET文の値にローカル変数参照 `＄var` を指定した場合, the pasta framework shall 変数の現在値をプロパティ値として設定する
4. When SET文の値にグローバル変数参照 `＄＊var` を指定した場合, the pasta framework shall 変数の現在値をプロパティ値として設定する
5. When SET文の値に単語参照 `＠word` を指定した場合, the pasta framework shall 単語の選択値をプロパティ値として設定する
6. When SET文の値に式（算術演算等）を指定した場合, the pasta framework shall 式の評価結果をプロパティ値として設定する

#### SET構文の想定例
```pasta
＄％system.name＝「新しい名前」
＄％currentghost.shellname＝＄選択シェル
＄％currentghost.balloon.scope(0).validwidth.initial＝400
```

### Requirement 3: プロパティ読み取り — 変数代入（GET）
**Objective:** ゴースト作者として、`＄var＝＄％prop.path` でSSPプロパティ値を変数に取得したい。非同期通信の詳細を意識せずにプロパティ値を利用できるようになる。

#### Acceptance Criteria
1. When ゴースト作者が `＄var＝＄％prop.path` を記述した場合, the pasta framework shall 実行時にSSPプロパティ値を取得し、指定ローカル変数に文字列として代入する
2. When ゴースト作者が `＄＊var＝＄％prop.path` を記述した場合, the pasta framework shall 実行時にSSPプロパティ値を取得し、指定グローバル変数に文字列として代入する
3. When プロパティGETの結果が変数に代入された後, the pasta framework shall 同一シーン内で後続のトーク生成や追加のプロパティ操作を正常に継続する
4. When 指定プロパティがSSPに存在しない場合, the pasta framework shall nilを変数に代入する
5. When `get_property()` が呼び出された時点でトークンバッファに未配信のトークンが存在する場合, the pasta framework shall 既存トークンを退避し、getタグのみでyieldし、resume後に退避トークンを復元する（トークンバッファ非汚染）

#### GET構文の想定例
```pasta
＄ゴースト名＝＄％currentghost.name
さくら：この子の名前は＄ゴースト名　です
```

### Requirement 4: プロパティ読み取り — アクション行インライン（GET）
**Objective:** ゴースト作者として、アクション行内に `＄％prop.path` を直接埋め込んでプロパティ値をトークに展開したい。一時変数を経由せず簡潔な辞書記述が可能になる。

#### Acceptance Criteria
1. When ゴースト作者がアクション行内に `＄％prop.path` を記述した場合, the pasta framework shall 実行時にプロパティ値を取得し、文字列としてトーク出力に展開する
2. When アクション行内に複数の `＄％` 参照が存在する場合, the pasta framework shall 全プロパティ値を正しく取得し、テキスト内の出現位置に対応する値をそれぞれ展開する
3. When アクション行内に `＄％` 参照と通常テキスト・変数参照 `＄var`・単語参照 `＠word` が混在する場合, the pasta framework shall すべての要素を記述順に正しくトーク出力に展開する
4. When アクション行内に `＄％` 参照が存在する場合, the pasta framework shall プロパティ値取得の前後で蓄積済みトークンを分断せずに保全する（R3-AC5のトークンバッファ非汚染保証による）

#### インラインGET構文の想定例
```pasta
さくら：ゴースト名は＄％currentghost.name　です
さくら：幅＝＄％currentghost.balloon.scope(0).validwidth.initial　高さ＝＄％currentghost.balloon.scope(0).validheight.initial
```

### Requirement 5: 既存構文との互換性
**Objective:** ゴースト作者として、`＄％` プロパティ構文の導入後も、既存のローカル変数 `＄var`、グローバル変数 `＄＊var`、式文 `＄＝expr` が従来通り動作することを保証したい。

#### Acceptance Criteria
1. The pasta parser shall 既存のローカル変数参照 `＄var` およびローカル変数代入 `＄var＝value` の解析結果を変更しない
2. The pasta parser shall 既存のグローバル変数参照 `＄＊var` およびグローバル変数代入 `＄＊var＝value` の解析結果を変更しない
3. The pasta parser shall 既存の式文 `＄＝expr` の解析結果を変更しない
4. When 行頭に `％` が出現する場合, the pasta parser shall 従来通りアクター辞書マーカーとして解析する（`＄％` は `＄` が先行するプロパティスコープ修飾子であり、行頭 `％` とは異なるコンテキスト）

### Requirement 6: 構文エラー報告
**Objective:** ゴースト作者として、プロパティ構文の誤用に対してパース時に明確なエラー情報を受け取りたい。辞書開発中のデバッグが容易になる。

#### Acceptance Criteria
1. If `＄％` の直後にプロパティ名の開始文字として不正な文字（非ASCII英字）が出現した場合, the pasta parser shall エラー位置を含むパースエラーを報告する
2. If `＄％` の直後にプロパティ名がなく行末または空白に達した場合, the pasta parser shall エラー位置を含むパースエラーを報告する
