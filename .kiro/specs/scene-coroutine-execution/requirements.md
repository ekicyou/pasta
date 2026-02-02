# Requirements Document

## Introduction

本仕様は、Pastaスクリプトエンジンにおけるシーン関数のコルーチン実行機能を定義する。
シーン関数がyieldで中断し、次回のOnTalkイベントで継続（チェイントーク）できるようにすることで、
長い対話を複数回に分けて自然に表示する機能を実現する。

## Project Description (Input)

シーンをコルーチンとして実行できるようにする。EVENT.fireで実行されたハンドラ関数が終了せず、サスペンド（続きのトークが存在）している場合に、次回のOnTalkイベントのタイミングでチェイントークが発動するようにする。

## Requirements

### Requirement 1: コルーチン直接管理

**目的:** SHIORIランタイムとして、コルーチン（thread）を`coroutine.create()`で直接生成・管理し、`coroutine.close()`で厳密にリソース解放できるようにする

**変更理由**: `CO.safe_wrap()`ではなく`coroutine.create()`を使用することで、`coroutine.close(co)`による厳密なリソース解放が可能になる

#### 受け入れ基準

1. The システム shall シーン関数を `coroutine.create(fn)` でコルーチン（thread）として生成する
2. When コルーチンを実行するとき, the システム shall `coroutine.resume(co, act)` を使用する（EVENT.fireの文脈）
3. When `coroutine.resume()` が `(false, error_message)` を返したとき, the システム shall エラーとして扱う
4. When `coroutine.resume()` が `(true, ...)` を返し、`coroutine.status(co)` が "suspended" のとき, the システム shall yieldとして扱う
5. When `coroutine.resume()` が `(true, ...)` を返し、`coroutine.status(co)` が "dead" のとき, the システム shall 正常終了として扱う
6. When コルーチンを解放するとき, the システム shall `coroutine.close(co)` を呼び出して厳密にリソース解放する

### Requirement 2: EVENT.fire コルーチン対応

**目的:** SHIORIランタイムとして、EVENT.fireがハンドラの戻り値に応じて柔軟に処理し、シーン関数の中断と継続が透過的に行えるようにする

#### ハンドラ戻り値の仕様

handlerは以下のいずれかを返す：
- `thread`（コルーチン）: EVENT.fireがresumeして実行
- `string`（SHIORIレスポンス）: そのまま返す（既存互換のみ）
- `nil`: no_contentを返す

#### 実装規約

**全ての新規実装ハンドラは必ずthreadを返すこと**（virtual_dispatcher, EVENT.no_entry, 新規REGハンドラ等）

EVENT.fireのみがstring/nil互換を処理（既存コードベースとの後方互換用）

#### 受け入れ基準

1. When EVENT.fireがhandlerを呼び出したとき, the EVENT module shall `local result = handler(act)` を実行する
2. If resultがthread（コルーチン）の場合, the EVENT module shall `local ok, yielded_value = coroutine.resume(result, act)` を実行する
3. When `coroutine.resume()` が `(false, error_message)` を返したとき, the EVENT module shall コルーチンを `coroutine.close()` で解放し、`error(error_message)` を発生させる
4. When `coroutine.resume()` 後に `coroutine.status(co)` が "suspended" のとき, the EVENT module shall `STORE.co_scene` にコルーチンを保存し、`RES.ok(yielded_value)` を返す（yieldされた値をレスポンスに使用）
5. When `coroutine.resume()` 後に `coroutine.status(co)` が "dead" のとき, the EVENT module shall `STORE.co_scene` を nil にクリアし、適切なレスポンスを返す
6. If resultがstringの場合, the EVENT module shall `RES.ok(result)` を返す（空文字列の場合は `RES.no_content()`）
7. If resultがnilの場合, the EVENT module shall `RES.no_content()` を返す
8. When 新しいコルーチンを `STORE.co_scene` に設定しようとしたとき、既存の `STORE.co_scene` が存在しsuspended状態の場合, the EVENT module shall 先に `coroutine.close(STORE.co_scene)` で解放してから更新する

### Requirement 3: シーン関数ハンドラの戻り値

**目的:** イベントシステムとして、シーン関数をコルーチンとして返すことで、EVENT.fireが統一的に処理できるようにする

#### 実装規約

**virtual_dispatcherとEVENT.no_entryは必ずthreadを返す**

シーン関数が見つからない場合でも、空のコルーチンまたはnilを返す（実装判断）

