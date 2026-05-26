# Requirements Document

## Introduction

本仕様は、pastaゴーストフレームワークにおける「トーク合成中のSHIORI非同期通信」機能を定義する。SSPの `\![get,property,...]` プロトコルを利用したプロパティ値の読み取りには、さくらスクリプトタグ発行 → SSPコールバック受信 → 処理再開という非同期ラウンドトリップが必要だが、現行の yield/resume 基盤にはこのパターンを透過的に処理する仕組みがない。

本機能により、ゴースト作者はトーク合成中に `act:get_property(name)` を呼び出すだけでSSPプロパティ値を取得でき、非同期通信の複雑さはフレームワーク内部に完全に隠蔽される。また、基盤は `\![get,...]` 系タグ全般に再利用可能な汎用設計とする。

## Boundary Context

- **In scope**:
  - トーク合成中（コルーチン実行中）でのSSPプロパティ読み取りAPI（`act:get_property()`）
  - コールバックイベントの自動ルーティング（イベント受信 → 待機中コルーチンへの値引き渡し）
  - 汎用的な非同期SHIORI通信メカニズム（プロパティ以外の `\![get,...]` パターンにも再利用可能）
  - コールバック未着時のエラーハンドリング
  - 入力バリデーション
- **Out of scope**:
  - `act:set_property()` — `property-write-helpers` specで実装済み
  - Pasta DSL構文拡張 — `property-dsl-extension` specの範囲
  - プロパティ値の型変換（文字列として返す）
  - `%property[name]` 環境変数展開（`get_property` が上位互換）
  - Rust側（`pasta_shiori`、`pasta_lua` src/）の変更
- **Adjacent expectations**:
  - 既存の yield/resume フロー（`STORE.co_scene` による通常チェーントーク）が本機能導入後も同一の動作を維持すること
  - `property-write-helpers` specの `act:set_property(name, value)` と対称的なAPIデザイン（引数バリデーション、エスケープ処理の一貫性）
  - `shiori-event-test-framework` specのモック基盤（SHIORIリクエスト/レスポンスサイクルのモック）を使用してテスト可能であること

## Requirements

### Requirement 1: 汎用非同期通信基盤

**Objective:** フレームワーク開発者として、プロパティ取得以外の `\![get,...]` 系タグ（将来の機能拡張）にも同じ非同期通信メカニズムを再利用したい。これにより、新しい非同期パターンの追加時にフレームワーク全体を改修する必要がなくなる。

#### Acceptance Criteria
1. The pasta framework shall プロパティ取得に限定されない汎用的なコールバック待機・ルーティング機構を提供する（特定のイベントIDを指定して待機し、そのイベント受信時に値を受け取って再開するパターンが `get_property` 以外からも利用可能であること）
2. When コールバックを登録する, the pasta framework shall 登録APIの引数としてタイムアウト絶対時刻とエラー理由文字列（`on_timeout`、nilまたは文字列）を受け取る（コールバック種別ごとに適切なタイムアウトと動作モードを設定できるようにするため、登録API自体にデフォルト値は設けない）
3. When 新しい `\![get,...]` 系コンシューマを追加する場合, the pasta framework shall 既存のコールバック待機・ルーティング機構を変更することなく、新しいコンシューマの追加のみで対応可能とする

#### モジュール配置

| モジュール            | ファイルパス                      | 責務                                                               |
| --------------------- | --------------------------------- | ------------------------------------------------------------------ |
| コールバック管理      | `pasta/shiori/event/callback.lua` | コールバック登録・ルーティング・タイムアウト sweep                 |
| get_property メソッド | `pasta/shiori/act.lua`            | `SHIORI_ACT_IMPL` への `get_property` メソッド追加（コンシューマ） |

- `callback.lua` は `event/` 配下に配置する。`EVENT.fire()` でのコールバックイベント割り込み、`OnSecondChange` での sweep、`REG` への動的登録がすべて既存の `event/` モジュールパターンに合致するため。
- `get_property` は `pasta/shiori/act.lua` の `SHIORI_ACT_IMPL` メタテーブルにメソッドとして追加する。base act を SHIORI 固有メソッドで拡張する既存パターンに従う。

