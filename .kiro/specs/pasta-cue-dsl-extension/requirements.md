# 要件定義書: pasta-cue-dsl-extension

> **バージョン**: v2（2026-03-07 — 親仕様 dola-cue-pasta-dsl-extension に基づき全面再構成）
> **親仕様**: `.kiro/specs/pasta-cue-dsl-extension/dola-cue-pasta-dsl-extension/`
> **スコープ**: pasta_core の構文解析と構造的 AST 出力のみ。意味解釈・CueIR・CueSheet 構築は dola 側の責務。

## イントロダクション

本仕様は、pasta_core の既存 PEG 文法と AST に Cue 拡張構文を追加するための要件を定義する。

Option C' 設計方針に基づき、pasta_core は**構文解析と構造的 AST 出力のみ**を担当する。

```
.pasta ファイル → pasta_core（PEG文法拡張・構造的AST出力・意味解釈なし）→ dola（AST→CueIR変換・CueSheet構築）
```

追加する構文要素は以下の 3 種類である:
- **キューコマンド行** (`!` / `！`): 12 種類のコマンドを構文解析し CueCommandNode として AST に保持
- **アクション行内 `@alias` 参照**: テキスト断片と分離して ActionFragment として AST に保持
- **スロット指定行** (`%`): 既存の `scene_actors_line` / `SceneActorItem` がそのまま充足（新規実装不要）

### 実装完了の定義

1. `!` 行を `local_scene_item` に追加してパースできる
2. シーン内に `!` 行が 1 つ以上あることを検出できる
3. `!command@name(args)` / `!command(args)` / `!mark@name` を AST に保持できる
4. `actor:content` 内の `@alias` を Text と分離して保持できる
5. 継続行 `:content` を直前アクション行の Text に `\n` 結合できる
6. 継続行内 `@alias` を構文エラーとして報告できる
7. `%actor、actor=2` 形式を既存 SceneActorItem で AST に保持できる（既存動作で充足）
8. `!` 行がないシーンでは既存 pasta の挙動を変えない

### 責務境界（pasta_core がやらないこと）

以下は dola 側の責務であり、pasta_core の実装スコープ外である:
- CueIR の定義・保持
- dola ドメイン型（ActorKey, CueCommand, CueTarget, EntityKey 等）への依存
- start_time の計算・Duration Resolver の呼び出し
- mark / seek / alias / routing の意味解釈
- RouteAdd 自動生成
- mark 重複・未登録参照・名前空間衝突の検出

---

## 要件

### Requirement 1: キューコマンド行の構文解析

**Objective:** As a DSL利用者, I want `!` / `！` で始まるキューコマンド行を pasta スクリプト内に記述できること, so that シーン内で演出指示・マーク・選択肢などのキュー情報を宣言的に表現できる

> **PEG 仕様参照**: 親仕様 design.md「pasta_core 層: PEG 文法拡張」セクション

#### Acceptance Criteria

1. When `!` または `！` で始まる行がシーン内に記述された場合, the pasta_dsl shall `local_scene_item` の一種としてキューコマンド行を構文解析し CueCommandNode を生成する
2. When `!mark@name` 形式のキューコマンドが記述された場合, the pasta_dsl shall コマンド種別 `mark` と ScopedName を持つ CueCommandNode を AST に保持する
3. When `!command@name(args)` 形式のキューコマンドが記述された場合, the pasta_dsl shall コマンド種別・ScopedName・引数トークン列を持つ CueCommandNode を AST に保持する
4. When `!command(args)` 形式（マーカーなし）のキューコマンドが記述された場合, the pasta_dsl shall コマンド種別と引数トークン列を持つ CueCommandNode を AST に保持する
5. The pasta_dsl shall 以下の 12 種類のキューコマンドを認識する: `mark`, `emote`, `choice`, `custom`, `seek`, `yield`, `select`, `wait`, `clear`, `route_add`, `route_switch`, `route_remove`
6. The pasta_dsl shall 以下の 4 コマンドについて日本語エイリアスを等価に認識する: `emote`=`表情`, `choice`=`選択肢`, `custom`=`演出`, `select`=`選択待ち`
7. The pasta_dsl shall `partial.rs` の `infer_rule_from_line` に `!` / `！` 行の行種推定を追加する

### Requirement 2: シーン内キューコマンド行の検出

**Objective:** As a 下流処理系（dola）, I want シーン内にキューコマンド行が存在するかどうかを判定できること, so that キュー処理の有無をシーン単位で効率的に判断できる

#### Acceptance Criteria

1. When シーン内に 1 つ以上の `!` 行が含まれる場合, the pasta_dsl shall そのシーンにキューコマンドが存在することを AST レベルで検出可能にする
2. When シーン内に `!` 行が 1 つも含まれない場合, the pasta_dsl shall そのシーンにキューコマンドが存在しないことを AST レベルで判定可能にする

### Requirement 3: アクション行内 `@alias` 参照の分離

**Objective:** As a DSL利用者, I want アクション行 (`actor:content`) 内で `@alias` を使用してキュー参照先を記述できること, so that テキスト内に埋め込まれたキュー参照をパーサーが構造的に識別できる

> **親仕様参照**: 親仕様 要件 4「アクション行の CueCommand マッピング」AC 2, 5

#### Acceptance Criteria

1. When アクション行の content 部分に `@alias` が含まれる場合, the pasta_dsl shall `@alias` を Text 断片から分離し、ActionFragment::AliasRef として AST に保持する
2. When アクション行の content 部分に `@alias` とテキストが混在する場合, the pasta_dsl shall 出現順に ActionFragment::Text と ActionFragment::AliasRef を交互に保持する
3. When アクション行の content 部分に `@alias` が含まれない場合, the pasta_dsl shall 従来通り ActionFragment::Text のみを保持する

