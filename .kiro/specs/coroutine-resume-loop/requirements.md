# Requirements Document

## Project Description (Input)
EVENT.fireでコルーチンの実行部にループを追加。コルーチンのresumeがnilを返し、coroutineがまだ終了していないなら、続けてresumeし、nil以外の有効値が返ってくるか、coroutineの終了を検出したらループを抜けるようにせよ。有効な値が来るまでループするローカル関数を作った方が良いと思う。

## Introduction

SHIORIイベントハンドラーにおいて、シーン関数がコルーチンとして実行される際、nil値をyieldすることで「まだ出力が確定していない」状態を表現できる。本機能は、コルーチンが有効な値（nil以外）を返すか、終了するまで自動的にresumeを継続するループ機構を`EVENT.fire`に導入する。

これにより、シーン関数は内部処理中に一時的にnilをyieldし、準備ができた時点で有効なさくらスクリプトを返すことが可能となる。

## Requirements

### Requirement 1: コルーチンresumeループ関数の実装

**Objective:** As a シーン開発者, I want コルーチンがnilをyieldした場合に自動的にresumeが継続される, so that 内部処理中の一時停止をシームレスに扱える

**実装方針**: `pasta.shiori.event.init`モジュール内にローカル関数`resume_until_valid`を新規作成し、既存の`set_co_scene`と同様の責務分離パターンを適用する（ギャップ分析Option B）。

#### Acceptance Criteria

1. When コルーチンのresumeがnilを返す and コルーチンステータスが"suspended"である, the EVENTモジュール shall 再度resumeを実行する
2. When コルーチンのresumeがnil以外の値を返す, the EVENTモジュール shall ループを終了してその値を返す
3. When コルーチンステータスが"dead"になる, the EVENTモジュール shall ループを終了する
4. If コルーチンのresume中にエラーが発生する, then the EVENTモジュール shall エラーを伝搬してコルーチンをcloseする
5. The EVENTモジュール shall ループ処理を担当するローカル関数`resume_until_valid`を実装する

**注記**: `RES.ok(nil)`は自動的に`RES.no_content()`に変換されるため、ループ内でのnil判定は`RES.ok`呼び出しの前に行う必要がある。

### Requirement 2: EVENT.fire統合

**Objective:** As a SHIORIランタイム, I want EVENT.fireがループ関数を利用してコルーチンを実行する, so that 既存のハンドラとの互換性を維持しつつ新機能を提供できる

#### Acceptance Criteria

1. When ハンドラがthreadを返す, the EVENT.fire shall `resume_until_valid`関数を使用してコルーチンを実行する
2. The EVENT.fire shall 既存の文字列ハンドラとの後方互換性を維持する
3. The EVENT.fire shall 既存のnil戻り値（RES.no_content()）との後方互換性を維持する
4. When ループ終了後にコルーチンがsuspended状態である, the EVENT.fire shall `set_co_scene`でコルーチンを保存する

### Requirement 3: エラーハンドリングとリソース管理

**Objective:** As a システム管理者, I want エラー発生時にリソースが適切に解放される, so that メモリリークや状態不整合を防止できる

#### Acceptance Criteria

1. If resume中にエラーが発生する, then the EVENTモジュール shall `set_co_scene`を呼び出してコルーチンをcloseする
2. If resume中にエラーが発生する, then the EVENTモジュール shall エラーメッセージをerror()で伝搬する
3. The EVENTモジュール shall deadステータスのコルーチンを適切にcloseする
