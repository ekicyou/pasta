# 実装タスク

## 1. トランスパイラー基盤の構築

- [x] 1.1 LuaTranspiler 構造体と基本的なトランスパイル処理フロー実装 (P)
  - Pasta AST をパーサー層から受け取るメイン構造体を定義
  - Pass 1 での統一処理フロー（ActorScope → LuaCodeGenerator → Lua出力）を実装
  - TranspileError 型定義と thiserror による統一的なエラーハンドリング
  - Write トレイト実装による汎用的な出力先対応（ファイル・StringWriter 等）
  - _Requirements: 4, 3_

- [x] 1.2 StringLiteralizer による Lua 文字列リテラル化処理 (P)
  - 危険パターン判定アルゴリズムの実装（n=0,1,2...と進むロジック）
  - テキス内に特殊文字（`\`, `"`）がない場合は単純形式 `"..."` で出力
  - テキスト内に特殊文字がある場合は適切な `[=...=]` 形式を選択
  - 複数の n 値をテストして最小の有効な n を決定
  - ユニットテスト（単語参照、Talk文、複雑なエスケープパターン）
  - _Requirements: 2_

- [x] 1.3 Parser インテグレーションと AST 構造体マッピング (P)
  - pasta_core::Parser から Pasta AST を取得
  - GlobalSceneScope, LocalSceneScope, ActorScope, Span 構造体との対応を確認
  - パーサーエラーの TranspileError への変換
  - _Requirements: 4_

## 2. LuaCodeGenerator による出力生成

- [x] 2.1 アクター定義ブロック生成 (P)
  - ActorScope ノードから `local ACTOR = PASTA:create_actor(...)` コードを生成
  - 各アクター属性を `ACTOR.属性名 = [...]=]` 形式で出力
  - 複数アクター定義を `do...end` ブロックで分離し、ACTOR 変数を再利用
  - 属性値の文字列リテラル化（StringLiteralizer 使用）
  - _Requirements: 1, 3a_

- [x] 2.2 グローバルシーン定義ブロック生成 (P)
  - GlobalSceneScope ノードから `local SCENE = PASTA:create_scene("モジュール名_N")` コードを生成
  - グローバルシーン定義順に番号付け（N=1,2,3...）し、重複有無に関わらず常に `_N` を付与
  - エントリーポイント関数 `function SCENE.__start__(ctx, ...) ... end` を生成
  - 引数テーブル化 `local args = { ... }` とセッション初期化 `local act, save, var = PASTA:create_session(SCENE, ctx)` を生成
  - 複数グローバルシーン定義を独立した `do...end` ブロックで分離
  - _Requirements: 1, 3b_

- [x] 2.3 ローカルシーン関数生成 (P)
  - LocalSceneScope ノードから `function SCENE.__シーン名_N__(ctx, ...) ... end` 形式を生成
  - グローバルシーン内でのローカルシーン定義順に番号付け（N=1,2,3...）
  - 同一グローバルシーン内での重複名に対応（番号で区別）
  - セッション初期化コード（args テーブル化と PASTA:create_session 呼び出し）を生成
  - _Requirements: 1, 3c_

- [x] 2.4 変数代入・参照コード生成 (P)
  - VarSet ノードからローカル変数 `var.変数名 = 値` を生成
  - グローバル変数 `save.変数名 = 値` を生成
  - 関数呼び出し（FnCall）を `SCENE:関数名(ctx, 引数...)` 形式で生成
  - 引数参照（＄N）を Pasta の 0-indexed から Lua の 1-indexed に変換（args[N+1]）
  - _Requirements: 1, 3d_

- [x] 2.5 Call 文と単語参照コード生成 (P)
  - CallScene ノードから `act:call("モジュール名", "ラベル名", {}, ...引数)` を生成
  - 属性フィルター第3引数は `{}` 空テーブル（将来拡張用）
  - 引数の `table.unpack(args)` による継承を実装
  - Action::WordRef を `act.アクター:word("単語名")` に変換
  - グローバル・ローカル単語定義を WordDefRegistry に登録（Lua コード出力なし）
  - _Requirements: 1, 3d, 3e_