### Requirement 4: 継続行の結合と `@alias` 制約

**Objective:** As a DSL利用者, I want 継続行 (`:content`) が直前アクション行のテキストに自然に結合されること, so that 複数行にわたるセリフを自然に記述できる

> **親仕様参照**: 親仕様 要件 4 AC 3-4、CONTINUATION.md Q3 確定事項

#### Acceptance Criteria

1. When アクション行の直後に継続行 (`:content`) が記述された場合, the pasta_dsl shall 継続行のテキストを直前アクション行の Text に `\n` を挟んで結合する
2. If 継続行 (`:content`) 内に `@alias` が含まれている場合, then the pasta_dsl shall 構文エラーとして報告する
3. When 複数の継続行が連続する場合, the pasta_dsl shall 各継続行を `\n` 区切りで順次結合する

### Requirement 5: AST モデルの拡張

**Objective:** As a パーサー利用者（dola / LSP）, I want Cue 拡張に対応した構造的な AST ノードにアクセスできること, so that 構文解析結果を型安全に利用して下流処理やエディタ支援を実装できる

> **親仕様参照**: handoff 文書 §7「AST に保持すべき情報」

#### Acceptance Criteria

1. The pasta_dsl shall CueCommandNode 型を提供し、コマンド種別（String）・オプショナルな ScopedName・引数リスト（Vec<CueArgToken>）・Span をフィールドとして保持する
2. The pasta_dsl shall CueArgToken 列挙型を提供し、以下の 6 バリアントを表現できるようにする: Ident(String), StringLiteral(String), FloatLiteral(String), AtRef(String), JsonObject(String), EntityKey(String)
3. The pasta_dsl shall ScopedName 型を提供し、`@name` / `@actor:name` 形式のスコープ付き識別子を表現する（actor: Option<String>, name: String）
4. The pasta_dsl shall ActionLine 型を拡張し、ActionFragment のリストとして content を保持する（設計判断: Vec<Action> からの移行戦略は設計フェーズで決定）
5. The pasta_dsl shall ActionFragment 列挙型を提供し、Text（テキスト断片）と AliasRef（`@alias` 参照）を区別可能にする
6. The pasta_dsl shall スロット指定については既存の SceneActorItem 型（name: String, number: u32, span: Span）を流用する（新規型の追加は不要、既存動作で充足）

### Requirement 6: 構文エラー報告

**Objective:** As a DSL利用者, I want キューコマンド構文に誤りがある場合に明確なエラーメッセージを受け取ること, so that スクリプトの誤りを迅速に修正できる

> **親仕様参照**: design.md「エラーハンドリング」セクション CueParseError

#### Acceptance Criteria

1. If 定義されていないキューコマンド名が `!` 行に記述された場合, then the pasta_dsl shall 未知のキューコマンドであることをエラーとして報告する
2. If 継続行内に `@alias` が記述された場合, then the pasta_dsl shall 継続行内での `@alias` 使用は許可されないことをエラーとして報告する

> **PEG 文法による自動保証**: 負の浮動小数点リテラルは `float_lit = { ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }` により文法レベルで排除される。不正なスロット番号は既存の `actors_item` 文法が `digit_id` で処理する。名前付き定義の構文不正は各コマンドの PEG ルール（`cue_emote_def` 等）により文法レベルで検出される。

### Requirement 7: 後方互換性の維持

**Objective:** As a 既存 pasta スクリプト利用者, I want キューコマンドを使用しない既存スクリプトが従来通り動作すること, so that 既存資産を壊すことなく新機能を利用開始できる

> **親仕様参照**: 親仕様 要件 6「後方互換性」

#### Acceptance Criteria

1. While シーン内に `!` 行が含まれない場合, the pasta_dsl shall そのシーンの構文解析結果を Cue 拡張導入前と意味的に等価にする（ActionFragment ラッピング等の構造的差異は許容する）
2. The pasta_dsl shall 既存の全テストがリグレッションなくパスすることを保証する

---

## 設計判断事項（設計フェーズで解決）

| ID | 項目 | 概要 |
|----|------|------|
| D1 | `@alias` と `@word_ref` の分離方法 | 既存の `word_ref` ルールと `@alias` は文法的に同一（`@id`）。文法レベルで分けるか、パーサー層で後処理するかの設計判断 |
| D2 | ActionLine の移行戦略 | `actions: Vec<Action>` → `Vec<ActionFragment>` への移行方法と下流クレート（pasta_lua, pasta_lsp）への波及管理 |
| D3 | Cue AST 型のファイル配置 | `ast/cue.rs` 新設 vs 既存ファイルへの追加 |
| D4 | `string_literal` / `json_object` の PEG ルール | choice/custom コマンドの引数パースに必要。既存文法との対応を設計 |
| D5 | `cue_cmd_line` の配置スコープ | `local_scene_item` のみか、`global_scene_init` にも追加するか |

---

## 確認用サンプル

以下のサンプルがパースできることを最低確認ラインとする（親仕様 cue.pasta シーン1「起動挨拶」相当）:

```pasta
＊起動挨拶

    %さくら

    !emote@普通(normal)
    !emote@笑顔(smile)
    !choice@はい(yes, 「はい、行きましょう！」)

    さくら：@普通
    さくら：こんにちは！
    ：今日はいい天気ですね。

    !mark@挨拶後

    さくら：@笑顔
    さくら：お散歩でも行きませんか？

    !yield(10.0)
    !clear
    さくら：@はい
    !select(30.0)
```
