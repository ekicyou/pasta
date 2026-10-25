# Requirements Document

## Project Description (Input)
**誰が困っているか**: Windows 以外の互換ベースウェア（areka など）でゴーストを動かしたい利用者と、その上で pasta のゴーストを読みたい作者。

**今の状況**: pasta.dll は Windows でしか動かない。入口は SHIORI/3.0 の DLL の形（`load`・`loadu`・`request`・`unload` と HGLOBAL）だけで、Lua VM を動かすアクタースレッドの実行器は Windows のメッセージ専用ウィンドウ（`wintf-winmsg-executor`）に頼っている。アクターの mailbox などはプロセスに 1 つの static に置かれている。

**何を変えるか**: pasta.dll が shiori4（crates.io 0.1.0）の版 3（SHIORI3 互換モード）の入口 `shiori_get_api(3)` を書き出し、SHIORI/3.0 の要求の全文をそのまま今の処理で扱う。版 4 を求められたら NULL を返す。アクターの実行器を OS に頼らない形にし、同じソースから Linux・macOS の共有ライブラリ（`libpasta.so`・`libpasta.dylib`）をビルドしてテストが通るようにする（CI で守る）。エンジンは状態をインスタンスごとに持ち、グローバルな状態を避ける。Windows の今の入口と SSP での動きは変えない。版 4 への本格対応は `shiori4-v4-host`（0.5.0）が持つ。詳しくは `brief.md`。

## Introduction

この spec は、pasta を「SHIORI/3.0 を Windows 以外でも動かせる SHIORI」にする。shiori4 の版 3 の入口を足し、アクターの実行器から Windows への依存を外し、Linux・macOS でビルドとテストを通す。SSP から見た pasta.dll は何も変わらない。

主な利用者は 4 人である。

- **ゴーストの作者**: 今の辞書（`.pasta`・`scripts/`）のまま、SSP でも、shiori4 を知るベースウェア（areka など）でも、同じように動くことを期待する。
- **ベースウェアの開発者（areka）**: shiori4 0.1.0 の `shiori4_host` で pasta を読む。Windows 以外では `libpasta.so`・`libpasta.dylib` を、Windows では `pasta.dll` の `shiori_get_api` を使う。
- **pasta の開発者**: 1 つのソースを、Windows・Linux・macOS の CI で守りたい。
- **後に続く spec**: `shiori4-v4-host`（同じ入口に版 4 を足す）・`ponytail-shiori`（同じ `windows.rs`・`actor/` の上で直す）。

shiori4 を知るホストから見た 1 往復は、次のようになる。

```
ホスト（Linux の areka）                         pasta（libpasta.so）
shiori_get_api(4)                            →  NULL（版 4 はまだ出さない）
shiori_get_api(3)                            →  版 3 のテーブル
load("/home/u/ghost/hello-pasta/ghost/master/")
                                             →  インスタンス
request("GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: areka\r\nID: OnBoot\r\n\r\n")
                                             →  戻り値 200、out = "SHIORI/3.0 200 OK\r\n…Value: \0…\e\r\n\r\n"
                                                （旧来の request が返すのと同じバイト列）
notify("NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nID: OnSecondChange\r\n\r\n")
                                             →  戻り値 204（処理はアクターが後で行う）
unload(インスタンス)                          →  SHIORI.unload・セーブの書き出しを終えて返る
```

SSP から見た動きは今のまま（`loadu`→`request`→`unload`、HGLOBAL の受け渡し）である。

## Boundary Context

