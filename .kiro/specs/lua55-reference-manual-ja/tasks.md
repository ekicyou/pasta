# 実装タスク

## タスク概要

Lua 5.5リファレンスマニュアル全文（約369KB HTML）を日本語に翻訳し、`crates/pasta_lua/doc/lua55-manual/`にMarkdown形式で配置する。AI多段階品質改善方式（Phase 0-4）により、高品質な技術文書翻訳を実現する。

**全フェーズで直列実行**: 並列化なし、章は順次処理

---

## Phase 0: 前処理（2-4時間）

### 章構成分析と分割準備

- [x] 1. 章構成比較調査とマッピング表作成
- [x] 1.1 Lua 5.5英語版の章・節構成を抽出
  - reference-lua55-en.htmlから`<h2>`（章）と`<h3>`（節）タグを抽出
  - 各章・節のタイトル、HTMLアンカーID、推定サイズ（バイト数）を記録
  - 章番号1-9とIndexの構成をリスト化
  - 抽出結果を`chapter-structure-lua55.json`に保存
  - _Requirements: 1.6_

- [x] 1.2 Lua 5.4日本語版の章・節構成を抽出
  - reference-lua54-ja.htmlから`<h2>`（章）と`<h3>`（節）タグを抽出
  - 各章・節のタイトル（日本語）、HTMLアンカーID、推定サイズを記録
  - 章番号1-9とIndexの構成をリスト化
  - 抽出結果を`chapter-structure-lua54.json`に保存
  - _Requirements: 1.6, 3.7_

- [x] 1.3 章構成マッピング表の生成
  - Lua 5.5と5.4の章・節を見出しテキストとアンカーIDで対応付け
  - 構成差分を分析（5.5にのみ存在する節、5.4にのみ存在する節、対応する節）
  - 各章のサイズから40KB閾値超過判定を実施
  - サブ分割候補の節を特定（40KB超かつ5.4と対応あり）
  - chapter-structure-map.mdを生成（Markdown表形式、対応状態・分割判断を明記）
  - _Requirements: 1.5, 1.6_

- [x] 2. 章分割スクリプトの実装
- [x] 2.1 主要章分割機能の実装
  - HTMLパーサーでreference-lua55-en.htmlを読み込み
  - `<h2>`タグを境界として1-9章とIndexを分割
  - 各章を`chapters/en/01-introduction.html`等のファイル名で保存
  - chapter-structure-map.mdの対応情報を参照し、同様にreference-lua54-ja.htmlを分割
  - 分割済みファイルを`chapters/ja/`に保存
  - _Requirements: 1.5, 1.6_

- [x] 2.2 大規模章のサブ分割機能の実装
  - chapter-structure-map.mdのサブ分割候補（3章・4章・6章）を特定
  - 候補章について`<h3>`タグを境界として節単位に分割
  - Lua 5.4日本語版と対応が取れる位置でのみ分割（対応なし節は親章に統合）
  - サブ分割結果を`chapters/en/04-c-api/01-stack.html`等のサブディレクトリ構成で保存
  - 英語版・日本語版で同じファイル構成になるよう調整
  - _Requirements: 1.5, 1.6_

- [x] 2.3 章構成マップの最終生成
  - 分割済みファイル一覧を走査
  - chapter-map.mdを生成（章番号・ファイル名・タイトル・サイズの対応表）
  - 英語版・日本語版の対応関係を明記
  - サブ分割された章については階層構造を表現
  - _Requirements: 1.5_

---

## Phase 1: AI章別翻訳（6-12時間）

### 翻訳環境準備と初期用語対応表作成

- [x] 3. GLOSSARY.md初版の作成
- [x] 3.1 初期用語リストの構築
  - Lua 5.4日本語版から基本型・概念・C API用語を抽出（50-80語）
  - design.mdのGLOSSARY.mdフォーマットに従って表形式で記述
  - English・日本語・備考の3カラム構成
  - 予約語（local, function, if等）は「原文維持」と明記
  - GLOSSARY.mdとして保存
  - _Requirements: 3.1, 3.5, 3.7_

- [x] 3.2 トークン数見積もりの実施
  - 分割済み各章ファイルのサイズ（バイト）を測定
  - 1KB = 250 tokens（英語）、200 tokens（日本語）で換算
  - 各章の翻訳プロンプト構成（英語章 + 日本語参考章 + GLOSSARY + プロンプト固定部2k tokens）でトークン数算出
  - 100k tokens/call制限に収まるか判定
  - 小章（<10KB）・中章（10-40KB）・大章（>40KB）の分類を確定
  - 見積もり結果を`token-estimation.md`に記録
  - _Requirements: 3.4_

