# Implementation Plan

## Task Breakdown

### Lua Script Layer

- [ ] 1. pasta.scene モジュールにカウンタ管理機能を拡張
- [ ] 1.1 (P) ベース名ごとのカウンタ管理機能を実装
  - グローバルシーン名のベース名に対して連番カウンタを保持
  - `get_or_increment_counter(base_name)` 関数で自動的にカウンタをインクリメント
  - 同じベース名のシーンが複数回作成された場合、一意な名前を保証
  - _Requirements: 8.1, 8.2, 8.4, 8.6_

- [ ] 1.2 (P) 全シーン情報取得APIを実装
  - `get_all_scenes()` 関数で `{global_name: {local_name: func}}` 形式のデータを返却
  - レジストリに登録されている全シーンを走査
  - グローバルシーン名とローカルシーン名のペアを収集
  - _Requirements: 1.1, 1.2, 1.6_

- [ ] 2. pasta.word モジュールを新規作成
- [ ] 2.1 (P) グローバル/ローカル単語レジストリ管理機能を実装
  - グローバル単語レジストリ（key → values[][]）の管理
  - ローカル単語レジストリ（scene_name → {key → values[][]}）の管理
  - 同じキーに対する複数回の登録でマージ処理を実行
  - _Requirements: 2.3, 2.4, 2.5_

- [ ] 2.2 (P) ビルダーパターンAPIを実装
  - `create_global(key)` でグローバル単語ビルダーを返却
  - `create_local(scene_name, key)` でローカル単語ビルダーを返却
  - `entry(...)` メソッドで可変長引数を受け取り、値リストとして登録
  - メソッドチェーン用に `entry()` は自身を返す
  - _Requirements: 9.1, 9.2, 9.3, 9.5_

- [ ] 2.3 (P) 全単語情報取得APIを実装
  - `get_all_words()` 関数で `{global: {key: [values]}, local: {scene: {key: [values]}}}` 形式のデータを返却
  - グローバル単語とローカル単語の両方を収集
  - _Requirements: 2.6_

- [ ] 3. pasta.init モジュールを修正
- [ ] 3.1 (P) create_scene() をカウンタ管理に対応させる
  - `PASTA.create_scene(base_name, local_name, scene_func)` でカウンタを使用
  - `pasta.scene.get_or_increment_counter(base_name)` を呼び出して番号を取得
  - `base_name .. counter` 形式でグローバルシーン名を生成
  - シーンレジストリに登録
  - _Requirements: 8.2, 8.5_

- [ ] 3.2 (P) create_word() API を追加
  - `PASTA.create_word(key)` でグローバル単語ビルダーを返却
  - `pasta.word.create_global(key)` を呼び出し
  - _Requirements: 9.1, 9.4_

### Transpiler Layer

- [ ] 4. LuaCodeGenerator にシーン/単語定義出力を実装
- [ ] 4.1 グローバルシーン生成コードをカウンタレス形式に修正
  - `generate_global_scene()` 関数を修正（lines 162-221）
  - 行180-182の `module_name` を `scene.name` (ベース名) に変更
  - `PASTA.create_scene("{base_name}")` 形式で出力（番号付与はLua実行時）
  - _Requirements: 8.5_

- [ ] 4.2 (P) ファイルレベル単語定義の出力を実装
  - `FileItem::GlobalWord(KeyWords)` を走査
  - `PASTA.create_word("{key}"):entry("{value1}", "{value2}", ...)` 形式で出力
  - do block 外（グローバルスコープ）に出力
  - _Requirements: 2.1_

- [ ] 4.3 (P) シーンレベル単語定義の出力を実装
  - `GlobalSceneScope::words: Vec<KeyWords>` を走査
  - `SCENE:create_word("{key}"):entry("{value1}", "{value2}", ...)` 形式で出力
  - ローカルシーン関数内に出力
  - _Requirements: 2.2_

### Runtime Layer

