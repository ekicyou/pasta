# Requirements Document

## Project Description (Input)
作者が「このシーンを今すぐ再生して観たい」（デバッグ／オーサリング）という要求を満たす手段が現状存在しない。Phase 5 のデバッグ位置ブレークでは代替できず、求められているのは任意シーンの**即時再生キック（テスト再生）**である。

`pasta-actor-runtime` 完了後を前提とし、宿主非依存コア＋アクタースレッド（`wintf_winmsg_executor`）＋ CH marshaling（GET=block-on-reply／NOTIFY=即204／drop・timeout→204）が存在する。出力は `pasta_shiori` アダプタが presentation event stream をさくらスクリプトへ描画し、debug backend は DAP-over-TCP で VSCode と接続済みである。ライブ SSP がプレビューを兼ねられる構造のため、別プレビュー画面は不要である。

本機能は **即時再生オンリー** として、talk FIFO ＋ OnSecondChange 無条件 drain、即時 preempt-and-abort（進行中会話を中断し前 `co_scene` を閉じ自動復帰しない）、エンジンによるシーン実行コンテキスト（ctx）の合成（通常トーク再生と同一手順を流用）、VSCode 拡張からのキックコマンド、既存 debug DAP チャネルの一般化（`playScene` custom request）、debug backend のアクタークライアント化を提供する。SSP `Status` による抑制ゲートや非即時（アイドル待ち）モードは設けない。

## Introduction

本仕様 **pasta-scene-kick** は、ゴースト作者がオーサリング／デバッグ中に「任意のシーンを今すぐライブ SSP 上で再生して観る」ことを可能にする。作者は VSCode 拡張のコマンド／ボタンからシーンを指名してキックし、本物のゴーストが実 SSP 上で反応する様子を ≤1 秒で確認できる。

キックは SHIORI/3.0 の pull 契約（OnSecondChange の GET 機会）を破らずに実現する。キックされたシーンはアクター（executor）スレッドで非同期にレンダリングされ、talk FIFO に積まれる。OnSecondChange の GET は FIFO を drain して返すだけに保ち、GET ブロックを短く維持する。SSP からの押し出し（SSTP / `\![raise]`）は用いない。

キックは**即時再生の単一モード**である。テスト再生という用途上、会話中（SSP `Status: talking`）であっても問答無用で再生する：進行中の会話を preempt（中断）し、中断された側の前 `co_scene` を閉じ、自動復帰しない（デバッグキックは礼儀正しいキューではない、preempt-and-abort）。`Status` を権威とする抑制ゲートや、アイドルまで待つ非即時モードは設けない。

キックされたシーンは SHIORI イベント由来の `act`（実行コンテキスト）を持たないため、エンジンが当該シーンを起動するための ctx を合成して与える。ctx の合成は**通常のトーク再生時に行っている合成手順をそのまま流用**する（キック専用の特別な ctx 構築は行わない）。外部 SHIORI の通常会話挙動はキックによって変化させない（キックは追加経路である）。

## Boundary Context

- **In scope（本機能が責務を持つ）**:
  - talk FIFO（キックされたシーンのレンダリング結果を順序保持で蓄積）。
  - OnSecondChange での無条件 drain（SSP `Status` による抑制を行わない）。
  - 即時 preempt-and-abort（進行中会話を中断し、前 `co_scene` を閉じ、自動復帰しない）— 唯一の再生挙動。
  - エンジンによるシーン実行コンテキスト（ctx／`act`）の合成（通常トーク再生と同一手順を流用）。
  - VSCode 拡張からのキックコマンド／ボタン（シーン名の指名）。
  - キック transport の一般化（既存 debug DAP チャネルへ `playScene` custom request を追加）。
  - debug backend をアクターのクライアントとして扱う（キックとデバッグを単一制御面に統合）。
  - キックの executor スレッド上での非同期実行・レンダリング → FIFO 投入。
