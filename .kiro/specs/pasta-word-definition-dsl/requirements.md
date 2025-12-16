# Requirements Document

## Project Description (Input)
pasta DSLで、現在、「＊挨拶」など、会話ブロックについては問題なく記載できるが、単語定義を行いたい場合に冗長である。単語定義のためのDSLを新たに要件定義し、実装に加えよ。単語定義について、会話内の「＠単語」などから呼び出せるようにせよ。グローバルセクションにも、ローカルセクションにも書けるようにせよ。

**確定した構文**（議題1で決定）:
```pasta
# グローバル単語定義（インデントなし）
＠場所：東京　ニューヨーク　千葉
＠場所：カナダ　オランダ　いつか見た場所  # 同名は自動マージ

# ローカル単語定義（インデント付き、グローバルラベル内）
＊会話
　＠天気：晴れ　雨　曇り　「晴れか、　あめ」
　＠場所＿外国：パリ　ロンドン
　
　さくら：今日は＠天気　ですね
　さくら：＠場所　に行こう  # 前方一致: 場所、場所＿外国がヒット
```

**設計決定事項**:
- **記号統一**: 単語定義と参照の両方で`＠`を使用（`＄`から変更）
- **フォールバック検索**: `＠場所`はRune変数/関数→単語辞書→前方一致ラベルの順で検索
- **前方一致**: `＠場所`は`場所`で始まるすべての単語定義・ラベルを候補に含む
- **自動マージ**: 同じ名前の単語定義は自動的に統合される
- **独立した辞書**: 単語辞書とラベル辞書は別のデータ構造で管理
- **`＠＊`廃止**: グローバル明示構文は不要（フォールバック機構で自動解決）
- **配置制約**（議題2で決定）:
  - グローバル単語定義（インデントなし）: ファイル内のどこでも配置可能
  - ローカル単語定義（インデントあり）: グローバルラベル直後の宣言部のみ配置可能（会話行・ローカルラベル・ジャンプ・コール文の後は不可）
  - 変数定義との違い: 変数定義は実行部でも可能だが、単語定義は宣言部のみ
- **エスケープ処理**（議題4で決定）:
  - 二重引用符エスケープ: `「「` → `「`、`」」` → `」`
  - 例: `「彼女は「「ありがとう」」と言った」` → `彼女は「ありがとう」と言った`
- **命名規則**（議題5で決定）:
  - Rust識別子ルール適用（ラベル名と同じ）
  - 使用可能: ASCII英数字・アンダースコア、Unicode XID_Start/XID_Continue（日本語含む）
  - 禁止: 数字始まり、予約記号（`＠`, `＄`, `＞`等）、空白、引用符
- **マージ順序**（議題6で決定）:
  - 順序保証なし（ランダム選択なので順序は無意味）
  - 重複許容（同じ単語を複数回登録で選択確率調整可能）
  - ファイル読み込み順序は環境依存
- **AST表現**（議題7で決定）:
  - `Statement::WordDef`として実装（既存のStatement enumにvariant追加）
  - 既存のStatement処理フローに統合、パーサー変更最小限

---

## Introduction

本要件は、Pasta DSLにおける単語定義機能の追加を定義します。現在のPasta DSLでは、ランダムに選択される単語や語句を定義する際に、前方一致ラベル（例：`＊場所ー東京`、`＊場所ー大阪`）を使う必要があり、冗長でした。

本機能により、`＠場所：東京　大阪`という簡潔な構文で単語リストを定義でき、会話内から`＠場所　`で参照できるようになります。これは既存の前方一致ラベル機構を補完し、より軽量で直感的な単語管理を実現します。

## Requirements

### Requirement 1: 単語定義構文

**Objective:** As a スクリプト作成者, I want 単語リストを簡潔に定義できる構文, so that 同じ種類の単語をまとめて管理し、ランダムに選択できる

#### Acceptance Criteria

