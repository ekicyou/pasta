# Implementation Plan: lua55-manual-consistency

## Phase 1: ナビゲーションリンク統一

- [x] 1.1 (P) 01-introduction.mdのナビゲーションリンクを詳細版に修正
  - 現在の簡易版ナビゲーションをパターン2�E�詳細版）に変更
  - 形弁E `[ↁE目次](./README.md) | [次へ: 2  E基本概念 →](02-basic-concepts.md)`
  - セパレータ`---`を確誁E
  - _Requirements: 1.2, 1.3_

- [x] 1.2 (P) 05-auxiliary-library.mdにナビゲーションリンクを追加
  - パターン2形式�Eナビゲーションを�E頭に追加
  - 形弁E `[ↁE前へ: 4  EC API](04-c-api.md) | [目次](./README.md) | [次へ: 6  E標準ライブラリ →](06-standard-libraries.md)`
  - セパレータ`---`を追加
  - _Requirements: 1.2_

- [x] 1.3 (P) 06-standard-libraries.mdにナビゲーションリンクを追加
  - パターン2形式�Eナビゲーションを�E頭に追加
  - 形弁E `[ↁE前へ: 5  E補助ライブラリ](05-auxiliary-library.md) | [目次](./README.md) | [次へ: 7  Eスタンドアロン →](07-standalone.md)`
  - セパレータ`---`を追加
  - _Requirements: 1.2_

- [x] 1.4 (P) 07-standalone.mdにナビゲーションリンクを追加
  - パターン2形式�Eナビゲーションを�E頭に追加
  - 形弁E `[ↁE前へ: 6  E標準ライブラリ](06-standard-libraries.md) | [目次](./README.md) | [次へ: 8  E非互換性 →](08-incompatibilities.md)`
  - セパレータ`---`を追加
  - _Requirements: 1.2_

- [x] 1.5 (P) 08-incompatibilities.mdにナビゲーションリンクを追加
  - パターン2形式�Eナビゲーションを�E頭に追加
  - 形弁E `[ↁE前へ: 7  Eスタンドアロン](07-standalone.md) | [目次](./README.md) | [次へ: 9  E完�Eな構文 →](09-complete-syntax.md)`
  - セパレータ`---`を追加
  - _Requirements: 1.2_

- [x] 1.6 (P) 09-complete-syntax.mdにナビゲーションリンクを追加
  - パターン2形式�Eナビゲーションを�E頭に追加�E�最後�E章�E�E
  - 形弁E `[ↁE前へ: 8  E非互換性](08-incompatibilities.md) | [目次](./README.md)`
  - セパレータ`---`を追加
  - _Requirements: 1.2, 1.4_

- [x] 1.7 (P) GLOSSARY.mdにナビゲーションリンクを追加
  - 形弁E `[ↁE目次](./README.md)`
  - セパレータ`---`を追加
  - _Requirements: 6.1, 6.3_

- [x] 1.8 (P) LICENSE.mdにナビゲーションリンクを追加
  - 形弁E `[ↁE目次](./README.md)`
  - セパレータ`---`を追加
  - _Requirements: 6.2, 6.3_

## Phase 2: 6章セクション重褁E��涁E

- [x] 2.1 06-standard-libraries.md セクション6.1-6.3の重褁E���Eしを統吁E
  - 6.2「基本関数」�Eパ�EチE�E�E�E�E回重褁E��を単一見�Eしに統吁E
  - 最初�E`## 6.2  E基本関数`見�Eし�Eみ残し、後続パート見�Eしを削除
  - コンチE��チE�E全て統合（削除なし！E
  - アンカーIDを正規化形式`#62-基本関数`�E�シングルハイフン�E�に確誁E
  - 統合後�E行数を�Eファイルと比輁E��証
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 2.2 06-standard-libraries.md セクション6.4-6.6の重褁E���Eしを統吁E
  - 6.4「モジュール」�Eパ�EチE�E�B�E�E回重褁E��を統吁E
  - 6.5「文字�E操作」�Eパ�EチE�E�D�E�E回重褁E��を統吁E
  - 最初�E見�Eし�Eみ残し、後続パート見�Eしを削除
  - コンチE��チE�E全て統吁E
  - アンカーIDを正規化形式に確誁E
  - 統合後�E行数を�Eファイルと比輁E��証
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 2.3 06-standard-libraries.md セクション6.7-6.9の重褁E���Eしを統吁E
  - 6.8「数学関数」�Eパ�Eト！E回重褁E��を統吁E
  - 6.9「�E出力機�E」�Eパ�Eト！E回重褁E��を統吁E
  - 最初�E見�Eし�Eみ残し、後続パート見�Eしを削除
  - コンチE��チE�E全て統吁E
  - アンカーIDを正規化形式に確誁E
  - 統合後�E行数を�Eファイルと比輁E��証
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 2.4 06-standard-libraries.md セクション6.10-6.11の重褁E���Eしを統吁E
  - 6.10「オペレーチE��ングシスチE��機�E」�Eパ�Eト！E回重褁E��を統吁E
  - 6.11「デバッグライブラリ」�Eパ�Eト！E回重褁E��を統吁E
  - 最初�E見�Eし�Eみ残し、後続パート見�Eしを削除
  - コンチE��チE�E全て統吁E
  - アンカーIDを正規化形式に確誁E
  - 統合後�E行数を�Eファイルと比輁E��証
  - 最終的に6章のセクション数ぁE1個！E.1-6.11�E�であることを確誁E
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

