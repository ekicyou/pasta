# Requirements Document

## Project Description (Input)
ここまでの議論を仕様にせよ。イベントハンドラからの「コール」は、「act:call()」と等価でなければならない。

## Introduction

現在のpastaランタイムでは、SHIORIイベントのディスパッチ経路（`EVENT.fire` → `REG[id]` → `EVENT.no_entry`）と、シーン関数内のコール経路（`act:call()`）が**異なる解決ロジック**を使用している。

- `act:call()` は多段フォールバック（ローカルシーン → スコープ付きシーン検索 → GLOBAL → actメソッド → スコープなし全体検索）を備える
- `EVENT.no_entry` は `SCENE.co_exec(name, nil, nil)` のみでグローバルシーン検索1段階しか行わず、GLOBALテーブルへのフォールバックも存在しない

この非対称性は、`＊OnHour` をDSLラベルで定義した場合とGLOBALテーブルに登録した場合で挙動が異なるという一貫性の欠如を生んでいる。

すべての `*.pasta` ファイルは**シーン辞書**であり、GLOBALテーブルもまた `act:call()` の解決空間に含まれる「シーン内」の存在である。シーンへの遷移は、その起点がイベントディスパッチであれシーン内コールであれ、すべて `act:call()` という**唯一の経路**を通るべきである。

本仕様では、**イベントハンドラからのシーン解決を `act:call()` そのものに委譲するリファクタリング**を行う。「同等のロジックを別経路で再実装する」のではなく、解決ロジックのコードパスを1本化し、経路の複製によるバグを根絶する。

## Requirements

### Requirement 1: イベントハンドラのシーン解決を act:call() に委譲する
**Objective:** ゴースト作者として、イベントハンドラ（OnHour, OnTalk等）からのシーン呼び出しが `act:call()` と同じ解決ルールに従うようにしたい。これにより、どの経路からコールされても一貫した名前解決が行われる。

**設計原則:** 解決ロジックのコードパスは1つだけとする。`EVENT.no_entry` は `act:call()` そのものを呼び出すことでシーン解決を行い、同等ロジックの複製を禁止する。

#### Acceptance Criteria
1. When `EVENT.no_entry` がイベント名でシーンを解決するとき, the pasta runtime shall `act:call()` を直接呼び出す（同等ロジックの再実装ではなく、同一コードパスを使用する）
2. When `act:call()` の解決結果が `nil` であるとき, the pasta runtime shall `RES.no_content()` (204) レスポンスとなる
3. The pasta runtime shall イベントディスパッチとシーン内コールで同一の `act:call()` 実装を共有する

### Requirement 2: 仮想イベント（OnHour, OnTalk）の解決経路統一
**Objective:** ゴースト作者として、`OnSecondChange` から発火する仮想イベント（OnHour, OnTalk）も `act:call()` 経由でシーン解決するようにしたい。これにより、DSLラベル `＊OnHour` だけでなく `GLOBAL.OnHour` としての登録も正しく解決される。

#### Acceptance Criteria
1. When `OnSecondChange` ハンドラが正時を検出して `OnHour` を発火するとき, the pasta runtime shall `act:call()` 経由でシーンまたはGLOBALハンドラを検索する
2. When `OnSecondChange` ハンドラがトークタイマー到達を検出して `OnTalk` を発火するとき, the pasta runtime shall `act:call()` 経由でシーンまたはGLOBALハンドラを検索する
3. When `GLOBAL.OnHour` に関数が登録されているとき, the pasta runtime shall DSLラベル `＊OnHour` が未定義であっても `act:call()` のフォールバックによりその関数を呼び出す
4. When DSLラベル `＊OnHour` と `GLOBAL.OnHour` の両方が存在するとき, the pasta runtime shall `act:call()` の既存優先順位に従いシーン検索（DSLラベル）をGLOBALより優先する

### Requirement 3: REGテーブル登録済みハンドラの優先制御
**Objective:** ゴースト作者として、`REG` テーブルに明示的に登録したイベントハンドラが最優先で実行されることを保証したい。`act:call()` へのフォールバックは `REG` 未登録時のみ発動する。

#### Acceptance Criteria
1. When `REG[event_id]` にハンドラが登録されているとき, the pasta runtime shall `act:call()` フォールバックをスキップし、登録済みハンドラを直接実行する
2. When `REG[event_id]` が `nil` であるとき, the pasta runtime shall `act:call()` にフォールバックする
3. The pasta runtime shall `REG` テーブルへの登録インターフェース（`REG.EventName = function(act) ... end`）を変更しない

### Requirement 4: コルーチン管理との整合性
**Objective:** ランタイム開発者として、`act:call()` 委譲によるリファクタリング後もコルーチンベースの出力管理（yield、チェイントーク）が正しく動作することを保証したい。

#### Acceptance Criteria
1. When `act:call()` がシーン関数またはGLOBAL関数を実行したとき, the pasta runtime shall その結果をコルーチンとしてラップし、既存の `resume_until_valid` による実行フローを維持する
2. While チェイントーク（`STORE.co_scene` に中断コルーチンが存在する状態）が進行中であるとき, the pasta runtime shall 新規イベントのシーン解決をスキップし、既存コルーチンを継続する
3. The pasta runtime shall `act:build()` によるさくらスクリプト最終構築が `act:call()` 経由でも正しく呼び出される

### Requirement 5: 解決優先順位の明確化
**Objective:** ゴースト作者として、イベントディスパッチにおける名前解決の優先順位を明確に理解し、予測可能な動作が得られるようにしたい。

#### Acceptance Criteria
1. The pasta runtime shall イベントディスパッチの名前解決において、以下の優先順位を適用する: (1) `REG[id]` 明示登録 → (2) `act:call()` によるフォールバック解決
2. The pasta runtime shall `act:call()` の既存優先順位をそのまま適用する（解決ロジックが1本化されているため、別途の優先順位定義は不要）
3. The pasta runtime shall 解決優先順位のドキュメントをスキルファイルまたは仕様書に記載する

### Requirement 6: 後方互換性の維持
**Objective:** 既存ゴースト作者として、本変更によって既存のゴースト辞書やスクリプトが壊れないことを保証したい。

#### Acceptance Criteria
1. When 既存のDSLラベル `＊OnHour` のみでイベントを定義しているゴーストがロードされたとき, the pasta runtime shall 従来と同一の動作を維持する
2. When 既存の `REG` テーブルにハンドラを登録しているゴーストがロードされたとき, the pasta runtime shall そのハンドラが引き続き最優先で実行される
3. The pasta runtime shall `EVENT.fire` の戻り値インターフェース（thread/string/nil → RES.ok/RES.no_content）を変更しない
4. The pasta runtime shall 既存テスト（`event_dispatch_test`, `event_handler_test`, `virtual_event_dispatch_test`, `virtual_event_config_test`）がすべてパスする