- **In scope**
  - `shiori_get_api` の書き出し（版 3 だけを返す。版 4 とそれ以外は NULL）と、版 3 の `load`・`request`・`notify`・`unload` を今のアクターへつなぐこと
  - エンジン（Lua VM・アクター・辞書・変数・セーブ・ログの出し先）を、インスタンスごとに持つ作りにすること。プロセスに 1 つしかないものを、インスタンスへ移すか、移せない理由を書いて閉じ込めること
  - 同じプロセスでの 2 つ目の `load` の扱い（受ける）
  - アクタースレッドの実行器を、OS のウィンドウやメッセージの仕組みに頼らない形にすること（全部の OS で同じ形）
  - Windows だけの依存を Windows のビルドに閉じ込め、Linux・macOS で共有ライブラリのビルドとテストを通すこと
  - CI（`build.yml`）に Linux（と macOS）のビルド・テスト・clippy と、shiori4 の試験用のホストで版 3 を 1 往復させる段を足すこと
  - 旧来の入口の FFI の型を C に合わせること（`load`・`loadu`・`unload` の戻り値を 32 ビットの int の 1／0 に、`request` の長さを C の `long` に）
  - `unload` が返るとき、アクタースレッドがライブラリのコードを出ていること（版 3 と旧来の入口の両方。`DllMain` の中の片付けを除く）
  - マニュアルの内部の章（`internals/shiori.md`）・`crates/pasta_shiori/README.md`・`tech.md` の更新
- **Out of scope**
  - 版 4（ホストへの同期の問い合わせ・post・OnHostComplete・Lua の API）。`shiori4-v4-host`・`shiori4-v4-lua-api` が持つ
  - 旧来の HGLOBAL の入口で複数のインスタンスを扱うこと（1 つのまま）
  - SHIORI/3.0 の要求の解析と、Lua に渡す `req` の形（変えない）。`pasta_lua` の Lua の振る舞い
  - Windows 以外の配布物をリリースに載せること（CI のビルドとテストで守るだけ）
  - ゴーストの配布（nar）と `pasta_check` の、Windows 以外での働き（`pasta_check` のテストが Linux で通るかは Requirement 9 で扱う）
  - `DllMain` の中の片付け（FreeLibrary の detach）で、アクタースレッドの終わりを待つこと（loader lock の中では待てない）
  - 旧来の入口の HGLOBAL の受け渡しを shiori4 のクレートへ移すこと（2026-10-24 の discovery の決定。移さない）
  - リリースのビルドの panic の扱い（`panic = "abort"`）を変えること
- **Adjacent expectations**
  - shiori4 0.1.0（`shiori4_abi`）の版 3 の取り決めに従う。応答の `out` は常にホストが作り、pasta は `copy_from_slice` で写す（写しは `shiori4_abi` のテーブルの関数が行う）。0.0.2 以前のホストでは読めない（`load` が NULL）。
  - ホストは `shiori_get_api` → `loadu` → `load` の順に探す。pasta は版 4 に NULL を返すので、shiori4 を知るホストは版 3 で読み、SSP は旧来の口で読む。版 3 の `load` が NULL のとき、ホストは旧来の口へ進まない。
  - areka は SHIORI ごとに helper のプロセスを起こすので、今は同じプロセスで 2 つ目の `load` は来ない。それでも受けられる作りにする（2026-10-23 開発者の決定）。
  - `ponytail-shiori` は、この spec が入った後の main で `windows.rs`・`actor/` の上を直す（同じウェーブに置かない）。あちらの 1 番（unload と FreeLibrary の競り）・4 番（FFI の型）は、この spec で閉じる。`shiori4-v4-host` は、この spec の入口と実行器の上に版 4 を足す。

## Requirements

### Requirement 1: shiori4 の入口の書き出し
**Objective:** As a ベースウェアの開発者, I want pasta のライブラリが shiori4 の入口を持つこと, so that shiori4 0.1.0 のホストから pasta を版 3 で読める

#### Acceptance Criteria
1. The pasta のライブラリ shall 公開シンボル `shiori_get_api` を書き出す（Windows の `pasta.dll` では、旧来の `load`・`loadu`・`request`・`unload` と並べて書き出す）。
2. When ホストが `shiori_get_api(3)` を呼ぶとき, the pasta のライブラリ shall 版 3（`abi_version` が 3）の関数テーブルを返す。
3. When ホストが `shiori_get_api` に 3 以外の版（4 を含む）を渡すとき, the pasta のライブラリ shall NULL を返す。
4. The pasta のライブラリ shall 何度 `shiori_get_api(3)` を呼ばれても、プロセスの間ずっと有効な同じテーブルを返す。
5. When shiori4 0.1.0 のホストが、許可する口に版 4・版 3・`loadu`・`load` を全部入れて `pasta.dll` を探すとき, the pasta.dll shall 版 3 で解決される（版 4 は NULL・旧来の口へは進まない）。

