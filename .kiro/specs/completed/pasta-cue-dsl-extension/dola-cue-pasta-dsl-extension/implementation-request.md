# 実装依頼書: pasta_core 向け Cue コマンド文法拡張

> **作成日**: 2026-03-07  
> **バージョン**: v2  
> **対象**: pasta プロジェクト実装者 / AI エージェント  
> **参照元仕様**: `areka-wintf/.kiro/specs/dola-cue-pasta-dsl-extension/`  
> **本書の役割**: pasta 側で実装すべき範囲だけを切り出した handoff 文書

---

## 1. この依頼書で実装してほしいこと

### 1.1 スコープ

pasta_core の既存行指向 DSL に、以下の 3 要素を追加してください。

1. `!` / `！` で始まる **キューコマンド行** の構文解析
2. アクション行内の `@alias` を **構造的フラグメント** として保持
3. `%` 行の既存構文を再利用し、スロット指定情報を AST に保持

### 1.2 この依頼書の最重要前提

今回の設計は **Option C'** です。旧版の「pasta 側が CueIR を出力する」前提は廃止されています。

#### 正しい責務分担

```text
.pasta ファイル
    ↓
pasta_core
    - PEG 文法拡張
    - 行種別判定
    - 構造的 AST 出力
    - コマンドの意味解釈はしない
    ↓
dola
    - pasta AST を解釈
    - AST → CueIR 変換
    - CueSheetBuilder で CueSheet を構築
```

#### pasta_core がやらないこと

- `CueIR` を定義しない
- `CueCommand`, `BarrierKind`, `RoutingCommand` など dola のドメイン型を直接知らない
- `start_time` を計算しない
- mark / seek / alias / routing の意味解釈をしない

#### pasta_core がやること

- `!` 行を既存 DSL と共存する新しい行種別として受理する
- `!` 行の中身を「コマンド名 + エイリアス名 + 引数トークン群」という**構造情報**で保持する
- アクション行中の `@alias` を Text 断片と分離して保持する
- シーン内に `!` 行が存在するかどうかを判定できるようにする

---

## 2. 実装ゴール

実装完了の定義は、pasta_core が以下を満たすことです。

1. `!` コマンド行を既存の `scene_line` 系列に追加してパースできる
2. シーン内に `!` 行が 1 つ以上あることを検出できる
3. `!command@name(args)` / `!command(args)` / `!mark@name` を構造的に AST へ保持できる
4. アクション行 `actor:content` 内の `@alias` を Text と分離して AST に保持できる
5. 継続行 `:content` を直前アクション行の Text に `\n` 結合できる
6. 継続行内 `@alias` を構文エラーとして報告できる
7. `%actor、actor=2` 形式の割り当て情報を AST に保持できる
8. `!` 行がないシーンでは既存 pasta 挙動を変更しない

---

## 3. 実装対象の文法仕様

### 3.1 モード判定

`&type:cuesheet` のような明示マーカーは **不要** です。

```text
シーン内に ! コマンド行が 1 つ以上存在する
  → そのシーンは cue 拡張シーンとして扱える
シーン内に ! コマンド行が存在しない
  → 既存 pasta シーンとして従来どおり処理
```

### 3.2 `scene_line` への統合順序

推奨順序は次の通りです。

1. `comment_line`
2. `attribute_line`
3. `cue_cmd_line`
4. `slot_line`
5. `action_line`
6. `continuation_line`

`cue_cmd_line` は先頭空白を親ルールが消費済みである前提で構いません。

### 3.3 キューコマンド行

```peg
cue_cmd_line = {
    cue_cmd_marker ~ SPACE* ~ cue_cmd_body ~ NEWLINE
}

cue_cmd_marker = _{ "!" | "！" }

cue_cmd_body = {
    cue_mark
    | cue_emote_def
    | cue_choice_def
    | cue_custom_def
    | cue_seek
    | cue_yield
    | cue_select
    | cue_wait
    | cue_clear
    | cue_route_add
    | cue_route_switch
    | cue_route_remove
}
```

### 3.4 コマンドキーワード

日本語エイリアスは **4 コマンドのみ** 実装対象です。

| 英語正式名 | 日本語エイリアス |
|-----------|------------------|
| `emote` | `表情` |
| `choice` | `選択肢` |
| `custom` | `演出` |
| `select` | `選択待ち` |

それ以外のコマンドは英語のみです。

