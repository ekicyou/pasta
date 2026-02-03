# Requirements Document

## Introduction

`pasta.shiori.act`モジュールは現在、`act._buffer`に直接さくらスクリプト文字列を構築しているが、これは親クラス`pasta.act`が定める`act.token`を用いた構造化トークン蓄積ルールに違反している。また、スポット切り替え検出という本来親クラスが担うべき責務を子クラスで重複実装している。

本仕様は、この設計違反を解消し、正しい責務分離を実現するための総合的なリファクタリングを定義する。親クラス`pasta.act`と子クラス`pasta.shiori.act`の両方を修正対象とする。

## Project Description (Input)
`pasta.shiori.act`のメソッド群は現在、act._bufferに直接さくらスクリプトを構築しているが、これは親クラスの`pasta.act`のact.tokenを用いた構築ルールに違反する。

`pasta.shiori.act`のメソッド群は`{ type = "talk", text = text }`などと、act.tokenにトークンタイプとパラメーターを蓄積する。この時、さくらスクリプト用の事前返還などは行わない。

act:build()はact.tokenを参照し、さくらスクリプトを組み立てる。

act:build()は高度な変換処理を行うため、`pasta.shiori.sakura_builder`モジュールに組み立て処理を移譲する。

これらの実装により、`pasta.shiori.act`から、さくらスクリプト構築処理本体を`pasta.shiori.sakura_builder`に移譲する。

### 追加のスコープ
pasta.shiori.actのtalk関数にて、以下の処理を行っているが不要。除外する。

```lua
    -- テキスト後の改行(固定\n)
    table.insert(self._buffer, "\\n")
```

## Requirements

### Requirement 1: 親クラスでのスポット切り替え検出とトークン蓄積
**Objective:** 開発者として、スポット（アクター位置）切り替え検出を親クラス`pasta.act`の責務としたい。これにより、子クラスでの重複実装を排除し、責務を明確化する。

#### Acceptance Criteria
1. The `pasta.act` shall 新規フィールド`_current_spot`を追加し、現在のスポットIDを追跡する
2. When `ACT_IMPL.talk()`が呼び出され、かつ`actor.spot`が前回と異なるとき, the `pasta.act` shall `{ type = "spot_switch" }`トークンを`actor`トークンと`talk`トークンの間に挿入する
3. When スポット切り替えが検出されたとき, the `pasta.act` shall `self._current_spot`を新しいスポットIDに更新する
4. The `pasta.act` shall `ACT.new()`で`_current_spot = nil`を初期化する
5. The `pasta.act` shall `yield()`および`end_action()`実行時に`_current_spot`をリセット（`nil`に設定）する

### Requirement 2: 親クラスでのトークン型拡張
**Objective:** 開発者として、SHIORI固有のUI操作（surface/wait/newline/clear）を親クラスの基本トークン型として定義したい。これにより、将来の他のUI実装でも再利用可能にする。

#### Acceptance Criteria
1. The `pasta.act` shall `surface(id)`メソッドを追加し、`{ type = "surface", id = id }`トークンを蓄積する
2. The `pasta.act` shall `wait(ms)`メソッドを追加し、`{ type = "wait", ms = ms }`トークンを蓄積する
3. The `pasta.act` shall `newline(n)`メソッドを追加し、`{ type = "newline", n = n or 1 }`トークンを蓄積する
4. The `pasta.act` shall `clear()`メソッドを追加し、`{ type = "clear" }`トークンを蓄積する
5. The `pasta.act` shall これらのメソッドはメソッドチェーン用に`self`を返す
子クラスの簡素化 - talk()オーバーライド削除
**Objective:** 開発者として、`pasta.shiori.act`のtalk()オーバーライドを削除したい。スポット切り替え検出は親クラスに移譲されたため、子クラス固有の処理は不要になった。

