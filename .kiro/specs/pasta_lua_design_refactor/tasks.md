# Implementation Plan

## Task Overview

本仕様の実装は2つの目標から構成される：

1. **子仕様の作成**: トランスパイラー、Lua実装、検索モジュールの各仕様を作成
2. **Luaスケルトンコード**: トランスパイラー出力がエラーにならないAPI定義を作成

## Tasks

### Major Task 1: 子仕様の準備と計画

- [ ] 1.1 子仕様の構造設計
  - 3つの子仕様（トランスパイラー、Lua実装、検索モジュール）の依存関係を整理
  - 各子仕様が参照すべき設計ドキュメントの範囲を特定
  - 実装順序（トランスパイラー → Lua実装 → 検索モジュール）の根拠を明確化
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

### Major Task 2: トランスパイラー子仕様の作成

- [ ] 2.1 トランスパイラー仕様の要件定義
  - code_generator.rs の現状出力形式と設計ドキュメントの差異を分析
  - シーン関数シグネチャ変更（ctx→act）の影響範囲を特定
  - init_scene() 呼び出しパターン、save/var参照取得の出力要件を定義
  - アクタープロキシ経由のメソッド呼び出しパターンを定義
  - _Requirements: 8.1, 8.4, 8.5, 8.6, 8.9, 10.1_

- [ ] 2.2 トランスパイラー仕様の設計
  - PASTA.create_scene() 呼び出し生成ロジックを設計
  - シーン関数冒頭テンプレートを設計（init_scene、clear_spot、set_spot）
  - act.アクター:talk()、act.アクター:word() 呼び出し生成を設計
  - act:call() によるシーン遷移コード生成を設計
  - _Requirements: 5.7, 5.8, 8.1, 8.7, 8.8_

- [ ] 2.3 トランスパイラー仕様ドキュメントの作成
  - `.kiro/specs/pasta_lua_transpiler/` ディレクトリを作成
  - requirements.md: 親仕様からの要件継承と具体化
  - design.md: code_generator.rs 修正内容の詳細設計
  - spec.json: メタデータ設定
  - _Requirements: 10.1, 10.4_

### Major Task 3: Lua実装子仕様の作成

- [ ] 3.1 (P) Lua実装仕様の要件定義
  - 5つのコアモジュール（init, ctx, act, actor, scene）の実装要件を定義
  - 各モジュールの公開API一覧を作成
  - メタテーブル設計（__index、__newindex）の詳細要件を定義
  - トークン蓄積・出力メカニズムの実装要件を定義
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 3.1, 3.4, 3.5, 4.1, 4.2, 5.1, 5.2, 10.2_

- [ ] 3.2 (P) Lua実装仕様の設計
  - モジュール間依存関係の詳細設計
  - CTX.new()、ACT.new()、ACTOR.get_or_create() の実装設計
  - co_action コルーチン制御フローの詳細設計
  - アクタープロキシ生成・逆参照メカニズムの設計
  - _Requirements: 2.4, 3.1, 3.2, 3.16, 4.3, 4.7_

- [ ] 3.3 Lua実装仕様ドキュメントの作成
  - `.kiro/specs/pasta_lua_implementation/` ディレクトリを作成
  - requirements.md: 親仕様からの要件継承と実装観点での詳細化
  - design.md: 各モジュールの内部実装設計
  - spec.json: メタデータ設定
  - _Requirements: 10.2, 10.4_

### Major Task 4: 検索モジュール子仕様の作成

- [ ] 4.1 (P) 検索モジュール仕様の要件定義
  - シーン辞書検索（前方一致、グローバル名・ローカル名ペア返却）の要件を定義
  - 単語辞書検索（4レベル優先順位）の要件を定義
  - Lua側からのRust関数呼び出しインターフェース要件を定義
  - _Requirements: 4.6, 4.8, 5.5, 8.7, 10.3_

- [ ] 4.2 (P) 検索モジュール仕様の設計
  - Rust側 mlua バインディング設計
  - シーン検索API（search_scene）の設計
  - 単語検索API（search_word）の設計（アクター名、グローバルシーン名パラメータ）
  - _Requirements: 4.6, 4.8, 5.5, 10.3_

- [ ] 4.3 検索モジュール仕様ドキュメントの作成
  - `.kiro/specs/pasta_search_module/` ディレクトリを作成
  - requirements.md: 検索機能の詳細要件
  - design.md: Rust側実装設計
  - spec.json: メタデータ設定
  - _Requirements: 10.3, 10.4_

