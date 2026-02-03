# Requirements Document

## Introduction

本仕様は「要件/設計/実装」の責務分離原則に基づき、`pasta.act`と`pasta.shiori.act`のアーキテクチャを再設計する。

| 層                 | 責務                                | モジュール                    |
| ------------------ | ----------------------------------- | ----------------------------- |
| **要件（What）**   | 「誰が何をした」の記録              | `pasta.act`                   |
| **設計（How）**    | トークン→さくらスクリプト変換ルール | `pasta.shiori.sakura_builder` |
| **実装（Output）** | さくらスクリプト文字列生成          | `pasta.shiori.act.build()`    |

### 現状の問題点
1. `pasta.shiori.act`が`_buffer`に直接さくらスクリプトを構築（要件と実装の混在）
2. スポット切り替え検出を子クラスで重複実装（親クラスの責務漏れ）
3. UI操作メソッド（surface/wait/newline/clear）が子クラスにのみ存在

### リファクタリング方針
- `pasta.act`: 全てのトークン蓄積責務を担う（「何が起きたか」の記録）
- `pasta.shiori.act`: さくらスクリプト生成のみ（`build()`のオーバーライド）
- `pasta.shiori.sakura_builder`: トークン→さくらスクリプト変換ロジック

## Requirements

### Requirement 1: 親クラスでのUI操作トークン蓄積
**Objective:** 開発者として、UI操作（surface/wait/newline/clear）のトークン蓄積を親クラス`pasta.act`の責務としたい。これにより、子クラスでの重複実装を排除し、将来の他のUI実装でも再利用可能にする。

#### Acceptance Criteria
1. The `pasta.act` shall `surface(id)`メソッドを追加し、`{ type = "surface", id = id }`トークンを蓄積する
2. The `pasta.act` shall `wait(ms)`メソッドを追加し、`{ type = "wait", ms = math.max(0, math.floor(ms or 0)) }`トークンを蓄積する
3. The `pasta.act` shall `newline(n)`メソッドを追加し、`{ type = "newline", n = n or 1 }`トークンを蓄積する
4. The `pasta.act` shall `clear()`メソッドを追加し、`{ type = "clear" }`トークンを蓄積する
5. The `pasta.act` shall これらのメソッドはメソッドチェーン用に`self`を返す

### Requirement 2: 親クラスでのスポット切り替え検出
**Objective:** 開発者として、スポット切り替え検出を親クラス`pasta.act`の責務としたい。これにより、子クラスでの重複実装を排除する。

#### Acceptance Criteria
1. The `pasta.act` shall 新規フィールド`_current_spot`を追加し、現在のスポットIDを追跡する
2. When `ACT_IMPL.talk()`が呼び出され、かつスポットが前回と異なるとき, the `pasta.act` shall `{ type = "spot_switch" }`トークンを`actor`トークンの直後に挿入する
3. When スポット切り替えが検出されたとき, the `pasta.act` shall `self._current_spot`を新しいスポットIDに更新する
4. The `pasta.act` shall `ACT.new()`で`_current_spot = nil`を初期化する
5. The `pasta.act` shall `yield()`実行時に`_current_spot`を`nil`にリセットする

### Requirement 3: 子クラスのトークン蓄積メソッド削除
**Objective:** 開発者として、`pasta.shiori.act`のトークン蓄積メソッドを削除したい。これらは親クラスに移譲されたため、子クラスでのオーバーライドは不要になった。

