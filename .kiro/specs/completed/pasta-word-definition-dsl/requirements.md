# Requirements Document: pasta-word-definition-dsl

## Introduction

本要件は、Pasta DSLにおける単語定義機能の追加を定義します。現在のPasta DSLでは、ランダムに選択される単語や語句を定義する際に、前方一致ラベル（例：`＊場所ー東京`、`＊場所ー大阪`）を使う必要があり、冗長でした。

本機能により、`＠場所：東京　大阪`という簡潔な構文で単語リストを定義でき、会話内から`＠場所`で参照できるようになります。これは既存の前方一致ラベル機構を補完し、より軽量で直感的な単語管理を実現します。

### Syntax Overview

```pasta
# グローバル単語定義（インデントなし、ファイル内どこでも配置可能）
＠場所：東京　ニューヨーク　千葉
＠場所：カナダ　オランダ　いつか見た場所  # 同名は自動マージ

# ローカル単語定義（インデント付き、グローバルラベル直後の宣言部のみ）
＊会話
　＠天気：晴れ　雨　曇り　「晴れか、　あめ」
　＠場所＿外国：パリ　ロンドン
　
　さくら：今日は＠天気ですね
　さくら：＠場所に行こう  # 前方一致: 場所、場所＿外国がヒット、両方の単語を候補にマージ
```

### Core Design Decisions

以下の設計決定により、実装可能で保守性の高い仕様とします：

1. **AST構造**: `PastaFile::global_words: Vec<WordDef>`と`LabelDef::local_words: Vec<WordDef>`として表現
   - Statementパターンではなく、属性（Attribute）やパラメータ（params）と同じメタデータパターン
   - 理由: 単語定義は宣言的データであり、実行時の「文」ではない

2. **非マージ収集戦略**: 単語定義をエントリ単位（WordEntry）で個別保持
   - 同名定義を早期マージせず、エントリのリストとして保持
   - 理由: 将来の属性機能（`＠key＆attr:value：word1 word2`）に対応するため
   - 各エントリは独立して属性を持ち、検索時に属性フィルタリング可能にする

3. **キー形式とスコープ管理**:
   - グローバル単語: キー = `"単語名"` （例: `"挨拶"`）
   - ローカル単語: キー = `":モジュール名:単語名"` （例: `":会話_1:挨拶"`）
   - コロンプレフィックス（`:`）により、前方一致検索でグローバルとローカルが衝突しない
   - モジュール名 = グローバルラベル名のサニタイズ版（`LabelRegistry`と同じロジック）

4. **検索・キャッシュ戦略**:
   - 検索: ローカル（`:module:key`前方一致）+ グローバル（`key`前方一致）の候補を統合
   - マージ: すべてのマッチしたエントリの単語リストを結合（`Vec::extend`）
   - キャッシュ: 統合リストを1回シャッフルし、順次消費（`LabelTable::CachedSelection`パターン踏襲）
   - 再シャッフル: キャッシュ枯渇時に全単語を再シャッフル

5. **データ構造設計**:
   - トランスパイラー層: `WordDefRegistry`（`LabelRegistry`パターン踏襲）
     - Pass1で`WordDef`を収集し、`WordEntry { key: String, values: Vec<String> }`に変換
     - グローバル/ローカルを区別してエントリリストに追加
   - ランタイム層: `WordTable`（`LabelTable`パターン踏襲）
     - `RadixMap<Vec<usize>>`で前方一致インデックス構築（キー → エントリIDリスト）
     - `entries: Vec<WordEntry>`で実体を保持
     - `cached_selections: HashMap<String, CachedSelection>`でシャッフルキャッシュ管理

6. **トランスパイラー出力仕様**:
   - グローバル単語参照: `＠単語名` → `yield Talk(pasta_stdlib::word("", "単語名", []))`
   - ローカル単語参照: `＠単語名` → `yield Talk(pasta_stdlib::word("モジュール名", "単語名", []))`
   - `pasta_stdlib::word(module_name, key, filters) -> String`は純粋な文字列を返却
   - `Talk()`によるラッピングはトランスパイラーが生成
   - モジュール名はトランスパイラーが現在のコンテキストから決定（グローバルスコープでは空文字列）

---

## Requirements

> **Note**: パーサー層の単語定義構文解析（`＠単語名：単語1　単語2`のパース、`WordDef`生成、エラーハンドリング等）は既に実装済みです。本要件はトランスパイラー層とランタイム層の実装に焦点を当てます。

### Requirement 1: グローバルスコープ単語定義

**Objective:** As a Pastaトランスパイラー, I want パーサーが生成した`PastaFile::global_words`を処理できる, so that ランタイムで使用可能な単語辞書を構築できる