### Requirement 2: 単一プロパティ取得

**Objective:** ゴースト作者として、トーク合成中に `act:get_property(name)` を呼び出してSSPプロパティ値を文字列として取得したい。これにより、ベースウェア情報やゴースト状態に基づく動的なトーク生成が可能になる。

#### Acceptance Criteria
1. When ゴースト作者がトーク合成中（シーンのコルーチン実行中）に `act:get_property(name)` を呼び出す, the pasta framework shall SSPにプロパティ取得リクエストを発行し、コールバックで受け取ったプロパティ値を文字列として返す
2. When `act:get_property(name)` が呼び出される, the pasta framework shall 呼び出し元コルーチンの一時停止とSSPコールバック受信時の自動再開を透過的に処理する（ゴースト作者がyield/resumeを意識する必要がないこと）
3. When `act:get_property(name)` が値を返した後, the pasta framework shall 同一シーン内で後続のトークン生成や追加の `get_property` 呼び出しを正常に継続可能とする

### Requirement 3: 複数プロパティ一括取得

**Objective:** ゴースト作者として、`act:get_property({name1, name2, ...})` で複数のSSPプロパティを一度に取得したい。これにより、関連する複数のプロパティ値を効率的にまとめて取得できる。

#### Acceptance Criteria
1. When ゴースト作者が `act:get_property({name1, name2, ...})` を配列で呼び出す, the pasta framework shall 各プロパティに対応する値を配列順に多値（multiple return values）として返す
2. When 複数プロパティ取得において存在しないプロパティ名が含まれる, the pasta framework shall 該当する戻り値位置に `nil` を返し、他のプロパティ値は正常に返す

#### 想定Luaコードと戻り値マッピング

```lua
local width, height = act:get_property({
    "currentghost.balloon.scope(0).validwidth.initial",
    "currentghost.balloon.scope(0).validheight.initial",
})
act:raw_script("width=" .. width .. " height=" .. height)
```

フレームワークが発行するさくらスクリプト:
```
\![get,property,OnPastaCallBack1,currentghost.balloon.scope(0).validwidth.initial,currentghost.balloon.scope(0).validheight.initial]
```

SSPコールバック時の Reference マッピング:
| 引数順 | プロパティ名                                        | Reference  | Lua戻り値             |
| ------ | --------------------------------------------------- | ---------- | --------------------- |
| 1      | `currentghost.balloon.scope(0).validwidth.initial`  | Reference0 | 第1戻り値（`width`）  |
| 2      | `currentghost.balloon.scope(0).validheight.initial` | Reference1 | 第2戻り値（`height`） |

存在しないプロパティが含まれる場合（例: `act:get_property({"valid.prop", "nonexistent.prop"})`）、SSPは該当Referenceを空文字列で返す。フレームワークはこれを `nil` に変換して返す。

### Requirement 4: 入力バリデーション

**Objective:** ゴースト作者として、不正な引数で `get_property` を呼び出した場合に明確なエラーメッセージを受け取りたい。これにより、辞書開発中のデバッグが容易になる。

#### Acceptance Criteria
1. When `act:get_property()` が引数なしで呼び出される, the pasta framework shall エラーを発生させ、プロパティ名が必要であることを示すメッセージを提供する
2. When `act:get_property(name)` の `name` に `nil` または空文字列が渡される, the pasta framework shall エラーを発生させ、プロパティ名が無効であることを示すメッセージを提供する
3. When `act:get_property(name)` がコルーチン実行コンテキスト外で呼び出される, the pasta framework shall エラーを発生させ、トーク合成中のみ使用可能であることを示すメッセージを提供する

### Requirement 5: コールバック未着時のエラーハンドリング

**Objective:** ゴースト作者として、SSPからの応答が届かない場合にトークが永久にフリーズせず、予測可能な結果を得たい。

#### タイムアウト検出メカニズム

コールバック登録時に、呼び出し元（コンシューマ）がタイムアウト絶対時刻（`os.time()` + タイムアウト秒数）を引数として渡す。`OnSecondChange` イベント処理時にコールバックモジュールが全ペンディングエントリを掃引（sweep）し、現在時刻がタイムアウト時刻を超過したエントリの待機コルーチンをエラー値で再開する。

