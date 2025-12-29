# 要件ドキュメント

## プロジェクト説明（入力）

**Pasta DSL → Lua コード生成（トランスパイラー層）の仕様化**

本仕様における実装スコープは「**トランスパイラー層のみ**」とする。ランタイム層（Lua VM実行、coroutine管理、word/talk関数実装）はスコープ外。

**成果物**:
- Pasta AST → Lua コード生成ルールの仕様書
- sample.lua のような形式でのコード出力
- AST の各ノードから Lua コード文字列への変換ルール定義
- シーン・単語レジストリへのエントリー登録ロジック

**出力形式**: `Write` トレイト実装による直接 Lua コード出力（ファイル/バッファに書き込み可能）

## 背景・コンテキスト

### 現在のアーキテクチャ（Rune層）
- **Parser**: Pasta DSL → AST変換（pasta_core、Pest使用）
- **Transpiler**: AST → Rune コード生成（2パス戦略）
  - Pass 1: シーン・単語登録＋モジュール生成
  - Pass 2: scene_selector関数＋pasta ラッパー生成
- **Runtime**: Rune VM実行、yield型出力

### Lua トランスパイラー化の目的
- Rune VM への依存性排除（トランスパイラー層での実装）
- Pasta AST → Lua コード生成メカニズムの確立
- pasta_lua クレート内にトランスパイルロジックを実現

### Lua選択の理由（トランスパイラー層視点）
| 理由 | 詳細 |
|------|------|
| **シンプルな文法** | テーブル、関数、ローカル変数の生成が直線的 |
| **Unicode識別子サポート** | Lua 5.3以降は日本語シーン名・変数名をネイティブサポート |

### 権威的リファレンス
- `crates/pasta_core/src/parser/grammar.pest`: Pasta DSL文法定義
- `/SPECIFICATION.md`: Pasta DSL仕様書
- **Reference Implementation**: `.kiro/specs/pasta-lua-specification/sample.lua`
- 既存 Rune トランスパイラー: `crates/pasta_rune/src/transpiler/code_generator.rs` (参考パターン)

## スコープ明確化

### ✅ 本仕様に含まれる（トランスパイラー層）
- Pasta AST ノードから Lua コード文字列への変換ルール
- シーン定義（do...end ブロック）の生成
- アクター定義の生成
- ローカル・グローバル変数参照の展開
- Call 文から `act:call()` への変換
- シーン・単語レジストリへのエントリー登録
- コメント付きコード生成（要件追跡用）

### ❌ 本仕様から除外される（別仕様で対応）
- word(), talk() メソッドの Lua 実装
- Lua coroutine の制御・管理
- create_session() の実装詳細
- メタテーブル、メタメソッドの動作
- シーンセレクター、単語選択のランダマイズ実装

## 要件

### 0. Luaローカル変数数制限への対応設計

**目的**: Luaのローカル変数数制限（約200個）を超えないトランスパイル出力設計を確立

**背景**: トランスパイラーで機械的にコードを生成する際、アクター・シーン・変数が増えるとローカル変数が蓄積しやすい。

**受け入れ基準**:
- The Transpiler shall ローカル変数の宣言を最小化する設計パターンを採用する（単一変数の再利用、テーブル格納、スコープ分離）
- When アクター定義が複数あるとき、Transpiler shall 各アクター定義を `do...end` ブロックで分離し、ブロックごとに `local ACTOR` を再利用する（チェッカー警告回避、スコープ明確化）
- When シーン定義が複数あるとき、Transpiler shall 各グローバルシーン定義を `do...end` ブロックで分離し、ブロックごとに `local SCENE` を再利用する
- When 関数内でローカル・グローバル・アクター動作を管理するとき、Transpiler shall `var`, `save`, `act` の3個テーブルのみを使用する
- The Transpiler shall スコープ分離パターン（`do...end` ブロック）により、各ブロック内で約200個のローカル変数枠を確保できる構造を生成する

### 0-2. Lua文字列リテラル形式の標準化

**目的**: Pastaテキスト・Sakuraスクリプトを含む文字列を、最適な形式で統一的に生成

**背景**: テキストに含まれる特殊文字（`\`, `"`）の扱いが複雑。簡潔さと安全性を両立させるアルゴリズムが必要。

**リテラル化ルール**:
- **ルール1**: テキスト内にエスケープ対象文字（`\` または `"`）が**含まれない**場合 → `"通常の文字列"` 形式
- **ルール2**: テキスト内にエスケープ対象文字が**1つ以上含まれる**場合 → Lua長文字列形式を使用（ルール3に従って`=`の個数を決定）

