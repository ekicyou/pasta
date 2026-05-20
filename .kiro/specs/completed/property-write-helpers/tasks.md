# Implementation Plan

## Task 1: raw_script トークンのバグ修正

- [x] 1.1 group_by_actor での raw_script ハイブリッド分類修正
  - `type == "raw_script"` を従属トークンの `else` ブランチから分離する
  - アクターグループ存在時はグループ内に追加する（既存挙動を維持）
  - アクターグループ不在時は result テーブルに直接追加する（バグ修正）
  - 修正後、talk を伴わない raw_script が group_by_actor の結果に独立トークンとして含まれることが確認できる
  - _Requirements: BugFix_

- [x] 1.2 (P) sakura_builder での最上位 raw_script ハンドリング追加
  - `BUILDER.build()` の最上位トークンループに `type == "raw_script"` の分岐を追加する
  - 処理内容: `table.insert(buffer, token.text)` — アクターグループ内の raw_script と同一処理
  - 修正後、最上位に raw_script トークンを含むグループ化済みトークンを build() に渡すと、text の内容が出力文字列に含まれることが確認できる
  - _Requirements: BugFix_
  - _Boundary: sakura_builder_

## Task 2: set_property メソッドの実装

- [x] 2.1 escape_tag_arg ローカル関数の実装
  - act.lua に、set_property からのみ使用するローカル関数として追加する
  - `\` → `\\`、`%` → `\%`、`]` → `\]` のエスケープを実装する
  - 引数に `,` または `"` を含む場合は全体を `""` で囲み、内部の `"` を `""` に二重化する
  - 処理順序は文字エスケープ → クォーティング判定の順に適用する（さくらスクリプトのパース崩壊を防ぐため）
  - `escape_tag_arg("foo,bar")` → `"foo,bar"` のように変換できることが確認できる
  - _Requirements: 2.5_

- [x] 2.2 ACT_IMPL.set_property メソッドの実装
  - name 引数が nil または空文字列の場合は error() を発生させる
  - value が nil の場合は空文字列に変換し、それ以外は tostring() を無条件適用する
  - name と value を escape_tag_arg() でそれぞれエスケープし `\![set,property,<name>,<value>]` タグ文字列を組み立てる
  - `{ type = "raw_script", text = tag }` トークンを self.token に追加して self を返す
  - `act:set_property("sakura.name", "Alice"):talk(sakura, "hello")` のようにメソッドチェーンが動作することが確認できる
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4_

## Task 3: テストの実装

- [x] 3.1 (P) act_grouping_test.lua への raw_script バグ修正テスト追記
  - raw_script のみ（talk なし）でのグループ化結果に、raw_script が独立トークンとして含まれることを検証する
  - talk の後に続く raw_script がアクターグループ内に含まれることを検証する（既存挙動の互換確認）
  - raw_script が talk の前にある場合と後にある場合の両ケースを網羅する
  - `cargo test` でこれらのテストが合格することが確認できる
  - _Requirements: BugFix_
  - _Boundary: act_grouping_test.lua_

- [x] 3.2 (P) set_property_test.lua の実装
  - set_property メソッドの単体テスト（raw_script トークン生成・self 返却・name/value バリデーション・型変換）を実装する
  - escape_tag_arg のエスケープ規則テスト（`\`、`%`、`]`、カンマ、引用符、複合ケース、エスケープ不要値）を実装する
  - set_property 単独 build テスト（`\![set,property,name,value]\e` が出力されること）を実装する
  - talk 混在 build テスト（talk + set_property が正しい順序で出力されること）を実装する
  - `cargo test` ですべてのテストが合格することが確認できる
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Boundary: set_property_test.lua_