タイムアウト値はコールバック種別ごとに異なる（例: `get_property` = 5秒、将来の選択肢コールバック = 数分）。コールバック登録API自体はタイムアウトのデフォルト値を持たず、各コンシューマが登録時に明示的に指定する。

#### タイムアウト時の動作モード

コールバック登録時に、タイムアウト時の動作を制御する**エラー理由文字列**（`on_timeout`）を引数として渡す:

- **`on_timeout = "callback timeout: get_property"`**（文字列指定）: タイムアウト時に `SHIORI/3.0 500 Internal Server Error` + `X-ERROR-REASON: <指定文字列>` をレスポンスとして返し、待機コルーチンをエラー値で再開する。ログにも記録する。
- **`on_timeout = nil`**（省略 / nil指定）: タイムアウト時にコールバック登録を静かに削除し、待機コルーチンを `nil` で再開する。エラーレスポンスは返さない。

`get_property` はデフォルトでエラー理由文字列を設定する（ゴースト作者がデバッグ時に原因を特定できるようにするため）。将来の選択肢コールバック等は、ユーザーが選ばなかった場合を正常系として扱いたいケースがあるため、静かに消える（`nil`）モードが適切。

#### Acceptance Criteria
1. When コールバックが登録される, the pasta framework shall 呼び出し元から指定されたタイムアウト絶対時刻とエラー理由文字列（`on_timeout`、nilまたは文字列）をエントリに記録する
2. When `OnSecondChange` イベント処理時に現在時刻がタイムアウト時刻を超過し、`on_timeout` が文字列のコールバックが存在する, the pasta framework shall `SHIORI/3.0 500` + `X-ERROR-REASON: <on_timeout>` をレスポンスとして返し、待機コルーチンをエラー値で再開し、ログ（`@pasta_log.warn`）に記録する
3. When `OnSecondChange` イベント処理時に現在時刻がタイムアウト時刻を超過し、`on_timeout` が nil のコールバックが存在する, the pasta framework shall コールバック登録を静かに削除し、待機コルーチンを `nil` で再開する（エラーレスポンスもログも出力しない）
4. When `get_property` がコールバックを登録する, the pasta framework shall デフォルトタイムアウト5秒 + エラー理由 `"callback timeout: get_property"` を適用する（ゴースト作者が第 2・第 3 引数で上書き可能: `act:get_property("name", 10, "custom reason")`）

### Requirement 6: 既存フローとの互換性

**Objective:** ゴースト作者として、本機能の導入によって既存のチェーントーク（`act:yield()` による継続トーク）や通常のイベントハンドリングが壊れないことを期待する。

#### Acceptance Criteria
1. While 本機能が有効な状態で, the pasta framework shall 既存の `act:yield()` による通常チェーントーク（次イベントでの再開）が従来どおり動作すること
2. While 本機能が有効な状態で, the pasta framework shall `get_property` を使用しないシーンやイベントハンドラの動作に一切の変更がないこと
3. While コールバック待機中のコルーチンが存在する状態で, the pasta framework shall コールバック対象でない通常イベント（`OnSecondChange`、`OnTalk` 等）の処理を阻害しないこと

## SHIORI プロトコルレベルシナリオ

以下のシナリオは、要件の受入基準を SHIORI プロトコルレベルで具体化したものである。設計・テストの判定基準として使用する。

### Scenario 1: 単純なプロパティ取得（Req 2）

```lua
local version = act:get_property("baseware.version")
act:raw_script("baseware.version: " .. version)
```

```
--- Round 1: 起点イベント ---
SSP → SHIORI:  GET SHIORI/3.0
               ID: OnTest

SHIORI → SSP:  SHIORI/3.0 200 OK
               Value: \![get,property,OnPastaCallBack1,baseware.version]

--- Round 2: コールバック ---
SSP → SHIORI:  GET SHIORI/3.0
               ID: OnPastaCallBack1
               Reference0: 2.6.77

SHIORI → SSP:  SHIORI/3.0 200 OK
               Value: baseware.version: 2.6.77\e
```