### Requirement 2: 版 3 の load
**Objective:** As a ゴーストの作者, I want 版 3 で読まれたときも SSP と同じようにゴーストが立ち上がること, so that ベースウェアを選ばずに同じ辞書が動く

#### Acceptance Criteria
1. When ホストが版 3 の `load` に、ゴーストのディレクトリ（絶対パス・OS の区切り・末尾に区切りあり・UTF-8）を渡すとき, the pasta のライブラリ shall そのディレクトリから旧来の `load` と同じ手順でゴーストを読み込み、インスタンスを返す。
2. The pasta のライブラリ shall 版 3 の `load` で、カレントディレクトリに頼らず、渡されたディレクトリだけを使う。
3. When 版 3 の `load` がインスタンスを返したとき, the pasta のライブラリ shall その時点で、そのインスタンスへの `request`・`notify` を受けられる状態にある（ゴーストの読み込みと `SHIORI.load` の呼び出しを終えている）。
4. If 渡されたディレクトリが無いとき、またはアクタースレッドを起こせないとき（SHIORI として立ち上がれない失敗）, the pasta のライブラリ shall NULL を返し、作ったスレッドと資源を残さない。
5. If 版 3 の `load` が NULL を返すとき, the pasta のライブラリ shall 理由を pasta 自身のログ（ゴーストのディレクトリの下のログ）に出す。ディレクトリが無くてログを置けないときは、ログを出さずに NULL を返してよい。
6. If ホストが渡したテーブルに `copy_from_slice` の欄が無いとき, the pasta のライブラリ shall NULL を返す（shiori4 0.1.0 の ABI 規約。0.0.2 以前のホスト）。
7. If 辞書・Lua のスクリプトの読み込み、または `SHIORI.load` が失敗するとき, the pasta のライブラリ shall NULL を返さずにインスタンスを返し、以後の GET に旧来の入口と同じ 500 の応答（理由の `X-ERROR-REASON` つき）を返す（作者が書き間違いの原因をホストの上で見られるため）。

### Requirement 3: 版 3 の request と notify
**Objective:** As a ゴーストの作者, I want 版 3 の要求に、旧来の入口と同じ応答が返ること, so that SSP と areka で台詞やイベントの反応が変わらない

#### Acceptance Criteria
1. When 版 3 の `request` が `GET SHIORI/3.0` で始まる要求の全文を受けるとき, the pasta のライブラリ shall 旧来の `request` に同じ要求を渡したときと同じバイト列の応答の全文を `out` に返し、戻り値を応答の開始行のステータスコード（200・204・500 など）にする。
2. When 版 3 の `request` が `NOTIFY SHIORI/3.0` で始まる要求の全文を受けるとき, the pasta のライブラリ shall 旧来の `request` と同じく、処理をアクターに渡して待たずに 204 の応答の全文を返す（戻り値 204）。
3. When 版 3 の `notify` が `NOTIFY SHIORI/3.0` で始まる要求の全文を受けるとき, the pasta のライブラリ shall 処理をアクターに渡して待たずに 204 を返す。
4. If 版 3 の `notify` が `GET` の要求を受けるとき, the pasta のライブラリ shall 400 を返し、その要求を Lua に渡さない。
5. If 版 3 の `request` または `notify` が SHIORI/2.x の要求を受けるとき, the pasta のライブラリ shall 400 を返し、その要求を Lua に渡さない（shiori4 の版 3 の規約）。
6. If 版 3 の `request` が、メソッド（GET・NOTIFY）を読み取れない要求を受けるとき, the pasta のライブラリ shall 400 の応答を返し、その要求を Lua に渡さない（旧来の入口は今どおり 204 を返す）。
7. If Lua が返した応答の開始行からステータスコードを読み取れないとき, the pasta のライブラリ shall 戻り値を 500 にし、応答の本文は Lua が返したまま `out` に返す。
8. While GET の処理が今の安全網の時間（5 秒）を超えるとき, the pasta のライブラリ shall 旧来の入口と同じく 204 の応答を返し、ホストのスレッドを止め続けない。
9. The pasta のライブラリ shall 版 3 で受けた要求を、旧来の入口で受けたときと同じ形で Lua に渡す（`req` の表・`SHIORI.request` の呼ばれ方・コルーチンの続き方が同じ）。
10. The pasta のライブラリ shall 版 3 の応答を UTF-8 で返す（旧来の入口と同じく `Charset: UTF-8`）。