- **Out of scope（本機能が所有しない）**:
  - 非即時（アイドル待ち）キックモード、および SSP `Status: talking` を権威とする抑制ゲート（即時再生オンリーのため不採用）。
  - 即時キックされた会話の退避→復帰セマンティクス（採用するのは preempt-and-abort）。
  - VSCode／キック要求からのシーン引数・アクター指定（ctx 合成はエンジン既定で行い、UI からの引数指定は将来別境界）。
  - ライブ SSP 以外の出力先（別プレビュー画面）。
  - SSTP / `\![raise]` によるライブ押し出し出力（別境界 `pasta-sstp-live-output`・将来）。
  - `*.pasta` 編集ウィンドウからのキック（別境界 `pasta-authoring-window`・将来）。
  - `pasta_novel` アダプタ。
- **Adjacent expectations（隣接システム／仕様への期待）**:
  - **`pasta-actor-runtime`（上流）**: アクタースレッド、CH marshaling、presentation event stream、さくらスクリプト描画アダプタを提供する。本機能はその上に talk FIFO とキック経路を追加する。
  - **`pasta-vscode-lua-debug`（拡張対象・上流）**: DAP-over-TCP チャネルと VSCode 接続を提供する。本機能はそのチャネルに `playScene` custom request を追加し、debug backend をアクタークライアント化する。
  - **`pasta-debug-lua-view-toggle`（隣接・前例）**: DAP custom request ＋ VSCode コマンドの実装前例（`pasta/sourcePresentation`）。本機能のキックコマンドは同じ前例に倣う。
  - **通常トーク再生経路（流用元）**: シーン実行用 ctx（`act`）の合成手順を提供する。本機能のキックは同一手順を流用して ctx を合成する。
  - **SSP（外部宿主）**: SHIORI/3.0 `Status` ヘッダの権威。OnSecondChange の tick 周期が配信レイテンシを規定する。

## Requirements

### Requirement 1: VSCode 拡張からのシーンキック起動
**Objective:** ゴースト作者として、VSCode 拡張のコマンド／ボタンから任意のシーンを指名してキックしたい。それにより、エディタを離れずに「このシーンを今すぐ観る」を実行できる。

#### Acceptance Criteria
1. When 作者が VSCode 拡張のシーンキックコマンドを実行する, the VSCode 拡張 shall キック対象のシーン名を作者から受け取る。
2. When 作者がシーン名を指定してキックを確定する, the VSCode 拡張 shall そのシーン名を含むキック要求をデバッグチャネル経由でエンジンへ送信する。
3. If デバッグセッションが接続されていない状態でキックが実行される, then the VSCode 拡張 shall キックを送信せずエラーまたは案内を作者に提示する。
4. If 作者がシーン名を指定せずキックを取り消す, then the VSCode 拡張 shall キック要求を送信しない。

### Requirement 2: キック transport（debug DAP チャネルの一般化）
**Objective:** 開発者として、既存の debug DAP チャネルを再利用してキック要求を運びたい。それにより、キックとデバッグを単一の制御面に統合し、新たな別チャネルを増やさずに済む。

#### Acceptance Criteria
1. When VSCode 拡張がキック要求を送信する, the エンジン shall 既存 debug DAP チャネル上の `playScene` custom request としてそれを受理する。
2. When `playScene` custom request を受理する, the エンジン shall 要求からキック対象のシーン名を抽出する。
3. The エンジン shall キック要求の受理を、別の独立した transport を新設することなく既存 debug チャネルの拡張として行う。
4. While debug backend が有効である, the エンジン shall debug backend をアクターのクライアントとして扱い、キック要求をアクターへ取り次ぐ。
5. If 受理した `playScene` 要求のシーン名が空または不正である, then the エンジン shall キックを実行せずエラーを要求元へ返す。
6. Where debug backend が既定で無効である, the エンジン shall debug backend が無効な間はキック経路を有効化しない。

