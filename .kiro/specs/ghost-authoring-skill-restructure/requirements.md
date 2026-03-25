# Requirements Document

## Project Description (Input)
永続化変数に関する記載がスキルにない。多分「pasta-ghost-authoring」の変数のところに書くべき。「永続的に有効」としか書いていないが、永続化期間（ファイル保存される）ことがはっきりしない。また、今回追加された永続化変数の追加もお願い。pasta.tomlにbudouxに関する記載もないなあ。。。pasta.tomlはそろそろリファレンスにするべきでは。SKILL.mdが肥大化しているため、リファレンス化を含め、深掘り再構成が必要と思われる。現在必要な記載を更新せよ。

## Introduction

本仕様は、`pasta-ghost-authoring` スキル（`.agents/skills/pasta-ghost-authoring/`）のドキュメント品質向上と構造再編を定義する。

**背景**: `talk-frequency-persistence` 仕様の完了により、以下のドキュメンテーションギャップが判明した。

1. **永続化の不透明性**: グローバル変数 `＄＊変数名` を「永続的に有効」と記述しているが、ファイル保存メカニズム（JSON/gzip）・タイミング・保存先の説明が欠如
2. **新規永続化変数の未記載**: `pasta_talk_interval_min` / `pasta_talk_interval_max` が SAVE テーブルに追加されたが、スキルドキュメントに反映されていない
3. **BudouX 設定の未記載**: `[actor."名前"].budoux` 設定が pasta.toml リファレンスに存在しない
4. **pasta.toml のリファレンス不在**: pasta.toml の全設定項目が体系的にまとめられたリファレンスファイルが `references/` に存在しない
5. **SKILL.md の肥大化**: 現在の SKILL.md が約620行あり、構造的な整理が必要

**スコープ**: 本仕様は pasta-ghost-authoring スキルのドキュメント（SKILL.md + references/）の内容更新と構造改善に限定する。pasta-lua-coding スキルや Rust/Lua 実装コードの変更は含まない。

---

## Requirements

### Requirement 1: グローバル変数の永続化メカニズム説明追加

**Objective:** ゴースト辞書制作者として、グローバル変数 `＄＊変数名` の永続化（ファイル保存）の仕組みを理解したい。これにより、データが保存される条件・タイミング・場所を正しく把握した上でスクリプトを記述できる。

#### Acceptance Criteria

1. When ゴースト辞書制作者が `references/variables.md` を参照した場合, the pasta-ghost-authoring スキル shall `＄＊変数名` DSL 構文によるグローバル変数が内部的に Lua の SAVE テーブルに展開され、セッション間で JSON ファイルに永続化されることを明記する。
2. The pasta-ghost-authoring スキル shall `SKILL.md` §3.4 Variables セクションの「永続的に有効」の記述を、SAVE テーブル経由のファイル永続化であることが分かる表現に更新する。
3. The pasta-ghost-authoring スキル shall 永続化の詳細な実装メカニズム（`@pasta_persistence` モジュール、gzip 圧縮等）については `pasta-lua-coding` スキルへのクロスリファレンスを記載する（重複記述を避ける）。

### Requirement 2: SAVE テーブルのエンジン予約キー記載

**Objective:** ゴースト辞書制作者として、`＄＊変数名` 構文でアクセスできる SAVE テーブルのエンジン予約キー（`pasta_` プレフィックス）とその影響を把握したい。これにより、キー名の衝突を回避し、エンジン動作（トーク間隔など）を意図的に `＄＊変数名` 構文で制御できるようになる。

#### Acceptance Criteria

1. The pasta-ghost-authoring スキル shall `references/variables.md` に SAVE テーブルのキー命名規約（`pasta_` プレフィックスはエンジン予約）を記載する。
2. The pasta-ghost-authoring スキル shall 現在存在するエンジン予約キー `pasta_talk_interval_min` および `pasta_talk_interval_max` の用途・既定値・影響を記載する。
3. When ゴースト辞書制作者が SAVE キーに `pasta_` プレフィックスを使用しようとした場合（DSL 内で `＄＊pasta_XXX = ...` と書く場合を含む）, the ドキュメント shall エンジン動作に影響する可能性があることを警告する。

### Requirement 3: pasta.toml リファレンス新設

**Objective:** ゴースト辞書制作者として、pasta.toml の全設定項目を一覧できるリファレンスが欲しい。これにより、利用可能な設定を発見しやすくなり、ゴーストの挙動をカスタマイズできる。

#### Acceptance Criteria

1. The pasta-ghost-authoring スキル shall `references/` 配下に pasta.toml の全セクション・全キーを網羅するリファレンスファイルを新設する。
2. The リファレンスファイル shall 以下のセクションを含む: `[package]`、`[loader]`、`[logging]`、`[lua]`、`[ghost]`、`[talk]`、`[actor."名前"]`、`[persistence]`（`[lua]` と `[package]` の記載深度は設計フェーズで決定: DJ-3）。
3. The リファレンスファイル shall 各キーについて、キー名・型・既定値・説明・使用例を記載する。
4. Where `[actor."名前"]` セクションに `budoux` キーが定義されている場合, the リファレンス shall BudouX 自動改行機能の設定方法（配列形式 `[行1幅, 行2+幅]`）と動作説明を含める。
5. The pasta-ghost-authoring スキル shall `SKILL.md` §4 Project Structure の pasta.toml セクションを新設リファレンスへの参照に置き換え、SKILL.md 上には要約のみを残す。

### Requirement 4: SKILL.md のセクション構造最適化（大規模整理）

**Objective:** LLM によるスキル読み込み効率を高めるために、SKILL.md 本体の情報密度を大幅に適正化する。§4 の pasta.toml 記述に加え、§6 のオーサリングパターン集（約204行）も `references/` に分離し、SKILL.md を 300行台まで削減する。

#### Acceptance Criteria

1. The pasta-ghost-authoring スキル shall SKILL.md の各セクションが「要約 + リファレンスリンク」の形式を一貫して維持する。
2. （Req 3 AC5 に統合）§4 の pasta.toml 記述の圧縮は Req 3 AC5 に従う。
3. While SKILL.md のセクション構造を変更する場合, the pasta-ghost-authoring スキル shall 既存の §1〜§6 のセクション番号体系を維持する（外部からの参照リンクを壊さない）。
4. The pasta-ghost-authoring スキル shall §6 Authoring Patterns の詳細パターン集（約204行）を `references/authoring-patterns.md` として分離し、SKILL.md §6 は「要約 + 📖 リファレンスリンク」形式に圧縮する。
5. The pasta-ghost-authoring スキル shall 実装後の SKILL.md の総行数が 350行以内に収まることを目標とする。
6. The pasta-ghost-authoring スキル shall SKILL.md と references/ の間に記述の矛盾がないことを保証する（矛盾がある場合は references/ を正として SKILL.md を更新する）。

### Requirement 5: metadata.version のバンプ

**Objective:** スキルドキュメント変更の追跡性を確保するために、変更内容に応じた適切なバージョン更新を行いたい。

#### Acceptance Criteria

1. When 上記要件 1〜4 の変更が完了した場合, the pasta-ghost-authoring スキル shall SKILL.md フロントマター（YAML ヘッダー）の `metadata.version` を現行 `1.3.0` からマイナーバージョンアップする。
2. The バージョン番号 shall 破壊的変更（セクション削除・移動）がなければパッチ、新規リファレンス追加を含むためマイナーバンプとする。
