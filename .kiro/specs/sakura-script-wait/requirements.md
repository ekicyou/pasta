# Requirements Document

## Introduction

本ドキュメントは、会話テキストに対してさくらスクリプトウェイト（`\_w[ms]`）を自動付与する機能の要件を定義する。日本語テキストの濁点・半濁点・感嘆符などの文字種別に応じた適切なウェイトを挿入することで、会話の自然なテンポを表現する。

## Project Description (Input)

### 濁点・半濁点の、さくらスクリプトウェイト付与
会話スピードを自然に表現するために、テキストに適切なさくらスクリプトウェイトを付与したい。`@pasta_sakura_script_wait`rustモジュールを公開し、lua側からウェイトなしテキストとパラメーターを指定することで、ウェイト設定したさくらスクリプトを取得したい。

**追加要件**: 対象文字セットおよびデフォルトウェイト値をpasta.tomlで設定可能とすること。セクション名は`[talk]`とする。

## Requirements

### Requirement 1: Luaモジュール公開

**Objective:** As a ゴースト開発者, I want Lua側から`@pasta_sakura_script_wait`モジュールを呼び出してウェイト付きスクリプトを取得できること, so that 会話テキストに自然なテンポを付与できる

#### Acceptance Criteria

1. When `require "@pasta_sakura_script_wait"` を呼び出したとき, the pasta_lua shall `talk_to_script` 関数を含むモジュールテーブルを返す
2. When `talk_to_script(actor, talk)` を呼び出したとき, the pasta_sakura_script_wait shall actorテーブルからウェイト設定を読み取り、talk文字列にウェイトを挿入したスクリプト文字列を返す
3. If actorテーブルがnilまたは無効な場合, then the pasta_sakura_script_wait shall pasta.tomlのデフォルト設定を使用してウェイトを挿入する

### Requirement 2: actorパラメーター参照

**Objective:** As a ゴースト開発者, I want actorオブジェクトからキャラクター固有のウェイト設定を参照できること, so that キャラクターごとに異なる会話テンポを表現できる

#### Acceptance Criteria

1. When actorテーブルにウェイト設定キーが存在するとき, the pasta_sakura_script_wait shall 以下のキーからウェイト値（ミリ秒）を取得する:
   - `script_wait_normal`: 通常文字のウェイト
   - `script_wait_period`: 濁点（句点）検出ウェイト
   - `script_wait_comma`: 半濁点（読点）検出ウェイト
   - `script_wait_exclaim`: 感嘆詞検出ウェイト
   - `script_wait_leader`: リーダー検出ウェイト
2. If actorテーブルに特定のウェイト設定キーが存在しない場合, then the pasta_sakura_script_wait shall pasta.tomlの`[talk]`セクションのデフォルト値を使用する
3. The pasta_sakura_script_wait shall 数値以外の値が設定されている場合、デフォルト値にフォールバックする

### Requirement 3: pasta.toml設定

**Objective:** As a ゴースト開発者, I want pasta.tomlでウェイト関連のデフォルト設定と対象文字セットを定義できること, so that ゴースト全体で一貫したウェイト設定を管理できる

#### Acceptance Criteria

1. The pasta_lua shall pasta.tomlに`[talk]`セクションを認識し、以下のデフォルトウェイト値を読み込む:
   - `script_wait_normal`: デフォルト50（ミリ秒）
   - `script_wait_period`: デフォルト1000（ミリ秒）
   - `script_wait_comma`: デフォルト500（ミリ秒）
   - `script_wait_exclaim`: デフォルト500（ミリ秒）
   - `script_wait_leader`: デフォルト200（ミリ秒）
2. The pasta_lua shall pasta.tomlの`[talk]`セクションで以下の対象文字セットを設定可能とする:
   - `chars_period`: 句点文字（デフォルト: `｡。．.`）
   - `chars_comma`: 読点文字（デフォルト: `、，,`）
   - `chars_exclaim`: 感嘆詞文字（デフォルト: `？！!?`）
   - `chars_leader`: リーダー文字（デフォルト: `･・‥…`）
   - `chars_line_start_prohibited`: 行頭禁則文字（デフォルト: `゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､･ｰﾞﾟ`）
   - `chars_line_end_prohibited`: 行末禁則文字（デフォルト: `（［｛「『([{｢`）
