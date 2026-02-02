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
2. When コルーチンを実行するとき, the システム shall `coroutine.resume(co, ...)` を使用する
3. When `coroutine.resume()` が `(false, error_message)` を返したとき, the システム shall エラーとして扱う
4. When `coroutine.resume()` が `(true, ...)` を返し、`coroutine.status(co)` が "suspended" のとき, the システム shall yieldとして扱う
5. When `coroutine.resume()` が `(true, ...)` を返し、`coroutine.status(co)` が "dead" のとき, the システム shall 正常終了として扱う
6. When コルーチンを解放するとき, the システム shall `coroutine.close(co)` を呼び出して厳密にリソース解放する

### Requirement 2: EVENT.fire コルーチン対応

**目的:** SHIORIランタイムとして、EVENT.fireがコルーチン（thread）を直接管理し、シーン関数の中断と継続が透過的に行えるようにする

#### 受け入れ基準

1. When EVENT.fireがシーン関数を受け取ったとき, the EVENT module shall `coroutine.create(fn)` でコルーチンを生成する
2. When `coroutine.resume(co, act)` が `(false, error_message)` を返したとき, the EVENT module shall コルーチンを `coroutine.close(co)` で解放し、`error(error_message)` を発生させる
3. When `coroutine.resume()` 後に `coroutine.status(co)` が "suspended" のとき, the EVENT module shall `STORE.co_thread` にコルーチンを保存する
4. When `coroutine.resume()` 後に `coroutine.status(co)` が "dead" のとき, the EVENT module shall `STORE.co_thread` を nil にクリアする
5. If valueが空でない文字列の場合, the EVENT module shall `RES.ok(value)` を返す
6. If valueがnilまたは空文字列の場合, the EVENT module shall `RES.no_content()` を返す
7. When 新しいコルーチンを設定しようとしたとき、既存の `STORE.co_thread` が存在しsuspended状態の場合, the EVENT module shall 先に `coroutine.close(STORE.co_thread)` で解放してから更新する

### Requirement 3: シーン関数取得とコルーチン生成

**目的:** イベントシステムとして、シーン関数を取得し、EVENT.fireに渡すことで、EVENT.fireが統一的にコルーチンを生成・処理できるようにする

#### 受け入れ基準

1. When SCENE.searchがシーン関数を返すとき, the 呼び出し元 shall シーン関数をそのままEVENT.fireに渡す
2. When EVENT.no_entryがシーン関数を取得するとき, the EVENT module shall シーン関数をそのまま処理に渡す
3. The コルーチン生成 shall EVENT.fire内部で `coroutine.create()` により行われる

### Requirement 4: virtual_dispatcher.lua dispatch()関数の改良

**目的:** 仮想イベントディスパッチャとして、シーン関数を返すことで、呼び出し元が統一的にハンドラを処理できるようにする

**注**: OnHourは通常イベントと同じ扱い（チェイントーク非対応）、OnTalkのみチェイントーク対応

#### 受け入れ基準

1. When check_hourがOnHourシーンを見つけたとき, the virtual_dispatcher module shall 実行せずにシーン関数を返す（通常イベントと同様、毎回完結実行）
2. When check_talkがOnTalkシーンを見つけたとき, the virtual_dispatcher module shall 実行せずにシーン関数を返す
3. When dispatch()がcheck_hourまたはcheck_talkから非nilハンドラを受け取ったとき, the virtual_dispatcher module shall それを呼び出し元に返す
4. The virtual_dispatcher module shall シーン関数を直接実行しない; 実行はEVENT.fireに委譲される

### Requirement 5: チェイントーク継続（STORE.co_thread）

**目的:** OnTalkハンドラとして、前回中断したコルーチンを継続できるようにし、複数回に分けた対話が自然に繋がるようにする

**スコープ**: チェイントーク機能はOnTalkイベントのみ対応。OnHourおよび他の通常イベントは対象外

#### 受け入れ基準

1. When OnTalkタイミングに到達し、`STORE.co_thread` が nil でないとき, the check_talk関数 shall `STORE.co_thread` (継続用コルーチン) を返す
2. When OnTalkタイミングに到達し、`STORE.co_thread` が nil のとき, the check_talk関数 shall 新しいOnTalkシーンを検索して返す
3. While コルーチンがsuspended状態の間（`STORE.co_thread` が設定されている）, 新しいOnTalkシーン shall 継続を優先してスキップされる
4. If 継続されたコルーチンが正常終了した場合, the STORE.co_thread shall クリアされる
5. The OnHourイベント shall チェイントーク機能を使用せず、常に完結実行する（STORE.co_threadに影響されない）

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

1. The STOREモジュール shall nilで初期化された `STORE.co_thread` フィールドを持つ
2. When STORE.reset()が呼び出されたとき, the STOREモジュール shall 既存の `STORE.co_thread` がsuspended状態なら `coroutine.close()` で解放し、その後 nil にクリアする
3. The STORE.co_threadフィールド shall コルーチン（thread）またはnilを保持する

### Requirement 8: テストによる検証

**目的:** 開発者として、コルーチン継続のE2Eテストにより、チェイントーク機能が正しく動作することを確認できるようにする

#### 受け入れ基準

1. When `act:yield()` を含むシーン関数が実行されたとき, the テスト shall `STORE.co_thread` が設定されていることを検証する
2. When 次のOnTalkイベントが発生したとき, the テスト shall 継続が再開されることを検証する
3. When 継続後にシーン関数が正常にreturnしたとき, the テスト shall `STORE.co_thread` がクリアされていることを検証する
4. The テスト shall コルーチンがエラーを発生させたときのエラー伝搬と `coroutine.close()` による解放を検証する
5. The テスト shall 既存のsuspendedコルーチンがある状態で新しいコルーチンを設定したとき、既存コルーチンが `coroutine.close()` で解放されることを検証する

---

## 設計判断事項（Design Decisions）
 ✅ 決定

**判断**: OnTalkのみチェイントーク対応（選択肢A）

**理由**:
- OnHourは時報であり、1回完結が自然
- OnHourは通常の一般イベントハンドラと同じ扱い
- STORE.co_handlerの管理がシンプル

**影響**: Requirement 4, 5に反映済みORE.co_handlerを共有）
- **C**: OnHour専用の継続ストレージを用意（STORE.co_handler_hour）

**影響**: Requirement 4, 5の実装方法

### DD2: ユーザー定義ハンドラ（REG）のco_handler化

**判断事項**: REG[req.id]で登録されたカスタムイベントハンドラもコルーチン対応すべきか？

**選択肢**:
- **A**: 仮想イベント（OnTalk/OnHour）のみコルーチン対応
- **B**: 全てのイベントハンドラをco_handler化（後方互換性注意）
- **C**: オプトイン方式（開発者が明示的にCO.safe_wrapを呼ぶ）

**影響**: Requirement 2, 3の実装範囲

### DD3: エラー発生時の継続状態管理

**判断事項**: コルーチンでエラーが発生した場合、STORE.co_handlerをどう扱うか？

**選択肢**:
- **A**: エラー時に即座にクリア（安全側、次回は新しいシーン）
- **B**: エラーを伝搬するがco_handlerは保持（リトライ可能）
- **C**: エラー種別により判断（Lua errorはクリア、nilはスキップ）

**影響**: Requirement 2の実装詳細、テスト戦略