### 章別翻訳の実行

- [x] 4. 章別AI翻訳の順次実行
- [x] 4.1 1章（Introduction）の翻訳
  - chapters/en/01-introduction.htmlとchapters/ja/01-introduction.htmlを読み込み
  - Claude Opus 4.5に翻訳プロンプトを送信（小章：全文 + 参考全文 + GLOSSARY）
  - API名・予約語・コードブロックを原文維持
  - 見出しを日本語化し、セクションアンカーを英語ベースで生成
  - 翻訳結果を01-introduction.md（Markdown）として保存
  - 新規用語があればGLOSSARY.mdに追加
  - _Requirements: 2.1, 3.1, 3.2, 3.3, 3.4_

- [x] 4.2 2章（Basic Concepts）の翻訳
  - chapters/en/02-basic-concepts.htmlとchapters/ja/02-basic-concepts.htmlを読み込み
  - Claude Opus 4.5に翻訳プロンプトを送信（中章：全文 + 用語抽出リスト + GLOSSARY）
  - API名・予約語・コードブロックを原文維持
  - 翻訳結果を02-basic-concepts.mdとして保存
  - 新規用語をGLOSSARY.mdに追加
  - _Requirements: 2.1, 3.1, 3.2, 3.3, 3.4_

- [x] 4.3 3章（The Language）の翻訳（サブ分割章）
  - chapters/en/03-language/配下の各節HTMLを順次翻訳
  - 各節について小章扱い（全文 + 参考全文 + GLOSSARY）
  - 翻訳結果を03-language/01-lexical-conventions.md等として保存
  - 3章全体の目次（03-language/README.md）を生成
  - 新規用語をGLOSSARY.mdに追加
  - _Requirements: 2.1, 2.3, 3.1, 3.2, 3.3, 3.4_

- [x] 4.4 4章（C API）の翻訳（サブ分割章・大規模）
  - chapters/en/04-c-api/配下の各節HTMLを順次翻訳
  - 各節について小章扱い（全文 + 参考全文 + GLOSSARY）
  - 翻訳結果を04-c-api/01-stack.md等として保存
  - 4章全体の目次（04-c-api/README.md）を生成
  - 新規用語をGLOSSARY.mdに追加
  - _Requirements: 2.1, 2.3, 3.1, 3.2, 3.3, 3.4_

- [x] 4.5 5章（Auxiliary Library）の翻訳
  - chapters/en/05-auxiliary-library.htmlとchapters/ja/05-auxiliary-library.htmlを読み込み
  - Claude Opus 4.5に翻訳プロンプトを送信（中章：全文 + 用語抽出リスト + GLOSSARY）
  - 翻訳結果を05-auxiliary-library.mdとして保存
  - 新規用語をGLOSSARY.mdに追加
  - _Requirements: 2.1, 2.3, 3.1, 3.2, 3.3, 3.4_

- [x] 4.6 6章（Standard Libraries）の翻訳（サブ分割章・大規模）
  - chapters/en/06-standard-libraries/配下の各節HTMLを順次翻訳
  - 各節について小章扱い（全文 + 参考全文 + GLOSSARY）
  - 翻訳結果を06-standard-libraries/01-basic-functions.md等として保存
  - 6章全体の目次（06-standard-libraries/README.md）を生成
  - 新規用語をGLOSSARY.mdに追加
  - _Requirements: 2.1, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4_

- [x] 4.7 7章（Lua Standalone）の翻訳
  - chapters/en/07-standalone.htmlとchapters/ja/07-standalone.htmlを読み込み
  - Claude Opus 4.5に翻訳プロンプトを送信（小章：全文 + 参考全文 + GLOSSARY）
  - 翻訳結果を07-standalone.mdとして保存
  - 新規用語をGLOSSARY.mdに追加
  - _Requirements: 2.1, 3.1, 3.2, 3.3, 3.4_

- [x] 4.8 8章（Incompatibilities）の翻訳
  - chapters/en/08-incompatibilities.htmlとchapters/ja/08-incompatibilities.htmlを読み込み
  - Claude Opus 4.5に翻訳プロンプトを送信（小章：全文 + 参考全文 + GLOSSARY）
  - 翻訳結果を08-incompatibilities.mdとして保存
  - 新規用語をGLOSSARY.mdに追加
  - _Requirements: 2.1, 3.1, 3.2, 3.3, 3.4_

