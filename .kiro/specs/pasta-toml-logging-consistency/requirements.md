# Requirements Document

## Project Description (Input)
**誰が困っているか**: ゴースト作者と、ゴーストの障害を調査する開発者。

**現状**: `pasta.toml` には読み込まれるのに使われない設定キー（`[lua] libs`・`[logging] rotation_days`）がある。ログについても、`request`・`unload`・プロセス終了時の `DllMain` detach で出したログが本番でファイルに残らず、`[logging] file_path` が不正なときは既定のログファイル `profile/pasta/logs/pasta.log` に書き続けるのに、マニュアル `reference/pasta-toml.md` は「ログファイルを作らない」と書いている。作者が設定を書いても効かず、障害の調査に要るログが残らない。

**何を変えるか**: `pasta.toml` のすべてのキーを「書けば効く」か「マニュアルにもサンプルにも無い」かのどちらかにする（U26・U32 は実装か撤去かを決め、棚卸の推奨は撤去）。FFI 入口スレッドのログを、ロガーが生きている範囲でゴーストのログファイルに残す。不正な `file_path` の挙動をマニュアルと一致させる。マニュアル（`reference/pasta-toml.md`・`internals/logging-encoding.md`・`internals/shiori.md`）を更新し、スキル `references/` を再生成する。詳細は同じディレクトリの `brief.md` を参照。

## Introduction

本仕様は、`pasta.toml` の設定キーとログ出力の挙動を、マニュアル（利用者向け設定の唯一の権威）と一致させるバグ修正である（ロードマップ Wave 1）。対象は次の 4 点である。

1. 読み込まれるが効かない設定キー `[lua] libs`（U26）と `[logging] rotation_days`（U32）を、マニュアル・サンプル・公開 API から撤去する（棚卸の推奨に従う。**前提 A1・A2**）。
2. `[lua] libs` の撤去後も組み込み向けに残す Rust API `RuntimeConfig::from_libs` で、pasta の動作に必要なライブラリを欠いた構成を渡すと、原因の分からない変換エラーで VM の構築が失敗する（要件フェーズで原因を確認済み: `package` が無いと `@pasta_search` の登録が失敗する）。これを明示的なエラーにする（**前提 A3**）。
3. SHIORI の FFI 入口（`request`・`unload`・`FreeLibrary` による detach）やアクタースレッドで出したログ、ゴーストの終了処理中のログが、ロガーが生きている間でもファイルに残らない問題を直す。
4. `[logging] file_path` が不正なときの挙動を「既定のログファイルへ書き続け、そのことを warn で知らせる」に確定し、マニュアルをそれに合わせる（棚卸の推奨に従う。**前提 A4**）。

各前提と未決事項は末尾の「前提と未決事項」にまとめる。

## Boundary Context

- **In scope**:
  - `pasta.toml` の `[lua]`（`libs`）と `[logging] rotation_days` の撤去（設定の読み取り・公開 API・サンプルゴースト・マニュアル・テスト）
  - `RuntimeConfig::from_libs` などで pasta に必要なライブラリを欠いた構成を渡したときのエラーの明示
  - pasta.dll（SHIORI）が出すログのうち、FFI 入口スレッド・アクタースレッド・ゴーストの終了処理で出すものの保存
  - 不正な `[logging] file_path` のときの挙動の確定と、それを知らせる warn
  - 上記を固定する自動テスト
  - マニュアルの該当章の更新とスキル `references/` の再生成
- **Out of scope**:
  - ログの書式・ログレベルの既定値（`info`）・既定のログファイルの場所の変更
  - デバッグバックエンド（DAP）が出すログ
  - ログのローテーション機能（撤去を選ぶため実装しない）
  - ゴーストから `@env`・`debug`・`ffi` などを有効にする手段（マニュアル X18 の決定を維持）
  - `pasta_lua` の Lua ランタイム（`pasta_scripts/`）、ローダのファイル探索・キャッシュ
  - リリース成果物ディレクトリ `release/hello-pasta/`（次のリリースで再生成される）
  - プロセス終了による `DLL_PROCESS_DETACH`（`unload` を経ない終了）でのログ出力（**前提 A7**）
- **Adjacent expectations**:
  - 公開 API の削除は破壊的変更であり、次のリリースでマイナーバージョンを上げる（`release-workflow` が担う。本仕様はリリース作業を行わない）。
  - 起動失敗の記録経路（`load-error-logging`・`lua-require-robustness`）の挙動を後退させない。
  - 棚卸の即時修正で直した挙動（`[logging]` の無い再読み込みでフィルタが既定に戻る、不正な `file_path` でも `level`・`filter` が効く、プロセス終了時の detach で待たない）を後退させない。