### Requirement 4: 版 3 の unload
**Objective:** As a ベースウェアの開発者, I want 版 3 の `unload` が後始末を終えて、長く止まらずに返ること, so that OS の終了の猶予の中でも安全にゴーストを閉じられる

#### Acceptance Criteria
1. When ホストが版 3 の `unload` を呼ぶとき, the pasta のライブラリ shall 旧来の `unload` と同じ後始末（`SHIORI.unload` の呼び出し・セーブの書き出し・ログの登録の解除・デバッグの待受の解放・アクターの停止）を、そのインスタンスについて行う。
2. If アクターが後始末の完了を知らせないとき, the pasta のライブラリ shall 上限の時間（今の 5 秒）が過ぎたら待つのをやめ、アクタースレッドを手放したことをログに出して返る。
3. The pasta のライブラリ shall 1 つのインスタンスの `unload` で、ほかのインスタンスを止めず、ほかのインスタンスの応答を変えない。
4. When アクターが上限の時間の中で後始末の完了を知らせたとき, the pasta のライブラリ shall アクタースレッドが終わるのを待ってから `unload` を返す（返った直後にホストがライブラリを外しても、pasta のコードを走るスレッドが残らない）。

### Requirement 5: インスタンスごとの状態
**Objective:** As a pasta の開発者, I want エンジンの状態がインスタンスごとに分かれていること, so that グローバルな状態に頼る、原因の分かりにくい不具合を避けられる

#### Acceptance Criteria
1. The pasta のエンジン shall Lua VM・アクタースレッド・要求の受け口（mailbox）・辞書・変数・セーブ・ログの出し先を、インスタンスごとに持つ。
2. While 1 つのプロセスに 2 つ以上のインスタンスがあるとき, the pasta のエンジン shall 各インスタンスへの要求を、そのインスタンスの Lua VM だけで処理する（変数・チェイントークの続き・自動トークの停止の状態が、ほかのインスタンスへ漏れない）。
3. While 1 つのプロセスに 2 つ以上のインスタンスがあるとき, the pasta のエンジン shall 各インスタンスのログを、そのインスタンスのゴーストのログのファイルだけに書く。
4. While 1 つのプロセスに 2 つ以上のインスタンスがあるとき, the pasta のエンジン shall 各インスタンスの `pasta.toml` の `[logging]` の量の設定を、そのインスタンスのログだけに効かせる。
5. When デバッガ（DAP）がシーンの再生を頼むとき, the pasta のエンジン shall その頼みを、そのデバッガがつながっているインスタンスのアクターだけに届ける。
6. When 2 つのインスタンスの `pasta.toml` が同じデバッグのポートを指すとき, the pasta のエンジン shall 後から読んだほうを、警告を出してデバッグなしで立ち上げる（今の振る舞い。読み込みは失敗しない）。
7. The pasta のエンジン shall プロセスに 1 つの状態を、次のものだけに限り、それぞれ理由を内部の文書に書く: 旧来の入口のインスタンスの置き場と `loadu` で初期化済みかの印（ハンドルの無い ABI のため。旧来の入口の中に閉じ込める）、振る舞いに効かない数を振るだけのもの（要求の連番など）、tracing の購読者（Rust の tracing の既定の購読者はプロセスに 1 つの仕組みのため）と、そこからログをゴーストのディレクトリごとに振り分けるロガーの登録（`REGISTRY`）。

