# Implementation Plan

## Phase 1: Core Implementation

- [ ] 1. データ構造の拡張
- [ ] 1.1 (P) LabelInfoにラベルID管理機能を追加
  - `LabelId` newtypeラッパーを定義（`LabelId(usize)`）
  - `LabelInfo`に`id: LabelId`フィールドを追加
  - ID生成ロジックを実装（Vec indexとの一致を保証）
  - _Requirements: 6.1_

- [ ] 1.2 (P) キャッシュ管理のためのデータ構造を実装
  - `CacheKey`構造体を実装（search_key + sorted filters）
  - `CachedSelection`構造体を実装（candidates, next_index, history）
  - `CacheKey`にHash/Eq/PartialEqトレイトを実装
  - _Requirements: 4.3, 4.4_

- [ ] 1.3 (P) PastaErrorに新規エラーバリアントを追加
  - `NoMatchingLabel`バリアント追加（label, filters含む）
  - `InvalidLabel`バリアント追加
  - `RandomSelectionFailed`バリアント追加
  - `DuplicateLabelName`バリアント追加
  - `NoMoreLabels`バリアント追加（search_key, filters含む）
  - _Requirements: 1.3, 1.4, 2.3, 3.3, 6.3_

- [ ] 2. LabelTable拡張
- [ ] 2.1 前方一致検索インデックスを構築
  - `RadixMap<Vec<LabelId>>`型の`prefix_index`フィールドを追加
  - fn_nameをキーとしたTrie構築ロジックを実装
  - 同一fn_nameの重複検出とエラー処理
  - _Requirements: 1.1, 1.2, 6.2, 6.3_

- [ ] 2.2 LabelTable内部状態を更新
  - `labels`フィールドを`Vec<LabelInfo>`に変更（HashMap → Vec）
  - `cache`フィールドを`HashMap<CacheKey, CachedSelection>`に変更
  - `shuffle_enabled`フィールドを追加（bool型、デフォルトtrue）
  - 既存`history`フィールドを削除（cacheに統合）
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 2.3 from_label_registryメソッドを実装
  - `LabelRegistry`から`Vec<LabelInfo>`への変換ロジック
  - ID割り当て（Vec index = LabelId）
  - RadixMapの構築（fn_name → Vec<LabelId>マッピング）
  - `RandomSelector`と`shuffle_enabled`の初期化
  - _Requirements: 6.1, 6.2, 6.4, 6.5_

- [ ] 2.4 resolve_label_idメソッドを実装
  - Phase 1: RadixMap.iter_prefix()による前方一致検索
  - Phase 2: 属性フィルタリング（AND条件）
  - Phase 3: CacheKeyの生成とキャッシュエントリ取得/作成
  - Phase 4: シャッフル実行（shuffle_enabled=trueの場合のみ）
  - Phase 5: 順次選択（next_index管理、history記録）
  - エラーハンドリング（NoMatchingLabel, NoMoreLabels等）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 4.1, 4.2, 4.4, 4.5_

- [ ] 2.5 (P) set_shuffle_enabledメソッドを実装
  - `shuffle_enabled`フィールドを変更する公開メソッド
  - ドキュメントコメント（テスト・デバッグ用途の説明）
  - _Requirements: 3.4_

- [ ] 2.6 (P) get_labelメソッドを実装
  - LabelIdからLabelInfo参照を返す
  - Vec indexによるO(1)アクセス
  - _Requirements: 4.4_

## Phase 2: Rune統合

- [ ] 3. pasta_stdlibブリッジの実装
- [ ] 3.1 create_moduleシグネチャを変更
  - `Arc<Mutex<LabelTable>>`引数を追加
  - 既存のモジュール登録コードを保持
  - _Requirements: 5.1, 5.5_

- [ ] 3.2 parse_rune_filters関数を実装
  - Rune Value::Unit → 空HashMapの変換
  - Rune Value::Object → HashMap<String, String>の変換
  - キー型チェック（非String keyのエラー処理）
  - 値型チェック（非String valueのエラー処理）
  - Array/Tuple/その他の型のエラー処理
  - 6つのエラーケースすべてをカバー
  - _Requirements: 5.2, 5.3_

- [ ] 3.3 select_label_to_idブリッジ関数を実装
  - Rune関数としてモジュール登録
  - parse_rune_filters()による型変換
  - LabelTable::resolve_label_id()の呼び出し（Mutex経由）
  - LabelId → i64への変換
  - エラーメッセージのString変換（Rune側panic用）
  - ロック取得失敗時のエラー処理
  - _Requirements: 5.1, 5.2, 5.4, 5.5_

## Phase 3: テストとドキュメント

