# Requirements Document

## Project Description (Input)

さくらスクリプトを含む日本語の文字列を解析し、所定の幅を超えないようにさくらスクリプト改行（`\n`）を挿入する関数を `pasta_lua` クレートに実装する。budoux による日本語分割と unicode_width による文字幅計算を用い、さくらスクリプトタグを透過的に扱う。`pasta.toml` のアクター設定で有効化し、`wait_inserter` の後段処理として Lua から呼び出す。

## Introduction

バルーン（吹き出し）上で日本語テキストを表示する際、行幅が固定されているにもかかわらず改行が入らないと、テキストが途中で切れたり不自然な折り返しが発生する。budoux の日本語分割モデルを活用し、自然な位置でさくらスクリプト改行を自動挿入することで、ゴースト作者が手動で改行位置を調整する負荷を軽減する。

## Requirements

### Requirement 1: クレート依存関係の追加

**Objective:** As a エンジン開発者, I want `pasta_lua` に `budoux` と `unicode-width` クレートを導入する, so that 日本語分割と文字幅計算の基盤が確保される

#### Acceptance Criteria
1. The pasta_lua crate shall `budoux` クレートを依存関係に含む
2. The pasta_lua crate shall `unicode-width` クレートを依存関係に含む
3. When ワークスペースをビルドした場合, the pasta_lua crate shall コンパイルエラーなくビルドが成功する

---

### Requirement 2: さくらスクリプト透過処理

**Objective:** As a エンジン開発者, I want さくらスクリプトタグを改行計算から除外しつつ最終出力に保持する, so that テキスト幅の計算が正確でありながらさくらスクリプトの機能が損なわれない

#### Acceptance Criteria
1. When さくらスクリプトタグ（例: `\_w[50]`, `\_q`, `\s[0]`, `\n` 等、`Tokenizer::SAKURA_TAG_PATTERN` にマッチするすべてのタグ）を含む文字列が入力された場合, the budoux-line-breaker shall さくらスクリプトタグを改行位置計算の対象から除外する
2. When さくらスクリプトタグを含む文字列を処理した場合, the budoux-line-breaker shall 最終出力にすべてのさくらスクリプトタグを元の位置関係を保って含める
3. When `こ\_w[50]れ\_w[50]は\_w[50]テ\_w[50]ス\_w[50]ト` を幅 `[6]` で処理した場合, the budoux-line-breaker shall `こ\_w[50]れ\_w[50]は\_w[50]\nテ\_w[50]ス\_w[50]ト` のように、タグを幅計算から除外しつつ適切な位置にさくらスクリプト改行を挿入する（タグなしテキスト `これはテスト` のCJK幅合計12、閾値6で3文字/行）
4. The budoux-line-breaker shall さくらスクリプトタグの除去と再合成を単一パスまたは線形時間の処理で実現する

---

### Requirement 3: budoux による日本語分割

**Objective:** As a エンジン開発者, I want budoux の日本語モデルで適切な分割位置を判定する, so that 自然な日本語の折り返しが実現される

#### Acceptance Criteria
1. The budoux-line-breaker shall budoux のデフォルト日本語モデル（`default_japanese_model`）を使用して分割位置を判定する
2. When 平文（さくらスクリプトタグなし）の日本語文字列が入力された場合, the budoux-line-breaker shall budoux の分割結果に基づいて改行位置を決定する
3. The budoux-line-breaker shall budoux に渡す文字列からさくらスクリプトタグを事前除去する

---

### Requirement 4: 行幅閾値による改行挿入

**Objective:** As a ゴースト作者, I want 行ごとの最大幅を指定して改行位置を制御する, so that バルーン上での表示レイアウトを調整できる

#### Acceptance Criteria
1. When 幅閾値スライス `[w1, w2]` が与えられた場合, the budoux-line-breaker shall 1行目が `w1` 文字幅を超えないように改行を挿入する
2. When 幅閾値スライス `[w1, w2]` が与えられた場合, the budoux-line-breaker shall 2行目が `w2` 文字幅を超えないように改行を挿入する
3. When 幅閾値スライス `[w1, w2]` が与えられた場合, the budoux-line-breaker shall 3行目以降がスライス末尾の値（`w2`）を超えないように改行を挿入する
4. The budoux-line-breaker shall 文字幅の計算に `unicode-width` の CJK 幅（`width_cjk`）を使用する
5. The budoux-line-breaker shall 改行の挿入にさくらスクリプトの改行タグ `\n` を使用する

---

### Requirement 5: pasta.toml アクター設定

**Objective:** As a ゴースト作者, I want `pasta.toml` のアクター定義で budoux 処理を有効/無効に制御する, so that アクターごとに改行処理の適用を選択できる

#### Acceptance Criteria
1. Where アクター設定に `budoux` フィールドが存在する場合, the pasta runtime shall そのアクターの出力に対して budoux 改行処理を適用する
2. Where アクター設定に `budoux` フィールドが存在しない場合, the pasta runtime shall そのアクターの出力に対して budoux 改行処理を適用しない
3. The `budoux` field shall 整数の配列（例: `[10, 12]`）として行幅閾値を受け付ける
4. When `pasta.toml` に以下の設定がある場合:
   ```toml
   [actor."女の子"]
   spot = 0
   budoux = [10, 12]
   ```
   the CONFIG table shall `CONFIG.actor["女の子"].budoux` として Lua からアクセス可能である

---

### Requirement 6: Lua API 公開

**Objective:** As a エンジン開発者, I want budoux 改行関数を `@pasta_sakura_script` モジュール経由で Lua に公開する, so that Lua 側から呼び出し可否を判定して処理を適用できる

#### Acceptance Criteria
1. The `@pasta_sakura_script` module shall budoux 改行処理関数を Lua から呼び出し可能な関数として公開する
2. When Lua から呼び出された場合, the budoux-line-breaker function shall 処理対象の文字列と幅閾値配列を引数に受け取る
3. When Lua から呼び出された場合, the budoux-line-breaker function shall 改行挿入済みの文字列を返す
4. The budoux-line-breaker function shall `wait_inserter.rs` と同一ディレクトリ（`sakura_script/`）内に独立したファイル・関数として実装する

---

### Requirement 7: 処理パイプラインへの統合

**Objective:** As a エンジン開発者, I want budoux 処理を wait_inserter の後段に配置する, so that ウェイト挿入済みのテキストに対して正しく改行が挿入される

#### Acceptance Criteria
1. When Lua 側でトーク出力を処理する際, the pasta runtime shall `wait_inserter` の処理完了後に budoux 改行処理を実行する
2. When 処理中のアクターオブジェクトに `budoux` フィールドが存在する場合, the Lua layer shall budoux 改行処理を呼び出す
3. When 処理中のアクターオブジェクトに `budoux` フィールドが存在しない場合, the Lua layer shall budoux 改行処理をスキップする

---

### Requirement 8: テスト

**Objective:** As a エンジン開発者, I want budoux 改行処理の正確性を自動テストで検証する, so that リグレッションを防止できる

#### Acceptance Criteria
1. The budoux-line-breaker shall さくらスクリプトタグなしの平文に対する改行挿入のユニットテストを持つ
2. The budoux-line-breaker shall さくらスクリプトタグを含むテキストに対する改行挿入のユニットテストを持つ
3. The budoux-line-breaker shall 複数行にまたがる幅閾値制御のユニットテストを持つ
4. When `cargo test -p pasta_lua` を実行した場合, the test suite shall すべてのbudoux関連テストが成功する