### Requirement 6: 同じプロセスでの 2 つ目の load
**Objective:** As a ベースウェアの開発者, I want 同じプロセスで 2 つ目の `load` を受けてもらえること, so that ゴーストごとにプロセスを分けないホストでも pasta を複数読める

#### Acceptance Criteria
1. When インスタンスがすでにあるプロセスで、別のゴーストのディレクトリを渡して版 3 の `load` が呼ばれるとき, the pasta のライブラリ shall 断らずに、独立した新しいインスタンスを返す（3 者会談の「できれば受ける」と、エンジンを複数持てる作りにする開発者の決定による）。
2. If 同じプロセスで、生きているインスタンスがすでに読んでいるのと同じゴーストのディレクトリを渡して版 3 の `load` が呼ばれるとき, the pasta のライブラリ shall NULL を返し、理由を pasta のログに出す（セーブとログのファイルを 2 つのインスタンスが取り合わないため）。見分けには、Requirement 5.7 に挙げたもの以外のプロセスに 1 つの状態を足さない。
3. When 2 つのインスタンスが、それぞれ別のホストのスレッドから呼ばれるとき, the pasta のライブラリ shall それぞれを正しく処理する（shiori4 の「単一のスレッドはインスタンスごと」の規約）。
4. When 旧来の入口で読んだインスタンスがあるプロセスで、版 3 の `load` が呼ばれるとき, the pasta のライブラリ shall 旧来の入口のインスタンスを止めずに、別のインスタンスとして扱う（Requirement 6.2 の同じディレクトリの場合を除く）。

### Requirement 7: 旧来の SHIORI/3.0 の入口の保存
**Objective:** As a ゴーストの作者, I want SSP での pasta.dll の動きが変わらないこと, so that 今のゴーストをそのまま配れる

#### Acceptance Criteria
1. The pasta.dll shall Windows で `load`・`loadu`・`request`・`unload` を書き出し、今と同じ振る舞い（HGLOBAL の所有の受け渡し・`loadu` の優先・`loadu` の後の `load` は何もしない・`DllMain` の FreeLibrary での片付け・プロセスの終了では何もしない・ロードが失敗してもアクターを残し、以後の GET に今と同じ 500 を返す）を保つ。
2. The pasta.dll shall 旧来の入口では、インスタンスを 1 つだけ使う（2 回目の `load` は、今と同じく前のインスタンスを片付けてから読み直す）。
3. The pasta.dll shall 旧来の入口の `load`・`loadu`・`unload` の戻り値を 32 ビットの int（成功は 1、失敗は 0）で返し、`load`・`loadu` の長さと `request` の長さ（受けと返しの両方）を C の `long` で受け渡す（x86 の SSP から見た振る舞いは今と同じ）。
4. The pasta.dll shall 旧来の入口の今のテスト（`windows_tests.rs`・`shiori_lifecycle_tests.rs`・`tests/ffi_*`・`tests/shiori_*` など）を、期待を変えずに通す。
5. The pasta のワークスペース shall Windows の出荷物を 1 種類（`pasta.dll`。静的 CRT のリンク・デバッグの待受は 127.0.0.1 固定）のまま保ち、増えるのは `shiori_get_api` の書き出しだけにする。
6. When 旧来の `unload` が呼ばれるとき（`DllMain` の中の片付けを除く）, the pasta.dll shall Requirement 4.2・4.4 と同じく、上限の時間の中で後始末が終わればアクタースレッドの終わりを待ってから返る。

### Requirement 8: OS に頼らないアクターの実行器
**Objective:** As a pasta の開発者, I want アクタースレッドが OS のウィンドウやメッセージの仕組みに頼らないこと, so that 同じ実行器が Windows・Linux・macOS で動く