- [ ] 4. ユニットテストの実装
- [ ] 4.1 前方一致検索のテストケースを実装
  - グローバルラベル検索（"会話" → "会話_1::__start__", "会話_2::__start__"）
  - ローカルラベル検索（"会話_1::選択肢" → "会話_1::選択肢_1", "会話_1::選択肢_2"）
  - 候補なしエラー（存在しない検索キー → LabelNotFound）
  - 空文字列エラー（"" → InvalidLabel）
  - 連番無視の前方一致（"会話" で "会話_1", "会話_2" 両方マッチ）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [ ] 4.2 属性フィルタリングのテストケースを実装
  - 単一フィルタ適用（{"time": "morning"}）
  - 複数フィルタAND条件（{"time": "morning", "weather": "sunny"}）
  - フィルタ不一致エラー（NoMatchingLabel）
  - 空フィルタ（フィルタリングスキップ）
  - 属性なしラベルの除外
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [ ] 4.3 ランダム選択のテストケースを実装
  - MockRandomSelector呼び出し確認（複数候補時）
  - 単一候補時のランダム選択スキップ
  - RandomSelectionFailedエラー（select_index() → None）
  - 2回目呼び出しで異なる候補が返る確認
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 4.4 キャッシュベース消化のテストケースを実装
  - 同一キー2回呼び出しで異なるIDが返る
  - 全消化後のNoMoreLabelsエラー
  - フィルタ別履歴管理（同一search_key、異なるfilters）
  - ラベルIDベース履歴（配列インデックスではない）
  - 履歴記録の正確性確認
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [ ] 4.5 Rune Value変換のテストケースを実装
  - Value::Unit → 空HashMap
  - Value::Object有効ケース → HashMap変換
  - 非String keyエラー
  - 非String valueエラー
  - Array型エラー
  - 複数フィルタObject変換
  - _Requirements: 5.2_

- [ ] 4.6 (P) LabelRegistry変換のテストケースを実装
  - 有効なLabelRegistry → LabelTable構築成功
  - 重複fn_nameエラー（DuplicateLabelName）
  - ID割り当ての正確性確認
  - RadixMap構築の検証
  - _Requirements: 6.1, 6.3_

- [ ] 5. 統合テストの実装
- [ ] 5.1 エンドツーエンド実行テストを実装
  - Pasta DSL → トランスパイル → Rune実行 → ラベル解決 → 関数実行
  - `examples/`に簡易スクリプトを作成
  - `cargo run --example`での動作確認
  - _Requirements: 1.1, 2.1, 3.1, 4.1, 5.1, 6.1_

- [ ] 5.2 決定論的テストを実装
  - MockRandomSelectorでシード固定
  - shuffle_enabled=falseモード検証
  - 期待される順序でのラベル選択確認
  - テスト再現性の保証
  - _Requirements: 3.1, 4.1_

- [ ] 5.3 (P) スレッドセーフ性テストを実装
  - `Arc<Mutex<LabelTable>>`の複数スレッド同時アクセス
  - デッドロックや不整合の検出
  - 結果の一貫性検証
  - _Requirements: 5.5, 6.5_

- [ ] 6. パフォーマンス検証
- [ ] 6.1 ベンチマークテストを実装
  - criterion.rsによる測定（N=100/300/500/1000）
  - 前方一致候補10%、フィルタ適用後5候補の条件
  - キャッシュミス時の初回実行時間測定
  - `crates/pasta/benches/label_resolution.rs`に実装
  - _Requirements: 1.1, 1.2_

- [ ] 6.2 (P) メモリ使用量測定を実施
  - LabelTable全体のメモリ使用量確認
  - 目標: 数MB以下（N=500ラベル）
  - プロファイリングツールの使用
  - _Requirements: 6.1_

- [ ] 6.3 (P) ベンチマーク結果をドキュメント化
  - README.mdにパフォーマンス結果を追記
  - 性能目標達成の確認（N=500で平均5ms以下、P95で10ms以下）
  - メモリ使用量の文書化
  - _Requirements: 1.1_

- [ ] 7. 実装完了検証
- [ ] 7.1 全テストの実行と成功確認
  - `cargo test --all-targets`を実行し、全テストが成功することを確認
  - 本仕様の実装により失敗したテストがある場合、本仕様のスコープとして修正
  - テストのリグレッション（既存機能の破壊）がないことを保証
  - 実装完了の必須条件として、全テスト成功状態を維持
  - **注記**: 実装開始時点では全テスト成功状態を確認済み
  - _Requirements: 1.1, 2.1, 3.1, 4.1, 5.1, 6.1_

## Task Summary

- **Total**: 7 major tasks, 26 sub-tasks
- **Requirements Coverage**: All 30 requirements (1.1-1.5, 2.1-2.5, 3.1-3.4, 4.1-4.5, 5.1-5.5, 6.1-6.5) mapped to tasks
- **Parallel Execution**: 11 tasks marked with (P) for parallel capability
- **Average Task Size**: 1-3 hours per sub-task
- **Completion Criteria**: All tests pass with `cargo test --all-targets`, no regressions

## Implementation Notes

### Phase Dependencies
- Phase 1は完全に独立して実行可能
- Phase 2はPhase 1のLabelTable実装に依存
- Phase 3はPhase 1-2の実装完了後に実行

### Parallel Execution Strategy
- Phase 1: 1.1, 1.2, 1.3は完全に並列実行可能（異なるファイル）
- Phase 1: 2.5, 2.6は2.1-2.4完了後に並列実行可能
- Phase 3: 4.6, 5.3, 6.2, 6.3は他タスクと並列実行可能

### Testing Strategy
- 各Phase完了時に対応するユニットテストを実行
- Phase 2完了後に統合テストを実行
- Phase 3で全体的なパフォーマンス検証

### Backward Compatibility
- 既存の`find_label()`メソッドは変更しない
- `from_label_registry()`のシグネチャ変更により、既存テストコード6箇所の修正が必要
- 移行パターン: `from_label_registry(registry, selector)` → `let mut table = ...; table.set_shuffle_enabled(false);`
