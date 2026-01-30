# Requirements Document

## Introduction
lua55-manualドキュメント（Lua 5.5リファレンスマニュアル日本語版）の整合性を修正し、全ページで統一された構造・スタイルを確保する。ファイルサイズが大きいため、タスクを細分化し頻繁に進捗確認を行う。

## Project Description (Input)
lua55-manualのドキュメント整合性を修正する。具体的には：
- 各ドキュメントの整合性がドキュメントごとに異なる問題を解決
- index.mdページの内容をREADME.mdに移行
- index.mdのリンク切れを修正
- 特に6章の各サブタイトルが重複している問題を修正（ファイルjoin時の影響）
- ヘッダ部（ナビゲーションリンク）の有無がページによって異なる問題を統一

ファイルサイズが大きいため、タスクを細分化し、頻繁に進捗を確認しながら段階的に実施する。

## Requirements

### Requirement 1: ドキュメントヘッダー統一（パターン2: 詳細版ナビゲーション）
**Objective:** ドキュメント管理者として、全ページで統一されたヘッダー構造を実現したい。これにより、ユーザーがドキュメント間をスムーズにナビゲートできるようになる。

#### Acceptance Criteria
1. When ドキュメントを開いたとき, the lua55-manual shall 章タイトル（# N – タイトル）を最初の見出しとして表示する
2. When ナビゲーションが必要なとき, the lua55-manual shall 「[← 前へ: N-1 – タイトル](前ファイル) | [目次](./README.md) | [次へ: N+1 – タイトル →](次ファイル)」形式（パターン2: 章番号とタイトルを含む詳細版）のナビゲーションリンクを含む
3. When 最初の章（1章）の場合, the lua55-manual shall 「[← 目次](./README.md) | [次へ: 2 – 基本概念 →](次ファイル)」形式を使用する
4. When 最後の章（9章）の場合, the lua55-manual shall 「[← 前へ: 8 – 非互換性 →](前ファイル) | [目次](./README.md)」形式を使用する
5. The lua55-manual shall ナビゲーションリンクと本文の間に「---」セパレータを配置する
6. The lua55-manual shall 01章のナビゲーションをパターン2形式に修正する（現在は簡易版）

### Requirement 2: メタデータコメント統一（HTMLコメント形式）
**Objective:** ドキュメント管理者として、全ページで統一されたHTMLコメント形式のメタデータを確保したい。これにより翻訳の出典と品質管理が容易になり、HTML変換時には非表示となる。

#### Acceptance Criteria
1. The lua55-manual shall 各ドキュメントの冒頭にHTMLコメント形式でメタデータを含める
2. When メタデータを記載するとき, the lua55-manual shall 「原文URL、参考URL、翻訳日、レビュー情報」を含める
3. If blockquote形式のメタデータが存在する場合, the lua55-manual shall HTMLコメント形式に変換する
4. The lua55-manual shall 用語対照表が必要な場合はメタデータ内に簡潔に含めるか、GLOSSARY.mdへのリンクを記載する

### Requirement 3: 6章セクション番号重複修正
**Objective:** ドキュメント閲覧者として、6章の構造が論理的で読みやすいことを期待する。現在は「6.2 – 基本関数（パートA）」「6.2 – 基本関数（パートB）」のように同じセクション番号が繰り返されており、これを修正する。

#### Acceptance Criteria
1. When 6章を閲覧するとき, the lua55-manual shall 各セクション番号が一意であり重複しない
2. If 現在のパート分割が「6.2（パートA〜E）」のように存在する場合, the lua55-manual shall 単一の「6.2 – 基本関数」見出しの下に全関数をまとめる
3. The lua55-manual shall 6章の全サブセクション（6.1〜6.11）を、パート分割なしの連続した構造にする
4. When index.mdのリンクを確認するとき, the lua55-manual shall 6章の各セクションへのアンカーリンクが正しく機能する

### Requirement 4: README.mdを目次とする役割整理
**Objective:** ドキュメント利用者として、README.mdが完全な目次であることを期待する。現在README.mdとindex.mdの役割が曖昧であり、README.mdを目次として確立する。

#### Acceptance Criteria
1. The lua55-manual shall README.mdをドキュメントのメインエントリーポイント兼完全な目次として使用する
2. When README.mdを開いたとき, the lua55-manual shall 概要、詳細目次（全セクションへのリンク付き）、付録リンクを含む
3. When ナビゲーションリンクで「目次」を参照するとき, the lua55-manual shall 一貫してREADME.mdを指す
4. If index.mdを関数・型索引専用にする場合, the lua55-manual shall ファイル名を `API_INDEX.md` などに変更し、README.mdからリンクする
5. The lua55-manual shall README.md一本化（index.md統合）も許容する

### Requirement 5: 目次・索引のリンク検証
**Objective:** ドキュメント利用者として、目次および索引のすべてのリンクが機能することを期待する。

#### Acceptance Criteria
1. When README.mdの目次リンクをクリックしたとき, the lua55-manual shall 対象ページの正しいアンカー位置に移動する
2. If アンカーID（例: `#62--基本関数`）が変更された場合, the lua55-manual shall README.md（および存在する場合はAPI_INDEX.md）内の対応リンクも更新する
3. The lua55-manual shall 全てのLua関数リンク、C API関数リンク、型リンクが有効である

### Requirement 6: 付録ファイルの整合性
**Objective:** ドキュメント管理者として、GLOSSARY.md・LICENSE.mdなどの付録ファイルも統一された構造を持つことを確保したい。

#### Acceptance Criteria
1. The lua55-manual shall GLOSSARY.mdにナビゲーションリンク「[← 目次](./README.md)」を含める
2. The lua55-manual shall LICENSE.mdにナビゲーションリンク「[← 目次](./README.md)」を含める
3. When 付録ファイルを開いたとき, the lua55-manual shall 本文との間にセパレータ「---」を配置する

### Requirement 7: 段階的作業とバリデーション
**Objective:** 開発者として、大きなファイルを安全に編集するために、小さなタスクに分割し頻繁にバリデーションを行いたい。

#### Acceptance Criteria
1. The lua55-manual shall 各章を個別のタスクとして扱い、1章ずつ修正する
2. When 1つのファイルの修正が完了したとき, the lua55-manual shall そのファイルのリンクとアンカーを検証する
3. The lua55-manual shall 6章は特に大きいため、サブセクション単位（6.1-6.3、6.4-6.6、6.7-6.9、6.10-6.11）で段階的に修正する
4. When 全修正が完了したとき, the lua55-manual shall 全ファイル間のクロスリンクをバリデーションする
