# Requirements Document

## Introduction

pasta-ghost-authoringスキル（`.agents/skills/pasta-ghost-authoring/SKILL.md`）を、pasta-lua-skillと同じアーキテクチャ（SKILL.md＋`references/`配下の分割リファレンス）で再構成し、Pasta DSL文法の**権威スキル**に昇格させる。

**背景問題**:
- まちがい「`＠地名からおらんようなってもた‥‥。`」
- ただしい「`＠地名　からおらんようなってもた‥‥。`」

`doc/spec/06-action-line.md` §6.3に明記されている「インライン要素の区切り文字」ルール（＠単語参照の後に空白を置いて通常テキストと区切る）が、スキルのみを参照したAIでは読み取れず、誤ったコードを生成してしまった。権威文書の重要ルールがスキルに十分移植されていないことが根本原因。

**情報ソース明記の原則**: スキルに記載する情報は権威的ドキュメント（`doc/spec/`章別仕様書）を正とし、情報源をスキル内に明記する。

**重複回避の原則**: `references/`は`doc/spec/`のコピーではなく、AIによるコード生成に特化した再構成とする。同一情報の冗長な重複を避け、ルール・例・表などAIが参照しやすい形式で再構成する。

**姉妹スキルとの関係**: `pasta-lua-coding`（Luaランタイム層）が完了済み仕様`pasta-lua-skill`により`references/`付きアーキテクチャに構成されている。本仕様は同じアーキテクチャを`pasta-ghost-authoring`に適用し、DSL文法層の権威リファレンスとして再構成する。

**自己完結性**: 姉妹スキルと同様に、別リポジトリにコピーして単体で機能することを前提とする。`doc/spec/`への参照に依存せず、必要な文法ルールをスキルフォルダ内に自己完結的に内包する。

## Project Description (Input)
pasta-ghost-authoringスキルをpasta-lua-skillと同様にリファレンス付きで強化し、AIが参照するpastaグラマーの権威スキルに構成しなおす。既存の権威文書（GRAMMAR.md、doc/spec/）の記述を移植し、重複を排除する。特に＠記号後のスペース要件などの構文ルールを明確に記載する。

## Requirements

### Requirement 1: リファレンス分割アーキテクチャの導入

**Objective:** LLMコーディングエージェントとして、スキルのコンテキストウィンドウ圧迫を避けつつ必要時に詳細情報を取得できるよう、SKILL.md＋`references/`の分割構成を導入したい。

#### Acceptance Criteria
1. The スキル shall `.agents/skills/pasta-ghost-authoring/SKILL.md`（メイン）と `.agents/skills/pasta-ghost-authoring/references/`（詳細リファレンス群）の2層構造で構成する
2. The SKILL.md shall 現行の全セクション（§1〜§6）を維持しつつ、権威文書から不足している文法ルールを補完する
3. The `references/` shall `doc/spec/`の章構成に対応した分割リファレンスファイルを配置する
4. The SKILL.md shall 各セクションから対応する`references/`ファイルへの参照パスを明記し、LLMが`read_file`で詳細をロードできるようにする
5. The スキルフォルダ shall 別リポジトリにコピーして単体で機能するよう、`doc/spec/`や`GRAMMAR.md`への外部参照に依存しない

### Requirement 2: インライン要素の区切りルール明文化

**Objective:** ゴースト辞書作者として、＠単語参照・＄変数参照の後続テキストとの区切りルールをAIが正確に理解し、正しいコードを生成できるようにしたい。

#### Acceptance Criteria
1. The SKILL.md shall §3.2（Action Lines）に「インライン要素の区切り文字」サブセクションを追加し、以下の3パターンを明記する：空白区切り、最長一致（空白なし）、＠＠エスケープ
2. The SKILL.md shall 空白区切りルール（`＠単語名　テキスト` → 単語参照＋通常テキスト）を正例・誤例のペアで示す
3. The SKILL.md shall 最長一致ルール（空白なしの場合、識別子に含まれない文字が現れるまでを識別子として切り出す）を説明し、意図しない吸収の例を含める
4. The SKILL.md shall 変数参照（`＄変数名　テキスト`）にも同じ空白区切りルールが適用されることを明記する
5. When AIがアクション行内でインライン要素（＠、＄）の後に通常テキストを配置するコードを生成する場合, the スキル shall 空白区切りの必要性を判断できる十分な情報を提供する
6. The `references/` shall `doc/spec/06-action-line.md` §6.3で定義されたインライン判定ルール（左から右への走査、マーカー文字列での分岐、最長一致での切り出し）をAI向けに再構成して収録する

### Requirement 3: 権威文書からの文法ルール移植