## Requirements

### Requirement 1: `[lua]` セクション（`libs`）の撤去

**Objective:** ゴースト作者として、`pasta.toml` に書けるキーはすべて効くものであってほしい。効かない `[lua] libs` に時間を使わないためである。

#### Acceptance Criteria
1. The pasta マニュアル shall `reference/pasta-toml.md` のセクション一覧・最小テンプレート・各セクションの節に `[lua]` を載せない。
2. The pasta マニュアル shall ゴーストが使える Lua 標準ライブラリと mlua-stdlib のモジュールを、`lua/modules/mlua-stdlib.md` だけで説明し、ゴーストからその構成を変える手段が無いことを示す。
3. When `pasta.toml` に `[lua]` セクション（`libs` を含む）が書かれたゴーストを読み込むとき, the pasta ローダ shall エラーにせずに読み込みを続け、Lua ライブラリの構成を既定のまま変えない。
4. The pasta_lua クレート shall `[lua]` セクションを読むための公開 API（`[lua]` の設定型、`pasta.toml` の設定からその型を取り出す関数、その型からランタイム構成への変換）を提供しない。
5. The pasta_lua クレート shall 組み込み向けのランタイム構成 API（ライブラリ名の一覧から構成を作る関数と、既定のライブラリ一覧）を引き続き提供する。

### Requirement 2: `[logging] rotation_days` の撤去

**Objective:** ゴースト作者として、ログがローテーションされると誤解させるキーを見たくない。ログファイルの扱いを正しく理解するためである。

#### Acceptance Criteria
1. The pasta マニュアル shall `rotation_days` をどの章にも載せない（内部設計の章を含む）。
2. The サンプルゴースト hello-pasta shall `pasta.toml` に `rotation_days` を書かない。
3. When `[logging]` に `rotation_days` が書かれたゴーストを読み込むとき, the pasta ローダ shall エラーにせず、同じ `[logging]` の `file_path`・`level`・`filter` を書かれたとおりに反映する。
4. The pasta_lua クレート shall `[logging]` の設定型に `rotation_days` を持たない。
5. The pasta ログ出力 shall 1 つのログファイルに追記し続け、日付による分割や古いファイルの削除をしない（現行の挙動を維持する）。

### Requirement 3: 必須ライブラリを欠いたランタイム構成の明示的なエラー

**Objective:** pasta_lua を組み込む開発者として、構成に必要なライブラリが欠けているとき、その理由が分かるエラーを受け取りたい。原因の分からない変換エラーで調査に時間を使わないためである。

#### Acceptance Criteria
1. If ランタイム構成のライブラリ一覧が pasta の動作に必須の Lua 標準ライブラリ（少なくとも `std_package`。ローダ経由で読み込むフレームワークスクリプトが要るものを含めた一覧は設計で確定する）を含まない, the pasta_lua ランタイム shall VM を構築せずに、欠けているライブラリ名を含む構成エラーを返す。
2. The pasta_lua ランタイム shall 必須ライブラリを含む構成（既定の構成・最小構成・全機能構成を含む）では、従来どおり VM を構築する。
3. The pasta_lua の Rust API ドキュメント shall ライブラリ一覧の説明で、どのライブラリが必須かを示す。

### Requirement 4: FFI 入口・アクタースレッド・終了処理のログの保存

**Objective:** ゴーストの障害を調査する開発者として、pasta.dll が出したログが、出したスレッドや時点によらずゴーストのログファイルに残ってほしい。`request`・`unload` まわりの異常を後から追えるようにするためである。

