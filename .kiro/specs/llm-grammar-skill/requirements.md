# Requirements Document

## Introduction

LLMがPasta DSL文法を正確に認識・生成し、`pasta_shiori` を用いたゴースト（「伺か」デスクトップマスコット）を自律的に作成できるようにするためのコーディングスキル（VS Code Copilot Skill形式）を開発する。

既存の `steering/grammar.md`（AI向け完全参照）および `doc/spec/`（権威的仕様書）を情報ソースとし、LLMがゴースト作成タスクを実行する際に必要な文法知識・プロジェクト構造・テンプレートパターンを、スキルファイル内に転記・体系化する。

**配置と使用形態**: スキルフォルダは**別リポジトリのゴーストディレクトリにコピーして使用する**ことを前提とする。そのため、スキルファイルはpastaリポジトリ内の他ドキュメントへの参照に依存せず、必要な情報をすべてスキルフォルダ内に自己完結的に内包しなければならない。配置先は `.agents/skills/pasta-ghost-authoring/` とし、VS Code GitHub Copilot の skill 機構により自動的にLLMのコンテキストへ注入される。

## Project Description (Input)
LLMがpasta_shioriを使ってゴーストを作るための文法認識用スキルを仕様フォルダに製作する。

## Requirements

### Requirement 1: スキルファイル構造の定義

**Objective:** LLMコーディングエージェントとして、Pasta DSLによるゴースト作成スキルが標準的なVS Code Copilot Skill形式（SKILL.md）で定義されていること により、GitHub Copilotが適切なタイミングでスキルを呼び出せるようにしたい。

#### Acceptance Criteria
1. The スキル shall `.agents/skills/pasta-ghost-authoring/SKILL.md` にファイルを配置する
2. The SKILL.md shall YAML Frontmatter形式で `name`, `description`（USE FOR / DO NOT USE FOR トリガーフレーズを含む）等のスキルメタデータを定義する
3. When LLMがゴースト作成・Pasta DSL記述・辞書ファイル追加に関する質問を受けた場合, the スキル shall 自動的にコンテキストとして提供される
4. The SKILL.md shall スキルの目的・対象ドメイン・前提条件を冒頭に明記する
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
7. The スキル shall アクター辞書（`％アクター名`）の定義と用途を説明する
8. The スキル shall さくらスクリプト（`\s[ID]`, `\n`, `\w数字`, `\_w[数字]`）の基本タグを説明する
9. The スキル shall Luaコードブロック（` ```lua ``` `）の記述方法と制約を説明する
10. The スキル shall コメント（`＃`）、属性（`＆属性名：値`）を説明する

### Requirement 3: ゴーストプロジェクト構造テンプレートの提供

**Objective:** LLMとして、ゴーストの正しいディレクトリ構成と設定ファイルを生成するために、プロジェクト構造のテンプレートがスキルに含まれていること により、新規ゴースト作成時に必要なファイルを漏れなく生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall `ghost/master/` 配下の標準ディレクトリ構成（`dic/`, `scripts/`, `profile/`）を定義する
2. The スキル shall `pasta.toml` の必須セクション（`[package]`, `[loader]`）と推奨セクション（`[logging]`, `[ghost]`, `[talk]`）のテンプレートを提供する
3. The スキル shall `descript.txt` の必須フィールド（`charset`, `type`, `name`, `sakura.name`, `kero.name`, `shiori`）のテンプレートを提供する
4. The スキル shall `dic/*.pasta` パターンによるスクリプト自動読み込みの仕組みを説明する

### Requirement 4: ゴースト作成パターン集の提供

**Objective:** LLMとして、典型的なゴースト会話シナリオを正確に記述するために、実証済みのパターン集が提供されること により、ゼロからコードを組み立てなくても定型パターンに基づいた高品質なコードを生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall アクター辞書定義パターン（`％アクター名` + `＠表情：\s[ID]` の組み合わせ）の例を提供する
2. The スキル shall イベントハンドラパターン（`＊OnBoot`, `＊OnFirstBoot`, `＊OnClose`, `＊OnMouseDoubleClick`, `＊OnTalk`, `＊OnHour`）の例を提供する
3. The スキル shall ランダムトーク（同名シーン複数定義）パターンの例を提供する
4. The スキル shall 単語によるランダム選択（`＠雑談：値1、値2、値3`）の使用例を提供する
5. The スキル shall ファイル分割の推奨構成（`actors.pasta`, `boot.pasta`, `talk.pasta`, `click.pasta` 等）を説明する

### Requirement 5: SHIORIイベントマッピングの説明

**Objective:** LLMとして、ゴーストが受け取るSHIORIイベントとPasta DSLシーン名の対応関係を理解するために、イベントディスパッチ機構の説明がスキルに含まれていること により、正しいイベントハンドラシーンを作成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall シーン関数フォールバック（シーン名 = SHIORIイベント名で自動ディスパッチ）の仕組みを説明する
2. The スキル shall 主要SHIORIイベント（OnBoot, OnFirstBoot, OnClose, OnMouseDoubleClick, OnSecondChange）とそのReference引数の概要を説明する
3. The スキル shall 仮想イベント（OnTalk, OnHour）のディスパッチ機構（OnSecondChange経由）を説明する
4. The スキル shall スコープ指定（`％アクター名1、アクター名2`）によるバルーン連動の仕組みを説明する

### Requirement 6: 既存ドキュメントとの整合性

**Objective:** プロジェクトメンテナーとして、スキルに転記された内容が権威的ドキュメント（`doc/spec/`, `steering/grammar.md`, `GRAMMAR.md`）と矛盾しないこと により、ドキュメントの信頼性を維持したい。

#### Acceptance Criteria
1. The スキル shall 作成時に `doc/spec/` を権威的ソースとして情報を転記し、独自の仕様解釈を含まない
2. The スキル shall 文法記述が `steering/grammar.md` のマーカー一覧・基本パターンと一致する
3. The スキル shall `GRAMMAR.md`（人間向け学習資料）との役割分離を明確にし、スキルはLLMのコード生成に特化する
4. If スキルの内容と `doc/spec/` の間に不一致が検出された場合, the 開発者 shall `doc/spec/` を優先してスキルを修正する