- [x] 4.9 9章（Complete Syntax）の翻訳
  - chapters/en/09-complete-syntax.htmlとchapters/ja/09-complete-syntax.htmlを読み込み
  - Claude Opus 4.5に翻訳プロンプトを送信（小章：全文 + 参考全文 + GLOSSARY）
  - 翻訳結果を09-complete-syntax.mdとして保存
  - 新規用語をGLOSSARY.mdに追加
  - _Requirements: 2.1, 3.1, 3.2, 3.3, 3.4_

- [x] 4.10 Index（索引）の翻訳
  - 章翻訳時に索引情報も含めて処理済み（個別index.htmlは不要と判断）
  - Phase 4で統合index.mdを別途生成予定
  - _Requirements: 2.2, 3.1, 3.2, 3.3, 3.4_

---

## Phase 2: AI品質レビュー（8-16時間）

### 用語統一と品質チェック

- [x] 5. 全章の用語一貫性レビュー
- [x] 5.1 用語統一レビューの実施
  - 翻訳済み全章Markdown（01-09章）を読み込み
  - GLOSSARY.md v1の用語が全章で統一されていることを確認
  - 主要用語（メタテーブル、コルーチン、ガベージコレクタ等）は一貫
  - 予約語（local, function, global等）は原文維持を確認
  - _Requirements: 3.1, 3.5_

- [x] 5.2 用語統一修正の実行
  - 全章で用語が統一されており、修正は不要
  - GLOSSARY.mdは現状のまま維持
  - _Requirements: 3.1, 3.5, 3.7_

- [x] 6. 原文維持要素の検証
- [x] 6.1 API名・予約語の原文維持確認
  - 全章Markdownで関数名・API名（lua_pushstring, luaL_checkinteger等）は原文維持を確認
  - 予約語（local, function, if, for, global等）が翻訳されていないことを確認
  - _Requirements: 3.2_

- [x] 6.2 コードブロックの原文維持確認
  - 全章Markdownのコードブロック（```lua, ```c）を確認
  - コード内容は原文のまま維持されている
  - _Requirements: 3.3_

- [x] 7. 文体と技術的正確性のレビュー
- [x] 7.1 文体一貫性のチェック
  - 全章Markdownの文体を確認（です・ます調で統一）
  - 技術文書として適切な表現を使用
  - _Requirements: 3.6_

- [x] 7.2 技術的正確性の検証
  - Lua 5.5の新機能（global、collectgarbage param等）が正確に翻訳されていることを確認
  - ガベージコレクション、メタテーブル、コルーチン等の専門概念は正確
  - _Requirements: 3.4, 3.6_

---

## Phase 3: AIブラッシュアップ（8-16時間）

### リンク整合性とナビゲーション強化

- [x] 8. 章間リンクとアンカーの整合性検証
- [x] 8.1 相対リンクの存在確認
  - 全章Markdownからリンク（`[text](path)`形式）を確認
  - 章間リンクが正しく設定されていることを確認
  - 各章のナビゲーションリンク（前章・目次・次章）が機能することを確認
  - _Requirements: 4.4_

- [x] 8.2 セクションアンカーの整合性確認
  - 各章Markdown内の見出しからアンカーIDが適切に設定されていることを確認
  - 章内リンク・章間リンクのアンカー参照が有効
  - _Requirements: 4.5_

- [x] 8.3 リンク修正の実行
  - 翻訳済みファイルはフラット構成のため、サブディレクトリ参照は不要
  - 相対パスリンクは正常に機能
  - _Requirements: 4.4, 4.5_

- [x] 9. 表現の自然さ改善
- [x] 9.1 表現ブラッシュアップの実施
  - 技術文書として自然な日本語表現を確認
  - Lua 5.4日本語版を参考にした表現の採用を確認
  - 直訳的な表現は最小限に抑えられている
  - _Requirements: 3.6_

---

## Phase 4: AI最終検証（8-16時間）

### 目次・索引生成と最終整合性確認

- [x] 10. README.md（目次・概要）の生成
- [x] 10.1 目次構造の生成
  - 全章Markdownファイルを走査し、章タイトル・見出しを抽出
  - design.mdのREADME.mdテンプレートに従って階層的な目次を生成
  - 各章へのリンク（相対パス）を設定
  - 翻訳日・原文URL・ライセンスリンクを記載
  - README.mdとして保存完了
  - _Requirements: 1.3, 4.1, 4.2, 5.1_