#### 受け入れ基準

1. When virtual_dispatcherがシーン関数を取得したとき, the dispatcher shall `coroutine.create(scene_fn)` でコルーチンを生成し、**必ずthreadを返す**（actはresume時に渡す - DD4参照）
2. When EVENT.no_entryがシーン関数を取得したとき, the EVENT module shall `coroutine.create(scene_fn)` でコルーチンを生成し、**threadを返す**（見つからない場合はnilを返してEVENT.fireがno_content処理）
3. The コルーチン生成 shall シーン関数を返すハンドラ内で行われ、EVENT.fireにはthreadが渡される
4. The 既存のREGハンドラ（stringを返す）shall 変更なしで動作し続ける（EVENT.fireレベルで後方互換性を保証）

### Requirement 4: virtual_dispatcher.lua dispatch()関数の改良

**目的:** 仮想イベントディスパッチャとして、シーン関数を返すことで、呼び出し元が統一的にハンドラを処理できるようにする

**注**: OnHourは通常イベントと同じ扱い（チェイントーク非対応）、OnTalkのみチェイントーク対応

#### 受け入れ基準

1. When check_hourがOnHourシーンを見つけたとき, the virtual_dispatcher module shall 実行せずにシーン関数を返す（通常イベントと同様、毎回完結実行）
2. When check_talkがOnTalkシーンを見つけたとき, the virtual_dispatcher module shall 実行せずにシーン関数を返す
3. When dispatch()がcheck_hourまたはcheck_talkから非nilハンドラを受け取ったとき, the virtual_dispatcher module shall それを呼び出し元に返す
4. The virtual_dispatcher module shall シーン関数を直接実行しない; 実行はEVENT.fireに委譲される

### Requirement 5: チェイントーク継続（STORE.co_scene）

**目的:** OnTalkハンドラとして、前回中断したコルーチンを継続できるようにし、複数回に分けた対話が自然に繋がるようにする

**スコープ**: チェイントーク機能はOnTalkイベントのみ対応。OnHourおよび他の通常イベントは対象外

#### 受け入れ基準

1. When OnTalkタイミングに到達し、`STORE.co_scene` が nil でないとき, the check_talk関数 shall `STORE.co_scene` (継続用コルーチン) を返す
2. When OnTalkタイミングに到達し、`STORE.co_scene` が nil のとき, the check_talk関数 shall 新しいOnTalkシーンを検索して返す
3. While コルーチンがsuspended状態の間（`STORE.co_scene` が設定されている）, 新しいOnTalkシーン shall 継続を優先してスキップされる
4. If 継続されたコルーチンが正常終了した場合, the STORE.co_scene shall クリアされる
5. The OnHourイベント shall チェイントーク機能を使用せず、常に完結実行する（STORE.co_sceneに影響されない）

### Requirement 6: act.yield() 機能

**目的:** シーン関数作成者として、act:yield()でトークを中断できるようにし、長い対話を次回OnTalkで継続できるようにする

#### 受け入れ基準

1. The ShioriActクラス shall 与えられた値でコルーチンをyieldする `act:yield(value)` メソッドを提供する
2. When `act:yield(value)` が呼び出されたとき, the actモジュール shall 内部で `coroutine.yield(value)` を呼び出す
3. When シーン関数が `act:yield()` を呼び出したとき, the 実行 shall 一時停止し、制御をEVENT.fireに返す
4. The yieldされた値 shall レスポンス内容としてEVENT.fireに渡される

### Requirement 7: STOREモジュール拡張

**目的:** データストアとして、コルーチン（thread）を保存できるようにし、セッション中の継続状態を管理できるようにする

#### 受け入れ基準

1. The STOREモジュール shall nilで初期化された `STORE.co_scene` フィールドを持つ
2. When STORE.reset()が呼び出されたとき, the STOREモジュール shall 既存の `STORE.co_scene` がsuspended状態なら `coroutine.close()` で解放し、その後 nil にクリアする
3. The STORE.co_sceneフィールド shall コルーチン（thread）またはnilを保持する

### Requirement 8: テストによる検証

**目的:** 開発者として、コルーチン継続のE2Eテストにより、チェイントーク機能が正しく動作することを確認できるようにする

#### 受け入れ基準

