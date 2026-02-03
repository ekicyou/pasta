# Requirements Document

## Introduction

`pasta.shiori.act`モジュールは現在、`act._buffer`に直接さくらスクリプト文字列を構築している。しかし、これは親クラス`pasta.act`が定める`act.token`を用いた構造化トークン蓄積ルールに違反している。本仕様は、この設計違反を解消し、責務分離を実現するためのリファクタリングを定義する。

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

### Requirement 1: トークン蓄積への移行
**Objective:** 開発者として、`pasta.shiori.act`のメソッド群が構造化トークンを蓄積する仕組みを求める。これにより、さくらスクリプト生成ロジックを分離し、将来の拡張性と保守性を確保したい。

#### Acceptance Criteria
1. When `SHIORI_ACT_IMPL.talk()`が呼び出されたとき, the `pasta.shiori.act` shall `_buffer`への直接文字列追加を行わず、代わりに`{ type = "talk", text = text }`形式のトークンを`self.token`に追加する
2. When `SHIORI_ACT_IMPL.surface()`が呼び出されたとき, the `pasta.shiori.act` shall `{ type = "surface", id = id }`形式のトークンを`self.token`に追加する
3. When `SHIORI_ACT_IMPL.wait()`が呼び出されたとき, the `pasta.shiori.act` shall `{ type = "wait", ms = ms }`形式のトークンを`self.token`に追加する
4. When `SHIORI_ACT_IMPL.newline()`が呼び出されたとき, the `pasta.shiori.act` shall `{ type = "newline", n = n }`形式のトークンを`self.token`に追加する
5. When `SHIORI_ACT_IMPL.clear()`が呼び出されたとき, the `pasta.shiori.act` shall `{ type = "clear" }`形式のトークンを`self.token`に追加する
6. The `pasta.shiori.act` shall さくらスクリプト変換処理（エスケープ、タグ生成等）をトークン蓄積時点では行わない

### Requirement 2: スポット切り替えトークンの蓄積
**Objective:** 開発者として、スポット（アクター位置）の切り替え情報をトークンとして記録したい。これにより、build時に適切なスポットタグ（`\p[n]`）と段落区切り改行を生成できるようにする。

#### Acceptance Criteria
1. When `SHIORI_ACT_IMPL.talk()`でスポット切り替えが発生したとき, the `pasta.shiori.act` shall 親クラス`ACT.IMPL.talk()`を呼び出す前に`{ type = "spot_switch", newlines = <設定値> }`トークンを挿入する
2. When `SHIORI_ACT_IMPL.talk()`が呼び出されたとき, the `pasta.shiori.act` shall 親クラス`ACT.IMPL.talk(self, actor, text)`に委譲し、actor/talkトークンの蓄積を行う
3. The `pasta.shiori.act` shall スポット切り替え検出のため`self._current_spot`を追跡する

### Requirement 3: talk後の固定改行の除去
**Objective:** 開発者として、`talk()`後に自動挿入される固定改行（`\n`）を除去したい。これにより、さくらスクリプト出力の柔軟性を向上させる。

#### Acceptance Criteria
1. When `SHIORI_ACT_IMPL.talk()`が呼び出されたとき, the `pasta.shiori.act` shall テキスト後に固定改行（`\n`）を自動挿入しない
2. The `pasta.shiori.act` shall 明示的な`newline()`呼び出しによってのみ改行トークンを追加する

### Requirement 4: sakura_builderモジュールの新設
**Objective:** 開発者として、さくらスクリプト構築ロジックを専用モジュールに分離したい。これにより、単一責務原則を遵守し、テスト容易性を向上させる。

#### Acceptance Criteria
1. The `pasta.shiori.sakura_builder` shall 新規モジュールとして`pasta/shiori/sakura_builder.lua`に作成される
2. The `pasta.shiori.sakura_builder` shall トークン配列を受け取り、さくらスクリプト文字列を返却する`build(tokens)`関数を公開する
3. When `build()`が呼び出されたとき, the `pasta.shiori.sakura_builder` shall 各トークンタイプに応じた変換処理を実行する:
   - `talk` → エスケープ済みテキスト
   - `actor` → スポットタグ（`\p[n]`）（actor.spotから決定）
   - `spot_switch` → 段落区切り改行（`\n[percent]`）（newlinesパラメーターから計算）
   - `surface` → サーフェスタグ（`\s[id]`）
   - `wait` → 待機タグ（`\w[ms]`）
   - `newline` → 改行タグ（`\n`）× n回
   - `clear` → クリアタグ（`\c`）
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

### Requirement 6: 既存APIの互換性維持
**Objective:** 開発者として、既存のゴーストスクリプトが変更なしで動作することを保証したい。これにより、後方互換性を維持する。

#### Acceptance Criteria
1. The `pasta.shiori.act` shall 既存の公開メソッドシグネチャ（`talk`, `surface`, `wait`, `newline`, `clear`, `build`, `yield`, `reset`）を維持する
2. The `pasta.shiori.act` shall メソッドチェーン（`return self`）パターンを維持する
3. When `yield()`が呼び出されたとき, the `pasta.shiori.act` shall 内部で`build()`を呼び出し、生成されたさくらスクリプト文字列をcoroutine.yieldする
4. The `pasta.shiori.act` shall `transfer_date_to_var()`メソッドの動作を変更しない

### Requirement 7: reset()メソッドの更新
**Objective:** 開発者として、`reset()`メソッドが新しいトークンベースのアーキテクチャに対応することを求める。

#### Acceptance Criteria
1. When `reset()`が呼び出されたとき, the `pasta.shiori.act` shall `self.token`を空配列に初期化する
2. When `reset()`が呼び出されたとき, the `pasta.shiori.act` shall `self._current_spot`と`self.now_actor`を`nil`にリセットする
3. The `pasta.shiori.act` shall `self._buffer`への参照を完全に削除する
