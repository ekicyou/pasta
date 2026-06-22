# Requirements Document

## Project Description (Input)
作者が「このシーンを今すぐ再生して観たい」（デバッグ／オーサリング）という要求を満たす手段が現状存在しない。Phase 5 のデバッグ位置ブレークでは代替できず、求められているのは任意シーンの即時再生キックである。

`pasta-actor-runtime` 完了後を前提とし、宿主非依存コア＋アクタースレッド（`wintf_winmsg_executor`）＋ CH marshaling（GET=block-on-reply／NOTIFY=即204／drop・timeout→204）が存在する。出力は `pasta_shiori` アダプタが presentation event stream をさくらスクリプトへ描画し、debug backend は DAP-over-TCP で VSCode と接続済みである。ライブ SSP がプレビューを兼ねられる構造のため、別プレビュー画面は不要である。

本機能は、talk FIFO ＋ OnSecondChange drain と、SSP `Status: talking` を権威とする抑制（gate）、即時キックの preempt-and-abort、非即時キックのアイドル待ち、VSCode 拡張からのキックコマンド、既存 debug DAP チャネルの一般化（`playScene` custom request）、debug backend のアクタークライアント化を提供する。

## Introduction

本仕様 **pasta-scene-kick** は、ゴースト作者がオーサリング／デバッグ中に「任意のシーンを今すぐライブ SSP 上で再生して観る」ことを可能にする。作者は VSCode 拡張のコマンド／ボタンからシーンを指名してキックし、本物のゴーストが実 SSP 上で反応する様子を ≤1 秒で確認できる。

キックは SHIORI/3.0 の pull 契約（OnSecondChange の GET 機会）を破らずに実現する。キックされたシーンはアクター（executor）スレッドで非同期にレンダリングされ、talk FIFO に積まれる。OnSecondChange の GET は FIFO を drain して返すだけに保ち、GET ブロックを短く維持する。SSP からの押し出し（SSTP / `\![raise]`）は用いない。

キックには 2 つのモードがある。**非即時キック**は会話中（SSP `Status: talking`）であれば FIFO で待ち、アイドルになってから吐く。**即時キック**は `talking` でも問答無用で FIFO を消費し、進行中の会話を preempt（中断）する。preempt では中断された側の前 `co_scene` を閉じ、自動復帰しない（デバッグキックは礼儀正しいキューではない、preempt-and-abort）。

抑制の権威は SSP の `Status: talking` ヘッダであり、エンジン内部の推測ではなく SSP が報告する状態を正とする。外部 SHIORI の通常会話挙動はキックによって変化させない（キックは追加経路である）。

## Boundary Context

- **In scope（本機能が責務を持つ）**:
  - talk FIFO（キックされたシーンのレンダリング結果を順序保持で蓄積）。
  - `Status: talking` を権威とした drain ゲート（会話中は通常 FIFO を消費しない）。
  - 即時キックの preempt-and-abort（進行中会話を中断し、前 `co_scene` を閉じ、自動復帰しない）。
  - 非即時キックのアイドル待ち（`talking` が消えてから吐く）。
  - VSCode 拡張からのキックコマンド／ボタン（シーン指名）。
  - キック transport の一般化（既存 debug DAP チャネルへ `playScene` custom request を追加）。
  - debug backend をアクターのクライアントとして扱う（キックとデバッグを単一制御面に統合）。
  - キックの executor スレッド上での非同期実行・レンダリング → FIFO 投入。
- **Out of scope（本機能が所有しない）**:
  - ライブ SSP 以外の出力先（別プレビュー画面）。
  - SSTP / `\![raise]` によるライブ押し出し出力（別境界 `pasta-sstp-live-output`・将来）。
  - `*.pasta` 編集ウィンドウからのキック（別境界 `pasta-authoring-window`・将来）。
  - `pasta_novel` アダプタ。
  - 即時キックされた会話の退避→復帰セマンティクス（採用するのは preempt-and-abort）。
  - 礼儀正しいキュー復帰セマンティクス全般。
