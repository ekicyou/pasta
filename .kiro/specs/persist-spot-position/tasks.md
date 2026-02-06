# Implementation Tasks: persist-spot-position

## タスク一覧

- [ ] 1. STORE.actor_spotsフィールドの追加と初期化
- [ ] 1.1 STORE.actor_spotsフィールド宣言
  - `STORE.actor_spots`テーブルを`{}`で初期化
  - アクター名をキー、スポットID（整数）を値とする構造
  - _Requirements: 2.1_

- [ ] 1.2 CONFIG.actorからのspot値転送
  - CONFIG.actor の各エントリについて、actor.spot が存在する場合に `STORE.actor_spots[name] = actor.spot` で転送
  - `type(actor.spot) == "number"` で数値型を検証（非数値はスキップ）
  - 既存のCONFIG.actor転送ブロック（pcall保護）の直後に配置
  - _Requirements: 1.1, 1.3, 2.1_

- [ ] 1.3 STORE.reset()へのactor_spotsクリア追加
  - `STORE.reset()` 内で `STORE.actor_spots = {}` を実行
  - 既存のリセット処理と同じパターンで追加
  - _Requirements: 2.1_

- [ ] 2. sakura_builder.build()のシグネチャ拡張
- [ ] 2.1 build()関数シグネチャの変更
  - 第3引数として `actor_spots` （`table<string, integer>|nil`）を追加
  - 第2戻り値として `table<string, integer>` （更新後のactor_spots）を追加
  - 現行シグネチャ: `build(grouped_tokens, config) → string`
  - 変更後: `build(grouped_tokens, config, actor_spots) → string, table`
  - _Requirements: 2.2_

- [ ] 2.2 actor_spotsのシャローコピー処理
  - 入力 `actor_spots` が nil の場合は空テーブル `{}` として扱う
  - 明示的ループコピーで新しいテーブルを作成（`for name, spot in pairs(input_actor_spots) do local_actor_spots[name] = spot end`）
  - 入力テーブルを変更しない（純粋関数性の保証）
  - _Requirements: 2.2_

- [ ] 2.3 clear_spot/spotトークン処理の更新
  - `clear_spot` トークン処理時: 個別nilクリア（`for name in pairs(local_actor_spots) do local_actor_spots[name] = nil end`）でテーブル再割り当てを回避
  - `spot` トークン処理時: `local_actor_spots[actor_name] = spot_id` で更新
  - 既存の `spot_to_id()` 関数を使用してスポットID正規化
  - _Requirements: 2.2, 2.3_

- [ ] 2.4 更新後のactor_spotsを第2戻り値で返却
  - `return script, local_actor_spots` の形式で返却
  - 第1戻り値（さくらスクリプト文字列）は変更なし
  - _Requirements: 2.2, 2.3_

- [ ] 3. SHIORI_ACT_IMPL.build()でのSTORE連携
- [ ] 3.1 STORE requireの追加
  - `local STORE = require("pasta.store")` をモジュール先頭のrequireセクションに追加
  - 既存の pasta.act, pasta.shiori.sakura_builder, pasta.config と同様のパターン
  - _Requirements: 2.3, 2.4_

- [ ] 3.2 STORE.actor_spotsの読み取りとビルダー呼び出し
  - `ACT.IMPL.build(self)` でトークン取得
  - `STORE.actor_spots` を読み取り
  - `BUILDER.build(grouped_tokens, self.config, STORE.actor_spots)` を呼び出し
  - 戻り値 `(script, updated_spots)` を受け取り
  - _Requirements: 2.3, 2.4_

- [ ] 3.3 STORE.actor_spotsへの書き戻し
  - `BUILDER.build()` の第2戻り値が nil でない場合のみ `STORE.actor_spots = updated_spots` で書き戻し
  - トークン0件時（nil返却）は STORE 更新をスキップ
  - `script` を返却（戻り値の型は変更なし）
  - _Requirements: 2.3, 2.4_

- [ ] 4. サンプルゴーストの設定追加
- [ ] 4.1 (P) pasta.tomlへのactor設定追加
  - `[persistence]` セクションの前に `[actor."女の子"]` と `[actor."男の子"]` セクションを追加
  - 女の子: `spot = 0`、男の子: `spot = 1` を設定
  - 論理的グルーピング: ghost → talk → actor → persistence の順序
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 4.2 (P) pasta.toml.templateへのactor設定追加
  - pasta.toml と同じ `[actor]` セクションをテンプレートに追加
  - テンプレート変数は使用せず固定値で設定
  - _Requirements: 3.1, 3.2_