**Objective:** LLMとして、`doc/spec/`（01〜11章）に記載されている文法ルールをAI向けに再構成し、スキルのみの参照で正確なPasta DSLコードを生成できるようにしたい。

#### Acceptance Criteria
1. The `references/` shall `doc/spec/06-action-line.md`のアクション行仕様（基本構文、インライン要素一覧表、行継続、改行セマンティクス）をAI向けに再構成して収録する
2. The `references/` shall `doc/spec/10-words.md`の単語定義仕様（グローバル/ローカル/複数キー、単語参照、動的単語参照、スコープ解決ルール）をAI向けに再構成して収録する
3. The `references/` shall `doc/spec/09-variables.md`の変数仕様（スコープ、代入構文、式サポート）をAI向けに再構成して収録する
4. The `references/` shall `doc/spec/04-call-spec.md`のCall仕様（スコープ解決アルゴリズム、前方一致検索、動的ターゲット）をAI向けに再構成して収録する
5. The `references/` shall `doc/spec/11-actor-dictionary.md`のアクター辞書仕様（スコープ指定、フォールバック検索、バルーン連動）をAI向けに再構成して収録する
6. The `references/` shall `doc/spec/07-sakura-script.md`のさくらスクリプト仕様（タグ一覧、透過ルール）をAI向けに再構成して収録する
7. The `references/` shall `doc/spec/`の残りの章（01-grammar-model, 02-markers, 03-block-structure, 05-literals, 08-attributes）もコード生成に必要なルールをAI向けに再構成して収録する（ファイルグルーピングは設計フェーズで決定）
8. The `references/` shall `doc/spec/12-future.md`（未確定事項・検討中仕様）を対象外とする（コード生成には不要）
9. The `references/` shall 各リファレンスファイルの冒頭に情報源の`doc/spec/`章番号を明記する

### Requirement 4: SKILL.mdの重複排除と構造強化

**Objective:** ゴースト辞書作者として、SKILL.mdが簡潔かつ正確に保たれ、権威文書との矛盾がない状態で維持されるようにしたい。

#### Acceptance Criteria
1. The SKILL.md shall 現行の§2（Quick Reference マーカー一覧表）を維持し、各マーカーの対応する`references/`ファイルへのリンクを追加する
2. The SKILL.md shall §3（DSL Syntax）の各サブセクションに「詳細は `references/xxx.md` を参照」の導線を明記する
3. The SKILL.md shall 情報の権威フローを`doc/spec/`（権威仕様）→ `references/`（AI向け再構成）→ `SKILL.md`（要約）の一方向に整理し、同一情報の冗長な重複を避ける
4. The SKILL.md shall §6（Authoring Patterns）の辞書制作パターン集を維持する（これはスキル固有の知識であり、権威文書には含まれない）
5. If SKILL.mdと`references/`の記述に矛盾がある場合, the `references/`（権威文書のAI向け再構成）shall 正とする

### Requirement 5: 危険パターンとピットフォール集

**Objective:** ゴースト辞書作者として、AIが生成するコードで頻出する誤りパターンをスキルが事前に警告し、正しいコードが生成されるようにしたい。

#### Acceptance Criteria
1. The SKILL.md shall §3.2に「⚠️ よくある間違い」セクションを新設し、インライン要素の区切り忘れパターンを列挙する
2. The SKILL.md shall 各危険パターンに対して「❌ まちがい」「✅ ただしい」の対比形式で具体例を提示する
3. The SKILL.md shall 以下の危険パターンを最低限含む：（a）＠単語参照の後に空白なしでテキストが続く、（b）＄変数参照の後に空白なしでテキストが続く、（c）行継続で行マーカーを使ってしまう、（d）属性をアクション行の後に配置する
4. The `references/` shall `doc/spec/06-action-line.md`の「意図しない吸収の例」を含む完全な区切りルールを保持する
5. When AIがPasta DSLコードを生成する場合, the スキル shall ピットフォール集を参照して自動的に検証できる水準の明確なルール記述を提供する

### Requirement 6: GRAMMAR.mdとの役割分離の明確化

**Objective:** プロジェクトメンテナーとして、スキル内部の情報階層（`references/` → `SKILL.md`）を明確にし、自己完結性を損なわない形で役割分離を実現したい。

#### Acceptance Criteria
1. The SKILL.md shall §1（Purpose）にスキル内部の情報階層を明記する：`references/`（詳細リファレンス）と `SKILL.md`（要約＋パターン集）の2層構成、および `references/` が SKILL.md より権威であることを記述する。外部ファイル（`doc/spec/`、`GRAMMAR.md`）のパスを SKILL.md に記載しない（自己完結性: 1.5）
2. The SKILL.md shall GRAMMAR.mdへの参照を含まない（GRAMMAR.mdはAI向けではないため）