#### Acceptance Criteria
1. While ゴーストのロガーが登録されている, when SHIORI の `request` 入口がログを出す（不正なリクエストの warn、境界での panic の error、観測用の debug・trace を含む）, the pasta.dll shall そのログを、設定されたフィルタを満たす範囲でゴーストのログファイルに書く。
2. While ゴーストのロガーが登録されている, when アクタースレッドがゴーストの呼び出しの外でログを出す（メッセージの受信・応答・停止の観測ログを含む）, the pasta.dll shall そのログをゴーストのログファイルに書く。
3. When ゴーストの終了処理（`unload`、または `FreeLibrary` による `DLL_PROCESS_DETACH`）が行われる, the pasta.dll shall `SHIORI.unload` の呼び出し・永続化データの保存の失敗・ロガーの登録解除など、終了処理の間に出したログを、ログファイルを閉じる前にゴーストのログファイルに書く。
4. If `unload` の teardown で異常（終了の確認の待ち時間切れ、または確認の経路の切断）が起きたとき、ゴーストのロガーがまだ登録されている, the pasta.dll shall その warn をゴーストのログファイルに書く（**前提 A6**）。
5. When `loadu` で初期化済みのときに `load` が呼ばれて無視される, the pasta.dll shall その warn をゴーストのログファイルに書く。
6. While 同じプロセスに複数のゴーストのロガーが登録されている, the pasta.dll shall あるゴーストに属すると決められないログを、別のゴーストのログファイルに書かない。
7. While ゴーストのロガーが 1 つも登録されていない（最初の `load` の前、または終了処理の完了後）, the pasta.dll shall ログを捨て、SHIORI の応答・戻り値を変えず、ホストを待たせない。
8. The pasta.dll shall ログの保存のために、SHIORI の応答内容・応答までの待ち時間の上限・`unload` の戻り値を変えない。

### Requirement 5: 不正な `[logging] file_path` のときの挙動

**Objective:** ゴースト作者として、`file_path` を書き間違えたとき、ログがどこへ行くかと、なぜそうなったかを知りたい。ログを失わずに設定を直せるようにするためである。

#### Acceptance Criteria
1. If `[logging] file_path` が条件（設置ディレクトリからの相対パスで、`profile` で始まり、`..` を含まない）を満たさない, the pasta.dll shall ゴーストの起動を続け、既定のログファイル `profile/pasta/logs/pasta.log` にログを書き続ける（**前提 A4**）。
2. If `[logging] file_path` が条件を満たさない, the pasta.dll shall 不正と判断した `file_path` の値と、既定のログファイルへ書き続けることを示す warn を、既定のログファイルに書く。
3. If `[logging] file_path` が条件を満たさない, the pasta.dll shall 同じ `[logging]` の `level`・`filter` を反映する（現行の挙動を維持する）。
4. When 不正な `file_path` で読み込んだゴーストを、正しい `file_path` に直して再読み込みする, the pasta.dll shall 以後のログを直した `file_path` のファイルに書く。
5. The pasta.dll shall `file_path` が条件を満たすかどうかの判定基準を変えない。

### Requirement 6: マニュアル・サンプル・スキルの一致

**Objective:** ゴースト作者として、マニュアルとスキルに書かれた挙動が実際の挙動と一致していてほしい。マニュアルが利用者向け設定の唯一の権威だからである。

#### Acceptance Criteria
1. The pasta マニュアル shall `reference/pasta-toml.md` の `[logging]` の節と `reference/startup.md` で、不正な `file_path` のときに既定のログファイルへ書き続けることと、warn が出ることを説明する。
2. The pasta マニュアル shall `internals/logging-encoding.md` で、ログがゴーストのログファイルへ届く条件（どのスレッド・どの時点のログが残り、どの場合に捨てられるか）と、不正な `file_path` のときのロガーの扱いを、本仕様の挙動のとおりに説明する。
3. The pasta マニュアル shall `internals/shiori.md` で、`request`・`unload`・`DllMain` の detach・終了処理のログの扱いを、本仕様の挙動のとおりに説明する。
4. The pasta マニュアル shall `internals/loader.md` など内部設計の章から、撤去した設定型（`[lua]` の設定型）と `rotation_days` への言及を除く。
5. When マニュアルの章を更新したとき, the pasta リポジトリ shall 生成スキル `references/`（`pasta-ghost-authoring`・`pasta-lua-coding`）を再生成し、マニュアルとの差分検査を通す。
6. The pasta リポジトリ shall マニュアル・サンプルゴースト・スキル `references/`・クレートの README のどこにも、効かない `pasta.toml` のキーを載せない。

### Requirement 7: 回帰を防ぐ自動テスト

**Objective:** pasta の開発者として、本仕様で直した挙動が将来の変更で戻らないようにしたい。

#### Acceptance Criteria
1. The pasta テストスイート shall `[lua] libs` と `rotation_days` を書いた `pasta.toml` でゴーストが読み込めることを検証する。
2. The pasta テストスイート shall 必須ライブラリを欠いた構成が、欠けたライブラリ名を含む構成エラーになることを検証する。
3. The pasta テストスイート shall SHIORI の FFI 入口から `load`・`request`・`unload` を通したとき、`request` 入口と終了処理で出したログがゴーストのログファイルに残ることを検証する。
4. The pasta テストスイート shall 不正な `file_path` で読み込んだとき、既定のログファイルに warn とその後のログが残ることを検証する。
5. The pasta テストスイート shall ロガーが登録されていないときのログが、エラーや panic を起こさずに捨てられることを検証する。