#### Acceptance Criteria

1. The Pasta Transpiler shall `PastaFile::global_words: Vec<WordDef>`を入力として受け取る（パーサーが生成済み）
2. The Pasta Transpiler shall グローバル単語定義を`WordEntry { key: String, values: Vec<String> }`形式で収集する
3. The Pasta Transpiler shall グローバル単語のキーを`"単語名"`形式で登録する（例: `"挨拶"`）
4. When 同じ名前のグローバル単語定義が複数行で定義された場合, the Pasta Transpiler shall 各定義を独立した`WordEntry`として保持する（早期マージしない）
5. The Pasta Transpiler shall 同名エントリの単語リストを検索時にマージし、統合候補リストを作成する
6. The Pasta Transpiler shall マージ時に重複する単語を排除せず、そのまま保持する（重複登録で選択確率を調整可能）
7. The Pasta Runtime shall グローバル単語エントリを`WordTable`で管理し、前方一致インデックス（`RadixMap<Vec<usize>>`）を構築する
   - インデックスは「プレフィックスキー → マッチしたエントリIDのリスト」を保持
   - 各キーでマッチしたエントリの`values`を統合して検索結果を作成

### Requirement 2: ローカルスコープ単語定義

**Objective:** As a Pastaトランスパイラー, I want パーサーが生成した`LabelDef::local_words`を処理できる, so that モジュールスコープに紐づく単語辞書を構築できる

#### Acceptance Criteria

1. The Pasta Transpiler shall `LabelDef::local_words: Vec<WordDef>`を入力として受け取る（パーサーが生成済み）
2. The Pasta Transpiler shall ローカル単語定義を`WordEntry { key: String, values: Vec<String> }`形式で収集する
3. The Pasta Transpiler shall ローカル単語のキーを`":モジュール名:単語名"`形式で登録する（例: `":会話_1:挨拶"`）
4. The Pasta Transpiler shall モジュール名をグローバルラベル名のサニタイズ版として生成する（`LabelRegistry::sanitize_name()`と同じロジック）
5. The Pasta Transpiler shall ローカル単語参照時に現在のモジュール名を`word()`関数の第2引数として渡す
6. When `pasta_stdlib::word("会話_1", "挨拶", [])`が呼ばれた場合, the Pasta Runtime shall `":会話_1:挨拶"`前方一致でローカルエントリを検索する
7. When ローカルとグローバルで同じ単語名が定義された場合, the Pasta Runtime shall 両方の候補を統合してマージする（ローカル優先ではなく、統合リストから選択）

### Requirement 3: 会話内での単語参照と展開

**Objective:** As a Pastaトランスパイラー/ランタイム, I want 会話行内の単語参照を適切なRune コードに変換し、実行時に単語辞書から選択できる, so that ランダムな単語展開を実現できる

#### Acceptance Criteria

1. The Pasta Parser shall 会話行（Speech）内の`＠単語名`を`SpeechPart::FuncCall { name, args, scope }`として解析する（既存実装）
2. When トランスパイラーが`SpeechPart::FuncCall`を処理する場合, the Pasta Transpiler shall `yield Talk(pasta_stdlib::word("モジュール名", "単語名", [filters]));` 形式のRune コードを生成する（グローバルスコープでは第2引数は空文字列）
3. The `pasta_stdlib::word(module_name, key, filters) -> String` function shall 純粋な文字列を返却し、`Talk()`によるラッピングはトランスパイラーが生成する
4. When 単語辞書で前方一致する定義が見つかった場合, the Pasta Runtime shall すべてのマッチした単語定義の単語を統合してシャッフルし、ランダムに1つを選択する
5. When 選択された単語が見つかった場合, the Pasta Runtime shall その単語文字列を返却する
6. When `＠name`が単語辞書で見つからない場合, the Pasta Runtime shall エラーログを出力し、空文字列として処理を継続する（panic禁止）
7. The Pasta Runtime shall 同一会話行内で複数の単語参照があった場合、それぞれ独立してランダム選択を実行する
8. The Pasta Runtime shall エラーハンドリングに`Result`型を使用し、実行時エラーでもスクリプト実行を停止しない

### Requirement 4: 前方一致による複合検索

**Objective:** As a Pastaランタイム, I want 単語参照時に前方一致で複数の定義をマッチさせ、統合できる, so that 柔軟な単語グループ化を実現できる

#### Acceptance Criteria

1. When `pasta_stdlib::word("会話_1", "場所", [])`が呼ばれた場合, the Pasta Runtime shall 以下の2段階検索を実行する：
   - ローカル検索: `":会話_1:場所"`前方一致（例: `":会話_1:場所"`, `":会話_1:場所_日本"`）
   - グローバル検索: `"場所"`前方一致（例: `"場所"`, `"場所_外国"`）