### Requirement 3: キックの非同期実行・ctx 合成・レンダリング
**Objective:** 開発者として、キックされたシーンを GET ブロックを延ばさずに実行・レンダリングしたい。またキックは SHIORI イベント由来の `act` を持たないため、シーン実行に必要な ctx をエンジンが与えたい。それにより、SHIORI の pull 契約（短い GET）を守ったままシーンを起動しライブ SSP へ反映できる。

#### Acceptance Criteria
1. When エンジンがキック要求を受理する, the エンジン shall キック対象シーンの実行とレンダリングを、OnSecondChange の GET 応答を待たせずに非同期で行う。
2. When キックされたシーンを実行する, the エンジン shall そのシーンの実行に必要な実行コンテキスト（ctx／`act`）を、通常のトーク再生時と同一の合成手順を流用して構築し与える。
3. When キックされたシーンのレンダリングが完了する, the エンジン shall その出力（さくらスクリプト）を talk FIFO へ投入する。
4. The エンジン shall シーンのレンダリング処理を OnSecondChange の GET 応答の内側で同期実行せず、GET 応答を短く保つ。
5. If 指名されたシーンが存在しない／解決できない, then the エンジン shall そのキックを talk FIFO に何も投入せず破棄し、診断可能な形で記録する。

### Requirement 4: talk FIFO と OnSecondChange 無条件 drain
**Objective:** 開発者として、キック出力を順序保持の talk FIFO に蓄積し、OnSecondChange で抑制なく取り出したい。それにより、pull 契約を破らずに ≤1 秒でライブ SSP へ届けられる。

#### Acceptance Criteria
1. The talk FIFO shall キック由来の出力を投入順（FIFO）で保持する。
2. When OnSecondChange の GET を受信する, the エンジン shall talk FIFO を SSP `Status` による抑制なく無条件に drain し、取り出した出力を GET 応答として返す。
3. While talk FIFO が空である, when OnSecondChange の GET を受信する, the エンジン shall キック由来の出力を含まない通常の応答を返す。
4. When talk FIFO に出力が存在する, the エンジン shall 実 SSP の tick 周期に依存して概ね 1 秒以内にライブ SSP へ届くよう、当該 OnSecondChange の GET で出力を返す。
5. The エンジン shall talk FIFO の drain を OnSecondChange の pull 機会に限定し、SSTP / `\![raise]` による押し出しを行わない。

### Requirement 5: 即時 preempt-and-abort（唯一の再生挙動）
**Objective:** ゴースト作者として、テスト再生のキックは会話中でも問答無用で直ちに再生してほしい。それにより、進行中の会話に阻まれず観たいシーンを即座に確認できる。

#### Acceptance Criteria
1. When キック要求を受理し進行中の会話が存在する, the エンジン shall SSP が `talking` 等を報告していても、抑制ゲートを介さず当該会話を直ちに preempt（中断）する。
2. When 即時キックが進行中の会話を中断する, the エンジン shall 中断された側の前 `co_scene` を閉じる。
3. The エンジン shall 即時キックによって中断された会話を自動的に復帰させない（preempt-and-abort、退避→復帰を行わない）。
4. The エンジン shall キック再生を即時再生の単一モードとし、SSP `Status` を権威とする抑制待ち（非即時・アイドル待ち）モードを提供しない。

### Requirement 6: ライブ SSP プレビューと既存挙動の不変
**Objective:** ゴースト作者として、別のプレビュー画面ではなく本物のライブ SSP 上で反応を観たい。それにより、実環境のゴーストの振る舞いをそのまま確認できる。

#### Acceptance Criteria
1. The エンジン shall キック由来の出力をライブ SSP（実際のゴースト表示）へ反映し、別途プレビュー専用画面を導入しない。
2. The エンジン shall 外部 SHIORI の通常会話挙動（キックを行わない通常イベント処理）をキック経路の追加によって変更しない。
3. While キック経路が一切使用されない, the エンジン shall キック導入前と同一の通常 SHIORI 応答挙動を維持する。
4. The エンジン shall SHIORI/3.0 の `Status` ヘッダ準拠（解釈）を保ち、SSP との既存プロトコル整合を崩さない。
