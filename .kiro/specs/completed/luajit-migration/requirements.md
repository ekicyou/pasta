# Requirements Document

## Introduction
pasta_luaクレートのLuaランタイムをLua 5.5（mlua 0.11 `lua55` feature + `lua-src` vendored）からLuaJIT 2.1（mlua 0.11 `luajit52` feature）に移行する。パフォーマンス向上（JITコンパイル）、FFI機能、ネイティブUTF-8マルチバイト識別子サポートを得ることが目的。vendored build方式を維持し、既存の全テストスイートの互換性を保証する。

## Boundary Context
- **In scope**: mlua feature切り替え（`lua55`→`luajit52`）、`lua-src`依存除去、テスト互換性確認、ステアリングドキュメント更新
- **Out of scope**: LuaJIT FFIを活用した新機能開発、LuaJIT固有パフォーマンスチューニング、Luaスクリプトの機能追加・変更
- **Adjacent expectations**: mlua 0.11が`luajit52` + `vendored`をサポートしていること、mlua-stdlib 0.1がLuaJITバックエンドで動作すること

## Requirements

### Requirement 1: Luaランタイムの切り替え
**Objective:** As a ゴースト開発者, I want pasta_luaのLuaランタイムがLuaJIT 2.1で動作すること, so that JITコンパイルによるパフォーマンス向上とFFI機能を利用できる

#### Acceptance Criteria
1. When pasta_luaクレートをビルドした時, the pasta_lua shall mlua 0.11の`luajit52` featureを使用してLuaJIT 2.1ランタイムを組み込む
2. When pasta_luaクレートをビルドした時, the pasta_lua shall `vendored` featureにより外部LuaJITインストール不要でビルドが完了する
3. The pasta_lua shall `serialize` featureを引き続き有効にし、シリアライゼーション機能を維持する

### Requirement 2: lua-src依存の除去
**Objective:** As a プロジェクトメンテナ, I want 不要になった`lua-src`依存が除去されること, so that 依存関係が簡潔になりビルド時間が短縮される

#### Acceptance Criteria
1. When ワークスペースCargo.tomlを参照した時, the ワークスペース設定 shall `lua-src`のワークスペース依存宣言を含まない
2. When pasta_lua/Cargo.tomlを参照した時, the pasta_lua shall `[build-dependencies]`セクションに`lua-src`を含まない

### Requirement 3: UTF-8マルチバイト識別子の互換性
**Objective:** As a ゴースト開発者, I want LuaスクリプトでUTF-8マルチバイト識別子（日本語変数名など）が引き続き使用できること, so that 既存のゴースト辞書との互換性が維持される

#### Acceptance Criteria
1. When UTF-8マルチバイト文字を含む識別子をLuaスクリプトで使用した時, the LuaJIT 2.1ランタイム shall エラーなくスクリプトを実行する
2. When 既存のucid_testを実行した時, the テストスイート shall 全テストケースがパスする

### Requirement 4: 既存テストスイートの互換性
**Objective:** As a プロジェクトメンテナ, I want LuaJIT移行後も既存の全テストが通ること, so that リグレッションがないことが保証される

#### Acceptance Criteria
1. When `cargo test`を実行した時, the ワークスペース全体 shall 全テストケースがパスする
2. When pasta_luaクレートのテストを実行した時, the pasta_lua shall トランスパイラテスト、SHIORIテスト、検索テスト、ローダーテストを含む全テストがパスする
3. When Luaスクリプトベースのテスト（lua_specs/）を実行した時, the pasta_lua shall コルーチン、永続化、アクター/シーンディスパッチを含む全テストがパスする

### Requirement 5: 下流クレートの互換性
**Objective:** As a プロジェクトメンテナ, I want pasta_luaに依存する全クレートが正常にビルド・動作すること, so that ランタイム変更による影響が波及しない

#### Acceptance Criteria
1. When pasta_shioriクレートをビルドした時, the pasta_shiori shall コンパイルエラーなくビルドが完了する
2. When pasta_checkクレートをビルドした時, the pasta_check shall コンパイルエラーなくビルドが完了する
3. When pasta_sample_ghostクレートをビルドした時, the pasta_sample_ghost shall コンパイルエラーなくビルドが完了する

### Requirement 6: mlua-stdlib互換性
**Objective:** As a ゴースト開発者, I want mlua-stdlibが提供する拡張ライブラリ（json, regex, yaml）がLuaJITバックエンドで動作すること, so that 既存のスクリプトで使用している機能が維持される

#### Acceptance Criteria
1. When LuaスクリプトからJSON操作を行った時, the mlua-stdlib shall json機能が正常に動作する
2. When Luaスクリプトから正規表現操作を行った時, the mlua-stdlib shall regex機能が正常に動作する
3. When LuaスクリプトからYAML操作を行った時, the mlua-stdlib shall yaml機能が正常に動作する

### Requirement 7: ステアリングドキュメントの更新
**Objective:** As a プロジェクトメンテナ, I want 技術ステアリングドキュメントがLuaJIT 2.1への移行を反映していること, so that プロジェクトの技術構成が正確に記録される

#### Acceptance Criteria
1. When tech.mdを参照した時, the ステアリングドキュメント shall Luaランタイムとして「LuaJIT 2.1（mlua 0.11）」を記載する
2. When tech.mdを参照した時, the ステアリングドキュメント shall `lua-src`への言及を除去または更新する