**長文字列形式の`=`決定アルゴリズム**:
- The Transpiler shall `n = 0`（`=`の個数）から開始する
- **危険パターン**: Lua長文字列の終端パターン `]=...=]` から末尾の `]` を除いた部分
  - `n=0`: 危険パターン = `]`（終端 `]]` から末尾除く）
  - `n=1`: 危険パターン = `]=`（終端 `]=]` から末尾除く）
  - `n=2`: 危険パターン = `]==`（終端 `]==]` から末尾除く）
  - 一般形: 危険パターン = `]` + `n個の=`
- When テキスト内に危険パターンが**含まれるとき**、Transpiler shall `n = n + 1`に増やして再度判定する
- The Transpiler shall 上記判定を繰り返し、危険パターンが含まれない最小の`n`値を選択し、`[=`+`n個の=`+`[...]=`+`n個の=`+`]` 形式で生成する

**具体例**:
- `hello world` → `"hello world"` (ルール1)
- `hello\nworld` → `[[hello\nworld]]` (ルール2, n=0)
- `hello]world` → `[=[hello]world]=]` (ルール2, n=1)

**受け入れ基準**:
- The Transpiler shall ルール1→ルール2の順に判定し、適切なリテラル形式を選択する
- The Transpiler shall 判定アルゴリズムを Rust コードで実装する

### 1. Pasta AST → Lua コード生成ルール定義

**目的**: Pasta AST構造をLuaコードにマッピングするルールセット確立

#### 1a. アクター定義のLua化

**目的**: アクター辞書（`％アクター名`）とその属性をLua構造に変換

**受け入れ基準**:
- When `％さくら` アクター定義があるとき、Transpiler shall `do...end` ブロックで分離し、ブロック内で `local ACTOR = PASTA:create_actor("さくら")` と生成する
- When アクター属性（`＄通常：\s[0]`）が続くとき、Transpiler shall `ACTOR.通常 = [=[\s[0]]=]` と生成する（Requirement 0-2の文字列リテラル形式判定アルゴリズムを適用）
- Where アクター属性が複数あるとき、Transpiler shall 同一ACTOR変数への連続代入として生成する
- The Transpiler shall 複数アクター定義時に各定義を独立した `do...end` ブロックで分離することで、ACTOR変数の再利用を明確化する

#### 1b. シーン定義とモジュール構造

**目的**: グローバルシーン（`＊メイン`）をLuaテーブル・関数構造に変換

**受け入れ基準**:
- When `＊メイン` グローバルシーン定義があるとき、Transpiler shall `do...end` ブロックで分離し、ブロック内で `local SCENE = PASTA:create_scene("モジュール名")` と生成する
- When ローカル単語定義（`＠場所：東京、大阪`）があるとき、Transpiler shall Lua コード出力を生成せず、内部の WordDefRegistry に登録する
- When グローバル単語定義（`＠挨拶：こんにちは、やあ`）があるとき、Transpiler shall Lua コード出力を生成せず、内部の WordDefRegistry に登録する
- When ファイルレベル属性（`＆天気：晴れ`）があるとき、Transpiler shall パーサーから取得するが、トランスパイラー層では処理しない（属性実装は後続仕様）
- The Transpiler shall 複数グローバルシーン定義時に各定義を独立した `do...end` ブロックで分離し、スコープ混在を避ける

#### 1c. ローカルシーン関数への変換

**目的**: ローカルシーン（`・自己紹介`）をLua関数として生成

**背景**: ローカルシーンは各グローバルシーン内でのみ一意である必要があり、別のグローバルシーン間では同じ名前を持つことが可能。命名規則は Rune 実装に従う

**受け入れ基準**:
- When グローバルシーン `＊メイン` のエントリーポイントを生成するとき、Transpiler shall `function SCENE.__start__(ctx, ...)` と生成する
- When 第1階層ローカルシーン `・自己紹介` があるとき、Transpiler shall `function SCENE.__自己紹介_N__(ctx, ...)` と生成する（`__ローカルシーン名_N__` 形式、N=1,2,3...）
- Where N は各グローバルシーン内でのローカルシーン定義順序（0-indexed を 1-indexed に変換）。同名シーンの重複有無に関わらず、常に 1 から開始
- The Transpiler shall すべてのシーン関数の第一行を `local args = { ... }` とし、第二行を `local act, save, var = PASTA:create_session(SCENE, ctx)` として生成する

#### 1d. 変数スコープ管理（var/save/act分離）

