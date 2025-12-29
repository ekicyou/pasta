# 調査・設計決定ログ

## 概要
- **機能名**: `pasta-lua-specification`
- **Discovery スコープ**: 拡張機能（Extension）- 既存 Rune トランスパイラーパターンをベースにした Lua トランスパイラー層
- **主要発見事項**:
  1. pasta_rune CodeGenerator パターン（`std::io::Write` トレイト）は Lua 出力にそのまま適用可能
  2. 2パス戦略（レジストリ登録 → コード生成）は Lua でも維持
  3. Lua 5.3+ の Unicode 識別子サポートにより日本語シーン名が直接使用可能

## 調査ログ

### Rune トランスパイラー実装パターン
- **コンテキスト**: 既存 pasta_rune のコード生成パターンを Lua 向けに適用可能か調査
- **参照ソース**: `crates/pasta_rune/src/transpiler/code_generator.rs`
- **発見事項**:
  - `CodeGenerator<W: Write>` 構造体でジェネリック Writer を使用
  - `write_indent()`, `writeln()`, `write_raw()` ヘルパーメソッド
  - `indent_level` による階層管理
  - `current_module` によるシーン解決コンテキスト保持
  - ローカルシーン命名: `format!("{}_{}", sanitized, index + 1)` - 常に 1 から開始
- **設計への影響**: Lua トランスパイラーでも同一の Writer パターンを採用

### Lua 文字列リテラル形式
- **コンテキスト**: Rune の文字列エスケープと Lua の長文字列形式の違いを調査
- **参照ソース**: Lua 5.3 Reference Manual、sample.lua
- **発見事項**:
  - Rune: `"text".replace('\\', "\\\\").replace('"', "\\\"")` でエスケープ
  - Lua 通常文字列: `"text"` - `\` と `"` のエスケープが必要
  - Lua 長文字列: `[[text]]`, `[=[text]=]`, `[==[text]==]` - エスケープ不要
  - 危険パターン: `]` + `n個の=` が含まれるとそのレベルは使用不可
- **設計への影響**: Requirement 3 のアルゴリズム（ルール1→ルール2判定）で実装

### Lua ローカル変数制限
- **コンテキスト**: Lua の 200 ローカル変数制限への対応策を調査
- **参照ソース**: sample.lua、Lua 5.3 Reference Manual
- **発見事項**:
  - Lua は関数あたり最大約 200 ローカル変数
  - `do...end` ブロックでスコープ分離可能
  - 同一ブロック内で `local` を再宣言すると新しいスロットを消費
  - テーブル（`var`, `save`, `act`）を使えば 3 つのローカルで多数の変数を管理可能
- **設計への影響**: Requirement 1 の do...end スコープ分離パターンを適用

### pasta_core パーサー AST 構造
- **コンテキスト**: トランスパイラーが受け取る AST ノード構造を確認
- **参照ソース**: `crates/pasta_core/src/parser/ast.rs`
- **発見事項**:
  - `GlobalSceneScope`: グローバルシーン定義、`local_scenes` と `code_blocks` を保持
  - `LocalSceneScope`: ローカルシーン定義、`items` リストを保持
  - `LocalSceneItem`: `VarSet`, `CallScene`, `ActionLine`, `ContinueAction` の enum
  - `Action`: `Talk`, `WordRef`, `VarRef`, `FnCall`, `SakuraScript`, `Escape` の enum
  - `VarScope`: `Local`, `Global`, `Args(usize)` の enum
  - **актерの型**: `ActorScope` （`name: String`, `attrs: HashMap<String, String>`, `words: Vec<WordScope>`, `var_sets: Vec<VarSet>` を保持）
  - **Span 情報**: すべての AST ノードに `span: Span` を含有（`start_line`, `start_col`, `end_line`, `end_col` で行番号位置を記録）
- **設計への影響**: 既存 AST 構造をそのまま利用し、Lua 出力パターンのみ変更。Span 情報により `comment_mode=true` 時に行番号コメント出力が可能

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク/制限 | 備考 |
|----------|------|------|-------------|------|
| Visitor パターン | AST を走査し各ノードで Lua 出力を生成 | 明確な責務分離、テスト容易 | 複雑な状態管理が必要な場合に困難 | Rune 実装と同等 |
| Template エンジン | テンプレートから Lua コードを生成 | 読みやすいテンプレート | 動的ロジックが複雑になる | 採用せず |
| 直接 String 生成 | 文字列連結で Lua コードを構築 | シンプル | 大規模で管理困難 | 採用せず |

**選択**: Visitor パターン（Write トレイトによる直接出力）

## 設計決定

### 決定: pasta_lua クレート構成
- **コンテキスト**: Lua トランスパイラーを pasta_rune と並列に配置する必要がある
- **代替案**:
  1. pasta_rune 内に Lua 出力を追加（既存クレート拡張）
  2. pasta_lua として新規クレート作成
- **選択アプローチ**: 新規 pasta_lua クレート作成
- **理由**: Rune 依存を持たない純粋な Lua 出力層が必要。pasta_core のみに依存
- **トレードオフ**: コードの重複リスクあり。共通パターンは pasta_core に移動検討
- **フォローアップ**: 共通ユーティリティ（文字列リテラル判定など）は後続で pasta_core に移動可能