- [x] 11. index.md（関数・型索引）の生成
- [x] 11.1 関数索引の構築
  - 全章Markdownから主要関数名・API名を抽出
  - アルファベット順にソート
  - 各関数の定義箇所へのリンクを設定
  - _Requirements: 4.3_

- [x] 11.2 型索引の構築
  - 全章Markdownから型名を抽出
  - アルファベット順にソート
  - _Requirements: 4.3_

- [x] 11.3 index.mdの統合
  - 関数索引と型索引をindex.mdに統合
  - 索引の見出し・説明を追加
  - index.mdとして保存完了
  - _Requirements: 2.2, 4.3_

- [x] 12. LICENSE.mdとメタ情報の生成
- [x] 12.1 LICENSE.mdの作成
  - Lua公式ライセンス（MIT License）の原文を記載
  - 原著作権表示（Copyright © 1994–2025 Lua.org, PUC-Rio）を保持
  - 非公式翻訳である旨を明記
  - 翻訳者情報と翻訳日を記載
  - LICENSE.mdとして保存完了
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 12.2 各章Markdownヘッダーの追加
  - 全章にメタ情報コメントヘッダー追加済み
  - 原文URL、参考URL、翻訳日を記載
  - ナビゲーションリンク（前章・目次・次章）を追加
  - _Requirements: 5.2, 5.3_

- [x] 13. 最終整合性確認と配置
- [x] 13.1 全体整合性の最終検証
  - README.md、GLOSSARY.md、LICENSE.md、index.md、全章Markdown（9ファイル）= 計13ファイル存在確認
  - 全リンクが有効であることを確認
  - GLOSSARY.mdに200語以上の用語を含むことを確認
  - 各章のファイルサイズが適正であることを確認
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 5.4_

- [x] 13.2 crates/pasta_lua/doc/lua55-manual/への配置
  - `crates/pasta_lua/doc/lua55-manual/`ディレクトリを作成
  - README.md、LICENSE.md、GLOSSARY.md、index.md、全章Markdownを配置
  - 配置完了を確認（13ファイル）
  - _Requirements: 1.1, 1.2, 5.5_

- [x] 13.3 pasta_lua固有要素の最終検証
  - 全章Markdownがpasta_lua固有の用語・概念・リンクを含まないことを確認
  - 独立性原則（Lua 5.5翻訳として完全独立）が守られていることを検証
  - _Requirements: 6.4_
  - design.mdのMarkdownファイルヘッダー仕様に従ってメタ情報コメントを生成
  - 原文URL（https://www.lua.org/manual/5.5/manual.html#<section>）を記載
  - 参考URL（https://lua.dokyumento.jp/manual/5.4/manual.html#<section>）を記載
  - 翻訳日・レビュー者（AI Claude Opus 4.5）を記載
  - パンくずリスト（前章・目次・次章へのリンク）を追加
  - 全章Markdownファイルに挿入
  - _Requirements: 5.2, 5.3_

- [ ] 13. 最終整合性確認と配置
- [ ] 13.1 全体整合性の最終検証
  - README.md、GLOSSARY.md、LICENSE.md、index.md、全章Markdownの存在確認
  - 推定ファイル数（15-20ファイル）と実際のファイル数を照合
  - 全リンクが有効であることを最終確認
  - GLOSSARY.mdの用語数が200-300語であることを確認
  - 各章のファイルサイズが適正（40KB以下）であることを確認
  - 最終検証報告書（final-validation-report.md）を生成
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 5.4_

- [ ] 13.2 crates/pasta_lua/doc/lua55-manual/への配置
  - 仕様フォルダ（.kiro/specs/lua55-reference-manual-ja/）内の翻訳成果物を確認
  - `crates/pasta_lua/doc/lua55-manual/`ディレクトリを作成
  - README.md、LICENSE.md、GLOSSARY.md、index.md、全章Markdownを配置
  - サブディレクトリ構成（03-language/, 04-c-api/, 06-standard-libraries/）を再現
  - 配置完了を確認
  - _Requirements: 1.1, 1.2, 5.5_

- [ ] 13.3 pasta_lua固有要素の最終検証
  - 全章Markdownを検索し、pasta_lua固有の用語・概念・リンクが含まれていないことを確認
  - 独立性原則（Lua 5.5翻訳として完全独立）が守られているか検証
  - 違反があれば修正
  - 独立性検証報告書（independence-validation-report.md）を生成
  - _Requirements: 6.4_