## 前提と未決事項

要件ディスカッションで確定させる。「確定」は brief.md・正典（マニュアルの決定・プロジェクトの原則）から自明として議題にせず閉じたもの、「議題」は開発者と 1 件ずつ確認するもの、「設計」は設計フェーズで決めるもの。

| ID | 関連要件 | 状態 | 前提 | 代替案・根拠 |
| -- | -------- | ---- | ---- | ------------ |
| A1 | 1 | 確定 | `[lua] libs` は撤去する（ゴーストから Lua ライブラリ構成を変える手段を設けない） | 実装は X18 の決定とマニュアル 2 ページを覆すセキュリティ上の変更になるため採らない。`[lua]` 節はスタブも残さず消す（brief の「マニュアルにもサンプルにも無い」） |
| A2 | 2 | 確定（議題 1） | `rotation_days` は撤去し、ログは 1 ファイル追記のまま据え置く | 本仕様は Wave 1 のバグ修正で、ローテーションは新機能。必要になれば、`file_path` の意味を保つ自前の起動時ローテーションを別仕様で起こす（日次の `tracing_appender` はファイル名が変わるため採らない） |
| A3 | 3 | 確定 | `from_libs` の失敗は本仕様の範囲に入れ、必須ライブラリが欠けた構成を明示的なエラーにする | 「問題があれば黙って補わずに止める」原則に従う。`package` を常に足す案は利用者の除外指定を黙って上書きするため採らない。必須の一覧は設計で確定する |
| A4 | 5 | 確定 | 不正な `file_path` では既定のログファイルへのフォールバックを残し、warn を出し、マニュアルを直す | brief の推奨どおり。SHIORI では既に実挙動で、ログファイルを作らない案は起動ログを失う |
| A5 | 1, 2 | 議題 | 撤去したキーが書かれていても警告は出さず、他の未知のキーと同じく黙って無視する | 読み込み時に「効かないキー」を 1 回 warn で知らせる |
| A6 | 4.4 | 確定 | teardown の異常の warn は、ロガーが登録されている間に起きたものを残す | brief の「ロガーが生きている範囲で」に従う。切断（Disconnected）は unwind の panic でしか起きず、release（`panic=abort`）では起きない。`PastaShiori::drop` の順序を直せば待ち時間切れはロガーの登録中に起きる |
| A7 | Boundary | 確定 | プロセス終了による `DLL_PROCESS_DETACH`（`unload` を経ない終了）では何もしない現行の挙動を保ち、ログの保存の対象にしない | 現行はこの経路でログを出さない。プロセス終了中は書き込み用のスレッドが止まっており、ロガーは「生きている範囲」の外 |
| A8 | 5 | 議題 | `PastaLoader` を SHIORI 以外から直接使う組み込みの場合は、不正な `file_path` のときロガーを作らない現行の挙動を保ち、内部設計の章に書くだけにする | 組み込みでも既定のログファイルへフォールバックする |
| A9 | 4 | 確定 | デバッグバックエンドのログは範囲外とし、保存の挙動を保証しない（実現方法によっては副次的に残るようになってもよい） | brief の Out of scope は「デバッグバックエンドのログ（を対象にすること）」。設計の自由度を残す |
| A10 | 5.5 | 議題 | `file_path` の判定基準（文字列として `profile` で始まる）は変えない | `profile` ディレクトリの下であることを厳密に判定する（`profiles/`・`profile.log` などを拒否する） |
| D1 | 1.5 | 設計 | `default_libs` の定義場所と公開パス（`pasta_lua::default_libs`・`pasta_lua::loader::default_libs`）の維持 | `LuaConfig` と同じ `sections.rs` から `runtime_config.rs` へ移すか |
| D2 | 3.1 | 設計 | 必須ライブラリの一覧とエラーの種類・文言 | `std_package` のほか、ローダ経由の `pasta_scripts` が要る `string`・`table`・`coroutine` などを調査して確定 |
| D3 | 4 | 設計 | 入口・アクタースレッドのログの振り分け方式（research.md 4.1 の案 A〜C）と `PastaShiori::drop` の順序 | 複数ロガー時の誤配防止（4.6）と、テストの並列実行下での E2E 検証の作り方を含む |
