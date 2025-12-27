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

### 1. Pasta AST → Lua コード生成ルール定義

**目的**: Rune AST構造をLuaコードにマッピングするルールセット確立

**受け入れ基準**:
- When トランスパイラーがPasta AST（GlobalSceneScope/LocalSceneItem）を入力するとき、Transpiler shall マッピングルールに従いLuaテーブル・関数構造を生成する
- When シーン内にローカル変数（`$var`）が存在するとき、Transpiler shall それをLuaローカル変数（`local var = ...`）に変換する
- When シーン内にグローバル変数（`$*var`）が存在するとき、Transpiler shall それを共有テーブル参照（`_G.pasta_vars.var`）に変換する
- When 単語定義（`@word:word1 word2`）があるとき、Transpiler shall ランダム選択関数をLuaで生成する（ipairs＋math.random使用）
- Where Sakuraスクリプト（`\\w[n]`等）が含まれるとき、Transpiler shall エスケープシーケンスを保持したLua文字列として生成する
- The Transpiler shall Call文（`>scene`）をLua関数呼び出しに変換する（スタック管理含む）

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
