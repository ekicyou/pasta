# Implementation Plan: surface-dictionary-sync

## タスク概要

全11箇所の表情名置換と整合性検証テストの実装。アーキテクチャ変更なし、データ修正のみ。

---

## 実装タスク

- [ ] 1. スクリプト定数の表情名置換 (P)
- [ ] 1.1 BOOT_PASTA の表情名置換 (P)
  - OnFirstBoot シーンの `男の子：＠元気　ぼくは男の子。ちゃんと使ってよね。` → `＠笑顔` に置換
  - design.md State Management テーブル #1 に従う
  - _Requirements: 1.1, 1.2, 2.1_

- [ ] 1.2 TALK_PASTA の表情名置換（＠元気） (P)
  - OnTalk シーンの `Lua 側も触ってみなよ。` → `＠笑顔` に置換（#2）
  - OnTalk シーンの `しょうがないなあ。` → `＠照れ` に置換（#3）
  - OnHour シーンの `もう ＄時１２ か、早いね。` → `＠笑顔` に置換（#4）
  - design.md State Management テーブル #2, #3, #4 に従う
  - _Requirements: 1.1, 1.2, 2.1_

- [ ] 1.3 TALK_PASTA の表情名置換（＠考え） (P)
  - OnTalk シーン 女の子 `今日は何しようかな...` → `＠眠い` に置換（#8）
  - OnTalk シーン 男の子 `さあ、外見てないからわかんないや。` → `＠困惑` に置換（#9）
  - OnHour シーン 女の子 `＄時１２ ...時間が経つのって不思議だね。` → `＠通常` に置換（#10）
  - OnHour シーン 男の子 `哲学的だね。` → `＠通常` に置換（#11）
  - design.md State Management テーブル #8, #9, #10, #11 に従う
  - _Requirements: 1.1, 1.2, 2.2_

- [ ] 1.4 CLICK_PASTA の表情名置換 (P)
  - OnMouseDoubleClick シーン `どうしたの？` → `＠笑顔` に置換（#5）
  - OnMouseDoubleClick シーン `照れてるの？` → `＠キラキラ` に置換（#6）
  - OnMouseDoubleClick シーン `ふふん、ぼくのことが気になる？` → `＠キラキラ` に置換（#7）
  - design.md State Management テーブル #5, #6, #7 に従う
  - _Requirements: 1.1, 1.2, 2.1_

- [ ] 2. 表情名整合性検証テストの実装
- [ ] 2.1 ExpressionConsistencyTest の実装
  - スクリプト定数（BOOT_PASTA, TALK_PASTA, CLICK_PASTA）から `＠表情名` を抽出する
  - 抽出時にグローバル単語辞書定義（`＠終了挨拶` / `＠雑談` 等）を除外する
  - シーン内（`＊` ブロック内）のアクション行 `アクター名：＠表情名　セリフ` パターンのみを対象とする
  - 抽出した全表情名が ACTORS_PASTA に定義済みであることを検証する
  - 参考実装: `scripts.rs` mod tests の `contains_global_actor_dictionary()` ヘルパー関数
  - テストファイル: `crates/pasta_sample_ghost/src/scripts.rs` の `#[cfg(test)]` セクション
  - テスト名: `test_script_expression_names_defined_in_actors`
  - _Requirements: 1.3, 3.2_

- [ ] 3. リグレッションテストの実行
- [ ] 3.1 全テストの実行確認
  - `cargo test --all` を実行し全テストがパスすることを確認する
  - 既存テスト（`test_actors_pasta_contains_all_characters`, `test_pasta_scripts`, `test_expression_variations` 等）の互換性を維持する
  - E2E テストは独自フィクスチャを使用しておりサンプルゴースト非参照のため影響なしを確認する
  - _Requirements: 4.1, 4.2_

- [ ] 4. ドキュメント整合性の確認と更新
- [ ] 4.1 プロジェクトドキュメントとの整合性確認
  - SOUL.md - コアバリュー・設計原則との整合性確認（該当なし）
  - doc/spec/ - 言語仕様の更新（該当なし）
  - GRAMMAR.md - 文法リファレンスの同期（該当なし）
  - TEST_COVERAGE.md - 新規テスト（`test_script_expression_names_defined_in_actors`）のマッピング追加
  - クレート README - API 変更の反映（該当なし）
  - steering/* - 該当領域のステアリング更新（該当なし）
  - _Requirements: 3.1, 3.2_

---

## 要件カバレッジ検証

| 要件 | タスク |
|------|--------|
| 1.1 | 1.1, 1.2, 1.3, 1.4 |
| 1.2 | 1.1, 1.2, 1.3, 1.4 |
| 1.3 | 2.1 |
| 2.1 | 1.1, 1.2, 1.4 |
| 2.2 | 1.3 |
| 3.1 | 4.1 |
| 3.2 | 2.1, 4.1 |
| 4.1 | 3.1 |
| 4.2 | 3.1 |

全9要件をカバー済み。
