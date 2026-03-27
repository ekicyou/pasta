# Requirements Document

## Introduction

OnSecondChange経由で自動発行されるOnTalk/OnHourの仮想ディスパッチャにおいて、SSP Statusヘッダのブロック条件が不十分であり、他イベント（OnUpdateComplete等）のトーク出力が上書きされる問題がある。SSP拡張仕様のStatus値を網羅的にチェックし、ゴーストが不適切なタイミングで喋らないようブロック条件を強化する。

### 参考仕様

- SSP SHIORI/3.0 Status [SSP拡張]: https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html
- 対象モジュール: `pasta.shiori.event.virtual_dispatcher`

### 現行のブロック条件（ギャップ）

現在 `check_hour()` と `check_talk()` は以下のStatusのみチェックしている:
- `talking` — トーク中
- `choosing` — 選択肢表示中

以下のSSP Status値がブロック条件に**含まれていない**:
- `online` — ネットワーク通信中（更新チェック等）
- `opening(...)` — 入力ボックス等が開いている
- `passive` — パッシブモード中
- `induction` — インダクションモード中
- `timecritical` — タイムクリティカルセクション中
- `nouserbreak` — ユーザーブレイク禁止中
- `minimizing` — 最小化中（バルーン非表示）

## Requirements

### Requirement 1: dispatchレベルの集約ブロックガード

**Objective:** ゴースト開発者として、OnTalk/OnHourの仮想ディスパッチャが不適切なタイミングで発火しないようにしたい。そうすれば、OnUpdateComplete等の他イベントトークが上書きされなくなる。

#### Acceptance Criteria

1. When `dispatch(act)` が呼ばれた時, the virtual_dispatcher shall `act.req.status` に以下のいずれかのキーワードが含まれる場合 `nil` を返して発行をブロックする: `talking`, `choosing`, `online`, `opening`, `passive`, `induction`, `timecritical`, `nouserbreak`, `minimizing`

### Requirement 2: check_hour / check_talk 個別チェックの廃止

**Objective:** ゴースト開発者として、ブロック条件の重複管理を排除し、将来のStatus追加時に1箇所だけ修正すれば済むようにしたい。

#### Acceptance Criteria

1. The virtual_dispatcher shall dispatch関数の入口で一括ブロック判定を行い、`check_hour()` および `check_talk()` 内の個別Status判定（talking/choosing）を除去する
2. The virtual_dispatcher shall ブロック判定のStatus一覧をモジュールローカルのテーブル（`local BLOCKED_STATUSES`）として一元管理する

### Requirement 3: テストカバレッジ

**Objective:** 開発者として、全ブロック条件が正しく機能することをテストで保証したい。

#### Acceptance Criteria

1. When 各ブロック対象Status（talking, choosing, online, opening, passive, induction, timecritical, nouserbreak, minimizing）が `act.req.status` に設定された場合, the テスト shall `dispatch()` が `nil` を返すことを検証する
2. When 複数Statusがカンマ区切りで設定された場合（例: `"choosing,balloon(0=0)"`）, the テスト shall ブロック対象が含まれていれば `nil` を返すことを検証する
3. When `act.req.status` が `nil` または空文字列の場合, the テスト shall ブロックされないことを検証する
4. When `M.is_blocked(status)` を直接呼び出した場合, the テスト shall 各ブロック対象Statusで `true` を返し、非ブロック対象で `false` を返すことを検証する

### Requirement 4: スキルドキュメント更新

**Objective:** LLMエージェントとして、ブロック条件の仕様変更をスキルリファレンスに反映し、将来の辞書制作やコード生成で正しい情報を参照できるようにしたい。

#### Acceptance Criteria

1. The shiori-handlers.md shall virtual_dispatcherセクションの記述を更新し、ブロック対象Status一覧を記載する
2. The shiori-handlers.md shall `dispatch()` の記述に「Statusブロックガードは `dispatch()` 入口で一括判定される」旨を追記する
3. The shiori-handlers.md shall `M.is_blocked(status)` の使用例（他イベントアルゴリズムへの応用例）を記載する

### Requirement 5: ブロック判定の汎用公開API化

**Objective:** ゴースト開発者（Luaスクリプト作者）として、SSP Statusブロック判定を他のイベントアルゴリズム（例: 撫で反応、OnMouseDoubleClick等）でも再利用できるようにしたい。

#### Acceptance Criteria

1. The virtual_dispatcher shall `M.is_blocked(status)` を公開関数として提供し、他モジュールから `require` して呼び出せるようにする
2. The virtual_dispatcher shall `dispatch()` 内のブロック判定を `M.is_blocked(act.req.status)` に委譲する
3. When `M.is_blocked(status)` が呼ばれた時, the function shall `BLOCKED_STATUSES` の全キーワードを評価し、1つでも一致すれば `true`、いずれも一致しなければ `false` を返す
