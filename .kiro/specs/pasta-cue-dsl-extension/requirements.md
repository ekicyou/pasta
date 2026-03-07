# Requirements Document

## Project Description (Input)
# pasta エージェント向け実装引き継ぎ

> 対象: pasta / pasta_core 側の実装担当エージェント
> 目的: areka-wintf 側で確定した Cue 拡張仕様を、pasta 側で実装するための **単一・自立・圧縮済み** handoff 文書として渡す
> 前提: この文書だけで実装開始できることを目指す

---

## 1. 今回やってほしいこと

pasta_core の既存 DSL に、次の 3 要素を追加してください。

1. `!` / `！` で始まるキューコマンド行の構文解析
2. アクション行内の `@alias` を Text 断片と分離して AST に保持
3. `%` 行のスロット指定情報を既存構文のまま AST に保持

重要なのは、**pasta 側は構文解析と構造的 AST 化だけを担当する**ことです。意味解釈、時刻計算、CueSheet 構築は pasta 側の責務ではありません。

## 2. 設計の最重要前提（Option C'）

```
.pasta ファイル → pasta_core（PEG文法拡張・構造的AST出力・意味解釈なし）→ dola（AST→CueIR変換・CueSheet構築）
```

## 3. 実装完了の定義

1. `!` 行を `local_scene_item` に追加してパースできる
2. シーン内に `!` 行が 1 つ以上あることを検出できる
3. `!command@name(args)` / `!command(args)` / `!mark@name` を AST に保持できる
4. `actor:content` 内の `@alias` を Text と分離して保持できる
5. 継続行 `:content` を直前アクション行の Text に `\n` 結合できる
6. 継続行内 `@alias` を構文エラーとして報告できる
7. `%actor、actor=2` 形式を既存 SceneActorItem で AST に保持できる（既存動作で充足）
8. `!` 行がないシーンでは既存 pasta の挙動を変えない

## Introduction

本仕様は、pasta DSL に Cue 拡張構文を追加するための要件を定義する。Option C' 設計方針に基づき、pasta_dsl は構文解析と構造的 AST 出力のみを担当し、意味解釈・時刻計算・CueSheet 構築は下流（dola）の責務とする。

追加する構文要素は以下の 3 種類である:
- **キューコマンド行** (`!` / `！`): 12 種類のコマンドを構文解析し AST ノードとして保持
- **アクション行内 `@alias` 参照**: テキスト断片と分離して ActionFragment として AST に保持
- **スロット指定行** (`%`): アクター/スロット番号の指定を AST に保持

## Requirements

### Requirement 1: キューコマンド行の構文解析

**Objective:** As a DSL利用者, I want `!` / `！` で始まるキューコマンド行を pasta スクリプト内に記述できること, so that シーン内で演出指示・マーク・選択肢などのキュー情報を宣言的に表現できる

#### Acceptance Criteria

1. When `!` または `！` で始まる行がシーン内に記述された場合, the pasta_dsl shall `local_scene_item` の一種としてキューコマンド行を構文解析し CueCommandNode を生成する
2. When `!mark@name` 形式のキューコマンドが記述された場合, the pasta_dsl shall コマンド種別 `mark` と `@name` マーカーを持つ CueCommandNode を AST に保持する
3. When `!command@name(args)` 形式のキューコマンドが記述された場合, the pasta_dsl shall コマンド種別・`@name` マーカー・括弧内引数リストを持つ CueCommandNode を AST に保持する
4. When `!command(args)` 形式（マーカーなし）のキューコマンドが記述された場合, the pasta_dsl shall コマンド種別と括弧内引数リストを持つ CueCommandNode を AST に保持する
5. The pasta_dsl shall 以下の 12 種類のキューコマンドを認識する: `mark`, `emote`, `choice`, `custom`, `seek`, `yield`, `select`, `wait`, `clear`, `route_add`, `route_switch`, `route_remove`
6. The pasta_dsl shall 以下の 4 コマンドについて日本語エイリアスを認識する: `emote`=`表情`, `choice`=`選択肢`, `custom`=`演出`, `select`=`選択待ち`
7. When キューコマンドの引数に名前付き定義（`key=value`）が含まれる場合, the pasta_dsl shall 名前付き引数として CueArgToken に保持する
8. When キューコマンドの引数に位置引数（名前なしの値）が含まれる場合, the pasta_dsl shall 位置引数として CueArgToken に保持する

### Requirement 2: シーン内キューコマンド行の検出

**Objective:** As a 下流処理系（dola）, I want シーン内にキューコマンド行が存在するかどうかを判定できること, so that キュー処理の有無をシーン単位で効率的に判断できる

#### Acceptance Criteria

1. When シーン内に 1 つ以上の `!` 行が含まれる場合, the pasta_dsl shall そのシーンにキューコマンドが存在することを AST レベルで検出可能にする
2. When シーン内に `!` 行が 1 つも含まれない場合, the pasta_dsl shall そのシーンにキューコマンドが存在しないことを AST レベルで判定可能にする

### Requirement 3: アクション行内 `@alias` 参照の分離

**Objective:** As a DSL利用者, I want アクション行 (`actor:content`) 内で `@alias` を使用してキュー参照先を記述できること, so that テキスト内に埋め込まれたキュー参照をパーサーが構造的に識別できる

#### Acceptance Criteria

1. When アクション行の content 部分に `@alias` が含まれる場合, the pasta_dsl shall `@alias` を Text 断片から分離し、ActionFragment::AliasRef として AST に保持する
2. When アクション行の content 部分に `@alias` とテキストが混在する場合, the pasta_dsl shall 出現順に ActionFragment::Text と ActionFragment::AliasRef を交互に保持する
3. When アクション行の content 部分に `@alias` が含まれない場合, the pasta_dsl shall 従来通り ActionFragment::Text のみを保持する