| コマンド | 記法 |
|---------|------|
| mark | `!mark@name` |
| seek | `!seek(@name)` / `!seek(@name, 0.5)` |
| yield | `!yield` / `!yield(10.0)` |
| select | `!select` / `！選択待ち（30.0）` |
| wait | `!wait(2.0)` |
| clear | `!clear` |
| route_add | `!route_add(shell, actor:さくら:shell)` |
| route_switch | `!route_switch(balloon, spot:stage_balloon)` |
| route_remove | `!route_remove(balloon)` |

### 3.5 名前付き定義

```peg
cue_emote_def = {
    ("emote" | "表情") ~ at_marker ~ cue_scoped_ident
    ~ paren_open ~ SPACE* ~ cue_ident ~ SPACE* ~ paren_close
}

cue_choice_def = {
    ("choice" | "選択肢") ~ at_marker ~ cue_scoped_ident
    ~ paren_open ~ SPACE* ~ cue_ident ~ SPACE* ~ "," ~ SPACE* ~ string_literal
    ~ SPACE* ~ paren_close
}

cue_custom_def = {
    ("custom" | "演出") ~ at_marker ~ cue_scoped_ident
    ~ paren_open ~ SPACE* ~ string_literal ~ SPACE* ~ "," ~ SPACE* ~ json_object
    ~ SPACE* ~ paren_close
}
```

### 3.6 タイムライン / バリア / ルーティング

```peg
cue_mark = {
    "mark" ~ at_marker ~ cue_ident
}

cue_seek = {
    "seek" ~ paren_open ~ SPACE* ~ at_marker ~ cue_ident
    ~ (SPACE* ~ "," ~ SPACE* ~ float_lit)? ~ SPACE* ~ paren_close
}

cue_yield = {
    "yield" ~ (paren_open ~ float_lit ~ paren_close)?
}

cue_select = {
    ("select" | "選択待ち") ~ (paren_open ~ float_lit ~ paren_close)?
}

cue_wait = {
    "wait" ~ paren_open ~ float_lit ~ paren_close
}

cue_clear = { "clear" }

cue_route_add = {
    "route_add" ~ paren_open ~ cue_target ~ "," ~ SPACE* ~ entity_key ~ paren_close
}

cue_route_switch = {
    "route_switch" ~ paren_open ~ cue_target ~ "," ~ SPACE* ~ entity_key ~ paren_close
}

cue_route_remove = {
    "route_remove" ~ paren_open ~ cue_target ~ paren_close
}
```

### 3.7 共通プリミティブ

```peg
at_marker = _{ "@" | "＠" }
paren_open  = _{ "(" | "（" }
paren_close = _{ ")" | "）" }

cue_scoped_ident = { (cue_ident ~ ":" ~ cue_ident) | cue_ident }

cue_ident = { (!(WHITESPACE | "(" | ")" | "（" | "）" | "," | "、" | ":" | NEWLINE) ~ ANY)+ }

entity_key = { entity_key_actor | entity_key_spot | entity_key_balloon }
entity_key_actor   = { "actor:" ~ cue_ident ~ ":" ~ cue_target }
entity_key_spot    = { "spot:"    ~ cue_ident }
entity_key_balloon = { "balloon:" ~ cue_ident }

cue_target = { "shell" | "balloon" }

float_lit = { ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }
```

---

## 4. pasta_core が保持すべき AST 情報

### 4.1 重要な原則

AST は **意味解釈済みの型** ではなく、**構造情報** を保持してください。

#### 良い例

```rust
pub struct CueCommandNode {
    pub keyword: String,
    pub alias: Option<ScopedName>,
    pub args: Vec<CueArgToken>,
    pub source_line: u32,
}
```

#### 悪い例

```rust
pub enum CueIrCommand {
    Barrier(BarrierKind),
    RouteAdd { target: CueTarget, to: EntityKey },
}
```

後者は dola ドメイン型を知ってしまうので、今回の設計と矛盾します。

### 4.2 推奨 AST イメージ

型名は pasta プロジェクト側の命名規約に合わせて変更して構いません。必要なのは保持情報です。

