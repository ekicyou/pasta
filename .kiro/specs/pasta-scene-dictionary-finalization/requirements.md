# Requirements Document

## Introduction

本仕様は、`pasta-lua-cache-transpiler`仕様で実装された`finalize_scene()`スタブを**本実装**に置き換えることを目的とする。

**アーキテクチャ変更**：
- **Before（現状）**: トランスパイル時にRust側でSceneRegistry/WordDefRegistry構築 → SearchContextに渡す
- **After（本仕様）**: トランスパイル時はLuaコード出力のみ → Lua実行時にLua側レジストリに登録 → `finalize_scene()`でLua側から収集 → Rust側SearchContext構築

`scene_dic.lua`から呼び出される`finalize_scene()`関数が実行されたタイミングで：
1. Lua側レジストリ（`pasta.scene`、`pasta.word`等）から登録済み情報を収集
2. 収集した情報からRust側の`SearchContext`（検索装置）を構築
3. `@pasta_search`モジュールを有効化

これにより、キャッシュ出力されたLuaコードから動的にシーン・単語辞書を構築する仕組みが完成する。

**スコープ境界**：
- **スコープ内**: Lua側レジストリからのシーン辞書・単語辞書収集とSearchContext構築
- **スコープ外**: アクター辞書（将来の拡張として認識。グローバルシーン名→アクター名のマッピングと、関数/単語辞書呼び出しコールバックを提供する仕組み。シーン辞書の亜流として後続仕様で実装）

## Requirements

### Requirement 1: Lua側シーン情報収集

**Objective:** ランタイムシステムとして、Lua側で登録されたシーン情報をRust側に伝達したい。これにより、検索装置の構築に必要なデータを取得できる。

**設計方針**: トランスパイル時のRust側収集を廃止し、キャッシュ出力されたLuaコード実行時にLua側レジストリに登録されたデータを収集する。

#### Acceptance Criteria

1. When `finalize_scene()`が呼び出された時, the PastaRuntime shall Lua側の`pasta.scene`レジストリから全シーン情報を収集する
2. The PastaRuntime shall 各シーンについて「グローバルシーン名」「ローカルシーン名」「シーン関数参照」を取得する
3. When シーンレジストリが空の場合, the PastaRuntime shall 警告ログを出力し、空のSearchContextを構築する
4. The PastaRuntime shall 収集したシーン情報を`SceneRegistry`形式に変換する
5. If シーン情報の収集中にエラーが発生した場合, the PastaRuntime shall エラー詳細を含む`LuaError`を返す
6. The PastaRuntime shall Lua側シーンレジストリのデータ構造を定義し、トランスパイラ出力コードがこのレジストリに登録するよう設計する

### Requirement 2: 単語辞書情報収集

**Objective:** ランタイムシステムとして、Lua側で登録された単語辞書情報を収集したい。これにより、単語の前方一致検索機能を有効化できる。

**設計方針**: トランスパイル時のRust側収集を廃止し、キャッシュ出力されたLuaコード実行時にLua側レジストリに登録されたデータを収集する。

#### Acceptance Criteria

1. When `finalize_scene()`が呼び出された時, the PastaRuntime shall Lua側の単語レジストリ（`pasta.word`等）から全単語定義を収集する
2. The PastaRuntime shall 各単語について「キー」「値リスト」「スコープ（グローバル/ローカル）」を取得する
3. The PastaRuntime shall 収集した単語情報を`WordDefRegistry`形式に変換する
4. When 単語定義が存在しない場合, the PastaRuntime shall 空の`WordDefRegistry`を使用してSearchContextを構築する
5. The PastaRuntime shall Lua側単語レジストリのデータ構造を定義し、トランスパイラ出力コードがこのレジストリに登録するよう設計する

### Requirement 3: SearchContext構築・登録

**Objective:** ランタイムシステムとして、収集した辞書情報から検索装置を構築したい。これにより、シーン/単語の前方一致検索が可能になる。

#### Acceptance Criteria