1. When `act:yield()` を含むシーン関数が実行されたとき, the テスト shall `STORE.co_scene` が設定されていることを検証する
2. When 次のOnTalkイベントが発生したとき, the テスト shall 継続が再開されることを検証する
3. When 継続後にシーン関数が正常にreturnしたとき, the テスト shall `STORE.co_scene` がクリアされていることを検証する
4. The テスト shall コルーチンがエラーを発生させたときのエラー伝搬と `coroutine.close()` による解放を検証する
5. The テスト shall 既存のsuspendedコルーチンがある状態で新しいコルーチンを設定したとき、既存コルーチンが `coroutine.close()` で解放されることを検証する

### Requirement 9: RES.ok()の空文字列処理拡張

**目的:** レスポンス生成モジュールとして、nil/空文字列を統一的にno_contentに変換し、イベント処理層での空チェックを不要にする

#### 受け入れ基準

1. When `RES.ok(script)` が呼び出され、scriptがnilの場合, the RESモジュール shall `RES.no_content()` を返す
2. When `RES.ok(script)` が呼び出され、scriptが空文字列 `""` の場合, the RESモジュール shall `RES.no_content()` を返す
3. When `RES.ok(script)` が呼び出され、scriptが有効な文字列の場合, the RESモジュール shall 通常のSHIORIレスポンスを生成して返す

---

## 設計判断事項（Design Decisions）

### DD1: チェイントーク対象イベント ✅ 決定

**判断**: OnTalkのみチェイントーク対応（選択肢A）

**理由**:
- OnHourは時報であり、1回完結が自然
- OnHourは通常の一般イベントハンドラと同じ扱い
- STORE.co_sceneの管理がシンプル

**影響**: Requirement 4, 5に反映済み

### DD2: ユーザー定義ハンドラ（REG）のco_handler化 ✅ 決定

**判断**: 新規実装はthread必須、既存互換はEVENT.fireで吸収

**設計原則**:
- **全ての新規実装ハンドラは必ずthreadを返す**（virtual_dispatcher, EVENT.no_entry, 新規REGハンドラ）
- EVENT.fireのみが `thread or string or nil` を処理（既存コードベースとの後方互換用）
- threadの場合: EVENT.fireがcoroutine.resume()で実行
- stringの場合: そのまま返す（レガシー互換）
- nilの場合: no_content()を返す

**影響**: Requirement 2, 3に反映済み

### DD3: エラー発生時の継続状態管理 ✅ 決定

**判断**: 選択肢A - エラー時に即座にクリア（安全側）

**理由**:
- エラー発生時は`coroutine.close()`でリソース解放
- STORE.co_sceneをnilにクリア
- エラーを伝搬させてログ出力
- 次回OnTalkは新しいシーンから開始

**影響**: Requirement 2.3に反映済み

### DD4: actオブジェクトのスコープ管理 ✅ 決定

**判断**: シーン開始時のactを継続利用

**設計原則**:
- **EVENT.fireはcoroutine.resume(co, act)でactを渡す**
- **初回実行時**: 新しいactを渡してシーン関数を開始
- **継続実行時（チェイントーク）**: 同じactを渡す（シーン関数内で最初のactを使い続ける）

**実装パターン**:
```lua
-- コルーチン生成（シーン関数をそのまま使用）
local co = coroutine.create(scene_fn)

-- resume時にactを渡す（初回・継続両方）
local ok, script = coroutine.resume(co, act)
if ok and coroutine.status(co) == "suspended" then
    return RES.ok(script)  -- ← yieldされたscriptを使う
end
```

**影響**: 
- Requirement 1.2: `coroutine.resume(co, act)` でactを渡す
- Requirement 2.2: `coroutine.resume(result, act)` でactを渡す
- Requirement 3: `coroutine.create(scene_fn)` でシーン関数を直接使用
- act:yield()はcoroutine.yield(script)でscriptをyield、EVENT.fireがそれを受け取る

### DD5: yieldされた値がnil/空文字列の場合の処理 ✅ 決定

**判断**: 選択肢B - RES.ok()内部でチェック

**設計原則**:
- **RES.ok()はnil/空文字列を自動的にno_contentに変換**
- EVENT.fire側では空チェック不要（RES.ok()に委譲）
- 統一的な空文字列処理（全てのRES.ok()呼び出しで一貫性）

**実装パターン**:
```lua
-- res.lua内部
function RES.ok(script)
    if script == nil or script == "" then
        return RES.no_content()
    end
    -- 通常のレスポンス生成
end
```

**影響**:
- 新規Requirement 9: RES.ok()の拡張
- Requirement 2.4: yielded_valueの空チェックは不要（RES.ok()が処理）
