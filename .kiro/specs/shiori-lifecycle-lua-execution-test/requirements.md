# Requirements Document

## Introduction
現行の`test_full_shiori_lifecycle`テストは、Lua実行の実証可能性に欠ける。テストでは`main.lua`が固定の204応答を返しているだけで、SHIORI関数が実際に呼び出されたかどうか、またPasta DSLがパース・トランスパイルされたかどうかを検証できていない。本仕様は、Luaコード実行の証拠を確認可能なE2Eライフサイクルテストを定義する。

## Project Description (Input)
「test_full_shiori_lifecycle」テスト関数は、実際にluaコードを読み込んでいるのか？必要なテストとしては、実際にLoaderがpasta DSLを読み込み、main.luaを読み込み、luaのSHIORI.load / request / unloadを実行していることの確認だが、現状コードでは実際にluaコードを実行しているように見えない。実際にluaコードが実行されていることを確認できる、SHIORIローディングライフサイクルテストにしてほしい。

## Requirements

### Requirement 1: SHIORI.load実行確認
**Objective:** As a 開発者, I want SHIORI.loadが実際にLua側で実行されたことを検証可能にしたい, so that ランタイム初期化の正当性を担保できる

#### Acceptance Criteria
1. When `PastaShiori::load()`が呼び出される, the テストフィクスチャ shall SHIORI.load内でグローバル変数を設定する
2. When SHIORI.loadが実行される, the テスト shall Luaグローバル状態を介して呼び出し実績を検証する
3. When SHIORI.loadにhinst, load_dir引数が渡される, the テスト shall 渡された引数値がLua側で受け取られたことを確認する

### Requirement 2: SHIORI.request実行確認
**Objective:** As a 開発者, I want SHIORI.requestが実際にリクエスト内容を処理していることを検証可能にしたい, so that プロトコル処理の正当性を担保できる

#### Acceptance Criteria
1. When `PastaShiori::request()`が呼び出される, the テストフィクスチャ shall リクエスト内容をパースしレスポンスに反映する
2. When 特定のリクエストヘッダーが送信される, the テスト shall レスポンス内容から当該ヘッダーが処理されたことを確認する
3. When SHIORI.requestが複数回呼び出される, the テスト shall 呼び出しカウントがインクリメントされることを確認する

### Requirement 3: SHIORI.unload実行確認
**Objective:** As a 開発者, I want SHIORI.unloadが実際にLua側で実行されたことを検証可能にしたい, so that リソースクリーンアップの正当性を担保できる

#### Acceptance Criteria
1. When `PastaShiori`がドロップされる, the SHIORI.unload shall ファイルシステム上にマーカーファイルを作成する
2. When unloadマーカーファイルが作成される, the テスト shall ドロップ後にマーカーファイルの存在を確認する
3. If SHIORI.unloadが未定義の場合, the テスト shall エラーなくドロップが完了することを確認する

### Requirement 4: Pasta DSL読み込み確認
**Objective:** As a 開発者, I want Pasta DSLファイルが実際にパース・トランスパイルされていることを検証可能にしたい, so that スクリプトエンジン統合の正当性を担保できる

#### Acceptance Criteria
1. When PastaLoaderがload_dirを読み込む, the テスト shall `.pasta`ファイルがトランスパイルされLuaランタイムに登録されることを確認する
2. When トランスパイル済みシーンがLuaから呼び出される, the テスト shall シーン出力がSHIORI応答に含まれることを確認する
   - **具体例**: フィクスチャ`dic/test/lifecycle.pasta`に`＊テスト挨拶`シーンを定義し、`main.lua`から以下のように呼び出す:
     ```lua
     local SEARCH = require "@pasta_search"
     local global_name, local_name = SEARCH:search_scene("テスト挨拶", nil)
     if global_name then
         -- シーン関数呼び出し（例: テスト挨拶_1.__start__()）
         local scene_fn = _G[global_name][local_name]
         if scene_fn then
             local output = scene_fn()  -- シーン実行
             return "SHIORI/3.0 200 OK\r\nValue: " .. output .. "\r\n\r\n"
         end
     end
     ```
   - テストは応答に`Value: <シーン出力>`が含まれることを検証する
3. If `.pasta`ファイルに構文エラーがある場合, the テスト shall PastaLoaderが適切なエラーメッセージを返すことを確認する

### Requirement 5: テストフィクスチャ整備
**Objective:** As a 開発者, I want ライフサイクルテスト専用のフィクスチャを整備したい, so that 各フェーズの実行証拠を明確に収集できる

#### Acceptance Criteria
1. The テストフィクスチャ shall `scripts/pasta/shiori/main.lua`に観測可能な副作用を持つSHIORI関数を定義する
2. The テストフィクスチャ shall `dic/`配下にテスト用`.pasta`シーンを配置する
3. Where 副作用としてファイル書き込みを使用する場合, the フィクスチャ shall テスト終了時にクリーンアップ可能な設計とする
4. The テストフィクスチャ shall 固定値ではなく入力に依存した動的な応答を生成する