**目的**: ローカル・グローバル・永続変数をコード生成として定義

**受け入れ基準**:
- The Transpiler shall ローカル変数（`＄カウンタ`）を `var.カウンタ = ...` として生成する
- The Transpiler shall グローバル変数（`＄＊グローバル`）を `save.グローバル = ...` として生成する
- The Transpiler shall Call文（`＞ラベル`）を `act:call("モジュール名", "ラベル名", {}, table.unpack(args))` として生成する
- The Transpiler shall 関数呼び出し（`＠関数`）を `SCENE:関数(ctx, ...)` コード文字列として生成する
- When 引数参照（`＄N`）があるとき、Transpiler shall Pasta DSL の 0-indexed を Lua の 1-indexed に変換し、`args[N+1]` として生成する（例：`＄０` → `args[1]`, `＄１` → `args[2]`）

#### 1e. 単語参照の処理（コード生成）

**目的**: 単語定義・参照をコード生成の観点で定義

**受け入れ基準**:
- When ローカル単語定義（`＠場所：東京、大阪`）があるとき、Transpiler shall Lua コード出力を生成せず、内部の WordDefRegistry に登録する
- When グローバル単語定義（`＠挨拶：こんにちは、やあ`）があるとき、Transpiler shall Lua コード出力を生成せず、内部の WordDefRegistry に登録する
- When アクション行内で単語参照（`＠挨拶`）があるとき、Transpiler shall `act.アクター:word("挨拶")` をコード出力する

#### 1f. Luaコードブロック（ブロック抽出）

**目的**: Pasta スクリプト内に埋め込まれたコードブロックをコード生成にどう含めるか定義

**背景**: コードブロックは ` ``` ` で識別。言語識別子（lua, rune など）はパーサーで取得するがトランスパイラー層では使用しない

**受け入れ基準**:
- When Pastaスクリプト内に ` ``` ` で囲まれたコードブロックがあるとき、Transpiler shall そのブロック内容をそのまま Lua 出力に含める
- The Transpiler shall ブロック開始の言語識別子（```lua, ```rune など）は無視し、内容のみを処理する
- The Transpiler shall ブロック内のコードをそのまま出力する（構文変換は実装スコープ外）

#### 1g. グローバルシーン間参照（コード生成パターン）

**目的**: AST 内のグローバルシーン参照をコード生成パターンとして定義

**受け入れ基準**:
- When Call 文で別のグローバルシーンを参照するとき、Transpiler shall 生成 Lua コード内では同じレジストリ経由の呼び出し構文を使用する

### 2. トランスパイラー実装の制約・前提

**目的**: トランスパイラー層の実装スコープを明確にする

**受け入れ基準**:
- The Transpiler shall Lua コードを `std::io::Write` トレイト実装により出力可能な形式で生成する
- The Transpiler shall コード内に Requirement 番号をコメント（`-- (Requirement 1a)` 等）で埋め込む
- The Transpiler shall 文法エラー時の処理を `Result<T, TranspileError>` 型で定義する
- When 複数パスが必要なとき、Transpiler shall Rune の 2pass 戦略（Pass 1: レジストリ登録、Pass 2: コード生成）を参考に実装する
- The Transpiler shall パーサーから属性（`＆key：value`）を取得するが、トランスパイラー層では処理しない（属性実装は後続仕様）

### 3. シーン・単語レジストリへのエントリー登録

**目的**: トランスパイル時にレジストリに登録すべき情報を定義

**受け入れ基準**:
- When グローバルシーン定義（`＊メイン`）があるとき、Transpiler shall SceneRegistry にシーン情報を登録する
- When ローカルシーン定義（`・自己紹介`）があるとき、Transpiler shall SceneRegistry にラベル情報として登録する
- When グローバル単語定義（`＠挨拶：こんにちは`）があるとき、Transpiler shall WordDefRegistry に登録する
- When ローカル単語定義があるとき、Transpiler shall ローカルスコープの WordDefRegistry に登録する
- The Transpiler shall パーサーから属性（`＆key：value`）を取得するが、トランスパイラー層では処理しない（属性実装は後続仕様）

### 4. 参照実装による検証

**目的**: sample.pasta → sample.lua の手作業変換により、トランスパイルルールを検証

**受け入れ基準**:
- The sample.lua shall sample.pasta に含まれる全機能をカバーする
- The sample.lua shall トランスパイルルールの参照実装として機能する
- When 各 AST ノードと生成 Lua コードの対応を示すとき、Comment shall Requirement 番号を含める