#### Acceptance Criteria
1. The pasta のアクター shall Windows・Linux・macOS で、OS のウィンドウやメッセージの仕組みを使わずに要求を待ち、処理する。
2. While Windows で動いているとき, the pasta のアクター shall ウィンドウ（USER オブジェクト）を作らない（全部の OS で同じ形にする）。
3. The pasta のアクター shall 今の受け渡しの約束（GET は応答を待つ・NOTIFY は即 204・要求は届いた順に 1 つずつ処理・コルーチンは要求をまたいで続く・停止は先の要求を処理し終えてから後始末の完了を知らせる・デバッガのシーンの再生の頼みを同じ列で処理する）を、全部の OS で保つ。
4. When `load` と `unload` を繰り返すとき, the pasta のアクター shall スレッドやハンドルを漏らさない（Windows では今の漏れの検査で確かめる。測る手段の無い OS では測らない）。

### Requirement 9: Windows 以外でのビルドとテスト
**Objective:** As a pasta の開発者, I want 同じソースから Linux・macOS の共有ライブラリを作り、テストを通せること, so that Windows 以外のホストで動くことを守れる

#### Acceptance Criteria
1. When `pasta_shiori` を Linux（x64）向けにビルドするとき, the pasta のワークスペース shall `shiori_get_api` を書き出す `libpasta.so` を作る（shiori4 のファイル名の決まりの名前。旧来の HGLOBAL の入口は含めない）。
2. When `pasta_shiori` を macOS 向けにビルドするとき, the pasta のワークスペース shall `shiori_get_api` を書き出す `libpasta.dylib` を作る。
3. The pasta のワークスペース shall Windows だけの依存（Windows の API・HGLOBAL・ANSI のコードページ・LuaJIT の FFI の DLL の読み込みの置き換え・静的 CRT の指定）を Windows のビルドに閉じ込め、Linux・macOS のビルドで要求しない。
4. When Linux・macOS でワークスペースのテストを全部走らせるとき, the pasta のワークスペース shall すべてのテストを通す。Windows だけの働き（HGLOBAL・ANSI のコードページ・DLL の読み込み・Windows の資源の測りなど）を試すテストは Windows だけで走らせ、その理由をテストに書く（ライブラリの 4 クレートに絞らず、ワークスペース全体）。
5. When Linux で clippy を警告を誤りとして扱って全ターゲットに走らせるとき, the pasta のワークスペース shall 警告を 1 つも出さない。
6. When shiori4 0.1.0 のホストが、Windows（x64）・Linux（x64）・macOS のそれぞれで、pasta のライブラリを版 3 で読み、要求を送って閉じるとき, the pasta のライブラリ shall 2xx の応答を返し、エラーなく閉じる（ライブラリの境界をまたぐ試験）。
7. The pasta のワークスペース shall リリースのビルドの panic の扱い（abort）を変えない。版 3 の入口がパニックを `SHIORI_E_PANIC` で返せるのは unwind のビルド（開発・テスト）だけであり、リリースではプロセスが終わることを内部の文書に書く。

### Requirement 10: CI
**Objective:** As a pasta の開発者, I want Windows 以外のビルドと版 3 の 1 往復を CI が毎回確かめること, so that Windows だけで開発していても壊れたことに気づける

#### Acceptance Criteria
1. When `build.yml` が走るとき（push・PR・手動・リリースの関門からの呼び出し）, the CI shall Linux（x64）で、ワークスペースのビルド・テスト・clippy（警告を誤りとして扱う）を走らせる。
2. When `build.yml` が走るとき, the CI shall macOS で、ワークスペースのビルドとテストを走らせる。
3. When `build.yml` が走るとき, the CI shall Windows（x64）・Linux（x64）・macOS で、shiori4 0.1.0 の試験用のホスト（`shiori4-testhost`）に `--entry v3` を付けて見本のゴースト（hello-pasta）を読ませ、`OnBoot` を送り、終了の状態 0（2xx）を確かめる（Windows の x86 は、x64 のホストが読めないので対象にせず、旧来の入口の今のテストで守る）。
4. If Linux・macOS のジョブ、または版 3 の 1 往復の段が失敗するとき, the CI shall `build.yml` 全体を失敗にする（リリースの関門も止まる）。
5. The CI shall Windows 以外のビルドの成果物を、リリースの配布物に加えない（リリース CI の配布物は今の 5 つのまま）。

