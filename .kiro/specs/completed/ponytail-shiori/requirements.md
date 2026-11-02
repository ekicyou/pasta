# Requirements Document

## Project Description (Input)
ゴーストの利用者と作者が、pasta.dll の SHIORI の層（`crates/pasta_shiori`）の端で困っている（2026-10-19 の `/ponytail:ponytail-audit` の指摘より）。ホストが `unload` を呼ばずに pasta.dll を外すと、SSP が 5 秒止まったうえで落ちうる。旧来の入口でゴーストのフォルダが見つからないと、理由の無い 500（`Not initialized error`）になる。コロンの後に何も無い見出しの行（`Reference0:`）を含む事象は、要求が丸ごと解けずに黙って 204 になり、Lua に届かない（SSP の記録 `doc/shiori-sample.log` に実例が 3 つある）。加えて、テストだけが使う道具とテストデータ（約 3,900 行）と、使われない部品・素通しの抽象が、出荷物と crates.io のパッケージに入っている。`crates/pasta_shiori` の監査の指摘を残らず直し、`unload` を呼ばずに外されてもホストを落とさず上限を超えて止めず、フォルダが無ければその名前を 500 の理由に出し、`Key:` の行は値 `""` として受け付け、解けない要求は警告のログを残し、テストだけのものを出荷物とパッケージから外すようにする。`shiori4-v3-portable`（PR #206）で閉じた指摘（`unload` の道の待ち・FFI の型・テストの錠・`marshal_request` の写しの一部・`actor_detach_on_poison`）は、今の振る舞いを保つ。

## Introduction

この spec は、2026-10-19 の監査が `crates/pasta_shiori` で見つけた指摘 1〜12 と、ギャップ分析（research.md）で同じ領域に増えた指摘を、残らず直す。直す中身は 2 種類ある。

- **挙動を変える直し**：`unload` を呼ばずに外されたとき（`DllMain` の道）・フォルダが無いときの 500 の理由・`Key:` の行・解けない要求の警告・204 の応答の写し。
- **挙動を変えない撤去**：テストだけの道具とデータ、使われない部品、実装が 1 つの抽象、素通しの引数、重なった判定。

要件は「その領域の指摘が残らず直っている」ことと、挙動を変える直しの決めごとである。どう直すか（関数・ファイル・型の形）は設計で決める。この spec の対象はコードの指摘そのものなので、受け入れの基準に Rust の項目の名前やファイルの名前が出るのは意図したものである（直し方は書かない）。

監査の番号と要件の対応は次のとおり。

| 監査の番号 | 今の状態（2026-11-02・main ae04a28b） | 要件 |
| ---------- | ------------------------------------- | ---- |
| 1 unload と FreeLibrary の競り | `unload` の道は閉じた。`DllMain` の道が残る | 1 |
| 2 フォルダが無いと理由の無い 500 | 残る | 2 |
| 3 `Key:` の行で黙って 204 | 残る | 3 |
| 4 FFI の型 | 閉じた | 4（保つ） |
| 5 テストの競り | 閉じた | 4（保つ） |
| 6 コードページの層 | 残る | 6 |
| 7 テストの支えが DLL に入っている | 残る（増えた分あり） | 6 |
| 8 ほぼ同じ写し | `marshal_request` の写しは半分残る、204 の写しは残る | 5・6 |
| 9 出力の引数の二重の受け渡し | 閉じた | 4（保つ） |
| 10 素通しと未使用 | `actor_detach_on_poison` は閉じた。残り 5 つが残る | 7 |
| 11 実装が 1 つの `Shiori` トレイト | 残る | 7 |
| 12 テストだけのパーサの写し・未使用のテストデータ・`exclude` | 残る | 6 |
| ギャップ分析で増えた指摘（research.md の N1〜N11） | 残る | 3・6・7 |