2. The Pasta Runtime shall ローカル検索とグローバル検索の候補を統合し、単一の候補リストを作成する
3. When 複数のエントリにマッチした場合, the Pasta Runtime shall すべてのエントリの`values`を`Vec::extend`でマージする
4. The Pasta Runtime shall マージした単語リストを1回シャッフルし、`CachedSelection`に格納する
5. The Pasta Runtime shall シャッフルキャッシュから順次単語を取り出し（Pop方式）、毎回異なる単語を返却する
6. When キャッシュの残り単語が0になった場合, the Pasta Runtime shall 全単語リストを再シャッフルしてキャッシュを再構築する
7. The Pasta Runtime shall 前方一致インデックスに`RadixMap<Vec<usize>>`を使用する（`fast_radix_trie` crate）
8. The Pasta Runtime shall キャッシュキーを`(module_name, search_key)`として管理し、モジュール間でキャッシュを分離する

### Requirement 5: AST構造と内部データ表現

**Objective:** As a Pasta Parser, I want 単語定義を適切なAST構造で表現できる, so that トランスパイラとランタイムで一貫した処理が可能になる

#### Acceptance Criteria

1. The Pasta Parser shall `PastaFile`に`global_words: Vec<WordDef>`フィールドを保持する（既存実装）
2. The Pasta Parser shall `LabelDef`に`local_words: Vec<WordDef>`フィールドを保持する（既存実装）
3. The Pasta AST shall `WordDef`構造体を以下のフィールドで定義する：`name: String`, `values: Vec<String>`, `span: Span`（既存実装）
4. When 単語定義のパースが完了した場合, the Pasta Parser shall 単語名の妥当性を検証し（識別子規則）、単語リストが非空であることを確認する
5. The Pasta Transpiler shall `WordDefRegistry`構造体を実装し、以下の責務を持たせる：
   - `WordEntry { key: String, values: Vec<String> }`のリスト管理
   - グローバル単語の登録（`register_global(name, values)`）
   - ローカル単語の登録（`register_local(module_name, name, values)`）
   - エントリIDの自動採番とトラッキング
6. The Pasta Transpiler shall Pass1で`PastaFile::global_words`と`LabelDef::local_words`を走査し、`WordDefRegistry`に登録する
7. The Pasta Runtime shall `WordTable`構造体を実装し、以下の責務を持たせる：
    - `entries: Vec<WordEntry>`で単語エントリを保持
    - `prefix_index: RadixMap<Vec<usize>>`で前方一致インデックス構築（プレフィックス → マッチしたエントリIDリスト）
    - `cached_selections: HashMap<(String, String), CachedSelection>`でシャッフルキャッシュ管理（キー: `(module_name, search_key)`）
    - `search_word(module_name, key) -> String`でランダム単語選択：
       - **ステップ1: ローカル検索**
          - `prefix_index`で`":module_name:key"`前方一致を実行（呼び出し元は常にアクション行＝ローカルスコープ。`__start__`もローカルなので必ずモジュール名が渡される）
          - マッチしたエントリIDリストを取得（ヒットなしなら空リスト）
       - **ステップ2: グローバル検索（常に実行）**
          - `prefix_index`で`key`前方一致を実行
          - マッチしたエントリIDリストを取得
       - **ステップ3: 統合とマージ**
          - ローカル検索とグローバル検索のエントリIDリストを結合
          - 結合IDリストから各`entries[id].values`を順次取得
          - `Vec::extend`ですべての単語リストをマージ
       - **ステップ4: キャッシュ処理**
          - キャッシュキー`(module_name, key)`でマッチを確認
          - キャッシュ未作成時：マージした単語リストをシャッフルして`CachedSelection`に格納
          - キャッシュ存在時：残り単語から1つをPop
          - 残り単語がない場合：全単語を再シャッフル
       - **ステップ5: 返却**
          - 選択された単語を返却
          - マッチなし時は空文字列を返却
8. The Pasta Runtime shall ランダム選択時の公平性を保証する（シャッフルにより各単語の選択確率が均等）

### Requirement 6: Call/Jump文からの単語辞書非アクセス

**Objective:** As a Pastaランタイム, I want 単語定義をcall/jump文から呼び出せないようにする, so that ラベルと単語定義の責務を明確に分離できる

#### Acceptance Criteria