1. When パーサーが`＠単語名：単語1　単語2　単語3`形式の行を検出した場合, the Pasta Parser shall 単語定義として解析する（**記号を`＄`から`＠`に変更**）
2. The Pasta Parser shall 単語名がRust識別子規則に従うことを検証する（ASCII: a-z, A-Z, 0-9, _、Unicode: XID_Start/XID_Continue、数字始まり禁止）
3. The Pasta Parser shall 全角スペースまたはタブ文字を単語の区切り文字として認識する
4. When 引用符`「」`で囲まれた文字列が定義された場合, the Pasta Parser shall 内部の全角スペースを区切り文字として扱わず、一つの単語として認識する
5. The Pasta Parser shall 引用符内で`「「`を`「`に、`」」`を`」`に変換する（二重引用符エスケープ）
6. When 同じ単語名が複数回定義された場合, the Pasta Parser shall すべての定義を自動的にマージし、単一の単語リストとして扱う
7. When `＠単語名＝`のように単語リストが空の場合, the Pasta Parser shall 構文エラーとして検出し、エラーを記録してパースを継続する
6. The Pasta Parser shall すべての構文エラーを収集した後、`Result::Err`でエラーを返す（panic禁止）
7. The Pasta Parser shall 空の単語定義を単語辞書に登録せず、マージ対象から除外する
8. When 会話行内で`＠場所　`のように単語参照があった場合, the Pasta Runtime shall `場所`で始まるすべての単語定義（例：`場所`, `場所＿日本`, `場所＿外国`）を前方一致検索し、すべての単語を候補として列挙する

### Requirement 2: グローバルスコープ単語定義

**Objective:** As a スクリプト作成者, I want ファイル全体で使用できる単語を定義できる, so that 複数の会話ブロックから共通の単語セットを参照できる

#### Acceptance Criteria

1. When 単語定義行がインデントなしで記述された場合, the Pasta Parser shall グローバルスコープの単語定義として登録する
2. The Pasta Parser shall インデントなしの単語定義をファイル内のどこにでも配置可能とする（ラベル定義の前後、ラベル内外を問わず）
3. The Pasta Runtime shall グローバルスコープの単語定義をスクリプト全体で参照可能にする
4. When 複数のファイルまたは複数行で同じグローバル単語名が定義された場合, the Pasta Runtime shall すべての定義を統合マージし、単一の単語リストとして管理する
5. The Pasta Runtime shall マージ時に重複する単語を排除せず、そのまま保持する（重複登録で選択確率を調整可能）
6. The Pasta Runtime shall マージ順序を保証せず、ファイル読み込み順序は環境依存とする（ランダム選択なので順序は非本質的）
7. The Pasta Runtime shall グローバル単語定義をファイル解析時に登録し、実行開始前に利用可能にする
8. The Pasta Runtime shall グローバル単語定義を`HashMap<String, Vec<String>>`形式で管理する（キー：単語名、値：単語リスト）

### Requirement 3: ローカルスコープ単語定義

**Objective:** As a スクリプト作成者, I want 特定の会話ブロック内でのみ使用する単語を定義できる, so that 会話のコンテキストに応じた単語セットを管理できる

#### Acceptance Criteria

1. When 単語定義行がラベル定義内でインデント付きで記述された場合, the Pasta Parser shall ローカルスコープの単語定義として登録する
2. The Pasta Parser shall インデント付き単語定義をグローバルラベル定義直後の宣言部（属性設定・変数定義と同じブロック）のみ配置可能とする
3. The Pasta Parser shall 会話行・ローカルラベル・ジャンプ文・コール文の後に単語定義が現れた場合、構文エラーとする
4. The Pasta Parser shall 変数定義（`＄variable = value`）は実行部でも配置可能だが、単語定義は宣言部のみに制限する
5. The Pasta Runtime shall ローカルスコープの単語定義を当該グローバルラベル実行中のみ参照可能にする
6. When ローカルとグローバルで同じ単語名が定義された場合, the Pasta Runtime shall ローカル定義を優先して参照する（シャドーイング）
7. The Pasta Runtime shall ラベル実行終了時にローカル単語定義のスコープを解放する
8. The Pasta Runtime shall ローカル単語定義を`HashMap<(String, String), Vec<String>>`形式で管理する（キー：(グローバルラベル名, 単語名)、値：単語リスト）

### Requirement 4: 会話内からの単語参照

**Objective:** As a スクリプト作成者, I want 会話行内で単語定義を参照できる, so that ランダムに選択された単語を発言内容に埋め込める

#### Acceptance Criteria

