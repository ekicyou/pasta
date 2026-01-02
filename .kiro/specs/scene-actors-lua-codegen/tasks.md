# 実装タスク: scene-actors-lua-codegen

## タスク概要
- **総数**: 3 主要タスク、7 サブタスク
- **要件カバレッジ**: 4 要件（1, 2, 3, 4）すべてを網羅
- **平均タスク規模**: 各サブタスク 1-3 時間
- **進行方向**: コンポーネント修正 → テスト追加 → 全体検証

---

## 実装計画

### 1. generate_local_scene 署名変更と set_spot 生成実装

- [ ] 1.1 (P) `generate_local_scene` 関数署名に `actors` パラメータを追加
  - `crates/pasta_lua/src/code_generator.rs` の `generate_local_scene` 関数（line 223）に `actors: &[SceneActorItem]` パラメータを追加
  - 既存の `counter: usize` パラメータの直後に配置
  - Rust コンパイラエラーが出ないことを確認（`cargo check -p pasta_lua`）
  - _Requirements: 1.1, 1.4, 2.1, 2.3_

- [ ] 1.2 (P) `generate_local_scene` 内に set_spot ブロック生成ロジックを実装
  - `counter == 0 && !actors.is_empty()` 条件で `set_spot` ブロックを生成
  - `PASTA.create_session` 行の直後（既存 line 241-242 後）に配置
  - 前後に空行を 1 行ずつ挿入（`self.write_blank_line()?` で実装）
  - 各アクターに対して `act.<name>:set_spot(<number>)` 形式で出力
  - アクター順序（`actors` 配列の順序）を保持
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2_

### 2. generate_global_scene 呼び出し部分の修正

- [ ] 2.1 (P) `generate_global_scene` から `generate_local_scene` 呼び出しを修正
  - `code_generator.rs` line 184 の呼び出しを修正
  - `self.generate_local_scene(local_scene, counter)?` から `self.generate_local_scene(local_scene, counter, &scene.actors)?` に変更
  - 変更後 `cargo check -p pasta_lua` で Rust コンパイルエラーが出ないことを確認
  - _Requirements: 2.3_

### 3. テスト追加と全体検証

- [ ] 3.1 複数アクター生成テスト
  - `crates/pasta_lua/tests/transpiler_integration_test.rs` に `test_set_spot_multiple_actors` 関数を追加
  - 入力: `％さくら、うにゅう＝２` のような複数アクター宣言を含む Pasta ファイル
  - 検証: 生成 Lua コード に `act.さくら:set_spot(0)` と `act.うにゅう:set_spot(2)` が含まれることを確認
  - 検証: `act.さくら` が `act.うにゅう` より先に出力されることで順序保持を確認
  - _Requirements: 3.1, 3.3_

- [ ] 3.2 単一アクター生成テスト
  - `test_set_spot_single_actor` 関数を追加
  - 入力: `％さくら` のような単一アクター宣言
  - 検証: 生成 Lua コード に `act.さくら:set_spot(0)` が含まれることを確認
  - _Requirements: 3.1, 3.3_

- [ ] 3.3 アクター未定義テスト
  - `test_set_spot_empty_actors` 関数を追加
  - 入力: アクター宣言なしの Pasta ファイル
  - 検証: 生成 Lua コード に `set_spot` 呼び出しが一切含まれないことを確認
  - _Requirements: 3.2, 3.3_

- [ ] 3.4 明示的番号付きアクターテスト
  - `test_set_spot_with_explicit_number` 関数を追加
  - 入力: `％さくら、うにゅう＝２、まりか` のように明示的な番号指定を含む宣言
  - 検証: `act.うにゅう:set_spot(2)` と `act.まりか:set_spot(3)` が正しく生成されることを確認（C# enum 採番ルール）
  - _Requirements: 1.2, 3.1, 3.3_

- [ ] 3.5 全テスト実行と検証
  - `cargo test --all` を実行
  - 既存テストすべてが引き続きパスすることを確認（リグレッションなし）
  - 新規追加テスト（3.1 ～ 3.4）がすべてパスすることを確認
  - テスト実行ログを確認してエラーや警告がないことを確認
  - _Requirements: 3.3_

---

## 実装チェックリスト

### コンパイル検証
- [ ] Task 1.1 実行後: `cargo check -p pasta_lua` パス
- [ ] Task 1.2 実行後: `cargo check -p pasta_lua` パス
- [ ] Task 2.1 実行後: `cargo check -p pasta_lua` パス

### テスト検証
- [ ] Task 3.1 ～ 3.4: 各テスト関数が `transpiler_integration_test.rs` に正しく追加される
- [ ] Task 3.5: `cargo test --all` 完全パス（既存テスト＋新規テスト）

### 機能検証
- [ ] 出力 Lua コード が要件 4 の「生成コード例」と一致
- [ ] 複数行アクター宣言が 1 つの `__start__` で集約初期化される
- [ ] アクター未定義時に `set_spot` が生成されない

---

## 要件マッピング確認

| 要件 | 対応タスク | 検証方法 |
|------|----------|---------|
| 1.1 | 1.2, 3.1-3.4 | `set_spot` コード生成、複数テスト |
| 1.2 | 1.2, 3.4 | 形式検証、明示的番号テスト |
| 1.3 | 1.2, 3.3 | 空配列時テスト |
| 1.4 | 1.2, 3.1 | 順序保持テスト |
| 2.1 | 1.2, 3.1-3.3 | `__start__` 条件分岐実装 |
| 2.2 | 1.2, 3.1-3.3 | 配置位置テスト（create_session 直後） |
| 2.3 | 2.1, 3.1-3.3 | generate_global_scene 修正 |
| 3.1 | 3.1, 3.2, 3.4 | 複数・単一・明示番号テスト |
| 3.2 | 3.3 | アクター未定義テスト |
| 3.3 | 3.5 | `cargo test --all` パス |
| 4 | 1.2, 3.1-3.3 | コード例との一致確認 |

**全要件カバレッジ**: ✅ 完全（未対応要件なし）