3. If `[talk]`セクションが存在しない場合, then the pasta_lua shall 組み込みデフォルト値を使用する
4. If 設定値が不正な型の場合, then the pasta_lua shall 警告ログを出力し、デフォルト値を使用する

### Requirement 4: トークン分解

**Objective:** As a システム, I want talk文字列を文字種別ごとにトークン分解できること, so that 適切なウェイトルールを各トークンに適用できる

#### Acceptance Criteria

1. The pasta_sakura_script_wait shall 入力文字列を以下のトークン種別に分解する:
   - `sakura_script`: さくらスクリプトタグ（正規表現: `\\[0-9a-zA-Z_!]+(?:\[[^\]]*\])?`）
   - `period`: 句点文字（pasta.tomlの`chars_period`で定義）
   - `comma`: 読点文字（pasta.tomlの`chars_comma`で定義）
   - `exclaim`: 感嘆詞文字（pasta.tomlの`chars_exclaim`で定義）
   - `leader`: リーダー文字（pasta.tomlの`chars_leader`で定義）
   - `line_start_prohibited`: 行頭禁則文字（pasta.tomlの`chars_line_start_prohibited`で定義）
   - `line_end_prohibited`: 行末禁則文字（pasta.tomlの`chars_line_end_prohibited`で定義）
   - `general`: 上記以外の一般文字
2. The pasta_sakura_script_wait shall トークン分解時にUnicode文字を正しく処理する
3. The pasta_sakura_script_wait shall さくらスクリプトタグを最優先でマッチングし、タグ内部を分解しない

### Requirement 5: ウェイト挿入ルール

**Objective:** As a システム, I want 文字種別に応じた適切なウェイトを挿入できること, so that 自然な会話テンポを実現できる

#### Acceptance Criteria

1. The pasta_sakura_script_wait shall `sakura_script`トークンと`line_end_prohibited`トークンにはウェイトを挿入しない
2. When `general`トークンの場合, the pasta_sakura_script_wait shall 各文字の直後に`(script_wait_normal - 50)`ミリ秒のウェイトを挿入する（計算結果が0以下の場合は挿入しない）
3. When `leader`トークンの場合, the pasta_sakura_script_wait shall 各文字の直後に`(script_wait_leader - 50)`ミリ秒のウェイトを挿入する（計算結果が0以下の場合は挿入しない）
4. While `period`, `comma`, `exclaim`, `line_start_prohibited`トークンが連続している間, the pasta_sakura_script_wait shall ウェイトを挿入しない
5. When `period`, `comma`, `exclaim`, `line_start_prohibited`トークンの連続が終了したとき, the pasta_sakura_script_wait shall 連続区間内で最大のウェイト値から50ミリ秒を引いた値のウェイトを挿入する
6. The pasta_sakura_script_wait shall ウェイトを`\_w[ms]`形式のさくらスクリプトとして挿入する

### Requirement 6: エラーハンドリング

**Objective:** As a ゴースト開発者, I want エラー時に適切なフォールバック動作が行われること, so that ゴーストの動作が中断しない

#### Acceptance Criteria

1. If talk文字列がnilまたは空の場合, then the pasta_sakura_script_wait shall 空文字列を返す
2. If トークン分解中にエラーが発生した場合, then the pasta_sakura_script_wait shall 元のtalk文字列をそのまま返し、警告ログを出力する
3. If ウェイト値の計算結果がオーバーフローする場合, then the pasta_sakura_script_wait shall 安全な最大値（i32::MAX）を使用する

### Requirement 7: 出力例

**Objective:** As a テスト担当者, I want 期待される出力形式を明確に理解できること, so that テストケースを正確に作成できる

#### Acceptance Criteria

1. When 入力が `」」」！？。、` でscript_wait_periodが1000の場合, the pasta_sakura_script_wait shall `」」」！？。、\_w[950]` を出力する（最大ウェイト1000から50を引いた値）
2. When 入力が `こんにちは` でscript_wait_normalが100の場合, the pasta_sakura_script_wait shall 各文字の後に`\_w[50]`を挿入する
3. When 入力が `\h\s[0]こんにちは` の場合, the pasta_sakura_script_wait shall さくらスクリプト`\h\s[0]`をそのまま保持し、後続の文字にのみウェイトを挿入する
