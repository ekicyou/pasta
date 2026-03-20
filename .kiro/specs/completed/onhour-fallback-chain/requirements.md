# Requirements Document

## Project Description (Input)
OnHourを呼ぶ代わりに、`act:find_scene()`候補として
1. ＊時報12（現在時刻より00-23）
2. ＊OnHour12（同上）
3. ＊時報その他
4. ＊OnHourOther
の4段階フォールバックを実施し、最初に　find_scene()でハンドラが得られた候補を呼ぶように変更して欲しい。「＊時報12」をシーン登録すれば正午の時報が書ける。

## Introduction

現在の `virtual_dispatcher.lua` の `check_hour()` は、時報発火時に固定のシーン名 `"OnHour"` で `create_scene_thread()` を呼び出す。この仕様では、時報ハンドラの解決に4段階フォールバックチェーンを導入し、辞書作者が時刻ごとの個別時報シーンを柔軟に定義できるようにする。

## Requirements

### Requirement 1: 4段階フォールバックチェーンによるOnHourシーン解決

**Objective:** ゴースト辞書作者として、特定時刻の時報シーンを個別に定義したい。それにより、時刻ごとに異なるトーク内容を簡潔に表現できるようになる。

#### Acceptance Criteria
1. When OnHour仮想イベントが発火した場合, the virtual_dispatcher shall 以下の順序で `act:find_scene()` を呼び出し、最初にハンドラが得られた候補のシーンスレッドを返す:
   - 候補1: `＊時報{HH}` （HHは現在時刻の24時間制2桁: 00〜23）
   - 候補2: `＊OnHour{HH}` （同上）
   - 候補3: `＊時報その他`
   - 候補4: `＊OnHourOther`
2. When いずれの候補でもハンドラが見つからない場合, the virtual_dispatcher shall `nil` を返す（時報は発火しない）。
3. When 候補1（`＊時報{HH}`）でハンドラが見つかった場合, the virtual_dispatcher shall 候補2〜4の検索を行わず、候補1のハンドラでシーンスレッドを生成する。

### Requirement 2: 時刻文字列のフォーマット

**Objective:** ゴースト辞書作者として、直感的な時刻シーン名を使いたい。それにより、シーン名の命名に迷わず辞書を記述できる。

#### Acceptance Criteria
1. The virtual_dispatcher shall フォールバック候補名の `{HH}` 部分を、現在時刻の24時間制で0埋め2桁にフォーマットする（例: 0時→`00`, 9時→`09`, 12時→`12`, 23時→`23`）。
2. When 辞書に `＊時報12` というシーンが定義されている場合, the virtual_dispatcher shall 正午（12時台）にそのシーンを選択する。

### Requirement 3: 既存の `＄時１２` 変数との互換性

**Objective:** ゴースト辞書作者として、既存の時刻変数が引き続き利用できることを期待する。それにより、フォールバックシーン内でも従来どおり `＄時１２` を参照できる。

#### Acceptance Criteria
1. When OnHourフォールバックチェーンが実行される前に, the virtual_dispatcher shall `act:transfer_date_to_var()` を呼び出し、`＄時１２` などの日時変数を設定済みにする。

### Requirement 4: 既存 `＊OnHour` シーンの移行方針

**Objective:** 既存の `＊OnHour` シーン定義を持つゴースト辞書作者に対して、新しいフォールバック仕様への移行方針を明確にする。それにより、辞書作者が更新作業を迷わず実施できる。

#### Background
`OnHour` をフォールバック候補5として追加する案（5段階化）も検討されたが、候補2 `OnHour{HH}`（例: `OnHour12`）と候補名が前方一致するためシーン解決に誤動作が生じる可能性がある。このため `OnHour` はフォールバック候補に含めない設計とした。

#### Acceptance Criteria
1. The virtual_dispatcher shall `OnHour` というシーン名をフォールバック候補として使用しない。
2. If 既存の辞書に `＊OnHour` シーンが定義されている場合, then 辞書作者は当該シーンを `＊OnHourOther`（フォールバック候補4）または時刻別シーン（`＊時報{HH}` / `＊OnHour{HH}` など）に移行する必要がある。

### Requirement 5: サンプルゴースト辞書の更新

**Objective:** hello-pastaサンプルゴーストの辞書を、新しいフォールバック仕様に合わせて更新する。それにより、リファレンス実装としての役割を維持する。

#### Acceptance Criteria
1. When サンプルゴースト辞書が更新された場合, the `talk.pasta` shall 既存の `＊OnHour` シーンを `＊OnHourOther`（または `＊時報その他`）にリネームする。
2. Where サンプルゴースト辞書にフォールバックの使用例を追加する場合, the `talk.pasta` shall 少なくとも1つの時刻別シーン（例: `＊時報12`）を含む。
