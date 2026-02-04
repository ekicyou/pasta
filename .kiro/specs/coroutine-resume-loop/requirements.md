# Requirements Document

## Project Description (Input)
EVENT.fireでコルーチンの実行部にループを追加。コルーチンのresumeがnilを返し、coroutineがまだ終了していないなら、続けてresumeし、nil以外の有効値が返ってくるか、coroutineの終了を検出したらループを抜けるようにせよ。有効な値が来るまでループするローカル関数を作った方が良いと思う。

## Introduction

SHIORIイベントハンドラーにおいて、シーン関数がコルーチンとして実行される際、想定外の挙動やバグにより途中でnil値をyieldする可能性がある。本機能は、EVENT.fire側で防御的にnil yieldを処理し、有効な値（nil以外）が得られるか、コルーチンが終了する（dead状態）まで自動的にresumeを継続するループ機構を導入する。

シーン関数は無限にyieldする可能性があるため、ループ上限は設定しない。`SCENE.co_exec`の`wrapped_fn`は最後に必ず1回`act:build()`を呼び出すことを保証しており、シーンが全く撮影されていない場合（空シーン）にnilを返す。このdead状態で返されるnilは「有効な値」として扱い、`RES.ok(nil)`→`RES.no_content()`変換により正常に処理される。

## Requirements

### Requirement 1: コルーチンresumeループ関数の実装

**Objective:** As a SHIORIランタイム, I want 想定外のnil yieldに対して堅牢に処理できる, so that シーン関数のバグや特殊ケースでもシステムが安定動作する

**実装方針**: `pasta.shiori.event.init`モジュール内にローカル関数`resume_until_valid`を新規作成し、既存の`set_co_scene`と同様の責務分離パターンを適用する（ギャップ分析Option B）。

**設計方針**: 
- シーン関数は無限にyieldする可能性があるため、ループ上限は設定しない
- `SCENE.co_exec`の`wrapped_fn`は最後に必ず1回`act:build()`を呼び出す
- dead状態で返されるnilは「有効な値」（空シーン）として扱う

#### Acceptance Criteria

1. When コルーチンのresumeがnilを返す and コルーチンステータスが"suspended"である, the EVENTモジュール shall 再度resumeを実行する
2. When コルーチンのresumeがnil以外の値を返す or コルーチンステータスが"dead"になる, the EVENTモジュール shall ループを終了してその値を返す（nilも含む）
3. If コルーチンのresume中にエラーが発生する, then the EVENTモジュール shall エラーを伝搬してコルーチンをcloseする
4. The EVENTモジュール shall ループ処理を担当するローカル関数`resume_until_valid`を実装する

**注記**: `RES.ok(nil)`は自動的に`RES.no_content()`に変換されるため、dead状態で返されるnilは正常に204レスポンスとなる。

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
