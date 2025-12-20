# Implementation Tasks: pasta-word-definition-dsl

## Overview
実装タスク一覧。9つの要件をトランスパイラ層（Pass 1・Pass 2）、ランタイム層、stdlib層、ドキュメント層に分割した、5つのメジャータスクで構成。

---

## Tasks

### 1. WordDefRegistry の実装（トランスパイラ Pass 1）

- [ ] 1.1 (P) WordEntry 構造体とレジストリの基本実装
  - `WordEntry { id, key, values }` 構造体を定義
  - `WordDefRegistry::new()` でレジストリを初期化
  - エントリ ID の自動採番ロジックを実装
  - `LabelRegistry::sanitize_name()` と同一のサニタイズロジックを再利用
  - _Requirements: 1.2, 2.2, 5.5_

- [ ] 1.2 (P) グローバル単語定義の登録
  - `register_global(name, values) -> usize` を実装
  - キー形式 `"単語名"` で登録
  - 同名複数定義時に各定義を独立したエントリとして保持（早期マージなし）
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [ ] 1.3 (P) ローカル単語定義の登録
  - `register_local(module_name, name, values) -> usize` を実装
  - キー形式 `":モジュール名:単語名"` で登録
  - モジュール名のサニタイズ処理を組み込む
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [ ] 1.4 (P) Pass 1 トランスパイラ処理への統合
  - `Transpiler::transpile_pass1()` を拡張し、`PastaFile::global_words` を処理
  - `WordDefRegistry::register_global()` で全グローバル単語を登録
  - `LabelDef::local_words` を処理し、各ラベルのローカル単語を登録
  - レジストリをランタイムに渡す準備
  - _Requirements: 1.1, 2.1, 5.6_

- [ ] 1.5 `WordDefRegistry::all_entries()` でエントリ一覧を取得
  - テストと `WordTable::from_word_def_registry()` での利用を可能に
  - _Requirements: 5.5_

### 2. WordTable の実装（ランタイム層）

- [ ] 2.1 (P) WordTable 構造体と RadixMap インデックスの実装
  - `WordTable { entries, prefix_index, cached_selections, random_selector }` を定義
  - `entries: Vec<WordEntry>` でエントリを保持
  - `prefix_index: RadixMap<Vec<usize>>` で前方一致インデックスを構築
  - `cached_selections: HashMap<(String, String), CachedSelection>` でキャッシュを管理
  - `WordTable::new(random_selector) -> Self` を実装
  - _Requirements: 5.7, 4.1, 4.7_

- [ ] 2.2 (P) `from_word_def_registry()` でレジストリからテーブルを構築
  - `WordDefRegistry` から `entries` を取得
  - `RadixMap` に全エントリをインデックス化（キー → エントリ ID リスト）
  - 重複 ID 検出エラーハンドリング（防御的プログラミング）
  - _Requirements: 5.7, 4.1, 4.7_

- [ ] 2.3 (P) 2 段階検索ロジック（ローカル + グローバル統合）
  - Step 1: `:module:key` 前方一致でローカルエントリ ID リストを取得
  - Step 2: `key` 前方一致でグローバルエントリ ID リストを取得
  - Step 3: 両リストを結合してマージ（`Vec::extend`）
  - 統合単語リストを返す
  - _Requirements: 4.1, 4.2, 4.3, 2.6_

- [ ] 2.4 シャッフルキャッシュの初期化と再構築
  - `CachedWordSelection { words, next_index }` を定義
  - キャッシュ未作成時にシャッフル実行（`random_selector.shuffle()`）
  - キャッシュ枯渇時に全単語を再シャッフル
  - _Requirements: 4.4, 4.5, 4.6, 4.8_

- [ ] 2.5 `search_word(module_name, key, filters) -> Result<String, PastaError>` の実装
  - 全 5 ステップを実装（2 段階検索 → 統合 → キャッシュ処理 → Pop → 返却）
  - 未ヒット時に `Err(PastaError::WordNotFound)` を返却
  - キャッシュキーを `(module_name, key)` で管理
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.8_

- [ ] 2.6 テスト支援機能（シャッフル無効化）
  - `set_shuffle_enabled(bool)` メソッドで決定的テストを可能に
  - _Requirements: 5.7_

### 3. トランスパイラ Pass 2（単語参照のコード生成）

- [ ] 3.1 TranspileContext への `current_module` フィールド追加
  - グローバルラベル処理時にモジュール名を設定
  - `set_current_module()` / `current_module()` アクセッサ実装
  - _Requirements: 3.2, 5.5_

- [ ] 3.2 (P) 会話行内の単語参照の Rune コード生成
  - `SpeechPart::FuncCall` で単語参照を検出
  - `yield Talk(pasta_stdlib::word("モジュール名", "単語名", []))` に変換
  - グローバルスコープではモジュール名を空文字列に設定
  - フィルタ引数を渡す準備（現在は空配列）
  - _Requirements: 3.2, 3.3, 6.1_

- [ ] 3.3 実装の統合テスト
  - Pass 1（単語定義収集） → Pass 2（コード生成） のフロー全体をテスト
  - _Requirements: 3.1, 3.2_

### 4. pasta_stdlib::word() 関数の実装（Stdlib 層）