#### Acceptance Criteria
1. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.talk()`メソッドを削除する（親クラスのtalk()を使用）
2. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.surface()`メソッドを削除する（親クラスのsurface()を使用）
3. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.wait()`メソッドを削除する（親クラスのwait()を使用）
4. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.newline()`メソッドを削除する（親クラスのnewline()を使用）
5. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.clear()`メソッドを削除する（親クラスのclear()を使用）
6. The `pasta.shiori.act` shall `SHIORI_ACT_IMPL.reset()`メソッドを削除する（親クラスのreset()で十分）

### Requirement 4: 子クラスの内部フィールド削除
**Objective:** 開発者として、`pasta.shiori.act`の不要になった内部フィールドを削除したい。

#### Acceptance Criteria
1. The `pasta.shiori.act` shall `_buffer`フィールドを完全に削除する
2. The `pasta.shiori.act` shall `_current_spot`フィールドを削除する（親クラスが管理）
3. The `pasta.shiori.act` shall `new()`で`_spot_switch_newlines`設定のみを保持する（さくらスクリプト生成用）

### Requirement 5: talk後の固定改行除去
**Objective:** 開発者として、`talk()`後に自動挿入される固定改行（`\n`）を除去したい。これにより、さくらスクリプト出力の柔軟性を向上させる。

#### Acceptance Criteria
1. The `pasta.act` shall talk()呼び出し時にテキスト後の固定改行を自動挿入しない
2. The `pasta.act` shall 改行は明示的な`newline()`呼び出しによってのみトークン化する

### Requirement 6: sakura_builderモジュールの新設
**Objective:** 開発者として、さくらスクリプト構築ロジック（設計層）を専用モジュールに分離したい。これにより、単一責務原則を遵守し、テスト容易性を向上させる。

#### Acceptance Criteria
1. The `pasta.shiori.sakura_builder` shall 新規モジュールとして`pasta/shiori/sakura_builder.lua`に作成される
2. The `pasta.shiori.sakura_builder` shall トークン配列と設定を受け取り、さくらスクリプト文字列を返却する`build(tokens, config)`関数を公開する
3. When `build()`が呼び出されたとき, the `pasta.shiori.sakura_builder` shall 各トークンタイプに応じた変換処理を実行する:
   - `talk` → エスケープ済みテキスト
   - `actor` → スポットタグ（`\p[n]`）（actor.spotから決定）
   - `spot_switch` → 段落区切り改行（`\n[percent]`）（configのspot_switch_newlinesから計算）
   - `surface` → サーフェスタグ（`\s[id]`）
   - `wait` → 待機タグ（`\w[ms]`）
   - `newline` → 改行タグ（`\n`）× n回
   - `clear` → クリアタグ（`\c`）
   - `sakura_script` → そのまま出力（エスケープなし）
   - `yield` → 無視（出力対象外）
4. When `build()`が完了したとき, the `pasta.shiori.sakura_builder` shall 出力文字列の末尾に`\e`を付与する
5. The `pasta.shiori.sakura_builder` shall さくらスクリプト用エスケープ処理（`\` → `\\`、`%` → `%%`）を内包する
6. The `pasta.shiori.sakura_builder` shall ヘルパー関数（`escape_sakura`, `spot_to_id`, `spot_to_tag`）を内部に持つ

### Requirement 7: 親クラスにbuild()メソッド新設
**Objective:** 開発者として、`pasta.act`に`build()`メソッドを新設したい。これにより、トークン取得とリセットの責務を親クラスに集約し、子クラスのbuild()は親を呼び出してフォーマット変換のみを行う。

#### Acceptance Criteria
1. The `pasta.act` shall `ACT_IMPL.build()`メソッドを新設する
2. When `ACT_IMPL.build()`が呼び出されたとき, the `pasta.act` shall 現在のトークン配列を取得し、`self.token = {}`でリセットする
3. When `ACT_IMPL.build()`が呼び出されたとき, the `pasta.act` shall `self.now_actor = nil`および`self._current_spot = nil`をリセットする
4. The `pasta.act` shall build()はトークン配列を返却する
5. The `pasta.act` shall 既存の`yield()`は内部で`build()`を呼び出すようリファクタリングする

### Requirement 8: 親クラスのyield()責務統一
**Objective:** 開発者として、`pasta.act`の`yield()`を「build()の結果をcoroutine.yieldする」という単一責務に統一したい。子クラスはbuild()のみオーバーライドすれば、yield()は自動的に正しい出力形式になる（多態性の活用）。

#### Acceptance Criteria
1. The `pasta.act` shall `ACT_IMPL.yield()`で`self:build()`を呼び出す
2. The `pasta.act` shall build()の戻り値をそのまま`coroutine.yield(result)`に渡す
3. The `pasta.act` shall メソッドチェーン用に`self`を返す
4. The `pasta.shiori.act` shall yield()をオーバーライドしない（親クラスのyield()をそのまま使用）
5. The `pasta.shiori.act` shall 既存のSHIORI_ACT_IMPL.yield()メソッドを削除する

### Requirement 9: 子クラスのbuild()オーバーライド
**Objective:** 開発者として、`SHIORI_ACT_IMPL.build()`が親の`build()`を呼び出した後、`sakura_builder`で変換するようにしたい。

#### Acceptance Criteria
1. When `SHIORI_ACT_IMPL.build()`が呼び出されたとき, the `pasta.shiori.act` shall `ACT.IMPL.build(self)`で親のbuild()を呼び出しトークン配列を取得する
2. When トークン配列を取得したとき, the `pasta.shiori.act` shall `pasta.shiori.sakura_builder.build(token, { spot_switch_newlines = self._spot_switch_newlines })`で変換する
3. The `pasta.shiori.act` shall 変換結果に`\e`を付与して返却する

### Requirement 10: 既存APIの互換性維持
**Objective:** 開発者として、既存のゴーストスクリプトが変更なしで動作することを保証したい。

#### Acceptance Criteria
1. The `pasta.shiori.act` shall 既存の公開メソッドシグネチャ（`talk`, `surface`, `wait`, `newline`, `clear`, `build`, `yield`）をすべて利用可能に保つ（親クラス継承含む）
2. The `pasta.act` および `pasta.shiori.act` shall メソッドチェーン（`return self`）パターンを維持する
3. The `pasta.shiori.act` shall `transfer_date_to_var()`メソッドの動作を変更しない
4. The `pasta.shiori.act` shall アクタープロキシ経由のメソッド呼び出し（`act.sakura:talk("Hello")`）を引き続きサポートする

### Requirement 11: end_action()の削除
**Objective:** 開発者として、`ACT_IMPL.end_action()`を公開APIから削除したい。`end_action()`は`build()`と意味が重複しており、出力終端は`sakura_builder`が`\e`を付与することで満たされるため、不要である。

#### Acceptance Criteria
1. The `pasta.act` shall `ACT_IMPL.end_action()`メソッドを削除する
2. The `pasta.act` shall 公開APIから`end_action()`を削除する

## 設計判断事項（設計フェーズで決定）

1. **スポットID計算のタイミング**: トークン蓄積時 vs build時
   - 推奨: build時（sakura_builder内で計算）
   - 理由: トークンは「何が起きたか」の純粋な記録であるべき

2. **spot_switchトークンの挿入位置**: actorトークンの前 vs 後
   - 推奨: actorトークンの後
   - 理由: スポットタグ出力後に段落区切り改行を出力する現行動作を維持

3. **CONFIG依存の渡し方**: `build(tokens, config)` の設計
   - 推奨: 設定オブジェクトとして渡す
   - 理由: sakura_builderの単体テストが容易になる
