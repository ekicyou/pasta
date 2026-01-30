# Implementation Plan: lua55-manual-consistency

## Phase 1: ナビゲーションリンク統一

- [ ] 1.1 (P) 01-introduction.mdのナビゲーションリンクを詳細版に修正
  - 現在の簡易版ナビゲーションをパターン2（詳細版）に変更
  - 形式: `[← 目次](./README.md) | [次へ: 2 – 基本概念 →](02-basic-concepts.md)`
  - セパレータ`---`を確認
  - _Requirements: 1.2, 1.3_

- [ ] 1.2 (P) 05-auxiliary-library.mdにナビゲーションリンクを追加
  - パターン2形式のナビゲーションを冒頭に追加
  - 形式: `[← 前へ: 4 – C API](04-c-api.md) | [目次](./README.md) | [次へ: 6 – 標準ライブラリ →](06-standard-libraries.md)`
  - セパレータ`---`を追加
  - _Requirements: 1.2_

- [ ] 1.3 (P) 06-standard-libraries.mdにナビゲーションリンクを追加
  - パターン2形式のナビゲーションを冒頭に追加
  - 形式: `[← 前へ: 5 – 補助ライブラリ](05-auxiliary-library.md) | [目次](./README.md) | [次へ: 7 – スタンドアロン →](07-standalone.md)`
  - セパレータ`---`を追加
  - _Requirements: 1.2_

- [ ] 1.4 (P) 07-standalone.mdにナビゲーションリンクを追加
  - パターン2形式のナビゲーションを冒頭に追加
  - 形式: `[← 前へ: 6 – 標準ライブラリ](06-standard-libraries.md) | [目次](./README.md) | [次へ: 8 – 非互換性 →](08-incompatibilities.md)`
  - セパレータ`---`を追加
  - _Requirements: 1.2_

- [ ] 1.5 (P) 08-incompatibilities.mdにナビゲーションリンクを追加
  - パターン2形式のナビゲーションを冒頭に追加
  - 形式: `[← 前へ: 7 – スタンドアロン](07-standalone.md) | [目次](./README.md) | [次へ: 9 – 完全な構文 →](09-complete-syntax.md)`
  - セパレータ`---`を追加
  - _Requirements: 1.2_

- [ ] 1.6 (P) 09-complete-syntax.mdにナビゲーションリンクを追加
  - パターン2形式のナビゲーションを冒頭に追加（最後の章）
  - 形式: `[← 前へ: 8 – 非互換性](08-incompatibilities.md) | [目次](./README.md)`
  - セパレータ`---`を追加
  - _Requirements: 1.2, 1.4_

- [ ] 1.7 (P) GLOSSARY.mdにナビゲーションリンクを追加
  - 形式: `[← 目次](./README.md)`
  - セパレータ`---`を追加
  - _Requirements: 6.1, 6.3_

- [ ] 1.8 (P) LICENSE.mdにナビゲーションリンクを追加
  - 形式: `[← 目次](./README.md)`
  - セパレータ`---`を追加
  - _Requirements: 6.2, 6.3_

## Phase 2: 6章セクション重複解消

- [ ] 2.1 06-standard-libraries.md セクション6.1-6.3の重複見出しを統合
  - 6.2「基本関数」のパートA～E（5回重複）を単一見出しに統合
  - 最初の`## 6.2 – 基本関数`見出しのみ残し、後続パート見出しを削除
  - コンテンツは全て統合（削除なし）
  - アンカーIDを正規化形式`#62-基本関数`（シングルハイフン）に確認
  - 統合後の行数を元ファイルと比較検証
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 2.2 06-standard-libraries.md セクション6.4-6.6の重複見出しを統合
  - 6.4「モジュール」のパートA～B（2回重複）を統合
  - 6.5「文字列操作」のパートA～D（4回重複）を統合
  - 最初の見出しのみ残し、後続パート見出しを削除
  - コンテンツは全て統合
  - アンカーIDを正規化形式に確認
  - 統合後の行数を元ファイルと比較検証
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 2.3 06-standard-libraries.md セクション6.7-6.9の重複見出しを統合
  - 6.8「数学関数」のパート（5回重複）を統合
  - 6.9「入出力機能」のパート（4回重複）を統合
  - 最初の見出しのみ残し、後続パート見出しを削除
  - コンテンツは全て統合
  - アンカーIDを正規化形式に確認
  - 統合後の行数を元ファイルと比較検証
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 2.4 06-standard-libraries.md セクション6.10-6.11の重複見出しを統合
  - 6.10「オペレーティングシステム機能」のパート（2回重複）を統合
  - 6.11「デバッグライブラリ」のパート（3回重複）を統合
  - 最初の見出しのみ残し、後続パート見出しを削除
  - コンテンツは全て統合
  - アンカーIDを正規化形式に確認
  - 統合後の行数を元ファイルと比較検証
  - 最終的に6章のセクション数が11個（6.1-6.11）であることを確認
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

