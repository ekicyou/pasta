# Requirements Document

## Project Description (Input)
「　＞チェイン or yield」とすれば、継続トークになるようにしたい。具体的には`pasta.global`モジュールが返すGLOBALテーブルに、チェイン、yieldシーン関数を登録して、その中で「act:yield()」だけ実行すればよいはず。コール先の解決ルールを調査し、この方法で実際に問題ないかを検証したうえで、仕様を実現せよ。

## Introduction

本仕様は、Pasta DSLの `＞チェイン` / `＞yield` 呼び出し構文による継続トーク機能を定義する。`pasta.global` モジュールの GLOBAL テーブルにシーン関数を事前登録することで、ユーザーが Pasta DSL 内で `＞チェイン` と記述するだけで `act:yield()` が実行され、チェイントーク（継続トーク）を実現する。

### 技術的根拠

`ACT_IMPL.call` の4段階優先順位検索において、GLOBAL テーブルは **Level 3** で検索される。シーンローカル（L1）やスコープ付き検索（L2）で見つからない場合に GLOBAL テーブルが参照されるため、`GLOBAL.チェイン` / `GLOBAL.yield` として登録した関数は `＞チェイン` / `＞yield` 構文で確実に呼び出される。また、シーン関数は `SCENE.co_exec()` によりコルーチン内で実行されるため、GLOBAL 関数内での `act:yield()` 呼び出しは正常に動作する。

## Requirements

### Requirement 1: GLOBAL テーブルへの継続トーク関数登録

**Objective:** As a ゴースト作者, I want `pasta.global` モジュールに「チェイン」と「yield」の継続トーク関数が事前登録されていること, so that Pasta DSL の `＞チェイン` や `＞yield` 構文だけで継続トークを実現できる。

#### Acceptance Criteria

1. When pasta ランタイムが初期化されたとき, the `pasta.global` module shall `GLOBAL.チェイントーク` 関数と `GLOBAL.yield` 関数を提供する。
2. The `GLOBAL.チェイントーク` function and `GLOBAL.yield` function shall `act:yield()` を実行し、蓄積されたトークンを中間出力として返す（両者は同一動作）。
3. When `＞チェイン` または `＞yield` が呼び出されたとき, the `ACT_IMPL.call` shall Level 3（GLOBAL テーブル完全一致）で解決する。ただし同名のローカルシーン（L1）やスコープ付きシーン（L2）が存在する場合はそちらが優先される（既存の4段階検索ルールに従う）。
4. While `pasta.global` モジュールにデフォルト実装が存在するとき, the runtime shall ユーザーが `main.lua` 等で明示的に上書きしない限りデフォルト実装を使用する（Luaテーブルの再代入による自然なオーバーライド）。

### Requirement 2: ランタイム動作試験 — `＞チェイントーク` から `GLOBAL.チェイントーク` への解決

**Objective:** As a 開発者, I want Pasta DSL で `＞チェイン` を記述したとき実際に `GLOBAL.チェイン` が呼び出されることをランタイムテストで確認できること, so that Call 解決ルール（L3: GLOBAL テーブル）が正しく機能していることを保証できる。

#### Acceptance Criteria

1. When Pasta DSL に `　＞チェイントーク` を含むシーンをトランスパイルして実行したとき, the runtime shall `GLOBAL.チェイントーク` 関数を呼び出す。
2. When `GLOBAL.チェイントーク` が呼び出されたとき, the runtime shall `act:yield()` を実行し、蓄積トークンを中間出力として返す。
3. When テスト用 Pasta スクリプトを作成したとき, the test shall トランスパイル → ランタイム実行 → 結果検証の一連のフローを自動で実行する。
4. When `＞チェイン` がコルーチン内で実行されたとき, the runtime shall `coroutine.yield()` を通じて蓄積トークンを呼び出し元に正しく返し、resume 後も後続のトークン蓄積を継続できる状態に復帰する。

### Requirement 3: インテグレーションテスト — `EVENT.fire` 経由のチェイントーク yield 挙動確認

**Objective:** As a 開発者, I want `EVENT.fire` を通じて `＞チェイン` を含むシーンを実行し、yield によりコルーチンが2回に分割されることをインテグレーションテストで確認できること, so that 継続トーク機能がSHIORI イベントディスパッチの文脈で正しく動作することを保証できる。

#### Acceptance Criteria

1. When `＞チェイントーク` を1回含むシーンが `EVENT.fire` 経由で実行されたとき, the runtime shall コルーチンを2回 resume する（yield 前の出力 + yield 後の出力）。
2. When コルーチンの1回目の resume が完了したとき, the runtime shall yield 前に蓄積されたトークンのみを中間出力として返す。
3. When コルーチンの2回目の resume が完了したとき, the runtime shall yield 後に蓄積されたトークンを最終出力として返す。
4. The integration test shall インテグレーションテストの実施方法（テスト用ゴーストの構成、EVENT.fire の呼び出し手順、結果の検証方法）を設計段階で検討・決定する。