> brief.md だけでは決めきれなかった論点（research.md の「要件ディスカッションへの持ち越し」の Q1〜Q9）は、2026-11-02 の要件のディスカッションで、開発者が全議題を推奨案で確定した。本文の「（議題 N で確定）」がその印で、決めたことは末尾の「要件のディスカッションで決めたこと」にまとめた。

## Boundary Context

- **In scope**
  - 監査の 1〜12 番のうち残るもの、と閉じたものの振る舞いを保つこと。
  - ギャップ分析で増えた、同じ領域（`crates/pasta_shiori`）の指摘。
  - 挙動を変える直しと撤去に合わせた、マニュアル `book/src/internals/shiori.md` の該当の節と `crates/pasta_shiori/README.md`。
  - `crates/pasta_shiori/Cargo.toml`（パッケージに入れるもの・依存の機能）。
- **Out of scope**
  - `panic = "abort"` の方針と静的な CRT リンク（確定済み）。
  - `pasta_lua` の文字コード（`ponytail-lua-loader` で完了）と、Lua の C モジュールの読み込みの道（`lua-cmodule-robustness` で完了）。
  - 版 3 の口の契約（`shiori4-v3-portable` の要件）。版 3 の `load` がフォルダの無いときに NULL を返すこと、読めない要求に 400 を返すことは変えない。
  - pub API の断り書き（ルートの `README.md` 88 行と `tech.md` に、`ponytail-lua-loader` が書き済み）。
  - SSP での実機の再現（任意。できれば `ssp-ghost-test-launch` の手順で確かめる）。
- **Adjacent expectations**
  - SSP（ベースウェア）は、`unload` を呼んでから `FreeLibrary` する。この spec が `unload` の道で保つ振る舞いは、その順に呼ぶホストのためのものである。
  - `pasta_lua` の実行時は、VM の片付け（Lua の C モジュールの解放を含む）をアクタースレッドの上で行う。この spec は、その片付けの中身を変えない。
  - `ponytail-lua-scripts` は `book/src/internals/shiori.md` の別の節（Lua 側）を触る。後から入る側が rebase で合わせる。
  - `shiori4-v4-host`（`actor/`・`req_parser.pest`・`req.rs`・`error.rs`・`marshaling.rs`）と `pasta-dll-multiarch`（`windows.rs`）は、同じファイルを触るので同じウェーブに置かない。

## Requirements

### Requirement 1: unload を呼ばずに外されてもホストを落とさない

**Objective:** As a ゴーストの利用者, I want ゴーストを閉じたり切り替えたりしたときに SSP が止まったり落ちたりしないでほしい, so that ホストがどの順で pasta.dll を外しても、デスクトップの常駐が巻き添えにならない。

#### Acceptance Criteria

1. When ホストが `unload` を呼んだ, the pasta.dll shall アクタースレッドが pasta.dll のコードを出てから `unload` を返す。後始末の完了の通知を待つのは上限（5 秒）までで、上限を過ぎたらスレッドを手放して警告をログに出し、`unload` を返す（`shiori4-v3-portable` で入った今の振る舞いを保つ）。
2. If ホストが `unload` を呼ばずに `FreeLibrary` で pasta.dll を外した, then the pasta.dll shall ホストのプロセスを落とさない。pasta.dll が外れた後に pasta.dll のコードを走らせるスレッドを残さない。そのために、アクタースレッドが終わるまで pasta.dll がメモリに残ってもよい（議題 1 で確定）。
3. If ホストが `unload` を呼ばずに `FreeLibrary` で pasta.dll を外した, then the pasta.dll shall その `FreeLibrary` を上限（5 秒）を超えて止めない。辞書が Lua の C モジュールを読んでいる場合も同じである（今は C モジュールの解放がローダーのロックを待って 5 秒止まり、その後に生きたスレッドの下で pasta.dll が外れうる）。
4. If ホストが `unload` を呼ばずに `FreeLibrary` で pasta.dll を外した, then the pasta.dll shall 2・3 を `SHIORI.unload` と永続化データの保存より優先する（その道では `SHIORI.unload` と保存が走らなくてよい。議題 1 で確定）。
5. The pasta.dll shall 2・3 を、旧来の入口のインスタンスと版 3 のインスタンスのどちらにも満たす（議題 1 で確定）。
6. When プロセスの終了で pasta.dll が外れる, the pasta.dll shall 今と同じく何もしない（後始末もログも行わない）。
7. The pasta.dll shall `DllMain` でスレッドを起こさない（今を保つ）。
8. While ホストが `unload` を呼んでから pasta.dll を外す, the pasta.dll shall `SHIORI.unload`・永続化データの保存・終了のログを今と同じに行う。

