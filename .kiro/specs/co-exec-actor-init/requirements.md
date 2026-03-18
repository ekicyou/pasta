# Requirements Document

## Project Description (Input)

バグレポート: co_exec経由の仮想イベントで `％` アクター宣言なしのシーンがアクター未初期化になる。

Pasta DSL の `％`（アクター宣言）行を持たないシーンが、直接SHIORIイベント（OnBoot等）では正常動作するが、`SCENE.co_exec()` 経由の仮想イベント（OnTalk/OnHour）では `act.アクター名` が nil になりシーン実行が失敗する。

**トップレベル要件**: `％` 設定が無い場合、直前のトークのスコープを引き継ぐべき。これは全トークに共通する要件とする。

## 現状分析

### スコープ状態のグローバル管理（現在の実装）

| レイヤー | 変数/テーブル | 役割 | ライフサイクル |
|----------|-----------|------|-------------|
| `STORE.actors` | `CONFIG.actor` のコピー | pasta.toml `[actor]` のアクター定義全体 | SHIORIセッション中永続 |
| `STORE.actor_spots` | `{[name]: spot_id}` | アクターごとのスポット位置マップ | シーン間で永続（`BUILDER.build()` が更新を書き戻す） |

### イベント経路別の初期化フロー

| 経路 | `act` 生成 | スポット初期化 |
|------|-----------|--------------|
| 直接SHIORI（`EVENT.fire`） | `SHIORI_ACT.new(STORE.actors, req)` → `act.actors` = `STORE.actors` | `％` がなくても `STORE.actor_spots` が `BUILDER.build()` に渡される |
| co_exec経由（`virtual_dispatcher`） | 同一の `act` が `coroutine.resume(co, act)` で渡される | `％` がなければ `clear_spot`/`set_spot` トークンが未生成 → スポット位置は `STORE.actor_spots` から引き継がれるが、**アクター切り替え検出でスポットが不正になる可能性がある** |

### 根本原因の構造

1. `％` 行あり → トランスパイラが `act:clear_spot()` + `act:set_spot("名前", N)` を生成 → スポットトークンが token stream に入る → `BUILDER.build()` が `actor_spots` を再構築
2. `％` 行なし → スポットトークン未生成 → `BUILDER.build()` は `STORE.actor_spots`（前回の状態）をそのまま使う → スコープ継承として正しい動作

**設計原則**: 全トークは最終的に `BUILDER.build()` を経由してさくらスクリプトに変換される。この関数に適切なスコープ情報（`actor_spots`）を注入し、変更されたスコープ情報を外部で正しく維持する設計は既に機能している。

**真の問題**: Rust 側の `register_config_module`（`module_registry.rs`）が `[actor]` セクション配下のサブテーブルを TOML → Lua 変換する際、テーブルのキー名（アクター名）を値テーブル内の `name` フィールドとして注入していない。そのため CONFIG由来アクター（`STORE.actors["さくら"]` = `{spot=0}`）に `name` フィールドが欠落し、`BUILDER.build()` 内部で `actor_name = actor.name` → `nil` となって、`actor_spots[nil]` で正しいスポット値を参照できない。データ提供側（Rust）で修正する（ディスカッションで合意済み）。

## Requirements

### Requirement 1: `％` 省略時のスコープ継承と正しいアクター解決（全シーン共通）

**Objective:** ゴースト作者として、`％` 行を省略したシーンでも直前のスコープ状態が自動的に引き継がれ、イベント経路によらず正しく動作してほしい。これにより、全シーンに `％` を記述する冗長性を排除し、直感的な辞書記述を実現する。

**設計原則**: 全トークは最終的に `BUILDER.build()` を経由してさくらスクリプトに変換される。この関数は渡された `actor_spots` テーブルを直接変更し、スクリプト文字列のみを返す。テスト再現性は入力テーブルの制御で担保する。

#### Acceptance Criteria

1. When `％` 行を持たないシーンが実行されたとき, pasta shall 直前のシーンで確定したスコープ状態（`STORE.actor_spots`）を引き継いでアクタースポット解決に使用する
2. When SHIORIセッション開始後の最初のシーン実行で `％` 行がないとき, pasta shall `pasta.toml` の `[actor]` セクションで定義された `spot` 値を初期スコープとして適用する
3. When `％` 行を持つシーンが実行されたとき, pasta shall 従来通り `clear_spot()` + `set_spot()` によるスポット再設定を行い、`STORE.actor_spots` を更新する（既存動作の維持）
4. The pasta runtime shall イベント経路（直接SHIORI / `co_exec` 経由）に関わらず、同一のスコープ継承ルールを適用する
5. When シーン関数内で `act.アクター名` を参照したとき, pasta shall `STORE.actors` に当該アクターが存在する限り、`name` フィールドを持つ有効なアクタープロキシを返す

> **統合経緯**: 旧 Req 2（初期スコープの自動設定）は store.lua の既存実装（CONFIG.actor → STORE.actor_spots 転送）で充足済み。旧 Req 3（co_exec/SHIORI直接経由の動作一貫性）は本 Req の AC 4, 5 に統合。全ギャップが同一根本原因（CONFIG由来アクターの `name` フィールド欠落）に帰結するため、1つの要件に集約。

### Requirement 2: `％` 行欠落時の診断支援

**Objective:** ゴースト作者として、`％` 行の省略が意図的かどうかを判別しやすくしたい。特にデバッグ時に、スコープ継承の状況をログで確認できるべき。

#### Acceptance Criteria

1. When `％` 行なしのシーンが実行され、スコープ継承が行われたとき, pasta shall `tracing::debug!` レベルで継承元のスコープ状態をログ出力する
2. If `％` 行なしのシーンで参照されたアクター名が `STORE.actors` に存在しない場合, pasta shall `tracing::warn!` レベルで未定義アクター参照を警告する
3. The pasta runtime shall `％` 行の有無自体をパースエラーや警告としない（省略は合法であり、スコープ継承として処理される）
