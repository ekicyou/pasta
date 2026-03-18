# Implementation Plan

## Task List

- [ ] 1. (P) CONFIGアクターへのnameフィールド注入
  - `register_config_module` 内で `toml_to_lua` 変換後、`[actor]` 配下の各サブテーブルに `name = キー名` を後処理として注入する
  - `[actor]` セクションが存在しない・各エントリがテーブルでない場合はスキップする
  - 既存の `name` フィールドがあってもキー名で上書きする（TOMLキーが権威的）
  - _Requirements: 1.1, 1.4, 1.5_

- [ ] 2. `BUILDER.build` のインターフェース簡素化
- [ ] 2.1 (P) 直接変更方式への移行
  - `input_actor_spots` のシャローコピー生成を廃止し、入力テーブルを直接変更する
  - 返却値をスクリプト文字列のみにし、第2返却値 `actor_spots` を削除する
  - `input_actor_spots` が `nil` のとき内部で空テーブルを生成する（外部影響なし）
  - _Requirements: 1.1_
- [ ] 2.2 フォールバック warn ログ追加
  - `actor_spots[actor_name]` が `nil` のとき `@pasta_log` の `warn` でアクター名とデフォルトスポット値（0）をログ出力する
  - _Requirements: 2.1_

- [ ] 3. (P) `SHIORI_ACT_IMPL.build` の書き戻し削除
  - `BUILDER.build` の返却値をスクリプト文字列のみで受け取り、`updated_spots` 変数を削除する
  - `STORE.actor_spots` への書き戻し処理（`if updated_spots then STORE.actor_spots = updated_spots end`）を削除する
  - `STORE.actor_spots` を直接 `BUILDER.build` の第3引数として渡す（直接変更により自動反映）
  - _Requirements: 1.1_

- [ ] 4. `sakura_builder_test.lua` の大規模更新
- [ ] 4.1 (P) 第2返却値依存テストの書き換え
  - `updated_spots` を参照しているテストをすべて `input_spots` への直接参照に変更する（6テスト: 返却値型チェック・スポット値検証）
  - nil 入力テストは result の内容（`\p[0]` 含有）のみ検証するよう変更する
  - _Requirements: 1.1_
- [ ] 4.2 純粋関数性テスト削除と直接変更確認テスト追加
  - 「入力テーブルがclear_spotで変更されないことを確認」テストを削除する（直接変更方式で動作逆転）
  - `clear_spot` 後に `input_spots` のエントリがクリアされることを確認する新規テストを追加する
  - `％` 行ありシーンが従来通り `clear_spot()` + `set_spot()` で動作することをリグレッションテストで確認する
  - `％` 行の省略がパースエラーや警告を生成しないことを確認する
  - _Requirements: 1.1, 1.3, 2.1, 2.2_

- [ ] 5. (P) `config_actors_initialization_test.rs` の更新
  - CONFIG由来アクターに `name` フィールドが正しく注入されていることを検証するテストを追加する
  - `toml_to_lua` 変換後に `name` フィールドが各スポット定義に存在することをアサートする
  - `STORE.actors` 経由でも `name` が参照できることを確認する（参照共有の確認）
  - 既存の「nameフィールドがないため」コメント付きテストを修正し、nameが存在する状態での動作検証に更新する
  - _Requirements: 1.1, 1.2, 1.5_

- [ ] 6. `pasta_sample_ghost` コピー同期
- [ ] 6.1 (P) `sakura_builder.lua` コピー更新
  - `pasta_sample_ghost` 配下の `sakura_builder.lua` を `pasta_lua/scripts/` と同一内容に更新する（直接変更・スクリプトのみ返却・warn ログ）
  - _Requirements: 1.1, 2.1_
- [ ] 6.2 (P) `act.lua` コピー更新
  - `pasta_sample_ghost` 配下の `act.lua` を `pasta_lua/scripts/` と同一内容に更新する（書き戻し削除）
  - _Requirements: 1.1_

- [ ] 7. ドキュメント整合性の確認と更新
  - [ ] SOUL.md - コアバリュー・設計原則との整合性確認
  - [ ] doc/spec/ - 言語仕様の更新（該当する場合）
  - [ ] GRAMMAR.md - 文法リファレンスの同期（該当する場合）
  - [ ] TEST_COVERAGE.md - 新規テストのマッピング追加
  - [ ] クレートREADME - API変更の反映（該当する場合）
  - [ ] steering/* - 該当領域のステアリング更新
