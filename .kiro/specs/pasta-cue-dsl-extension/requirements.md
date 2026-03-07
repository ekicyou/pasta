# 要件定義書: pasta-cue-dsl-extension

> **バージョン**: v3（2026-03-07 — 議題1回答を反映: `@alias` は既存 `@word_ref` で充足、AST 変更不要）
> **親仕様**: `.kiro/specs/pasta-cue-dsl-extension/dola-cue-pasta-dsl-extension/`
> **スコープ**: pasta_core の構文解析と構造的 AST 出力のみ。意味解釈・CueIR・CueSheet 構築は dola 側の責務。

## イントロダクション

本仕様は、pasta_core の既存 PEG 文法と AST に Cue 拡張構文を追加するための要件を定義する。

Option C' 設計方針に基づき、pasta_core は**構文解析と構造的 AST 出力のみ**を担当する。

```
.pasta ファイル → pasta_core（PEG文法拡張・構造的AST出力・意味解釈なし）→ dola（AST→CueIR変換・CueSheet構築）
```

### 拡張の原則

親仕様の要求は **DSL の拡張点確保**である。既存の文法で既にパースできる箇所（`@word_ref`、`%` 行、継続行等）は変更しない。新規追加は `!` キューコマンド行のみ。

追加する構文要素:
- **キューコマンド行** (`!` / `！`): 12 種類のコマンドを構文解析し CueCommandNode として AST に保持（**新規追加**）

既存構文で充足する要素:
- **`@alias` 参照**: 既存の `Action::WordRef` がそのまま充足。dola が Cue 拡張シーンで意味解釈する
- **スロット指定行** (`%`): 既存の `scene_actors_line` / `SceneActorItem` がそのまま充足
- **継続行**: 既存の `ContinueAction` がそのまま充足。`\n` 結合は dola 側の責務

### 実装完了の定義

1. `!` 行を `local_scene_item` に追加してパースできる
2. シーン内に `!` 行が 1 つ以上あることを検出できる
3. `!command@name(args)` / `!command(args)` / `!mark@name` を AST に保持できる
4. `!` 行がないシーンでは既存 pasta の挙動を変えない

### 責務境界（pasta_core がやらないこと）

以下は dola 側の責務であり、pasta_core の実装スコープ外である:
- CueIR の定義・保持
- dola ドメイン型（ActorKey, CueCommand, CueTarget, EntityKey 等）への依存
- start_time の計算・Duration Resolver の呼び出し
- mark / seek / alias / routing の意味解釈
- RouteAdd 自動生成
- mark 重複・未登録参照・名前空間衝突の検出
- `@word_ref` のエイリアス解決（dola が Cue 拡張シーンで解釈)
- 継続行の `\n` 結合（dola が ContinueAction を結合処理）
- 継続行内 `@word_ref` の制約（dola が Cue 拡張シーンで検証）

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

### Requirement 3: AST モデルの拡張

**Objective:** As a パーサー利用者（dola / LSP）, I want Cue 拡張に対応した構造的な AST ノードにアクセスできること, so that 構文解析結果を型安全に利用して下流処理やエディタ支援を実装できる

> **親仕様参照**: handoff 文書 §7「AST に保持すべき情報」

#### Acceptance Criteria

1. The pasta_dsl shall CueCommandNode 型を提供し、コマンド種別（String）・オプショナルな ScopedName・引数リスト（Vec<CueArgToken>）・Span をフィールドとして保持する
2. The pasta_dsl shall CueArgToken 列挙型を提供し、以下の 6 バリアントを表現できるようにする: Ident(String), StringLiteral(String), FloatLiteral(String), AtRef(String), JsonObject(String), EntityKey(String)
3. The pasta_dsl shall ScopedName 型を提供し、`@name` / `@actor:name` 形式のスコープ付き識別子を表現する（actor: Option<String>, name: String）
4. The pasta_dsl shall LocalSceneItem に CueCommand バリアントを追加する

### Requirement 4: 構文エラー報告

**Objective:** As a DSL利用者, I want キューコマンド構文に誤りがある場合に明確なエラーメッセージを受け取ること, so that スクリプトの誤りを迅速に修正できる

> **親仕様参照**: design.md「エラーハンドリング」セクション CueParseError

#### Acceptance Criteria

1. If 定義されていないキューコマンド名が `!` 行に記述された場合, then the pasta_dsl shall 未知のキューコマンドであることをエラーとして報告する

> **PEG 文法による自動保証**: 負の浮動小数点リテラルは `float_lit = { ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }` により文法レベルで排除される。名前付き定義の構文不正は各コマンドの PEG ルール（`cue_emote_def` 等）により文法レベルで検出される。

### Requirement 5: 後方互換性の維持

**Objective:** As a 既存 pasta スクリプト利用者, I want キューコマンドを使用しない既存スクリプトが従来通り動作すること, so that 既存資産を壊すことなく新機能を利用開始できる

> **親仕様参照**: 親仕様 要件 6「後方互換性」

#### Acceptance Criteria

1. While シーン内に `!` 行が含まれない場合, the pasta_dsl shall そのシーンの構文解析結果を Cue 拡張導入前と同一にする
2. The pasta_dsl shall 既存の全テストがリグレッションなくパスすることを保証する

---

## 設計判断事項（設計フェーズで解決）

| ID | 項目 | 概要 |
|----|------|------|
| D1 | Cue AST 型のファイル配置 | `ast/cue.rs` 新設 vs 既存ファイルへの追加 |
| D2 | `string_literal` / `json_object` の PEG ルール | choice/custom コマンドの引数パースに必要。既存文法との対応を設計 |

## 確定済み設計判断

| ID | 項目 | 決定 |
|----|------|------|
| D3 | `cue_cmd_line` の配置スコープ | `local_scene_item` のみ。`global_scene_init` には追加しない。アクション行と同じくくり |
| D4 | `@alias` と `@word_ref` の関係 | 既存 `Action::WordRef` をそのまま利用。ActionFragment / AliasRef の新設不要 |
| D5 | ActionLine の構造変更 | 不要。既存 `actions: Vec<Action>` を維持 |
| D6 | 継続行の `\n` 結合 | pasta_core は `ContinueAction` を独立 AST ノードとして維持。結合は dola 側 |

---

## 既存構文で充足する要素（変更不要の確認）

| 要素 | 既存 AST 型 | 理由 |
|------|-----------|------|
| `@alias` 参照 | `Action::WordRef` | 既存の `@word_ref` 文法で既にパース済み。dola が Cue シーンで意味解釈 |
| `%` スロット指定 | `SceneActorItem` | 既存の `scene_actors_line` で name/number/span を保持済み |
| 継続行 | `ContinueAction` | 既存の `continue_action_line` で独立 AST ノードとして保持済み。`\n` 結合は dola 側 |

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
