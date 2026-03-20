# Requirements Document

## Project Description (Input)
現在、pasta_luaおよびhello-pastaのluaスクリプトの配置フォルダが下記となっている。
```toml
[loader]
# pasta DSL ファイルパターン
pasta_patterns = ["dic/*.pasta"]
# Lua モジュール検索パス（優先順位順）
lua_search_paths = [
    "profile/pasta/save/lua",   # ユーザー保存スクリプト
    "user_scripts",              # ユーザー作成スクリプト（カスタマイズ用）
    "scripts",                   # pasta 標準ランタイム
    "profile/pasta/cache/lua",   # トランスパイル済みキャッシュ
    "scriptlibs",                # 追加ライブラリ
]
# トランスパイル出力先
transpiled_output_dir = "profile/pasta/cache/lua"
```
このフォルダ構成だと、pasta標準ランタイムが分かりにくく、
ユーザーがふいに上書きしてしまうように感じるため、下記としたい。

```toml
# Lua モジュール検索パス（優先順位順）
lua_search_paths = [
    "profile/pasta/save/lua",    # ユーザー保存スクリプト
    "scripts",                   # ユーザー作成スクリプト（カスタマイズ用）
    "pasta_scripts",             # pasta 標準ランタイム
    "profile/pasta/cache/lua",   # トランスパイル済みキャッシュ
    "scriptlibs",                # 追加ライブラリ
]
```

この修正は、hello_pastaだけの宣言の修正だけとはせず、
pasta_luaの現在の開発ディレクトリに存在する
`scripts`フォルダーを`pasta_scripts`に変更し、影響するユニットテストも
更新するなど抜本的な修正とすること。
hallo_pastaのビルドやリリース手順なども含めて、網羅修正がひつようなので
留意せよ。

## Introduction

Luaスクリプト検索パスを再構成し、pasta標準ランタイムスクリプトとユーザー作成スクリプトの役割を明確に分離する。現行の `scripts`（標準ランタイム）/ `user_scripts`（ユーザー用）構成を `pasta_scripts`（標準ランタイム）/ `scripts`（ユーザー用）に変更することで、ユーザーが標準ランタイムを誤って上書きするリスクを排除する。

## Requirements

### Requirement 1: Lua検索パスの再定義

**Objective:** ゴースト開発者として、pasta標準ランタイムとユーザースクリプトの配置先が直感的に区別できるようにし、標準ランタイムの誤上書きを防止したい。

#### Acceptance Criteria

1. The pasta_lua shall デフォルトのLua検索パス順序を以下のとおり定義する: `profile/pasta/save/lua` → `scripts` → `pasta_scripts` → `profile/pasta/cache/lua` → `scriptlibs`
2. The pasta_lua shall `user_scripts` をLua検索パスに含めない
3. When pasta.tomlに `lua_search_paths` が明示指定されている場合, the pasta_lua shall 設定ファイルの値をデフォルト値より優先する

### Requirement 2: 標準ランタイムスクリプトのディレクトリ移動

**Objective:** パッケージメンテナーとして、pasta標準ランタイムスクリプトが `pasta_scripts` ディレクトリに格納されるようにし、ユーザー作成スクリプト（`scripts`）と物理的に分離したい。

#### Acceptance Criteria

1. The pasta_lua crate shall 開発ディレクトリ内の標準ランタイムスクリプトを `scripts/` ではなく `pasta_scripts/` に格納する
2. The pasta_lua shall `pasta_scripts/` 配下の全Luaファイルのコードロジックを変更なく保持する（ファイル名の同一性。案内コメント内のパス参照は新構成に合わせて更新する）
3. When ビルド実行時, the pasta_lua shall `pasta_scripts/` ディレクトリのスクリプトを正しく発見・ロードする
4. The `pasta_scripts/` ディレクトリに README.md を配置し、ゴースト開発者がこのフォルダーを編集すべきでないことを明記する
5. The `scripts/` ディレクトリに README.md を配置し、ユーザーカスタムスクリプト用フォルダーであること・`pasta_scripts/` より優先される旨を明記する
6. The `hello.lua`（サンプルスクリプト）を削除し、それを参照するテスト（`transpiler_test.lua`）および VSCode デバッグ設定（`launch.json` の該当エントリ）も削除する（ランタイム不要、テスト用途としても残す価値なし）

### Requirement 3: hello-pasta サンプルゴーストの設定更新

**Objective:** ゴースト開発者として、hello-pastaリファレンス実装が新しいパス構成を正しく反映し、動作の手本となるようにしたい。

#### Acceptance Criteria

1. The hello-pasta の pasta.toml shall `lua_search_paths` に新パス構成（`scripts`, `pasta_scripts`）を使用する
2. The hello-pasta shall ビルド・リリース手順（release.ps1）が新しいディレクトリ構成でゴーストパッケージを正しく生成する
3. The hello-pasta shall 生成される配布物（.nar）に `pasta_scripts/` 配下のランタイムスクリプトが正しく含まれる
4. The hello-pasta shall 生成される配布物に旧 `user_scripts/` ディレクトリへの参照を含まない

### Requirement 4: ユニットテスト・統合テストの整合性

**Objective:** 開発者として、全テストが新しいディレクトリ構成で引き続きパスし、リグレッションがないことを保証したい。

#### Acceptance Criteria

1. When `cargo test --all` を実行した場合, the テストスイート shall 全テストがパスする
2. The pasta_lua テスト shall テストフィクスチャ内のスクリプトパス参照が新しいディレクトリ構成（`pasta_scripts`）を使用する
3. The pasta_lua テスト shall テスト用一時ディレクトリの構成が新しいパス構成を反映する
4. If テスト内でデフォルト検索パスを検証している場合, the テスト shall 新しいデフォルト値（`scripts`, `pasta_scripts`）を使用する

### Requirement 5: ドキュメント・ステアリングの整合性

**Objective:** 開発者・AI支援ツールとして、プロジェクトドキュメントが新しいディレクトリ構成を正確に反映し、一貫した情報を提供したい。

#### Acceptance Criteria

1. The steering/structure.md shall 新しいディレクトリ構成を反映する
2. The pasta_lua README shall Lua検索パスの記述を新しいデフォルト値に更新する
3. The pasta_sample_ghost README/RELEASE.md shall 新しいディレクトリ構成に対応した手順を記載する
4. If Luaスクリプト配置パスに言及しているドキュメントがある場合, the ドキュメント shall `scripts`（ユーザー用）・`pasta_scripts`（標準ランタイム）の役割の違いを明記する
