# Requirements Document

## Introduction

本仕様は、pasta_luaの「起動シーケンス」を確立する。既にトランスパイラー（`LuaTranspiler`）とランタイム（`PastaLuaRuntime`）は実装されているが、「起動ディレクトリを起点として、どのようにpastaスクリプトを発見・トランスパイル・ロードし、ランタイムを起動するか」という一連のフローが欠落している。

この仕様では、起動フォルダからPastaスクリプトを収集し、それらをトランスパイルしてLuaランタイムで実行可能な状態にするまでの起動手順を整理し、実際のブートストラップコードを実装する。

### ディレクトリ構造仕様

```
ghost/master/              # 起動ディレクトリ（PastaLoader::load引数）
├── pasta.toml            # 設定ファイル（オプション、配布ファイル）
├── dic/                  # Pastaスクリプト配置（配布ファイル）
│   ├── greeting/
│   │   ├── hello.pasta
│   │   └── goodbye.pasta
│   └── conversation/
│       └── chat.pasta
├── scripts/              # Luaスクリプト（pasta_lua固有実装、配布ファイル）
│   └── *.lua
├── scriptlibs/           # Luaライブラリ（外部ダウンロード品、配布ファイル）
│   └── *.lua
└── profile/              # ランタイムデータ（配布禁止、自動生成）
    └── pasta/            # pasta専用領域
        ├── save/         # 永続化データ
        │   ├── variables.json  # ＄＊＊システム変数
        │   └── lua/      # 永続化Luaスクリプト（最優先検索）
        │       └── *.lua
        └── cache/        # キャッシュデータ
            └── lua/      # トランスパイル済みLuaキャッシュ
                └── *.lua
```

**探索パターン**: `dic/*/*.pasta`（dicの1階層下のサブディレクトリ内の.pastaファイル）  
**除外パターン**: `profile/**`（ランタイムデータ）  
**Luaモジュール検索パス（優先順位）**: 
1. `<起動dir>/profile/pasta/save/lua/?.lua` （永続化lua、最優先）
2. `<起動dir>/scripts/?.lua` （pasta_lua固有実装）
3. `<起動dir>/profile/pasta/cache/lua/?.lua` （pasta DSLキャッシュlua）
4. `<起動dir>/scriptlibs/?.lua` （外部ライブラリ、最低優先）
※すべて起動ディレクトリ基準の絶対パス

## Requirements

### Requirement 1: 起動ディレクトリ探索

**Objective:** As a pasta_lua利用者, I want 起動ディレクトリを指定するだけでPastaスクリプトが自動収集される, so that 複雑なファイル指定なしにゴーストを起動できる

#### Acceptance Criteria
1. When 起動ディレクトリパスが指定される, the PastaLoader shall `dic/*/*.pasta` パターンで `.pasta` ファイルを収集する
2. When 指定されたディレクトリが存在しない, the PastaLoader shall 明確なエラーメッセージと共にエラーを返す
3. When `dic/` ディレクトリが存在しないまたは `.pasta` ファイルが存在しない, the PastaLoader shall 空のファイルリストを返し警告を発行する
4. The PastaLoader shall `profile/**` ディレクトリを探索対象から除外する
5. The PastaLoader shall `dic/` 直下のファイル（`dic/*.pasta`）を無視し、サブディレクトリ内のみを対象とする

### Requirement 2: 設定ファイル解釈

**Objective:** As a pasta_lua利用者, I want プロジェクト設定ファイルで起動オプションを制御したい, so that 柔軟な起動設定が可能になる

#### Acceptance Criteria
1. When 起動ディレクトリに `pasta.toml` が存在する, the PastaLoader shall 設定ファイルを読み込みデフォルト設定を上書きする
2. If 設定ファイルの解析に失敗した, the PastaLoader shall 詳細なエラー位置情報と共にエラーを返す
3. While 設定ファイルが存在しない状態, the PastaLoader shall デフォルト設定（`dic/*/*.pasta` 探索、profile除外、全モジュール有効）で動作を継続する
4. The PastaLoader shall 設定ファイルでランタイム設定（RuntimeConfig相当）を指定できる
5. The PastaLoader shall `profile/pasta/save/`, `profile/pasta/save/lua/`, `profile/pasta/cache/`, `profile/pasta/cache/lua/` ディレクトリが存在しない場合に自動作成する
6. The PastaLoader shall 設定ファイル全体をTranspileContextに保持し、ランタイムからアクセス可能にする

### Requirement 3: 複数ファイルトランスパイル

**Objective:** As a pasta_lua利用者, I want 複数のPastaファイルを一括でトランスパイルしたい, so that プロジェクト全体を1つのランタイムで実行できる

