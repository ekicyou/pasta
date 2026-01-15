# Requirements Document

## Introduction

PastaShioriのload関数に最低限の実装を行い、pasta_luaエンジンをローダー経由で初期化し、ランタイム準備を完了させる。この実装により、SHIORI DLLとしてのPastaスクリプトエンジンが「伺か」ベースシェルから呼び出し可能な状態になる。

## Project Description (Input)

PastaShioriのload関数の最低限の実装を行う。pasta_luaのエンジンをローダー経由でロードして、ランタイムの準備ができるところまでをとりあえず実施。hinstにDLLのモジュールID、load_dirに設定ファイルのディレクトリが渡されます。

## Requirements

### Requirement 1: PastaLuaRuntimeインスタンス管理

**Objective:** SHIORI DLL開発者として、load関数呼び出し時にPastaLuaRuntimeインスタンスを生成・保持したい。これにより、後続のrequest呼び出しでスクリプトを実行できる状態を確保する。

#### Acceptance Criteria

1. When load関数が呼び出された場合, PastaShiori shall PastaLoader::load()を使用してランタイムを初期化する
2. When ランタイム初期化が成功した場合, PastaShiori shall PastaLuaRuntimeインスタンスを内部フィールドに保持する
3. When load関数が複数回呼び出された場合, PastaShiori shall 既存のランタイムを破棄して新しいランタイムを生成する

### Requirement 2: load_dirパス処理

**Objective:** SHIORI DLL開発者として、load_dirパラメータを正しく処理したい。これにより、ゴーストのmaster/ディレクトリからスクリプトをロードできる。

#### Acceptance Criteria

1. When load_dirがOsStr形式で渡された場合, PastaShiori shall PathBufに変換して保存する
2. When load_dirがPastaLoader::loadに渡される場合, PastaShiori shall pasta.tomlとdic/ディレクトリを含むベースディレクトリとして使用する
3. The PastaShiori shall load_dirの存在確認を行い、存在しない場合はエラーを返す

### Requirement 3: エラーハンドリング

**Objective:** SHIORI DLL開発者として、ロード時のエラーを適切に処理したい。これにより、問題発生時にデバッグ可能な情報を提供できる。

#### Acceptance Criteria

1. If PastaLoader::loadがLoaderErrorを返した場合, PastaShiori shall エラー情報をログ出力してfalseを返す
2. If load_dirが存在しない場合, PastaShiori shall DirectoryNotFoundエラーとしてfalseを返す
3. If pasta.toml設定ファイルが見つからない場合, PastaShiori shall ConfigNotFoundエラーとして処理する
4. The PastaShiori shall tracing crateを使用してエラー詳細をログ出力する

### Requirement 4: hinstパラメータ保持

**Objective:** SHIORI DLL開発者として、DLLモジュールハンドル(hinst)を保持したい。これにより、将来的なWindows API統合に備える。

#### Acceptance Criteria

1. When load関数が呼び出された場合, PastaShiori shall hinstパラメータを内部フィールドに保存する
2. The PastaShiori shall hinstをisize型として保持する（Windows HINSTANCE互換）

### Requirement 5: ランタイム状態管理

**Objective:** SHIORI DLL開発者として、ランタイムの初期化状態を追跡したい。これにより、request呼び出し時にランタイムが利用可能か判断できる。

#### Acceptance Criteria

1. While ランタイムが初期化されていない状態で, PastaShiori shall request呼び出しに対してエラーを返す
2. When load関数が成功した場合, PastaShiori shall ランタイム参照をOption<PastaLuaRuntime>フィールドに格納する
3. The PastaShiori shall Drop trait実装でランタイムを適切に解放する
