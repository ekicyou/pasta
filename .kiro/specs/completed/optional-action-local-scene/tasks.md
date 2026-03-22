# 実装計画

- [x] 1. Pest文法の量指定子を緩和する
  - `grammar.pest` の `local_start_scene_scope` と `local_scene_scope` の `local_scene_item+` を `local_scene_item*` に変更する（2箇所）
  - 変更後、`cargo check -p pasta_dsl` でコンパイルエラーがないことを確認する
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. 空ローカルシーンのE2Eインテグレーションテストを追加する
- [x] 2.1 名前付き空ローカルシーンのE2Eテストを追加する
  - 以下のPastaコードをパース→トランスパイル→実行するテストを `scene_test.rs` に追加する：`・分岐会話無し` の直後に `・分岐会話アリ` が続く構文（アクション行なし分岐と通常分岐の混在）
  - `transpile()` ヘルパーでパース・トランスパイルが成功し、生成されたLuaコードに空シーン関数が含まれることを assert する
  - `create_runtime_with_finalize()` でLuaランタイムを作成しトランスパイル出力を実行、`finalize_scene` までエラーなく完了することを確認する
  - `SEARCH:search_scene()` でシーンが検索可能なことを assert する（Req 3.1は空Lua関数が正常終了することで暗黙的に検証）
  - _Requirements: 1.1, 2.1, 2.2, 3.1, 4.1_
- [x] 2.2 空スタートスコープのE2Eテストを追加する
  - 以下のPastaコードをパース→トランスパイル→実行するテストを `scene_test.rs` に追加する：グローバルシーン直後にアクション行なしで `・分岐A` に分岐する構文（`local_start_scene_scope` が空のケース）
  - `transpile()` でパース・トランスパイルが成功することを assert する
  - `create_runtime_with_finalize()` でLuaランタイムを作成しエラーなく完了することを確認する
  - _Requirements: 1.3, 2.1, 3.1, 4.2_

- [x] 3. 全テストの通過を確認する
  - `cargo test --all` を実行し、新規テスト2件を含む全テストがパスすることを確認する
  - 既存スナップショットに意図しない変更がないことを確認する
  - _Requirements: 1.2, 4.3_
