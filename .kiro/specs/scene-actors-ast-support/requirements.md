# 要件ドキュメント

## プロジェクト説明（入力）
grammar.pestに新しい文法`scene_actors_line`（グローバルシーンに登場するアクターの一覧を列挙する、例「％さくら、うにゅう＝１」）を追加したため、対応するAST構造の定義と実装を行う。スコープはpasta_coreのみで、pasta_luaはコンパイルエラーが出ない最低限の対応のみを行う。

## 導入
本仕様は、grammar.pestに追加された`scene_actors_line`文法に対応するAST構造を定義・実装する。この文法はグローバルシーン初期化フェーズ（`global_scene_init`）で使用され、シーンに登場するアクター（キャラクター）の一覧とオプションの番号を宣言する。

### 背景
- `scene_actors_line = { pad ~ actor_marker ~ actors ~ or_comment_eol }`
- `actors = _{ actors_item ~ ( comma_sep ~ actors_item )* ~ comma_sep? }`
- `actors_item = { id ~ ( s ~ set_marker ~ s ~ digit_id )? }`
- 例：`　％さくら、うにゅう＝２`（アクター「さくら」と「うにゅう」を番号2で宣言）

### 前提条件
- **grammar.pestは検証済みの憲法** - 文法定義は完成・検証済み、変更対象外
- **パース動作確認済み** - 「　％さくら、うにゅう＝２」が正常にパース可能
- **スコープ制限** - `scene_actors_line`は`global_scene_init`の文脈でのみ登場可能

### スコープ
- **対象**: pasta_core（AST定義、パーサー変換）
- **対象外**: pasta_lua（コンパイルエラー回避の最低限対応のみ）

## 要件

### 要件1: SceneActorItem AST型定義
**目的:** パーサー開発者として、`actors_item`文法に対応するAST型を定義したい。これにより、パース結果を型安全に表現できるようになる。

#### 受け入れ基準
1. The `SceneActorItem` shall アクター名（`String`）と任意の番号（`Option<u32>`）を保持する
2. The `SceneActorItem` shall ソース位置情報（`Span`）を保持する

#### 設計決定（議題1で確定）
- `SceneActorLine`型は定義しない（中間型不要）
- パース関数は1行単位で`Vec<SceneActorItem>`を返す
- `GlobalSceneScope.actors`には複数行の結果を統合して保持

### 要件2: GlobalSceneScopeへの統合
**目的:** パーサー開発者として、`SceneActorLine`を`GlobalSceneScope`に統合したい。これにより、シーンレベルのアクター宣言にアクセスできるようになる。

#### 受け入れ基準
1. The `GlobalSceneScope` shall `actors`フィールド（`Vec<SceneActorItem>`）を保持する
2. When grammar.pestの`global_scene_init`で`scene_actors_line`が出現した時, pasta_core shall 対応する`GlobalSceneScope.actors`にアクター項目を追加する
3. When 複数の`scene_actors_line`が同一シーンに存在する時, pasta_core shall すべてのアクター項目を`actors`フィールドに蓄積する

### 要件3: パーサー変換実装
**目的:** パーサー開発者として、Pest パースペア から AST ノードへの変換ロジックを実装したい。これにより、文法定義とASTが正しく連携する。

#### 受け入れ基準
1. When Pestが`scene_actors_line`ルールをパースした時, pasta_core shall 1行分の`Vec<SceneActorItem>`を生成する
2. When Pestが`actors_item`ルールをパースした時, pasta_core shall `SceneActorItem`型に変換する
3. When `actors_item`に`set_marker`と`digit_id`が含まれる時, pasta_core shall `SceneActorItem.number`に対応する数値を設定する
4. If `actors_item`に`digit_id`が含まれない時, pasta_core shall `SceneActorItem.number`を`None`に設定する
5. The pasta_core shall 各ASTノードに正確な`Span`情報を設定する

### 要件4: pasta_lua最低限対応
**目的:** pasta_lua開発者として、新しいAST型によるコンパイルエラーを回避したい。これにより、pasta_coreの変更がpasta_luaのビルドを破壊しない。

#### 受け入れ基準
1. When pasta_coreに`SceneActorLine`/`SceneActorItem`が追加された時, pasta_lua shall コンパイルエラーなくビルドできる
2. The pasta_lua shall `GlobalSceneScope.actors`フィールドを無視（スキップ）してトランスパイルを継続する
3. The pasta_lua shall 新しいAST型に対して警告（unused）を発生させない

### 要件5: テスト
**目的:** 品質保証担当者として、新機能が正しく動作することを検証したい。これにより、リグレッションを防止できる。

#### 受け入れ基準
1. The pasta_core shall `scene_actors_line`のパーステストを持つ
2. When `％さくら、うにゅう＝１`をパースした時, pasta_core shall アクター「さくら」（番号なし）と「うにゅう」（番号1）を正しく抽出する
3. When 複数アクターを含む行をパースした時, pasta_core shall すべてのアクターを順序通りに保持する
4. When `cargo test --all`を実行した時, すべてのテスト（既存＋新規）が成功する
