# Requirements Document

## Introduction

本仕様は、pasta Luaランタイムにおけるモジュールパス解決機構をLua標準の`require()`ルールに準拠させ、ユーザースクリプトによる拡張性と柔軟性を向上させるものです。

主な目的：
- `main.lua`等の読み込みをLua標準`require()`に統一
- ユーザー作成スクリプト用検索パスの追加によるモジュール上書き機能
- ローダー初期化順序の変更によるユーザー辞書登録の実現

## Project Description (Input)

### luaモジュールのパス解決方法をluaルールに準拠させる
main.luaの読み込みを、luaの検索パス設定の後、`require("main")`相当の処理で読み込むようにする。また、本番サンプルゴーストの検索パスに「ユーザー作成スクリプト」を追加する。中身のない`scripts/main.lua`を配置する。

これにより、ユーザーが`main.lua`を作成しなかったとき、空の`scripts/main.lua`が読み込まれる。ユーザーが自作スクリプトを「ユーザー作成スクリプト」に配置でき、pastaランタイムのモジュール挙動を上書きしたいときは同名モジュールを配置することで上書きを実施できる。

さらに、ローダーのlua初期化挙動を変更し、辞書登録のファイナライズ処理より、mainの解決を先に行うことでユーザースクリプトによる辞書登録を可能とする。

### デフォルトのrequre検索パスの追加
検索パスに「ユーザー作成スクリプト」を追加する。「pasta標準ランタイム」より先に解決することでユーザーによる上書きを許可し、ユーザーが「pasta標準ランタイム」を書き換える必要をなくす。

```toml
lua_search_paths = [
    "profile/pasta/save/lua",   # ユーザー保存スクリプト
    "user_scripts",              # ユーザー作成スクリプト
    "scripts",                   # pasta 標準ランタイム
    "profile/pasta/cache/lua",   # トランスパイル済みキャッシュ
    "scriptlibs",                # 追加ライブラリ
]
```

### main/pasta.scene_dic/pasta.shiori.entry/などのluaファイル読み込み方法
main.luaなど、luaコードの読み込みは、ファイルをrustで直接読み込むのではなく、`require("main")`相当のmlua関数またはコード実行でおこなう。luaの検索パスにより解決させる。同様の処理を行っている部分について、「&Luaとモジュール名を渡せばrequireしてくれる関数」を作るとよい。

### ローダーのlua初期化挙動変更
scene_dicによるファイナライズより先にmainを解決する。これにより、ユーザースクリプトによる`辞書.lua`の事前登録を可能とする。

1. （lua検索パスの解決やrust側モジュールのluaへの公開など）
2. pasta DSLのトランスパイル
3. **`require("main")`**
4. **`require("pasta.scene_dic")`**
5. （以降同じ）

手順3,4は1回のスクリプト実行で行ってもよいし、手順3,4を行う`pasta.ファイナライズ？`luaモジュールを用意する形でもよい。

### pasta_sample_ghost サンプルゴーストの修正
デフォルト検索パスの追加などの変更対応を、pasta_sample_ghost サンプルゴーストに反映させること。setup.batの実行で反映したゴーストが作成できること。

---

## Requirements

### Requirement 1: Lua検索パスへのユーザースクリプトディレクトリ追加

**Objective:** ゴースト開発者として、自作スクリプトを専用ディレクトリに配置してpastaランタイムの挙動を上書きしたい。これにより、標準ランタイムを直接編集せずにカスタマイズが可能になる。

#### Acceptance Criteria

1. When pasta.tomlが読み込まれたとき、the PastaLoader shall `user_scripts`ディレクトリをLua検索パスに含める。

2. The default_lua_search_paths shall 以下の優先順位で検索パスを設定する:
   - `profile/pasta/save/lua` (ユーザー保存スクリプト)
   - `user_scripts` (ユーザー作成スクリプト)
   - `scripts` (pasta標準ランタイム)
   - `profile/pasta/cache/lua` (トランスパイル済みキャッシュ)
   - `scriptlibs` (追加ライブラリ)

3. When ユーザーが`user_scripts/`に同名モジュールを配置したとき, the Lua require shall 標準ランタイムより先にユーザースクリプトを解決する。