### Requirement 4: 継続行の結合と `@alias` 制約

**Objective:** As a DSL利用者, I want 継続行 (`:content`) が直前アクション行のテキストに自然に結合されること, so that 複数行にわたるセリフを自然に記述できる

#### Acceptance Criteria

1. When アクション行の直後に継続行 (`:content`) が記述された場合, the pasta_dsl shall 継続行のテキストを直前アクション行の Text に `\n` を挟んで結合する
2. If 継続行 (`:content`) 内に `@alias` が含まれている場合, then the pasta_dsl shall 構文エラーとして報告する
3. When 複数の継続行が連続する場合, the pasta_dsl shall 各継続行を `\n` 区切りで順次結合する

### Requirement 5: スロット指定行の構文解析

**Objective:** As a DSL利用者, I want `%` 行でアクターのスロット番号を指定できること, so that キューシステムで各アクターのスロット配置を宣言的に定義できる

> **既存実装ノート**: `scene_actors_line` / `SceneActorItem`（name: String, number: u32, span: Span）が既にスロット番号付きアクター指定を完全にサポートしている。本要件は既存構文・既存 AST 型がそのまま Cue 拡張のスロット指定要件を充足することを確認するものである。新規型の追加は不要。

#### Acceptance Criteria

1. When `%actor` 形式のスロット指定行が記述された場合, the pasta_dsl shall アクター名を持つ SceneActorItem を AST に保持する（既存動作で充足）
2. When `%actor、actor=2` 形式のスロット指定行が記述された場合, the pasta_dsl shall 複数のアクター名とスロット番号の対応を SceneActorItem として AST に保持する（既存動作で充足）
3. When スロット指定行にスロット番号（`=N`）が付与されている場合, the pasta_dsl shall スロット番号を整数値として SceneActorItem.number に保持する（既存動作で充足）

### Requirement 6: AST モデルの拡張

**Objective:** As a パーサー利用者（dola / LSP）, I want Cue 拡張に対応した構造的な AST ノードにアクセスできること, so that 構文解析結果を型安全に利用して下流処理やエディタ支援を実装できる

#### Acceptance Criteria

1. The pasta_dsl shall CueCommandNode 型を提供し、コマンド種別・オプショナルなマーカー名（ScopedName）・引数リスト（Vec<CueArgToken>）をフィールドとして保持する
2. The pasta_dsl shall CueArgToken 型を提供し、位置引数と名前付き引数（key=value）の両方を表現できるようにする
3. The pasta_dsl shall ScopedName 型を提供し、`@name` 形式のマーカー参照を表現する
4. The pasta_dsl shall ActionLine 型を拡張し、content フィールドを ActionFragment のリストとして保持する
5. The pasta_dsl shall ActionFragment 列挙型を提供し、Text（テキスト断片）と AliasRef（`@alias` 参照）を区別可能にする
6. The pasta_dsl shall スロット指定については既存の SceneActorItem 型（name: String, number: u32, span: Span）を流用する（新規型の追加は不要）

### Requirement 7: 構文エラー報告

**Objective:** As a DSL利用者, I want キューコマンド構文に誤りがある場合に明確なエラーメッセージを受け取ること, so that スクリプトの誤りを迅速に修正できる

#### Acceptance Criteria

1. If 定義されていないキューコマンド名が記述された場合, then the pasta_dsl shall 未知のキューコマンドであることをエラーとして報告する
2. If キューコマンドの引数に負の浮動小数点リテラルが記述された場合, then the pasta_dsl shall 負のリテラルは許可されないことをエラーとして報告する
3. If 名前付き定義（`key=value`）の形式が不正である場合, then the pasta_dsl shall 名前付き定義の形式エラーを報告する
4. If スロット指定行のスロット番号が不正な値である場合, then the pasta_dsl shall 無効なスロット番号であることをエラーとして報告する
5. If 継続行内に `@alias` が記述された場合, then the pasta_dsl shall 継続行内での `@alias` 使用は許可されないことをエラーとして報告する

### Requirement 8: 意味解釈のスコープ外定義

**Objective:** As a 開発チーム, I want pasta_dsl が処理しない意味チェックの範囲を明確にすること, so that pasta_dsl と dola の責務境界を維持し、過剰な実装を防止できる

#### Acceptance Criteria

1. The pasta_dsl shall mark の重複検出を行わない（下流の責務とする）
2. The pasta_dsl shall 未登録の mark 参照の検証を行わない（下流の責務とする）
3. The pasta_dsl shall 名前空間の衝突検出を行わない（下流の責務とする）
4. The pasta_dsl shall ルーティング・バリアの意味的整合性検証を行わない（下流の責務とする）
5. The pasta_dsl shall CueSheet の構築や時刻計算を行わない（下流の責務とする）

### Requirement 9: 後方互換性の維持

**Objective:** As a 既存 pasta スクリプト利用者, I want キューコマンドを使用しない既存スクリプトが従来通り動作すること, so that 既存資産を壊すことなく新機能を利用開始できる

#### Acceptance Criteria

1. While シーン内に `!` 行が含まれない場合, the pasta_dsl shall そのシーンの構文解析結果を Cue 拡張導入前と意味的に等価にする（ActionFragment ラッピング等の構造的差異は許容する）
2. While 既存の `%` 行（アクター辞書定義）がスロット番号なしで記述されている場合, the pasta_dsl shall 従来と同一の構文解析結果を生成する
3. The pasta_dsl shall 既存のアクション行（`@alias` を含まないもの）の構文解析結果を変更しない
4. The pasta_dsl shall 既存の全テストがリグレッションなくパスすることを保証する