#### Acceptance Criteria
1. When 複数の `.pasta` ファイルが収集される, the LuaTranspiler shall すべてのファイルを統合してトランスパイルする
2. When トランスパイル中にエラーが発生した, the PastaLoader shall エラー発生ファイル名と行番号を含む詳細なエラー情報を返す
3. The LuaTranspiler shall 複数ファイルのシーン・単語定義を単一のレジストリに統合する
4. If 異なるファイルで同名のグローバルシーンが定義された, the SceneRegistry shall 重複シーンとしてランダム選択対象に登録する
5. When すべてのファイルのトランスパイルが完了した, the PastaLoader shall トランスパイル結果を `profile/pasta/cache/lua/` に保存する（デバッグ・検証用、キャッシュとしては再利用しない）
6. The PastaLoader shall 起動時に毎回トランスパイルを実行し、前回のキャッシュを無視する

### Requirement 4: ランタイム初期化

**Objective:** As a pasta_lua利用者, I want トランスパイル結果から自動的にランタイムを初期化したい, so that 手動のランタイム設定なしにスクリプトを実行できる

#### Acceptance Criteria
1. When TranspileContextが生成される, the PastaLuaRuntime shall 当該コンテキストからランタイムを初期化する
2. When ランタイム初期化が完了した, the PastaLuaRuntime shall 起動ディレクトリ基準の絶対パスで4階層のLuaモジュール検索パス（package.path）を設定する（`save/lua` → `scripts` → `cache/lua` → `scriptlibs` の優先順位）
3. When ランタイム初期化が完了した, the PastaLuaRuntime shall `profile/pasta/cache/lua/` に保存されたトランスパイル結果をロードする
4. While ランタイム設定（RuntimeConfig）が明示指定されている, the PastaLuaRuntime shall 指定設定でモジュールを初期化する
5. If Lua VMの初期化に失敗した, the PastaLuaRuntime shall 失敗理由を含むエラーを返す
6. The PastaLuaRuntime shall カレントディレクトリの変更に影響されずモジュールをロードできる

### Requirement 5: 統合起動API

**Objective:** As a pasta_lua利用者, I want 1つのエントリーポイントで起動シーケンス全体を実行したい, so that シンプルなAPIでゴーストを起動できる

#### Acceptance Criteria
1. When PastaLoader::load(path)が呼び出される, the PastaLoader shall ディレクトリ探索→トランスパイル→ランタイム初期化を順次実行する
2. When 起動シーケンスが完了した, the PastaLoader shall 実行可能なPastaLuaRuntimeインスタンスを返す
3. The PastaLoader shall 起動進捗をtracingログとして出力する
4. If 起動シーケンス中にいずれかのステップで失敗した, the PastaLoader shall 失敗ステップと原因を明示したエラーを返す
5. Where 高度なカスタマイズが必要, the 利用者 shall 既存のLuaTranspiler・PastaLuaRuntime APIを直接使用できる

### Requirement 6: エラーハンドリングと診断

**Objective:** As a pasta_lua開発者, I want 起動失敗時の原因を素早く特定したい, so that デバッグ効率が向上する

#### Acceptance Criteria
1. When ファイル読み込みエラーが発生した, the PastaLoader shall ファイルパスとIO エラー詳細を含むエラーを返す
2. When パースエラーが発生した, the PastaLoader shall ファイル名・行番号・エラー内容を構造化エラーとして返す
3. When トランスパイルエラーが発生した, the LuaTranspiler shall ソースマッピング情報を含むエラーを返す
4. The LoaderError shall Display traitを実装し、人間可読なエラーメッセージを提供する
5. The LoaderError shall thiserrorによる型階層でエラー種別を分類する

### Requirement 7: 設定ファイルへのLuaアクセス

**Objective:** As a pasta_luaスクリプト作成者, I want Luaスクリプトから設定ファイルの内容を参照したい, so that ゴースト固有の設定値に基づいてスクリプトを動的に変更できる

#### Acceptance Criteria
1. When PastaLuaRuntimeが初期化される, the PastaLuaRuntime shall `@pasta_config` モジュールをLua VMに登録する
2. When Luaスクリプトが `require("@pasta_config")` を実行した, the `@pasta_config` module shall 設定ファイル（pasta.toml）の内容をLuaテーブルとして返す
3. While 設定ファイルが存在しない状態, the `@pasta_config` module shall 空のテーブル `{}` を返す
4. The `@pasta_config` module shall 設定ファイルのTOML構造をLuaテーブルに忠実にマッピングする（ネストされたテーブル、配列、文字列、数値、真偽値）
5. The `@pasta_config` module shall 読み取り専用で設定を提供し、Lua側からの変更を許可しない