```rust
pub struct CueCommandNode {
    pub keyword: String,
    pub alias: Option<ScopedName>,
    pub args: Vec<CueArgToken>,
    pub source_line: u32,
}

pub enum CueArgToken {
    Ident(String),
    StringLiteral(String),
    FloatLiteral(String),
    AtRef(String),
    JsonObject(String),
    EntityKey(String),
}

pub struct ScopedName {
    pub actor: Option<String>,
    pub name: String,
}

pub struct ActionNode {
    pub actor: String,
    pub fragments: Vec<ActionFragment>,
    pub source_line: u32,
}

pub enum ActionFragment {
    Text(String),
    AliasRef(String),
}

pub struct SlotAssignmentNode {
    pub actor: String,
    pub slot: Option<u32>,
    pub source_line: u32,
}
```

### 4.3 必須保持項目

最低限、以下は落とさず保持してください。

- コマンド名
- エイリアス名
- actor 修飾の有無
- 引数の順序
- 文字列リテラルの値
- 数値リテラルの値
- `@name` 参照の文字列
- `%` 行の actor と slot 値
- ソース行番号

---

## 5. アクション行の扱い

### 5.1 `@alias` の分割

```text
さくら：ふふーん @笑顔 いいでしょう？
```

これは概念的に次のように保持してください。

```rust
ActionNode {
    actor: "さくら",
    fragments: [
        Text("ふふーん "),
        AliasRef("笑顔"),
        Text(" いいでしょう？"),
    ],
}
```

### 5.2 継続行

```text
さくら：こんにちは。
：今日はいい天気ですね。
```

継続行は直前アクション行の Text に `\n` 連結してください。

### 5.3 継続行中の `@alias`

```text
：@笑顔
```

これは **構文エラー** としてください。

---

## 6. `%` 行の扱い

既存構文を流用し、C# enum 式自動番号付けを保持してください。

```text
%さくら
%さくら、うにゅう=2、まりか
```

意味解釈や永続管理は dola / areka 側の責務です。pasta 側では構造的に保持できれば十分です。

---

## 7. エラーハンドリング

pasta 側で報告すべきエラーは、構文レベルのものに限定してください。

### 必須エラー

- 不明なキューコマンド名
- 負の数値リテラル
- 名前付き定義の構文不正
- 不正なスロット番号
- 継続行内 `@alias`

### pasta 側で扱わないエラー

以下は dola 側の解釈・ビルド時エラーです。

- mark の重複
- 未登録 mark の参照
- mark と alias の名前衝突
- actor 付き mark
- mark の多重使用
- routing や barrier の意味整合性

---

## 8. 簡易 cue.pasta 例

この例がパースできることを、pasta 側実装の最低確認ラインとしてください。

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

期待されるのは **意味解釈済み CueSheet** ではなく、以下が保持できることです。

- `%さくら` が slot assignment として保持される
- `!emote@普通(normal)` が cue command node として保持される
- `さくら：@普通` が action fragment `[AliasRef("普通")]` として保持される
- 継続行が直前 Text に `\n` 結合される
- `!mark@挨拶後` が mark コマンドとして保持される
- `!yield(10.0)` と `!select(30.0)` の数値引数が保持される

完全版サンプルは別ファイルの [cue.pasta](cue.pasta) を参照してください。

---

## 9. 受け入れチェックリスト

- [ ] `!` 行を既存 `scene_line` に追加した
- [ ] `!` 行の存在有無をシーン単位で検出できる
- [ ] `!command@name(args)` を AST に保持できる
- [ ] `!seek(@name, 0.5)` の `@name` と数値引数を保持できる
- [ ] `!route_add(shell, actor:さくら:shell)` を構造的に保持できる
- [ ] `@alias` をアクション行中で Text と分離できる
- [ ] 継続行を `\n` 結合できる
- [ ] 継続行内 `@alias` をエラーにできる
- [ ] `%` 行の actor / slot を保持できる
- [ ] `!` 行なしシーンの既存挙動を壊していない

---

## 10. 参照資料

- [design.md](design.md) — 最新設計本体
- [cue.pasta](cue.pasta) — 全機能網羅サンプル
- [requirements.md](requirements.md) — 要件定義書
- [research.md](research.md) — 設計判断の背景

---

## 11. 実装者への最終メモ

この文書で最も重要なのは、**pasta_core は dola を知らない** という点です。

実装対象は「DSL 拡張の構文解析と構造的 AST 化」であり、「演出システムの意味解釈」ではありません。ここを越境すると、最新設計と整合しません。

そのため、pasta 側の成果物は次の一文で要約できます。

> `!` 行、`@alias`、`%` 行を既存 DSL に統合し、それらを dola 非依存の構造的 AST として出力すること。