1. When 会話行（Speech文）内に`＠単語名`形式の参照が記述された場合, the Pasta Runtime shall 対応する単語定義からランダムに一つの単語を選択する
2. The Pasta Runtime shall 選択された単語を会話内容に展開して出力する
3. When `＠name`がRune変数/関数・単語辞書・前方一致ラベルのいずれにも該当しない場合, the Pasta Runtime shall エラーログを出力し、空文字列として処理を継続する（panic禁止）
4. The Pasta Runtime shall エラーハンドリングに`Result`型を使用し、実行時エラーでもスクリプト実行を停止しない
5. The Pasta Runtime shall 同一会話行内で複数の単語参照があった場合、それぞれ独立してランダム選択を実行する
6. When 単語参照が前方一致で複数の単語定義にマッチした場合（例：`＠場所`が`場所`, `場所＿日本`, `場所＿外国`にマッチ）, the Pasta Runtime shall すべての単語定義の単語を統合し、ランダムに1つ選択する

### Requirement 5: 単語参照のフォールバック検索

**Objective:** As a Pasta Runtime, I want 単語参照時に適切な検索順序で定義を解決できる, so that Rune変数・関数、単語辞書、前方一致ラベルの優先順位に従った値選択が行われる

#### Acceptance Criteria

1. When 会話行内で`＠名前　`形式の参照が発生した場合, the Pasta Runtime shall 以下の順序でフォールバック検索を実行する：
   - a. Runeスコープ検索（ローカル変数・関数 → グローバル変数・関数）
   - b. 単語辞書検索（前方一致、ローカル → グローバル）
   - c. 前方一致ラベル検索（既存機構、`＊名前ーXXX`形式）
2. When Runeスコープで変数または関数が見つかった場合, the Pasta Runtime shall その値または関数戻り値を使用し、単語辞書検索をスキップする
3. When 単語辞書検索で前方一致する定義が見つかった場合, the Pasta Runtime shall すべてのマッチした単語を統合し、シャッフル・キャッシュ機構を適用する
4. When いずれの検索でも見つからない場合, the Pasta Runtime shall 未定義エラーを発生させる
5. The Pasta Runtime shall `＠＊名前`構文（グローバル明示指定）をサポートしない（フォールバック機構で自動解決されるため廃止）

### Requirement 6: AST表現と内部データ構造

**Objective:** As a Pasta Parser, I want 単語定義を適切なAST構造で表現できる, so that トランスパイラとランタイムで一貫した処理が可能になる

#### Acceptance Criteria

1. The Pasta Parser shall 単語定義を`Statement::WordDef`として新規AST要素で表現する
2. The Pasta AST shall 単語名、単語リスト、スコープ情報（Global/Local）を保持する
3. The Pasta Runtime shall グローバル単語定義を`HashMap<String, Vec<String>>`形式で管理する
4. The Pasta Runtime shall ローカル単語定義を`HashMap<(String, String), Vec<String>>`形式で管理する（キー：(グローバルラベル名, 単語名)）
5. The Pasta Runtime shall 同じ名前の単語定義が複数回現れた場合、AST全体をスキャンして単語リストをマージする
6. The Pasta Runtime shall ランダム選択時の公平性を保証する（各単語の選択確率が均等）
7. The Pasta Transpiler shall 単語定義をRune静的変数（`GLOBAL_WORD_DICT`, `LOCAL_WORD_DICT`）に変換し、実行時に前方一致検索・シャッフル・キャッシュ機構を適用する

### Requirement 7: call/jumpからの非呼び出し制約

**Objective:** As a Pasta Runtime, I want 単語定義をcall/jump文から呼び出せないように制限する, so that ラベルと単語定義の責務を明確に分離できる

**Note:** `＞name`や`－name`構文は、Rune関数呼び出し・前方一致ラベル検索のみを実行し、単語辞書には一切アクセスしない。これにより、単語定義はテキスト置換専用の機能として明確に分離される。

#### Acceptance Criteria

1. When call文（`＞name`）またはjump文（`－name`）で名前が指定された場合, the Pasta Runtime shall Rune関数呼び出し→前方一致ラベル検索のみを実行し、単語辞書を検索対象としない
2. When call/jumpで指定された名前がRune関数・ラベルとして存在せず、単語定義として存在する場合, the Pasta Runtime shall 関数/ラベル未定義エラーを発生させる
3. The Pasta Runtime shall 単語定義とラベル定義を別のデータ構造で管理する（単語辞書 vs Runeスコープ＋前方一致ラベルマップ）
4. The Pasta Documentation shall 単語定義はcall/jumpから呼び出せないこと、単語辞書はcall/jump処理フェーズで一切参照されないことを明記する

