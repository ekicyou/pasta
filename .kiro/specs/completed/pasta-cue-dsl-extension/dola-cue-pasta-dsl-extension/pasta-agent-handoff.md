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

---

## 2. 設計の最重要前提

今回の採用設計は **Option C'** です。

```text
.pasta ファイル
    ↓
pasta_core
    - PEG 文法拡張
    - 行種別判定
    - 構造的 AST 出力
    - 意味解釈はしない
    ↓
dola
    - pasta AST を解釈
    - AST → CueIR 変換
    - CueSheetBuilder で CueSheet 構築
```

### pasta 側がやらないこと

- `CueIR` を定義しない
- dola のドメイン型に依存しない
- `start_time` を計算しない
- mark / seek / alias / routing の意味解釈をしない
- RouteAdd 自動生成をしない

### pasta 側がやること

- `!` 行を既存 DSL と共存する新規行種別として追加する
- `!` 行を「コマンド名 + 任意のエイリアス名 + 引数トークン列」として保持する
- アクション行中の `@alias` を Text と分離して保持する
- `%` 行の actor / slot 指定を保持する
- シーン内に `!` 行が 1 つ以上あることを検出できるようにする

---

## 3. 実装完了の定義

次を満たせば、pasta 側の実装ゴール達成です。

1. `!` 行を `scene_line` に追加してパースできる
2. シーン内に `!` 行が 1 つ以上あることを検出できる
3. `!command@name(args)` / `!command(args)` / `!mark@name` を AST に保持できる
4. `actor:content` 内の `@alias` を Text と分離して保持できる
5. 継続行 `:content` を直前アクション行の Text に `\n` 結合できる
6. 継続行内 `@alias` を構文エラーとして報告できる
7. `%actor、actor=2` 形式を AST に保持できる
8. `!` 行がないシーンでは既存 pasta の挙動を変えない

---

## 4. モード判定

`&type:cuesheet` は **不要** です。

```text
シーン内に ! コマンド行が 1 つ以上ある
  → cue 拡張シーンとして扱える

シーン内に ! コマンド行が 1 つもない
  → 通常の pasta シーンとして従来どおり扱う
```

つまり、判定条件は **`!` 行の有無** だけです。

---

## 5. 既存文法への統合

`scene_line` の推奨順序は次の通りです。

1. `comment_line`
2. `attribute_line`
3. `cue_cmd_line`
4. `slot_line`
5. `action_line`
6. `continuation_line`

`cue_cmd_line` は、親ルールが先頭空白を消費済みである前提で構いません。

---

## 6. PEG 仕様

### 6.1 キューコマンド行

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

### 6.2 コマンドキーワード

日本語エイリアスは **4 個だけ** 実装対象です。

| 英語 | 日本語 |
|------|--------|
| `emote` | `表情` |
| `choice` | `選択肢` |
| `custom` | `演出` |
| `select` | `選択待ち` |

それ以外は英語のみです。

| コマンド | 記法 |
|---------|------|
| `mark` | `!mark@name` |
| `seek` | `!seek(@name)` / `!seek(@name, 0.5)` |
| `yield` | `!yield` / `!yield(10.0)` |
| `select` | `!select` / `！選択待ち（30.0）` |
| `wait` | `!wait(2.0)` |
| `clear` | `!clear` |
| `route_add` | `!route_add(shell, actor:さくら:shell)` |
| `route_switch` | `!route_switch(balloon, spot:stage_balloon)` |
| `route_remove` | `!route_remove(balloon)` |

### 6.3 名前付き定義

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

### 6.4 タイムライン / バリア / ルーティング

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

### 6.5 共通プリミティブ

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

## 7. AST に保持すべき情報

### 7.1 原則

**意味解釈済みの型ではなく、構造情報だけを持つこと。**

良い方向性の例:

```rust
pub struct CueCommandNode {
    pub keyword: String,
    pub alias: Option<ScopedName>,
    pub args: Vec<CueArgToken>,
    pub source_line: u32,
}
```

### 7.2 推奨保持モデル

型名は変えて構いません。必要なのは保持内容です。

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

### 7.3 最低限必要な保持項目

- コマンド名
- エイリアス名
- actor 修飾の有無
- 引数の順序
- 文字列リテラルの値
- 数値リテラルの値
- `@name` 参照文字列
- `%` 行の actor / slot
- ソース行番号

---

## 8. アクション行 / 継続行

### 8.1 `@alias` の分割

入力:

```text
さくら：ふふーん @笑顔 いいでしょう？
```

保持イメージ:

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

### 8.2 継続行

入力:

```text
さくら：こんにちは。
：今日はいい天気ですね。
```

継続行は直前アクション行の Text に `\n` 連結してください。

### 8.3 継続行中の `@alias`

```text
：@笑顔
```

これは **構文エラー** にしてください。

---

## 9. `%` 行

既存構文を流用し、C# enum 式自動番号付けを保持してください。

```text
%さくら
%さくら、うにゅう=2、まりか
```

意味解釈や永続管理は pasta 側の責務ではありません。構造的に保持できれば十分です。

---

## 10. pasta 側で出すべきエラー

### 必須

- 不明なキューコマンド名
- 負の数値リテラル
- 名前付き定義の構文不正
- 不正なスロット番号
- 継続行内 `@alias`

### pasta 側で扱わないもの

以下は dola 側の解釈・ビルド時エラーです。

- mark の重複
- 未登録 mark の参照
- mark と alias の名前衝突
- actor 付き mark
- mark の多重使用
- routing / barrier の意味整合性

---

## 11. 最低確認用の簡易サンプル

このサンプルがパースできることを最低確認ラインにしてください。

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

この入力に対して必要なのは、**CueSheet ではなく AST が正しく保持されること** です。

期待観点:

- `%さくら` が slot assignment として保持される
- `!emote@普通(normal)` が cue command node として保持される
- `さくら：@普通` が `AliasRef("普通")` として保持される
- 継続行が前行 Text に `\n` 結合される
- `!mark@挨拶後` が mark コマンドとして保持される
- `!yield(10.0)` / `!select(30.0)` の数値引数が保持される

---

## 12. 実装チェックリスト

- [ ] `!` 行を既存 `scene_line` に追加した
- [ ] `!` 行の有無をシーン単位で検出できる
- [ ] `!command@name(args)` を AST に保持できる
- [ ] `!seek(@name, 0.5)` の `@name` と数値引数を保持できる
- [ ] `!route_add(shell, actor:さくら:shell)` を構造的に保持できる
- [ ] `@alias` をアクション行中で Text と分離できる
- [ ] 継続行を `\n` 結合できる
- [ ] 継続行内 `@alias` をエラーにできる
- [ ] `%` 行の actor / slot を保持できる
- [ ] `!` 行なしシーンの既存挙動を壊していない

---

## 13. 必要なら参照してよい元資料

この文書だけで足りるよう圧縮していますが、詳細確認が必要なら以下を参照してください。

- `design.md`
- `cue.pasta`
- `requirements.md`
- `research.md`

---

## 14. 最後に一文で言うと

> `!` 行、`@alias`、`%` 行を既存 DSL に統合し、それらを **dola 非依存の構造的 AST** として出力してください。