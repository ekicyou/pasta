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

### Requirement 4: 既存 `＊OnHour` シーンの後方互換性

**Objective:** 既存のゴースト辞書作者として、現在の `＊OnHour` シーン定義が引き続き動作することを期待する。それにより、フォールバック導入による辞書の書き換えが不要になる。

#### Acceptance Criteria
1. When 辞書に `＊OnHour` のみ定義されている（時刻別シーンなし）場合, the virtual_dispatcher shall フォールバック候補4（`＊OnHourOther`）として既存の `＊OnHour` シーンを検索する。
2. If 既存の辞書が `＊OnHour` のシーン名で定義されている場合, then the virtual_dispatcher shall フォールバック候補4の名前を `OnHourOther` とするため、既存辞書のシーン名を `＊OnHourOther` に変更する必要があることを明示する。

### Requirement 5: サンプルゴースト辞書の更新

**Objective:** hello-pastaサンプルゴーストの辞書を、新しいフォールバック仕様に合わせて更新する。それにより、リファレンス実装としての役割を維持する。

#### Acceptance Criteria
1. When サンプルゴースト辞書が更新された場合, the `talk.pasta` shall 既存の `＊OnHour` シーンを `＊OnHourOther`（または `＊時報その他`）にリネームする。
2. Where サンプルゴースト辞書にフォールバックの使用例を追加する場合, the `talk.pasta` shall 少なくとも1つの時刻別シーン（例: `＊時報12`）を含む。