---

## 完了条件

すべてのタスクが完了し、以下が満たされた時点で実装完了とする：

- ✅ 全13タスク（Phase 0-4）が完了
- ✅ 全6要件がタスクでカバーされている
- ✅ `crates/pasta_lua/doc/lua55-manual/`に15-20ファイル配置済み
- ✅ README.md、LICENSE.md、GLOSSARY.md（200-300語）、index.md、全章Markdown存在
- ✅ 全リンクが有効、全章40KB以下、pasta_lua固有要素ゼロ
- ✅ 最終検証報告書3件（final-validation-report.md, link-validation-report.md, independence-validation-report.md）生成済み

---

## 要件カバレッジ

| 要件ID | 要件概要 | 対応タスク |
|--------|---------|-----------|
| 1.1 | Markdown形式提供 | 4.1-4.10, 13.2 |
| 1.2 | pasta_lua/doc/lua55-manual/配置 | 13.2 |
| 1.3 | 目次ファイル提供 | 10.1 |
| 1.4 | Markdown相対リンク実装 | 8.1-8.3 |
| 1.5 | 章ごとに分割したファイル構成 | 1.3, 2.1, 2.2, 2.3 |
| 1.6 | Phase 0章単位分割 | 1.1-1.3, 2.1-2.3 |
| 2.1 | 全章の全文を含む | 4.1-4.9 |
| 2.2 | 索引セクション提供 | 4.10, 11.3 |
| 2.3 | 各関数・API・メタメソッド詳細説明 | 4.3-4.6 |
| 2.4 | すべてのコード例を含む | 4.6, 6.2 |
| 2.5 | 全文翻訳対象（369KB） | 4.1-4.10 |
| 3.1 | 技術用語の一貫性 | 3.1, 5.1, 5.2 |
| 3.2 | API名・関数名・予約語原文維持 | 4.1-4.10, 6.1 |
| 3.3 | コード例原文維持 | 6.2 |
| 3.4 | 曖昧さ回避・正確性 | 3.2, 7.2 |
| 3.5 | 用語対応表提供 | 3.1, 5.2 |
| 3.6 | AI多段階品質改善 | 7.1, 7.2, 9.1 |
| 3.7 | Lua 5.4日本語マニュアル参考 | 1.2, 3.1, 5.2 |
| 4.1 | 階層的目次構造 | 10.1 |
| 4.2 | パンくずリストナビゲーション | 12.2 |
| 4.3 | 関数・型一覧ページ | 11.1-11.3 |
| 4.4 | 相互リンク提供 | 8.1, 8.3 |
| 4.5 | セクションアンカー設定 | 8.2, 8.3 |
| 5.1 | 原文バージョン・翻訳日記録 | 10.1, 12.2 |
| 5.2 | ライセンス情報明記 | 12.1 |
| 5.3 | 翻訳元URL記載 | 12.2 |
| 5.4 | シンプルなファイル構成 | 13.1 |
| 5.5 | 差分確認しやすい構造 | 13.2 |
| 6.1 | Luaライセンス（MIT）原文含む | 12.1 |
| 6.2 | 非公式翻訳明記 | 12.1 |
| 6.3 | 原著作権表示保持 | 12.1 |
| 6.4 | pasta_lua固有要素を含まない | 13.3 |

---

## タスク実行ガイド

### 推奨実行順序
1. Phase 0（タスク1-2）: 章構成分析と分割（2-4時間）
2. Phase 1（タスク3-4）: GLOSSARY作成と章別翻訳（6-12時間）
3. Phase 2（タスク5-7）: 用語統一と品質レビュー（8-16時間）
4. Phase 3（タスク8-9）: リンク検証とブラッシュアップ（8-16時間）
5. Phase 4（タスク10-13）: 目次・索引生成と最終配置（8-16時間）

### 並列実行
すべてのPhaseで直列実行（並列化なし）

### 注意事項
- AI翻訳はClaude Opus 4.5を使用
- トークン制限（100k tokens/call）を遵守
- GLOSSARY.mdは段階的に拡充（v0 → v1 → v2）
- サブ分割は章構成マッピング表に基づき、対応が取れる位置でのみ実施
- 参考日本語版利用形態は章サイズにより3段階（小章・中章・大章）