- [ ] 4.1 (P) `word(module_name, key, filters) -> String` の実装
  - `WordTable::search_word()` を呼び出し
  - `Ok(word)` → そのまま単語を返却
  - `Err(PastaError::WordNotFound)` → WARN ログ発行、空文字列返却
  - その他 `Err` → ERROR ログ発行、空文字列返却
  - no panic 原則を遵守
  - _Requirements: 3.2, 3.3, 3.4, 3.5, 7.1, 7.2_

- [ ] 4.2 ログメッセージの実装
  - 日本語エラーメッセージ（例：「単語定義 @場所 が見つかりません」）を提供
  - ラベルと単語の区別が明確なメッセージを心がける
  - _Requirements: 7.3_

- [ ] 4.3 `WordTable` への Mutex ラッピングと登録
  - stdlib 関数から参照できるように `Mutex<WordTable>` でラップ
  - Rune VM にコンテキストとして登録
  - ロック失敗時のエラーハンドリング
  - _Requirements: 4.1_

### 5. テスト・ドキュメント・検証

- [ ] 5.1 (P) WordDefRegistry ユニットテスト
  - `register_global()` と `register_local()` の基本動作
  - キー形式の正確性（`"key"` vs `":module:key"`）
  - エントリ ID の連番性と重複なし
  - サニタイズロジックの検証
  - _Requirements: 9.1_

- [ ] 5.2 (P) WordTable ユニットテスト
  - `search_word()` の 2 段階検索ロジック
  - 前方一致のエッジケース（単語名前方一致、複数マッチ、マッチなし）
  - シャッフルキャッシュの初回シャッフルと再シャッフル
  - キャッシュキー `(module_name, key)` の分離確認
  - _Requirements: 9.2, 9.3, 9.4_

- [ ] 5.3 (P) Call/Jump 文から単語辞書非アクセスの検証
  - 既存 `select_label_to_id()` がラベル辞書のみを参照することを確認
  - 統合テストで単語辞書へのアクセスがないことを検証
  - _Requirements: 6.1, 6.2, 6.3, 9.5_

- [ ] 5.4 エンドツーエンド統合テスト
  - Pasta スクリプト → トランスパイラ → Rune VM → 単語選択
  - グローバル/ローカル定義の混合テスト
  - 前方一致複合検索の動作確認
  - _Requirements: 1.1, 2.1, 3.1, 4.1, 5.1_

- [ ] 5.5 GRAMMAR.md への単語定義セクション追加
  - 構文形式（`＠単語名：単語1　単語2`）を説明
  - グローバル/ローカルスコープの使い分け方法
  - 会話行内からの参照方法と複数例
  - 単語辞書検索の前方一致ロジック（ローカル→グローバル統合マージ）
  - 同名定義の自動マージ動作
  - 前方一致検索の具体例（`＠挨` が `＠挨拶` / `＠挨拶_朝` にマッチ）
  - Call/Jump から単語辞書がアクセスされないことを明記
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_

- [ ] 5.6 サンプルスクリプト作成（3 つ以上）
  - グローバル単語定義サンプル
  - ローカル単語定義と前方一致複合検索サンプル
  - グローバル/ローカル混合参照サンプル
  - _Requirements: 8.9_

- [ ] 5.7 引用符エスケープの動作確認（ドキュメント化）
  - 例：`「「test」」` → `「test」` の動作をドキュメント化
  - パーサー層で既に実装済みであることを確認
  - _Requirements: 8.8_

---

## Task Progression

**Phase 1 (Transpiler & Runtime Core)**:
- タスク 1: WordDefRegistry（Pass 1 収集）
- タスク 2: WordTable（ランタイム検索）

**Phase 2 (Code Generation & API)**:
- タスク 3: Pass 2 コード生成
- タスク 4: pasta_stdlib::word() 実装

**Phase 3 (Testing & Documentation)**:
- タスク 5: テスト・ドキュメント・検証

**実装順序**: 1 → 2 → 3 → 4 → 5（推奨順序）

---

## Parallel Opportunities

タスク 1.1, 1.2, 1.3 は独立した API なので並行実装可能。
タスク 2.1, 2.2, 2.3, 2.4 も互いに依存しないため並行実装可能。
ただし 1 の完了後に 2 を開始、2 の完了後に 3, 4 を開始する必要がある。

---

## Requirement Coverage

| 要件 | タスク | 状態 |
|------|--------|------|
| 1 (グローバルスコープ) | 1.1-1.5, 2.1-2.5 | ✓ |
| 2 (ローカルスコープ) | 1.3, 1.4, 2.1-2.5 | ✓ |
| 3 (会話内参照と展開) | 3.1-3.3, 4.1-4.3 | ✓ |
| 4 (前方一致複合検索) | 2.3-2.5 | ✓ |
| 5 (AST・データ表現) | 1.1-1.5, 2.1-2.6 | ✓ |
| 6 (Call/Jump 非アクセス) | 5.3 (検証) | ✓ |
| 7 (エラーハンドリング) | 2.5, 4.1-4.3 | ✓ |
| 8 (ドキュメント) | 5.5-5.7 | ✓ |
| 9 (テスト) | 5.1-5.4 | ✓ |

すべての要件がタスクにマッピングされています。