### Requirement 2: フォルダが無いときは、その名前を 500 の理由に出す

**Objective:** As a ゴーストの作者, I want 旧来の入口でゴーストのフォルダが見つからないときに、どのフォルダを探したのかが分かってほしい, so that 設置パスの文字が ANSI で表せずに `?` に化けた、などの原因を自分で辿れる。

#### Acceptance Criteria

1. If 旧来の入口（`load`・`loadu`）に渡された設置パスのフォルダが存在しない, then the pasta.dll shall その入口に FALSE を返す（今と同じ）。
2. While 1 のとおりフォルダが見つからずに読み込みが失敗している, when GET の要求が来た, the pasta.dll shall 500 の応答を返し、`X-ERROR-REASON` に、フォルダが見つからないことと、探したパス（入口がデコードした形）を 1 行で含める。
3. While 1 のとおりフォルダが見つからずに読み込みが失敗している, when NOTIFY の要求が来た, the pasta.dll shall 今と同じく 204 を返す。
4. When `load`（ANSI）に、システムの ANSI コードページで表せない文字を含む設置パスが渡され、デコードの結果のフォルダが存在しない, the pasta.dll shall 2 の理由に、デコードの結果のパス（表せない文字が置き換わった形）を出す。
5. The pasta.dll shall 版 3 の `load` がフォルダの無いときに NULL を返す振る舞いを変えない。
6. The pasta.dll shall フォルダが存在する場合の読み込みの失敗の理由（`Load error: …`）を変えない。

### Requirement 3: `Key:` の行を受け付け、解けない要求は警告のログを残す

**Objective:** As a ゴーストの作者, I want 値の空いた見出しの行を含む事象も Lua に届いてほしく、届かない要求はログで分かってほしい, so that OnInitialize や OnNotifyUserInfo が黙って消えず、消えたときに原因を探せる。

#### Acceptance Criteria

1. When 要求に、コロンの直後で行が終わる見出しの行（例：`Reference0:`）がある, the pasta.dll shall その見出しを値 `""` として受け付け、要求を Lua の `SHIORI.request` に届ける（`req.reference[0]` と `req.dic["Reference0"]` は `""`）。
2. When 見出しの行がコロンと空白 1 つの後に値を持つ, the pasta.dll shall 今と同じ値を Lua に渡す。
3. When 見出しの行がコロンの直後に空白を置かずに値を持つ（例：`Reference0:abc`）, the pasta.dll shall その値（`abc`）として受け付ける（議題 2 で確定。1 の直しの帰結）。
4. When SSP の記録 `crates/pasta_shiori/doc/shiori-sample.log` にある、値の空いた見出しの行を含む 3 つの要求（47 行の `Reference0:` の OnInitialize、720 行の `Reference2:` の OnNotifyUserInfo、1236 行の `Reference1:` の GET OnTranslate）が旧来の入口に来た, the pasta.dll shall どれも Lua の `SHIORI.request` に届ける。この 3 つを確かめるテストは、要求を自分で文字列として持つ（`doc/` は要件 6.6 でパッケージから外れるので、テストから読まない）。
5. The pasta.dll shall 1〜3 の受け付けを、旧来の入口と版 3 の入口で同じにする。
6. If 旧来の入口に来た要求を解けない, then the pasta.dll shall 今と同じく 204 を返し、そのゴーストのログのファイルに警告を 1 行残す。
7. If 版 3 の入口に来た要求を解けない, then the pasta.dll shall 今と同じく 400 を返し、6 と同じ警告を 1 行残す（議題 3 で確定。今は debug。SHIORI/2.x の要求と、`notify` に来た GET を弾くログは今の debug のまま）。
8. When 6・7 の警告を残す, the pasta.dll shall 警告に、解けなかった位置（行と列）と、要求の 1 行目を含め、要求の全文は含めない（議題 3 で確定。要求は利用者の名前を含みうる）。

