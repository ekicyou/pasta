# 実装計画

- [x] 1. 基盤: コンパイラ警告調査とデッドコード特定
- [x] 1.1 pasta_coreの現状警告・デッドコード全数調査
  - `cargo clippy -p pasta_core -- -W warnings` を実行し、全警告を記録する
  - 未使用関数・メソッド・型・インポートを一覧化する
  - `#[allow(...)]` アトリビュートの有無を確認する
  - 完了条件: 修正対象の警告・デッドコード一覧が確定し、修正方針が決定している
  - _Requirements: 3_

- [x] 2. コア: エラー型の改善
- [x] 2.1 (P) エラーメッセージ言語の統一
  - `error.rs` の `WordTableError::WordNotFound` メッセージを英語に変更: `"Word not found: @{key}"`
  - `SceneTableError` の全バリアントの使用状況を確認する
  - 未使用バリアントが存在する場合は除去する
  - 完了条件: 全エラーメッセージが英語で統一され、未使用バリアントが除去されている
  - _Requirements: 5.1, 5.2, 5.3_
  - _Boundary: error.rs_

- [x] 2.2 (P) エラーメッセージ変更に伴うテスト更新
  - `crates/pasta_core/tests/word_table_test.rs` のエラーメッセージ文字列比較を更新する
  - 下流クレートへの影響は `cargo test --workspace` で検出し、修正が必要な場合は audit-workspace-patterns（Wave 2）に委譲する
  - 完了条件: `cargo test -p pasta_core` が全パスする
  - _Requirements: 5.1, 6.2_
  - _Depends: 2.1_
  - _Boundary: crates/pasta_core/tests/_

- [x] 3. コア: SceneTable のパニック安全化とリファクタリング
- [x] 3.1 キャッシュ処理の共通ヘルパー抽出
  - `resolve_scene_id` と `resolve_scene_id_unified` の Phase 3-5（キャッシュ取得/作成、リセット、逐次選択）を共通ヘルパーメソッド `select_from_cache` に抽出する
  - ヘルパーは `SceneCacheKey` と `Vec<SceneId>`（フィルタ済み候補）を受け取り `Result<SceneId, SceneTableError>` を返す
  - 両メソッドは Phase 1（候補収集）のみを担当し、Phase 3-5 はヘルパーに委譲する
  - 完了条件: `resolve_scene_id` と `resolve_scene_id_unified` の重複コードが排除され、既存テストが全パスする
  - _Requirements: 4.1, 6.1, 6.2_
  - _Boundary: scene_table.rs_

- [x] 3.2 SceneTable の unwrap() 排除
  - `cache.get_mut(&cache_key).unwrap()` を安全なパターン（`ok_or` + エラーバリアント、または `if let`）に置換する
  - `candidates[next_index]` を `.get(next_index).copied().ok_or(...)` に置換する
  - `fn_name_to_search_key` 内の `unwrap_or` パターンの安全性を確認する（既に安全であれば変更不要）
  - 完了条件: `scene_table.rs` の非テストコードに `unwrap()` / `expect()` / `panic!` が存在しない
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Boundary: scene_table.rs_
  - _Depends: 3.1_

- [x] 4. コア: WordTable のパニック安全化
- [x] 4.1 (P) WordTable のインデックスアクセス安全化
  - `search_word` 内の `shuffled_words[0].clone()` を `.first().cloned().ok_or(...)` に置換する
  - `cached.words[cached.next_index].clone()` を `.get(cached.next_index).cloned().ok_or(...)` に置換する
  - 冗長なイテレータチェーンがあれば簡素化する
  - 完了条件: `word_table.rs` の非テストコードに直接インデックスアクセス `[i]` が存在しない（または安全性がコメントで証明済み）
  - _Requirements: 1.1, 1.2, 1.4, 4.3_
  - _Boundary: word_table.rs_

- [x] 5. コア: デッドコード除去と冗長表現削減
- [x] 5.1 (P) 全ファイルのデッドコード除去
  - タスク1.1で特定したデッドコードを除去する
  - 未使用の `use` 文を除去する
  - `random.rs` の `DefaultRandomSelector::select` / `shuffle` ジェネリックメソッドの使用状況を確認し、未使用であれば除去する
  - `mod.rs` / `lib.rs` の未使用 re-export を確認する
  - 完了条件: `cargo clippy -p pasta_core -- -W warnings` で dead_code / unused 系の警告が0件
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Boundary: 全pasta_coreファイル_

- [x] 5.2 (P) 冗長表現の簡素化
  - `scene_registry.rs` / `word_registry.rs` の冗長パターンを確認し、簡素化可能であれば修正する
  - 不要な中間コレクション（`collect()` → 即座にイテレーション等）があれば排除する
  - 不要なクローン・アロケーションを参照・借用で代替する
  - 完了条件: 冗長パターンが排除され、既存テストが全パスする
  - _Requirements: 4.2, 4.3, 4.4, 7.2, 7.3_
  - _Boundary: scene_registry.rs, word_registry.rs_

- [x] 6. 検証: 全体回帰テストとコンパイラ検証
- [x] 6.1 全テスト回帰確認
  - `cargo test -p pasta_core` が全パスする
  - `cargo test` がワークスペース全体で全パスする（950+ テスト）
  - 完了条件: テスト全パス、失敗0件
  - _Requirements: 6.2, 6.3_

- [x] 6.2 コンパイラ警告・clippy 最終確認
  - `cargo clippy -p pasta_core -- -W warnings` で新規警告0件
  - `cargo clippy -- -W warnings` でワークスペース全体の新規警告0件
  - 公開APIシグネチャが変更されていないことを差分レビューで確認する
  - 完了条件: 警告0件、公開API不変が確認済み
  - _Requirements: 3.4, 6.1, 6.4, 7.1_
