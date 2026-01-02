# 実装タスク

## タスク概要
- **合計**: 4メジャータスク、10サブタスク
- **全要件カバレッジ**: 要件1.1-1.2, 2.1-2.3, 3.1-3.5, 4.1-4.2, 5.1-5.5
- **平均タスクサイズ**: 1-3時間/サブタスク

---

## タスク一覧

### 1. AST型定義の実装

- [ ] 1.1 (P) SceneActorItem構造体を定義する
  - `ast.rs`にアクター項目を表す型を追加
  - アクター名（String）、番号（u32）、ソース位置（Span）の3フィールドを持つ
  - `Debug`, `Clone`, `PartialEq`, `Eq`トレイトを実装
  - ドキュメンテーションコメントで文法との対応を明記
  - _Requirements: 1.1, 1.2_

- [ ] 1.2 (P) GlobalScopeScopeにactorsフィールドを追加する
  - `GlobalSceneScope`構造体に`actors: Vec<SceneActorItem>`フィールドを追加（wordsフィールドの直後）
  - `new()`と`continuation()`コンストラクタで`actors: Vec::new()`初期化
  - 既存フィールド順を維持（name, is_continuation, attrs, words, actors, code_blocks, local_scenes, span）
  - _Requirements: 2.1_

### 2. パーサー変換ロジックの実装

- [ ] 2.1 parse_actors_item関数を実装する
  - 1件の`actors_item`ルールをパースして`SceneActorItem`を生成
  - `id`ルールからアクター名を抽出
  - `digit_id`があれば`normalize_number_str`で全角→半角変換後パース
  - C#のenum採番ルール適用（番号指定時: その値を使用＆next_number更新、番号なし: next_number使用＆インクリメント）
  - `Span::from()`でソース位置を記録
  - _Requirements: 3.2, 3.3, 3.4, 3.5_

- [ ] 2.2 parse_scene_actors_line関数を実装する
  - `scene_actors_line`ルールをパースして`Vec<SceneActorItem>`を返す
  - silent rule `actors`の内部イテレータで`actors_item`を順次処理
  - `parse_actors_item`を呼び出して各項目を変換
  - `next_number`状態を引数で受け取り、複数項目で引き継ぐ
  - pad、actor_marker、or_comment_eolはスキップ
  - _Requirements: 3.1_

- [ ] 2.3 parse_global_scene_scopeを拡張する
  - `Rule::scene_actors_line`の分岐を追加
  - `next_actor_number: u32 = 0`で採番状態を初期化
  - `parse_scene_actors_line`を呼び出して結果を`actors.extend()`で蓄積
  - 複数行のアクター宣言で番号を引き継ぐ（next_actor_numberが複数行で継続）
  - GlobalScopeScope構築時にactorsフィールドを設定
  - _Requirements: 2.2, 2.3_

### 3. pasta_lua互換性対応

- [ ] 3.1 (P) pasta_luaのテストコードを修正する
  - `context.rs`の`create_test_scene()`に`actors: Vec::new()`追加
  - `transpiler.rs`の`create_simple_scene()`に`actors: vec![]`追加
  - `transpiler.rs`の`create_scene_with_words()`に`actors: vec![]`追加
  - `transpiler.rs`の`create_scene_with_local()`に`actors: vec![]`追加
  - 全4箇所のGlobalScopeScope構築でactorsフィールドを初期化
  - _Requirements: 4.1_

- [ ] 3.2 (P) pasta_luaビルド検証を実施する
  - `cargo build -p pasta_lua`でコンパイルエラーがないことを確認
  - トランスパイラ・コード生成でactorsフィールドが無視されることを確認（既存動作維持）
  - _Requirements: 4.2_

### 4. テスト実装と検証

- [ ] 4.1 単一行アクターパースのテストを実装する
  - `％さくら`（番号0）を正しくパース
  - `％さくら、うにゅう＝２`（さくら=0, うにゅう=2）を正しくパース
  - 全角数字（`＝１０`）が半角に正規化されてパース
  - _Requirements: 5.1, 5.2_

- [ ] 4.2 C#のenum採番ルールのテストを実装する
  - `％さくら、うにゅう＝２、まりか`（さくら=0, うにゅう=2, まりか=3）を検証
  - 番号なし→指定→なしのパターンで連番が正しく継続
  - _Requirements: 5.3_

- [ ] 4.3 複数行アクター宣言のテストを実装する
  - グローバルシーン内の複数`scene_actors_line`で番号が引き継がれることを検証
  - 例: 1行目`％さくら`、2行目`％うにゅう＝５`、3行目`％まりか`で、さくら=0, うにゅう=5, まりか=6
  - actors Vecに全アクターが順序通り蓄積されることを確認
  - _Requirements: 5.4_

- [ ] 4.4 全体テストを実行して品質を確保する
  - `cargo test --all`で既存テスト＋新規テストが全て成功
  - リグレッションがないことを確認
  - _Requirements: 5.5_

---

## 実装順序ガイド

1. **AST定義** (タスク1): 型の定義のみ、他への依存なし → 並列実行可能
2. **パーサー実装** (タスク2): 2.1→2.2→2.3の順で実装（依存関係あり）
3. **pasta_lua対応** (タスク3): AST定義後ならいつでも可能 → 並列実行可能
4. **テスト** (タスク4): パーサー実装完了後に実施

**並列実行推奨**:
- タスク1.1, 1.2, 3.1は完全に独立 → 同時実行可能
- タスク2はシーケンシャル実行必須
- タスク3.2はタスク3.1完了後
- タスク4は全実装完了後
