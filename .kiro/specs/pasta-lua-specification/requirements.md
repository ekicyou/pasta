# 要件ドキュメント

## プロジェクト説明（入力）
トランスパイラー・ランタイム層をLuaに変更したpasta_luaを作る。まずは、トランスパイルルールを決定したい。成果物としてpasta_luaのルートフォルダあたりにトランスパイル？.mdとかを作りたいと思う。本仕様における実装は成果物ドキュメントを作ることである。

## 背景・コンテキスト

### 現在のアーキテクチャ（Rune層）
- **Parser**: Pasta DSL → AST変換（pasta_core、Pest使用）
- **Transpiler**: AST → Rune コード生成（2パス戦略）
  - Pass 1: シーン・単語登録＋モジュール生成
  - Pass 2: scene_selector関数＋pasta ラッパー生成
- **Runtime**: Rune VM実行、yield型出力、Sakuraスクリプト埋め込み

### Lua化の目的
- Runeへの依存性を排除
- 軽量なスクリプト実行環境（Lua）への移行
- pasta_lua クレート内に完全な独立実装を実現

### Lua採用の理由

| 理由 | 詳細 |
|------|------|
| **スタックフルコルーチン** | Luaのcoroutineはスタックフル（任意の深さでのyield）であり、アクションの継続（Wait/Sync後の再開）を自然に表現できる。Rune generatorとの差異を吸収し、pasta_lua独自の実装戦略が可能。 |
| **メタテーブル機構** | 連想配列（テーブル）にメタテーブルを設定することで、カスタム動作（演算子オーバーロード、キー未検出時の動作）を実装可能。シーンレジストリ・単語定義の拡張性が向上。 |
| **日本語識別子サポート** | Lua 5.3以降はUnicode識別子をサポート。pastaの「日本語シーン名・変数名」をLua側でも自然に利用可能（エスコープ不要）。 |

### 権威的リファレンス
- `crates/pasta_core/src/parser/grammar.pest`: Pasta DSL文法定義（最上位仕様）
- `/SPECIFICATION.md`: Pasta DSL仕様書（全11章、grammar.pestの説明）
- `grammar.md`: マーカー・制御フロー・IR定義
- 既存トランスパイラー: `src/transpiler/code_generator.rs`

## 要件

### 0. Luaローカル変数数制限への対応設計

**目的**: Luaのローカル変数数制限（約200個）を超えないトランスパイル出力設計を確立

**背景**: トランスパイラーで機械的にコードを生成する際、アクター・シーン・変数が増えるとローカル変数が蓄積しやすい。この制約に引っかからない設計パターンの確立が必須。

**受け入れ基準**:
- The Transpiler shall ローカル変数の宣言を最小化する設計パターンを採用する（単一変数の再利用、テーブル格納）
- When アクター定義が複数あるとき、Transpiler shall `local ACTOR`（1個）と`local ACTORS`（1個）で管理し、アクター数に依らずローカル変数を2個に抑える
- When シーン定義が複数あるとき、Transpiler shall `local SCENE`（1個）で管理し、シーン数に依らずローカル変数を1個に抑える
- When 関数内でローカル・グローバル・アクター動作を管理するとき、Transpiler shall `var`, `save`, `act` の3個テーブルのみを使用し、個別変数を避ける
- The Transpiler shall テーブル再利用パターン（`VAR = VAR or {}; VAR.name = value`）で複数定義に対応する
- The Document shall 生成Luaコード内で使用するローカル変数数の見積もり方法（最大値の計算方法）を記載する

### 0-2. Lua文字列リテラル形式の標準化

**目的**: Pastaテキスト・Sakuraスクリプトを含む文字列を、最適な形式で統一的に生成

**背景**: Pastaテキストに含まれる特殊文字（`\`, `"`）の扱いが複雑。簡潔さと安全性を両立させる「リテラル化関数」の仕様が必要。

**リテラル化ルール**:
- **ルール1**: テキスト内にエスケープ対象文字（`\` または `"`）が**含まれない**場合 → `"通常の文字列"` 形式を使用
- **ルール2**: テキスト内にエスケープ対象文字が**1つ以上含まれる**場合 → Lua長文字列形式を使用（ルール3に従って`=`の個数を決定）