4. While `user_scripts`ディレクトリが存在しないとき, the PastaLoader shall エラーを発生させず正常に動作を継続する。

---

### Requirement 2: require()ベースのモジュール読み込み統一

**Objective:** pasta_luaランタイム開発者として、モジュール読み込みをLua標準の`require()`に統一したい。これにより、Luaの検索パス解決ルールに準拠し、一貫性のあるモジュール解決が実現できる。

**設計原則**: Lua検索パス優先順位による上書き可能領域を極限まで広げる。例外は設けない。ユーザーが上書きした場合の挙動はユーザー責任とする。

#### Acceptance Criteria

1. When main.luaを読み込むとき, the PastaRuntime shall `require("main")`でモジュールを解決する。

2. When scene_dic.luaを読み込むとき, the PastaRuntime shall `require("pasta.scene_dic")`でモジュールを解決する。

3. When entry.luaを読み込むとき, the PastaRuntime shall `require("pasta.shiori.entry")`でモジュールを解決する。

4. The PastaRuntime shall `&Lua`とモジュール名を受け取り`require`を実行するヘルパー関数を提供する。

5. If require対象のモジュールが見つからないとき, the PastaRuntime shall 適切なエラーメッセージを返す。

6. The require helper function shall Luaの`package.path`設定に従ってモジュールを検索する。

---

### Requirement 3: ローダー初期化順序の変更

**Objective:** ゴースト開発者として、main.luaで辞書登録処理を行いたい。これにより、DSLファイナライズ前にユーザー定義の辞書設定が可能になる。

#### Acceptance Criteria

1. When PastaRuntimeが初期化されるとき, the loader shall 以下の順序で処理を実行する:
   1. Lua検索パスの設定とRust側モジュールのLua公開
   2. pasta DSLのトランスパイル
   3. `require("main")`の実行
   4. `require("pasta.scene_dic")`の実行（ファイナライズ含む）

2. When `require("main")`が実行されるとき, the PastaRuntime shall scene_dicファイナライズより先にmainモジュールを解決する。

3. While main.luaが存在しないとき, the PastaRuntime shall 空の`scripts/main.lua`をフォールバックとして使用する。

4. The main.lua execution shall 辞書登録APIを利用可能な状態で実行される。

---

### Requirement 4: デフォルトmain.luaの提供

**Objective:** ゴースト開発者として、main.luaを作成しなくても動作するデフォルト挙動がほしい。これにより、シンプルなゴーストは追加設定なしで動作する。

#### Acceptance Criteria

1. The pasta_lua crate shall `scripts/main.lua`に空のデフォルトmain.luaを提供する。

2. When ユーザーがmain.luaを作成していないとき, the require("main") shall 標準ランタイムの空main.luaを読み込む。

3. When ユーザーが`user_scripts/main.lua`を作成したとき, the require("main") shall ユーザーのmain.luaを優先的に読み込む。

4. The default main.lua shall コメントでユーザー向けの使用例を含む。

---

### Requirement 5: pasta_sample_ghostへの変更反映

**Objective:** サンプルゴースト利用者として、新しい検索パス設定が反映されたゴーストを作成したい。これにより、最新のpastaランタイム機能を即座に利用できる。

#### Acceptance Criteria

1. When pasta.toml.templateが更新されるとき, the template shall `user_scripts`を含む新しい検索パス設定を持つ。

2. When setup.batが実行されるとき, the build process shall 新しい検索パス設定を含むゴーストを生成する。

3. The generated ghost shall `user_scripts/`ディレクトリを含む（空でも可）。

4. The integration tests shall 新しい検索パス設定の正当性を検証する。

---

### Requirement 6: 後方互換性の維持

**Objective:** 既存ゴースト開発者として、既存のゴーストが引き続き動作することを保証したい。これにより、移行の手間なく新機能を利用できる。

#### Acceptance Criteria

1. While 既存のpasta.tomlに`user_scripts`が設定されていないとき, the PastaLoader shall デフォルト設定で`user_scripts`を検索パスに含める。

2. The existing scripts directory structure shall 変更なく動作を継続する。

3. The API changes shall 破壊的変更を最小限に抑える。