### Requirement 4: 閉じた指摘の振る舞いを保つ

**Objective:** As a pasta の開発者, I want `shiori4-v3-portable` で閉じた指摘（FFI の型・テストの競り・出力の受け渡し）が、この spec の直しで戻らないでほしい, so that 撤去や移動のついでに古い不具合を作り直さない。

#### Acceptance Criteria

1. The pasta.dll shall `load`・`loadu`・`unload` の戻り値を C の `BOOL`（32 ビットの int の 1／0）に、`load`・`loadu`・`request` の長さを C の `long` に保つ。
2. When `request` が応答を返す, the pasta.dll shall 返す長さを、応答の HGLOBAL の中身のバイト数と一致させる（今を保つ）。
3. When `cargo test -p pasta_shiori` を既定の並列の数で繰り返し走らせる, the pasta_shiori のテスト shall プロセスに 1 つの状態（旧来の入口の置き場・`loadu` の印）を取り合って落ちない。
4. The pasta_shiori のテスト shall プロセスに 1 つの状態に触る単体テストを、1 つの錠で直列にする（今を保つ）。

### Requirement 5: 自前で作る 204 の応答を 1 つにする

**Objective:** As a pasta の開発者, I want Rust の側で作る 204 の応答が 1 か所で決まってほしい, so that ヘッダの違う 204 が経路によって混ざらず、直すときに 1 か所で済む。

#### Acceptance Criteria

1. The pasta.dll shall Rust の側で作る 204 の応答（NOTIFY の即答・アクターが居ない・メソッドを判定できない・GET の応答が来ない・panic・UTF-8 として読めない・`SHIORI.request` が定義されていない）を、すべて同じバイト列（`Charset: UTF-8`・`Sender: Pasta`・`SecurityLevel: local`）にする。
2. When `SHIORI.request` が定義されていないゴーストに要求が来た, the pasta.dll shall 1 の 204 を返す（`SecurityLevel: local` が増える。brief で受け入れ済み）。
3. The pasta.dll shall 1 の 204 のバイト列を、Lua の `RES.no_content()` が既定の設定で作る 204 と同じに保つ。

### Requirement 6: テストだけのものを、出荷物と crates.io のパッケージから外す

**Objective:** As a pasta の開発者, I want テストだけが使う道具とデータが出荷するクレートのコードとパッケージに入っていないでほしい, so that 読み手が本番の道とテストの足場を取り違えず、パッケージが小さくなる。

#### Acceptance Criteria