## Phase 3: リンク検証・修正

- [x] 3.1 index.md冁E�E目次リンクを検証・修正
  - 全章へのリンク�E�紁E0件�E�を確誁E
  - 6章のアンカーID変更に伴ぁE��ンク修正�E�E#62--基本関数` ↁE`#62-基本関数`�E�E
  - サンプルリンクで動作確誁E
  - _Requirements: 3.5, 5.1, 5.2_

- [x] 3.2 index.md冁E�ELua関数索引リンクを検証・修正
  - Lua関数索引（紁E0件�E�を確誁E
  - 6章関数へのリンクをアンカーID変更に合わせて修正
  - サンプルリンクで動作確誁E
  - _Requirements: 3.5, 5.3_

- [x] 3.3 index.md冁E�EC API索引リンクを検証・修正
  - C API索引（紁E0件�E�を確誁E
  - 該当するリンクをアンカーID変更に合わせて修正
  - サンプルリンクで動作確誁E
  - _Requirements: 3.5, 5.3_

- [x] 3.4 index.md冁E�E型索引リンクを検証・修正
  - 型索引（紁E0件�E�を確誁E
  - 該当するリンクをアンカーID変更に合わせて修正
  - サンプルリンクで動作確誁E
  - _Requirements: 3.5, 5.3_

## Phase 4: メタチE�Eタ統一

- [x] 4.1 (P) 05-auxiliary-library.mdのメタチE�EタをHTMLコメント形式に変換
  - 現在のblockquote形式を削除
  - HTMLコメント形式�EメタチE�Eタを�E頭に追加
  - 冁E��: 原文URL、参考URL、翻訳日、レビュー惁E��、用語対照
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 4.2 (P) 07-standalone.mdのメタチE�EタをHTMLコメント形式に変換
  - 現在のblockquote形式を削除
  - HTMLコメント形式�EメタチE�Eタを�E頭に追加
  - 冁E��: 原文URL、参考URL、翻訳日、レビュー惁E��、用語対照
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 4.3 (P) 08-incompatibilities.mdのメタチE�EタをHTMLコメント形式に変換
  - 現在のblockquote形式を削除
  - HTMLコメント形式�EメタチE�Eタを�E頭に追加
  - 冁E��: 原文URL、参考URL、翻訳日、レビュー惁E��、用語対照
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 4.4 (P) 09-complete-syntax.mdのメタチE�EタをHTMLコメント形式に変換
  - 現在のblockquote形式を削除
  - HTMLコメント形式�EメタチE�Eタを�E頭に追加
  - 冁E��: 原文URL、参考URL、翻訳日、レビュー惁E��、用語対照
  - _Requirements: 2.1, 2.2, 2.3_

## Phase 5: ファイル構�E変更

- [x] 5.1 現README.mdをABOUT.mdにリネ�Eム
  - gitコマンド使用: `git mv crates/pasta_lua/doc/lua55-manual/README.md crates/pasta_lua/doc/lua55-manual/ABOUT.md`
  - 概要、翻訳方針、�E責事頁E��どのコンチE��チE�Eそ�Eまま維持E
  - _Requirements: 4.2_

- [x] 5.2 現index.mdをREADME.mdにリネ�Eム
  - gitコマンド使用: `git mv crates/pasta_lua/doc/lua55-manual/index.md crates/pasta_lua/doc/lua55-manual/README.md`
  - 詳細目次+索引コンチE��チE�Eそ�Eまま維持E
  - _Requirements: 4.1_

- [x] 5.3 新README.md冒頭にHTMLコメントメタチE�Eタを追加
  - 原文URL、翻訳日、レビュー惁E��を含むHTMLコメントを追加
  - 既存�E冒頭コンチE��チE�E前に配置
  - _Requirements: 4.1_

- [x] 5.4 新README.md末尾にABOUT.mdへのリンクを追加
  - セクション「翻訳につぁE��」を追加
  - 冁E��: `翻訳につぁE��の詳細は[ABOUT.md](ABOUT.md)を参照してください。`
  - _Requirements: 4.6_

- [x] 5.5 index.md削除確誁E
  - index.mdファイルが存在しなぁE��とを確誁E
  - README.mdとABOUT.mdが正しく配置されてぁE��ことを確誁E
  - _Requirements: 4.3_

- [x] 5.6 全ファイルのナビゲーションリンクがREADME.mdを指すことを確誁E
  - 全章ファイル�E�E1-09�E��Eナビリンクを確誁E
  - GLOSSARY.md、LICENSE.mdのナビリンクを確誁E
  - 全て`./README.md`を指してぁE��ことを検証
  - GitHubプレビューで目次からの遷移を確誁E
  - _Requirements: 4.5, 5.1, 7.4_

## バリチE�Eション

- [x] 6. 最終検証
  - Phase 1完亁E 吁E��のナビリンク動作確誁E
  - Phase 2完亁E 6章セクション数=11確誁E
  - Phase 3完亁E リンク全件検証
  - Phase 4完亁E メタチE�Eタ形式統一確誁E
  - Phase 5完亁E README.md表示確認、�Eナビリンク動作確誁E
  - _Requirements: 7.1, 7.2, 7.3, 7.4_
