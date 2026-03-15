# Implementation Plan

- [x] 1. スキルファイル骨格と Frontmatter の作成
  - `.agents/skills/pasta-ghost-authoring/` ディレクトリを作成する
  - YAML Frontmatter (`name: pasta-ghost-authoring`、`description`、`metadata`) を記述する
  - `description` の USE FOR フレーズに日英両方のトリガー語句（`pasta`, `.pasta`, `ゴースト`, `辞書`, `トーク作成` 等）を設定する
  - `description` の DO NOT USE FOR フレーズに除外語句（`pasta料理`, `pasta_dsl crate`, `Rustクレート実装` 等）を設定する
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. §1 Purpose & Prerequisites と §2 Quick Reference の記述
- [x] 2.1 §1 Purpose & Prerequisites の記述
  - スキルの主目的「自然言語→Pasta DSL変換サポート」を冒頭に宣言する
  - 対象ユーザー（既存ゴーストプロジェクトを持つ開発者）と前提条件を記述する
  - 本スキルと `GRAMMAR.md` / `steering/grammar.md` との役割分離を注記する
  - _Requirements: 1.4, 1.5, 6.3_

- [x] 2.2 §2 Quick Reference マーカー一覧表の記述
  - 全マーカー（`＊`, `・`, `＠`, `＄`, `＞`, `＆`, `＃`, `％`, `！`）の全角・半角・用途を表形式で記述する
  - `steering/grammar.md` のマーカー一覧と一致することを確認する
  - _Requirements: 2.1, 6.2_

- [x] 3. §3 DSL Syntax 各構文サブセクションの記述
- [x] 3.1 シーン定義・アクション行・単語定義の記述
  - グローバルシーン（`＊`）とローカルシーン（`・`）の定義構文、重複シーンによるランダム選択、前方一致検索を記述する
  - アクション行の構文（`アクター名：発話内容`）、アクター省略、インライン要素（`＠ref`・`＄var`）を記述する
  - 単語定義（`＠単語名：値1、値2`）のグローバル/ローカルスコープと参照方法を記述する
  - 各サブセクションに2-3行のコード例を付与する
  - _Requirements: 2.2, 2.3, 2.4_

- [x] 3.2 変数・Call 文・コメント/属性の記述
  - ローカル変数（`＄変数名`）とグローバル変数（`＄＊変数名`）のスコープ・代入・参照を記述する
  - Call 文（`＞シーン名`）と特殊 Call（`＞ゴースト終了（ミリ秒）`）の構文を記述する
  - コメント（`＃`）と属性（`＆属性名：値`）の構文を記述する
  - _Requirements: 2.5, 2.6, 2.10_

- [x] 3.3 アクター辞書・さくらスクリプト・Lua ブロックの記述
  - アクター辞書（`％アクター名`）の定義、表情単語パターン、スコープ指定（`％名前1、名前2`）によるバルーン連動の構文を記述する
  - さくらスクリプト基本タグ（`\s[ID]`、`\n`、`\w数字`、`\_w[数字]`）を記述する
  - Lua コードブロック（` ```lua ``` `）の記述方法を最小限で記述する（辞書制作における位置付けを補足）
  - _Requirements: 2.7, 2.8, 2.9_

- [x] 4. §4 Project Structure と §5 Event Mapping の記述
- [x] 4.1 §4 プロジェクト構造の記述
  - `dic/*.pasta`・`pasta.toml`・`descript.txt` の役割と配置を記述する
  - `pasta.toml` の `[ghost]` セクション（トーク間隔）と `[actor."名前"]` がコード生成に影響する点を記述する
  - `descript.txt` の必須フィールドと `pasta_patterns = ["dic/*.pasta"]` による自動読み込みの仕組みを記述する
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 4.2 §5 SHIORI イベントマッピングの記述
  - 「`＊イベント名` のシーンを定義すれば自動実行される」という核心ルールを冒頭に1-2行で記述する
  - 主要イベント（OnBoot、OnFirstBoot、OnClose、OnMouseDoubleClick）と仮想イベント（OnTalk、OnHour）の自然言語→シーン名マッピングテーブルを記述する
  - OnTalk / OnHour が内部タイマーで自動ディスパッチされる仮想イベントであることを補足する
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 5. §6 Authoring Patterns 辞書制作パターン集の記述
- [x] 5.1 アクター辞書・イベントハンドラ・ランダムトーク・単語選択パターンの記述
  - `actors.pasta` から `％名前` + `＠表情：\s[ID]` パターンの実例を転記する
  - `boot.pasta` から OnBoot / OnFirstBoot / OnClose の最小実例を転記する
  - `talk.pasta` から OnTalk 同名複数定義パターンと `＠雑談：値1、値2、値3` 単語ランダム選択パターンを転記する
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 5.2 ファイル分割ガイドと自然言語→シーン変換指針の記述
  - `actors.pasta` / `boot.pasta` / `talk.pasta` / `click.pasta` の推奨分割構成と各ファイルの責務を記述する
  - 変換ワークフロー（テーマ理解→シーン名決定→アクター選定→アクション行構成→表情選択）を記述する
  - 骨格1パターン（`＠表情名` プレースホルダ形式・〜10行）の変換実例を追加する
  - _Requirements: 4.5, 4.6_

- [x] 6. 整合性検証と行数確認
  - 全マーカー・構文記述が `doc/spec/` と一致していることを確認し、不一致があれば修正する
  - 外部ファイルへの参照パス・リンクが含まれていないことを確認する（自己完結性）
  - 総行数を確認し、400行超過時は削減優先順位（§6 Req4.6 実例圧縮 → §3 コード例削減）に従って圧縮する
  - _Requirements: 1.5, 6.1, 6.2, 6.3, 6.4_
