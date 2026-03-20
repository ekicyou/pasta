# 実装タスク: onhour-fallback-chain

## 実装計画

- [ ] 1. `check_hour()` に4段階フォールバックチェーンを実装する
  - `check_hour()` 末尾の `create_scene_thread("OnHour", act)` 1行を削除し、ループに置き換える
  - `act.req.date.hour` から0埋め2桁の文字列 `hh` を生成する（`string.format("%02d", hour)`）
  - `{"時報" .. hh, "OnHour" .. hh, "時報その他", "OnHourOther"}` の順で候補リストを構成する
  - 候補ごとに `create_scene_thread(name, act)` を呼び、最初にスレッドが返された候補で即リターンする
  - 全候補で `nil` だった場合は `nil` を返す
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 3.1, 4.1_

- [ ] 2. 既存テストを新しいフォールバック仕様に合わせて更新する
- [ ] 2.1 `virtual_dispatcher_spec.lua` のモックを候補名対応に更新する (P)
  - `_set_scene_executor` の `event_name == "OnHour"` 判定を候補名（`"OnHourOther"` など）に変更する
  - `received_event_name` の期待値を `"OnHour"` から新候補名に変更する
  - `_Requirements: 1.1, 4.1_`
- [ ] 2.2 `virtual_dispatcher_thread_test.lua` のモックを候補名対応に更新する (P)
  - `scene_executor` が `"OnHourOther"` などを受け取るようにモックを修正する
  - `_Requirements: 1.1, 4.1_`
- [ ] 2.3 `global_fallback_integration_test.lua` の OnHour 参照を OnHourOther に変更する (P)
  - `GLOBAL.OnHour`、`GLOBAL.OnHour = nil` の参照をすべて `GLOBAL.OnHourOther` に変更する
  - `EVENT.fire` のイベント呼び出し部分は変更しない（`"OnHour"` は仮想イベント名で変わらない）
  - _Requirements: 1.1, 4.1_
- [ ] 2.4 `second_change_thread_test.lua` の候補名への追従確認 (P)
  - `scene_executor` 経由でのイベント名受け取り箇所を確認し、必要なら候補名に更新する
  - _Requirements: 1.1_

- [ ] 3. フォールバックチェーンの新規テストを追加する
- [ ] 3.1 フォールバック順序テストを追加する
  - 候補1（`"時報{HH}"`）が最初に検索されることを `scene_executor` の引数で検証する
  - 候補2（`"OnHour{HH}"`）、候補3（`"時報その他"`）、候補4（`"OnHourOther"`）の順で検索されることを検証する
  - _Requirements: 1.1_
- [ ] 3.2 早期打ち切りテストを追加する
  - 候補1（`"時報12"`）でヒットした場合、`scene_executor` への呼び出しが1回だけであることを検証する
  - _Requirements: 1.3_
- [ ] 3.3 全候補未発見テストを追加する
  - 全候補で `nil` を返すモックを設定し、`check_hour()` が `nil` を返すことを検証する
  - _Requirements: 1.2_
- [ ] 3.4 HH フォーマットテストを追加する (P)
  - `hour=0` のとき候補1が `"時報00"` であることを検証する
  - `hour=9` のとき候補1が `"時報09"` であることを検証する
  - `hour=12` のとき候補1が `"時報12"` であることを検証する
  - _Requirements: 2.1, 2.2_

- [ ] 4. サンプルゴースト辞書を新しい仕様に更新する
- [ ] 4.1 `＊OnHour` シーンを `＊時報その他` にリネームする
  - `talk.pasta` 内の3つの `＊OnHour` シーン定義を `＊時報その他` に変更する
  - シーン内のトーク内容・`＄時１２` 変数参照・アクター指定・コメント行は変更しない
  - _Requirements: 5.1_
- [ ] 4.2 時刻別シーン `＊時報12` を追加する
  - 既存の `＊時報その他` シーンの直前に `＊時報12` を1つ追加する
  - シーン内容は既存の `＊時報その他` を1つ参考にした最小限の例とする
  - _Requirements: 5.2_