**長文字列形式の`=`決定アルゴリズム**:
- The Transpiler shall `n = 0`（`=`の個数）から開始する
- **終端文字列**: `n`個の`=`を使った場合の終端は `]` + `n個の=` + `]` （例: n=0なら`]]`, n=1なら`]=]`, n=2なら`]==]`）
- **危険パターン**: 終端文字列から**末尾の1文字を除いた部分** （例: n=0なら`]`, n=1なら`]=`, n=2なら`]==`）
- When テキスト内に危険パターンが**含まれるとき**、Transpiler shall `n = n + 1`に増やして再度判定する
- The Transpiler shall 上記判定を繰り返し、危険パターンが含まれない最小の`n`値を選択する

**具体例**:
- テキストが`hello world`の場合: エスケープ対象文字（`\`, `"`）なし → `"hello world"`
- テキストが`hello\nworld`の場合: `\`が含まれる、`]`（n=0の危険パターン）なし → n=0 → `[[hello\nworld]]`
- テキストが`hello]world`の場合: `]`（n=0の危険パターン）が含まれる → n=1 → `[=[hello]world]=]`
- テキストが`hello]=world`の場合: `]=`（n=1の危険パターン）が含まれる → n=2 → `[==[hello]=world]==]`

**受け入れ基準**:
- The Transpiler shall ルール1→ルール2の順に判定し、適切なリテラル形式を選択する
- The Document shall 判定アルゴリズムの疑似コード（例：正規表現で`]\=*`パターンをマッチングし、最大の`=`個数+1を`n`とする方法）を記載する

### 1. Pasta AST → Lua コード生成ルール定義

**目的**: Pasta AST構造をLuaコードにマッピングするルールセット確立（5つの領域に分割）

#### 1a. アクター定義のLua化

**目的**: アクター辞書（`％アクター名`）とその属性をLua構造に変換

**受け入れ基準**:
- When `％さくら` アクター定義があるとき、Transpiler shall `ACTOR = PASTA:create_actor("さくら")` と生成する
- When アクター属性（`＄通常：\s[0]`）が続くとき、Transpiler shall `ACTOR.通常 = [==[\s[0]]==]` と生成する（マルチライン文字列で保存）
- The Transpiler shall Sakuraスクリプトシーケンスをエスケープせず、Lua文字列リテラル（`[==[...]==]`）として保持する
- Where アクター属性が複数あるとき、Transpiler shall 同一ACTOR変数への連続代入として生成する

#### 1b. シーン定義とモジュール構造

**目的**: グローバルシーン（`＊メイン`）をLuaテーブル・関数構造に変換

**受け入れ基準**:
- When `＊メイン` グローバルシーン定義があるとき、Transpiler shall `SCENE = PASTA:create_scene("メイン1")` と生成する（重複対策のカウンタ付与）
- The Transpiler shall グローバルシーンをモジュール化せず、同一ファイル内のテーブルメンバーとして実装する（関数は`function SCENE.関数名(...)`形式）
- When ファイルレベル属性（`＆天気：晴れ`）があるとき、Transpiler shall Lua出力を生成せず、内部レジストリに記録する
- When グローバル単語定義（`＠挨拶：こんにちは、やあ`）があるとき、Transpiler shall Lua出力を生成せず、WordDefRegistry に登録する

#### 1c. ローカルシーン関数への変換

**目的**: ローカルシーン（`・自己紹介`）をLua関数として生成

**受け入れ基準**:
- When ローカルシーン `・自己紹介` があるとき、Transpiler shall `function SCENE.__自己紹介1__(scene, ctx, ...)` と生成する（カウンタ付与）
- The Transpiler shall ローカルシーンの実装を名前付きLua関数（アンダースコア+ローマ字+数字）とし、メタテーブルやクロージャでの隠蔽は避ける
- When ローカルシーン内に最初のアクション行があるとき、Transpiler shall 関数の第一行を `local args = { ... }; local act, save, var = PASTA:create_session(scene, ctx)` と生成する

#### 1d. 変数スコープ管理（var/save/act分離）

**目的**: ローカル・グローバル・永続変数を明確に管理するLua構造を定義

**受け入れ基準**:
- When ローカル変数（`＄カウンタ`）が代入されるとき、Transpiler shall `var.カウンタ = 10` と生成する
- When グローバル変数（`＄＊グローバル`）が代入されるとき、Transpiler shall `save.グローバル = ...` と生成する（`save`は永続テーブル）
- When アクター発言が発生するとき、Transpiler shall `act.さくら:talk("テキスト")` と生成する（`act`はアクター動作テーブル）
- The Transpiler shall act/save/var の3つのテーブルを `PASTA:create_session()` で初期化し、メタテーブル設定は避ける
- When 関数呼び出しが必要なとき、Transpiler shall `scene.関数(...)` で呼び出す（既存レジストリとの連携）

#### 1e. 単語・属性の処理戦略

**目的**: 単語定義（`@word`）と属性（`&attr`）のLua側処理を明確化

**受け入れ基準**:
- When ローカル単語定義（`＠場所：東京、大阪`）があるとき、Transpiler shall Lua出力を生成せず、ローカルスコープのWordDefRegistry に登録する
- When アクション行内で単語参照（`＠挨拶`）があるとき、Transpiler shall `act.さくら:word("挨拶")` と生成し、Lua側で単語選択ロジックを実装する
- The Transpiler shall 単語選択のランダマイズロジックを（Lua生成ではなく）PASTA.word()メソッド内に委譲する

### 2. Rune Generator → Lua Coroutine マッピング

**目的**: Rune generator（yield型）をLua coroutineに対応付ける仕様化

**受け入れ基準**:
- When Pastaスクリプトがyield出力を行うとき、Transpiler shall Lua coroutine.yield()による段階的実行に変換する
- The Transpiler shall generator関数の実行流をLua coroutineの再開可能状態として模式化する
- When複数シーン間の遷移（Call/Jump的動作）が必要なとき、Transpiler shall coroutineスタック管理メカニズムを定義する
- The Transpiler shall 実行コンテキスト保存（変数スコープ・実行位置）をLua環境テーブルで実装可能なレイアウトを提示する

### 3. シーンセレクター・レジストリのLua実装モデル

**目的**: 既存のRune scene_selector()相当機能をLuaで実装するための仕様

**受け入れ基準**:
- The Transpiler shall シーンテーブル（ID→関数マッピング）をLuaテーブル構造で定義する（例：`pasta.scenes[scene_id] = function_ref`）
- When 前方一致検索（LabelTable::find_by_prefix）が必要なとき、Transpiler shall Lua内での検索ロジック（string.sub + ipairs）を提示する
- The Transpiler shall シーン優先度・重複時ランダム選択をLua側（math.random）で実装可能な形式を定義する
- When ローカルシーン呼び出しが発生するとき、Transpiler shall モジュール内ローカル参照（同一テーブル内関数）として処理する仕様を定義する

### 4. Pasta標準ライブラリ関数のLua相当実装

**目的**: 既存 pasta_stdlib（Rune）相当機能をLuaで実装するためのインターフェース定義

**受け入れ基準**:
- The Transpiler shall 単語ランダム選択（`select_word`）の実装型（Lua function）を提示する
- The Transpiler shall シーン選択関数（`select_scene_to_id`）の引数・戻り値型をLua関数として定義する
- The Transpiler shall 変数スコープ管理関数（`set_global_var`, `get_local_var`等）のLua実装APIを提示する
- When Sakuraスクリプト埋め込み処理が必要なとき、Transpiler shall エスケープルール（`\\` → Lua文字列内での表現）を明確化する

### 5. トランスパイル・ドキュメント成果物

**目的**: `crates/pasta_lua/`直下に詳細なトランスパイルルール仕様書を作成

**受け入れ基準**:
- The Transpiler specification document shall 全5つのルール領域（AST→Lua、Generator→Coroutine、シーンセレクター、stdlib、ドキュメント）を網羅する
- When ドキュメントが作成されるとき、Document shall 各ルール領域に対して以下を含む：概要説明、パスタ側AST構造、Lua側実装形式、具体例（コード片）、テスト戦略
- Where 既存実装（Rune版）があるとき、Document shall 対応関係の説明（「Rune X → Lua Y」）を明示する
- The Document shall 実装の曖昧性を解消するための判断基準（どちらを選ぶかの決定ルール）を記載する

### 6. 実装検証用テストケース仕様

**目的**: 各トランスパイルルールの実装可能性を検証するテスト戦略定義

**受け入れ基準**:
- The Transpiler specification shall 各ルール領域（1-4）に対して最低3つのテストシナリオを示す
- When ローカル変数処理テストを記述するとき、Test shall 初期化・参照・更新の3パターンを含む
- When Call文テストを記述するとき、Test shall ネストされたCall、戻り値処理、エラーハンドリングを含む
- The Test shall 既存Rune実装との比較（同等性検証）を念頭に設計する