## Phase 3: リンク検証・修正

- [ ] 3.1 index.md内の目次リンクを検証・修正
  - 全章へのリンク（約50件）を確認
  - 6章のアンカーID変更に伴うリンク修正（`#62--基本関数` → `#62-基本関数`）
  - サンプルリンクで動作確認
  - _Requirements: 3.5, 5.1, 5.2_

- [ ] 3.2 index.md内のLua関数索引リンクを検証・修正
  - Lua関数索引（約80件）を確認
  - 6章関数へのリンクをアンカーID変更に合わせて修正
  - サンプルリンクで動作確認
  - _Requirements: 3.5, 5.3_

- [ ] 3.3 index.md内のC API索引リンクを検証・修正
  - C API索引（約50件）を確認
  - 該当するリンクをアンカーID変更に合わせて修正
  - サンプルリンクで動作確認
  - _Requirements: 3.5, 5.3_

- [ ] 3.4 index.md内の型索引リンクを検証・修正
  - 型索引（約20件）を確認
  - 該当するリンクをアンカーID変更に合わせて修正
  - サンプルリンクで動作確認
  - _Requirements: 3.5, 5.3_

## Phase 4: メタデータ統一

- [ ] 4.1 (P) 05-auxiliary-library.mdのメタデータをHTMLコメント形式に変換
  - 現在のblockquote形式を削除
  - HTMLコメント形式のメタデータを冒頭に追加
  - 内容: 原文URL、参考URL、翻訳日、レビュー情報、用語対照
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 4.2 (P) 07-standalone.mdのメタデータをHTMLコメント形式に変換
  - 現在のblockquote形式を削除
  - HTMLコメント形式のメタデータを冒頭に追加
  - 内容: 原文URL、参考URL、翻訳日、レビュー情報、用語対照
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 4.3 (P) 08-incompatibilities.mdのメタデータをHTMLコメント形式に変換
  - 現在のblockquote形式を削除
  - HTMLコメント形式のメタデータを冒頭に追加
  - 内容: 原文URL、参考URL、翻訳日、レビュー情報、用語対照
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 4.4 (P) 09-complete-syntax.mdのメタデータをHTMLコメント形式に変換
  - 現在のblockquote形式を削除
  - HTMLコメント形式のメタデータを冒頭に追加
  - 内容: 原文URL、参考URL、翻訳日、レビュー情報、用語対照
  - _Requirements: 2.1, 2.2, 2.3_

## Phase 5: ファイル構成変更

- [ ] 5.1 現README.mdをABOUT.mdにリネーム
  - gitコマンド使用: `git mv crates/pasta_lua/doc/lua55-manual/README.md crates/pasta_lua/doc/lua55-manual/ABOUT.md`
  - 概要、翻訳方針、免責事項などのコンテンツはそのまま維持
  - _Requirements: 4.2_

- [ ] 5.2 現index.mdをREADME.mdにリネーム
  - gitコマンド使用: `git mv crates/pasta_lua/doc/lua55-manual/index.md crates/pasta_lua/doc/lua55-manual/README.md`
  - 詳細目次+索引コンテンツはそのまま維持
  - _Requirements: 4.1_

- [ ] 5.3 新README.md冒頭にHTMLコメントメタデータを追加
  - 原文URL、翻訳日、レビュー情報を含むHTMLコメントを追加
  - 既存の冒頭コンテンツの前に配置
  - _Requirements: 4.1_

- [ ] 5.4 新README.md末尾にABOUT.mdへのリンクを追加
  - セクション「翻訳について」を追加
  - 内容: `翻訳についての詳細は[ABOUT.md](ABOUT.md)を参照してください。`
  - _Requirements: 4.6_

- [ ] 5.5 index.md削除確認
  - index.mdファイルが存在しないことを確認
  - README.mdとABOUT.mdが正しく配置されていることを確認
  - _Requirements: 4.3_

- [ ] 5.6 全ファイルのナビゲーションリンクがREADME.mdを指すことを確認
  - 全章ファイル（01-09）のナビリンクを確認
  - GLOSSARY.md、LICENSE.mdのナビリンクを確認
  - 全て`./README.md`を指していることを検証
  - GitHubプレビューで目次からの遷移を確認
  - _Requirements: 4.5, 5.1, 7.4_

## バリデーション

- [ ] 6. 最終検証
  - Phase 1完了: 各章のナビリンク動作確認
  - Phase 2完了: 6章セクション数=11確認
  - Phase 3完了: リンク全件検証
  - Phase 4完了: メタデータ形式統一確認
  - Phase 5完了: README.md表示確認、全ナビリンク動作確認
  - _Requirements: 7.1, 7.2, 7.3, 7.4_
