# Implementation Plan: pasta-engine-independence

## Task Overview

本実装計画では、PastaEngineの複数インスタンス完全独立性を実現するため、グローバルキャッシュのインスタンス化、エンジン構造の変更、包括的なテストスイート追加を段階的に実施します。

**実装戦略**: 3フェーズ移行（ParseCache簡素化 → PastaEngine構造変更 → テストスイート作成）

---

## Implementation Tasks

### Phase 1: ParseCache構造の簡素化

- [ ] 1. ParseCacheをインスタンスローカル仕様に変更
- [ ] 1.1 (P) ParseCache構造体の内部実装を簡素化
  - `Arc<RwLock<HashMap>>` を `HashMap` に変更
  - `CacheEntry` の `Arc<PastaFile>` と `Arc<String>` を直接所有型に変更
  - スレッドセーフ機構（RwLock）を削除
  - _Requirements: 2.1, 2.2_

- [ ] 1.2 (P) ParseCacheメソッドシグネチャの変更
  - `get()` を `Option<(PastaFile, String)>` 返却に変更（所有値返却）
  - `insert()`, `clear()` を `&mut self` に変更
  - `len()`, `is_empty()` はそのまま維持
  - _Requirements: 2.1, 2.5_

- [ ] 1.3 (P) ParseCacheユニットテストの更新
  - 既存テスト（`cache.rs`内）を新シグネチャに適合
  - cloneコストが許容範囲であることを確認
  - キャッシュヒット/ミスの動作検証
  - _Requirements: 2.1_

### Phase 2: PastaEngine構造の変更

- [ ] 2. PastaEngineにキャッシュフィールドを追加し、グローバルキャッシュを削除
- [ ] 2.1 `static PARSE_CACHE`とグローバルキャッシュ参照を削除
  - `static PARSE_CACHE: OnceLock<ParseCache>` 削除
  - `global_cache()` 関数削除
  - デバッグ出力の調整（キャッシュヒット/ミスログ）
  - _Requirements: 1.5, 2.1_

- [ ] 2.2 PastaEngine構造体にcacheフィールドを追加
  - `cache: ParseCache` フィールド追加
  - 構造体定義の更新
  - _Requirements: 1.1, 2.1_

- [ ] 2.3 PastaEngine::with_random_selector()の構築フロー変更
  - Step 1: 空のキャッシュ作成（`let mut cache = ParseCache::new()`）
  - Step 2: キャッシュからパース結果取得（`cache.get(script)`）、ミス時はパース→トランスパイル→`cache.insert()`
  - Step 3: ラベルテーブル構築（既存処理）
  - Step 4: Runeコンパイル（既存処理）
  - Step 5: 全フィールドを持つPastaEngine構築
  - _Requirements: 1.1, 2.1, 2.3, 2.5_

- [ ] 2.4 (P) PastaEngine::new()をwith_random_selector()経由に更新
  - `new()` が `with_random_selector()` を呼び出す実装に変更
  - DefaultRandomSelectorの使用を維持
  - _Requirements: 1.1_

- [ ] 2.5 既存のengine.rs統合テストを実行し動作確認
  - 既存テストが全て成功することを確認
  - キャッシュ機能が正常に動作することを検証
  - エンジン作成・実行の基本動作を確認
  - _Requirements: 2.1, 2.3_

### Phase 3: インスタンス独立性テストの追加

- [ ] 3. engine_independence_testの作成（主要検証）
- [ ] 3.1 (P) 複数インスタンスの独立実行テスト
  - 同一プロセス内で2つのエンジンを作成
  - 異なるスクリプトを実行し、結果が相互に影響しないことを検証
  - 各エンジンが期待通りのScriptEventを返すことを確認
  - _Requirements: 1.2, 4.1_

- [ ] 3.2 (P) グローバル変数の独立性テスト
  - 2つのエンジンで同名のグローバル変数（`@*変数名`）を設定
  - 各エンジン内で独立した変数空間を持つことを検証
  - 変数値が相互に干渉しないことを確認
  - _Requirements: 1.3, 4.2_

- [ ] 3.3 (P) 独立パース・コンパイルテスト
  - 同一スクリプト文字列から複数のエンジンを作成
  - 各エンジンが独立してパース・トランスパイル・コンパイルを実行
  - 各インスタンスのキャッシュが独立していることを確認
  - _Requirements: 2.3, 4.3, 6.2_

- [ ] 3.4 (P) RandomSelector独立性テスト
  - 異なるRandomSelector実装を持つエンジンを作成
  - 乱数選択が独立していることを検証
  - 予測可能なテスト用RandomSelectorで動作確認
  - _Requirements: 1.4, 4.4_

- [ ] 3.5 (P) エンジン破棄後の独立性テスト
  - 2つのエンジンを作成し、一方を破棄
  - 破棄後も他方のエンジンが正常に動作することを確認
  - 所有権システムによる自動解放を検証
  - _Requirements: 2.5, 6.1, 6.3_

### Phase 4: 並行実行テストの追加（補助検証）