#### Acceptance Criteria
1. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.talk()`メソッドを削除する（親クラスのtalk()をそのまま使用）
2. The `pasta.shiori.act` shall `_current_spot`フィールドを削除する（親クラスが管理）
3. The `pasta.shiori.act` shall talk()後の固定改行（`\n`）自動挿入を行わない（トークンベースで改行制御）all テキスト後に固定改行（`\n`）を自動挿入しない
2. The `pasta.shiori.act` shall 明示的な`newline()`呼び出しによってのみ改行トークンを追加する
子クラスのUI操作メソッド削除
**Objective:** 開発者として、`pasta.shiori.act`のUI操作メソッド（surface/wait/newline/clear）を削除したい。これらは親クラスに移譲されたため、子クラスでのオーバーライドは不要になった。

#### Acceptance Criteria
1. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.surface()`メソッドを削除する（親クラスのsurface()を使用）
2. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.wait()`メソッドを削除する（親クラスのwait()を使用）
3. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.newline()`メソッドを削除する（親クラスのnewline()を使用）
4. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.clear()`メソッドを削除する（親クラスのclear()を使用）

### Requirement 5: sakura_builderモジュールの新設
**Objective:** 開発者として、さくらスクリプト構築ロジックを専用モジュールに分離したい。これにより、単一責務原則を遵守し、テスト容易性を向上させる。

#### Acceptance Criteria
1. The `pasta.shiori.sakura_builder` shall 新規モジュールとして`pasta/shiori/sakura_builder.lua`に作成される
2. The `pasta.shiori.sakura_builder` shall トークン配列とCONFIG設定を受け取り、さくらスクリプト文字列を返却する`build(tokens, config)`関数を公開する
3. When `build()`が呼び出されたとき, the `pasta.shiori.sakura_builder` shall 各トークンタイプに応じた変換処理を実行する:
   - `talk` → エスケープ済みテキスト
   - `actor` → スポットタグ（`\p[n]`）（actor.spotから決定）
   - `spot_switch` → 段落区切り改行（`\n[percent]`）（configのspot_switch_newlinesから計算）
   - `surface` → サーフェスタグ（`\s[id]`）
   - `wait` → 待機タグ（`\w[ms]`）
   - `newline` → 改行タグ（`\n`）× n回
   - `clear` → クリアタグ（`\c`）
   - `sakura_script` → そのまま出力（
   - `sakura_script` → そのまま出力（親クラス由来、エスケープなし）
4. When `build()`が完了したとき, the `pasta.shiori.sakura_builder` shall 出力文字列の末尾に`\e`を付与する
5. The `pasta.shiori.sakura_builder` shall さくらスクリプト用エスケープ処理（`\` → `\\`、`%` → `%%`）を内包する
6. The `pasta.shiori.sakura_builder` shall ヘルパー関数（`escape_sakura`, `spot_to_id`, `spot_to_tag`）を内部に持つ

### Requirement 5: build()メソッドの委譲実装
**Objective:** 開発者として、`SHIORI_ACT_IMPL.build()`が`sakura_builder`に処理を委譲するようにしたい。これにより、actモジュールの責務をトークン蓄積に限定する。

#### Acceptance Criteriaし、`self.now_actor`と`self._current_spot`をリセットする
3. The `pasta.shiori.act` shall `_buffer`フィールドを完全に削除する
4. The `pasta.shiori.act` shall build()実行後も自動的に`reset()`を呼び出す既存動作を維持ll `self.token`をクリア（空配列に初期化）する
3. When `build()`が完了したとき, the `pasta.shiori.act` shall `self.now_actor`と`self._current_spot`をリセットする
4. The `pasta.shiori.act` shall `_buffer`フィールドを完全に削除する

### Requirement 7: 親クラスreset()の更新
**Objective:** 開発者として、親クラスの`reset()`メソッドが`_current_spot`もリセットすることを求める。これにより、状態管理の一貫性を保つ。

#### Acceptance Criteria
1. When `ACT_IMPL.yield()`または`end_action()`が呼び出されたとき, the `pasta.act` shall `self._current_spot = nil`をリセットする
2. The `pasta.act` shall 既存の`self.token = {}`および`self.now_actor = nil`リセット処理を維持する

### Requirement 8: 子クラスreset()の簡素化
**Objective:** 開発者として、`pasta.shiori.act`の`reset()`を簡素化したい。`_buffer`と`_current_spot`は削除されたため、リセット処理は不要になった。

#### Acceptance Criteria
1. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.reset()`メソッドを削除する（親クラスのreset()で十分）
2. The `pasta.shiori.act` shall `_buffer`フィールドへの参照を完全に削除する

### Requirement 9: 既存APIの互換性維持
**Objective:** 開発者として、既存のゴーストスクリプトが変更なしで動作することを保証したい。これにより、後方互換性を維持する。

#### Acceptance Criteria
1. The `pasta.shiori.act` shall 既存の公開メソッドシグネチャ（`talk`, `surface`, `wait`, `newline`, `clear`, `build`, `yield`）をすべて利用可能に保つ（親クラス継承含む）
2. The `pasta.act` および `pasta.shiori.act` shall メソッドチェーン（`return self`）パターンを維持する
3. When `yield()`が呼び出されたとき, the `pasta.shiori.act` shall 内部で`build()`を呼び出し、生成されたさくらスクリプト文字列をcoroutine.yieldする
4. The `pasta.shiori.act` shall `transfer_date_to_var()`メソッドの動作を変更しない
