# Implementation Tasks

## Overview

実装タスク一覧。`pasta.shiori.act` モジュールによるさくらスクリプト組み立て機能を実装する。

---

## Tasks

### 1. 親モジュール（pasta.act）の拡張

- [ ] 1.1 ACT_IMPL の公開
  - `pasta/act.lua` の末尾に `ACT.IMPL = ACT_IMPL` を追加
  - 既存テストで回帰確認（`cargo test` で pasta.act の動作確認）
  - _Requirements: 1_

### 2. pasta.shiori.act モジュールの基盤構築

- [ ] 2.1 (P) モジュールファイルの作成と骨格実装
  - `crates/pasta_lua/scripts/pasta/shiori/act.lua` を作成
  - モジュールテーブル `SHIORI_ACT` と実装メタテーブル `SHIORI_ACT_IMPL` を定義
  - `require("pasta.act")` で親モジュールを参照
  - 継承チェーン設定: `setmetatable(SHIORI_ACT_IMPL, {__index = ACT.IMPL})`
  - `SHIORI_ACT.IMPL = SHIORI_ACT_IMPL` で公開
  - _Requirements: 1, 8_

- [ ] 2.2 (P) コンストラクタの実装
  - `SHIORI_ACT.new(ctx)` を実装
  - `ACT.new(ctx)` を呼び出してベースインスタンスを生成
  - `_buffer` フィールド（空テーブル）を追加
  - `_current_scope` フィールド（nil）を追加
  - `setmetatable(obj, SHIORI_ACT_IMPL)` でメタテーブルを設定
  - インスタンスを返却
  - _Requirements: 2_

### 3. さくらスクリプトタグ生成メソッドの実装

- [ ] 3.1 (P) バッファ操作メソッドの実装
  - `build()`: `_buffer` を `table.concat()` で結合し、末尾に `\e` を追加して返却
  - `reset()`: `_buffer` を空テーブルにリセット、`_current_scope` を nil にリセット
  - メソッドチェーン対応（`return self`）
  - _Requirements: 6, 7_

- [ ] 3.2 (P) 基本タグ生成メソッドの実装
  - `surface(id)`: `\s[id]` をバッファに追加（数値・文字列両対応）
  - `wait(ms)`: `\w[ms]` をバッファに追加（負数は 0 に補正）
  - `newline(n)`: `\n` を n 回バッファに追加（デフォルト n=1、n<1 時は何もしない）
  - `clear()`: `\c` をバッファに追加
  - メソッドチェーン対応（`return self`）
  - _Requirements: 4, 5_

- [ ] 3.3 talk メソッドのオーバーライド実装
  - `talk(actor, text)` メソッドを実装
  - `actor.spot` からスコープ番号を決定（`"sakura"` → 0, `"kero"` → 1, `"char<N>"` → N）
  - `_current_scope` と異なる場合: スコープタグ（`\0`, `\1`, `\p[N]`）とスコープ切り替え時の改行を追加
  - テキストをエスケープ（`\` → `\\`, `%` → `%%`）してバッファに追加
  - テキスト後に改行を追加
  - `_current_scope` を更新
  - 親クラスの `talk()` も呼び出し（token バッファ用）
  - メソッドチェーン対応（`return self`）
  - _Requirements: 3_

### 4. テストの実装

- [ ] 4.1 継承検証テストの作成
  - `crates/pasta_lua/tests/lua_specs/shiori_act_spec.lua` を作成
  - 親クラスメソッド（`sakura_script()` 等）の継承動作を検証
  - `SHIORI_ACT_IMPL` のメタテーブルチェーンが正しく設定されていることを確認
  - _Requirements: 9_

- [ ] 4.2 (P) talk メソッドのテスト
  - スコープ切り替え動作を検証（actor 変更時にスコープタグと改行が発行される）
  - 同一 actor での連続 `talk()` でスコープタグが再発行されないことを確認
  - スコープタグのパターン検証（`\0`, `\1`, `\p[2]` 等）
  - _Requirements: 9_

- [ ] 4.3 (P) タグ生成メソッドのテスト
  - `surface(5)` → `\s[5]\e`, `surface("smile")` → `\s[smile]\e`
  - `wait(500)` → `\w[500]\e`
  - `newline()` → `\n\e`, `newline(3)` → `\n\n\n\e`
  - `clear()` → `\c\e`
  - _Requirements: 9_

- [ ] 4.4 (P) エスケープ処理のテスト
  - バックスラッシュ（`\` → `\\`）のエスケープ検証
  - パーセント（`%` → `%%`）のエスケープ検証
  - 複数特殊文字の組み合わせ検証
  - _Requirements: 9_

- [ ] 4.5 (P) build/reset メソッドのテスト
  - `build()` が `\e` 終端を付与することを検証
  - 空バッファ時に `\e` のみ返却されることを確認
  - `reset()` がバッファとスコープをクリアすることを検証
  - メソッドチェーンの動作を確認
  - _Requirements: 9_

- [ ] 4.6 (P) E2Eシナリオテスト
  - 複数メソッドを組み合わせた実践的なスクリプト生成を検証
  - 例: `act:talk(sakura, "こんにちは"):surface(5):wait(500):talk(kero, "やあ"):build()`
  - 生成されたさくらスクリプトが期待通りの構造であることを確認
  - _Requirements: 9_

### 5. ドキュメント整合性の確認と更新

- [ ] 5.1 実装完了後のドキュメント整合性確認
  - SOUL.md - コアバリュー・設計原則との整合性確認（SHIORI基盤の提供、日本語フレンドリー）
  - SPECIFICATION.md - 言語仕様の更新（該当なし）
  - GRAMMAR.md - 文法リファレンスの同期（該当なし）
  - TEST_COVERAGE.md - 新規テストのマッピング追加（`shiori_act_spec.lua`）
  - クレート README - API変更の反映（`pasta_lua/README.md` に `pasta.shiori.act` 追加の検討）
  - steering/* - 該当領域のステアリング更新（lua-coding.md の整合性確認）
  - 特にコアバリュー（日本語フレンドリー、SHIORI 対応）への影響を確認
  - _Requirements: 全要件_

---

## 実装順序

1. **Task 1**: 親モジュール拡張（前提条件）
2. **Task 2**: モジュール基盤（骨格とコンストラクタ）
3. **Task 3**: メソッド実装（バッファ操作 → タグ生成 → talk オーバーライド）
4. **Task 4**: テスト実装（継承 → 各メソッド → E2E）
5. **Task 5**: ドキュメント整合性確認

**並列実行可能なタスク**:
- Task 2.1, 2.2 は相互依存なし（ただし 2.2 は 2.1 の骨格が必要）
- Task 3.1, 3.2 は独立（ただし 3.3 は 3.1, 3.2 に依存）
- Task 4.1-4.6 はすべて並列実行可能（ただし Task 3 完了後）

---

## Notes

- すべてのメソッドはメソッドチェーン対応（`return self`）
- `build()` は複数回呼び出し可能（バッファはクリアしない）
- `reset()` は `_buffer` と `_current_scope` のみクリア（`token` は維持）
- テストは `lua_specs` フレームワークを使用（BDD スタイル）