### Requirement 11: 文書の更新
**Objective:** As a pasta の開発者, I want 2 つの入口と実行器の作りが文書に書かれていること, so that 後に続く spec（`shiori4-v4-host`・`ponytail-shiori`）が正しい前提で始められる

#### Acceptance Criteria
1. The マニュアルの内部の章（`book/src/internals/shiori.md`） shall 2 つの入口（旧来の HGLOBAL の入口と shiori4 の版 3）と同じ芯へのつながり、インスタンスごとの状態、OS に頼らない実行器、プロセスに 1 つ残す状態とその理由、リリースの panic の扱いを書く。
2. The `crates/pasta_shiori/README.md` shall 依存の表（実行器の依存を外し、`shiori4_abi` を足す）と、Linux・macOS でのビルドの仕方を書く。
3. The `.kiro/steering/tech.md` shall `pasta_shiori` の依存と、エンジン駆動モデル・CI の説明を、この spec の後の形にする。
4. The 利用者向けのマニュアル shall Windows 以外の配布物が無いあいだ、Windows 以外での導入の手順を書かない（作者のすることは変わらないので、利用者向けの章には書き足さない）。

## 要件のディスカッションで決めたこと（2026-10-25）

開発者が全議題を推奨案で承認した。

- **版 3 の load の失敗（Requirement 2.4・2.5・2.7）**: SHIORI として立ち上がれない失敗（ディレクトリが無い・アクタースレッドを起こせない）だけ NULL。辞書・Lua・`SHIORI.load` の失敗ではインスタンスを返し、GET に旧来と同じ 500＋`X-ERROR-REASON` を返す。理由: 書き間違いの原因が分かる pasta の方針を、shiori4 のホストの上でも保つため。
- **同じディレクトリの 2 つ目（Requirement 6.2・5.7）**: 断る（NULL＋ログ）。見分けに新しいグローバルな状態を足さない。tracing の購読者とロガーの登録（`REGISTRY`）は、プロセスに 1 つ残すものとして Requirement 5.7 に理由つきで挙げる。
- **ログの量（Requirement 5.4・5.7）**: インスタンスごとに分ける。分けられなかったときの「共有」の逃げ道は置かない。作り方は設計で決める。
- **unload とライブラリを外すことの競り（Requirement 4.2・4.4・7.6）**: この spec で直す。後始末の完了が上限の中で届けばスレッドの終わりを待ってから返り、上限を過ぎたら手放してログに出す。`DllMain` の中の片付けは loader lock のため除く。`ponytail-shiori` の 1 番を閉じる。
- **旧来の入口の FFI の型（Requirement 7.3）**: `BOOL`（i32 の 1／0）と、長さの C の `long` を両方この spec で直す。`ponytail-shiori` の 4 番を閉じる。
- **版 3 の変な要求と応答（Requirement 3.6・3.7）**: メソッドを読めない要求は 400（旧来の入口は 204 のまま）。開始行が壊れた応答は戻り値 500・本文はそのまま。
- **unload の待ちの上限（Requirement 4.2）**: 5 秒のまま。
- **実行器（Requirement 8.2）**: 全部の OS で同じ形にし、Windows でもウィンドウを作らない。
- **macOS と配布物（Requirement 9.2・10.2・10.3・10.5）**: macOS も CI でビルド・テスト・版 3 の 1 往復をする。Windows 以外の配布物は載せない。
- **Linux で回すテスト（Requirement 9.4・9.5）**: ワークスペース全体。Windows だけの働きのテストは理由を書いて囲う。直しが膨らんだら brief の分割の案（`pasta-portable-build`）を使う。
- **試験用のホスト（Requirement 10.3）**: `OnBoot` を送る。Windows は x64 だけ（x64 のホストは x86 の `pasta.dll` を読めない）。
- **利用者向けのマニュアル（Requirement 11.4）**: 書き足さない。説明は内部の章だけ。
