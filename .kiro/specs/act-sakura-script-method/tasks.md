# Implementation Plan

## Task Overview

3層（トランスパイラ / actランタイム / sakura_builder）への修正タスクはすべて別ファイル・別層を対象とするため相互並行実行可能。テスト追加タスク群は実装完了後に並行実行可能。

---

- [ ] 1. (P) トランスパイラでアクター紐付きコード生成に修正する
  - `Action::SakuraScript` のコード生成出力を `act.{actor}:sakura_script(literal)` 形式に変更する
  - `actor` パラメータは `generate_action()` の引数として既に利用可能なため、format 文字列の修正のみで完結する
  - 他の6つの Action 型の出力形式に影響を与えないことを確認する
  - _Requirements: 1.1, 1.2_

- [ ] 2. actランタイムに `sakura_script` メソッドを追加する
- [ ] 2.1 (P) プロキシレイヤーに `sakura_script` メソッドを追加する
  - アクタープロキシに `talk()` と同構造の `sakura_script(self, text)` メソッドを追加する
  - `self.act:sakura_script(self.actor, text)` を呼び出す1行実装とする
  - `PROXY_IMPL.talk()` の直後に配置する
  - _Requirements: 2.1_

- [ ] 2.2 (P) ACT_IMPL に `sakura_script` メソッドを追加し、グループ化ロジックを拡張する
  - `ACT_IMPL.talk()` と同構造の `sakura_script(self, actor, text)` メソッドを追加し、`{ type = "sakura_script", actor = actor, text = text }` トークンを蓄積する
  - `group_by_actor()` の `talk` 判定条件に `sakura_script` を加え、アクター変更検出に参加させる
  - トークンの `actor` フィールドから変更検出する既存ロジックを再利用する
  - _Requirements: 2.2, 2.3, 2.4_

- [ ] 3. (P) sakura_builder でさくらスクリプトトークンを処理できるようにする
  - `BUILDER.build()` 内部のトークン処理ループに `sakura_script` 用の分岐を追加する
  - `talk` トークンと同じく `talk_to_script(actor, inner.text)` を呼び出して出力バッファに追加する
  - `talk` 分岐の直後に `elseif` 形式で追加し、`raw_script` 分岐は変更しない
  - さくらスクリプトタグ（`\n`, `\w9` 等）はトークナイザーがパススルーするため、追加処理は不要
  - _Requirements: 3.1, 3.2_

- [ ] 4. テストを追加して全件パスを確認する
- [ ] 4.1 (P) スナップショットテストでトランスパイル出力を検証する
  - `snapshot_test.rs` にインライン文字列形式でテスト関数を追加する（既存7/8件と同じパターン）
  - さくらスクリプトタグを含むアクション行の Pasta DSL をトランスパイルした出力が `act.{actor}:sakura_script()` 形式になることを `assert_snapshot!` で確認する
  - _Requirements: 1.1, 4.1, 4.2_

- [ ] 4.2 (P) `act_grouping_test.lua` に `sakura_script` グループ化テストを追加する
  - `sakura_script` 単体トークンがアクターグループを開始することをテストする
  - `talk` + `sakura_script` 混合配列が正しくグループ化されることをテストする
  - `sakura_script` によるアクター切り替えが検出されることをテストする
  - `merge_consecutive_talks()` において `sakura_script` が `talk` とは統合されず分離トークンとして扱われることをテストする
  - _Requirements: 2.3, 4.2_

- [ ] 4.3 (P) `sakura_builder_test.lua` に `sakura_script` ビルドテストを追加する
  - `{ type="sakura_script", actor=..., text="\\n" }` トークンを含むグループを `BUILDER.build()` に渡し、さくらスクリプトタグがそのまま出力に含まれることをアサートする
  - `raw_script` トークンの既存動作が変化していないことを既存テストで確認する
  - _Requirements: 3.1, 3.2, 4.2_

- [ ] 4.4 全テストパスを確認する
  - `cargo test -p pasta_lua` を実行してすべてのテストがパスすることを確認する
  - スナップショットが更新された場合は `cargo insta review` で承認する
  - _Requirements: 4.2, 4.3_