- [ ] 5. runtime/finalize.rs モジュールを新規作成
- [ ] 5.1 Lua側シーン情報収集機能を実装
  - `collect_scenes(lua: &Lua)` 関数で `pasta.scene.get_all_scenes()` を呼び出し
  - Luaテーブルを走査して `Vec<(global_name, local_name)>` 形式に変換
  - シーンレジストリが空の場合は警告ログを出力
  - テーブルアクセスエラー時は詳細なエラーメッセージを含む `LuaError` を返す
  - _Requirements: 1.1, 1.2, 1.3, 1.5_

- [ ] 5.2 Lua側単語情報収集機能を実装
  - `collect_words(lua: &Lua)` 関数で `pasta.word.get_all_words()` を呼び出し
  - Luaテーブルを走査して `Vec<WordCollectionEntry>` 形式に変換
  - グローバル単語とローカル単語の両方を処理
  - 単語定義が存在しない場合は空のベクタを返す
  - _Requirements: 2.6, 2.8_

- [ ] 5.3 SceneRegistry/WordDefRegistry 構築機能を実装
  - `collect_scenes()` の結果から `SceneRegistry` を構築
  - `collect_words()` の結果から `WordDefRegistry` を構築
  - 同じキーの複数エントリは別 `WordEntry` として登録
  - 重複シーン名などの構築エラー時は原因を含むエラーメッセージを報告
  - _Requirements: 1.4, 2.7, 6.1, 6.2_

- [ ] 5.4 SearchContext 構築・登録機能を実装
  - `SceneRegistry` から `SceneTable` を構築
  - `WordDefRegistry` から `WordTable` を構築
  - `SearchContext::new(scene_table, word_table)` で検索装置を構築
  - `package.loaded["@pasta_search"]` に SearchContext を UserData として登録
  - 既存の `@pasta_search` がある場合は置換
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [ ] 5.5 finalize_scene_impl() 統合関数を実装
  - `collect_scenes()` と `collect_words()` を呼び出し
  - レジストリとSearchContextを構築
  - 成功時は `Ok(true)` を返す
  - エラー発生時は `LuaError` として伝播
  - debug_mode 有効時は収集したシーン数・単語数をログ出力
  - SearchContext 構築成功時に情報レベルのログを出力
  - _Requirements: 1.1, 2.6, 4.2, 6.4, 6.5_

- [ ] 5.6 (P) register_finalize_scene() バインディング関数を実装
  - `lua.create_function()` で Rust 関数を作成
  - `finalize_scene_impl(lua)` を呼び出すクロージャ
  - `package.loaded["pasta"]` テーブルに `finalize_scene` フィールドとして登録
  - Lua側スタブ関数を上書き
  - _Requirements: 4.1, 4.3, 4.4_

- [ ] 5.7 (P) 将来拡張ポイントを設計
  - `collect_scenes()` と `collect_words()` を個別関数に分離（既に実装済み）
  - `finalize_scene_impl()` に追加の辞書タイプを受け入れる構造を確保
  - アクター辞書収集用の関数追加スロットをコメントで示す
  - _Requirements: 7.1, 7.2, 7.3_

- [ ] 6. runtime/mod.rs を修正
- [ ] 6.1 SearchContext 初期構築を削除
  - `with_config()` 関数から `crate::search::register()` 呼び出しを削除（lines 160 付近）
  - `@pasta_search` モジュールは `finalize_scene()` で登録される旨をコメント追加
  - _Requirements: 5.1, 5.4_

- [ ] 6.2 (P) runtime/finalize.rs モジュールを export
  - `mod finalize;` 宣言を追加
  - `pub(crate) use finalize::register_finalize_scene;` を追加
  - `from_loader_with_scene_dic()` 等の初期化関数で `register_finalize_scene(&lua)` を呼び出し
  - _Requirements: 4.3_

### Testing Layer

- [ ] 7. 既存テストファイルに finalize_scene() 呼び出しを追加
- [ ] 7.1 search_module_test.rs の全16テストを修正
  - `PastaLuaRuntime::new(ctx).unwrap()` の直後に `runtime.exec(r#"require('pasta').finalize_scene()"#).unwrap()` を追加
  - すべてのテストが `@pasta_search` モジュールに依存
  - _Requirements: 1.1, 2.6, 3.3, 5.1, 5.2_

