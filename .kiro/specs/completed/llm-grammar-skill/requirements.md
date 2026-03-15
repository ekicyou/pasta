# Requirements Document

## Introduction

ゴースト（「伺か」デスクトップマスコット）開発者の辞書制作をサポートするためのコーディングスキル（VS Code Copilot Skill形式）を開発する。

**主な使用シナリオ**: 開発者が「こんなトークを作って」「このデータを参考にトークを作成してみて」等と自然言語で指示し、LLMがPasta DSLコードを生成する。LLMはまず自然言語でネタ・構成を考え、その後Pasta DSL構文に変換する。本スキルはこの**自然言語→Pasta DSL変換**に必要な文法知識・パターン知識をLLMに提供する。

既存の `steering/grammar.md`（AI向け完全参照）および `doc/spec/`（権威的仕様書）を情報ソースとし、LLMがPasta DSLコードを正確に生成するために必要な文法知識・パターン集を、スキルファイル内に転記・体系化する。

**配置と使用形態**: スキルフォルダは**別リポジトリのゴーストディレクトリにコピーして使用する**ことを前提とする。そのため、スキルファイルはpastaリポジトリ内の他ドキュメントへの参照に依存せず、必要な情報をすべてスキルフォルダ内に自己完結的に内包しなければならない。配置先は `.agents/skills/pasta-ghost-authoring/` とし、VS Code GitHub Copilot の skill 機構により自動的にLLMのコンテキストへ注入される。

## Project Description (Input)
ゴースト開発者がPasta DSLで辞書（トーク・イベントハンドラ等）を制作する際に、LLMが自然言語→Pasta DSLコード変換をサポートするための文法認識用スキルを製作する。

## Requirements

### Requirement 1: スキルファイル構造の定義

**Objective:** LLMコーディングエージェントとして、Pasta DSLによるゴースト辞書制作サポートスキルが標準的なVS Code Copilot Skill形式（SKILL.md）で定義されていること により、GitHub Copilotが適切なタイミングでスキルを呼び出せるようにしたい。

#### Acceptance Criteria
1. The スキル shall `.agents/skills/pasta-ghost-authoring/SKILL.md` にファイルを配置する
2. The SKILL.md shall YAML Frontmatter形式で `name`, `description`（USE FOR / DO NOT USE FOR トリガーフレーズを含む）等のスキルメタデータを定義する
3. When 開発者がトーク作成・辞書ファイル記述・Pasta DSLコード生成を依頼した場合, the スキル shall 自動的にコンテキストとして提供される
4. The SKILL.md shall スキルの目的（自然言語→Pasta DSL変換サポート）・対象ドメイン・前提条件を冒頭に明記する
5. The スキルフォルダ shall 別リポジトリにコピーして単体で機能するよう、pastaリポジトリ内の他ファイルへの参照に依存せず、必要な情報をすべてスキルフォルダ内に自己完結的に内包する

### Requirement 2: Pasta DSL文法リファレンスの組み込み

**Objective:** LLMとして、ゴースト用`.pasta`ファイルを正確に記述するために必要十分なPasta DSL文法知識がスキル内に含まれていること により、外部ドキュメントを逐一参照せずに正しいコードを生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall 全マーカー一覧（`＊`, `・`, `＠`, `＄`, `＞`, `＆`, `＃`, `％`, `！` およびそれぞれの半角対応）の用途と構文を含む
2. The スキル shall シーン定義（グローバル`＊`・ローカル`・`）、重複シーンによるランダム選択、前方一致検索の概念を説明する
3. The スキル shall アクション行の構文（`アクター名：発話内容`）、アクター省略、インライン要素（`＠単語参照`、`＄変数参照`）を説明する
4. The スキル shall 単語定義（`＠単語名：値1、値2`）のグローバル/ローカルスコープと参照方法を説明する
5. The スキル shall 変数（`＄変数名`、`＄＊グローバル変数名`）のスコープと代入・参照を説明する
6. The スキル shall Call文（`＞シーン名`）および特殊Call（`＞ゴースト終了`等）の構文を説明する
7. The スキル shall アクター辞書（`％アクター名`）の定義と用途、およびスコープ指定（`％アクター名1、アクター名2`）によるバルーン連動の構文を説明する
8. The スキル shall さくらスクリプト（`\s[ID]`, `\n`, `\w数字`, `\_w[数字]`）の基本タグを説明する
9. The スキル shall Luaコードブロック（` ```lua ``` `）の記述方法と制約を説明する
10. The スキル shall コメント（`＃`）、属性（`＆属性名：値`）を説明する