- [x] 2.6 Talk 文とアクション生成 (P)
  - Action::Talk ノードから `act.アクター:talk("文字列")` を生成
  - テキスト内の通常テキストと単語参照の分離（walk して生成）
  - 文字列リテラル化（StringLiteralizer 経由）
  - Action::VarRef を `var.変数名` または `save.変数名` に展開
  - _Requirements: 1, 3d, 3e_

- [x] 2.7 コードブロック処理 (P)
  - CodeBlock ノード（``` ... ``` で囲まれたコード）をそのまま Lua 出力に含める
  - 言語識別子（lua, rune など）は無視
  - ブロック内容を構文変換なしで直接出力
  - _Requirements: 1, 3f_

## 3. レジストリ統合と状態管理

- [x] 3.1 SceneRegistry へのシーン登録処理 (P)
  - グローバルシーン定義（GlobalSceneScope）をシーン情報として SceneRegistry に登録
  - ローカルシーン定義（LocalSceneScope）をラベル情報として登録
  - グローバルシーン名の `_N` 番号付けロジック（定義順 0-indexed → 1-indexed）
  - _Requirements: 5_

- [x] 3.2 WordDefRegistry への単語登録処理 (P)
  - グローバル単語定義（＠単語：選択肢1, 選択肢2）を WordDefRegistry に登録
  - ローカル単語定義をローカルスコープの WordDefRegistry に登録
  - 属性（＆属性名：値）をパーサーから取得（トランスパイラー層では処理なし）
  - _Requirements: 5_

## 4. Error Handling と Span フォーマット

- [x] 4.1 TranspileError 列挙型と Display トレイト実装
  - IoError, InvalidAst, UndefinedScene, UndefinedWord, InvalidContinuation, StringLiteralError, TooManyLocalVariables, Unsupported 各エラー型を定義
  - Span 構造体に Display トレイト実装：`[L{start_line}:{start_col}-L{end_line}:{end_col}]` 形式
  - #[error] マクロで `{span}` プレースホルダーを自動展開
  - エラーメッセージの一貫性確保
  - _Requirements: 4_

- [x] 4.2 エラーハンドリングの統合
  - トランスパイル処理各ステップでの TranspileError 生成
  - Writer エラーの処理（IO エラー発生時の動作）
  - 参照エラー（UndefinedScene, UndefinedWord）をランタイム層での検証に委譲（トランスパイラーは出力継続）
  - _Requirements: 4_

## 5. インテグレーションテストと参照実装検証

- [x] 5.1 sample.pasta → Lua トランスパイル統合テスト (P)
  - sample.pasta を入力としてトランスパイラーを実行
  - 生成 Lua コードをコメント行で分割して比較
  - 参照実装 sample.lua との行ごとの整合性をチェック
  - コメント行は除外、コード内容の一致を検証
  - インデント調整差異を許容
  - テスト結果レポート（一致行数/総行数、不一致パターン分類）
  - _Requirements: 6_

- [x] 5.2 単位テストと部分的な機能検証
  - 各 generate_* メソッド（アクター、シーン、変数等）の単位テスト
  - StringLiteralizer の危険パターン判定テスト
  - Span Display フォーマットの単位テスト（[L行:列-L行:列] 形式）
  - エラー出力テスト（各 TranspileError タイプ）
  - _Requirements: 6_

- [x] 5.3 要件トレーサビリティ検証 (P)
  - sample.lua コメント内の Requirement 参照が妥当か確認
  - 各要件（1-6）に対応するコード出力が sample.lua に存在するか確認
  - 削除された要件2（comment_mode）への遺存参照がないか確認
  - _Requirements: 6_

## 6. ローカル変数数制限への対応と最適化

- [x] 6.1 do...end スコープ分離パターンの実装
  - アクター定義：各 do...end ブロックで ACTOR 変数を再利用
  - グローバルシーン定義：各 do...end ブロックで SCENE 変数を再利用
  - スコープ分離により約200個ローカル変数枠を確保
  - _Requirements: 1_

- [x] 6.2 var/save/act テーブル運用パターンの実装と検証
  - 各シーン関数内で var（ローカル）, save（グローバル）, act（アクター操作）の3テーブルを使用
  - ローカル変数数制限を回避するパターン設計
  - パフォーマンステスト（大規模 Pasta スクリプト）
  - _Requirements: 1, 3d_