- Round 1 の Value に `\e` は付与しない（コールバック待ちレスポンス）
- コールバックルーティングは `create_act` より前で分岐し、元コルーチンの act を使って続行する

### Scenario 2: トーク蓄積後のプロパティ取得（Req 2）

```lua
act:talk(actor, "調べています...")
local version = act:get_property("baseware.version")
act:talk(actor, version .. "です")
```

```
--- Round 1 ---
SSP → SHIORI:  GET SHIORI/3.0
               ID: OnTest

SHIORI → SSP:  SHIORI/3.0 200 OK
               Value: \p[0]調べています...\![get,property,OnPastaCallBack1,baseware.version]

--- Round 2 ---
SSP → SHIORI:  GET SHIORI/3.0
               ID: OnPastaCallBack1
               Reference0: 2.6.77

SHIORI → SSP:  SHIORI/3.0 200 OK
               Value: \p[0]2.6.77です\e
```

- Round 1 で蓄積済みトークンと get タグが同一 Value に含まれる
- バルーンに「調べています...」が表示された状態でコールバックを待機

### Scenario 3: チェーントーク → コールバック待ちへの遷移（Req 2 + Req 6）

```lua
act:talk(actor, "起動しました")
act:yield()                                        -- 通常チェーントーク
local ver = act:get_property("baseware.version")   -- コールバック待ち
act:talk(actor, "v" .. ver)
```

```
--- Round 1: OnBoot ---
SSP → SHIORI:  NOTIFY SHIORI/3.0
               ID: OnBoot

SHIORI → SSP:  SHIORI/3.0 200 OK
               Value: \p[0]起動しました\e

  ※ act:yield() → 通常チェーントーク → STORE.co_scene = co

--- Round 2: チェーントーク再開 ---
SSP → SHIORI:  NOTIFY SHIORI/3.0
               ID: OnSecondChange

  ※ STORE.co_scene 再開 → get_property → コールバック待ち yield
  ★ 同一コルーチン内で yield 種別が「チェーントーク」から「コールバック待ち」に遷移

SHIORI → SSP:  SHIORI/3.0 200 OK
               Value: \![get,property,OnPastaCallBack1,baseware.version]

  ※ STORE.co_scene はクリア（コールバック待ちコルーチンは別レジストリで管理）

--- Round 3: コールバック ---
SSP → SHIORI:  GET SHIORI/3.0
               ID: OnPastaCallBack1
               Reference0: 2.6.77

SHIORI → SSP:  SHIORI/3.0 200 OK
               Value: \p[0]v2.6.77\e
```

- Round 1 の yield は通常チェーントーク（`STORE.co_scene` に格納）
- Round 2 でコルーチン再開後、`get_property` のyieldは**コールバック待ち**（`STORE.co_scene` ではなくコールバックレジストリで管理）
- **yield 種別の遷移判定**が最大の設計課題

### Scenario 4: コールバック前に無関係イベントが到着（Req 6 AC3）

```
--- Round 1 ---
SSP → SHIORI:  GET SHIORI/3.0
               ID: OnTest

SHIORI → SSP:  SHIORI/3.0 200 OK
               Value: \![get,property,OnPastaCallBack1,baseware.version]

--- Round 2: 無関係イベント ---
SSP → SHIORI:  NOTIFY SHIORI/3.0
               ID: OnSecondChange

  ※ pending callback: OnPastaCallBack1 ≠ OnSecondChange → 不一致
  ※ REG.OnSecondChange を通常ディスパッチ（コールバック待ちコルーチンに触れない）

SHIORI → SSP:  SHIORI/3.0 204 No Content

--- Round 3: コールバック到着 ---
SSP → SHIORI:  GET SHIORI/3.0
               ID: OnPastaCallBack1
               Reference0: 2.6.77

SHIORI → SSP:  SHIORI/3.0 200 OK
               Value: \p[0]Version: 2.6.77\e
```

- 無関係イベントはコールバック待ちコルーチンに一切影響しない
- 通常の REG ディスパッチが正常に動作すること