- [ ] 7.2 stdlib_modules_test.rs の minimal_config テストを修正
  - `test_minimal_config_only_search_available()` (lines 263付近) に finalize_scene() 呼び出しを追加
  - 他17箇所は `@pasta_search` 非依存のため修正不要
  - _Requirements: 1.1, 2.6, 3.3, 5.1, 5.2_

- [ ] 8. 統合テストを追加
- [ ] 8.1 シーン辞書収集の E2E テストを実装
  - トランスパイル → Lua実行 → finalize_scene → シーン検索の完全なフロー
  - 複数グローバルシーンとローカルシーンを含むテストケース
  - 同名シーンのカウンタ機能を検証
  - _Requirements: 1.1, 1.2, 1.4, 8.2, 8.4_

- [ ] 8.2 単語辞書収集の E2E テストを実装
  - トランスパイル → Lua実行 → finalize_scene → 単語検索の完全なフロー
  - グローバル単語とローカル単語の両方を含むテストケース
  - ビルダーパターン API のメソッドチェーンを検証
  - _Requirements: 2.1, 2.2, 2.6, 2.7, 9.3, 9.5_

- [ ] 8.3 エラーハンドリングのテストを実装
  - 空レジストリでの finalize_scene() 実行（警告ログ + 空SearchContext）
  - 重複シーン名での SceneTable 構築失敗
  - Luaレジストリアクセス失敗時のエラーメッセージ検証
  - _Requirements: 1.3, 6.1, 6.2, 6.3_

- [ ] 8.4* 初期化タイミング制御のテストを実装（オプション）
  - finalize_scene() 複数回呼び出しでの SearchContext 再構築
  - SearchContext 未構築状態での検索実行エラー
  - scene_dic.lua ロード前の @pasta_search モジュール不存在を許容
  - _Requirements: 5.2, 5.3, 5.4_

## Implementation Notes

### Task Execution Order
1. **Lua Script Layer** (Tasks 1-3): Lua側の基盤を先に構築
2. **Transpiler Layer** (Task 4): トランスパイラ出力を修正し、Lua側APIを呼び出すコードを生成
3. **Runtime Layer** (Tasks 5-6): Rust側の収集・構築ロジックを実装
4. **Testing Layer** (Tasks 7-8): 既存テスト修正と新規E2Eテストで検証

### Parallel Execution
- `(P)` マークの付いたタスクは並行実行可能
- Task 1.1, 1.2 は pasta.scene 内で独立（並行可能）
- Task 2.1, 2.2, 2.3 は pasta.word 内で独立（並行可能）
- Task 3.1, 3.2 は pasta.init 内だが異なる関数（並行可能）
- Task 4.2, 4.3 は異なるAST走査対象（並行可能）
- Task 5.6, 5.7 は runtime/finalize.rs 内だが異なる関数（並行可能）
- Task 6.2 は runtime/mod.rs の独立した変更（並行可能）

### Design References
- **Architecture Pattern & Boundary Map**: design.md の Mermaid 図を参照し、Lua ↔ Rust 境界を意識
- **Component Contracts**: design.md Section "Components and Interfaces" の依存関係を確認
- **AST Structure**: `FileItem::GlobalWord(KeyWords)`, `GlobalSceneScope::words: Vec<KeyWords>` を使用

### Migration Strategy
- 既存の17箇所のテスト修正は機械的なパターン適用
- search_module_test.rs: 全16箇所に finalize_scene() 追加
- stdlib_modules_test.rs: 1箇所のみ修正（他は @pasta_search 非依存）

### Quality Checkpoints
- Task 5.5 完了時: finalize_scene_impl() の単体テストを実行
- Task 7.2 完了時: 既存テスト17箇所の回帰テストを実行
- Task 8.4 完了時: 全E2Eテストを実行し、SearchContext構築フローを検証