1. The pasta_shiori の出荷のコード（テストのビルドでないときにコンパイルされるコード） shall 再読み込みの反復のリーク検査（`ReloadProbe`・`ReloadReport`・`LeakMetric`・OS の資源の数え上げ）と、それだけが使う後始末の道（`teardown::teardown_actor`・`TeardownReport::is_clean`）を含まない。
2. The pasta_shiori の出荷のコード shall テストだけが使う写しと窓口（`marshaling::marshal_request`、旧来の置き場へ kick を積む `lifecycle::kick_sink`・`kick_into`・`ActorInstance::kick`、`ActorMsg::get`・`notify`・`kick` の作り手、`spawn_actor_thread`）を含まない（議題 4 で確定。観測の値（スレッド ID・DAP の待受）の残し方は設計で決める）。
3. The pasta_shiori の出荷のコード shall ANSI の設置パスのデコードに要る 1 つの変換（システムの ANSI コードページ・不正な文字は誤り）だけを持ち、テストだけが使う変換（UTF-8 から ANSI への変換・OEM のコードページ）と、実装が 1 つの変換のトレイトを含まない。
4. The pasta_shiori の出荷のコード shall 初期化していない領域を長さだけ伸ばして使う書き方（`Vec::set_len`）を使わない。
5. When `load`（ANSI）に設置パスが渡された, the pasta.dll shall 今と同じ結果（同じパス・同じ誤り）にデコードする。
6. The crates.io の `pasta_shiori` のパッケージ shall `doc/`（SSP の記録）と、どのテストも使わないテストデータ（`src/util/parsers/test_data/logfile.txt`）を含まない。
7. The pasta_shiori の単体テスト shall テストの中にパーサの写し（`src/util/parsers/req.rs` の `ShioriRequest`）を持たず、出荷のパーサで同じ性質を確かめる。
8. The pasta_shiori の通常の依存 shall テストの道具だけが使う OS の API の機能（`windows-sys` の `Win32_System_Threading`）を有効にしない。
9. When 1・2 で移したり外したりした道具を使っていたテストを走らせる, the pasta_shiori のテスト shall 同じ性質（再読み込みを繰り返しても OS のハンドルが増えない、kick がシーンの最初の拍まで届く、など）を確かめ続ける。kick のテストは、本番で kick が通る道を通す。
10. The pasta_shiori shall テストのファイル（`tests/`・`src/*_tests.rs`）と、試験から短い上限を差し込むための 1 行の口（`marshal_get_with_timeout`・`shutdown_with_timeout`・`spawn_actor_with_teardown_timeout`）を今のまま残す（議題 4 で確定。crates.io のパッケージから外すのは 6.6 の `doc/` とテストデータだけ）。

### Requirement 7: 使われない部品と素通しを消す

**Objective:** As a pasta の開発者, I want 使われない部品・実装が 1 つの抽象・いつも同じ値の引数・重なった判定が無くなってほしい, so that 読む・試す・直すコードが減る。

#### Acceptance Criteria

1. The pasta_shiori shall 次を含まない：variant が 1 つだけの `Reply`、`_GRAMMAR` の `include_str!`、文法の使われない規則（`sp`・`eol`・`key_sep`）、`ShioriString` の `unsafe impl Send`・`unsafe impl Sync`、`MyError::Poison` と `From<PoisonError>`、実装が 1 つの `Shiori` トレイト。
2. When `Shiori` トレイトを消した, the `PastaShiori` shall `load`・`request` を同じ形で呼べる（`use pasta::Shiori` が要らなくなるだけ）。
3. The pasta_shiori shall アクターを起こす道（`spawn_actor` から `PastaShiori::load` まで）に、いつも 0 の `hinst` を引数として通さず、`PastaShiori` に読まれない `hinst` を持たない。Lua の `SHIORI.load(hinst, load_dir)` には今と同じく 0 を渡す（議題 5 で確定。`PastaShiori::load` の引数に残すかは設計で決める）。
4. The pasta_shiori shall 要求のメソッドの判定を 1 つにする（今は `determine_method` と `classify_request` の 2 つ）。旧来の入口は SHIORI/2.x の要求を今と同じく Lua に届け、版 3 の入口は今と同じく 400 にする。
5. The pasta_shiori shall 文法の上で起きない分岐（SHIORI/2.x の版の数の 0 未満と 9 超え）を持たない。
6. When 1〜5 を消した, the pasta.dll shall ホストへ返す応答のバイト列を変えない（`byte_invariant_test` などの特性化テストが通る）。
7. The pasta_shiori shall 公開の API を消してよい（2026-10-19 開発者の決定）。消したものは `crates/pasta_shiori/README.md` の公開 API の節から外す。
8. The pasta_shiori shall 撤去と食い違うコメント（`MyError` に variant を足すと semver-major になる、など）と、同じ前提を 2 度確かめる所（`PastaShiori::request` と `call_lua_request` のランタイムの有無の確認）を残さない（research.md の N11）。

### Requirement 8: マニュアルと README を直しに合わせる

