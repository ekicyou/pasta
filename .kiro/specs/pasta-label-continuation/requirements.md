# Requirements Document

## Introduction
pasta DSLにおいて、グローバルシーンを示す「＊」行でシーン名を省略した場合に、同一DSLファイル内で最後に指定されたグローバルシーン名を継続利用する振る舞いを定義する。また、最初の「＊」行でシーン名を省略した場合には、前回のシーン名を参照できないためエラーとして扱うことを明確化する。

## Requirements

### 1. グローバルシーン名継続解決
**Objective:** シーン名を省略した「＊」行で直近のグローバルシーン名を継続利用し、重複シーン定義を簡潔に記述できるようにする。

#### Acceptance Criteria
1. When the Pasta DSL parser encounters a `＊` line without an explicit scene name after at least one named global scene has been parsed in the same file, the Pasta DSL parser shall set the current global scene name to the last named global scene and register the scene as a new variant of that name.
2. While multiple successive `＊` lines without scene names are parsed, the Pasta DSL parser shall maintain the last resolved global scene name and apply it to each successive unnamed `＊` line.
3. When a local scene `・` is parsed immediately after an unnamed `＊` continuation, the Pasta DSL parser shall associate the local scene with the continued global scene name.
4. When attributes or action lines follow an unnamed `＊` continuation, the Pasta DSL parser shall attach those lines to the continued global scene context without altering the remembered scene name.

### 2. シーン名更新のコンテキスト管理
**Objective:** 明示的なシーン名指定時に継続用コンテキストを正しく更新し、以降の省略時利用を一貫させる。

#### Acceptance Criteria
1. When the Pasta DSL parser parses a `＊` line with an explicit scene name, the Pasta DSL parser shall update the last-used global scene name context to that explicit name for subsequent continuations.
2. While parsing non-scene lines (comments, attributes, actions, or local scenes), the Pasta DSL parser shall preserve the last-used global scene name context until another explicit `＊` with a name is encountered.
3. If a new DSL file is parsed, the Pasta DSL parser shall reset the last-used global scene name context before processing the first line of that file.

### 3. 省略シーン名のエラーハンドリング
**Objective:** シーン名未確定の状態での省略を検出し、診断可能なエラーを報告する。

#### Acceptance Criteria
1. If the first `＊` line in a DSL file omits the scene name, the Pasta DSL parser shall reject the file with an error indicating that no prior global scene name exists for continuation.
2. If an unnamed `＊` line appears when no named global scene has been successfully parsed earlier in the same file (including when preceding lines are only comments, whitespace, or local scenes), the Pasta DSL parser shall emit an error that identifies the missing global scene context.
3. When emitting an error for an unnamed `＊` without available context, the Pasta DSL parser shall include the line number and a message that specifies the need for a preceding named global scene.

### 4. 無名「＊」行のフォーマット制約
**Objective:** 無名「＊」行の許容フォーマットを明確化し、曖昧さを排除する。

#### Acceptance Criteria
1. Comment markers (`#` or `＃`) shall not be permitted on unnamed `＊` lines. Any characters other than optional trailing whitespace after the `＊` marker shall be rejected as syntax errors.
2. Comment lines shall be recognized only when a line begins with optional whitespace followed by a comment marker (`#` or `＃`), consistent with the DSL grammar; inline trailing comments are not recognized anywhere in the file.
