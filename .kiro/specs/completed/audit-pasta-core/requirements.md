# 要件定義書

## 概要

pasta_coreクレートは、Pasta DSLの言語非依存レジストリ層であり、シーン管理（SceneRegistry / SceneTable）、単語管理（WordDefRegistry / WordTable）、ランダム選択（RandomSelector）、エラー型定義を提供する基盤クレートである。長期開発の中で脆弱性の体系的調査やコード簡素化が行われておらず、入力検証の不足、パニック安全性の懸念、デッドコード蓄積の可能性がある。

本仕様は、公開APIの外部振る舞いを完全に保持したまま、以下を達成する：
- 全脆弱性カテゴリ（入力検証、エラーハンドリング、パニック安全性）の調査と修正
- デッドコード除去および冗長表現削減によるコード量削減
- 既存テスト（950+）のリグレッションなし全パス

## 境界コンテキスト

- **対象**: `crates/pasta_core/src/` 配下の全ファイル（lib.rs、error.rs、registry/ 以下全モジュール）
- **対象外**: APIシグネチャの変更、新機能追加、他クレート（pasta_dsl, pasta_lua, pasta_shiori 等）への変更、`fast_radix_trie`の内部実装変更
- **隣接期待**: 下流クレート（pasta_dsl, pasta_lua, pasta_shiori）が依存するpasta_coreの公開API・振る舞いは不変であること

## 要件

### 要件 1: パニック安全性の確保

**目的:** pasta_coreの開発者として、全ての非テストコードパスでパニックが発生しないことを保証したい。これにより、呼び出し元クレートが予期しないプロセスクラッシュに遭遇しなくなる。

#### 受入基準
1. When `unwrap()` または `expect()` が非テストコードに存在する場合, the pasta_core shall 安全な代替（`Result`返却、デフォルト値使用、`if let`パターン等）に置換する
2. When 配列・Vecへのインデックスアクセスが存在する場合, the pasta_core shall 境界チェック付きアクセス（`.get()` 等）を使用するか、事前条件で安全性が証明済みであることをコメントで明記する
3. When キャッシュリセット後に `cache.get_mut()` を呼び出す場合, the pasta_core shall `unwrap()` ではなく安全なパターンで値を取得する
4. The pasta_core shall 非テストコード内に `unwrap()`、`expect()`、`panic!` を含まない

### 要件 2: 入力検証の強化

**目的:** pasta_coreの利用者として、不正な入力に対して明確なエラーが返されることを期待する。これにより、デバッグ時の問題特定が容易になる。

#### 受入基準
1. When 空文字列が検索キーとして渡された場合, the pasta_core shall 適切なエラーバリアントを返す（パニックせず）
2. When SceneRegistryに無効なシーン名（空文字列）が登録される場合, the pasta_core shall エラーを返すかまたは安全にハンドリングする
3. When `SceneId` の値がラベルVecの範囲外である場合, the pasta_core shall `None` を返すかまたはエラーを返す（パニックせず）
4. The pasta_core shall 全ての公開メソッドにおいて、無効な入力に対してパニックではなくエラーまたは安全なデフォルト値を返す

### 要件 3: デッドコード除去

**目的:** pasta_coreのメンテナとして、使用されていないコードを除去したい。これにより、コードベースの可読性と保守性が向上する。

#### 受入基準
1. When Rustコンパイラが `dead_code` 警告を出す関数・メソッド・型が存在する場合, the pasta_core shall それらを除去するか、使用されていることを確認する
2. When `#[allow(dead_code)]` アトリビュートが存在する場合, the pasta_core shall そのアトリビュートの妥当性を検証し、不要であれば除去する
3. When 未使用のインポート（`use`文）が存在する場合, the pasta_core shall それらを除去する
4. The pasta_core shall コンパイル時にpasta_coreクレート内のデッドコード警告が0件である

### 要件 4: 冗長表現の削減

**目的:** pasta_coreのメンテナとして、同一ロジックの重複実装を排除したい。これにより、変更時の修正箇所が減り、バグ混入リスクが低下する。

#### 受入基準
1. When `resolve_scene_id` と `resolve_scene_id_unified` に重複するキャッシュ処理ロジックが存在する場合, the pasta_core shall 共通ヘルパーに抽出して重複を排除する
2. When `collect_word_candidates` と `collect_scene_candidates` に類似したプレフィックス検索パターンが存在する場合, the pasta_core shall 共通化可能な部分を特定し、適切であれば共通化する
3. When 冗長なイテレータチェーン（不要な中間コレクション等）が存在する場合, the pasta_core shall より簡潔な表現に置換する
4. The pasta_core shall 重複する実装ロジックを最小限に抑える

### 要件 5: エラーハンドリングの改善

**目的:** pasta_coreの利用者として、エラーメッセージが一貫性を持ち、問題の診断に十分な情報を含むことを期待する。

#### 受入基準
1. When エラーメッセージに日本語と英語が混在している場合, the pasta_core shall エラーメッセージの言語を統一する
2. When `SceneTableError` に使用されていないバリアントが存在する場合, the pasta_core shall そのバリアントの使用状況を確認し、未使用であれば除去を検討する
3. The pasta_core shall 全てのエラーバリアントが `thiserror` の `#[error(...)]` アトリビュートで適切にフォーマットされている

### 要件 6: 外部振る舞いの不変性保証

**目的:** 下流クレートの開発者として、pasta_coreの監査後も全ての公開APIの振る舞いが変わらないことを保証したい。

#### 受入基準
1. The pasta_core shall 公開API（`pub` 関数・メソッド・型・トレイト）のシグネチャを変更しない
2. The pasta_core shall 既存テスト（`cargo test -p pasta_core` および全体テスト `cargo test`）が全パスする
3. When 内部リファクタリングを行った場合, the pasta_core shall 同一の入力に対して同一の出力を返す
4. The pasta_core shall コンパイルエラー0件かつ、新規の警告を導入しない

### 要件 7: 性能の維持

**目的:** pasta_coreの利用者として、監査後の実装が性能劣化していないことを保証したい。

#### 受入基準
1. When アルゴリズムを変更した場合, the pasta_core shall 変更前と同等以上の計算量オーダーを維持する
2. When 不要なクローン・アロケーションを発見した場合, the pasta_core shall 参照や借用で代替可能であれば置換する
3. The pasta_core shall 新たな不要なヒープアロケーションを導入しない