- [ ] 5. 単体テスト
- [ ] 5.1 (P) store.lua: actor_spots初期化テスト
  - CONFIG.actor からの spot 値転送が正しく行われることを確認
  - 非数値の spot 値がスキップされることを確認
  - CONFIG.actor 未定義時にエラーなく動作することを確認
  - _Requirements: 1.1, 1.2, 1.3, 2.1_

- [ ] 5.2 (P) store.lua: reset()動作テスト
  - `STORE.reset()` が actor_spots を空テーブルにクリアすることを確認
  - _Requirements: 2.1_

- [ ] 5.3 (P) sakura_builder.lua: build()純粋関数性テスト
  - 入力 actor_spots テーブルが変更されないことを確認
  - nil 入力時に空テーブルとして扱われることを確認
  - _Requirements: 2.2_

- [ ] 5.4 (P) sakura_builder.lua: clear_spotトークン処理テスト
  - clear_spot トークンで actor_spots がリセットされることを確認
  - テーブル再割り当てではなく個別nilクリアが行われることを確認
  - _Requirements: 2.2, 2.3_

- [ ] 5.5 (P) sakura_builder.lua: spotトークン処理テスト
  - spot トークンで actor_spots が正しく更新されることを確認
  - spot_to_id() による正規化が機能することを確認
  - _Requirements: 2.2, 2.3_

- [ ] 6. 統合テスト
- [ ] 6.1 SHIORI_ACT_IMPL.build(): STORE読み書きフロー
  - STORE.actor_spots の読み取り → BUILDER.build() 呼び出し → STORE 書き戻しのフロー全体を確認
  - トークン0件時（nil返却）で STORE 更新がスキップされることを確認
  - _Requirements: 2.3, 2.4_

- [ ] 6.2 シーン連続実行: スポット値の引き継ぎ
  - ％行ありシーン → ％行なしシーンでスポット値が引き継がれることを確認
  - ％行ありシーンで spot が正しく更新されることを確認
  - _Requirements: 2.2, 2.3, 2.4_

- [ ] 6.3 CONFIG未設定時のデフォルト動作
  - CONFIG.actor が未定義の場合に actor_spots が空テーブルで動作することを確認
  - デフォルト spot=0 で動作することを確認
  - _Requirements: 1.2_

- [ ] 7. E2Eテスト
- [ ] 7.1 サンプルゴースト起動: actor設定の反映
  - サンプルゴースト起動時に `[actor]` セクション設定が STORE.actor_spots に反映されることを確認
  - 女の子 spot=0、男の子 spot=1 の初期化を確認
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 7.2* ランダムトーク: スポット切り替え正常動作
  - ％行付きシーンでのスポット切り替えが正常に動作することを確認
  - 生成されるさくらスクリプトの `\p[ID]` が正しいことを確認
  - _Requirements: 2.3_

- [ ] 7.3* ％行省略シーン連続実行: スポット継続保持
  - ％行ありシーン → ％行なしシーン（複数）でスポット位置が引き継がれることを確認
  - サンプルゴーストに検証用シーン追加（最初のシーンで `% 女の子 男の子`、以降2シーン以上でアクター行省略）
  - 生成されるさくらスクリプトの `\p[ID]` が正しく継続することを確認
  - _Requirements: 2.1, 2.2, 2.4_

## 要件カバレッジ確認

- 要件 1.1: タスク 1.2, 5.1
- 要件 1.2: タスク 5.1, 6.3
- 要件 1.3: タスク 1.2, 5.1
- 要件 2.1: タスク 1.1, 1.2, 1.3, 5.1, 5.2, 7.3*
- 要件 2.2: タスク 2.1, 2.2, 2.3, 2.4, 5.3, 5.4, 5.5, 6.2, 7.3*
- 要件 2.3: タスク 2.3, 2.4, 3.1, 3.2, 3.3, 5.4, 5.5, 6.1, 6.2, 7.2*
- 要件 2.4: タスク 3.1, 3.2, 3.3, 6.1, 6.2, 7.3*
- 要件 2.5: 実装済み（code_generator.rs）
- 要件 3.1: タスク 4.1, 4.2, 7.1
- 要件 3.2: タスク 4.1, 4.2, 7.1
- 要件 3.3: タスク 4.1, 7.1

全12要件中11要件をカバー（要件2.5は実装済み）。

## 並列実行可能タスク

以下のタスクは `(P)` マークで並列実行可能:
- タスク 4.1, 4.2: サンプルゴースト設定（他のコード変更と独立）
- タスク 5.1-5.5: 単体テスト（各モジュール独立）

その他のタスクは以下の理由で順次実行:
- タスク 1.x: STORE 初期化は後続の全タスクの前提
- タスク 2.x: ビルダー変更は SHIORI_ACT 変更の前提
- タスク 3.x: SHIORI_ACT 変更は統合テストの前提
- タスク 6.x, 7.x: 統合/E2Eテストは実装完了が前提