### Requirement 8: 会話行からのラベル呼び出し非対応

**Objective:** As a Pasta Parser, I want 会話行内の`＠名前`記法を単語参照専用とする, so that 構文の一貫性を保ち、ラベル呼び出しとの混同を防ぐ

**Note:** `＠name`は会話行内で単語辞書へのアクセスを行い、フォールバック検索（Runeスコープ→単語辞書→前方一致ラベル）を適用する。これにより、単語定義が見つからない場合は自動的に前方一致ラベルへフォールバックし、柔軟な検索が可能になる。

#### Acceptance Criteria

1. When 会話行内に`＠名前`形式の記述がある場合, the Pasta Runtime shall フォールバック検索を実行する（Runeスコープ→単語辞書前方一致→前方一致ラベル）
2. When `＠名前`で指定された名前が単語辞書で前方一致した場合, the Pasta Runtime shall 単語リストからランダム選択してテキスト置換する
3. When `＠名前`で指定された名前が単語辞書で見つからない場合, the Pasta Runtime shall 前方一致ラベルへフォールバックする
4. When `＠名前`がRune変数/関数、単語定義、前方一致ラベルのいずれにも該当しない場合, the Pasta Runtime shall エラーを発生させる
5. The Pasta Runtime shall 会話行内から明示的にラベルを呼び出す専用構文を提供しない（フォールバックのみ）
6. The Pasta Documentation shall `＠名前`のフォールバック検索順序とラベル自動フォールバックの動作を明記する

### Requirement 9: エラーハンドリングと診断

**Objective:** As a スクリプト作成者, I want 単語定義に関するエラーを明確に理解できる, so that 問題を迅速に修正できる

#### Acceptance Criteria

1. When 単語定義の構文エラーが発生した場合（例：閉じ「が欠落、空の単語リスト）, the Pasta Parser shall ファイル名、行番号、エラー内容を含むメッセージを出力する
2. The Pasta Parser shall すべての構文エラーを収集した後、`Result::Err`でエラーを返す（panic禁止）
3. The Pasta Parser shall 空の単語定義（`＠単語名＝`）をエラーとして記録し、単語辞書に登録しない
4. When `＠name`がRuneスコープ・単語辞書・前方一致ラベルのいずれにも該当しない場合, the Pasta Runtime shall 参照箇所、名前、試行した検索順序を含むエラーログを出力し、空文字列として処理を継続する（panic禁止）
5. The Pasta Runtime shall エラーハンドリングに`Result`型を使用し、実行時エラーでもスクリプト実行を停止しない
6. The Pasta Error Messages shall 日本語でわかりやすいメッセージを提供する

### Requirement 10: ドキュメント更新

**Objective:** As a Pasta DSL ユーザー, I want 単語定義機能の使い方を理解できる, so that 効果的にスクリプトを作成できる

#### Acceptance Criteria

1. The Pasta Documentation shall GRAMMAR.mdに単語定義の構文セクションを追加する（`＠単語名＝「単語1」「単語2」...`形式の説明を含む）
2. The Pasta Documentation shall グローバル/ローカルスコープの使い分け方法を説明する（ラベル外=グローバル、ラベル内=ローカルの自動判定ルールを含む）
3. The Pasta Documentation shall 会話行内からの参照方法と例を提供する（`＠name`によるフォールバック検索の動作を含む）
4. The Pasta Documentation shall フォールバック検索の順序を明記する（Runeスコープ→単語辞書前方一致→前方一致ラベル）
5. The Pasta Documentation shall call/jumpからの単語辞書非アクセスの制約を明記する（`＞name`/`－name`は単語辞書を一切参照しない）
6. The Pasta Documentation shall `＠＊name`構文が非推奨であることを明記する（フォールバック検索で自動的にグローバル単語へアクセス可能）
7. The Pasta Documentation shall 前方一致検索の動作を説明する（`＠挨`が`＠挨拶`や`＠挨拶_朝`にマッチする例を含む）
8. The Pasta Documentation shall 同名単語定義の自動マージ動作を説明する（複数回定義された単語リストが結合される挙動を含む）
9. The Pasta Documentation shall 最低3つの実用的なサンプルスクリプトを提供する（グローバル/ローカル単語定義、前方一致、フォールバックの各ケースを含む）

---