- **Adjacent expectations（隣接システム／仕様への期待）**:
  - **`pasta-actor-runtime`（上流）**: アクタースレッド、CH marshaling、presentation event stream、さくらスクリプト描画アダプタを提供する。本機能はその上に talk FIFO とキック経路を追加する。
  - **`pasta-vscode-lua-debug`（拡張対象・上流）**: DAP-over-TCP チャネルと VSCode 接続を提供する。本機能はそのチャネルに `playScene` custom request を追加し、debug backend をアクタークライアント化する。
  - **`pasta-debug-lua-view-toggle`（隣接・前例）**: DAP custom request ＋ VSCode コマンドの実装前例（`pasta/sourcePresentation`）。本機能のキックコマンドは同じ前例に倣う。
  - **SSP（外部宿主）**: SHIORI/3.0 `Status` ヘッダ（`talking` 等）の権威。OnSecondChange の tick 周期が配信レイテンシを規定する。

## Requirements

### Requirement 1: VSCode 拡張からのシーンキック起動
**Objective:** ゴースト作者として、VSCode 拡張のコマンド／ボタンから任意のシーンを指名してキックしたい。それにより、エディタを離れずに「このシーンを今すぐ観る」を実行できる。

#### Acceptance Criteria
1. When 作者が VSCode 拡張のシーンキックコマンドを実行する, the VSCode 拡張 shall キック対象のシーン名を作者から受け取る。
2. When 作者がシーン名を指定してキックを確定する, the VSCode 拡張 shall そのシーン名を含むキック要求をデバッグチャネル経由でエンジンへ送信する。
3. Where 即時キックと非即時キックの双方が提供される, the VSCode 拡張 shall 作者がどちらのモードでキックするかを選択できる手段を提供する。
4. If デバッグセッションが接続されていない状態でキックが実行される, then the VSCode 拡張 shall キックを送信せずエラーまたは案内を作者に提示する。
5. If 作者がシーン名を指定せずキックを取り消す, then the VSCode 拡張 shall キック要求を送信しない。

### Requirement 2: キック transport（debug DAP チャネルの一般化）
**Objective:** 開発者として、既存の debug DAP チャネルを再利用してキック要求を運びたい。それにより、キックとデバッグを単一の制御面に統合し、新たな別チャネルを増やさずに済む。

#### Acceptance Criteria
1. When VSCode 拡張がキック要求を送信する, the エンジン shall 既存 debug DAP チャネル上の `playScene` custom request としてそれを受理する。
2. When `playScene` custom request を受理する, the エンジン shall 要求からシーン名とキックモード（即時／非即時）を抽出する。
3. The エンジン shall キック要求の受理を、別の独立した transport を新設することなく既存 debug チャネルの拡張として行う。
4. While debug backend が有効である, the エンジン shall debug backend をアクターのクライアントとして扱い、キック要求をアクターへ取り次ぐ。
5. If 受理した `playScene` 要求のシーン名が空または不正である, then the エンジン shall キックを実行せずエラーを要求元へ返す。
6. Where debug backend が既定で無効である, the エンジン shall debug backend が無効な間はキック経路を有効化しない。

### Requirement 3: キックの非同期実行とレンダリング
**Objective:** 開発者として、キックされたシーンを GET ブロックを延ばさずに実行・レンダリングしたい。それにより、SHIORI の pull 契約（短い GET）を守ったままライブ SSP へ反映できる。

#### Acceptance Criteria
1. When エンジンがキック要求を受理する, the エンジン shall キック対象シーンの実行とレンダリングを、OnSecondChange の GET 応答を待たせずに非同期で行う。
2. When キックされたシーンのレンダリングが完了する, the エンジン shall その出力（さくらスクリプト）を talk FIFO へ投入する。
3. The エンジン shall シーンのレンダリング処理を OnSecondChange の GET 応答の内側で同期実行せず、GET 応答を短く保つ。
4. If 指名されたシーンが存在しない／解決できない, then the エンジン shall そのキックを talk FIFO に何も投入せず破棄し、診断可能な形で記録する。

### Requirement 4: talk FIFO と OnSecondChange drain
**Objective:** 開発者として、キック出力を順序保持の talk FIFO に蓄積し OnSecondChange で取り出したい。それにより、pull 契約を破らずに ≤1 秒でライブ SSP へ届けられる。