1. When Call文（`＞name`）またはJump文（`－name`）で名前が指定された場合, the Pasta Runtime shall Rune関数呼び出し→前方一致ラベル検索のみを実行し、単語辞書は検索対象としない
2. When call/jumpで指定された名前がRune関数・ラベルとして存在せず、単語定義として存在する場合, the Pasta Runtime shall 関数/ラベル未定義エラーを発生させる
3. The Pasta Runtime shall 単語定義とラベル定義を別のデータ構造で管理し、call/jump処理フェーズでは単語辞書にアクセスしない
4. The Pasta Documentation shall call/jumpから単語辞書がアクセスされないことを明記する

### Requirement 7: エラーハンドリングと診断

**Objective:** As a Pastaランタイム, I want 単語検索時のエラーを適切に処理できる, so that スクリプト実行を継続できる

#### Acceptance Criteria

1. When `＠name`が単語辞書で見つからない場合, the Pasta Runtime shall エラーログを出力し、空文字列として処理を継続する（panic禁止）
2. The Pasta Runtime shall エラーハンドリングに以下の2層戦略を採用する：
   - **内部API** (`WordTable::search_word`): `Result<String, PastaError>`を返却し、未ヒット時は`Err(PastaError::WordNotFound)`
   - **公開API** (`pasta_stdlib::word`): `String`を返却し、エラー時はログ発行のみで空文字列を返却（Rune側にエラーを伝播させない、no panic原則）
   - この設計によりテスト可能性を確保しつつ、Rune側でのエラー処理実装困難性を回避する
3. The Pasta Error Messages shall 日本語でわかりやすいメッセージを提供する（例：「単語定義 @場所 が見つかりません」）

### Requirement 8: ドキュメント更新

**Objective:** As a Pasta DSL ユーザー, I want 単語定義機能の使い方を理解できる, so that 効果的にスクリプトを作成できる

#### Acceptance Criteria

1. The Pasta Documentation shall GRAMMAR.mdに「単語定義」セクションを追加し、構文形式を説明する（例：`＠単語名：単語1　単語2`）
2. The Pasta Documentation shall グローバル/ローカルスコープの使い分け方法を説明する（ラベル外=グローバル、ラベル内=ローカル）
3. The Pasta Documentation shall 会話行内からの参照方法と複数例を提供する
4. The Pasta Documentation shall 単語辞書検索の前方一致ロジックを説明する（ローカル→グローバル統合マージ、複数マッチ時は全単語統合）
5. The Pasta Documentation shall call/jumpから単語辞書がアクセスされないことを明記する
6. The Pasta Documentation shall 前方一致検索の動作を具体例で説明する（例：`＠挨`が`＠挨拶`や`＠挨拶_朝`にマッチする）
7. The Pasta Documentation shall 同名単語定義の自動マージ動作を説明する
8. The Pasta Documentation shall 引用符エスケープの動作を説明する（例：`「「test」」` → `「test」`）
9. The Pasta Documentation shall 最低3つの実用的なサンプルスクリプトを提供する（グローバル/ローカル単語定義、前方一致、複合検索の各ケース）

### Requirement 9: テスト可能性と検証

**Objective:** As a 開発チーム, I want 単語定義機能の各要素をユニットテストできる, so that 品質を保証できる

#### Acceptance Criteria

1. The Pasta Test Suite shall トランスパイラテストで単語辞書の統合マージ（WordDefRegistry）を検証する
2. The Pasta Test Suite shall ランタイムテストで単語検索・キャッシュ・前方一致（WordTable）を検証する
3. The Pasta Test Suite shall 前方一致のエッジケース（単語名の前方一致、複数マッチ、マッチなし）を検証する
4. The Pasta Test Suite shall シャッフルキャッシュの動作（初回シャッフル、再シャッフル）を検証する
5. The Pasta Test Suite shall call/jumpから単語辞書がアクセスされないことを検証する

---

## Summary of Requirement Areas

本要件定義は、**トランスパイラー層とランタイム層**の実装に焦点を当て、以下の要件領域を網羅します：

1. **トランスパイラ層**: WordDefRegistry実装、単語エントリ収集、キー形式管理、Rune コード生成
2. **ランタイム層**: WordTable実装、前方一致検索、シャッフルキャッシング、ランダム選択
3. **スコープ管理**: グローバル/ローカル定義の統合マージ戦略
4. **エラーハンドリング**: Graceful degradation（パニック不可）
5. **ドキュメント**: 使用例、API仕様
6. **テスト**: ユニットテストと統合テストの包括的カバレッジ

**パーサー層の前提条件**: 以下は既に実装済みとしてスコープ外とします：
- 単語定義構文のパース（`＠単語名：単語1　単語2`）
- `WordDef`構造体の生成
- `PastaFile::global_words`および`LabelDef::local_words`への格納
- 引用符エスケープ、識別子検証、構文エラーハンドリング

````
