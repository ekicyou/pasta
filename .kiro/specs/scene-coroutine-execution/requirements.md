# Requirements Document

## Introduction

本仕様は、Pastaスクリプトエンジンにおけるシーン関数のコルーチン実行機能を定義する。
シーン関数がyieldで中断し、次回のOnTalkイベントで継続（チェイントーク）できるようにすることで、
長い対話を複数回に分けて自然に表示する機能を実現する。

## Project Description (Input)

シーンをコルーチンとして実行できるようにする。EVENT.fireで実行されたハンドラ関数が終了せず、サスペンド（続きのトークが存在）している場合に、次回のOnTalkイベントのタイミングでチェイントークが発動するようにする。

## Requirements

### Requirement 1: コルーチンラッパーモジュール（CO）

**目的:** ゴースト開発者として、シーン関数をコルーチンとして安全にラップできるようにし、エラーハンドリングと継続管理を統一的に行えるようにする

#### 受け入れ基準

1. The CO module shall provide `CO.safe_wrap(fn)` 関数により、関数をコルーチンハンドラにラップする
2. When `co_handler(act)` が呼び出されたとき, the CO module shall return タプル `(status, value)` を返す（statusは "yield", "return", または nil (エラー)）
3. If ラップされた関数がエラーを発生させた場合, the CO module shall return `(nil, error_message)` をstatus-valueタプルとして返す
4. If ラップされた関数がyieldした場合, the CO module shall return `("yield", yielded_value)` を返し、継続状態を保持する
5. If ラップされた関数が正常にreturnした場合, the CO module shall return `("return", return_value)` を返し、完了をマークする
6. The CO module shall allow 同じ `co_handler` への後続呼び出しによる再開実行を許可する

### Requirement 2: EVENT.fire コルーチン対応

**目的:** SHIORIランタイムとして、EVENT.fireがコルーチンハンドラを処理できるようにし、シーン関数の中断と継続が透過的に行えるようにする

#### 受け入れ基準

1. When EVENT.fireがハンドラを受け取ったとき, the EVENT module shall `co_handler`（`CO.safe_wrap`でラップされたもの）であることを期待する
2. When `co_handler(act)` が `(nil, error_message)` を返したとき, the EVENT module shall `error(error_message)` によりエラーを発生させる
3. When `co_handler(act)` が `("yield", value)` を返したとき, the EVENT module shall `STORE.co_handler` に `co_handler` を保存する
4. When `co_handler(act)` が `("return", value)` を返したとき, the EVENT module shall `STORE.co_handler` を nil にクリアする
5. If valueが空でない文字列の場合, the EVENT module shall `RES.ok(value)` を返す
6. If valueがnilまたは空文字列の場合, the EVENT module shall `RES.no_content()` を返す

### Requirement 3: ハンドラ取得関数のco_handler変換

**目的:** イベントシステムとして、シーン関数を取得する際にco_handlerとして返すことで、EVENT.fireが統一的にコルーチンを処理できるようにする

#### 受け入れ基準

1. When SCENE.searchがシーン関数を返すとき, the 呼び出し元 shall 返却前に `CO.safe_wrap()` でラップする
2. When EVENT.no_entryがシーン関数を取得するとき, the EVENT module shall 実行前に `CO.safe_wrap()` でラップする
3. The ラッピング shall 実行時ではなく、ハンドラ取得時点で発生する

### Requirement 4: virtual_dispatcher.lua dispatch()関数の改良

**目的:** 仮想イベントディスパッチャとして、co_handlerを返すことで、呼び出し元が統一的にハンドラを処理できるようにする

**注**: OnHourは通常イベントと同じ扱い（チェイントーク非対応）、OnTalkのみチェイントーク対応

#### 受け入れ基準

1. When check_hourがOnHourシーンを見つけたとき, the virtual_dispatcher module shall 実行せずに `co_handler` を返す（通常イベントと同様、毎回完結実行）
2. When check_talkがOnTalkシーンを見つけたとき, the virtual_dispatcher module shall 実行せずに `co_handler` を返す
3. When dispatch()がcheck_hourまたはcheck_talkから非nilハンドラを受け取ったとき, the virtual_dispatcher module shall それを呼び出し元に返す
4. The virtual_dispatcher module shall シーン関数を直接実行しない; 実行はEVENT.fireに委譲される

### Requirement 5: チェイントーク継続（STORE.co_handler）

**目的:** OnTalkハンドラとして、前回中断したコルーチンを継続できるようにし、複数回に分けた対話が自然に繋がるようにする

**スコープ**: チェイントーク機能はOnTalkイベントのみ対応。OnHourおよび他の通常イベントは対象外

#### 受け入れ基準

1. When OnTalkタイミングに到達し、`STORE.co_handler` が nil でないとき, the check_talk関数 shall `STORE.co_handler` (継続) を返す
2. When OnTalkタイミングに到達し、`STORE.co_handler` が nil のとき, the check_talk関数 shall 新しいOnTalkシーンを検索し、`CO.safe_wrap()` でラップする
3. While コルーチンがyieldされている間（`STORE.co_handler` が設定されている）, 新しいOnTalkシーン shall 継続を優先してスキップされる
4. If 継続されたコルーチンが ("return", value) を返した場合, the STORE.co_handler shall クリアされる
5. The OnHourイベント shall チェイントーク機能を使用せず、常に完結実行する（STORE.co_handlerに影響されない）

### Requirement 6: act.yield() 機能

**目的:** シーン関数作成者として、act:yield()でトークを中断できるようにし、長い対話を次回OnTalkで継続できるようにする

#### 受け入れ基準

1. The ShioriActクラス shall 与えられた値でコルーチンをyieldする `act:yield(value)` メソッドを提供する
2. When `act:yield(value)` が呼び出されたとき, the actモジュール shall 内部で `coroutine.yield(value)` を呼び出す
3. When シーン関数が `act:yield()` を呼び出したとき, the 実行 shall 一時停止し、制御をEVENT.fireに返す
4. The yieldされた値 shall レスポンス内容としてEVENT.fireに渡される

### Requirement 7: STOREモジュール拡張

**目的:** データストアとして、コルーチンハンドラを保存できるようにし、セッション中の継続状態を管理できるようにする

#### 受け入れ基準

1. The STOREモジュール shall nilで初期化された `STORE.co_handler` フィールドを持つ
2. When STORE.reset()が呼び出されたとき, the STOREモジュール shall `STORE.co_handler` を nil にクリアする
3. The STORE.co_handlerフィールド shall 呼び出し可能なco_handler関数またはnilを保持する

### Requirement 8: テストによる検証

**目的:** 開発者として、コルーチン継続のE2Eテストにより、チェイントーク機能が正しく動作することを確認できるようにする

#### 受け入れ基準

1. When `act:yield()` を含むシーン関数が実行されたとき, the テスト shall `STORE.co_handler` が設定されていることを検証する
2. When 次のOnTalkイベントが発生したとき, the テスト shall 継続が再開されることを検証する
3. When 継続後にシーン関数が正常にreturnしたとき, the テスト shall `STORE.co_handler` がクリアされていることを検証する
4. The テスト shall コルーチンがエラーを発生させたときのエラー伝搬を検証する

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