#### Acceptance Criteria
1. The talk FIFO shall キック由来の出力を投入順（FIFO）で保持する。
2. When OnSecondChange の GET を受信する, the エンジン shall talk FIFO を drain して取り出した出力を GET 応答として返す。
3. While talk FIFO が空である, when OnSecondChange の GET を受信する, the エンジン shall キック由来の出力を含まない通常の応答を返す。
4. When 抑制状態でなく talk FIFO に出力が存在する, the エンジン shall 実 SSP の tick 周期に依存して概ね 1 秒以内にライブ SSP へ届くよう、当該 OnSecondChange の GET で出力を返す。
5. The エンジン shall talk FIFO の drain を OnSecondChange の pull 機会に限定し、SSTP / `\![raise]` による押し出しを行わない。

### Requirement 5: 抑制ゲート（`Status: talking` 権威）
**Objective:** ゴースト作者として、会話中は通常のキックが進行中の会話に割り込まないでほしい。それにより、観たいシーン以外でゴーストの現在の会話を不用意に壊さずに済む。

#### Acceptance Criteria
1. While SSP の `Status` ヘッダに `talking` が含まれる, the エンジン shall 非即時キック由来の talk FIFO を通常 drain しない（会話に割り込まない）。
2. The エンジン shall 抑制の判定を SSP が報告する `Status` ヘッダを権威として行い、エンジン内部の推測で代替しない。
3. While SSP の `Status` に `talking` が含まれない（アイドル）, when talk FIFO に非即時キック出力が存在する, the エンジン shall 当該 OnSecondChange の GET で当該出力を drain して返す。
4. When `talking` が消えてアイドルへ遷移する, the エンジン shall 待機していた非即時キック出力を後続の OnSecondChange で吐く。

### Requirement 6: 非即時キック（アイドル待ち）
**Objective:** ゴースト作者として、急がないキックは会話中なら待ってアイドルで吐いてほしい。それにより、進行中の会話を壊さずに観たいシーンを後で確認できる。

#### Acceptance Criteria
1. When 非即時モードのキックを受理し SSP が `talking` を報告している, the エンジン shall その出力を talk FIFO に積み、アイドルになるまで drain を保留する。
2. While 非即時キック出力が talk FIFO で待機中であり SSP が `talking` を報告し続ける, the エンジン shall その出力を吐かずに保持し続ける。
3. When SSP がアイドル（`talking` なし）になる, the エンジン shall 待機していた非即時キック出力を後続の OnSecondChange で FIFO 順に吐く。

### Requirement 7: 即時キック（preempt-and-abort）
**Objective:** ゴースト作者として、デバッグ目的の即時キックは会話中でも問答無用で再生してほしい。それにより、進行中の会話に阻まれず観たいシーンを直ちに確認できる。

#### Acceptance Criteria
1. When 即時モードのキックを受理する, the エンジン shall SSP が `talking` を報告していても当該キック由来の出力を talk FIFO から drain する（抑制ゲートを問わない）。
2. When 即時キックが進行中の会話を中断する, the エンジン shall 中断された側の前 `co_scene` を閉じる。
3. The エンジン shall 即時キックによって中断された会話を自動的に復帰させない（preempt-and-abort、退避→復帰を行わない）。
4. When 即時キックの出力が talk FIFO に投入された, the エンジン shall 当該出力を後続の OnSecondChange の GET で drain して返す。

### Requirement 8: ライブ SSP プレビューと既存挙動の不変
**Objective:** ゴースト作者として、別のプレビュー画面ではなく本物のライブ SSP 上で反応を観たい。それにより、実環境のゴーストの振る舞いをそのまま確認できる。

#### Acceptance Criteria
1. The エンジン shall キック由来の出力をライブ SSP（実際のゴースト表示）へ反映し、別途プレビュー専用画面を導入しない。
2. The エンジン shall 外部 SHIORI の通常会話挙動（キックを行わない通常イベント処理）をキック経路の追加によって変更しない。
3. While キック経路が一切使用されない, the エンジン shall キック導入前と同一の通常 SHIORI 応答挙動を維持する。
4. The エンジン shall SHIORI/3.0 の `Status` ヘッダ準拠（`talking` 等の解釈）を保ち、SSP との既存プロトコル整合を崩さない。