### 決定: 2パス変換戦略の維持
- **コンテキスト**: Rune では Pass 1 でレジストリ登録、Pass 2 でコード生成
- **代替案**:
  1. 1パス変換（登録と生成を同時）
  2. 2パス維持
- **選択アプローチ**: 2パス維持
- **理由**: シーン間参照解決に前方参照が必要。1パスでは未定義シーンの参照が解決不能
- **トレードオフ**: 若干のパフォーマンスオーバーヘッド
- **フォローアップ**: Pass 1 はレジストリ登録のみ、Pass 2 で Lua コード生成

### 決定: コメント出力モードの実装方式
- **コンテキスト**: Pasta 源コードをトレースするデバッグ機能
- **代替案**:
  1. コンストラクタでフラグ設定
  2. 各メソッドにフラグを渡す
  3. 環境変数で制御
- **選択アプローチ**: コンストラクタでフラグ設定（`comment_mode: bool`）
- **理由**: 一度の設定でトランスパイル全体に適用。シンプルな API
- **コメント出力形式**:
  - `comment_mode=true` 時、AST ノードの `span.start_line` からファイルレベルの行番号を取得
  - 行番号取得不可な場合（古い AST など）はコメント出力をスキップ
  - コメント形式: `-- [Pasta src:L○]` で生成行の Pasta ソース行番号を記録
- **トレードオフ**: 部分的なコメント出力制御は不可能
- **フォローアップ**: デフォルト false。テストでは両モードを検証

### 決定: シーン呼び出しメカニズム - ジャンプテーブル不要化
- **コンテキスト**: pasta_rune では シーン ID → 関数へのジャンプテーブルをハードコード出力
- **代替案**:
  1. pasta_rune と同様にジャンプテーブル出力（ID 管理が必要）
  2. Lua テーブル（連想配列）を参照するだけ（ID 不要）
- **選択アプローチ**: テーブル参照形式（ID 不要）
- **理由**: 
  - Lua の動的テーブル参照で十分に機能
  - ID 番号管理の複雑性を排除
  - シーン追加時に ID 再割り当ての必要なし
  - テーブルキー（シーン名文字列）で直感的
- **実装分離**:
  - **トランスパイラー（pasta_lua）**: シーン定義テーブル構造 + CallScene → act:call() 出力
  - **ランタイム層（後続仕様）**: テーブル参照 + 実行制御 + word/talk 関数実装
- **トレードオフ**: テーブル参照の若干のランタイムオーバーヘッド（Lua の場合無視できる）

### 決定: Pass 1 統一設計 - 参照検証をランタイム層に委譲
- **コンテキスト**: Lua テーブル参照により、トランスパイラー層での前方参照検証が不要に
- **代替案**:
  1. Pass 1 で参照検証、Pass 2 でコード生成（pasta_rune パターン）
  2. Pass 1 で機械的に Lua コード生成（Lua テーブル特性を活用）
- **選択アプローチ**: Pass 1 で機械的に Lua コード生成
- **理由**:
  - Lua テーブルは実行時に存在確認できるため、トランスパイラーで検証不要
  - 前方参照エラー（UndefinedScene, UndefinedWord）はランタイム層で処理
  - トランスパイラーは変換のみ、参照検証なし
- **例**:
  ```lua
  -- Pasta DSL: @メイン:会話（$カウンタ）
  -- Lua 出力 (トランスパイラー、引数をそのまま展開):
  act:call("メイン", "会話", {}, var.カウンタ)
  ```
- **ランタイム層の責務**:
  - act:call() 実行時に scenes テーブル存在確認
  - 参照エラー時は Lua エラーハンドリング
- **トレードオフ**: トランスパイラー層でのエラー検出ができず、ランタイム層でのエラーになる

## リスクと軽減策
- **リスク 1**: Lua 5.3 未満での Unicode 識別子非サポート → Lua 5.3+ を必須要件に明記
- **リスク 2**: 長文字列形式の無限ループ（すべての n で危険パターン発生）→ 実用上 n=10 で十分、上限設定
- **リスク 3**: sample.lua との不一致によるテスト失敗 → インテグレーションテストで Lua 構文的同一性を検証
- **リスク 4**: ランタイム層での テーブル検証エラー → トランスパイラーの Pass 1 で参照検証済みのため、ランタイムでの追加検証は予防的
- **リスク 5**: CallScene の引数変換複雑性 → 各 Action 処理ルール（VarRef, FnCall など）を design.md に明記
  - コメント行（`--` で開始）と空行（`^\\s*$`）は比較対象外
  - インデント量の差異も許容（構文木レベルで同等か検証）
  - トランスパイラールール微調整による出力変更は、テスト側で期待値を調整
  - sample.lua 自体は原著なので修正しない

## 参考資料
- [Lua 5.3 Reference Manual - Strings](https://www.lua.org/manual/5.3/manual.html#3.1) - 長文字列形式の仕様
- [pasta_rune code_generator.rs](crates/pasta_rune/src/transpiler/code_generator.rs) - 参照実装パターン
- [sample.lua](../../pasta-lua-specification/sample.lua) - Lua 出力の参照実装
