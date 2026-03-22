# リサーチ & 設計判断

## サマリー
- **機能**: `optional-action-local-scene`
- **ディスカバリー分類**: Extension（軽量ディスカバリー）
- **主要な知見**:
  - 変更はPest文法ファイルの2文字のみ（`+` → `*`）
  - パーサー（AST変換）、トランスパイラ（Lua生成）、ランタイム（Lua実行）の全レイヤーが空の `Vec<LocalSceneItem>` を安全に処理済み
  - インテグレーションテスト2件の追加で全要件をカバー可能

## リサーチログ

### Pest文法の `+` → `*` 変更の影響範囲
- **背景**: `local_scene_item+` が1個以上必須のため、空ローカルシーンがパースエラーになる
- **調査対象**: `grammar.pest` L240-241
- **知見**:
  - `local_start_scene_scope` と `local_scene_scope` の2ルールが対象
  - `+`（1個以上）を `*`（0個以上）に変更するだけで文法的に空シーンを許容
  - `code_scope*` は既に0個以上なので変更不要
- **影響**: Pestパーサーが空の `inner()` イテレータを返すようになる

### パーサー（parse_scene.rs）の安全性
- **背景**: Pestの `pair.into_inner()` が空になった場合にパニックしないか確認
- **調査対象**: `crates/pasta_dsl/src/parser/parse_scene.rs` L183-285
- **知見**:
  - `parse_local_start_scene_scope()` と `parse_local_scene_scope()` は `for inner in pair.into_inner()` ループでアイテムを処理
  - 空イテレータに対して `for` ループは単にスキップされる（0回実行）
  - `LocalSceneScope` は `items: Vec::new()` で初期化済みなので空リストは正当な状態
- **影響**: コード変更不要

### トランスパイラ（scope_gen.rs）の安全性
- **背景**: 空のitemsリストでLuaコード生成がパニックしないか確認
- **調査対象**: `crates/pasta_lua/src/code_gen/scope_gen.rs` L185-310
- **知見**:
  - `generate_local_scene_items()` はTCO判定に `items.last().is_some_and()` を使用 → 空リストでは `false` を返す（安全）
  - `items.len().saturating_sub(1)` は空リストで `0` を返す（安全）
  - `for (index, item) in items.iter().enumerate()` は空リストで0回実行（安全）
- **影響**: コード変更不要。空シーンは関数本体が空のLua関数を生成

### ランタイム（act.lua / scene.lua）の安全性
- **背景**: 空のシーン関数が呼び出された場合の動作確認
- **調査対象**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` L335-400, `scene.lua` L140-185
- **知見**:
  - `ACT_IMPL.call()` は `find_scene()` で5段階検索してハンドラを取得
  - 空シーン関数は `init_scene` + 空のbody + `end` で構成され、呼び出し→即return
  - `SCENE.register()` は関数オブジェクトを登録するため、空関数でも正常に登録・検索・実行可能
- **影響**: コード変更不要

## 設計判断

### 判断: Pest文法のみ変更（Option A）
- **背景**: 空ローカルシーンを許容する最小変更を選択
- **検討した選択肢**:
  1. Option A — `local_scene_item+` → `local_scene_item*`（Pest文法のみ変更）
  2. Option B — 追加のバリデーション層で空シーンに警告を出す
- **選択**: Option A
- **根拠**: ギャップ分析により全レイヤーが既に空リストを安全に処理していることを確認済み。追加のバリデーションは不要な複雑性を導入する
- **トレードオフ**: 空シーンに対する警告がないためユーザーが意図せず空シーンを作成する可能性があるが、これは有用なユースケース（プレースホルダ分岐など）であるため許容
- **フォローアップ**: LSPでの空シーン警告は将来的な検討事項（スコープ外）

## リスク & 緩和策
- **リスク**: グローバルシーンまで空を許容してしまう → **緩和**: グローバルシーンの文法（`global_scene_scope`）は変更しない。`local_start_scene_scope` のみが `*` になるため、グローバルシーン自体には最低1つのローカルスタートスコープが必要（既存制約維持）
- **リスク**: 既存テストのスナップショットが壊れる → **緩和**: 文法の緩和は厳密なスーパーセットであり、既存の有効な入力は全て引き続き有効。スナップショット変更は発生しない

## 参考資料
- [grammar.pest](crates/pasta_dsl/src/parser/grammar.pest) L240-241 — 変更対象のPestルール
- [parse_scene.rs](crates/pasta_dsl/src/parser/parse_scene.rs) L183-285 — パーサーAST変換処理
- [scope_gen.rs](crates/pasta_lua/src/code_gen/scope_gen.rs) L185-310 — Luaコード生成処理
- [act.lua](crates/pasta_lua/pasta_scripts/pasta/act.lua) L335-400 — ランタイム呼び出し処理