### Major Task 5: Luaスケルトンコード作成（既存コード調査）

- [ ] 5.1 既存Luaコードの分析
  - 現在の `pasta/init.lua`, `pasta/ctx.lua`, `pasta/act.lua`, `pasta/actor.lua` の構造を分析
  - 現在のメタテーブルパターンを抽出
  - 設計ドキュメントとの差異を特定
  - 再利用可能なコードパターンを識別
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

### Major Task 6: Luaスケルトンコード作成（init.lua）

- [ ] 6.1 pasta/init.lua スケルトン作成
  - PASTA公開APIテーブルの定義
  - create_actor() 関数シグネチャとスタブ実装
  - create_scene() 関数シグネチャとスタブ実装
  - 依存モジュールのrequireパターン
  - _Requirements: 1.2, 1.3, 8.2, 8.3, 11.1, 11.6, 11.7_

### Major Task 7: Luaスケルトンコード作成（ctx.lua）

- [ ] 7.1 pasta/ctx.lua スケルトン作成
  - CTXクラスのメタテーブル定義
  - CTX.new() コンストラクタとスタブ実装
  - save, actors フィールド定義
  - co_action(), start_action(), yield(), end_action() メソッドシグネチャ
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 11.5, 11.6, 11.7_

### Major Task 8: Luaスケルトンコード作成（act.lua）

- [ ] 8.1 pasta/act.lua スケルトン作成
  - ACTクラスのメタテーブル定義
  - __indexメタメソッドによるアクタープロキシ動的生成
  - ACT.new() コンストラクタとスタブ実装
  - ctx, var, token, now_actor, current_scene フィールド定義
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 11.2, 11.6, 11.7_

- [ ] 8.2 pasta/act.lua APIメソッド定義
  - init_scene(), talk(), sakura_script() メソッドシグネチャ
  - word() メソッドシグネチャ（3レベル検索）
  - yield(), end_action() メソッドシグネチャ
  - call(), set_spot(), clear_spot() メソッドシグネチャ
  - _Requirements: 3.6, 3.7, 3.8, 3.9, 3.10, 3.11, 3.12, 3.13, 11.2, 11.6, 11.7_

### Major Task 9: Luaスケルトンコード作成（actor.lua）

- [ ] 9.1 pasta/actor.lua スケルトン作成
  - ACTORクラスのメタテーブル定義
  - actor_cache キャッシュ機構
  - ACTOR.get_or_create() 関数シグネチャ
  - name フィールド定義
  - _Requirements: 4.1, 4.3, 11.4, 11.6, 11.7_

- [ ] 9.2 pasta/actor.lua プロキシ定義
  - PROXYクラスのメタテーブル定義
  - ACTOR.create_proxy() 関数シグネチャ
  - actor, act フィールド定義（逆参照）
  - PROXY:talk(), PROXY:word() メソッドシグネチャ
  - _Requirements: 4.5, 4.6, 4.7, 4.8, 11.4, 11.6, 11.7_

### Major Task 10: Luaスケルトンコード作成（scene.lua）

- [ ] 10.1 pasta/scene.lua スケルトン作成
  - SCENEレジストリのメタテーブル定義
  - registry テーブル（階層構造）
  - SCENE.register() 関数シグネチャ
  - SCENE.get(), SCENE.get_global_table(), SCENE.get_global_name() 関数シグネチャ
  - SCENE.get_start() 関数シグネチャ
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 11.3, 11.6, 11.7_

### Major Task 11: Luaスケルトンコード作成（拡張モジュール）

- [ ] 11.1 (P) pasta/areka/init.lua スケルトン作成
  - AREKAモジュールのスタブ定義
  - 将来の拡張ポイントコメント
  - _Requirements: 9.1, 9.3, 9.4_

- [ ] 11.2 (P) pasta/shiori/init.lua スケルトン作成
  - SHIORIモジュールのスタブ定義
  - 将来の拡張ポイントコメント
  - _Requirements: 9.2, 9.3, 9.4_

### Major Task 12: スケルトン統合検証

- [ ] 12.1 モジュール依存関係の検証
  - require パスの確認
  - 循環参照がないことを確認
  - トランスパイラー出力パターンとの整合性確認
  - _Requirements: 1.5, 11.6_

- [ ] 12.2 トランスパイラー出力テスト
  - 簡単なPastaスクリプトをトランスパイル
  - 生成されたLuaコードがスケルトンAPIを呼び出せることを確認
  - Luaビルドエラーがないことを確認
  - _Requirements: 8.1, 11.6, 11.7_