**Objective:** As a pasta の開発者と作者, I want 内部設計の章と README が、直した後の振る舞いと部品を正しく書いていてほしい, so that 読んだとおりに動き、消えた部品を探さない。

#### Acceptance Criteria

1. The `book/src/internals/shiori.md` shall 要件 1〜7 で変わった振る舞いと部品を、今の形で書く：`DllMain` の道の後始末、フォルダが無いときの 500 の理由、`Key:` の行、解けない要求の警告、自前の 204 の表（`default_204_response` の行）、`ReloadProbe` の行、`hinst`、メソッドの判定。
2. The `book/src/internals/shiori.md` shall この spec が変えない節（Lua 側の SHIORI エントリなど、`ponytail-lua-scripts` が触る節）を書き換えない。
3. The `crates/pasta_shiori/README.md` shall `Shiori` トレイトの節と使用例を、トレイトの無い形に直し、公開 API の一覧から消したものを外す。
4. The pasta_shiori shall 新しい pub API の断り書きを足さない（ルートの `README.md` と `tech.md` に書き済み）。

### Requirement 9: 品質の関門

**Objective:** As a pasta の開発者, I want 直しのたびに同じ関門を通してほしい, so that 撤去や挙動の直しで既存の振る舞いが壊れたことにすぐ気づける。

#### Acceptance Criteria

1. When 直しを終えた, the pasta_shiori shall `cargo fmt --check` と clippy（警告 0）を通す。
2. When 直しを終えた, the pasta_shiori shall `cargo test -p pasta_shiori` を通す（cargo の前に環境変数 `NoDefaultCurrentDirectoryInExePath` を外す）。
3. When 直しを終えた, the pasta.dll shall i686（配布する 32 ビット版）のビルドを通す。
4. When 挙動を変える直し（要件 1・2・3・5）を入れる, the pasta_shiori shall 直す前の振る舞いを固めるテストを先に置き、直した後の振る舞いを確かめるテストに改める。

## 要件のディスカッションで決めたこと（2026-11-02）

開発者が、議題 1〜5 を推奨案のまま一括で承認した。

1. **`unload` を呼ばずに外されたとき**（Q1・Q2・Q9 → 要件 1.2〜1.5）：ホストを落とさないこと・`FreeLibrary` を 5 秒を超えて止めないことを最優先にする。その道では `SHIORI.unload` と保存を約束しない。pasta.dll とアクター・VM が、プロセスの終わりか次の `load` まで残ってもよい（版 3 のインスタンスは、手放されたものと生きたものを見分けられないので、プロセスの終わりまで残り、同じフォルダの版 3 の `load` は NULL になる。2026-11-02 の設計のディスカッションで受け入れた）。旧来の入口と版 3 のインスタンスの両方に効かせる。DLL をピン留めする案（D3）は、`unload` を呼ぶ SSP の振る舞いまで変えるので採らない。
2. **`Key:value` を受け付ける**（Q3 → 要件 3.3）：文法の直し（`":" ~ " "?`）の帰結として受け付ける。
3. **解けない要求の警告**（Q4・Q5 → 要件 3.6〜3.8）：旧来の入口と版 3 の入口の両方で warn にする。中身は、解けなかった位置（行と列）と要求の 1 行目だけで、全文は載せない。
4. **テストだけのものの外し方**（Q6・Q7 → 要件 6.2・6.10）：出荷のコードから、テストだけの窓口・写し・作り手・包みを外す。kick のテストは本番の kick の道へ寄せる。`tests/`・`src/*_tests.rs`・短い上限の口は残し、crates.io のパッケージからは `doc/` と使われないテストデータだけを外す（リポジトリのクレートに `exclude` を使う前例は無く、`tests/` を入れるのは Rust の慣習）。
5. **`hinst`**（Q8 → 要件 7.3）：アクターの道から消す。Lua の `SHIORI.load(hinst, load_dir)` には 0 を渡し続ける（作者に見える約束は変えない）。
