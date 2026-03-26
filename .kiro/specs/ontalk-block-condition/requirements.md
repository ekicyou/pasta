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

## Requirements

### Requirement 1: dispatchレベルの集約ブロックガード

**Objective:** ゴースト開発者として、OnTalk/OnHourの仮想ディスパッチャが不適切なタイミングで発火しないようにしたい。そうすれば、OnUpdateComplete等の他イベントトークが上書きされなくなる。

#### Acceptance Criteria

1. When `dispatch(act)` が呼ばれた時, the virtual_dispatcher shall `act.req.status` に以下のいずれかのキーワードが含まれる場合 `nil` を返して発行をブロックする: `talking`, `choosing`, `online`, `opening`, `passive`, `induction`, `timecritical`, `nouserbreak`

### Requirement 2: check_hour / check_talk 個別チェックの廃止

**Objective:** ゴースト開発者として、ブロック条件の重複管理を排除し、将来のStatus追加時に1箇所だけ修正すれば済むようにしたい。

#### Acceptance Criteria

1. The virtual_dispatcher shall dispatch関数の入口で一括ブロック判定を行い、`check_hour()` および `check_talk()` 内の個別Status判定（talking/choosing）を除去する
2. The virtual_dispatcher shall ブロック判定のStatus一覧を公開テーブル（`M.blocked_statuses`）として一元管理する

### Requirement 3: ブロックStatusのカスタマイズ可能性

**Objective:** ゴースト開発者として、特定のStatus（例: online中でも喋らせたい）をオーバーライドできるようにしたい。

#### Acceptance Criteria

1. The virtual_dispatcher shall ブロック対象のStatusキーワードリストをテーブルとして公開し、ゴースト開発者が `scripts/` でエントリを追加・削除できるようにする
2. When ゴースト開発者がブロックリストからキーワードを除去した場合, the virtual_dispatcher shall そのStatusではブロックしなくなる

### Requirement 4: minimizingの扱い

**Objective:** ゴースト開発者として、最小化中はバルーンが見えないため、トークを発行しても無意味にならないようにしたい。

#### Acceptance Criteria

1. When `act.req.status` に `minimizing` が含まれる場合, the virtual_dispatcher shall `nil` を返して発行をブロックする（デフォルト動作）
2. The virtual_dispatcher shall `minimizing` をブロックリストテーブルに含め、Requirement 3のカスタマイズ手段で除外可能とする

### Requirement 5: テストカバレッジ

**Objective:** 開発者として、全ブロック条件が正しく機能することをテストで保証したい。

#### Acceptance Criteria

1. When 各ブロック対象Status（talking, choosing, online, opening, passive, induction, timecritical, nouserbreak, minimizing）が `act.req.status` に設定された場合, the テスト shall `dispatch()` が `nil` を返すことを検証する
2. When 複数Statusがカンマ区切りで設定された場合（例: `"choosing,balloon(0=0)"`）, the テスト shall ブロック対象が含まれていれば `nil` を返すことを検証する
3. When `act.req.status` が `nil` または空文字列の場合, the テスト shall ブロックされないことを検証する
4. When ブロックリストからキーワードを除去した場合, the テスト shall そのStatusでブロックされなくなることを検証する

### Requirement 6: スキルドキュメント更新

**Objective:** LLMエージェントとして、ブロック条件の仕様変更をスキルリファレンスに反映し、将来の辞書制作やコード生成で正しい情報を参照できるようにしたい。

#### Acceptance Criteria

1. The shiori-handlers.md shall virtual_dispatcherセクションの記述を更新し、ブロック対象Status一覧とカスタマイズ方法を記載する
2. The shiori-handlers.md shall `dispatch()` の記述に「Statusブロックガードは `dispatch()` 入口で一括判定される」旨を追記する