- [ ] 4. concurrent_execution_testの作成（Send実装実証）
- [ ] 4.1 (P) マルチスレッド独立実行テスト
  - 複数スレッドで独立したエンジンを作成
  - 各スレッドで`execute_label()`を実行
  - 各スレッドが期待通りのイベントを返すことを検証
  - `thread::spawn() + join()`で実装（Barrier不要）
  - _Requirements: 3.1, 3.2, 5.1, 5.4_

- [ ] 4.2 (P) マルチスレッドパース独立性テスト
  - 複数スレッドが同一スクリプトから各自のエンジンを作成
  - 各スレッドが独立してパース・コンパイルを実行
  - グローバル状態不在による構造的安全性を確認
  - _Requirements: 3.2, 3.5, 5.2, 5.3_

- [ ] 4.3 (P) Sendトレイト実装の実証テスト
  - エンジンがスレッド境界を越えて移動できることを確認
  - `Send`トレイトが自動導出されていることを検証
  - _Requirements: 3.4_

### Phase 5: 静的検証とCI統合

- [ ] 5. グローバル状態不在の検証とCI統合
- [ ] 5.1 (P) 静的チェックスクリプトの作成
  - `grep`による`static`変数検出スクリプト
  - `static mut`, `OnceLock`, `LazyLock`の不在確認
  - CI/CD統合用のチェック手順
  - _Requirements: 1.5, 6.4_

- [ ] 5.2 CI/CDでの自動テスト実行確認
  - `cargo test`で全テストが実行可能であることを確認
  - テストファイルが`crates/pasta/tests/`に配置されていることを検証
  - テスト失敗時のCI失敗を確認
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 5.3 (P) 全要件の最終検証
  - 全acceptance criteriaが満たされていることを確認
  - インスタンス独立性の包括的動作確認
  - リグレッションテストの実行
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3, 6.5, 7.1, 7.2, 7.3, 7.4, 7.5_

---

## Requirements Coverage

全35個のacceptance criteriaを15タスクでカバー：

| Requirement | Acceptance Criteria | Covered by Tasks |
|-------------|---------------------|------------------|
| 1.1 | 全データ所有 | 2.2, 2.3, 5.3 |
| 1.2 | 実行状態独立 | 3.1, 5.3 |
| 1.3 | グローバル変数独立 | 3.2, 5.3 |
| 1.4 | RandomSelector独立 | 3.4, 5.3 |
| 1.5 | static変数ゼロ | 2.1, 5.1, 5.3 |
| 2.1 | インスタンス内キャッシュ | 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.5, 5.3 |
| 2.2 | キャッシュ所有 | 1.1, 5.3 |
| 2.3 | 独立パース | 2.3, 2.5, 3.3, 5.3 |
| 2.4 | 純粋関数 | 5.3（既存実装で適合） |
| 2.5 | 自動解放 | 1.2, 2.3, 3.5, 5.3 |
| 3.1 | 並行execute_label | 4.1, 5.3 |
| 3.2 | 並行エンジン作成 | 4.1, 4.2, 5.3 |
| 3.3 | エラー独立性 | 5.3（構造的保証） |
| 3.4 | Send実装 | 4.3, 5.3 |
| 3.5 | データ競合不在 | 4.2, 5.3 |
| 4.1 | 独立実行テスト | 3.1 |
| 4.2 | グローバル変数テスト | 3.2 |
| 4.3 | 独立パーステスト | 3.3 |
| 4.4 | RandomSelectorテスト | 3.4 |
| 4.5 | テスト自動実行 | 5.2 |
| 5.1 | 並行execute_labelテスト | 4.1 |
| 5.2 | 並行パーステスト | 4.2 |
| 5.3 | データ競合不在テスト | 4.2 |
| 5.4 | 独立結果生成テスト | 4.1 |
| 5.5 | Miri対応 | 5.2（cargo test対応） |
| 6.1 | 破棄後独立性テスト | 3.5 |
| 6.2 | 同時パーステスト | 3.3 |
| 6.3 | ドロップ独立性テスト | 3.5 |
| 6.4 | 静的チェック | 5.1 |
| 6.5 | Arc非共有確認 | 5.3 |
| 7.1 | cargo test実行 | 5.2 |
| 7.2 | テストファイル配置 | 5.2 |
| 7.3 | CI検証条件 | 5.2 |
| 7.4 | CI失敗制御 | 5.2 |
| 7.5 | 標準テストFW使用 | 5.2 |

---

## Implementation Notes

### 並行実行可能タスク（P標記）
Phase 1の全タスク（1.1-1.3）、Phase 3の全テスト作成（3.1-3.5）、Phase 4の全テスト作成（4.1-4.3）、Phase 5の静的チェック（5.1, 5.3）は並行実行可能です。

### 依存関係
- Phase 2はPhase 1完了後に実行（ParseCache実装に依存）
- Phase 3-4はPhase 2完了後に実行（PastaEngine構造に依存）
- Phase 5はPhase 3-4完了後に最終検証

### 推定所要時間
- Phase 1: 2-3時間
- Phase 2: 3-4時間
- Phase 3: 4-5時間
- Phase 4: 2-3時間
- Phase 5: 1-2時間
- **合計**: 12-17時間（2-3日）