### Requirement 3: ゴーストプロジェクト構造の理解

**Objective:** LLMとして、既存ゴーストプロジェクトの構造を理解し、辞書ファイルの適切な配置と設定ファイルとの関係を把握するために、プロジェクト構造の概要がスキルに含まれていること により、開発者の既存プロジェクトに適合するコードを生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall ゴースト作者が作成・編集するファイル（`dic/*.pasta`, `pasta.toml`, `descript.txt`）の役割と配置を説明する
2. The スキル shall `pasta.toml` の主要設定（`[ghost]` セクションのアクター名定義等）がDSLコード生成に影響する点を説明する
3. The スキル shall `descript.txt` の必須フィールド（`charset`, `type`, `name`, `sakura.name`, `kero.name`, `shiori`）の概要を説明する
4. The スキル shall `dic/*.pasta` パターンによるスクリプト自動読み込みの仕組みを説明する

### Requirement 4: 辞書制作パターン集の提供

**Objective:** LLMとして、開発者の自然言語による指示を正確なPasta DSLコードに変換するために、典型的な辞書制作パターン集が提供されること により、変換精度の高いコードを生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall アクター辞書定義パターン（`％アクター名` + `＠表情：\s[ID]` の組み合わせ）の例を提供する
2. The スキル shall イベントハンドラパターン（`＊OnBoot`, `＊OnFirstBoot`, `＊OnClose`, `＊OnMouseDoubleClick`, `＊OnTalk`, `＊OnHour`）の例を提供する
3. The スキル shall ランダムトーク（同名シーン複数定義）パターンの例を提供する
4. The スキル shall 単語によるランダム選択（`＠雑談：値1、値2、値3`）の使用例を提供する
5. The スキル shall ファイル分割の推奨構成（`actors.pasta`, `boot.pasta`, `talk.pasta`, `click.pasta` 等）を説明する
6. The スキル shall 自然言語の会話テーマ（例：「天気の雑談」「自己紹介」）からシーン構成・アクション行への変換の指針を示す

### Requirement 5: SHIORIイベントマッピングの説明

**Objective:** LLMとして、開発者が「起動時の挨拶を作って」「クリック反応を追加して」等と指示した際に、対応するSHIORIイベント名（シーン名）を正しく選択するために、イベントとシーン名の対応関係がスキルに含まれていること により、正しいイベントハンドラシーンを生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall シーン関数フォールバック（シーン名 = SHIORIイベント名で自動ディスパッチ）の仕組みを説明する
2. The スキル shall 主要SHIORIイベント（OnBoot, OnFirstBoot, OnClose, OnMouseDoubleClick）とその概要を説明する
3. The スキル shall 仮想イベント（OnTalk, OnHour）が内部タイマーにより自動ディスパッチされる仕組みを説明する（ゴースト作者は `＊OnTalk` / `＊OnHour` シーンを定義するだけでよい）

### Requirement 6: 既存ドキュメントとの整合性

**Objective:** プロジェクトメンテナーとして、スキルに転記された内容が権威的ドキュメント（`doc/spec/`, `steering/grammar.md`, `GRAMMAR.md`）と矛盾しないこと により、ドキュメントの信頼性を維持したい。

#### Acceptance Criteria
1. The スキル shall 作成時に `doc/spec/` を権威的ソースとして情報を転記し、独自の仕様解釈を含まない
2. The スキル shall 文法記述が `steering/grammar.md` のマーカー一覧・基本パターンと一致する
3. The スキル shall `GRAMMAR.md`（人間向け学習資料）との役割分離を明確にし、スキルはLLMのコード生成に特化する
4. If スキルの内容と `doc/spec/` の間に不一致が検出された場合, the 開発者 shall `doc/spec/` を優先してスキルを修正する
