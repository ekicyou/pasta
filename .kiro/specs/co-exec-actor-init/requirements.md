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
2. `％` 行なし → スポットトークン未生成 → `BUILDER.build()` は `STORE.actor_spots`（前回の状態）をそのまま使う → **初回シーン実行時は `STORE.actor_spots` が空のため `actor_spots[name]` が nil → デフォルト 0 にフォールバック**

**問題**: `STORE.actor_spots` が空（初回起動直後、または `clear_spot` 後にリセットされた場合）のとき、`％` なしシーンでは明示的なスポット設定がないため、すべてのアクターがスポット 0 にフォールバックする。これ自体は動作するが、`pasta.toml` で定義したスポット設定が無視される。

## Requirements

### Requirement 1: `％` 省略時のスコープ継承（全シーン共通）

**Objective:** ゴースト作者として、`％` 行を省略したシーンでも直前のスコープ状態が自動的に引き継がれてほしい。これにより、全シーンに `％` を記述する冗長性を排除し、直感的な辞書記述を実現する。

#### Acceptance Criteria

1. When `％` 行を持たないシーンが実行されたとき, pasta shall 直前のシーンで確定したスコープ状態（`STORE.actor_spots`）を引き継いでアクタースポット解決に使用する
2. When SHIORIセッション開始後の最初のシーン実行で `％` 行がないとき, pasta shall `pasta.toml` の `[actor]` セクションで定義された `spot` 値を初期スコープとして適用する
3. When `％` 行を持つシーンが実行されたとき, pasta shall 従来通り `clear_spot()` + `set_spot()` によるスポット再設定を行い、`STORE.actor_spots` を更新する（既存動作の維持）
4. The pasta runtime shall イベント経路（直接SHIORI / `co_exec` 経由）に関わらず、同一のスコープ継承ルールを適用する

### Requirement 2: 初期スコープの自動設定

**Objective:** ゴースト作者として、`pasta.toml` の `[actor]` 定義がセッション開始時に自動的にスコープに反映されてほしい。これにより、OnBoot 等の最初のシーンでも `％` 行なしで正しいスポット配置が得られる。

#### Acceptance Criteria

1. When SHIORIセッションが開始された（load完了）とき, pasta shall `pasta.toml` の `[actor]` セクションの `spot` 値を `STORE.actor_spots` に自動設定する
2. If `pasta.toml` の `[actor]` セクションに `spot` 値が定義されていないアクターが存在する場合, pasta shall そのアクターの初期スポットを `0` とする
3. While 初期スコープが設定された状態で, pasta shall 最初のシーン実行で `％` 行がなくても正しいスポット位置でさくらスクリプトを生成する

### Requirement 3: co_exec 経由とSHIORI直接経由の動作一貫性

**Objective:** ゴースト作者として、シーンの呼び出し経路によって動作が変わらないことを保証してほしい。直接SHIORIイベント（OnBoot等）でも `co_exec` 経由の仮想イベント（OnTalk/OnHour）でも、同じシーンが同じ結果を返すべき。

#### Acceptance Criteria

1. When `％` 行なしのシーンが `SCENE.co_exec()` 経由で実行されたとき, pasta shall 直接SHIORIイベントで同一シーンを実行した場合と同一のスコープ解決結果を返す
2. When `virtual_dispatcher` が `SCENE.co_exec("OnTalk", nil, nil)` を呼び出したとき, pasta shall `act` オブジェクトの `actors` テーブルに `STORE.actors` の全アクターが設定された状態でシーン関数を実行する
3. If `SCENE.co_exec()` 経由で実行されたシーンが `act.アクター名` を参照した場合, pasta shall nil ではなく有効なアクタープロキシを返す（`STORE.actors` に当該アクターが存在する限り）

### Requirement 4: `％` 行欠落時の診断支援

**Objective:** ゴースト作者として、`％` 行の省略が意図的かどうかを判別しやすくしたい。特にデバッグ時に、スコープ継承の状況をログで確認できるべき。

#### Acceptance Criteria

1. When `％` 行なしのシーンが実行され、スコープ継承が行われたとき, pasta shall `tracing::debug!` レベルで継承元のスコープ状態をログ出力する
2. If `％` 行なしのシーンで参照されたアクター名が `STORE.actors` に存在しない場合, pasta shall `tracing::warn!` レベルで未定義アクター参照を警告する
3. The pasta runtime shall `％` 行の有無自体をパースエラーや警告としない（省略は合法であり、スコープ継承として処理される）
