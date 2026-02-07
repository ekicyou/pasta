# Requirements Document

## Introduction
本仕様は、SHIORIリクエストテーブル（`act.req`）の主要パラメーターを `act.var` に展開し、Pasta DSLスクリプトからのアクセス性を向上させる機能を定義する。既存の `act:transfer_date_to_var()` と同様の明示的呼び出しパターンを採用し、必要なイベントでのみ展開を行うことでパフォーマンスへの影響を抑える。

加えて、Referenceパラメーターへの簡便なアクセス手段として、全角変数 `＄ｒ０`〜`＄ｒ９` による参照を提供する。`＄０`〜`＄９`（`digit_id`）は既にCall/Jump引数参照（`VarScope::Args`）として使用されているため、全角ローマ字接頭辞 `ｒ`（reference）を付与した `＄ｒ０` 形式を採用する。`ｒ０` は通常の `id` ルール（XID_START + XID_CONTINUE）でパースされ `VarScope::Local` → `var.ｒ０` に解決されるため、文法変更は不要である。

## Project Description (Input)
reqテーブルをact.varに展開してアクセスしやすくしたい。reqテーブルを良い感じに参照して、たとえば、どこがクリックされたのかを判断する。varに展開できれば良いのだが、全イベントで無条件に展開すると処理負担が大きい気がするので、act:XXX関数とかを作ってvarテーブルへの展開処理を実装する。時刻のvar展開と同じような処理。まずは転記すべきパラメーターについて検討調査する必要があります。

## Requirements

### Requirement 1: Referenceパラメーターのvar展開
**Objective:** ゴースト開発者として、`act.req.reference[N]` の値を `act.var` に転記し、Pasta DSL変数（`＄ｒ０`〜`＄ｒ９`）で直接アクセスできるようにしたい。

#### Acceptance Criteria
1. When `act:transfer_req_to_var()` が呼び出された場合, the pasta runtime shall `act.req.reference[0]`〜`act.req.reference[9]` の値を `act.var` のキー `"ｒ０"`〜`"ｒ９"`（全角）にそれぞれ転記する。
2. When `act.req.reference[N]` が `nil` である場合, the pasta runtime shall 対応する `act.var` のキーに `nil` を設定する（キーを存在させない）。
3. When `act:transfer_req_to_var()` が呼び出された場合, the pasta runtime shall `act.req.reference[0]`〜`act.req.reference[9]` の値を `act.var` のキー `"r0"`〜`"r9"`（半角）にも転記する（ASCIIキーによる代替アクセス）。
4. The pasta runtime shall `transfer_req_to_var()` をメソッドチェーン対応とし、`return self` で呼び出し元に自身を返す。

### Requirement 2: イベント識別情報のvar展開
**Objective:** ゴースト開発者として、イベントIDやその他のリクエストメタ情報にもvar経由で簡便にアクセスしたい。イベント種別に応じた分岐処理をDSL変数で記述できるようにする。

#### Acceptance Criteria
1. When `act:transfer_req_to_var()` が呼び出された場合, the pasta runtime shall `act.req.id` の値を `act.var.req_id` に転記する。
2. When `act:transfer_req_to_var()` が呼び出された場合, the pasta runtime shall `act.req.base_id` の値を `act.var.req_base_id` に転記する。

### Requirement 3: 明示的呼び出しパターン
**Objective:** ゴースト開発者として、パフォーマンスへの悪影響なく必要なタイミングでのみreq展開を実行したい。既存の `transfer_date_to_var()` と一貫した設計パターンを維持する。

#### Acceptance Criteria
1. The pasta runtime shall `transfer_req_to_var()` を `SHIORI_ACT`（または `ACT`）のメソッドとして提供する。
2. The pasta runtime shall イベントディスパッチ時に `transfer_req_to_var()` を自動呼び出ししない（明示的呼び出しのみ）。
3. When ゴースト開発者がLuaコードブロック内で `act:transfer_req_to_var()` を記述した場合, the pasta runtime shall その時点で `act.req` の内容を `act.var` に展開する。
4. While `act:transfer_req_to_var()` が呼び出されていない場合, the pasta runtime shall `act.var` にreq由来のキーを一切設定しない。
5. When `act.req` が `nil` である場合, the pasta runtime shall 何も転記せず即座に `self` を返す（ガード句）。

### Requirement 4: 全角変数 `＄ｒ０` によるDSLアクセス
**Objective:** ゴースト開発者として、Pasta DSLの変数参照構文 `＄ｒ０`〜`＄ｒ９` でReferenceパラメーターにアクセスしたい。里々の`（０）`〜`（９）`に相当する簡便なアクセス手段を、IME切替不要の全角入力で提供する。

#### Background（ギャップ分析結果）
- `＄０`〜`＄９` は `digit_id` ルールにより `VarScope::Args(N)` に解決され、Call/Jump引数参照として使用済み
- `ｒ０` は通常の `id` ルール（`ｒ` = XID_START、`０` = XID_CONTINUE）でパースされ `VarScope::Local` → `var.ｒ０` に正しく解決される
- 文法変更不要、全角モードのままIME切替なしで入力可能

#### Acceptance Criteria
1. When `act:transfer_req_to_var()` 呼び出し後にDSLスクリプト内で `＄ｒ０` が参照された場合, the pasta runtime shall `act.var["ｒ０"]`（= `act.req.reference[0]`）の値を返す。
2. The pasta runtime shall 全角Reference変数の展開範囲を `ｒ０`〜`ｒ９`（10個）に限定する。
3. The pasta runtime shall `＄ｒ０`〜`＄ｒ９`（全角）と `＄r0`〜`＄r9`（半角）の両方でアクセスできるよう、全角・半角両キーに同一の値を転記する。
4. The pasta runtime shall `＄０`〜`＄９`（`digit_id`）の既存動作（Call/Jump引数参照 `args[N+1]`）を変更しない。

### Requirement 5: 既存機能との整合性
**Objective:** 本機能が既存のvar展開機能（時刻展開）やactテーブル構造と矛盾なく共存することを保証する。

#### Acceptance Criteria
1. When `act:transfer_req_to_var()` と `act:transfer_date_to_var()` が同一actインスタンスで両方呼び出された場合, the pasta runtime shall 両方の展開結果を `act.var` に共存させる（キー衝突がないこと）。
2. The pasta runtime shall `act.var` の初期化タイミング（アクションごとの `{}` リセット）を変更しない。
3. The pasta runtime shall `act.req` テーブルを読み取り専用として扱い、`transfer_req_to_var()` が `act.req` の内容を変更しない。