1. When シーン・単語情報の収集が完了した時, the PastaRuntime shall `SceneRegistry`から`SceneTable`を構築する
2. When シーン・単語情報の収集が完了した時, the PastaRuntime shall `WordDefRegistry`から`WordTable`を構築する
3. The PastaRuntime shall 構築した`SearchContext`を`@pasta_search`モジュールとして登録する
4. When `@pasta_search`モジュールが既に登録されている場合, the PastaRuntime shall 既存のモジュールを新しいSearchContextで置換する
5. The PastaRuntime shall SearchContext構築完了後、Luaスクリプトから`require "@pasta_search"`で検索機能にアクセス可能とする

### Requirement 4: Rust-Lua連携メカニズム

**Objective:** システム開発者として、LuaからRust関数を呼び出す仕組みを確立したい。これにより、`finalize_scene()`がRust側処理をトリガーできる。

#### Acceptance Criteria

1. The PastaRuntime shall Lua関数`PASTA.finalize_scene()`からRust関数を呼び出すためのバインディングを提供する
2. When `finalize_scene()`が呼び出された時, the PastaRuntime shall Rust側の`finalize_scene_impl`関数を実行する
3. The PastaRuntime shall `finalize_scene_impl`関数に現在のLuaコンテキストへのアクセスを提供する
4. If Rust側処理が失敗した場合, the PastaRuntime shall Luaエラーとして伝播させる
5. The PastaRuntime shall `finalize_scene()`の戻り値として成功/失敗を示すブール値をLuaに返す

### Requirement 5: 初期化タイミング制御

**Objective:** ランタイムシステムとして、検索装置の初期化タイミングを適切に制御したい。これにより、すべてのシーンが登録された後に検索装置が構築される。

#### Acceptance Criteria

1. The PastaRuntime shall `scene_dic.lua`のロード完了後に`finalize_scene()`が呼び出されることを前提とする
2. When `finalize_scene()`が複数回呼び出された場合, the PastaRuntime shall 毎回SearchContextを再構築する
3. While SearchContextが未構築の状態で検索が実行された場合, the PastaRuntime shall 「検索装置未初期化」エラーを返す
4. The PastaRuntime shall `finalize_scene()`呼び出し前に`@pasta_search`モジュールが存在しないことを許容する

### Requirement 6: エラーハンドリング

**Objective:** ゴースト開発者として、検索装置構築時のエラーを把握したい。これにより、問題の原因を特定し修正できる。

#### Acceptance Criteria

1. If SceneTable構築に失敗した場合, the PastaRuntime shall エラー原因（重複シーン名等）を含むエラーメッセージを報告する
2. If WordTable構築に失敗した場合, the PastaRuntime shall エラー原因を含むエラーメッセージを報告する
3. If Luaレジストリへのアクセスに失敗した場合, the PastaRuntime shall アクセス対象とエラー詳細を報告する
4. While debug_modeが有効な場合, the PastaRuntime shall 収集したシーン数・単語数をログ出力する
5. The PastaRuntime shall SearchContext構築成功時に情報レベルのログを出力する

### Requirement 7: 将来拡張への備え（アクター辞書）

**Objective:** システム設計者として、将来のアクター辞書実装に備えた拡張ポイントを用意したい。これにより、後続仕様での実装がスムーズになる。

#### Acceptance Criteria

1. The PastaRuntime shall `finalize_scene()`の実装において、追加の辞書タイプ（アクター辞書等）を受け入れる拡張ポイントを設計する
2. The SearchContext shall 将来的にアクター検索メソッドを追加できる構造を維持する
3. The PastaRuntime shall 辞書収集処理を個別関数に分離し、新しい辞書タイプの追加を容易にする

**Note**: アクター辞書の実装詳細は本仕様のスコープ外。アクター辞書は「グローバルシーン名→アクター名」のマッピングを提供し、関数または単語辞書の呼び出しコールバックを行う仕組み（シーン辞書の亜流）として後続仕様で実装予定。
